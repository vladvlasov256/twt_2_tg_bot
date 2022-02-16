use std::env;
use std::string::String;
use std::{fmt::Debug, sync::Arc};

use teloxide::dispatching::update_listeners::UpdateListener;
use teloxide::prelude::*;
use teloxide::adaptors::DefaultParseMode;
use teloxide::types::*;
use egg_mode::*;
use tokio_stream::wrappers::UnboundedReceiverStream;

use bot::bot_errors::BotErrorKind;
use bot::webhook::webhook;
use bot::twitter_utils::twitter_api_token_value;
use bot::update_processor::UpdateProcessor;
use bot::text_message_processor::TextMessageProcessor;
use bot::inline_query_processor::InlineQueryProcessor;
use bot::analytics::track_hit;

#[tokio::main]
async fn main() {
    run().await.log_on_error().await;
}

async fn run() -> Result<(), BotErrorKind> {
    teloxide::enable_logging!();
    log::info!("Starting Twitter to Telegram Converter...");

    let twitter_client_id = env::var("TWITTER_CLIENT_ID").expect("TWITTER_CLIENT_ID not set");
    let twitter_secret = env::var("TWITTER_SECRET").expect("TWITTER_SECRET not set");
    let twitter_token = twitter_api_token_value(twitter_client_id, twitter_secret).await?;

    let bot = Bot::from_env().parse_mode(ParseMode::MarkdownV2).auto_send();
    let listener = webhook(bot.clone()).await;
    reply_with_listener(bot, listener, twitter_token).await;

    Ok(())
}

async fn reply_with_listener<'a, L, ListenerE>(
    bot: AutoSend<DefaultParseMode<Bot>>,
    listener: L,
    token: Token
) where L: UpdateListener<ListenerE> + Send + 'a, ListenerE: Debug {
    let token = Arc::new(token);
    let token_copy = token.clone();
    Dispatcher::new(bot)
        .messages_handler(move |rx| {
            UnboundedReceiverStream::new(rx)
                .for_each_concurrent(None, move |message| {
                    let token = Arc::clone(&token);
                    async move {
                        process_message(message, &*token).await.log_on_error().await;
                    }
                })
        })
        .inline_queries_handler(move |rx| {
            UnboundedReceiverStream::new(rx)
                .for_each_concurrent(None, move |query| {
                    let token = Arc::clone(&token_copy);
                    async move {
                        process_inline_query(query, &*token).await.log_on_error().await;
                    }
                })
        })
        .setup_ctrlc_handler()
        .dispatch_with_listener(
            listener,
            LoggingErrorHandler::with_custom_text("An error from the update listener"),
        )
        .await;
}

async fn process_message(cx: UpdateWithCx<AutoSend<DefaultParseMode<Bot>>, Message>, token: &Token) -> Result<(), BotErrorKind> {
    track_hit(String::from("message")).await?;
    match message_text(&cx) {
        Some(text) => {
            let processor = TextMessageProcessor {  message: cx, text: text };
            return processor.process(token).await;
        },
        _ => Ok(())
    }
}

fn message_text(cx: &UpdateWithCx<AutoSend<DefaultParseMode<Bot>>, Message>) -> Option<String> {
    match cx.update.kind {
        MessageKind::Common(ref message) => {
            match &message.media_kind {
                MediaKind::Text(text) => Some(text.text.clone()),
                _ => None
            }
        },
        _ => None
    }
}

async fn process_inline_query(cx: UpdateWithCx<AutoSend<DefaultParseMode<Bot>>, InlineQuery>, token: &Token) -> Result<(), BotErrorKind> {
    track_hit(String::from("inline")).await?;
    let processor = InlineQueryProcessor { query: cx };
    return processor.process(token).await;
}
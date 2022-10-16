use std::env;
use std::string::String;
use std::{sync::Arc};

use bot::callback_query_processor::CallbackQueryProcessor;
use teloxide::{prelude::*};
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

    let is_webhooks_enabled = env::var("WEBHOOKS_ENABLED").expect("WEBHOOKS_ENABLED not set");

    let bot = Bot::from_env().parse_mode(ParseMode::MarkdownV2).auto_send();

    if is_webhooks_enabled.to_lowercase() == String::from("true") {
        dispatcher(bot.clone(), twitter_token)
        .dispatch_with_listener(
            webhook(bot).await,
            LoggingErrorHandler::with_custom_text("An error from the update listener"),
        )
        .await;
    } else {
        dispatcher(bot.clone(), twitter_token)
        .dispatch()
        .await;
    }

    Ok(())
}

fn dispatcher(bot: AutoSend<DefaultParseMode<Bot>>, token: Token) -> Dispatcher<AutoSend<DefaultParseMode<Bot>>> {
    let token = Arc::new(token);
    let inline_token = token.clone();
    let callback_token = token.clone();
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
                    let token = Arc::clone(&inline_token);
                    async move {
                        process_inline_query(query, &*token).await.log_on_error().await;
                    }
                })
        })
        .callback_queries_handler(move |rx| {
            UnboundedReceiverStream::new(rx)
                .for_each_concurrent(None, move |query| {
                    let token = Arc::clone(&callback_token);
                    async move {
                        process_callback_query(query, &*token).await.log_on_error().await;
                    }
                })
        })
        .setup_ctrlc_handler()
}

async fn process_message(cx: UpdateWithCx<AutoSend<DefaultParseMode<Bot>>, Message>, token: &Token) -> Result<(), BotErrorKind> {
    log::info!("Received a message");
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
    log::info!("Received an inline query");
    track_hit(String::from("inline")).await?;
    let processor = InlineQueryProcessor { query: cx };
    return processor.process(token).await;
}

async fn process_callback_query(cx: UpdateWithCx<AutoSend<DefaultParseMode<Bot>>, CallbackQuery>, token: &Token) -> Result<(), BotErrorKind> {
    log::info!("Received a callback query");
    let processor = CallbackQueryProcessor { query: cx };
    return processor.process(token).await;
}
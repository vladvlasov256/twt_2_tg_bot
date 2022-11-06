use std::env;
use std::error::Error;
use std::string::String;

use bot::callback_query_processor::CallbackQueryProcessor;
use teloxide::dispatching::DefaultKey;
use teloxide::{prelude::*, RequestError};
use teloxide::types::*;
use egg_mode::*;

use bot::bot_errors::{BotError};
use bot::webhook::webhook;
use bot::twitter_utils::twitter_api_token_value;
use bot::update_processor::UpdateProcessor;
use bot::text_message_processor::TextMessageProcessor;
use bot::inline_query_processor::InlineQueryProcessor;
use bot::analytics::track_hit;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting Twitter to Telegram Converter...");

    let is_webhooks_enabled = env::var("WEBHOOKS_ENABLED").expect("WEBHOOKS_ENABLED not set");

    let bot = Bot::from_env();

    if is_webhooks_enabled.to_lowercase() == String::from("true") {
        log::info!("Creating a webhook...");
        dispatcher(bot.clone())
        .dispatch_with_listener(
            webhook(bot).await,
            LoggingErrorHandler::with_custom_text("An error from the update listener"),
        )
        .await;
    } else {
        log::info!("Starting polling...");
        dispatcher(bot.clone())
        .dispatch()
        .await;
    }
}

fn dispatcher(bot: Bot) -> Dispatcher<Bot, RequestError, DefaultKey> {
    let handler = dptree::entry()
    .branch(Update::filter_message().endpoint(|bot: Bot, msg: Message, _me: Me| async {
        process_message(bot, msg).await.log_on_error().await;
        respond(())
    }))
    .branch(Update::filter_inline_query().endpoint(|bot: Bot, q: InlineQuery| async {
        process_inline_query(bot, q).await.log_on_error().await;
        respond(())
    }))
    .branch(Update::filter_callback_query().endpoint(|bot: Bot, q: CallbackQuery| async {
        process_callback_query(bot, q).await.log_on_error().await;
        respond(())
    }));

    Dispatcher::builder(bot, handler)
    .enable_ctrlc_handler()
    .build()
}


async fn process_message(bot: Bot, message: Message) -> Result<(), Box<dyn Error + Send + Sync>> {
    log::info!("Received a message");
    track_hit(String::from("message")).await?;
    let token = twitter_token_data().await?;
    match message_text(&message) {
        Some(text) => {
            let processor = TextMessageProcessor {  message: message, text: text };
            return Ok(processor.process(bot, &token).await?);
        },
        _ => Ok(())
    }
}

fn message_text(message: &Message) -> Option<String> {
    match message.kind {
        MessageKind::Common(ref message) => {
            match &message.media_kind {
                MediaKind::Text(text) => Some(text.text.clone()),
                _ => None
            }
        },
        _ => None
    }
}

async fn process_inline_query(bot: Bot, query: InlineQuery) -> Result<(), Box<dyn Error + Send + Sync>> {
    log::info!("Received an inline query");
    track_hit(String::from("inline")).await?;
    let token = twitter_token_data().await?;
    let processor = InlineQueryProcessor { query: query };
    return Ok(processor.process(bot, &token).await?);
}

async fn process_callback_query(bot: Bot, query: CallbackQuery) -> Result<(), Box<dyn Error + Send + Sync>> {
    log::info!("Received a callback query");
    track_hit(String::from("callback")).await?;
    let token = twitter_token_data().await?;
    let processor = CallbackQueryProcessor { query: query };
    return Ok(processor.process(bot, &token).await?);
}

async fn twitter_token_data() -> Result<Token, BotError> {
    let twitter_client_id = env::var("TWITTER_CLIENT_ID").expect("TWITTER_CLIENT_ID not set");
    let twitter_secret = env::var("TWITTER_SECRET").expect("TWITTER_SECRET not set");
    return twitter_api_token_value(twitter_client_id, twitter_secret).await;
}
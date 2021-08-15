use std::env;
use std::string::String;
use std::boxed::Box;
use std::{convert::Infallible, net::SocketAddr, fmt::Debug, future::Future, sync::Arc};
// use std::{convert::Infallible, env, net::SocketAddr};

use teloxide::{dispatching::{update_listeners::{self, StatefulListener}, stop_token::AsyncStopToken}, prelude::*, types::Update};
use teloxide::adaptors::DefaultParseMode;
use teloxide::{prelude::*, types::*};
use egg_mode::*;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio::sync::mpsc;
use warp::Filter;
use reqwest::{StatusCode, Url};

use bot::update_processor::UpdateProcessor;
use bot::text_message_processor::TextMessageProcessor;
use bot::inline_query_processor::InlineQueryProcessor;
use bot::bot_errors::{BotError, BotErrorKind};

static mut TOKEN_VALUE: Option<String> = None;

#[tokio::main]
async fn main() {
    match run().await {
        Ok(_) => {},
        Err(error) => log::error!("Error on start: {}", BotError::from(error))
    }
}

async fn handle_rejection(error: warp::Rejection) -> Result<impl warp::Reply, Infallible> {
    log::error!("Cannot process the request due to: {:?}", error);
    Ok(StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn webhook(bot: AutoSend<DefaultParseMode<Bot>>) -> impl update_listeners::UpdateListener<Infallible> {
    // Heroku auto defines a port value
    let teloxide_token = env::var("TELOXIDE_TOKEN").expect("TELOXIDE_TOKEN env variable missing");
    let port: u16 = env::var("PORT")
        .expect("PORT env variable missing")
        .parse()
        .expect("PORT value to be integer");
    // Heroku host example .: "heroku-ping-pong-bot.herokuapp.com"
    let host = env::var("HOST").expect("have HOST env variable");
    let path = format!("bot{}", teloxide_token);
    let url = Url::parse(&format!("https://{}/{}", host, path)).unwrap();

    bot.set_webhook(url).await.expect("Cannot setup a webhook");

    let (tx, rx) = mpsc::unbounded_channel();

    let server = warp::post()
        .and(warp::path(path))
        .and(warp::body::json())
        .map(move |json: serde_json::Value| {
            if let Ok(update) = Update::try_parse(&json) {
                tx.send(Ok(update)).expect("Cannot send an incoming update from the webhook")
            }

            StatusCode::OK
        })
        .recover(handle_rejection);

    let (stop_token, stop_flag) = AsyncStopToken::new_pair();

    let addr = format!("0.0.0.0:{}", port).parse::<SocketAddr>().unwrap();
    let server = warp::serve(server);
    let (_addr, fut) = server.bind_with_graceful_shutdown(addr, stop_flag);

    // You might want to use serve.key_path/serve.cert_path methods here to
    // setup a self-signed TLS certificate.

    tokio::spawn(fut);
    let stream = UnboundedReceiverStream::new(rx);

    fn streamf<S, T>(state: &mut (S, T)) -> &mut S { &mut state.0 }
    
    StatefulListener::new((stream, stop_token), streamf, |state: &mut (_, AsyncStopToken)| state.1.clone())
}

// !!! println -> logs

// async fn run() -> Result<(), BotErrorKind> {
//     teloxide::enable_logging!();
//     log::info!("Starting bot...");

//     let bot = Bot::from_env().parse_mode(ParseMode::MarkdownV2).auto_send();
//     set_twitter_api_token_value().await?;

//     Dispatcher::new(bot.clone())
//         .messages_handler(|rx: DispatcherHandlerRx<AutoSend<DefaultParseMode<Bot>>, Message>| {
//             return UnboundedReceiverStream::new(rx).for_each_concurrent(None, |message| async move {
//                 match process_message(message).await {
//                     Ok(_) => {},
//                     Err(error) => log::error!("Error during processing: {}", BotError::from(error))
//                 }
//             })
//         })
//         .inline_queries_handler(|rx: DispatcherHandlerRx<AutoSend<DefaultParseMode<Bot>>, InlineQuery>| {
//             UnboundedReceiverStream::new(rx).for_each_concurrent(None, |query| async move {
//                 match process_inline_query(query).await {
//                     Ok(_) => {},
//                     Err(error) => log::error!("Error during processing: {}", BotError::from(error))
//                 }
//             })
//         })
//         .dispatch()
//         .await;
        
//     Ok(())
// }

async fn run() -> Result<(), BotErrorKind> {
    teloxide::enable_logging!();
    log::info!("Starting heroku_ping_pong_bot...");

    match set_twitter_api_token_value().await {
        Ok(_) => {},
        Err(error) => log::error!("Error during twitter token processing: {}", BotError::from(error))
    }

    let bot = Bot::from_env().parse_mode(ParseMode::MarkdownV2).auto_send();
    let cloned_bot = bot.clone();
    // teloxide::repl_with_listener(
    //     bot,
    //     |message| async move {
    //         match process_message(message).await {
    //             Ok(_) => {},
    //             Err(error) => log::error!("Error during message processing: {}", BotError::from(error))
    //         }
    //         respond(())
    //     },
    //     webhook(cloned_bot).await,
    // )
    // .await;
    let message_handler = Arc::new(
        |message| async move {
            match process_message(message).await {
                Ok(_) => {},
                Err(error) => log::error!("Error during message processing: {}", BotError::from(error))
            }
            respond(())
        },
    );
    
    let inline_query_handler = Arc::new(
        |query| async move {
            match process_inline_query(query).await {
                Ok(_) => {},
                Err(error) => log::error!("Error during processing: {}", BotError::from(error))
            }
            respond(())
        },
    );

    Dispatcher::new(bot)
        .messages_handler(|rx| {
            UnboundedReceiverStream::new(rx)
                .for_each_concurrent(None, move |message| {
                    let handler = Arc::clone(&message_handler);
                    async move {
                        handler(message).await.log_on_error().await;
                    }
                })
        })
        .inline_queries_handler(|rx| {
            UnboundedReceiverStream::new(rx)
                .for_each_concurrent(None, move |query| {
                    let handler = Arc::clone(&inline_query_handler);
                    async move {
                        handler(query).await.log_on_error().await;
                    }
                })
        })
        .setup_ctrlc_handler()
        .dispatch_with_listener(
            webhook(cloned_bot).await,
            LoggingErrorHandler::with_custom_text("An error from the update listener"),
        )
        .await;

    Ok(())
}

// !!! prelude method

async fn process_message(cx: UpdateWithCx<AutoSend<DefaultParseMode<Bot>>, Message>) -> Result<(), BotErrorKind> {
    match message_processor(cx) {
        Some(processor) => {
            unsafe {
                let token = Token::Bearer(String::from(TOKEN_VALUE.clone().unwrap_or_default()));    
                match processor.process(token).await {
                    Ok(_) => {},
                    Err(error) => println!("Error during processing: {}", BotError::from(error))
                }
            }
        },
        _ => {}
    }
    Ok(())
}

fn message_processor(cx: UpdateWithCx<AutoSend<DefaultParseMode<Bot>>, Message>) -> Option<Box<dyn UpdateProcessor>> {
    match message_text(&cx) {
        Some(text) => {
            return Some(Box::new(TextMessageProcessor { 
                message: cx,
                text: text
            }));
        },
        _ => None
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

async fn process_inline_query(cx: UpdateWithCx<AutoSend<DefaultParseMode<Bot>>, InlineQuery>) -> Result<(), BotErrorKind> {
    let processor = InlineQueryProcessor { query: cx };
    unsafe {
        let token = Token::Bearer(String::from(TOKEN_VALUE.clone().unwrap_or_default()));    
        match processor.process(token).await {
            Ok(_) => {},
            Err(error) => println!("Error during processing: {}", BotError::from(error))
        }
    }
    Ok(())
}

async fn set_twitter_api_token_value() -> Result<(), BotErrorKind> {
    let twitter_client_id = env::var("TWITTER_CLIENT_ID").expect("TWITTER_CLIENT_ID not set");
    let twitter_secret = env::var("TWITTER_SECRET").expect("TWITTER_SECRET not set");
    let con_token = KeyPair::new(twitter_client_id, twitter_secret);
    let twitter_token = auth::bearer_token(&con_token).await?;

    unsafe {
        TOKEN_VALUE = twitter_token_value(twitter_token);
    }

    Ok(())
}

fn twitter_token_value(token: Token) -> Option<String> {
    if let Token::Bearer(token_str) = token {
        Some(token_str)
    } else {
        None
    }
}
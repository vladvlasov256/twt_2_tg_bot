use std::env;
use std::{convert::Infallible, net::SocketAddr};
use teloxide::{dispatching::update_listeners::{webhooks, UpdateListener}, prelude::*};

pub async fn webhook(bot: Bot) -> impl UpdateListener<Err = Infallible> {
    let teloxide_token = env::var("TELOXIDE_TOKEN").expect("TELOXIDE_TOKEN env variable missing");
    let port: u16 = env::var("PORT")
        .expect("PORT env variable missing")
        .parse()
        .expect("PORT value to be integer");
    let host = env::var("HOST").expect("have HOST env variable");
    let path = format!("bot{}", teloxide_token);
    let url = format!("https://{}/{}", host, path).parse().unwrap();
    let addr = format!("0.0.0.0:{}", port).parse::<SocketAddr>().unwrap();

    return webhooks::axum(bot.clone(), webhooks::Options::new(addr, url))
        .await
        .expect("Couldn't setup webhook");
}
use std::string::String;
use std::env;

use crate::bot_errors::{BotErrorKind};
use serde::{Serialize};

#[derive(Serialize, Debug)]
struct PageViewPayload {
    website: String,
    url: String,
    referrer: String,
    hostname: String,
    language: String,
    screen: String,
}

#[derive(Serialize, Debug)]
struct PageView {
    payload: PageViewPayload,
    r#type: String,
}

fn page_view(umami_id: String) -> PageView {
    PageView {
        payload: PageViewPayload {
            website: umami_id,
            url: String::from("/"),
            referrer: String::from(""),
            hostname: String::from("twt_2_tg_bot"),
            language: String::from("en-US"),
            screen: String::from("1920x1080")
        },
        r#type: String::from("pageview")
    }
}

pub async fn track_hit() -> Result<(), BotErrorKind> {
    let umami_url = env::var("UMAMI_URL").expect("UMAMI_URL not set");
    let umami_id = env::var("UMAMI_ID").expect("UMAMI_ID not set");
    let event = page_view(umami_id);

    let client = reqwest::Client::new();
    let url = format!("{}/api/collect", umami_url);
    let _res = client.post(&url)
        .json(&event)
        .send()
        .await?;
    
    Ok(())
}
use std::string::String;
use std::env;

use crate::bot_errors::{BotError};
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

fn page_view(umami_id: String, page: String) -> PageView {
    PageView {
        payload: PageViewPayload {
            website: umami_id,
            url: page,
            referrer: String::from(""),
            hostname: String::from("twt_2_tg_bot"),
            language: String::from("en-US"),
            screen: String::from("1920x1080")
        },
        r#type: String::from("pageview")
    }
}

pub async fn track_hit(page: String) -> Result<(), BotError> {
    let umami_url_result = env::var("UMAMI_URL");
    let umami_id_result = env::var("UMAMI_ID");

    match (umami_url_result, umami_id_result) {
        (Ok(umami_url), Ok(umami_id)) => {
            let event = page_view(umami_id, page);

            let client = reqwest::Client::new();
            let url = format!("{}/api/collect", umami_url);
            let _res = client.post(&url)
                .json(&event)
                .send()
                .await;
        },
        _ => ()
    }    
    
    Ok(())
}
use std::string::String;

use egg_mode::tweet::Tweet;
use egg_mode::entities::MediaEntity;
use regex::Regex;
use htmlescape::*;
use mime;
use reqwest::Url;

use crate::bot_errors::{BotError, BotErrorKind};

pub struct VideoEntity {
    pub id: String,
    pub url: Url,
    pub thumb_url: Url,
    pub mime_type: mime::Mime,
    pub width: i32,
    pub height: i32
}

pub struct ImageEntity {
    pub id: String,
    pub url: Url,
    pub width: i32,
    pub height: i32
}

pub enum ParsedMedia {
    Video(VideoEntity),
    Image(ImageEntity),
}

pub struct Reply {
    pub user_name: Option<String>,
    pub thumb_url: Option<Url>,
    pub text: String,
    pub media_entities: Vec<ParsedMedia>
}

pub fn tweet_id_from_link(text: &String) -> Result<u64, BotError> {
    tweet_id(text, r"twitter.com/\w+/status/(\d+)")
}

pub fn tweet_id(text: &String, re: &str) -> Result<u64, BotError> {
    let link_regex = Regex::new(re)?;
    for caps in link_regex.captures_iter(text) {
        if caps.len() != 2 {
            continue
        }
        
        match caps.get(1) {
            Some(group) => {
                match group.as_str().parse().ok() {
                    Some(value) => return Ok(value),
                    None => continue
                }
            },
            None => continue
        }
    }

    return Err(BotError::from(BotErrorKind::TweetParsingError));
}

pub async fn tweet_to_reply(tweet: &Tweet) -> Result<Reply, BotError> {
    let text = tweet_text_to_displayable_string(&tweet.text);
    if let Some(user) = tweet.user.as_ref() {
        let name = decode_html(&user.name)?;
        let thumb_url = Url::parse(user.profile_image_url_https.as_str())?;
        return Ok(Reply {
            user_name: Some(name),
            thumb_url: Some(thumb_url),
            text: text,
            media_entities: tweet_media(&tweet),
        });
    } else {
        return Ok(Reply { 
            user_name: None,
            thumb_url: None,
            text: text,
            media_entities: tweet_media(&tweet),
        });
    }
}

pub fn tweet_text_to_displayable_string(text: &str) -> String {
    let text = decode_html(&text).unwrap_or(String::from(""));
    if let Ok(trimmed_text) = trim_short_link(&text) {
        return trimmed_text;
    }
    text
}

pub fn trim_short_link(s: &String) -> Result<String, BotError> {
    let mut s = s.clone();
    let regex = Regex::new(r"\shttps://t\.co/[\w\./]+$")?;
    if let Some(mat) = regex.find_iter(&s.as_str()).last() {
        s.replace_range(mat.range(), "");
        return Ok(String::from(s));
    }
    Ok(s)
}

fn tweet_media(tweet: &Tweet) -> Vec<ParsedMedia> {
    let media_entities: &Vec<MediaEntity>;
    if let Some(entities) = &tweet.extended_entities {
        media_entities = &entities.media;
    } else {
        return vec![]
    }

    return media_entities.iter().filter_map(|entity| {
        if let Ok(thumb_url) = Url::parse(entity.media_url_https.clone().as_str()) {
            let id = format!("{}", entity.id);
            if let Some(info) = &entity.video_info {
                for variant in info.variants.iter().filter(|v| v.content_type == "video/mp4") {
                    if let Ok(url) = Url::parse(variant.url.as_str()) {
                        return Some(ParsedMedia::Video(VideoEntity {
                            id,
                            url: url,
                            thumb_url: thumb_url,
                            mime_type: variant.content_type.clone(),
                            width: entity.sizes.large.w,
                            height: entity.sizes.large.h
                        }));
                    }
                }
            }

            return Some(ParsedMedia::Image(ImageEntity {
                id,
                url: thumb_url,
                width: entity.sizes.large.w,
                height: entity.sizes.large.h
            }));
        }

        None        
    }).collect::<Vec<_>>();
}

pub trait ReplyData {
    fn user_name(&self) -> Option<String>;
    fn text(&self) -> String;
}

impl ReplyData for Reply {
    fn user_name(&self) -> Option<String> {
        return self.user_name.clone();
    }

    fn text(&self) -> String {
        return self.text.clone();
    }
}
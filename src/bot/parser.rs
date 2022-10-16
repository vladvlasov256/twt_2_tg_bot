use std::string::String;

use egg_mode::tweet::{Tweet};
use egg_mode::entities::{MediaEntity, VideoInfo, VideoVariant};
use regex::Regex;
use htmlescape::*;
use mime;

use crate::bot_errors::BotErrorKind;

pub struct VideoReply {
    pub text: String,
    pub user_name: Option<String>,
    pub url: String,
    pub mime_type: mime::Mime,
    pub thumb_url: String
}

pub struct TextReply {
    pub user_name: Option<String>,
    pub thumb_url: Option<String>,
    pub text: String
}

pub struct Image {
    pub id: String,
    pub url: String,
    pub width: i32,
    pub height: i32
}

pub struct ImageReply {
    pub user_name: Option<String>,
    pub thumb_url: Option<String>,
    pub text: String,
    pub images: Vec<Image>
}

pub enum Reply {
    Video(VideoReply),
    Text(TextReply),
    Image(ImageReply)
}

pub fn tweet_id_from_link(text: &String) -> Result<u64, BotErrorKind> {
    tweet_id(text, r"twitter.com/\w+/status/(\d+)")
}

pub fn tweet_id(text: &String, re: &str) -> Result<u64, BotErrorKind> {
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

    return Err(BotErrorKind::TweetParsingError);
}

pub async fn tweet_to_reply(tweet: &Tweet) -> Result<Reply, BotErrorKind> {
	let text_reply = tweet_to_text_reply(&tweet)?;

    let user_name: Option<String>;
    if let Some(reply_user_name) = text_reply.user_name.clone() {
        user_name = Some(reply_user_name);
    } else {
        user_name = None;
    }

    if let Some(video_variant_result) = tweet_video_variant(tweet) {
        let (video_variant, thumb_url) = video_variant_result;

        let video = VideoReply {
            text: text_reply.text,
            user_name: user_name,
            url: String::from(video_variant.url.as_str()),
            mime_type: video_variant.content_type.clone(),
            thumb_url: thumb_url
        };

        return Ok(Reply::Video(video));
    }

    if let Some(images) = tweet_images(tweet) {
        let image = ImageReply {
            user_name: user_name,
            thumb_url: text_reply.thumb_url,
            text: text_reply.text,            
            images: images
        };

        return Ok(Reply::Image(image));
    }

    Ok(Reply::Text(text_reply))
}

fn tweet_video_variant(tweet: &Tweet) -> Option<(&VideoVariant, String)> {
    let media_entities: &Vec<MediaEntity>;
    if let Some(entities) = &tweet.extended_entities {
        media_entities = &entities.media
    } else {
        return None
    }

    if media_entities.is_empty() {
        return None
    }

    let video_info: &VideoInfo;
    if let Some(info) = &media_entities[0].video_info {
        video_info = info
    } else {
        return None
    }

    for variant in &video_info.variants {
        if variant.content_type == "video/mp4" {
            return Some((variant, media_entities[0].media_url_https.clone()))
        }
    }

	return None
}

fn tweet_images(tweet: &Tweet) -> Option<Vec<Image>> {
    let media_entities: &Vec<MediaEntity>;
    if let Some(entities) = &tweet.extended_entities {
        media_entities = &entities.media
    } else {
        return None
    }

    if media_entities.is_empty() {
        return None
    }

    let urls = media_entities.iter().map(|entity| {
        Image {
            id: format!("{}", entity.id),
            url: entity.media_url_https.clone(),
            width: entity.sizes.large.w,
            height: entity.sizes.large.h
        }
    }).collect::<Vec<_>>();
    
    return Some(urls)
}

fn tweet_to_text_reply(tweet: &Tweet) -> Result<TextReply, BotErrorKind> {
    let text = decode_html(&tweet.text).unwrap_or(String::from(""));
    if let Some(user) = tweet.user.as_ref() {
        let name = decode_html(&user.name)?;
        return Ok(TextReply {
            user_name: Some(name),
            thumb_url: Some(user.profile_image_url_https.clone()),
            text: text
        });
    } else {
        return Ok(TextReply { 
            user_name: None,
            thumb_url: None,
            text: text
        });
    }
}

pub trait ReplyData {
    fn user_name(&self) -> Option<String>;
    fn text(&self) -> String;
}

impl ReplyData for VideoReply {
    fn user_name(&self) -> Option<String> {
        return self.user_name.clone();
    }

    fn text(&self) -> String {
        return self.text.clone();
    }
}

impl ReplyData for TextReply {
    fn user_name(&self) -> Option<String> {
        return self.user_name.clone();
    }

    fn text(&self) -> String {
        return self.text.clone();
    }
}

impl ReplyData for ImageReply {
    fn user_name(&self) -> Option<String> {
        return self.user_name.clone();
    }

    fn text(&self) -> String {
        return self.text.clone();
    }
}
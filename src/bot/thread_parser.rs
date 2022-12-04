use std::collections::HashMap;
use std::string::String;

use egg_mode::raw::ParamList;
use egg_mode::{raw, Response, Token};
use egg_mode::tweet::{Tweet, show};
use htmlescape::*;
use reqwest::Url;
use serde::{Serialize, Deserialize};

use crate::bot_errors::{BotError, BotErrorKind};
use crate::parser::{tweet_text_to_displayable_string, ParsedMedia, ImageEntity, VideoEntity, tweet_media};

pub struct ThreadEntity {
    pub text: String,
    pub media_entities: Vec<ParsedMedia>
}
pub struct ThreadReply {
    pub user_name: Option<String>,
    pub thumb_url: Option<Url>,
    pub entities: Vec<ThreadEntity>
}

pub async fn is_included_in_thread(tweet: &Tweet, token: &Token) -> Result<bool, BotError> {
    if let Some(thread_user) = tweet.user.as_ref() {
        if let Some(conversation_id) = conversation_id_from_tweet(tweet.id, &token).await? {
            let replies = replies_in_conversation(
                &conversation_id,
                &thread_user.screen_name,
                10, 
                false,
                None,
                &token
            ).await?;
            return Ok(replies.data.len() >= 2);
        }
    }

    Ok(false)
}

pub async fn tweet_to_thread(start_tweet: &Tweet, token: &Token) -> Result<ThreadReply, BotError> {
    if let Some(thread_user) = start_tweet.user.as_ref() {
        let start_id = start_tweet.id;
        if let Some(conversation_id) = conversation_id_from_tweet(start_id, &token).await? {            
            let response = all_replies_in_conversation(
                &conversation_id, 
                &thread_user.screen_name,
                100,
                true,
                &token
            ).await?;

            let mut entities = vec![];
            let first_reply_id = response.data.last().map(|t| { t.id.clone() }).unwrap();
            let first_reply = show(first_reply_id.parse().unwrap(), &token).await?;
            if let Some(head_id) = first_reply.in_reply_to_status_id {
                let head = show(head_id, &token).await?;
                entities.push(tweet_to_thread_entity(&head));
            }

            let mut includes_map = HashMap::new();
            if let Some(includes) = response.includes {
                for media in includes.media {
                    includes_map.insert(media.media_key.clone(), media.clone());
                }
            }
    
            for reply in response.data.iter().rev() {
                entities.push(conversation_reply_to_thread_entity(&reply, &includes_map));
            }
    
            let name = decode_html(&thread_user.name)?;
            let thumb_url = Url::parse(thread_user.profile_image_url_https.as_str())?;
            return Ok(ThreadReply {
                user_name: Some(name),
                thumb_url: Some(thumb_url),
                entities
            });
        } else {
            return Err(BotError::from(BotErrorKind::MissedConversationId))
        }
    } else {
        return Err(BotError::from(BotErrorKind::MissedUserInTweet));
    }
}

async fn conversation_id_from_tweet(tweet_id: u64, token: &Token) -> Result<Option<String>, BotError> {
    let url = format!("https://api.twitter.com/2/tweets/{}", tweet_id);

    let params = raw::ParamList::new()
        .add_param("tweet.fields", "conversation_id");

    let req = raw::request_get(url.as_str(), &token, Some(&params));
    let output: Response<serde_json::Value> = raw::response_json(req).await?;
    match output.response["data"]["conversation_id"].as_str() {
        Some(str) => Ok(Some(String::from(str))),
        _ => Ok(None)
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct ConversationResponse {
    pub data: Vec<ConversationReply>,
    pub includes: Option<ConversationIncludes>,
    pub meta: ConversationMeta,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct ConversationReply {
    pub id: String,
    pub text: String,
    pub attachments: Option<ConversationReplyAttachments>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct ConversationReplyAttachments {
    pub media_keys: Vec<String>
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct ConversationIncludes {
    pub media: Vec<ConversationIncludesMediaEntity>
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct ConversationIncludesMediaEntity {
    pub media_key: String,
    pub r#type: String,
    pub width: i32,
    pub height: i32,
    pub url: Option<String>,
    pub preview_image_url: Option<String>,
    pub duration_ms: Option<i32>,
    pub variants: Option<Vec<ConversationIncludesMediaVariant>>
}                 

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct ConversationIncludesMediaVariant {
    pub content_type: String,
    pub url: String,
    pub bit_rate: Option<i32>
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct ConversationMeta {
    pub next_token: Option<String>,
}

async fn all_replies_in_conversation(conversation_id: &String, user_screen_name: &String, max_count: u64, with_includes: bool, token: &Token) -> Result<ConversationResponse, BotError> {
    let result: ConversationResponse = replies_in_conversation(conversation_id, user_screen_name, max_count, with_includes, None, token).await?;
    let mut replies = result.data;
    let mut includes = result.includes.map(|i| i.media).unwrap_or(vec![]);
    let mut next_token = result.meta.next_token;

    while let Some(next_token_value) = &next_token {
        let next_result = replies_in_conversation(conversation_id, user_screen_name, max_count, with_includes, Some(next_token_value), token).await?;
        let mut next_replies = next_result.data;
        replies.append(&mut next_replies);

        let mut next_includes = next_result.includes.map(|i| i.media).unwrap_or(vec![]);
        includes.append(&mut next_includes);

        next_token = next_result.meta.next_token;
    }

    return Ok(ConversationResponse {
        data: replies,
        includes: match includes.len() {
            0 => None,
            _ => Some(ConversationIncludes { media: includes })
        },
        meta: ConversationMeta { next_token: None },
    })
}

async fn replies_in_conversation(conversation_id: &String, user_screen_name: &String, max_count: u64, with_includes: bool, next_token: Option<&String>, token: &Token) -> Result<ConversationResponse, BotError> {
    let url = format!("https://api.twitter.com/2/tweets/search/recent");
    let mut params = ParamList::new()
    .add_param("query", format!("conversation_id:{} from:{} to:{}", conversation_id, user_screen_name, user_screen_name))
    .add_param("max_results", max_count.to_string());

    if with_includes {
        params = params
        .add_param("expansions", "attachments.media_keys")
        .add_param("media.fields", "alt_text,duration_ms,height,media_key,preview_image_url,type,url,variants,width");
    }

    if let Some(next) = next_token {
        params = params.add_param("next_token", next.clone())
    }

    let req = raw::request_get(url.as_str(), &token, Some(&params));
    let output: Response<ConversationResponse> = raw::response_json(req).await?;
    Ok(output.response)
}

fn tweet_to_thread_entity(tweet: &Tweet) -> ThreadEntity {
    ThreadEntity {
        text: tweet_text_to_displayable_string(&tweet.text),
        media_entities: tweet_media(&tweet),
    }
}

fn conversation_reply_to_thread_entity(reply: &ConversationReply, includes: &HashMap<String, ConversationIncludesMediaEntity>) -> ThreadEntity {
    ThreadEntity {
        text: tweet_text_to_displayable_string(&reply.text),
        media_entities: conversation_reply_media(reply, includes),    
    }
}

fn conversation_reply_media(reply: &ConversationReply, includes: &HashMap<String, ConversationIncludesMediaEntity>) -> Vec<ParsedMedia> {
    let mut media_entities: Vec<ConversationIncludesMediaEntity> = vec![];
    if let Some(attachments) = &reply.attachments {
        for media_key in &attachments.media_keys {
            if let Some(media) = includes.get(media_key) {
                media_entities.push(media.clone())
            }
        }
    } else {
        return vec![]
    }

    return media_entities.iter().filter_map(|entity| {
        let preview_url = match (&entity.url, &entity.preview_image_url) {
            (Some(url), _) => url.clone(),
            (_, Some(url)) => url.clone(),
            _ => String::from("")
        };

        if let Ok(thumb_url) = Url::parse(preview_url.as_str()) {
            let id = entity.media_key.clone();
            if let Some(variants) = &entity.variants {
                let mut mp4_variants = variants.iter().filter(|v| v.content_type == "video/mp4").collect::<Vec<_>>();
                mp4_variants.sort_by_key(|v| v.bit_rate.unwrap_or(0));
                if let Some(variant) = mp4_variants.last() {
                    if let Ok(url) = Url::parse(variant.url.as_str()) {
                        return Some(ParsedMedia::Video(VideoEntity {
                            id,
                            url: url,
                            thumb_url: thumb_url,
                            mime_type: "video/mp4".parse().unwrap(),
                            width: entity.width,
                            height: entity.height
                        }));
                    }
                }
            }

            return Some(ParsedMedia::Image(ImageEntity {
                id,
                url: thumb_url,
                width: entity.width,
                height: entity.height
            }));
        }

        None        
    }).collect::<Vec<_>>();
}
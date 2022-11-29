use std::collections::HashSet;
use std::iter::FromIterator;
use std::string::String;

use egg_mode::raw::ParamList;
use egg_mode::{raw, Response, Token};
use egg_mode::tweet::{Tweet, show};
use htmlescape::*;
use reqwest::Url;
use serde::{Serialize, Deserialize};

use crate::bot_errors::{BotError, BotErrorKind};
use crate::parser::{ReplyData, tweet_text_to_displayable_string};
pub struct ThreadReply {
    pub user_name: Option<String>,
    pub thumb_url: Option<Url>,
    pub texts: Vec<String>
}

pub async fn is_included_in_thread(tweet: &Tweet, token: &Token) -> Result<bool, BotError> {
    if let Some(thread_user) = tweet.user.as_ref() {
        if let Some(conversation_id) = conversation_id_from_tweet(tweet.id, &token).await? {
            let replies = replies_in_conversation(
                conversation_id,
                thread_user.screen_name.clone(),
                10, 
                &token
            ).await?;
            return Ok(replies.len() >= 2);
        }
    }

    Ok(false)
}

pub async fn tweet_to_thread(start_tweet: &Tweet, token: &Token, first_chunk_len: usize, default_chunk_len: usize) -> Result<ThreadReply, BotError> {
    if let Some(thread_user) = start_tweet.user.as_ref() {
        let start_id = start_tweet.id;
        if let Some(conversation_id) = conversation_id_from_tweet(start_id, &token).await? {            
            let replies = replies_in_conversation(
                conversation_id, 
                thread_user.screen_name.clone(),
                100, 
                &token
            ).await?;

            let thread_ids = replies.iter()
            .map(|r| { r.id.parse().unwrap() });
            let thread_ids = HashSet::<u64>::from_iter(thread_ids);

            let mut texts = vec![];
            let mut text = String::from("");
            if thread_ids.contains(&start_id) {
                // Start tweet is in the middle
                let first_reply_id = replies.last().map(|t| { t.id.clone() }).unwrap();
                let first_reply = show(first_reply_id.parse().unwrap(), &token).await?;
                if let Some(head_id) = first_reply.in_reply_to_status_id {
                    let head = show(head_id, &token).await?;
                    text = tweet_text_to_displayable_string(&head.text);
                }               
            } else {
                // Start tweet is the head
                text = tweet_text_to_displayable_string(&start_tweet.text);
            }
    
            let mut chunk_max_size = first_chunk_len;
            for reply in replies.iter().rev() {
                let tweet_text = tweet_text_to_displayable_string(reply.text.as_str());
                if text.len() + tweet_text.len() > chunk_max_size {
                    texts.push(text);
                    text = tweet_text;
                    chunk_max_size = default_chunk_len;
                } else {
                    text = format!("{}\n\n{}", text, tweet_text);
                }
            }

            texts.push(text);
    
            let name = decode_html(&thread_user.name)?;
            let thumb_url = Url::parse(thread_user.profile_image_url_https.as_str())?;
            return Ok(ThreadReply {
                user_name: Some(name),
                thumb_url: Some(thumb_url),
                texts: texts
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
struct ConversationReply {
    pub id: String,
    pub text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct ConversationResponse {
    pub data: Vec<ConversationReply>,
}

async fn replies_in_conversation(conversation_id: String, user_screen_name: String, max_count: u64, token: &Token) -> Result<Vec<ConversationReply>, BotError> {
    let url = format!("https://api.twitter.com/2/tweets/search/recent");
    let params = ParamList::new()
    .add_param("query", format!("conversation_id:{} from:{} to:{}", conversation_id, user_screen_name, user_screen_name))
    .add_param("max_results", max_count.to_string());

    let req = raw::request_get(url.as_str(), &token, Some(&params));
    let output: Response<ConversationResponse> = raw::response_json(req).await?;
    Ok(output.response.data)
}

impl ReplyData for ThreadReply {
    fn user_name(&self) -> Option<String> {
        return self.user_name.clone();
    }

    fn text(&self) -> String {
        return self.texts.first().unwrap_or(&String::from("")).clone();
    }
}
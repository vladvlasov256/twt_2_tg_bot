use std::convert::TryInto;
use std::string::String;
use async_trait::async_trait;
use egg_mode::*;
use teloxide::payloads::{SendVideoSetters, SendPhotoSetters, SendMessageSetters};
use teloxide::requests::Requester;
use teloxide::types::{ParseMode, ChatId, InputFile, InputMedia, InputMediaPhoto, InputMediaVideo};

use crate::analytics::track_hit;
use crate::thread_parser::{tweet_to_thread, ThreadReply, ThreadEntity};
use crate::{bot_errors::BotError, thread_parser::is_included_in_thread};
use crate::parser::*;

use teloxide::{utils::markdown::{bold, escape}, Bot};

#[async_trait]
pub trait UpdateProcessor: Sync + Send {
    fn text_with_link(&self) -> Option<&String>;

    fn message_chat_id(&self) -> Option<ChatId> {
        None
    }

    async fn process(&self, bot: Bot, token: &Token) -> Result<(), BotError> {
        match self.text_with_link() {
            Some(text) => {
                let id = tweet_id_from_link(text)?;
                return self.process_tweet(bot, id, token).await;
            },
            _ => Ok(())
        }
    }

    async fn process_tweet(&self, bot: Bot, id: u64, token: &Token) -> Result<(), BotError> {
        self.track_hit_if_necessary().await?;
        let tweet = tweet::show(id, &token).await?.response;
        let reply = tweet_to_reply(&tweet).await?;
        let id = format!("{}", tweet.id);
        let included_in_thread = is_included_in_thread(&tweet, &token).await.unwrap_or(false);
        self.answer(bot, id, reply, included_in_thread).await
    }

    async fn unroll_tweet(&self, bot: Bot, id: u64, token: &Token) -> Result<(), BotError> {
        track_hit(String::from("unroll")).await?;
        let tweet = tweet::show(id, &token).await?.response;
        let reply = tweet_to_thread(&tweet, &token).await?;
        self.send_thread_reply(&bot, format!("{}", tweet.id), reply, false).await
    }

    async fn answer(&self, bot: Bot, id: String, reply: Reply, included_in_thread: bool) -> Result<(), BotError>;

    async fn send_thread_reply(&self, bot: &Bot, _id: String, thread_reply: ThreadReply, _included_in_thread: bool) -> Result<(), BotError> {
        if thread_reply.entities.is_empty() {
            return Ok(())
        }

        let mut message_entity = thread_reply.entities.first().unwrap();
        let mut max_chunk_length = max_text_size(message_entity.media_entities.len() == 1);

        let escaped_text = escape(message_entity.text.as_str());
        let mut message_text = match thread_reply.user_name {
            Some(name) => format!("{}\n\n{}", bold(escape(name.as_str()).as_str()), escaped_text),
            None => escaped_text
        };

        let mut did_edit_original_message = false;

        if thread_reply.entities.len() > 1 {
            for entity in &thread_reply.entities[1..] {
                let entity_text = escape(entity.text.as_str());
                let chunk_length = message_text.len() + 2 + entity_text.len();
                if entity.media_entities.len() > 0 || chunk_length > max_chunk_length {
                    if did_edit_original_message {
                        self.send_thread_entity(bot, message_entity, &message_text).await?;
                    } else {
                        self.edit_message_with_thread_entity(bot, message_entity, &message_text).await?;
                        did_edit_original_message = true;
                    }
                    message_entity = entity;
                    message_text = entity_text;
                    max_chunk_length = max_text_size(message_entity.media_entities.len() == 1);
                } else {
                    message_text = format!("{}\n\n{}", message_text, entity_text);
                }
            }
        }

        if did_edit_original_message {
            self.send_thread_entity(bot, message_entity, &message_text).await?;
        } else {
            self.edit_message_with_thread_entity(bot, message_entity, &message_text).await?;
        }  

        Ok(())
    }

    async fn edit_message_with_thread_entity(&self, bot: &Bot, entity: &ThreadEntity, escaped_text: &String) -> Result<(), BotError> { 
        return self.send_thread_entity(bot, entity, escaped_text).await
    }

    async fn send_thread_entity(&self, bot: &Bot, entity: &ThreadEntity, escaped_text: &String) -> Result<(), BotError> {
        match entity.media_entities.len() {
            0 => self.send_text_thread_entity(bot, &escaped_text).await,
            1 => match entity.media_entities.first().unwrap() {
                ParsedMedia::Image(image) => self.send_image_thread_entity(bot, image, &escaped_text).await,
                ParsedMedia::Video(video) => self.send_video_thread_entity(bot, video, &escaped_text).await,
            },
            _ => self.send_media_group_thread_entity(bot, &entity, &escaped_text).await
        }
    }

    async fn send_text_thread_entity(&self, bot: &Bot, escaped_text: &String) -> Result<(), BotError> {
        bot.send_message(self.message_chat_id().unwrap(), escaped_text)
        .parse_mode(ParseMode::MarkdownV2)
        .disable_web_page_preview(true)
        .await?;

        Ok(())
    }

    async fn send_image_thread_entity(&self, bot: &Bot, image: &ImageEntity, escaped_text: &String) -> Result<(), BotError> {
        let video = InputFile::url(image.url.clone());
        bot.send_photo(self.message_chat_id().unwrap(), video)
        .caption(escaped_text)
        .parse_mode(ParseMode::MarkdownV2)
        .await?;

        Ok(())
    }

    async fn send_video_thread_entity(&self, bot: &Bot, video: &VideoEntity, escaped_text: &String) -> Result<(), BotError> {
        let video = InputFile::url(video.url.clone());
        bot.send_video(self.message_chat_id().unwrap(), video)
        .caption(escaped_text)
        .parse_mode(ParseMode::MarkdownV2)
        .await?;

        Ok(())
    }

    async fn send_media_group_thread_entity(&self, bot: &Bot, reply: &ThreadEntity, escaped_text: &String) -> Result<(), BotError> {
        let group = reply.media_entities.iter()
        .map(|media_entity| {
            match media_entity {
                ParsedMedia::Image(image) => InputMedia::Photo(InputMediaPhoto {
                    media: InputFile::url(image.url.clone()),
                    caption: None,
                    parse_mode: None,
                    caption_entities: None
                }),
                ParsedMedia::Video(video) => InputMedia::Video(InputMediaVideo {
                    media: InputFile::url(video.url.clone()),
                    thumb: None,
                    caption: None,
                    parse_mode: None,
                    caption_entities: None,
                    width: Some(video.width.try_into().unwrap()),
                    height: Some(video.height.try_into().unwrap()),
                    duration: None,
                    supports_streaming: None,
                })
            }            
        }).collect::<Vec<_>>();
        
        let chat_id = self.message_chat_id().unwrap();
        bot.send_media_group(chat_id, group).await?;

        bot.send_message(chat_id, escaped_text)
        .parse_mode(ParseMode::MarkdownV2)
        .disable_web_page_preview(true)
        .await?;

        Ok(())
    }

    async fn track_hit_if_necessary(&self) -> Result<(), BotError>;
}

/// Returns escaped text with the user name as a bold title.
pub fn escaped_text<T>(data: &T) -> String where T: ReplyData {
    let escaped_text = escape(data.text().as_str());
    match data.user_name() {
        Some(name) => format!("{}\n\n{}", bold(escape(name.as_str()).as_str()), escaped_text),
        None => escaped_text
    }
}

pub fn max_text_size(caption: bool) -> usize {
    match caption {
        true => 1024,
        false => 4096
    }
}
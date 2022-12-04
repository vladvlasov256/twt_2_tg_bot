use std::string::String;

use async_trait::async_trait;

use egg_mode::tweet;
use teloxide::prelude::*;
use teloxide::types::{ParseMode, InputMediaPhoto, InputFile, InputMedia, InputMediaVideo};

use crate::analytics::track_hit;
use crate::bot_errors::{BotError, BotErrorKind};
use crate::thread_parser::{tweet_to_thread, ThreadEntity};
use crate::update_processor::{UpdateProcessor, escaped_text};
use crate::parser::{tweet_to_reply, Reply, ParsedMedia};

pub struct CallbackQueryProcessor {
    pub query: CallbackQuery
}

#[async_trait]
impl UpdateProcessor for CallbackQueryProcessor {
    fn text_with_link(&self) -> Option<&String> { 
        return None
    }

    async fn process(&self, bot: Bot, token: &egg_mode::Token) -> Result<(), BotError> {
        let data = self.data_as_str()?;

        // Processes "Unroll" reply button from a regular text message.
        if data.starts_with("unroll_") {
            track_hit(String::from("unroll")).await?;
            let tweet_id = data.strip_prefix("unroll_").unwrap();
            let id = tweet_id.parse().unwrap();
            let tweet = tweet::show(id, &token).await?.response;
            let reply = tweet_to_thread(&tweet, &token).await?;
            return self.send_thread_reply(&bot, String::from(tweet_id), reply, false).await;
        } else {
            track_hit(String::from("callback")).await?;
            let id = data.parse().unwrap();
            let tweet = tweet::show(id, &token).await?.response;
            let reply = tweet_to_reply(&tweet).await?;
            return self.answer(bot, data, reply, false).await;
        }
    }

    async fn answer(&self, bot: Bot, _id: String, reply: Reply, _included_in_thread: bool) -> Result<(), BotError> {
        let images = reply.media_entities.iter()
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
                    width: None,
                    height: None,
                    duration: None,
                    supports_streaming: None,
                })
            }            
        }).collect::<Vec<_>>();

        let chat_id = self.chat_id()?;
        let messages: Vec<Message> = bot.send_media_group(chat_id.clone(), images).await?;

        if let Some(reply_message) = messages.first() {
            bot
            .edit_message_caption(chat_id.clone(), reply_message.id)
            .caption(escaped_text(&reply))
            .parse_mode(ParseMode::MarkdownV2)
            .await?;
        }

        Ok(())
    }

    async fn edit_message_with_thread_entity(&self, bot: &Bot, _entity: &ThreadEntity, escaped_text: &String) -> Result<(), BotError> {
        if let Some(message) = &self.query.message {
            if let Some(_text) = message.text() {
                bot
                .edit_message_text(message.chat.id, message.id, escaped_text)
                .parse_mode(ParseMode::MarkdownV2)
                .disable_web_page_preview(true)
                .await?;
            } else if let Some(_capiton) = message.caption() {
                bot
                .edit_message_caption(message.chat.id, message.id)
                .caption(escaped_text)
                .parse_mode(ParseMode::MarkdownV2)
                .await?;
            }
        }
        Ok(())
    }

    async fn track_hit_if_necessary(&self) -> Result<(), BotError> {
        Ok(())
    }

    fn message_chat_id(&self) -> Option<ChatId> {
        if let Some(message) = &self.query.message {
            return Some(message.chat.id)
        }
        None
    }
}

impl CallbackQueryProcessor {
    fn chat_id(&self) -> Result<ChatId, BotError> {
        if let Some(value) = self.query.chat_instance.parse().ok() {
            return Ok(ChatId(value))
        }

        if let Some(chat_id) = self.message_chat_id() {
            return Ok(chat_id)
        }

        Err(BotError::from(BotErrorKind::CallbackDataParsingError))
    }

    fn data_as_str(&self) -> Result<String, BotError> {
        match &self.query.data {
            Some(data) => Ok(data.clone()),
            _ => Err(BotError::from(BotErrorKind::CallbackDataParsingError))
        }
    }
}
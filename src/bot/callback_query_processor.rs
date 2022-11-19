use std::string::String;

use async_trait::async_trait;

use egg_mode::tweet;
use teloxide::prelude::*;
use teloxide::types::{ParseMode, InputMediaPhoto, InputFile, InputMedia};
use teloxide::utils::markdown::escape;

use crate::bot_errors::{BotError, BotErrorKind};
use crate::thread_parser::tweet_to_thread;
use crate::update_processor::{UpdateProcessor, escaped_text};
use crate::parser::{ImageReply, tweet_to_reply, Reply, ThreadReply};

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
            let tweet_id = data.strip_prefix("unroll_").unwrap();
            let id = tweet_id.parse().unwrap();
            let tweet = tweet::show(id, &token).await?.response;
            let reply = tweet_to_thread(&tweet, &token, self.first_chunk_len(), 3072).await?;
            return self.answer(bot, String::from(tweet_id), Reply::Thread(reply), false).await;
        } else {
            let id = data.parse().unwrap();
            let tweet = tweet::show(id, &token).await?.response;
            let reply = tweet_to_reply(&tweet).await?;
            return self.answer(bot, data, reply, false).await;
        }
    }
    
    async fn send_thread_reply(&self, bot: Bot, _id: String, thread_reply: ThreadReply, _included_in_thread: bool) -> Result<(), BotError> {
        if let Some(message) = self.query.message.clone() {
            if let Some(_text) = message.text() {
                if thread_reply.texts.len() > 0 {
                    bot
                    .edit_message_text(message.chat.id, message.id, escaped_text(&thread_reply))
                    .parse_mode(ParseMode::MarkdownV2)
                    .disable_web_page_preview(true)
                    .await?;
                }
            } else if let Some(_capiton) = message.caption() {
                if thread_reply.texts.len() > 0 {
                    bot
                    .edit_message_caption(message.chat.id, message.id)
                    .caption(escaped_text(&thread_reply))
                    .parse_mode(ParseMode::MarkdownV2)
                    .await?;
                }
            }

            if thread_reply.texts.len() > 1 {
                for text in &thread_reply.texts[1..] {
                    bot.send_message(message.chat.id, escape(&text))
                    .parse_mode(ParseMode::MarkdownV2)
                    .disable_web_page_preview(true)
                    .await?;
                }
            }            
        }

        Ok(())
    }

    async fn send_image_reply(&self, bot: Bot, _id: String, reply: ImageReply, _included_in_thread: bool) -> Result<(), BotError> {
        let images = reply.images.iter()
        .map(|image| {
            let image = InputMediaPhoto {
                media: InputFile::url(image.url.clone()),
                caption: None,
                parse_mode: None,
                caption_entities: None
            };
            InputMedia::Photo(image)
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
}

impl CallbackQueryProcessor {
    fn chat_id(&self) -> Result<ChatId, BotError> {
        if let Some(value) = self.query.chat_instance.parse().ok() {
            return Ok(ChatId(value))
        }

        if let Some(message) = self.query.message.clone() {
            return Ok(message.chat.id)
        }

        Err(BotError::from(BotErrorKind::CallbackDataParsingError))
    }

    fn data_as_str(&self) -> Result<String, BotError> {
        match &self.query.data {
            Some(data) => Ok(data.clone()),
            _ => Err(BotError::from(BotErrorKind::CallbackDataParsingError))
        }
    }

    fn first_chunk_len(&self) -> usize {
        if let Some(message) = self.query.message.clone() {
            if let Some(_text) = message.text() {
                return 3072
            } else {
                return 768
            }
        }
        return 0
    }
}
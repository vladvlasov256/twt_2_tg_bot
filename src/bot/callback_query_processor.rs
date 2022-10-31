use std::string::String;

use async_trait::async_trait;

use egg_mode::tweet;
use teloxide::prelude::*;
use teloxide::types::{ParseMode, InputMediaPhoto, InputFile, InputMedia};

use crate::bot_errors::{BotError, BotErrorKind};
use crate::update_processor::{UpdateProcessor, escaped_text};
use crate::parser::{ImageReply, tweet_to_reply};

pub struct CallbackQueryProcessor {
    pub query: CallbackQuery
}

impl CallbackQueryProcessor {
    fn chat_id(&self) -> Result<i64, BotError> {
        match self.query.chat_instance.parse().ok() {
            Some(value) => Ok(value),
            _ => Err(BotError::from(BotErrorKind::CallbackDataParsingError))
        }
    }
}

#[async_trait]
impl UpdateProcessor for CallbackQueryProcessor {
    fn text_with_link(&self) -> Option<&String> { 
        return None
    }

    async fn process(&self, bot: Bot, token: &egg_mode::Token) -> Result<(), BotError> {
        let id = self.tweet_id()?;
        let tweet = tweet::show(id, &token).await?;
        let reply = tweet_to_reply(&tweet.response).await?;
        let id = format!("{}", tweet.id);
        return self.answer(bot, id, reply).await;
    }

    async fn send_image_reply(&self, bot: Bot, _id: String, reply: ImageReply) -> Result<(), BotError> {
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

        let chat_id = ChatId(self.chat_id()?);
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
    fn tweet_id(&self) -> Result<u64, BotError> {
        match &self.query.data {
            Some(data) => {
                match data.parse().ok() {
                    Some(value) => Ok(value),
                    _ => Err(BotError::from(BotErrorKind::CallbackDataParsingError))
                }
            },
            _ => Err(BotError::from(BotErrorKind::CallbackDataParsingError))
        }
    }
}
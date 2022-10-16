use std::string::String;

use async_trait::async_trait;

use egg_mode::tweet;
use teloxide::adaptors::DefaultParseMode;
use teloxide::prelude::*;
use teloxide::types::{ParseMode, InputMediaPhoto, InputFile, InputMedia};

use crate::bot_errors::BotErrorKind;
use crate::update_processor::{UpdateProcessor, escaped_text};
use crate::parser::{ImageReply, tweet_to_reply};

pub struct CallbackQueryProcessor {
    pub query: UpdateWithCx<AutoSend<DefaultParseMode<Bot>>, CallbackQuery>
}

#[async_trait]
impl UpdateProcessor for CallbackQueryProcessor {
    fn text_with_link(&self) -> Option<&String> { 
        return None
    }

    async fn process(&self, token: &egg_mode::Token) -> Result<(), BotErrorKind> {
        let id = self.tweet_id()?;
        let tweet = tweet::show(id, &token).await?;
        let reply = tweet_to_reply(&tweet.response).await?;
        let id = format!("{}", tweet.id);
        return self.answer(id, reply).await;
    }

    async fn send_image_reply(&self, _id: String, reply: ImageReply) -> Result<(), BotErrorKind> {
        let images = reply.images.iter()
        .map(|image| {
            let image = InputMediaPhoto {
                media: InputFile::Url(image.url.clone()),
                caption: None,
                parse_mode: None,
                caption_entities: None
            };
            InputMedia::Photo(image)
        }).collect::<Vec<_>>();

        let messages: Vec<Message> = self.query.requester
        .send_media_group(self.query.update.chat_instance.clone(), images)
        .await?;

        if let Some(reply_message) = messages.first() {
            self.query.requester
            .edit_message_caption(self.query.update.chat_instance.clone(), reply_message.id)
            .caption(escaped_text(&reply))
            .parse_mode(ParseMode::MarkdownV2)
            .await?;
        }

        Ok(())
    }
}

impl CallbackQueryProcessor {
    fn tweet_id(&self) -> Result<u64, BotErrorKind> {
        match &self.query.update.data {
            Some(data) => {
                match data.parse().ok() {
                    Some(value) => Ok(value),
                    _ => Err(BotErrorKind::CallbackDataParsingError)
                }
            },
            _ => Err(BotErrorKind::CallbackDataParsingError)
        }
    }
}
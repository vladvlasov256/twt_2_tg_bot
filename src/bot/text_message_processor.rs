use std::string::String;
use async_trait::async_trait;

use teloxide::adaptors::DefaultParseMode;
use teloxide::prelude::*;
use teloxide::types::{InputFile, InputMedia, InputMediaPhoto, ParseMode};

use crate::update_processor::{UpdateProcessor, escaped_text};
use crate::bot_errors::{BotErrorKind};
use crate::parser::{TextReply, VideoReply, ImageReply}; 

pub struct TextMessageProcessor {
    pub message: UpdateWithCx<AutoSend<DefaultParseMode<Bot>>, Message>,
    pub text: String
}

#[async_trait]
impl UpdateProcessor for TextMessageProcessor {
    fn text_with_link(&self) -> &String {
        &self.text
    }

    async fn send_video_reply(&self, _id: String, video_reply: VideoReply) -> Result<(), BotErrorKind> {
        let video = InputFile::Url(video_reply.url);
        self.message.answer_video(video).await?;    
        Ok(())
    }
    
    async fn send_text_reply(&self, _id: String, text_reply: TextReply) -> Result<(), BotErrorKind> {
        let answer_text = escaped_text(&text_reply);
        self.message.answer(answer_text).await?;
        Ok(())
    }

    async fn send_image_reply(&self, _id: String, reply: ImageReply) -> Result<(), BotErrorKind> {
        let photos = reply.photos.iter()
        .map(|image| {
            let photo = InputMediaPhoto {
                media: InputFile::Url(image.url.clone()),
                caption: None,
                parse_mode: None,
                caption_entities: None
            };
            InputMedia::Photo(photo)
        })
        .collect::<Vec<_>>();
        
        let messages: Vec<Message> = self.message.answer_media_group(photos).await?;

        if let Some(reply_message) = messages.first() {
            self.message.requester
            .edit_message_caption(self.message.chat_id(), reply_message.id)
            .caption(escaped_text(&reply))
            .parse_mode(ParseMode::MarkdownV2)
            .await?;
        }

        Ok(())
    }
}
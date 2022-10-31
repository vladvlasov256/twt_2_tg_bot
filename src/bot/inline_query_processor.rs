use std::string::String;

use async_trait::async_trait;

use teloxide::prelude::*;
use teloxide::types::{InlineQueryResult, InlineQueryResultArticle, InlineQueryResultVideo, InputMessageContent, InputMessageContentText, ParseMode, InlineQueryResultPhoto};
use teloxide::utils::markdown::escape;

use crate::bot_errors::{BotError};
use crate::update_processor::{UpdateProcessor, escaped_text};
use crate::parser::{TextReply, VideoReply, ImageReply};

pub struct InlineQueryProcessor {
    pub query: InlineQuery
}

#[async_trait]
impl UpdateProcessor for InlineQueryProcessor {
    fn text_with_link(&self) -> Option<&String> { 
       Some(&self.query.query)
    }

    async fn send_video_reply(&self, bot: Bot, id: String, video_reply: VideoReply) -> Result<(), BotError> {
        let result = InlineQueryResult::Video(self.result_video(id, video_reply));
        return self.answer(bot, vec![result]).await;
    }
    
    async fn send_text_reply(&self, bot: Bot, id: String, text_reply: TextReply) -> Result<(), BotError> {
        let result = InlineQueryResult::Article(self.result_article(id, text_reply));
        return self.answer(bot, vec![result]).await;
    }

    async fn send_image_reply(&self, bot: Bot, id: String, image_reply: ImageReply) -> Result<(), BotError> {
        let image_count = image_reply.images.len();
        
        let results = self.result_photos(id.clone(), image_reply).iter()
        .map(|photo| {
            InlineQueryResult::Photo(photo.clone())
        }).collect::<Vec<_>>();

        let mut answer = bot.answer_inline_query(self.query_id(), results);

        if image_count > 1 {
            answer = answer.switch_pm_text("Get all images").switch_pm_parameter(id.clone());
        }

        answer.await?;
        Ok(())
    }
}

impl InlineQueryProcessor {
    async fn answer<R: IntoIterator>(&self, bot: Bot, results: R) -> Result<(), BotError> 
    where R: IntoIterator<Item = InlineQueryResult> {
        bot.answer_inline_query(self.query_id(), results).await?;
        Ok(())
    }

    fn query_id(&self) -> String {
        self.query.id.clone()
    }

    fn result_video(&self, id: String, video_reply: VideoReply) -> InlineQueryResultVideo {
        let title: String;
        let description: Option<String>;
        if let Some(user_name) = video_reply.user_name.clone() {
            title = escape(user_name.clone().as_str());
            description = Some(video_reply.text.clone());
        } else {
            title = video_reply.text.clone();
            description = None;
        }

        return InlineQueryResultVideo {
            id: id,
            video_url: video_reply.url.clone(),
            mime_type: video_reply.mime_type.clone(),
            thumb_url: video_reply.thumb_url.clone(),
            title: title,
            parse_mode: Some(ParseMode::MarkdownV2),
            caption: Some(escaped_text(&video_reply)),
            description,
            input_message_content: None,
            reply_markup: None,
            video_duration: None,
            caption_entities: None,
            video_width: None,
            video_height: None
        };
    }

    fn result_article(&self, id: String, text_reply: TextReply) -> InlineQueryResultArticle {
        let title: String;
        let description: Option<String>;
        if let Some(user_name) = text_reply.user_name.clone() {
            title = escape(user_name.clone().as_str());
            description = Some(text_reply.text.clone());
        } else {
            title = text_reply.text.clone();
            description = None;
        }

        return InlineQueryResultArticle {
            id: id,
            title: title,
            input_message_content: self.message_content(escaped_text(&text_reply)),
            reply_markup: None,
            url: None,
            hide_url: None,
            description: description,
            thumb_url: text_reply.thumb_url.clone(),
            thumb_width: None,
            thumb_height: None,
        }
    }

    fn result_photos(&self, id: String, reply: ImageReply) -> Vec<InlineQueryResultPhoto> {
        let title: String;
        let description: Option<String>;
        if let Some(user_name) = reply.user_name.clone() {
            title = escape(user_name.clone().as_str());
            description = Some(reply.text.clone());
        } else {
            title = reply.text.clone();
            description = None;
        }

        reply.images.iter().map(|image| {
            InlineQueryResultPhoto {
                id: format!("{}_{}", id, image.id),
                photo_url: image.url.clone(),
                thumb_url: image.url.clone(),
                photo_width: Some(image.width),
                photo_height: Some(image.height),
                title: Some(title.clone()),
                description: description.clone(),
                caption: Some(escaped_text(&reply)),
                parse_mode: Some(ParseMode::MarkdownV2),
                caption_entities: None,
                reply_markup: None,
                input_message_content: None
            }
        })
        .collect::<Vec<_>>()
    }

    fn message_content(&self, text: String) -> InputMessageContent {
        return InputMessageContent::Text(InputMessageContentText {
            message_text: text,
            parse_mode: Some(ParseMode::MarkdownV2),
            entities: None,
            disable_web_page_preview: None,
        })        
    }
}
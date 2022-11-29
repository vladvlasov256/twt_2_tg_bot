use std::string::String;

use async_trait::async_trait;

use teloxide::prelude::*;
use teloxide::types::{InlineQueryResult, InlineQueryResultArticle, InlineQueryResultVideo, InputMessageContent, InputMessageContentText, ParseMode, InlineQueryResultPhoto};
use teloxide::utils::markdown::escape;

use crate::analytics::track_hit;
use crate::bot_errors::{BotError};
use crate::update_processor::{UpdateProcessor, escaped_text};
use crate::parser::{Reply, ParsedMedia};

pub struct InlineQueryProcessor {
    pub query: InlineQuery
}

#[async_trait]
impl UpdateProcessor for InlineQueryProcessor {
    fn text_with_link(&self) -> Option<&String> { 
       Some(&self.query.query)
    }

    async fn answer(&self, bot: Bot, id: String, reply: Reply, included_in_thread: bool) -> Result<(), BotError> {
        match reply.media_entities.len() {
            0 => self.send_text_reply(bot, id, reply, included_in_thread).await,
            _ => self.send_media_reply(bot, id, reply, included_in_thread).await
        }
    }

    async fn track_hit_if_necessary(&self) -> Result<(), BotError> {
        track_hit(String::from("inline")).await
    }
}

impl InlineQueryProcessor {    
    async fn send_text_reply(&self, bot: Bot, id: String, reply: Reply, included_in_thread: bool) -> Result<(), BotError> {
        let result = InlineQueryResult::Article(self.result_article(id.clone(), reply));

        let pm_text = match included_in_thread {
            true => Some(String::from("Unroll")),
            false => None
        };
        let pm_parameter = match included_in_thread {
            true => Some(String::from(format!("unroll_{}", id.clone()))),
            false => None
        };

        return self.answer(bot, vec![result], pm_text, pm_parameter).await;
    }

    async fn send_media_reply(&self, bot: Bot, id: String, reply: Reply, included_in_thread: bool) -> Result<(), BotError> {  
        let media_entity_count = reply.media_entities.len();

        let results = self.result_media(id.clone(), reply);

        let pm_text = match (included_in_thread, media_entity_count > 1) {
            (true, false) => Some(String::from("Unroll Thread")),
            (false, true) => Some(String::from("All Media")),
            (true, true) => Some(String::from("More")),
            _ => None
        };
        let pm_parameter = match (included_in_thread, media_entity_count > 1) {
            (true, false) => Some(String::from(format!("unroll_{}", id.clone()))),
            (_, true) => Some(id.clone()),
            _ => None
        };

        return self.answer(bot, results, pm_text, pm_parameter).await;
    }

    async fn answer<R: IntoIterator>(&self, bot: Bot, results: R, pm_text: Option<String>, pm_parameter: Option<String>) -> Result<(), BotError> 
    where R: IntoIterator<Item = InlineQueryResult> {
        let mut answer = bot.answer_inline_query(self.query_id(), results);

        if let Some(text) = pm_text {
            answer = answer.switch_pm_text(text);
        }
        if let Some(parameter) = pm_parameter {
            answer = answer.switch_pm_parameter(parameter);
        }
        
        answer.await?;
        Ok(())
    }

    fn query_id(&self) -> String {
        self.query.id.clone()
    }

    fn result_article(&self, id: String, reply: Reply) -> InlineQueryResultArticle {
        let title: String;
        let description: Option<String>;
        if let Some(user_name) = reply.user_name.clone() {
            title = escape(user_name.clone().as_str());
            description = Some(reply.text.clone());
        } else {
            title = reply.text.clone();
            description = None;
        }

        return InlineQueryResultArticle {
            id: id,
            title: title,
            input_message_content: self.message_content(escaped_text(&reply)),
            reply_markup: None,
            url: None,
            hide_url: None,
            description: description,
            thumb_url: reply.thumb_url.clone(),
            thumb_width: None,
            thumb_height: None,
        }
    }

    fn result_media(&self, id: String, reply: Reply) -> Vec<InlineQueryResult> {
        let title: String;
        let description: Option<String>;
        if let Some(user_name) = reply.user_name.clone() {
            title = escape(user_name.clone().as_str());
            description = Some(reply.text.clone());
        } else {
            title = reply.text.clone();
            description = None;
        }

        reply.media_entities.iter().map(|entity| {
            match entity {
                ParsedMedia::Image(image) => InlineQueryResult::Photo(InlineQueryResultPhoto {
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
                }),
                ParsedMedia::Video(video) => InlineQueryResult::Video(InlineQueryResultVideo {
                    id: format!("{}_{}", id, video.id),
                    video_url: video.url.clone(),
                    mime_type: video.mime_type.clone(),
                    thumb_url: video.thumb_url.clone(),
                    title: title.clone(),
                    parse_mode: Some(ParseMode::MarkdownV2),
                    caption: Some(escaped_text(&reply)),
                    description: description.clone(),
                    input_message_content: None,
                    reply_markup: None,
                    video_duration: None,
                    caption_entities: None,
                    video_width: Some(video.width),
                    video_height: Some(video.height)
                })
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
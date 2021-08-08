use std::string::String;

use async_trait::async_trait;

use teloxide::adaptors::DefaultParseMode;
use teloxide::prelude::*;
use teloxide::types::{InlineQueryResult, InlineQueryResultArticle, InlineQueryResultVideo, InputMessageContent, InputMessageContentText, ParseMode};
use teloxide::utils::markdown::escape;

use crate::bot_errors::BotErrorKind;
use crate::update_processor::{UpdateProcessor, escaped_text};
use crate::parser::{TextReply, VideoReply};

pub struct InlineQueryProcessor {
    pub query: UpdateWithCx<AutoSend<DefaultParseMode<Bot>>, InlineQuery>
}

#[async_trait]
impl UpdateProcessor for InlineQueryProcessor {
    fn text_with_link(&self) -> &String { 
        &self.query.update.query
    }

    async fn send_video_reply(&self, id: String, video_reply: VideoReply) -> Result<(), BotErrorKind> {
        let result = InlineQueryResult::Video(self.result_video(id, video_reply));
        return self.answer(result).await;
    }
    
    async fn send_text_reply(&self, id: String, text_reply: TextReply) -> Result<(), BotErrorKind> {
        let result = InlineQueryResult::Article(self.result_article(id, text_reply));
        return self.answer(result).await;
    }
}

impl InlineQueryProcessor {
    async fn answer(&self, result: InlineQueryResult) -> Result<(), BotErrorKind> {
        self.query.requester.answer_inline_query(self.query_id(), vec![result]).await?;        
        Ok(())
    }

    fn query_id(&self) -> String {
        self.query.update.id.clone()
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

    fn message_content(&self, text: String) -> InputMessageContent {
        return InputMessageContent::Text(InputMessageContentText {
            message_text: text,
            parse_mode: Some(ParseMode::MarkdownV2),
            entities: None,
            disable_web_page_preview: None,
        })        
    }
}
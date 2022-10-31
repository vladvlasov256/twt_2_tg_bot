use std::string::String;
use async_trait::async_trait;

use teloxide::prelude::*;
use teloxide::types::{InputFile, InputMedia, InputMediaPhoto, ParseMode};
use teloxide::utils::markdown::{bold, escape};

use crate::update_processor::{UpdateProcessor, escaped_text};
use crate::bot_errors::{BotError};
use crate::parser::{TextReply, VideoReply, ImageReply, tweet_id_from_link, tweet_id}; 

pub struct TextMessageProcessor {
    pub message: Message,
    pub text: String
}

#[async_trait]
impl UpdateProcessor for TextMessageProcessor {
    fn text_with_link(&self) -> Option<&String> {
        Some(&self.text)
    }

    async fn process(&self, bot: Bot, token: &egg_mode::Token) -> Result<(), BotError> {
        if self.text.as_str().starts_with("/start") {
            match self.tweet_id_from_deeplink(&self.text) {
                Ok(id) => {
                    return self.process_tweet(bot, id, token).await;
                },
                _ => {
                    return self.send_info_message(bot).await;
                }
            }
        } else {
            match tweet_id_from_link(&self.text) {
                Ok(id) => { return self.process_tweet(bot, id, token).await },
                _ => Ok(())
            }            
        }
    }

    async fn send_video_reply(&self, bot: Bot, _id: String, video_reply: VideoReply) -> Result<(), BotError> {
        let video = InputFile::url(video_reply.url);
        bot.send_video(self.message.chat.id, video)
        .parse_mode(ParseMode::MarkdownV2)
        .await?;    
        Ok(())
    }
    
    async fn send_text_reply(&self, bot: Bot, _id: String, text_reply: TextReply) -> Result<(), BotError> {
        bot.send_message(self.message.chat.id, escaped_text(&text_reply))
        .parse_mode(ParseMode::MarkdownV2)
        .disable_web_page_preview(true)
        .await?;
        Ok(())
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
        })
        .collect::<Vec<_>>();
        
        let chat_id = self.message.chat.id;
        let messages: Vec<Message> = bot.send_media_group(chat_id, images).await?;

        if let Some(reply_message) = messages.first() {
            bot.edit_message_caption(chat_id, reply_message.id)
            .caption(escaped_text(&reply))
            .parse_mode(ParseMode::MarkdownV2)
            .await?;
        }

        Ok(())
    }
}

impl TextMessageProcessor {
    fn tweet_id_from_deeplink(&self, text: &String) -> Result<u64, BotError> {
        tweet_id(text, r"/start (\d+)")
    }

    async fn send_info_message(&self, bot: Bot) -> Result<(), BotError> {
        let info_text = "This bot allows you to convert tweet links to regular Telegram messages. It can download videos, images, and text from tweets. Just send a link herr or address @twt2tgbot in any chat.";
        let text = format!("{}\n\n{}", bold(escape("What can this bot do?").as_str()), escape(info_text));
        bot.send_message(self.message.chat.id, text).parse_mode(ParseMode::MarkdownV2).await?;
        Ok(())
    }
}
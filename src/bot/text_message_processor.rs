use std::string::String;
use async_trait::async_trait;

use teloxide::prelude::*;
use teloxide::types::{InputFile, InputMedia, InputMediaPhoto, ParseMode, InlineKeyboardMarkup, InlineKeyboardButton};
use teloxide::utils::markdown::{bold, escape};

use crate::update_processor::{UpdateProcessor, escaped_text};
use crate::bot_errors::{BotError};
use crate::parser::{TextReply, VideoReply, ImageReply, tweet_id_from_link, tweet_id, ThreadReply}; 

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
            if self.text.as_str().starts_with("/start unroll_") {
                let id = self.text.as_str().strip_prefix("/start unroll_").unwrap().parse().unwrap();
                self.unroll_tweet(bot, id, token).await
            } else {
                match self.tweet_id_from_deeplink(&self.text) {
                    Ok(id) => self.process_tweet(bot, id, token).await,
                    _ => self.send_info_message(bot).await
                }
            }
        } else {
            match tweet_id_from_link(&self.text) {
                Ok(id) => self.process_tweet(bot, id, token).await,
                _ => Ok(())
            }            
        }
    }

    async fn send_video_reply(&self, bot: Bot, id: String, video_reply: VideoReply, included_in_thread: bool) -> Result<(), BotError> {
        let video = InputFile::url(video_reply.url.clone());
        let mut request = bot.send_video(self.message.chat.id, video)
        .caption(escaped_text(&video_reply))
        .parse_mode(ParseMode::MarkdownV2); 

        if included_in_thread {
            request = request.reply_markup(self.make_keyboard(&id));
        }
        
        request.await?;

        Ok(())
    }
    
    async fn send_text_reply(&self, bot: Bot, id: String, text_reply: TextReply, included_in_thread: bool) -> Result<(), BotError> {
        let mut request = bot.send_message(self.message.chat.id, escaped_text(&text_reply))
        .parse_mode(ParseMode::MarkdownV2)
        .disable_web_page_preview(true);

        if included_in_thread {
            request = request.reply_markup(self.make_keyboard(&id));
        }

        request.await?;
        Ok(())
    }

    async fn send_image_reply(&self, bot: Bot, id: String, reply: ImageReply, included_in_thread: bool) -> Result<(), BotError> {
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
        bot.send_media_group(chat_id, images).await?;

        let mut request = bot.send_message(self.message.chat.id, escaped_text(&reply))
        .parse_mode(ParseMode::MarkdownV2)
        .disable_web_page_preview(true);

        if included_in_thread {
            request = request.reply_markup(self.make_keyboard(&id));
        }

        request.await?;
        Ok(())
    }

    async fn send_thread_reply(&self, bot: Bot, _id: String, thread_reply: ThreadReply, _included_in_thread: bool) -> Result<(), BotError> {
        if thread_reply.texts.len() > 0 {
            bot.send_message(self.message.chat.id, escaped_text(&thread_reply))
            .parse_mode(ParseMode::MarkdownV2)
            .disable_web_page_preview(true)
            .await?;
        }

        if thread_reply.texts.len() > 1 {
            for text in &thread_reply.texts[1..] {
                bot.send_message(self.message.chat.id, escape(&text))
                .parse_mode(ParseMode::MarkdownV2)
                .disable_web_page_preview(true)
                .await?;
            }
        }

        Ok(())
    }
}

impl TextMessageProcessor {
    fn tweet_id_from_deeplink(&self, text: &String) -> Result<u64, BotError> {
        tweet_id(text, r"/start (\d+)")
    }

    async fn send_info_message(&self, bot: Bot) -> Result<(), BotError> {
        let info_text = "This bot allows you to convert tweet links to regular Telegram messages. It can download videos and images from tweets. And also it can unroll threads. Just send a link herr or address @twt2tgbot in any chat.";
        let text = format!("{}\n\n{}", bold(escape("What can this bot do?").as_str()), escape(info_text));
        bot.send_message(self.message.chat.id, text).parse_mode(ParseMode::MarkdownV2).await?;
        Ok(())
    }
    
    fn make_keyboard(&self, tweet_id: &String) -> InlineKeyboardMarkup {
        let unroll_button = InlineKeyboardButton::callback(
            String::from("Unroll Thread"),
            format!("unroll_{}", tweet_id.clone())
        );
        let keyboard: Vec<Vec<InlineKeyboardButton>> = vec![vec![unroll_button]];    
        InlineKeyboardMarkup::new(keyboard)
    }
}
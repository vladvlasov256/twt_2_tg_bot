use std::convert::TryInto;
use std::string::String;
use async_trait::async_trait;

use teloxide::prelude::*;
use teloxide::types::{InputFile, InputMedia, InputMediaPhoto, ParseMode, InlineKeyboardMarkup, InlineKeyboardButton, InputMediaVideo};
use teloxide::utils::markdown::{bold, escape};

use crate::analytics::track_hit;
use crate::update_processor::{UpdateProcessor, escaped_text};
use crate::bot_errors::BotError;
use crate::parser::{tweet_id_from_link, tweet_id, Reply, ParsedMedia, VideoEntity, ImageEntity}; 

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

    async fn answer(&self, bot: Bot, id: String, reply: Reply, included_in_thread: bool) -> Result<(), BotError> {
        match reply.media_entities.len() {
            0 => self.send_text_reply(bot, id, &reply, included_in_thread).await,
            1 => match reply.media_entities.first().unwrap() {
                ParsedMedia::Image(image) => self.send_image_reply(bot, id, &reply, image, included_in_thread).await,
                ParsedMedia::Video(video) => self.send_video_reply(bot, id, &reply, video, included_in_thread).await,
            },
            _ => self.send_media_group_reply(bot, id, &reply, included_in_thread).await
        }
    }

    async fn track_hit_if_necessary(&self) -> Result<(), BotError> {
        track_hit(String::from("message")).await
    }

    fn message_chat_id(&self) -> Option<ChatId> {
        Some(self.message.chat.id)
    }
}

impl TextMessageProcessor {
    async fn send_text_reply(&self, bot: Bot, id: String, reply: &Reply, included_in_thread: bool) -> Result<(), BotError> {
        let mut request = bot.send_message(self.message.chat.id, escaped_text(reply))
        .parse_mode(ParseMode::MarkdownV2)
        .disable_web_page_preview(true);

        if included_in_thread {
            request = request.reply_markup(self.make_keyboard(&id));
        }

        request.await?;
        Ok(())
    }

    async fn send_image_reply(&self, bot: Bot, id: String, reply: &Reply, image: &ImageEntity, included_in_thread: bool) -> Result<(), BotError> {
        let video = InputFile::url(image.url.clone());
        let mut request = bot.send_photo(self.message.chat.id, video)
        .caption(escaped_text(reply))
        .parse_mode(ParseMode::MarkdownV2); 

        if included_in_thread {
            request = request.reply_markup(self.make_keyboard(&id));
        }
        
        request.await?;

        Ok(())
    }

    async fn send_video_reply(&self, bot: Bot, id: String, reply: &Reply, video: &VideoEntity, included_in_thread: bool) -> Result<(), BotError> {
        let video = InputFile::url(video.url.clone());
        let mut request = bot.send_video(self.message.chat.id, video)
        .caption(escaped_text(reply))
        .parse_mode(ParseMode::MarkdownV2); 

        if included_in_thread {
            request = request.reply_markup(self.make_keyboard(&id));
        }
        
        request.await?;

        Ok(())
    }

    async fn send_media_group_reply(&self, bot: Bot, id: String, reply: &Reply, included_in_thread: bool) -> Result<(), BotError> {
        let group = reply.media_entities.iter()
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
                    width: Some(video.width.try_into().unwrap()),
                    height: Some(video.height.try_into().unwrap()),
                    duration: None,
                    supports_streaming: None,
                })
            }            
        }).collect::<Vec<_>>();
        
        let chat_id = self.message.chat.id;
        bot.send_media_group(chat_id, group).await?;

        let mut request = bot.send_message(self.message.chat.id, escaped_text(reply))
        .parse_mode(ParseMode::MarkdownV2)
        .disable_web_page_preview(true);

        if included_in_thread {
            request = request.reply_markup(self.make_keyboard(&id));
        }

        request.await?;
        Ok(())
    }

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
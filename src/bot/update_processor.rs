use std::string::String;
use async_trait::async_trait;
use egg_mode::*;

use crate::thread_parser::tweet_to_thread;
use crate::{bot_errors::BotError, thread_parser::is_included_in_thread};
use crate::parser::*;

use teloxide::{utils::markdown::{bold, escape}, Bot};

#[async_trait]
pub trait UpdateProcessor: Sync + Send {
    fn text_with_link(&self) -> Option<&String>;

    async fn process(&self, bot: Bot, token: &Token) -> Result<(), BotError> {
        match self.text_with_link() {
            Some(text) => {
                let id = tweet_id_from_link(text)?;
                return self.process_tweet(bot, id, token).await;
            },
            _ => Ok(())
        }
    }

    async fn process_tweet(&self, bot: Bot, id: u64, token: &Token) -> Result<(), BotError> {
        let tweet = tweet::show(id, &token).await?.response;
        let reply = tweet_to_reply(&tweet).await?;
        let id = format!("{}", tweet.id);
        let included_in_thread = is_included_in_thread(&tweet, &token).await.unwrap_or(false);
        self.answer(bot, id, reply, included_in_thread).await
    }

    async fn unroll_tweet(&self, bot: Bot, id: u64, token: &Token) -> Result<(), BotError> {
        let tweet = tweet::show(id, &token).await?.response;
        let reply = tweet_to_thread(&tweet, &token, 3072, 3072).await?;
        self.send_thread_reply(bot, format!("{}", tweet.id), reply, false).await
    }

    async fn answer(&self, bot: Bot, id: String, reply: Reply, included_in_thread: bool) -> Result<(), BotError> {
        match reply {
            Reply::Video(video_data) => self.send_video_reply(bot, id, video_data, included_in_thread).await,
            Reply::Text(text_reply) => self.send_text_reply(bot, id, text_reply, included_in_thread).await,
            Reply::Image(image_reply) => self.send_image_reply(bot, id, image_reply, included_in_thread).await,
            Reply::Thread(thread_reply) => self.send_thread_reply(bot, id, thread_reply, included_in_thread).await
        }
    }

    async fn send_video_reply(&self, _bot: Bot, _id: String, _video_reply: VideoReply, _included_in_thread: bool) -> Result<(), BotError> {
        Ok(())
    }
    
    async fn send_text_reply(&self, _bot: Bot, _id: String, _text_reply: TextReply, _included_in_thread: bool) -> Result<(), BotError> {
        Ok(())
    }

    async fn send_image_reply(&self, bot: Bot, id: String, image_reply: ImageReply, included_in_thread: bool) -> Result<(), BotError>;

    async fn send_thread_reply(&self, _bot: Bot, _id: String, _thread_reply: ThreadReply, _included_in_thread: bool) -> Result<(), BotError> {
        Ok(())
    }
}

/// Returns escaped text with the user name as a bold title.
pub fn escaped_text<T>(data: &T) -> String where T: ReplyData {
    let escaped_text = escape(data.text().as_str());
    match data.user_name() {
        Some(name) => format!("{}\n\n{}", bold(escape(name.as_str()).as_str()), escaped_text),
        None => escaped_text
    }
}
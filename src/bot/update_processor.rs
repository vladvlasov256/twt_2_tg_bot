use std::string::String;
use async_trait::async_trait;
use egg_mode::*;

use crate::bot_errors::BotError;
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
        let tweet = tweet::show(id, &token).await?;
        let reply = tweet_to_reply(&tweet.response).await?;
        let id = format!("{}", tweet.id);
        return self.answer(bot, id, reply).await
    }

    async fn answer(&self, bot: Bot, id: String, reply: Reply) -> Result<(), BotError> {
        match reply {
            Reply::Video(video_data) => self.send_video_reply(bot, id, video_data).await,
            Reply::Text(text_reply) => self.send_text_reply(bot, id, text_reply).await,
            Reply::Image(image_reply) => self.send_image_reply(bot, id, image_reply).await
        }
    }

    async fn send_video_reply(&self, _bot: Bot, _id: String, _video_reply: VideoReply) -> Result<(), BotError> {
        Ok(())
    }
    
    async fn send_text_reply(&self, _bot: Bot, _id: String, _text_reply: TextReply) -> Result<(), BotError> {
        Ok(())
    }

    async fn send_image_reply(&self, bot: Bot, id: String, image_reply: ImageReply) -> Result<(), BotError>;
}

/// Returns escaped text with the user name as a bold title.
pub fn escaped_text<T>(data: &T) -> String where T: ReplyData {
    let escaped_text = escape(data.text().as_str());
    match data.user_name() {
        Some(name) => format!("{}\n\n{}", bold(escape(name.as_str()).as_str()), escaped_text),
        None => escaped_text
    }
}
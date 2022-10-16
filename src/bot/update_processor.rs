use std::string::String;
use async_trait::async_trait;
use egg_mode::*;

use crate::bot_errors::BotErrorKind;
use crate::parser::*;

use teloxide::{utils::markdown::{bold, escape}};

#[async_trait]
pub trait UpdateProcessor: Sync + Send {
    fn text_with_link(&self) -> Option<&String>;

    async fn process(&self, token: &Token) -> Result<(), BotErrorKind> {
        match self.text_with_link() {
            Some(text) => {
                let id = tweet_id_from_link(text)?;
                return self.process_tweet(id, token).await;
            },
            _ => Ok(())
        }
    }

    async fn process_tweet(&self, id: u64, token: &Token) -> Result<(), BotErrorKind> {
        let tweet = tweet::show(id, &token).await?;
        let reply = tweet_to_reply(&tweet.response).await?;
        let id = format!("{}", tweet.id);
        return self.answer(id, reply).await
    }

    async fn answer(&self, id: String, reply: Reply) -> Result<(), BotErrorKind> {
        match reply {
            Reply::Video(video_data) => self.send_video_reply(id, video_data).await,
            Reply::Text(text_reply) => self.send_text_reply(id, text_reply).await,
            Reply::Image(image_reply) => self.send_image_reply(id, image_reply).await
        }
    }

    async fn send_video_reply(&self, _id: String, _video_reply: VideoReply) -> Result<(), BotErrorKind> {
        Ok(())
    }
    
    async fn send_text_reply(&self, _id: String, _text_reply: TextReply) -> Result<(), BotErrorKind> {
        Ok(())
    }

    async fn send_image_reply(&self, id: String, image_reply: ImageReply) -> Result<(), BotErrorKind>;
}

/// Returns escaped text with the user name as a bold title.
pub fn escaped_text<T>(data: &T) -> String where T: ReplyData {
    let escaped_text = escape(data.text().as_str());
    match data.user_name() {
        Some(name) => format!("{}\n\n{}", bold(escape(name.as_str()).as_str()), escaped_text),
        None => escaped_text
    }
}
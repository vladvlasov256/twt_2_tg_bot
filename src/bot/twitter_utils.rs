use egg_mode::*;
use crate::bot_errors::{BotErrorKind};

pub async fn twitter_api_token_value(client_id: String, secret: String) -> Result<Token, BotErrorKind> {
    let con_token = KeyPair::new(client_id, secret);
    return Ok(auth::bearer_token(&con_token).await?);
}
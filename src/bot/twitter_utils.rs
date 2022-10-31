use egg_mode::*;
use crate::bot_errors::BotError;

pub async fn twitter_api_token_value(client_id: String, secret: String) -> Result<Token, BotError> {
    let con_token = KeyPair::new(client_id, secret);
    Ok(auth::bearer_token(&con_token).await?)
}
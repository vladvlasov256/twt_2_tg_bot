use std::string::String;
use std::fs::File;
use std::io::copy;
use std::io::Cursor;

use crate::bot_errors::{BotErrorKind};

pub async fn download_file(url: &String, dest: &mut File) -> Result<(), BotErrorKind> {
    let response = reqwest::get(url.as_str()).await?;
    let bytes = response.bytes().await?;
    let mut content = Cursor::new(bytes);
    copy(&mut content, dest)?;
    
    Ok(())
}
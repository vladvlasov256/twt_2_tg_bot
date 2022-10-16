use std::error;
use std::fmt;

use teloxide::RequestError;

#[derive(Debug)]
pub struct BotError(BotErrorKind);

#[derive(Debug)]
pub enum BotErrorKind {
    TweetParsingError,
    Io(std::io::Error),
    HTTPError(reqwest::Error),
    MessageSendingError(RequestError),
    TwitterAPIError(egg_mode::error::Error),
    TextParsingError(regex::Error),
    HTMLDecodeError(htmlescape::DecodeErr),
    CallbackDataParsingError
}

impl From<std::io::Error> for BotErrorKind {
    fn from(error: std::io::Error) -> Self {
        BotErrorKind::Io(error)
    }
}

impl From<reqwest::Error> for BotErrorKind {
    fn from(error: reqwest::Error) -> Self {
        BotErrorKind::HTTPError(error)
    }
}

impl From<RequestError> for BotErrorKind {
    fn from(error: RequestError) -> Self {
        BotErrorKind::MessageSendingError(error)
    }
}

impl From<egg_mode::error::Error> for BotErrorKind {
    fn from(error: egg_mode::error::Error) -> Self {
        BotErrorKind::TwitterAPIError(error)
    }
}

impl From<regex::Error> for BotErrorKind {
    fn from(error: regex::Error) -> Self {
        BotErrorKind::TextParsingError(error)
    }
}

impl From<htmlescape::DecodeErr> for BotErrorKind {
    fn from(error: htmlescape::DecodeErr) -> Self {
        BotErrorKind::HTMLDecodeError(error)
    }
}

impl From<BotErrorKind> for BotError {
    fn from(kind: BotErrorKind) -> Self {
        BotError(kind)
    }
}

impl fmt::Display for BotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            BotErrorKind::TweetParsingError => write!(f, "Tweet parsing error"),
            BotErrorKind::Io(error) => write!(f, "{}", error),
            BotErrorKind::HTTPError(error) => write!(f, "{}", error),
            BotErrorKind::MessageSendingError(error) => write!(f, "{}", error),
            BotErrorKind::TwitterAPIError(error) => write!(f, "{}", error),
            BotErrorKind::TextParsingError(error) => write!(f, "{}", error),
            BotErrorKind::HTMLDecodeError(_) => write!(f, "HTML decoding error"),
            BotErrorKind::CallbackDataParsingError => write!(f, "Callback data parsing error"),
        }
    }
}

impl error::Error for BotError {}
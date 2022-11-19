use std::error;
use std::fmt;

use teloxide::RequestError;
use url::ParseError;

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
    CallbackDataParsingError,
    MissedConversationId,
    InvalidThreadResponse,
    MissedUserInTweet,
}

impl From<BotErrorKind> for BotError {
    fn from(error: BotErrorKind) -> Self {
        BotError(error)
    }
}

impl From<std::io::Error> for BotError {
    fn from(error: std::io::Error) -> Self {
        BotError(BotErrorKind::Io(error))
    }
}

impl From<reqwest::Error> for BotError {
    fn from(error: reqwest::Error) -> Self {
        BotError(BotErrorKind::HTTPError(error))
    }
}

impl From<RequestError> for BotError {
    fn from(error: RequestError) -> Self {
        BotError(BotErrorKind::MessageSendingError(error))
    }
}

impl From<egg_mode::error::Error> for BotError {
    fn from(error: egg_mode::error::Error) -> Self {
        BotError(BotErrorKind::TwitterAPIError(error))
    }
}

impl From<regex::Error> for BotError {
    fn from(error: regex::Error) -> Self {
        BotError(BotErrorKind::TextParsingError(error))
    }
}

impl From<ParseError> for BotError {
    fn from(_error: ParseError) -> Self {
        BotError(BotErrorKind::TweetParsingError)
    }
}

impl From<htmlescape::DecodeErr> for BotError {
    fn from(error: htmlescape::DecodeErr) -> Self {
        BotError(BotErrorKind::HTMLDecodeError(error))
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
            BotErrorKind::MissedConversationId => write!(f, "Convesation ID is missed for a thread"), 
            BotErrorKind::InvalidThreadResponse => write!(f, "Invalid thread response"),     
            BotErrorKind::MissedUserInTweet => write!(f, "Missed user in tweet"),           
        }
    }
}

impl error::Error for BotError {}
use std::fmt;

use crate::Error;

/// Authentication strategy for Telegram Bot API requests.
#[derive(Clone, Eq, PartialEq)]
#[non_exhaustive]
pub enum Auth {
    /// No authentication token.
    None,
    /// Bot token authentication.
    BotToken(BotToken),
}

impl Auth {
    /// Build an auth object with no credentials.
    pub const fn none() -> Self {
        Self::None
    }

    /// Build an auth object from a Telegram bot token.
    pub fn bot_token(token: impl Into<String>) -> Result<Self, Error> {
        Ok(Self::BotToken(BotToken::new(token)?))
    }

    pub(crate) fn token(&self) -> Option<&str> {
        match self {
            Self::None => None,
            Self::BotToken(token) => Some(token.expose()),
        }
    }
}

impl fmt::Debug for Auth {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => formatter.debug_tuple("Auth::None").finish(),
            Self::BotToken(_) => formatter
                .debug_struct("Auth::BotToken")
                .field("token", &"<redacted>")
                .finish(),
        }
    }
}

/// Bot token wrapper with redacted debug output.
#[derive(Clone, Eq, PartialEq)]
pub struct BotToken(String);

impl BotToken {
    /// Create a new bot token.
    pub fn new(token: impl Into<String>) -> Result<Self, Error> {
        let token = token.into();
        if token.trim().is_empty() {
            return Err(Error::InvalidBotToken);
        }

        Ok(Self(token))
    }

    pub(crate) fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for BotToken {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BotToken")
            .field("value", &"<redacted>")
            .finish()
    }
}

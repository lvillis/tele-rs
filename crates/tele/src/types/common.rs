use serde::{Deserialize, Serialize};

use crate::{Error, Result};

/// Telegram user id wrapper.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct UserId(pub i64);

/// Telegram message id wrapper.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct MessageId(pub i64);

/// Numeric Telegram chat id wrapper.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct NumericChatId(pub i64);

impl From<i64> for NumericChatId {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl From<i64> for UserId {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl From<i64> for MessageId {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl UserId {
    /// Validates that this is a concrete Telegram user id.
    pub fn validate(&self) -> Result<()> {
        if self.0 <= 0 {
            return Err(invalid_request("user_id must be greater than 0"));
        }

        Ok(())
    }
}

impl MessageId {
    /// Validates that this is a concrete Telegram message id.
    pub fn validate(&self) -> Result<()> {
        if self.0 <= 0 {
            return Err(invalid_request("message_id must be greater than 0"));
        }

        Ok(())
    }
}

impl NumericChatId {
    /// Validates that this is a concrete numeric Telegram chat id.
    pub fn validate(&self) -> Result<()> {
        if self.0 == 0 {
            return Err(invalid_request("chat_id cannot be 0"));
        }

        Ok(())
    }
}

/// Chat id can be numeric or @username string.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ChatId {
    /// Numeric chat id.
    Id(i64),
    /// Public channel username like `@my_channel`.
    Username(String),
}

impl From<i64> for ChatId {
    fn from(value: i64) -> Self {
        Self::Id(value)
    }
}

impl From<String> for ChatId {
    fn from(value: String) -> Self {
        Self::Username(value)
    }
}

impl From<&str> for ChatId {
    fn from(value: &str) -> Self {
        Self::Username(value.to_owned())
    }
}

impl ChatId {
    /// Validates the local representation before sending it to Telegram.
    pub fn validate(&self) -> Result<()> {
        match self {
            Self::Id(0) => Err(invalid_request("chat_id cannot be 0")),
            Self::Id(_) => Ok(()),
            Self::Username(username) => validate_chat_username(username),
        }
    }
}

fn validate_chat_username(username: &str) -> Result<()> {
    let Some(handle) = username.strip_prefix('@') else {
        return Err(invalid_request(
            "chat_id username must start with `@` when provided as a string",
        ));
    };
    if handle.is_empty() {
        return Err(invalid_request("chat_id username cannot be empty"));
    }
    if !(5..=32).contains(&handle.len()) {
        return Err(invalid_request(
            "chat_id username must be 5-32 ASCII characters after `@`",
        ));
    }
    if !handle
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    {
        return Err(invalid_request(
            "chat_id username may only contain ASCII letters, digits or `_` after `@`",
        ));
    }

    Ok(())
}

fn invalid_request(reason: impl Into<String>) -> Error {
    Error::InvalidRequest {
        reason: reason.into(),
    }
}

/// Telegram parse mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ParseMode {
    #[serde(rename = "Markdown")]
    Markdown,
    #[serde(rename = "MarkdownV2")]
    MarkdownV2,
    #[serde(rename = "HTML")]
    Html,
}

/// Telegram API error parameters.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ResponseParameters {
    /// Group migration target chat id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub migrate_to_chat_id: Option<i64>,
    /// Retry after N seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retry_after: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_chat_id_values() {
        assert!(ChatId::from(1_i64).validate().is_ok());
        assert!(ChatId::from(-100_i64).validate().is_ok());
        assert!(ChatId::from("@channel_name").validate().is_ok());

        for chat_id in [
            ChatId::from(0_i64),
            ChatId::from("channel"),
            ChatId::from("@"),
            ChatId::from("@abcd"),
            ChatId::from("@bad-name"),
            ChatId::from("@bad name"),
            ChatId::from("@abcdefghijklmnopqrstuvwxyzabcdefg"),
        ] {
            assert!(matches!(
                chat_id.validate(),
                Err(Error::InvalidRequest { .. })
            ));
        }
    }

    #[test]
    fn validates_user_and_message_ids() {
        assert!(UserId::from(1_i64).validate().is_ok());
        assert!(MessageId::from(1_i64).validate().is_ok());
        assert!(NumericChatId::from(1_i64).validate().is_ok());
        assert!(NumericChatId::from(-100_i64).validate().is_ok());

        for user_id in [UserId::from(0_i64), UserId::from(-1_i64)] {
            assert!(matches!(
                user_id.validate(),
                Err(Error::InvalidRequest { .. })
            ));
        }

        for message_id in [MessageId::from(0_i64), MessageId::from(-1_i64)] {
            assert!(matches!(
                message_id.validate(),
                Err(Error::InvalidRequest { .. })
            ));
        }

        assert!(matches!(
            NumericChatId::from(0_i64).validate(),
            Err(Error::InvalidRequest { .. })
        ));
    }
}

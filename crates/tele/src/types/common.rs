use serde::{Deserialize, Serialize};

/// Telegram user id wrapper.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct UserId(pub i64);

/// Telegram message id wrapper.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct MessageId(pub i64);

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

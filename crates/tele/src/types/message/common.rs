use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::bot::User;
use crate::types::common::MessageId;

/// Telegram chat type.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChatType {
    Private,
    Group,
    Supergroup,
    Channel,
}

/// Telegram chat object.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Chat {
    pub id: i64,
    #[serde(rename = "type")]
    pub kind: ChatType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_forum: Option<bool>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl Chat {
    pub fn is_private(&self) -> bool {
        matches!(self.kind, ChatType::Private)
    }

    pub fn is_group(&self) -> bool {
        matches!(self.kind, ChatType::Group)
    }

    pub fn is_supergroup(&self) -> bool {
        matches!(self.kind, ChatType::Supergroup)
    }

    pub fn is_channel(&self) -> bool {
        matches!(self.kind, ChatType::Channel)
    }

    pub fn is_group_chat(&self) -> bool {
        self.is_group() || self.is_supergroup()
    }
}

/// Telegram message entity kind.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum MessageEntityKind {
    Mention,
    Hashtag,
    Cashtag,
    BotCommand,
    Url,
    Email,
    PhoneNumber,
    Bold,
    Italic,
    Underline,
    Strikethrough,
    Spoiler,
    Blockquote,
    ExpandableBlockquote,
    Code,
    Pre,
    TextLink,
    TextMention,
    CustomEmoji,
    DateTime,
    Unknown(String),
}

impl MessageEntityKind {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Mention => "mention",
            Self::Hashtag => "hashtag",
            Self::Cashtag => "cashtag",
            Self::BotCommand => "bot_command",
            Self::Url => "url",
            Self::Email => "email",
            Self::PhoneNumber => "phone_number",
            Self::Bold => "bold",
            Self::Italic => "italic",
            Self::Underline => "underline",
            Self::Strikethrough => "strikethrough",
            Self::Spoiler => "spoiler",
            Self::Blockquote => "blockquote",
            Self::ExpandableBlockquote => "expandable_blockquote",
            Self::Code => "code",
            Self::Pre => "pre",
            Self::TextLink => "text_link",
            Self::TextMention => "text_mention",
            Self::CustomEmoji => "custom_emoji",
            Self::DateTime => "date_time",
            Self::Unknown(kind) => kind.as_str(),
        }
    }
}

impl<'de> Deserialize<'de> for MessageEntityKind {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let kind = String::deserialize(deserializer)?;
        Ok(match kind.as_str() {
            "mention" => Self::Mention,
            "hashtag" => Self::Hashtag,
            "cashtag" => Self::Cashtag,
            "bot_command" => Self::BotCommand,
            "url" => Self::Url,
            "email" => Self::Email,
            "phone_number" => Self::PhoneNumber,
            "bold" => Self::Bold,
            "italic" => Self::Italic,
            "underline" => Self::Underline,
            "strikethrough" => Self::Strikethrough,
            "spoiler" => Self::Spoiler,
            "blockquote" => Self::Blockquote,
            "expandable_blockquote" => Self::ExpandableBlockquote,
            "code" => Self::Code,
            "pre" => Self::Pre,
            "text_link" => Self::TextLink,
            "text_mention" => Self::TextMention,
            "custom_emoji" => Self::CustomEmoji,
            "date_time" => Self::DateTime,
            _ => Self::Unknown(kind),
        })
    }
}

impl Serialize for MessageEntityKind {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

/// Telegram message entity.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct MessageEntity {
    #[serde(rename = "type")]
    pub kind: MessageEntityKind,
    pub offset: u32,
    pub length: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<User>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_emoji_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unix_time: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date_time_format: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram photo size object.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct PhotoSize {
    pub file_id: String,
    pub file_unique_id: String,
    pub width: u32,
    pub height: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_size: Option<u64>,
}

/// Telegram forwarded message origin.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[non_exhaustive]
pub enum MessageOrigin {
    User {
        date: i64,
        sender_user: User,
    },
    HiddenUser {
        date: i64,
        sender_user_name: String,
    },
    Chat {
        date: i64,
        sender_chat: Chat,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        author_signature: Option<String>,
    },
    Channel {
        date: i64,
        chat: Chat,
        message_id: MessageId,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        author_signature: Option<String>,
    },
}

impl MessageOrigin {
    pub fn date(&self) -> i64 {
        match self {
            Self::User { date, .. }
            | Self::HiddenUser { date, .. }
            | Self::Chat { date, .. }
            | Self::Channel { date, .. } => *date,
        }
    }

    pub fn user(&self) -> Option<&User> {
        match self {
            Self::User { sender_user, .. } => Some(sender_user),
            _ => None,
        }
    }

    pub fn chat(&self) -> Option<&Chat> {
        match self {
            Self::Chat { sender_chat, .. } => Some(sender_chat),
            Self::Channel { chat, .. } => Some(chat),
            _ => None,
        }
    }

    pub fn author_signature(&self) -> Option<&str> {
        match self {
            Self::Chat {
                author_signature, ..
            }
            | Self::Channel {
                author_signature, ..
            } => author_signature.as_deref(),
            _ => None,
        }
    }

    pub fn sender_name(&self) -> Option<&str> {
        match self {
            Self::User { sender_user, .. } => Some(sender_user.first_name.as_str()),
            Self::HiddenUser {
                sender_user_name, ..
            } => Some(sender_user_name.as_str()),
            Self::Chat { sender_chat, .. } => sender_chat
                .title
                .as_deref()
                .or(sender_chat.username.as_deref())
                .or(sender_chat.first_name.as_deref()),
            Self::Channel { chat, .. } => chat
                .title
                .as_deref()
                .or(chat.username.as_deref())
                .or(chat.first_name.as_deref()),
        }
    }

    pub fn message_id(&self) -> Option<MessageId> {
        match self {
            Self::Channel { message_id, .. } => Some(*message_id),
            _ => None,
        }
    }
}

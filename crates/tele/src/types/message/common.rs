use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::bot::User;
use crate::types::common::MessageId;
use crate::types::tagged::{serialize_tagged, strip_type, tagged_kind};

/// Telegram chat type.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum ChatType {
    Private,
    Group,
    Supergroup,
    Channel,
    Unknown(String),
}

impl ChatType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Private => "private",
            Self::Group => "group",
            Self::Supergroup => "supergroup",
            Self::Channel => "channel",
            Self::Unknown(value) => value.as_str(),
        }
    }
}

impl<'de> Deserialize<'de> for ChatType {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Ok(match value.as_str() {
            "private" => Self::Private,
            "group" => Self::Group,
            "supergroup" => Self::Supergroup,
            "channel" => Self::Channel,
            _ => Self::Unknown(value),
        })
    }
}

impl Serialize for ChatType {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
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
        self.kind == ChatType::Private
    }

    pub fn is_group(&self) -> bool {
        self.kind == ChatType::Group
    }

    pub fn is_supergroup(&self) -> bool {
        self.kind == ChatType::Supergroup
    }

    pub fn is_channel(&self) -> bool {
        self.kind == ChatType::Channel
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

/// Telegram forwarded message origin sent by a user.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct MessageOriginUser {
    pub date: i64,
    pub sender_user: User,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram forwarded message origin sent by a hidden user.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct MessageOriginHiddenUser {
    pub date: i64,
    pub sender_user_name: String,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram forwarded message origin sent on behalf of a chat.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct MessageOriginChat {
    pub date: i64,
    pub sender_chat: Chat,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author_signature: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram forwarded message origin sent from a channel.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct MessageOriginChannel {
    pub date: i64,
    pub chat: Chat,
    pub message_id: MessageId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author_signature: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram forwarded message origin.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum MessageOrigin {
    User(MessageOriginUser),
    HiddenUser(MessageOriginHiddenUser),
    Chat(Box<MessageOriginChat>),
    Channel(Box<MessageOriginChannel>),
    Unknown(Value),
}

impl MessageOrigin {
    pub fn kind(&self) -> Option<&str> {
        match self {
            Self::User(_) => Some("user"),
            Self::HiddenUser(_) => Some("hidden_user"),
            Self::Chat(_) => Some("chat"),
            Self::Channel(_) => Some("channel"),
            Self::Unknown(value) => tagged_kind(value),
        }
    }

    pub fn date(&self) -> Option<i64> {
        match self {
            Self::User(value) => Some(value.date),
            Self::HiddenUser(value) => Some(value.date),
            Self::Chat(value) => Some(value.date),
            Self::Channel(value) => Some(value.date),
            Self::Unknown(_) => None,
        }
    }

    pub fn as_user_origin(&self) -> Option<&MessageOriginUser> {
        match self {
            Self::User(value) => Some(value),
            Self::HiddenUser(_) | Self::Chat(_) | Self::Channel(_) | Self::Unknown(_) => None,
        }
    }

    pub fn into_user_origin(self) -> Option<MessageOriginUser> {
        match self {
            Self::User(value) => Some(value),
            Self::HiddenUser(_) | Self::Chat(_) | Self::Channel(_) | Self::Unknown(_) => None,
        }
    }

    pub fn as_hidden_user_origin(&self) -> Option<&MessageOriginHiddenUser> {
        match self {
            Self::HiddenUser(value) => Some(value),
            Self::User(_) | Self::Chat(_) | Self::Channel(_) | Self::Unknown(_) => None,
        }
    }

    pub fn into_hidden_user_origin(self) -> Option<MessageOriginHiddenUser> {
        match self {
            Self::HiddenUser(value) => Some(value),
            Self::User(_) | Self::Chat(_) | Self::Channel(_) | Self::Unknown(_) => None,
        }
    }

    pub fn as_chat_origin(&self) -> Option<&MessageOriginChat> {
        match self {
            Self::Chat(value) => Some(value),
            Self::User(_) | Self::HiddenUser(_) | Self::Channel(_) | Self::Unknown(_) => None,
        }
    }

    pub fn into_chat_origin(self) -> Option<MessageOriginChat> {
        match self {
            Self::Chat(value) => Some(*value),
            Self::User(_) | Self::HiddenUser(_) | Self::Channel(_) | Self::Unknown(_) => None,
        }
    }

    pub fn as_channel_origin(&self) -> Option<&MessageOriginChannel> {
        match self {
            Self::Channel(value) => Some(value),
            Self::User(_) | Self::HiddenUser(_) | Self::Chat(_) | Self::Unknown(_) => None,
        }
    }

    pub fn into_channel_origin(self) -> Option<MessageOriginChannel> {
        match self {
            Self::Channel(value) => Some(*value),
            Self::User(_) | Self::HiddenUser(_) | Self::Chat(_) | Self::Unknown(_) => None,
        }
    }

    pub fn as_unknown_value(&self) -> Option<&Value> {
        match self {
            Self::Unknown(value) => Some(value),
            Self::User(_) | Self::HiddenUser(_) | Self::Chat(_) | Self::Channel(_) => None,
        }
    }

    pub fn into_unknown_value(self) -> Option<Value> {
        match self {
            Self::Unknown(value) => Some(value),
            Self::User(_) | Self::HiddenUser(_) | Self::Chat(_) | Self::Channel(_) => None,
        }
    }

    pub fn user(&self) -> Option<&User> {
        match self {
            Self::User(value) => Some(&value.sender_user),
            _ => None,
        }
    }

    pub fn chat(&self) -> Option<&Chat> {
        match self {
            Self::Chat(value) => Some(&value.sender_chat),
            Self::Channel(value) => Some(&value.chat),
            _ => None,
        }
    }

    pub fn author_signature(&self) -> Option<&str> {
        match self {
            Self::Chat(value) => value.author_signature.as_deref(),
            Self::Channel(value) => value.author_signature.as_deref(),
            _ => None,
        }
    }

    pub fn sender_name(&self) -> Option<&str> {
        match self {
            Self::User(value) => Some(value.sender_user.first_name.as_str()),
            Self::HiddenUser(value) => Some(value.sender_user_name.as_str()),
            Self::Chat(value) => value
                .sender_chat
                .title
                .as_deref()
                .or(value.sender_chat.username.as_deref())
                .or(value.sender_chat.first_name.as_deref()),
            Self::Channel(value) => value
                .chat
                .title
                .as_deref()
                .or(value.chat.username.as_deref())
                .or(value.chat.first_name.as_deref()),
            Self::Unknown(_) => None,
        }
    }

    pub fn message_id(&self) -> Option<MessageId> {
        match self {
            Self::Channel(value) => Some(value.message_id),
            _ => None,
        }
    }
}

impl<'de> Deserialize<'de> for MessageOrigin {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        match tagged_kind(&value) {
            Some("user") => serde_json::from_value(strip_type(value))
                .map(Self::User)
                .map_err(serde::de::Error::custom),
            Some("hidden_user") => serde_json::from_value(strip_type(value))
                .map(Self::HiddenUser)
                .map_err(serde::de::Error::custom),
            Some("chat") => serde_json::from_value(strip_type(value))
                .map(Box::new)
                .map(Self::Chat)
                .map_err(serde::de::Error::custom),
            Some("channel") => serde_json::from_value(strip_type(value))
                .map(Box::new)
                .map(Self::Channel)
                .map_err(serde::de::Error::custom),
            Some(_) | None => Ok(Self::Unknown(value)),
        }
    }
}

impl Serialize for MessageOrigin {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::User(value) => serialize_tagged(serializer, "user", value),
            Self::HiddenUser(value) => serialize_tagged(serializer, "hidden_user", value),
            Self::Chat(value) => serialize_tagged(serializer, "chat", value),
            Self::Channel(value) => serialize_tagged(serializer, "channel", value),
            Self::Unknown(value) => value.serialize(serializer),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{Chat, ChatType};

    #[test]
    fn chat_type_preserves_unknown_values() -> Result<(), Box<dyn std::error::Error>> {
        let chat: Chat = serde_json::from_value(json!({
            "id": 1,
            "type": "future_chat",
            "future": {"kept": true}
        }))?;

        assert_eq!(chat.kind, ChatType::Unknown("future_chat".to_owned()));
        assert_eq!(chat.kind.as_str(), "future_chat");
        assert!(!chat.is_private());
        assert!(!chat.is_group_chat());
        assert_eq!(chat.extra["future"], json!({"kept": true}));
        assert_eq!(serde_json::to_value(chat)?["type"], "future_chat");
        Ok(())
    }

    #[test]
    fn chat_type_known_values_round_trip_as_strings() -> Result<(), Box<dyn std::error::Error>> {
        let chat_type: ChatType = serde_json::from_value(json!("supergroup"))?;

        assert_eq!(chat_type, ChatType::Supergroup);
        assert_eq!(chat_type.as_str(), "supergroup");
        assert_eq!(serde_json::to_value(chat_type)?, json!("supergroup"));
        Ok(())
    }
}

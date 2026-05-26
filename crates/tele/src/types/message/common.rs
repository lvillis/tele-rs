use std::collections::BTreeMap;

use serde::ser::SerializeMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::bot::User;
use crate::types::common::MessageId;
use crate::types::tagged::{strip_type, tagged_kind};

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

/// Telegram chat object.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct Chat {
    pub id: i64,
    #[serde(rename = "type")]
    pub kind: ChatType,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub first_name: Option<String>,
    #[serde(default)]
    pub last_name: Option<String>,
    #[serde(default)]
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
#[derive(Clone, Debug, Deserialize)]
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

struct MessageEntityUser<'a>(&'a User);

impl Serialize for MessageEntityUser<'_> {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let user = self.0;
        let optional_len = usize::from(user.last_name.is_some())
            + usize::from(user.username.is_some())
            + usize::from(user.language_code.is_some());
        let mut object = serializer.serialize_map(Some(optional_len + 3))?;
        object.serialize_entry("id", &user.id)?;
        object.serialize_entry("is_bot", &user.is_bot)?;
        object.serialize_entry("first_name", &user.first_name)?;
        if let Some(last_name) = user.last_name.as_ref() {
            object.serialize_entry("last_name", last_name)?;
        }
        if let Some(username) = user.username.as_ref() {
            object.serialize_entry("username", username)?;
        }
        if let Some(language_code) = user.language_code.as_ref() {
            object.serialize_entry("language_code", language_code)?;
        }
        object.end()
    }
}

impl Serialize for MessageEntity {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let optional_len = usize::from(self.url.is_some())
            + usize::from(self.user.is_some())
            + usize::from(self.language.is_some())
            + usize::from(self.custom_emoji_id.is_some())
            + usize::from(self.unix_time.is_some())
            + usize::from(self.date_time_format.is_some());
        let mut object = serializer.serialize_map(Some(optional_len + 3))?;
        object.serialize_entry("type", &self.kind)?;
        object.serialize_entry("offset", &self.offset)?;
        object.serialize_entry("length", &self.length)?;
        if let Some(url) = self.url.as_ref() {
            object.serialize_entry("url", url)?;
        }
        if let Some(user) = self.user.as_ref() {
            object.serialize_entry("user", &MessageEntityUser(user))?;
        }
        if let Some(language) = self.language.as_ref() {
            object.serialize_entry("language", language)?;
        }
        if let Some(custom_emoji_id) = self.custom_emoji_id.as_ref() {
            object.serialize_entry("custom_emoji_id", custom_emoji_id)?;
        }
        if let Some(unix_time) = self.unix_time {
            object.serialize_entry("unix_time", &unix_time)?;
        }
        if let Some(date_time_format) = self.date_time_format.as_ref() {
            object.serialize_entry("date_time_format", date_time_format)?;
        }
        object.end()
    }
}

/// Telegram photo size object.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct PhotoSize {
    pub file_id: String,
    pub file_unique_id: String,
    pub width: u32,
    pub height: u32,
    #[serde(default)]
    pub file_size: Option<u64>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram forwarded message origin sent by a user.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct MessageOriginUser {
    pub date: i64,
    pub sender_user: User,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram forwarded message origin sent by a hidden user.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct MessageOriginHiddenUser {
    pub date: i64,
    pub sender_user_name: String,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram forwarded message origin sent on behalf of a chat.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct MessageOriginChat {
    pub date: i64,
    pub sender_chat: Chat,
    #[serde(default)]
    pub author_signature: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram forwarded message origin sent from a channel.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct MessageOriginChannel {
    pub date: i64,
    pub chat: Chat,
    pub message_id: MessageId,
    #[serde(default)]
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

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{Chat, ChatType, MessageEntity, MessageEntityKind, MessageOrigin};

    #[test]
    fn chat_type_preserves_unknown_values() -> Result<(), Box<dyn std::error::Error>> {
        let chat: Chat = serde_json::from_value(json!({
            "id": 1,
            "type": "future_chat",
            "title": "Official",
            "future": {"kept": true}
        }))?;

        assert_eq!(chat.kind, ChatType::Unknown("future_chat".to_owned()));
        assert_eq!(chat.kind.as_str(), "future_chat");
        assert!(!chat.is_private());
        assert!(!chat.is_group_chat());
        assert_eq!(chat.extra["future"], json!({"kept": true}));

        Ok(())
    }

    #[test]
    fn chat_type_known_values_parse_from_strings() -> Result<(), Box<dyn std::error::Error>> {
        let chat_type: ChatType = serde_json::from_value(json!("supergroup"))?;

        assert_eq!(chat_type, ChatType::Supergroup);
        assert_eq!(chat_type.as_str(), "supergroup");
        Ok(())
    }

    #[test]
    fn message_entity_response_extra_is_not_reserialized() -> Result<(), Box<dyn std::error::Error>>
    {
        let mut entity: MessageEntity = serde_json::from_value(json!({
            "type": "text_link",
            "offset": 3,
            "length": 5,
            "url": "https://example.com",
            "future": {"kept": true}
        }))?;
        assert_eq!(entity.kind, MessageEntityKind::TextLink);

        entity.extra.insert("type".to_owned(), json!("bold"));
        entity.extra.insert("offset".to_owned(), json!(0));
        entity.extra.insert("length".to_owned(), json!(1));
        entity
            .extra
            .insert("url".to_owned(), json!("https://example.com/overridden"));
        entity
            .extra
            .insert("another_future".to_owned(), json!("kept"));

        let value = serde_json::to_value(entity)?;
        assert_eq!(value["type"], "text_link");
        assert_eq!(value["offset"], 3);
        assert_eq!(value["length"], 5);
        assert_eq!(value["url"], "https://example.com");
        assert!(value.get("future").is_none());
        assert!(value.get("another_future").is_none());

        Ok(())
    }

    #[test]
    fn message_entity_text_mention_serializes_user_without_response_extra()
    -> Result<(), Box<dyn std::error::Error>> {
        let entity: MessageEntity = serde_json::from_value(json!({
            "type": "text_mention",
            "offset": 0,
            "length": 5,
            "user": {
                "id": 42,
                "is_bot": false,
                "first_name": "Alice",
                "future_user_field": "response-only"
            }
        }))?;

        let value = serde_json::to_value(entity)?;
        assert_eq!(value["type"], "text_mention");
        assert_eq!(value["user"]["id"], 42);
        assert_eq!(value["user"]["first_name"], "Alice");
        assert!(value["user"].get("future_user_field").is_none());

        Ok(())
    }

    #[test]
    fn message_origin_preserves_future_fields() -> Result<(), Box<dyn std::error::Error>> {
        let user_origin: MessageOrigin = serde_json::from_value(json!({
            "type": "user",
            "date": 10,
            "sender_user": {"id": 1, "is_bot": false, "first_name": "Alice"},
            "future": {"kept": true}
        }))?;
        let MessageOrigin::User(user) = &user_origin else {
            return Err(std::io::Error::other("expected user origin").into());
        };
        assert_eq!(user_origin.kind(), Some("user"));
        assert_eq!(user.date, 10);
        assert_eq!(user.sender_user.id.0, 1);
        assert_eq!(user.sender_user.first_name, "Alice");
        assert_eq!(user.extra["future"], json!({"kept": true}));

        let channel_origin: MessageOrigin = serde_json::from_value(json!({
            "type": "channel",
            "date": 20,
            "chat": {"id": -100, "type": "channel", "title": "News"},
            "message_id": 42,
            "author_signature": "editor",
            "future": {"kept": true}
        }))?;
        let MessageOrigin::Channel(channel) = &channel_origin else {
            return Err(std::io::Error::other("expected channel origin").into());
        };
        assert_eq!(channel_origin.kind(), Some("channel"));
        assert_eq!(channel.date, 20);
        assert_eq!(channel.chat.id, -100);
        assert_eq!(channel.chat.kind, ChatType::Channel);
        assert_eq!(channel.message_id.0, 42);
        assert_eq!(channel.author_signature.as_deref(), Some("editor"));
        assert_eq!(channel.extra["future"], json!({"kept": true}));

        Ok(())
    }
}

use std::collections::BTreeMap;

use serde::ser::SerializeMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::bot::User;
use crate::types::common::MessageId;
use crate::types::extra::{
    field_len as extra_field_len, serialize_fields as serialize_extra_fields,
};
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
#[derive(Clone, Debug, Deserialize)]
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

impl Serialize for Chat {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = [
            "id",
            "type",
            "title",
            "username",
            "first_name",
            "last_name",
            "is_forum",
        ];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let optional_len = usize::from(self.title.is_some())
            + usize::from(self.username.is_some())
            + usize::from(self.first_name.is_some())
            + usize::from(self.last_name.is_some())
            + usize::from(self.is_forum.is_some());
        let mut object = serializer.serialize_map(Some(extra_len + optional_len + 2))?;
        object.serialize_entry("id", &self.id)?;
        object.serialize_entry("type", &self.kind)?;
        if let Some(title) = self.title.as_ref() {
            object.serialize_entry("title", title)?;
        }
        if let Some(username) = self.username.as_ref() {
            object.serialize_entry("username", username)?;
        }
        if let Some(first_name) = self.first_name.as_ref() {
            object.serialize_entry("first_name", first_name)?;
        }
        if let Some(last_name) = self.last_name.as_ref() {
            object.serialize_entry("last_name", last_name)?;
        }
        if let Some(is_forum) = self.is_forum {
            object.serialize_entry("is_forum", &is_forum)?;
        }
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
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

impl Serialize for MessageEntity {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = [
            "type",
            "offset",
            "length",
            "url",
            "user",
            "language",
            "custom_emoji_id",
            "unix_time",
            "date_time_format",
        ];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let optional_len = usize::from(self.url.is_some())
            + usize::from(self.user.is_some())
            + usize::from(self.language.is_some())
            + usize::from(self.custom_emoji_id.is_some())
            + usize::from(self.unix_time.is_some())
            + usize::from(self.date_time_format.is_some());
        let mut object = serializer.serialize_map(Some(extra_len + optional_len + 3))?;
        object.serialize_entry("type", &self.kind)?;
        object.serialize_entry("offset", &self.offset)?;
        object.serialize_entry("length", &self.length)?;
        if let Some(url) = self.url.as_ref() {
            object.serialize_entry("url", url)?;
        }
        if let Some(user) = self.user.as_ref() {
            object.serialize_entry("user", user)?;
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
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
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
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct MessageOriginUser {
    pub date: i64,
    pub sender_user: User,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl Serialize for MessageOriginUser {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = ["type", "date", "sender_user"];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let mut object = serializer.serialize_map(Some(extra_len + 2))?;
        object.serialize_entry("date", &self.date)?;
        object.serialize_entry("sender_user", &self.sender_user)?;
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
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

impl Serialize for MessageOriginHiddenUser {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = ["type", "date", "sender_user_name"];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let mut object = serializer.serialize_map(Some(extra_len + 2))?;
        object.serialize_entry("date", &self.date)?;
        object.serialize_entry("sender_user_name", &self.sender_user_name)?;
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
}

/// Telegram forwarded message origin sent on behalf of a chat.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct MessageOriginChat {
    pub date: i64,
    pub sender_chat: Chat,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author_signature: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl Serialize for MessageOriginChat {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = ["type", "date", "sender_chat", "author_signature"];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let optional_len = usize::from(self.author_signature.is_some());
        let mut object = serializer.serialize_map(Some(extra_len + optional_len + 2))?;
        object.serialize_entry("date", &self.date)?;
        object.serialize_entry("sender_chat", &self.sender_chat)?;
        if let Some(author_signature) = self.author_signature.as_ref() {
            object.serialize_entry("author_signature", author_signature)?;
        }
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
}

/// Telegram forwarded message origin sent from a channel.
#[derive(Clone, Debug, Deserialize)]
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

impl Serialize for MessageOriginChannel {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = ["type", "date", "chat", "message_id", "author_signature"];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let optional_len = usize::from(self.author_signature.is_some());
        let mut object = serializer.serialize_map(Some(extra_len + optional_len + 3))?;
        object.serialize_entry("date", &self.date)?;
        object.serialize_entry("chat", &self.chat)?;
        object.serialize_entry("message_id", &self.message_id)?;
        if let Some(author_signature) = self.author_signature.as_ref() {
            object.serialize_entry("author_signature", author_signature)?;
        }
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
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

    use super::{
        Chat, ChatType, MessageEntity, MessageEntityKind, MessageOrigin, MessageOriginChannel,
        MessageOriginUser,
    };

    #[test]
    fn chat_type_preserves_unknown_values() -> Result<(), Box<dyn std::error::Error>> {
        let mut chat: Chat = serde_json::from_value(json!({
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

        chat.extra.insert("id".to_owned(), json!(9));
        chat.extra.insert("type".to_owned(), json!("private"));
        chat.extra.insert("title".to_owned(), json!("Overridden"));
        chat.extra
            .insert("another_future".to_owned(), json!("kept"));

        let value = serde_json::to_value(chat)?;
        assert_eq!(value["id"], 1);
        assert_eq!(value["type"], "future_chat");
        assert_eq!(value["title"], "Official");
        assert_eq!(value["future"], json!({"kept": true}));
        assert_eq!(value["another_future"], "kept");
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

    #[test]
    fn message_entity_extra_cannot_override_reserved_fields()
    -> Result<(), Box<dyn std::error::Error>> {
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
        assert_eq!(value["future"], json!({"kept": true}));
        assert_eq!(value["another_future"], "kept");

        Ok(())
    }

    #[test]
    fn message_origin_extra_cannot_override_reserved_fields()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut user_origin: MessageOriginUser = serde_json::from_value(json!({
            "date": 10,
            "sender_user": {"id": 1, "is_bot": false, "first_name": "Alice"},
            "future": {"kept": true}
        }))?;
        user_origin
            .extra
            .insert("type".to_owned(), json!("channel"));
        user_origin.extra.insert("date".to_owned(), json!(1));
        user_origin.extra.insert(
            "sender_user".to_owned(),
            json!({"id": 9, "is_bot": true, "first_name": "spoofed"}),
        );
        user_origin
            .extra
            .insert("another_future".to_owned(), json!("kept"));

        let user_value = serde_json::to_value(MessageOrigin::User(user_origin))?;
        assert_eq!(user_value["type"], "user");
        assert_eq!(user_value["date"], 10);
        assert_eq!(user_value["sender_user"]["id"], 1);
        assert_eq!(user_value["sender_user"]["first_name"], "Alice");
        assert_eq!(user_value["future"], json!({"kept": true}));
        assert_eq!(user_value["another_future"], "kept");

        let mut channel_origin: MessageOriginChannel = serde_json::from_value(json!({
            "date": 20,
            "chat": {"id": -100, "type": "channel", "title": "News"},
            "message_id": 42,
            "author_signature": "editor",
            "future": {"kept": true}
        }))?;
        channel_origin
            .extra
            .insert("type".to_owned(), json!("user"));
        channel_origin.extra.insert("date".to_owned(), json!(2));
        channel_origin.extra.insert(
            "chat".to_owned(),
            json!({"id": 1, "type": "private", "first_name": "spoofed"}),
        );
        channel_origin
            .extra
            .insert("message_id".to_owned(), json!(1));
        channel_origin
            .extra
            .insert("author_signature".to_owned(), json!("spoofed"));
        channel_origin
            .extra
            .insert("another_future".to_owned(), json!("kept"));

        let channel_value = serde_json::to_value(MessageOrigin::Channel(Box::new(channel_origin)))?;
        assert_eq!(channel_value["type"], "channel");
        assert_eq!(channel_value["date"], 20);
        assert_eq!(channel_value["chat"]["id"], -100);
        assert_eq!(channel_value["chat"]["type"], "channel");
        assert_eq!(channel_value["message_id"], 42);
        assert_eq!(channel_value["author_signature"], "editor");
        assert_eq!(channel_value["future"], json!({"kept": true}));
        assert_eq!(channel_value["another_future"], "kept");

        Ok(())
    }
}

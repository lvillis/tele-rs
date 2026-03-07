use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::Error;
use crate::types::bot::User;
use crate::types::common::{ChatId, MessageId, ParseMode, UserId};
use crate::types::sticker::Sticker;
use crate::types::telegram::{LinkPreviewOptions, ReplyMarkup, ReplyParameters, WebAppData};

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

/// Telegram animation file object.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Animation {
    pub file_id: String,
    pub file_unique_id: String,
    pub width: u32,
    pub height: u32,
    pub duration: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<PhotoSize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_size: Option<u64>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram audio file object.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Audio {
    pub file_id: String,
    pub file_unique_id: String,
    pub duration: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub performer: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_size: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<PhotoSize>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram generic document object.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Document {
    pub file_id: String,
    pub file_unique_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<PhotoSize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_size: Option<u64>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram story object.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Story {
    pub chat: Chat,
    pub id: i64,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram video quality descriptor.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct VideoQuality {
    pub file_id: String,
    pub file_unique_id: String,
    pub width: u32,
    pub height: u32,
    pub codec: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_size: Option<u64>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram video object.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Video {
    pub file_id: String,
    pub file_unique_id: String,
    pub width: u32,
    pub height: u32,
    pub duration: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<PhotoSize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cover: Option<Vec<PhotoSize>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_timestamp: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub qualities: Option<Vec<VideoQuality>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_size: Option<u64>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram video note object.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct VideoNote {
    pub file_id: String,
    pub file_unique_id: String,
    pub length: u32,
    pub duration: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<PhotoSize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_size: Option<u64>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram voice note object.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Voice {
    pub file_id: String,
    pub file_unique_id: String,
    pub duration: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_size: Option<u64>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram phone contact object.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Contact {
    pub phone_number: String,
    pub first_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_id: Option<UserId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vcard: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram animated dice object.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Dice {
    pub emoji: DiceEmoji,
    pub value: u8,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram geographic location object.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Location {
    pub latitude: f64,
    pub longitude: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub horizontal_accuracy: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub live_period: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub heading: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proximity_alert_radius: Option<u32>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram venue object.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Venue {
    pub location: Location,
    pub title: String,
    pub address: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub foursquare_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub foursquare_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub google_place_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub google_place_type: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
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

/// Telegram poll type.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum PollKind {
    Regular,
    Quiz,
    Unknown(String),
}

impl PollKind {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Regular => "regular",
            Self::Quiz => "quiz",
            Self::Unknown(kind) => kind.as_str(),
        }
    }
}

impl<'de> Deserialize<'de> for PollKind {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let kind = String::deserialize(deserializer)?;
        Ok(match kind.as_str() {
            "regular" => Self::Regular,
            "quiz" => Self::Quiz,
            _ => Self::Unknown(kind),
        })
    }
}

impl Serialize for PollKind {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

/// Telegram poll option.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct PollOption {
    pub text: String,
    pub voter_count: u64,
}

/// Telegram poll object.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Poll {
    pub id: String,
    pub question: String,
    pub options: Vec<PollOption>,
    pub total_voter_count: u64,
    pub is_closed: bool,
    pub is_anonymous: bool,
    #[serde(rename = "type")]
    pub kind: PollKind,
    pub allows_multiple_answers: bool,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram write access allowed service payload.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct WriteAccessAllowed {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_request: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub web_app_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_attachment_menu: Option<bool>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Classified message payload kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[non_exhaustive]
pub enum MessageKind {
    WriteAccessAllowed,
    WebAppData,
    Poll,
    Animation,
    Audio,
    Contact,
    Dice,
    Document,
    Location,
    Photo,
    Sticker,
    Story,
    Venue,
    Video,
    VideoNote,
    Voice,
    Text,
    Caption,
    Unknown,
}

const KNOWN_MESSAGE_KINDS: [MessageKind; 18] = [
    MessageKind::WriteAccessAllowed,
    MessageKind::WebAppData,
    MessageKind::Poll,
    MessageKind::Animation,
    MessageKind::Audio,
    MessageKind::Contact,
    MessageKind::Dice,
    MessageKind::Document,
    MessageKind::Location,
    MessageKind::Photo,
    MessageKind::Sticker,
    MessageKind::Story,
    MessageKind::Venue,
    MessageKind::Video,
    MessageKind::VideoNote,
    MessageKind::Voice,
    MessageKind::Text,
    MessageKind::Caption,
];

/// Telegram message object.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Message {
    pub message_id: MessageId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from: Option<User>,
    pub chat: Chat,
    pub date: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub forward_origin: Option<MessageOrigin>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub is_automatic_forward: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entities: Option<Vec<MessageEntity>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caption_entities: Option<Vec<MessageEntity>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub animation: Option<Animation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio: Option<Audio>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contact: Option<Contact>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dice: Option<Dice>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub document: Option<Document>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<Location>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub photo: Option<Vec<PhotoSize>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sticker: Option<Sticker>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub story: Option<Story>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub venue: Option<Venue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub video: Option<Video>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub video_note: Option<VideoNote>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub voice: Option<Voice>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub poll: Option<Poll>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub web_app_data: Option<WebAppData>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub write_access_allowed: Option<WriteAccessAllowed>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub edit_date: Option<i64>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

const fn is_false(value: &bool) -> bool {
    !*value
}

fn is_unmodeled_message_content_key(key: &str) -> bool {
    matches!(
        key,
        "game"
            | "invoice"
            | "new_chat_members"
            | "left_chat_member"
            | "new_chat_title"
            | "new_chat_photo"
            | "delete_chat_photo"
            | "group_chat_created"
            | "supergroup_chat_created"
            | "channel_chat_created"
            | "message_auto_delete_timer_changed"
            | "migrate_to_chat_id"
            | "migrate_from_chat_id"
            | "pinned_message"
            | "successful_payment"
            | "users_shared"
            | "chat_shared"
            | "forum_topic_created"
            | "forum_topic_edited"
            | "forum_topic_closed"
            | "forum_topic_reopened"
            | "general_forum_topic_hidden"
            | "general_forum_topic_unhidden"
    )
}

impl Message {
    pub fn chat(&self) -> &Chat {
        &self.chat
    }

    pub fn from_user(&self) -> Option<&User> {
        self.from.as_ref()
    }

    fn has_modeled_kind(&self) -> bool {
        self.write_access_allowed.is_some()
            || self.web_app_data.is_some()
            || self.poll.is_some()
            || self.animation.is_some()
            || self.audio.is_some()
            || self.contact.is_some()
            || self.dice.is_some()
            || self.document.is_some()
            || self.location.is_some()
            || self.photo.is_some()
            || self.sticker.is_some()
            || self.story.is_some()
            || self.venue.is_some()
            || self.video.is_some()
            || self.video_note.is_some()
            || self.voice.is_some()
            || self.text.is_some()
            || self.caption.is_some()
    }

    fn has_unmodeled_content(&self) -> bool {
        self.extra
            .keys()
            .any(|key| is_unmodeled_message_content_key(key))
    }

    pub fn web_app_data(&self) -> Option<&WebAppData> {
        self.web_app_data.as_ref()
    }

    pub fn write_access_allowed(&self) -> Option<&WriteAccessAllowed> {
        self.write_access_allowed.as_ref()
    }

    pub fn forward_origin(&self) -> Option<&MessageOrigin> {
        self.forward_origin.as_ref()
    }

    pub fn is_automatic_forward(&self) -> bool {
        self.is_automatic_forward
    }

    /// Returns the primary message kind using stable precedence.
    pub fn kind(&self) -> MessageKind {
        for kind in KNOWN_MESSAGE_KINDS {
            if self.has_kind(kind) {
                return kind;
            }
        }

        MessageKind::Unknown
    }

    /// Returns all detected kinds for this message.
    pub fn kinds(&self) -> Vec<MessageKind> {
        let mut kinds = Vec::with_capacity(KNOWN_MESSAGE_KINDS.len() + 1);
        for kind in KNOWN_MESSAGE_KINDS {
            if self.has_kind(kind) {
                kinds.push(kind);
            }
        }

        if self.has_kind(MessageKind::Unknown) {
            kinds.push(MessageKind::Unknown);
        }

        kinds
    }

    /// Returns whether this message contains the given kind.
    pub fn has_kind(&self, kind: MessageKind) -> bool {
        match kind {
            MessageKind::WriteAccessAllowed => self.write_access_allowed.is_some(),
            MessageKind::WebAppData => self.web_app_data.is_some(),
            MessageKind::Poll => self.poll.is_some(),
            MessageKind::Animation => self.animation.is_some(),
            MessageKind::Audio => self.audio.is_some(),
            MessageKind::Contact => self.contact.is_some(),
            MessageKind::Dice => self.dice.is_some(),
            MessageKind::Document => self.document.is_some(),
            MessageKind::Location => self.location.is_some(),
            MessageKind::Photo => self.photo.is_some(),
            MessageKind::Sticker => self.sticker.is_some(),
            MessageKind::Story => self.story.is_some(),
            MessageKind::Venue => self.venue.is_some(),
            MessageKind::Video => self.video.is_some(),
            MessageKind::VideoNote => self.video_note.is_some(),
            MessageKind::Voice => self.voice.is_some(),
            MessageKind::Text => self.text.is_some(),
            MessageKind::Caption => self.caption.is_some(),
            MessageKind::Unknown => self.has_unmodeled_content() || !self.has_modeled_kind(),
        }
    }
}

/// `sendMessage` request.
#[derive(Clone, Debug, Serialize)]
pub struct SendMessageRequest {
    pub chat_id: ChatId,
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parse_mode: Option<ParseMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_web_page_preview: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_notification: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protect_content: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_thread_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_parameters: Option<ReplyParameters>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_markup: Option<ReplyMarkup>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub link_preview_options: Option<LinkPreviewOptions>,
}

impl SendMessageRequest {
    pub fn new(chat_id: impl Into<ChatId>, text: impl Into<String>) -> Result<Self, Error> {
        let text = text.into();
        if text.trim().is_empty() {
            return Err(Error::InvalidRequest {
                reason: "sendMessage requires non-empty text".to_owned(),
            });
        }

        Ok(Self {
            chat_id: chat_id.into(),
            text,
            parse_mode: None,
            disable_web_page_preview: None,
            disable_notification: None,
            protect_content: None,
            message_thread_id: None,
            reply_parameters: None,
            reply_markup: None,
            link_preview_options: None,
        })
    }

    pub fn parse_mode(mut self, parse_mode: ParseMode) -> Self {
        self.parse_mode = Some(parse_mode);
        self
    }
}

/// `forwardMessage` request.
#[derive(Clone, Debug, Serialize)]
pub struct ForwardMessageRequest {
    pub chat_id: ChatId,
    pub from_chat_id: ChatId,
    pub message_id: MessageId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_thread_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_notification: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protect_content: Option<bool>,
}

impl ForwardMessageRequest {
    pub fn new(
        chat_id: impl Into<ChatId>,
        from_chat_id: impl Into<ChatId>,
        message_id: MessageId,
    ) -> Self {
        Self {
            chat_id: chat_id.into(),
            from_chat_id: from_chat_id.into(),
            message_id,
            message_thread_id: None,
            disable_notification: None,
            protect_content: None,
        }
    }
}

/// `copyMessage` request.
#[derive(Clone, Debug, Serialize)]
pub struct CopyMessageRequest {
    pub chat_id: ChatId,
    pub from_chat_id: ChatId,
    pub message_id: MessageId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_thread_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parse_mode: Option<ParseMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_notification: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protect_content: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_parameters: Option<ReplyParameters>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_markup: Option<ReplyMarkup>,
}

impl CopyMessageRequest {
    pub fn new(
        chat_id: impl Into<ChatId>,
        from_chat_id: impl Into<ChatId>,
        message_id: MessageId,
    ) -> Self {
        Self {
            chat_id: chat_id.into(),
            from_chat_id: from_chat_id.into(),
            message_id,
            message_thread_id: None,
            caption: None,
            parse_mode: None,
            disable_notification: None,
            protect_content: None,
            reply_parameters: None,
            reply_markup: None,
        }
    }
}

/// `copyMessages` request.
#[derive(Clone, Debug, Serialize)]
pub struct CopyMessagesRequest {
    pub chat_id: ChatId,
    pub from_chat_id: ChatId,
    pub message_ids: Vec<MessageId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_thread_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_notification: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protect_content: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remove_caption: Option<bool>,
}

impl CopyMessagesRequest {
    pub fn new(
        chat_id: impl Into<ChatId>,
        from_chat_id: impl Into<ChatId>,
        message_ids: Vec<MessageId>,
    ) -> Result<Self, Error> {
        if message_ids.is_empty() {
            return Err(Error::InvalidRequest {
                reason: "copyMessages requires at least one message id".to_owned(),
            });
        }

        Ok(Self {
            chat_id: chat_id.into(),
            from_chat_id: from_chat_id.into(),
            message_ids,
            message_thread_id: None,
            disable_notification: None,
            protect_content: None,
            remove_caption: None,
        })
    }
}

/// Telegram message id response object.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct MessageIdObject {
    pub message_id: MessageId,
}

/// Telegram `answerWebAppQuery` response object.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct SentWebAppMessage {
    pub inline_message_id: String,
}

/// `sendPhoto` request.
#[derive(Clone, Debug, Serialize)]
pub struct SendPhotoRequest {
    pub chat_id: ChatId,
    pub photo: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parse_mode: Option<ParseMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_spoiler: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_notification: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protect_content: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_thread_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_parameters: Option<ReplyParameters>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_markup: Option<ReplyMarkup>,
}

impl SendPhotoRequest {
    pub fn new(chat_id: impl Into<ChatId>, photo: impl Into<String>) -> Self {
        Self {
            chat_id: chat_id.into(),
            photo: photo.into(),
            caption: None,
            parse_mode: None,
            has_spoiler: None,
            disable_notification: None,
            protect_content: None,
            message_thread_id: None,
            reply_parameters: None,
            reply_markup: None,
        }
    }
}

/// `sendAudio` request.
#[derive(Clone, Debug, Serialize)]
pub struct SendAudioRequest {
    pub chat_id: ChatId,
    pub audio: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parse_mode: Option<ParseMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub performer: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_notification: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protect_content: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_thread_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_parameters: Option<ReplyParameters>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_markup: Option<ReplyMarkup>,
}

impl SendAudioRequest {
    pub fn new(chat_id: impl Into<ChatId>, audio: impl Into<String>) -> Self {
        Self {
            chat_id: chat_id.into(),
            audio: audio.into(),
            caption: None,
            parse_mode: None,
            duration: None,
            performer: None,
            title: None,
            thumbnail: None,
            disable_notification: None,
            protect_content: None,
            message_thread_id: None,
            reply_parameters: None,
            reply_markup: None,
        }
    }
}

/// `sendDocument` request.
#[derive(Clone, Debug, Serialize)]
pub struct SendDocumentRequest {
    pub chat_id: ChatId,
    pub document: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parse_mode: Option<ParseMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_content_type_detection: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_notification: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protect_content: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_thread_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_parameters: Option<ReplyParameters>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_markup: Option<ReplyMarkup>,
}

impl SendDocumentRequest {
    pub fn new(chat_id: impl Into<ChatId>, document: impl Into<String>) -> Self {
        Self {
            chat_id: chat_id.into(),
            document: document.into(),
            thumbnail: None,
            caption: None,
            parse_mode: None,
            disable_content_type_detection: None,
            disable_notification: None,
            protect_content: None,
            message_thread_id: None,
            reply_parameters: None,
            reply_markup: None,
        }
    }
}

/// `sendVideo` request.
#[derive(Clone, Debug, Serialize)]
pub struct SendVideoRequest {
    pub chat_id: ChatId,
    pub video: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parse_mode: Option<ParseMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supports_streaming: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_spoiler: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_notification: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protect_content: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_thread_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_parameters: Option<ReplyParameters>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_markup: Option<ReplyMarkup>,
}

impl SendVideoRequest {
    pub fn new(chat_id: impl Into<ChatId>, video: impl Into<String>) -> Self {
        Self {
            chat_id: chat_id.into(),
            video: video.into(),
            duration: None,
            width: None,
            height: None,
            thumbnail: None,
            caption: None,
            parse_mode: None,
            supports_streaming: None,
            has_spoiler: None,
            disable_notification: None,
            protect_content: None,
            message_thread_id: None,
            reply_parameters: None,
            reply_markup: None,
        }
    }
}

/// `sendAnimation` request.
#[derive(Clone, Debug, Serialize)]
pub struct SendAnimationRequest {
    pub chat_id: ChatId,
    pub animation: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parse_mode: Option<ParseMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_spoiler: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_notification: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protect_content: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_thread_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_parameters: Option<ReplyParameters>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_markup: Option<ReplyMarkup>,
}

impl SendAnimationRequest {
    pub fn new(chat_id: impl Into<ChatId>, animation: impl Into<String>) -> Self {
        Self {
            chat_id: chat_id.into(),
            animation: animation.into(),
            duration: None,
            width: None,
            height: None,
            thumbnail: None,
            caption: None,
            parse_mode: None,
            has_spoiler: None,
            disable_notification: None,
            protect_content: None,
            message_thread_id: None,
            reply_parameters: None,
            reply_markup: None,
        }
    }
}

/// `sendVoice` request.
#[derive(Clone, Debug, Serialize)]
pub struct SendVoiceRequest {
    pub chat_id: ChatId,
    pub voice: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parse_mode: Option<ParseMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_notification: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protect_content: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_thread_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_parameters: Option<ReplyParameters>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_markup: Option<ReplyMarkup>,
}

impl SendVoiceRequest {
    pub fn new(chat_id: impl Into<ChatId>, voice: impl Into<String>) -> Self {
        Self {
            chat_id: chat_id.into(),
            voice: voice.into(),
            caption: None,
            parse_mode: None,
            duration: None,
            disable_notification: None,
            protect_content: None,
            message_thread_id: None,
            reply_parameters: None,
            reply_markup: None,
        }
    }
}

/// `sendVideoNote` request.
#[derive(Clone, Debug, Serialize)]
pub struct SendVideoNoteRequest {
    pub chat_id: ChatId,
    pub video_note: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub length: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_notification: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protect_content: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_thread_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_parameters: Option<ReplyParameters>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_markup: Option<ReplyMarkup>,
}

impl SendVideoNoteRequest {
    pub fn new(chat_id: impl Into<ChatId>, video_note: impl Into<String>) -> Self {
        Self {
            chat_id: chat_id.into(),
            video_note: video_note.into(),
            duration: None,
            length: None,
            thumbnail: None,
            disable_notification: None,
            protect_content: None,
            message_thread_id: None,
            reply_parameters: None,
            reply_markup: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub struct InputMediaPhoto {
    pub media: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parse_mode: Option<ParseMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_spoiler: Option<bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub struct InputMediaVideo {
    pub media: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parse_mode: Option<ParseMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supports_streaming: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_spoiler: Option<bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub struct InputMediaAnimation {
    pub media: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parse_mode: Option<ParseMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_spoiler: Option<bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub struct InputMediaAudio {
    pub media: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parse_mode: Option<ParseMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub performer: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub struct InputMediaDocument {
    pub media: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parse_mode: Option<ParseMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_content_type_detection: Option<bool>,
}

/// Input media objects for `sendMediaGroup`.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InputMedia {
    Photo(Box<InputMediaPhoto>),
    Video(Box<InputMediaVideo>),
    Animation(Box<InputMediaAnimation>),
    Audio(Box<InputMediaAudio>),
    Document(Box<InputMediaDocument>),
}

impl From<InputMediaPhoto> for InputMedia {
    fn from(value: InputMediaPhoto) -> Self {
        Self::Photo(Box::new(value))
    }
}

impl From<InputMediaVideo> for InputMedia {
    fn from(value: InputMediaVideo) -> Self {
        Self::Video(Box::new(value))
    }
}

impl From<InputMediaAnimation> for InputMedia {
    fn from(value: InputMediaAnimation) -> Self {
        Self::Animation(Box::new(value))
    }
}

impl From<InputMediaAudio> for InputMedia {
    fn from(value: InputMediaAudio) -> Self {
        Self::Audio(Box::new(value))
    }
}

impl From<InputMediaDocument> for InputMedia {
    fn from(value: InputMediaDocument) -> Self {
        Self::Document(Box::new(value))
    }
}

/// `sendMediaGroup` request.
#[derive(Clone, Debug, Serialize)]
pub struct SendMediaGroupRequest {
    pub chat_id: ChatId,
    pub media: Vec<InputMedia>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_notification: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protect_content: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_thread_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_parameters: Option<ReplyParameters>,
}

impl SendMediaGroupRequest {
    pub fn new(chat_id: impl Into<ChatId>, media: Vec<InputMedia>) -> Result<Self, Error> {
        if media.is_empty() {
            return Err(Error::InvalidRequest {
                reason: "sendMediaGroup requires at least one media item".to_owned(),
            });
        }

        Ok(Self {
            chat_id: chat_id.into(),
            media,
            disable_notification: None,
            protect_content: None,
            message_thread_id: None,
            reply_parameters: None,
        })
    }
}

/// `sendLocation` request.
#[derive(Clone, Debug, Serialize)]
pub struct SendLocationRequest {
    pub chat_id: ChatId,
    pub latitude: f64,
    pub longitude: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub horizontal_accuracy: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub live_period: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub heading: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proximity_alert_radius: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_notification: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protect_content: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_thread_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_parameters: Option<ReplyParameters>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_markup: Option<ReplyMarkup>,
}

impl SendLocationRequest {
    pub fn new(chat_id: impl Into<ChatId>, latitude: f64, longitude: f64) -> Self {
        Self {
            chat_id: chat_id.into(),
            latitude,
            longitude,
            horizontal_accuracy: None,
            live_period: None,
            heading: None,
            proximity_alert_radius: None,
            disable_notification: None,
            protect_content: None,
            message_thread_id: None,
            reply_parameters: None,
            reply_markup: None,
        }
    }
}

/// `sendVenue` request.
#[derive(Clone, Debug, Serialize)]
pub struct SendVenueRequest {
    pub chat_id: ChatId,
    pub latitude: f64,
    pub longitude: f64,
    pub title: String,
    pub address: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub foursquare_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub foursquare_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub google_place_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub google_place_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_notification: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protect_content: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_thread_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_parameters: Option<ReplyParameters>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_markup: Option<ReplyMarkup>,
}

impl SendVenueRequest {
    pub fn new(
        chat_id: impl Into<ChatId>,
        latitude: f64,
        longitude: f64,
        title: impl Into<String>,
        address: impl Into<String>,
    ) -> Self {
        Self {
            chat_id: chat_id.into(),
            latitude,
            longitude,
            title: title.into(),
            address: address.into(),
            foursquare_id: None,
            foursquare_type: None,
            google_place_id: None,
            google_place_type: None,
            disable_notification: None,
            protect_content: None,
            message_thread_id: None,
            reply_parameters: None,
            reply_markup: None,
        }
    }
}

/// `sendContact` request.
#[derive(Clone, Debug, Serialize)]
pub struct SendContactRequest {
    pub chat_id: ChatId,
    pub phone_number: String,
    pub first_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vcard: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_notification: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protect_content: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_thread_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_parameters: Option<ReplyParameters>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_markup: Option<ReplyMarkup>,
}

impl SendContactRequest {
    pub fn new(
        chat_id: impl Into<ChatId>,
        phone_number: impl Into<String>,
        first_name: impl Into<String>,
    ) -> Self {
        Self {
            chat_id: chat_id.into(),
            phone_number: phone_number.into(),
            first_name: first_name.into(),
            last_name: None,
            vcard: None,
            disable_notification: None,
            protect_content: None,
            message_thread_id: None,
            reply_parameters: None,
            reply_markup: None,
        }
    }
}

/// `sendPoll` request.
#[derive(Clone, Debug, Serialize)]
pub struct SendPollRequest {
    pub chat_id: ChatId,
    pub question: String,
    pub options: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_anonymous: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allows_multiple_answers: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub correct_option_id: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub explanation: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub explanation_parse_mode: Option<ParseMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub open_period: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub close_date: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_closed: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_notification: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protect_content: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_thread_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_parameters: Option<ReplyParameters>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_markup: Option<ReplyMarkup>,
}

impl SendPollRequest {
    pub fn new(
        chat_id: impl Into<ChatId>,
        question: impl Into<String>,
        options: Vec<String>,
    ) -> Result<Self, Error> {
        if options.len() < 2 {
            return Err(Error::InvalidRequest {
                reason: "sendPoll requires at least two options".to_owned(),
            });
        }

        Ok(Self {
            chat_id: chat_id.into(),
            question: question.into(),
            options,
            is_anonymous: None,
            r#type: None,
            allows_multiple_answers: None,
            correct_option_id: None,
            explanation: None,
            explanation_parse_mode: None,
            open_period: None,
            close_date: None,
            is_closed: None,
            disable_notification: None,
            protect_content: None,
            message_thread_id: None,
            reply_parameters: None,
            reply_markup: None,
        })
    }
}

/// `stopPoll` request.
#[derive(Clone, Debug, Serialize)]
pub struct StopPollRequest {
    pub chat_id: ChatId,
    pub message_id: MessageId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_markup: Option<ReplyMarkup>,
}

impl StopPollRequest {
    pub fn new(chat_id: impl Into<ChatId>, message_id: MessageId) -> Self {
        Self {
            chat_id: chat_id.into(),
            message_id,
            reply_markup: None,
        }
    }
}

/// Dice emoji.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum DiceEmoji {
    #[serde(rename = "🎲")]
    Dice,
    #[serde(rename = "🎯")]
    Darts,
    #[serde(rename = "🏀")]
    Basketball,
    #[serde(rename = "⚽")]
    Football,
    #[serde(rename = "🎳")]
    Bowling,
    #[serde(rename = "🎰")]
    SlotMachine,
}

/// `sendDice` request.
#[derive(Clone, Debug, Serialize)]
pub struct SendDiceRequest {
    pub chat_id: ChatId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub emoji: Option<DiceEmoji>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_notification: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protect_content: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_thread_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_parameters: Option<ReplyParameters>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_markup: Option<ReplyMarkup>,
}

impl SendDiceRequest {
    pub fn new(chat_id: impl Into<ChatId>) -> Self {
        Self {
            chat_id: chat_id.into(),
            emoji: None,
            disable_notification: None,
            protect_content: None,
            message_thread_id: None,
            reply_parameters: None,
            reply_markup: None,
        }
    }
}

/// Chat action values.
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ChatAction {
    Typing,
    UploadPhoto,
    RecordVideo,
    UploadVideo,
    RecordVoice,
    UploadVoice,
    UploadDocument,
    ChooseSticker,
    FindLocation,
    RecordVideoNote,
    UploadVideoNote,
}

/// `sendChatAction` request.
#[derive(Clone, Debug, Serialize)]
pub struct SendChatActionRequest {
    pub chat_id: ChatId,
    pub action: ChatAction,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_thread_id: Option<i64>,
}

impl SendChatActionRequest {
    pub fn new(chat_id: impl Into<ChatId>, action: ChatAction) -> Self {
        Self {
            chat_id: chat_id.into(),
            action,
            message_thread_id: None,
        }
    }
}

/// `editMessageText` request.
#[derive(Clone, Debug, Serialize)]
pub struct EditMessageTextRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chat_id: Option<ChatId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_id: Option<MessageId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inline_message_id: Option<String>,
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parse_mode: Option<ParseMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_markup: Option<ReplyMarkup>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub link_preview_options: Option<LinkPreviewOptions>,
}

impl EditMessageTextRequest {
    pub fn for_chat_message(
        chat_id: impl Into<ChatId>,
        message_id: MessageId,
        text: impl Into<String>,
    ) -> Result<Self, Error> {
        let text = text.into();
        if text.trim().is_empty() {
            return Err(Error::InvalidRequest {
                reason: "editMessageText requires non-empty text".to_owned(),
            });
        }

        Ok(Self {
            chat_id: Some(chat_id.into()),
            message_id: Some(message_id),
            inline_message_id: None,
            text,
            parse_mode: None,
            reply_markup: None,
            link_preview_options: None,
        })
    }

    pub fn for_inline_message(
        inline_message_id: impl Into<String>,
        text: impl Into<String>,
    ) -> Result<Self, Error> {
        let inline_message_id = inline_message_id.into();
        if inline_message_id.trim().is_empty() {
            return Err(Error::InvalidRequest {
                reason: "inline_message_id cannot be empty".to_owned(),
            });
        }

        let text = text.into();
        if text.trim().is_empty() {
            return Err(Error::InvalidRequest {
                reason: "editMessageText requires non-empty text".to_owned(),
            });
        }

        Ok(Self {
            chat_id: None,
            message_id: None,
            inline_message_id: Some(inline_message_id),
            text,
            parse_mode: None,
            reply_markup: None,
            link_preview_options: None,
        })
    }

    pub fn validate(&self) -> Result<(), Error> {
        validate_edit_target(
            self.chat_id.is_some() && self.message_id.is_some(),
            &self.inline_message_id,
        )?;

        if self.text.trim().is_empty() {
            return Err(Error::InvalidRequest {
                reason: "editMessageText requires non-empty text".to_owned(),
            });
        }

        Ok(())
    }
}

/// `editMessageCaption` request.
#[derive(Clone, Debug, Serialize)]
pub struct EditMessageCaptionRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chat_id: Option<ChatId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_id: Option<MessageId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inline_message_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parse_mode: Option<ParseMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_markup: Option<ReplyMarkup>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub show_caption_above_media: Option<bool>,
}

impl EditMessageCaptionRequest {
    pub fn validate(&self) -> Result<(), Error> {
        validate_edit_target(
            self.chat_id.is_some() && self.message_id.is_some(),
            &self.inline_message_id,
        )
    }
}

/// `editMessageReplyMarkup` request.
#[derive(Clone, Debug, Serialize)]
pub struct EditMessageReplyMarkupRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chat_id: Option<ChatId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_id: Option<MessageId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inline_message_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_markup: Option<ReplyMarkup>,
}

impl EditMessageReplyMarkupRequest {
    pub fn validate(&self) -> Result<(), Error> {
        validate_edit_target(
            self.chat_id.is_some() && self.message_id.is_some(),
            &self.inline_message_id,
        )
    }
}

/// `editMessageLiveLocation` request.
#[derive(Clone, Debug, Serialize)]
pub struct EditMessageLiveLocationRequest {
    pub latitude: f64,
    pub longitude: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chat_id: Option<ChatId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_id: Option<MessageId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inline_message_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub live_period: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub horizontal_accuracy: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub heading: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proximity_alert_radius: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_markup: Option<ReplyMarkup>,
}

impl EditMessageLiveLocationRequest {
    pub fn validate(&self) -> Result<(), Error> {
        validate_edit_target(
            self.chat_id.is_some() && self.message_id.is_some(),
            &self.inline_message_id,
        )
    }
}

/// `stopMessageLiveLocation` request.
#[derive(Clone, Debug, Serialize)]
pub struct StopMessageLiveLocationRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chat_id: Option<ChatId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_id: Option<MessageId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inline_message_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_markup: Option<ReplyMarkup>,
}

impl StopMessageLiveLocationRequest {
    pub fn validate(&self) -> Result<(), Error> {
        validate_edit_target(
            self.chat_id.is_some() && self.message_id.is_some(),
            &self.inline_message_id,
        )
    }
}

/// Return type for edit message methods.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum EditMessageResult {
    Message(Box<Message>),
    Success(bool),
}

impl EditMessageResult {
    pub fn message(&self) -> Option<&Message> {
        match self {
            Self::Message(message) => Some(message.as_ref()),
            Self::Success(_) => None,
        }
    }

    pub fn into_message(self) -> Option<Message> {
        match self {
            Self::Message(message) => Some(*message),
            Self::Success(_) => None,
        }
    }

    pub fn success(&self) -> Option<bool> {
        match self {
            Self::Message(_) => None,
            Self::Success(success) => Some(*success),
        }
    }
}

impl From<Message> for EditMessageResult {
    fn from(value: Message) -> Self {
        Self::Message(Box::new(value))
    }
}

/// `deleteMessage` request.
#[derive(Clone, Debug, Serialize)]
pub struct DeleteMessageRequest {
    pub chat_id: ChatId,
    pub message_id: MessageId,
}

impl DeleteMessageRequest {
    pub fn new(chat_id: impl Into<ChatId>, message_id: MessageId) -> Self {
        Self {
            chat_id: chat_id.into(),
            message_id,
        }
    }
}

/// `deleteMessages` request.
#[derive(Clone, Debug, Serialize)]
pub struct DeleteMessagesRequest {
    pub chat_id: ChatId,
    pub message_ids: Vec<MessageId>,
}

impl DeleteMessagesRequest {
    pub fn new(chat_id: impl Into<ChatId>, message_ids: Vec<MessageId>) -> Result<Self, Error> {
        if message_ids.is_empty() {
            return Err(Error::InvalidRequest {
                reason: "deleteMessages requires at least one message id".to_owned(),
            });
        }

        Ok(Self {
            chat_id: chat_id.into(),
            message_ids,
        })
    }
}

fn validate_edit_target(
    has_chat_target: bool,
    inline_message_id: &Option<String>,
) -> Result<(), Error> {
    let has_inline_target = inline_message_id
        .as_ref()
        .is_some_and(|inline_message_id| !inline_message_id.trim().is_empty());

    if has_chat_target ^ has_inline_target {
        return Ok(());
    }

    Err(Error::InvalidRequest {
        reason: "method requires either chat_id+message_id or inline_message_id".to_owned(),
    })
}

macro_rules! impl_reply_markup_setter {
    ($($ty:ty),* $(,)?) => {
        $(
            impl $ty {
                pub fn reply_markup(mut self, reply_markup: impl Into<ReplyMarkup>) -> Self {
                    self.reply_markup = Some(reply_markup.into());
                    self
                }
            }
        )*
    };
}

macro_rules! impl_reply_parameters_setter {
    ($($ty:ty),* $(,)?) => {
        $(
            impl $ty {
                pub fn reply_parameters(mut self, reply_parameters: ReplyParameters) -> Self {
                    self.reply_parameters = Some(reply_parameters);
                    self
                }

                pub fn reply_to_message(mut self, message_id: MessageId) -> Self {
                    self.reply_parameters = Some(ReplyParameters::new(message_id));
                    self
                }
            }
        )*
    };
}

macro_rules! impl_link_preview_setter {
    ($($ty:ty),* $(,)?) => {
        $(
            impl $ty {
                pub fn link_preview_options(
                    mut self,
                    link_preview_options: LinkPreviewOptions,
                ) -> Self {
                    self.link_preview_options = Some(link_preview_options);
                    self
                }

                pub fn disable_link_preview(mut self) -> Self {
                    self.link_preview_options = Some(LinkPreviewOptions::disabled());
                    self
                }
            }
        )*
    };
}

impl_reply_markup_setter!(
    SendMessageRequest,
    CopyMessageRequest,
    SendPhotoRequest,
    SendAudioRequest,
    SendDocumentRequest,
    SendVideoRequest,
    SendAnimationRequest,
    SendVoiceRequest,
    SendVideoNoteRequest,
    SendLocationRequest,
    SendVenueRequest,
    SendContactRequest,
    SendPollRequest,
    SendDiceRequest,
    StopPollRequest,
    EditMessageTextRequest,
    EditMessageCaptionRequest,
    EditMessageReplyMarkupRequest,
    EditMessageLiveLocationRequest,
    StopMessageLiveLocationRequest
);

impl_reply_parameters_setter!(
    SendMessageRequest,
    CopyMessageRequest,
    SendPhotoRequest,
    SendAudioRequest,
    SendDocumentRequest,
    SendVideoRequest,
    SendAnimationRequest,
    SendVoiceRequest,
    SendVideoNoteRequest,
    SendMediaGroupRequest,
    SendLocationRequest,
    SendVenueRequest,
    SendContactRequest,
    SendPollRequest,
    SendDiceRequest
);

impl_link_preview_setter!(SendMessageRequest, EditMessageTextRequest);

#[cfg(test)]
mod tests {
    use std::error::Error as StdError;

    use serde_json::{Value, json};

    use super::*;

    #[test]
    fn detects_primary_text_message_kind() -> std::result::Result<(), Box<dyn StdError>> {
        let message: Message = serde_json::from_value(json!({
            "message_id": 1,
            "date": 1700000000,
            "chat": {"id": 1, "type": "private"},
            "text": "hello"
        }))?;

        assert_eq!(message.kind(), MessageKind::Text);
        assert_eq!(message.kinds(), vec![MessageKind::Text]);
        assert!(message.has_kind(MessageKind::Text));
        Ok(())
    }

    #[test]
    fn includes_secondary_caption_kind() -> std::result::Result<(), Box<dyn StdError>> {
        let message: Message = serde_json::from_value(json!({
            "message_id": 2,
            "date": 1700000001,
            "chat": {"id": 1, "type": "private"},
            "photo": [{
                "file_id": "p1",
                "file_unique_id": "u1",
                "width": 16,
                "height": 16
            }],
            "caption": "preview"
        }))?;

        assert_eq!(message.kind(), MessageKind::Photo);
        assert_eq!(
            message.kinds(),
            vec![MessageKind::Photo, MessageKind::Caption]
        );
        assert!(message.has_kind(MessageKind::Caption));
        Ok(())
    }

    #[test]
    fn marks_unmodeled_content_as_unknown() -> std::result::Result<(), Box<dyn StdError>> {
        let message: Message = serde_json::from_value(json!({
            "message_id": 3,
            "date": 1700000002,
            "chat": {"id": 1, "type": "private"},
            "game": {"title": "demo"}
        }))?;

        assert_eq!(message.kind(), MessageKind::Unknown);
        assert_eq!(message.kinds(), vec![MessageKind::Unknown]);
        assert!(message.has_kind(MessageKind::Unknown));
        Ok(())
    }

    #[test]
    fn keeps_unknown_alongside_modeled_kind() -> std::result::Result<(), Box<dyn StdError>> {
        let message: Message = serde_json::from_value(json!({
            "message_id": 4,
            "date": 1700000003,
            "chat": {"id": 1, "type": "private"},
            "text": "hello",
            "game": {"title": "demo"}
        }))?;

        assert_eq!(message.kind(), MessageKind::Text);
        assert_eq!(
            message.kinds(),
            vec![MessageKind::Text, MessageKind::Unknown]
        );
        assert!(message.has_kind(MessageKind::Unknown));
        Ok(())
    }

    fn base_message_payload() -> serde_json::Map<String, Value> {
        let mut object = serde_json::Map::new();
        object.insert("message_id".to_owned(), json!(99));
        object.insert("date".to_owned(), json!(1700000999));
        object.insert("chat".to_owned(), json!({"id": 1, "type": "private"}));
        object
    }

    fn message_for_kind(kind: MessageKind) -> std::result::Result<Message, Box<dyn StdError>> {
        let mut object = base_message_payload();
        match kind {
            MessageKind::WriteAccessAllowed => {
                object.insert(
                    "write_access_allowed".to_owned(),
                    json!({"from_request": true}),
                );
            }
            MessageKind::WebAppData => {
                object.insert(
                    "web_app_data".to_owned(),
                    json!({"data": "payload", "button_text": "open"}),
                );
            }
            MessageKind::Poll => {
                object.insert(
                    "poll".to_owned(),
                    json!({
                        "id": "poll-1",
                        "question": "q?",
                        "options": [{"text": "a", "voter_count": 1}],
                        "total_voter_count": 1,
                        "is_closed": false,
                        "is_anonymous": false,
                        "type": "regular",
                        "allows_multiple_answers": false
                    }),
                );
            }
            MessageKind::Animation => {
                object.insert(
                    "animation".to_owned(),
                    json!({
                        "file_id": "anim-1",
                        "file_unique_id": "anim-u-1",
                        "width": 320,
                        "height": 240,
                        "duration": 4
                    }),
                );
            }
            MessageKind::Audio => {
                object.insert(
                    "audio".to_owned(),
                    json!({
                        "file_id": "audio-1",
                        "file_unique_id": "audio-u-1",
                        "duration": 42
                    }),
                );
            }
            MessageKind::Contact => {
                object.insert(
                    "contact".to_owned(),
                    json!({
                        "phone_number": "+123",
                        "first_name": "contact"
                    }),
                );
            }
            MessageKind::Dice => {
                object.insert(
                    "dice".to_owned(),
                    json!({
                        "emoji": "🎲",
                        "value": 6
                    }),
                );
            }
            MessageKind::Document => {
                object.insert(
                    "document".to_owned(),
                    json!({
                        "file_id": "doc-1",
                        "file_unique_id": "doc-u-1"
                    }),
                );
            }
            MessageKind::Location => {
                object.insert(
                    "location".to_owned(),
                    json!({
                        "latitude": 1.25,
                        "longitude": 103.8
                    }),
                );
            }
            MessageKind::Photo => {
                object.insert(
                    "photo".to_owned(),
                    json!([{
                        "file_id": "p-1",
                        "file_unique_id": "u-1",
                        "width": 16,
                        "height": 16
                    }]),
                );
            }
            MessageKind::Sticker => {
                object.insert(
                    "sticker".to_owned(),
                    json!({
                        "file_id": "s-1",
                        "file_unique_id": "su-1",
                        "type": "regular",
                        "width": 128,
                        "height": 128,
                        "is_animated": false,
                        "is_video": false
                    }),
                );
            }
            MessageKind::Story => {
                object.insert(
                    "story".to_owned(),
                    json!({
                        "chat": {"id": -1001, "type": "channel", "title": "stories"},
                        "id": 7
                    }),
                );
            }
            MessageKind::Venue => {
                object.insert(
                    "venue".to_owned(),
                    json!({
                        "location": {
                            "latitude": 1.25,
                            "longitude": 103.8
                        },
                        "title": "HQ",
                        "address": "Somewhere"
                    }),
                );
            }
            MessageKind::Video => {
                object.insert(
                    "video".to_owned(),
                    json!({
                        "file_id": "video-1",
                        "file_unique_id": "video-u-1",
                        "width": 640,
                        "height": 480,
                        "duration": 8
                    }),
                );
            }
            MessageKind::VideoNote => {
                object.insert(
                    "video_note".to_owned(),
                    json!({
                        "file_id": "video-note-1",
                        "file_unique_id": "video-note-u-1",
                        "length": 240,
                        "duration": 8
                    }),
                );
            }
            MessageKind::Voice => {
                object.insert(
                    "voice".to_owned(),
                    json!({
                        "file_id": "voice-1",
                        "file_unique_id": "voice-u-1",
                        "duration": 5
                    }),
                );
            }
            MessageKind::Text => {
                object.insert("text".to_owned(), json!("hello"));
            }
            MessageKind::Caption => {
                object.insert("caption".to_owned(), json!("preview"));
            }
            MessageKind::Unknown => {
                object.insert("game".to_owned(), json!({"title": "demo"}));
            }
        }

        Ok(serde_json::from_value(Value::Object(object))?)
    }

    #[test]
    fn message_kind_matrix_stays_in_sync() -> std::result::Result<(), Box<dyn StdError>> {
        for kind in KNOWN_MESSAGE_KINDS {
            let message = message_for_kind(kind)?;
            assert!(
                message.has_kind(kind),
                "missing has_kind mapping for {kind:?}"
            );
            assert!(
                message.kinds().contains(&kind),
                "missing kinds mapping for {kind:?}"
            );
        }
        Ok(())
    }

    #[test]
    fn unknown_kind_matrix_stays_in_sync() -> std::result::Result<(), Box<dyn StdError>> {
        let message = message_for_kind(MessageKind::Unknown)?;
        assert_eq!(message.kind(), MessageKind::Unknown);
        assert!(message.has_kind(MessageKind::Unknown));
        assert_eq!(message.kinds(), vec![MessageKind::Unknown]);
        Ok(())
    }

    #[test]
    fn parses_forward_origin_and_automatic_forward_flag()
    -> std::result::Result<(), Box<dyn StdError>> {
        let message: Message = serde_json::from_value(json!({
            "message_id": 42,
            "date": 1700000042,
            "chat": {"id": -1001, "type": "supergroup", "title": "mods"},
            "is_automatic_forward": true,
            "forward_origin": {
                "type": "channel",
                "date": 1700000000,
                "chat": {"id": -1002, "type": "channel", "title": "announcements"},
                "message_id": 777,
                "author_signature": "admin"
            }
        }))?;

        assert!(message.is_automatic_forward());
        let Some(origin) = message.forward_origin() else {
            return Err("missing forward origin".into());
        };
        assert_eq!(origin.date(), 1_700_000_000);
        assert_eq!(origin.sender_name(), Some("announcements"));
        assert_eq!(origin.message_id(), Some(MessageId(777)));
        assert_eq!(origin.author_signature(), Some("admin"));

        Ok(())
    }

    #[test]
    fn parses_typed_message_entity_and_poll_kinds() -> std::result::Result<(), Box<dyn StdError>> {
        let message: Message = serde_json::from_value(json!({
            "message_id": 43,
            "date": 1700000043,
            "chat": {"id": 1, "type": "private"},
            "text": "/start hello",
            "entities": [
                {"type": "bot_command", "offset": 0, "length": 6},
                {"type": "mystery_entity", "offset": 7, "length": 5}
            ],
            "poll": {
                "id": "poll-2",
                "question": "q?",
                "options": [{"text": "a", "voter_count": 1}],
                "total_voter_count": 1,
                "is_closed": false,
                "is_anonymous": false,
                "type": "quiz",
                "allows_multiple_answers": false
            }
        }))?;

        let entities = message.entities.as_ref().ok_or("missing entities")?;
        assert_eq!(entities[0].kind, MessageEntityKind::BotCommand);
        assert_eq!(
            entities[1].kind,
            MessageEntityKind::Unknown("mystery_entity".to_owned())
        );

        let poll = message.poll.as_ref().ok_or("missing poll")?;
        assert_eq!(poll.kind, PollKind::Quiz);

        Ok(())
    }

    #[test]
    fn input_media_round_trips_with_boxed_variants() -> std::result::Result<(), Box<dyn StdError>> {
        let media = InputMedia::from(InputMediaPhoto {
            media: "attach://photo".to_owned(),
            caption: Some("preview".to_owned()),
            parse_mode: Some(ParseMode::Html),
            has_spoiler: None,
        });

        let value = serde_json::to_value(&media)?;
        assert_eq!(value.get("type"), Some(&json!("photo")));
        let parsed: InputMedia = serde_json::from_value(value)?;
        let InputMedia::Photo(photo) = parsed else {
            return Err("expected photo input media".into());
        };
        assert_eq!(photo.media, "attach://photo");
        assert_eq!(photo.caption.as_deref(), Some("preview"));

        Ok(())
    }

    #[test]
    fn edit_message_result_helpers_cover_both_variants()
    -> std::result::Result<(), Box<dyn StdError>> {
        let message = message_for_kind(MessageKind::Text)?;
        let result = EditMessageResult::from(message.clone());
        assert_eq!(
            result.message().map(|message| message.message_id),
            Some(message.message_id)
        );
        assert_eq!(result.success(), None);
        assert_eq!(
            result.into_message().map(|message| message.message_id),
            Some(message.message_id)
        );

        let success = EditMessageResult::Success(true);
        assert!(success.message().is_none());
        assert_eq!(success.success(), Some(true));

        Ok(())
    }
}

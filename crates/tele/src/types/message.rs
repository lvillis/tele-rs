use std::collections::BTreeMap;

use serde::de::{DeserializeOwned, Error as DeError};
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub question_entities: Option<Vec<MessageEntity>>,
    pub options: Vec<PollOption>,
    pub total_voter_count: u64,
    pub is_closed: bool,
    pub is_anonymous: bool,
    #[serde(rename = "type")]
    pub kind: PollKind,
    pub allows_multiple_answers: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub correct_option_id: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub explanation: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub explanation_entities: Option<Vec<MessageEntity>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub open_period: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub close_date: Option<i64>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Quoted reply segment.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct TextQuote {
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entities: Option<Vec<MessageEntity>>,
    pub position: u32,
    #[serde(default, skip_serializing_if = "is_false")]
    pub is_manual: bool,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram paid media payload.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[non_exhaustive]
pub enum PaidMedia {
    Preview {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        width: Option<u32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        height: Option<u32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        duration: Option<u32>,
        #[serde(flatten)]
        extra: BTreeMap<String, Value>,
    },
    Photo {
        photo: Vec<PhotoSize>,
        #[serde(flatten)]
        extra: BTreeMap<String, Value>,
    },
    Video {
        video: Box<Video>,
        #[serde(flatten)]
        extra: BTreeMap<String, Value>,
    },
}

/// Telegram paid media info.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct PaidMediaInfo {
    pub star_count: u64,
    pub paid_media: Vec<PaidMedia>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram checklist task.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ChecklistTask {
    pub id: i64,
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_entities: Option<Vec<MessageEntity>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completed_by_user: Option<User>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completed_by_chat: Option<Chat>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completion_date: Option<i64>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram checklist payload.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Checklist {
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title_entities: Option<Vec<MessageEntity>>,
    pub tasks: Vec<ChecklistTask>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub others_can_add_tasks: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub others_can_mark_tasks_as_done: bool,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram game payload.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Game {
    pub title: String,
    pub description: String,
    pub photo: Vec<PhotoSize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_entities: Option<Vec<MessageEntity>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub animation: Option<Animation>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram service payload for checklist task state changes.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ChecklistTasksDone {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checklist_message: Option<Box<Message>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub marked_as_done_task_ids: Option<Vec<i64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub marked_as_not_done_task_ids: Option<Vec<i64>>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram service payload for checklist task additions.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ChecklistTasksAdded {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checklist_message: Option<Box<Message>>,
    pub tasks: Vec<ChecklistTask>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram service payload for a chat owner leaving.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ChatOwnerLeft {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub new_owner: Option<User>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram service payload for a chat ownership change.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ChatOwnerChanged {
    pub new_owner: User,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram invoice payload.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Invoice {
    pub title: String,
    pub description: String,
    pub start_parameter: String,
    pub currency: String,
    pub total_amount: i64,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram shipping address payload.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ShippingAddress {
    pub country_code: String,
    pub state: String,
    pub city: String,
    pub street_line1: String,
    pub street_line2: String,
    pub post_code: String,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram order info payload.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct OrderInfo {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phone_number: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shipping_address: Option<ShippingAddress>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram successful payment payload.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct SuccessfulPayment {
    pub currency: String,
    pub total_amount: i64,
    pub invoice_payload: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subscription_expiration_date: Option<i64>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub is_recurring: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub is_first_recurring: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shipping_option_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order_info: Option<OrderInfo>,
    pub telegram_payment_charge_id: String,
    pub provider_payment_charge_id: String,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram refunded payment payload.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct RefundedPayment {
    pub currency: String,
    pub total_amount: i64,
    pub invoice_payload: String,
    pub telegram_payment_charge_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_payment_charge_id: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram proximity alert payload.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ProximityAlertTriggered {
    pub traveler: User,
    pub watcher: User,
    pub distance: u32,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram chat boost added payload.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ChatBoostAdded {
    pub boost_count: u32,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram video chat scheduled payload.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct VideoChatScheduled {
    pub start_date: i64,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram video chat started payload.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct VideoChatStarted {
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram video chat ended payload.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct VideoChatEnded {
    pub duration: u32,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram video chat participants invited payload.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct VideoChatParticipantsInvited {
    pub users: Vec<User>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram paid-message price changed payload.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct PaidMessagePriceChanged {
    pub paid_message_star_count: u64,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram direct-message price changed payload.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct DirectMessagePriceChanged {
    pub are_direct_messages_enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub direct_message_star_count: Option<u64>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram giveaway created payload.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct GiveawayCreated {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prize_star_count: Option<u64>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram giveaway payload.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Giveaway {
    pub chats: Vec<Chat>,
    pub winners_selection_date: i64,
    pub winner_count: u32,
    #[serde(default, skip_serializing_if = "is_false")]
    pub only_new_members: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub has_public_winners: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prize_description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub country_codes: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prize_star_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub premium_subscription_month_count: Option<u32>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram giveaway winners payload.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct GiveawayWinners {
    pub chat: Chat,
    pub giveaway_message_id: MessageId,
    pub winners_selection_date: i64,
    pub winner_count: u32,
    pub winners: Vec<User>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub additional_chat_count: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prize_star_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub premium_subscription_month_count: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unclaimed_prize_count: Option<u32>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub only_new_members: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub was_refunded: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prize_description: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram giveaway completed payload.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct GiveawayCompleted {
    pub winner_count: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unclaimed_prize_count: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub giveaway_message: Option<Box<Message>>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub is_star_giveaway: bool,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram amount of Stars.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct StarAmount {
    pub amount: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nanostar_amount: Option<i64>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram suggested-post price payload.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct SuggestedPostPrice {
    pub currency: String,
    pub amount: i64,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram suggested-post state.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum SuggestedPostState {
    Pending,
    Approved,
    Declined,
    Unknown(String),
}

impl SuggestedPostState {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Pending => "pending",
            Self::Approved => "approved",
            Self::Declined => "declined",
            Self::Unknown(value) => value.as_str(),
        }
    }
}

impl<'de> Deserialize<'de> for SuggestedPostState {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let state = String::deserialize(deserializer)?;
        Ok(match state.as_str() {
            "pending" => Self::Pending,
            "approved" => Self::Approved,
            "declined" => Self::Declined,
            _ => Self::Unknown(state),
        })
    }
}

impl Serialize for SuggestedPostState {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

/// Telegram suggested-post info payload.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct SuggestedPostInfo {
    pub state: SuggestedPostState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub price: Option<SuggestedPostPrice>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub send_date: Option<i64>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram suggested-post approved payload.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct SuggestedPostApproved {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suggested_post_message: Option<Box<Message>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub price: Option<SuggestedPostPrice>,
    pub send_date: i64,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram suggested-post approval failed payload.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct SuggestedPostApprovalFailed {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suggested_post_message: Option<Box<Message>>,
    pub price: SuggestedPostPrice,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram suggested-post declined payload.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct SuggestedPostDeclined {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suggested_post_message: Option<Box<Message>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram suggested-post paid payload.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct SuggestedPostPaid {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suggested_post_message: Option<Box<Message>>,
    pub currency: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub amount: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub star_amount: Option<StarAmount>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram suggested-post refund reason.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum SuggestedPostRefundReason {
    PostDeleted,
    PaymentRefunded,
    Unknown(String),
}

impl SuggestedPostRefundReason {
    pub fn as_str(&self) -> &str {
        match self {
            Self::PostDeleted => "post_deleted",
            Self::PaymentRefunded => "payment_refunded",
            Self::Unknown(value) => value.as_str(),
        }
    }
}

impl<'de> Deserialize<'de> for SuggestedPostRefundReason {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let reason = String::deserialize(deserializer)?;
        Ok(match reason.as_str() {
            "post_deleted" => Self::PostDeleted,
            "payment_refunded" => Self::PaymentRefunded,
            _ => Self::Unknown(reason),
        })
    }
}

impl Serialize for SuggestedPostRefundReason {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

/// Telegram suggested-post refunded payload.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct SuggestedPostRefunded {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suggested_post_message: Option<Box<Message>>,
    pub reason: SuggestedPostRefundReason,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// External reply metadata.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ExternalReplyInfo {
    pub origin: MessageOrigin,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chat: Option<Chat>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_id: Option<MessageId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub link_preview_options: Option<LinkPreviewOptions>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub animation: Option<Animation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio: Option<Audio>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub document: Option<Document>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub paid_media: Option<PaidMediaInfo>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub photo: Option<Vec<PhotoSize>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sticker: Option<Sticker>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub story: Option<Story>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub video: Option<Video>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub video_note: Option<VideoNote>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub voice: Option<Voice>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub has_media_spoiler: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checklist: Option<Checklist>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contact: Option<Contact>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dice: Option<Dice>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub game: Option<Game>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub giveaway: Option<Giveaway>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub giveaway_winners: Option<GiveawayWinners>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub invoice: Option<Invoice>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<Location>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub poll: Option<Poll>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub venue: Option<Venue>,
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

/// Telegram auto-delete timer change service payload.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct MessageAutoDeleteTimerChanged {
    pub message_auto_delete_time: u32,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram forum topic creation service payload.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ForumTopicCreated {
    pub name: String,
    pub icon_color: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_custom_emoji_id: Option<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub is_name_implicit: bool,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram forum topic edit service payload.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ForumTopicEdited {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_custom_emoji_id: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram forum topic closed service payload.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ForumTopicClosed {
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram forum topic reopened service payload.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ForumTopicReopened {
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram general forum topic hidden service payload.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct GeneralForumTopicHidden {
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram general forum topic unhidden service payload.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct GeneralForumTopicUnhidden {
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Shared user payload returned by `KeyboardButtonRequestUsers`.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct SharedUser {
    pub user_id: UserId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub photo: Option<Vec<PhotoSize>>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram users-shared service payload.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct UsersShared {
    pub request_id: i64,
    pub users: Vec<SharedUser>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram chat-shared service payload.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ChatShared {
    pub request_id: i64,
    pub chat_id: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub photo: Option<Vec<PhotoSize>>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram message that is no longer accessible to the bot.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct InaccessibleMessage {
    pub chat: Chat,
    pub message_id: MessageId,
    pub date: i64,
}

/// Telegram message reference that may be inaccessible.
#[derive(Clone, Debug)]
pub enum MaybeInaccessibleMessage {
    Accessible(Box<Message>),
    Inaccessible(InaccessibleMessage),
}

/// Classified message payload kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[non_exhaustive]
pub enum MessageKind {
    WriteAccessAllowed,
    WebAppData,
    ConnectedWebsite,
    Poll,
    PaidMedia,
    Checklist,
    Game,
    Invoice,
    SuccessfulPayment,
    RefundedPayment,
    NewChatMembers,
    LeftChatMember,
    ChatOwnerLeft,
    ChatOwnerChanged,
    NewChatTitle,
    NewChatPhoto,
    DeleteChatPhoto,
    GroupChatCreated,
    SupergroupChatCreated,
    ChannelChatCreated,
    PinnedMessage,
    MessageAutoDeleteTimerChanged,
    MigrateToChat,
    MigrateFromChat,
    UsersShared,
    ChatShared,
    ProximityAlertTriggered,
    BoostAdded,
    ChecklistTasksDone,
    ChecklistTasksAdded,
    DirectMessagePriceChanged,
    ForumTopicCreated,
    ForumTopicEdited,
    ForumTopicClosed,
    ForumTopicReopened,
    GeneralForumTopicHidden,
    GeneralForumTopicUnhidden,
    GiveawayCreated,
    Giveaway,
    GiveawayWinners,
    GiveawayCompleted,
    PaidMessagePriceChanged,
    SuggestedPostApproved,
    SuggestedPostApprovalFailed,
    SuggestedPostDeclined,
    SuggestedPostPaid,
    SuggestedPostRefunded,
    VideoChatScheduled,
    VideoChatStarted,
    VideoChatEnded,
    VideoChatParticipantsInvited,
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

const KNOWN_MESSAGE_KINDS: &[MessageKind] = &[
    MessageKind::WriteAccessAllowed,
    MessageKind::WebAppData,
    MessageKind::ConnectedWebsite,
    MessageKind::Poll,
    MessageKind::PaidMedia,
    MessageKind::Checklist,
    MessageKind::Game,
    MessageKind::Invoice,
    MessageKind::SuccessfulPayment,
    MessageKind::RefundedPayment,
    MessageKind::NewChatMembers,
    MessageKind::LeftChatMember,
    MessageKind::ChatOwnerLeft,
    MessageKind::ChatOwnerChanged,
    MessageKind::NewChatTitle,
    MessageKind::NewChatPhoto,
    MessageKind::DeleteChatPhoto,
    MessageKind::GroupChatCreated,
    MessageKind::SupergroupChatCreated,
    MessageKind::ChannelChatCreated,
    MessageKind::PinnedMessage,
    MessageKind::MessageAutoDeleteTimerChanged,
    MessageKind::MigrateToChat,
    MessageKind::MigrateFromChat,
    MessageKind::UsersShared,
    MessageKind::ChatShared,
    MessageKind::ProximityAlertTriggered,
    MessageKind::BoostAdded,
    MessageKind::ChecklistTasksDone,
    MessageKind::ChecklistTasksAdded,
    MessageKind::DirectMessagePriceChanged,
    MessageKind::ForumTopicCreated,
    MessageKind::ForumTopicEdited,
    MessageKind::ForumTopicClosed,
    MessageKind::ForumTopicReopened,
    MessageKind::GeneralForumTopicHidden,
    MessageKind::GeneralForumTopicUnhidden,
    MessageKind::GiveawayCreated,
    MessageKind::Giveaway,
    MessageKind::GiveawayWinners,
    MessageKind::GiveawayCompleted,
    MessageKind::PaidMessagePriceChanged,
    MessageKind::SuggestedPostApproved,
    MessageKind::SuggestedPostApprovalFailed,
    MessageKind::SuggestedPostDeclined,
    MessageKind::SuggestedPostPaid,
    MessageKind::SuggestedPostRefunded,
    MessageKind::VideoChatScheduled,
    MessageKind::VideoChatStarted,
    MessageKind::VideoChatEnded,
    MessageKind::VideoChatParticipantsInvited,
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

impl Serialize for MaybeInaccessibleMessage {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Accessible(message) => message.serialize(serializer),
            Self::Inaccessible(message) => message.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for MaybeInaccessibleMessage {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        let date = value
            .get("date")
            .and_then(Value::as_i64)
            .unwrap_or_default();
        if date == 0 {
            InaccessibleMessage::deserialize(value)
                .map(Self::Inaccessible)
                .map_err(serde::de::Error::custom)
        } else {
            Message::deserialize(value)
                .map(|message| Self::Accessible(Box::new(message)))
                .map_err(serde::de::Error::custom)
        }
    }
}

/// Telegram message object.
#[derive(Clone, Debug, Serialize)]
#[non_exhaustive]
pub struct Message {
    pub message_id: MessageId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from: Option<User>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sender_chat: Option<Chat>,
    pub chat: Chat,
    pub date: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author_signature: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sender_tag: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_thread_id: Option<i64>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub is_topic_message: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub forward_origin: Option<MessageOrigin>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub is_automatic_forward: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_to_message: Option<Box<MaybeInaccessibleMessage>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_reply: Option<Box<ExternalReplyInfo>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quote: Option<TextQuote>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_to_story: Option<Story>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_to_checklist_task_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub via_bot: Option<User>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub edit_date: Option<i64>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub has_protected_content: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub is_from_offline: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub is_paid_post: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub media_group_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub paid_star_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entities: Option<Vec<MessageEntity>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caption_entities: Option<Vec<MessageEntity>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub link_preview_options: Option<LinkPreviewOptions>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suggested_post_info: Option<Box<SuggestedPostInfo>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effect_id: Option<String>,
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
    pub paid_media: Option<Box<PaidMediaInfo>>,
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
    pub poll: Option<Box<Poll>>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub show_caption_above_media: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub has_media_spoiler: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checklist: Option<Box<Checklist>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub game: Option<Box<Game>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub web_app_data: Option<WebAppData>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub write_access_allowed: Option<WriteAccessAllowed>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub new_chat_members: Option<Vec<User>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub left_chat_member: Option<User>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chat_owner_left: Option<ChatOwnerLeft>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chat_owner_changed: Option<ChatOwnerChanged>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub new_chat_title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub new_chat_photo: Option<Vec<PhotoSize>>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub delete_chat_photo: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub group_chat_created: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub supergroup_chat_created: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub channel_chat_created: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pinned_message: Option<Box<MaybeInaccessibleMessage>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_auto_delete_timer_changed: Option<MessageAutoDeleteTimerChanged>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub migrate_to_chat_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub migrate_from_chat_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub invoice: Option<Box<Invoice>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub successful_payment: Option<Box<SuccessfulPayment>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refunded_payment: Option<Box<RefundedPayment>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub users_shared: Option<UsersShared>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chat_shared: Option<ChatShared>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub connected_website: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proximity_alert_triggered: Option<Box<ProximityAlertTriggered>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub boost_added: Option<Box<ChatBoostAdded>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checklist_tasks_done: Option<Box<ChecklistTasksDone>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checklist_tasks_added: Option<Box<ChecklistTasksAdded>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub direct_message_price_changed: Option<Box<DirectMessagePriceChanged>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub forum_topic_created: Option<ForumTopicCreated>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub forum_topic_edited: Option<ForumTopicEdited>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub forum_topic_closed: Option<ForumTopicClosed>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub forum_topic_reopened: Option<ForumTopicReopened>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub general_forum_topic_hidden: Option<GeneralForumTopicHidden>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub general_forum_topic_unhidden: Option<GeneralForumTopicUnhidden>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub giveaway_created: Option<Box<GiveawayCreated>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub giveaway: Option<Box<Giveaway>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub giveaway_winners: Option<Box<GiveawayWinners>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub giveaway_completed: Option<Box<GiveawayCompleted>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub paid_message_price_changed: Option<Box<PaidMessagePriceChanged>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suggested_post_approved: Option<Box<SuggestedPostApproved>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suggested_post_approval_failed: Option<Box<SuggestedPostApprovalFailed>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suggested_post_declined: Option<Box<SuggestedPostDeclined>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suggested_post_paid: Option<Box<SuggestedPostPaid>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suggested_post_refunded: Option<Box<SuggestedPostRefunded>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub video_chat_scheduled: Option<VideoChatScheduled>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub video_chat_started: Option<VideoChatStarted>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub video_chat_ended: Option<VideoChatEnded>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub video_chat_participants_invited: Option<VideoChatParticipantsInvited>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_markup: Option<Box<ReplyMarkup>>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

const fn is_false(value: &bool) -> bool {
    !*value
}

fn take_required_field<T, E>(
    object: &mut serde_json::Map<String, Value>,
    field: &'static str,
) -> std::result::Result<T, E>
where
    T: DeserializeOwned,
    E: DeError,
{
    let value = object
        .remove(field)
        .ok_or_else(|| E::missing_field(field))?;
    serde_json::from_value(value).map_err(E::custom)
}

fn take_optional_field<T, E>(
    object: &mut serde_json::Map<String, Value>,
    field: &'static str,
) -> std::result::Result<Option<T>, E>
where
    T: DeserializeOwned,
    E: DeError,
{
    object
        .remove(field)
        .map(|value| serde_json::from_value(value).map_err(E::custom))
        .transpose()
}

fn take_bool_field<E>(
    object: &mut serde_json::Map<String, Value>,
    field: &'static str,
) -> std::result::Result<bool, E>
where
    E: DeError,
{
    Ok(take_optional_field::<bool, E>(object, field)?.unwrap_or(false))
}

impl<'de> Deserialize<'de> for Message {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut object = serde_json::Map::<String, Value>::deserialize(deserializer)?;

        Ok(Self {
            message_id: take_required_field(&mut object, "message_id")?,
            from: take_optional_field(&mut object, "from")?,
            sender_chat: take_optional_field(&mut object, "sender_chat")?,
            chat: take_required_field(&mut object, "chat")?,
            date: take_required_field(&mut object, "date")?,
            author_signature: take_optional_field(&mut object, "author_signature")?,
            sender_tag: take_optional_field(&mut object, "sender_tag")?,
            message_thread_id: take_optional_field(&mut object, "message_thread_id")?,
            is_topic_message: take_bool_field(&mut object, "is_topic_message")?,
            forward_origin: take_optional_field(&mut object, "forward_origin")?,
            is_automatic_forward: take_bool_field(&mut object, "is_automatic_forward")?,
            reply_to_message: take_optional_field(&mut object, "reply_to_message")?,
            external_reply: take_optional_field(&mut object, "external_reply")?,
            quote: take_optional_field(&mut object, "quote")?,
            reply_to_story: take_optional_field(&mut object, "reply_to_story")?,
            reply_to_checklist_task_id: take_optional_field(
                &mut object,
                "reply_to_checklist_task_id",
            )?,
            via_bot: take_optional_field(&mut object, "via_bot")?,
            edit_date: take_optional_field(&mut object, "edit_date")?,
            has_protected_content: take_bool_field(&mut object, "has_protected_content")?,
            is_from_offline: take_bool_field(&mut object, "is_from_offline")?,
            is_paid_post: take_bool_field(&mut object, "is_paid_post")?,
            media_group_id: take_optional_field(&mut object, "media_group_id")?,
            paid_star_count: take_optional_field(&mut object, "paid_star_count")?,
            text: take_optional_field(&mut object, "text")?,
            caption: take_optional_field(&mut object, "caption")?,
            entities: take_optional_field(&mut object, "entities")?,
            caption_entities: take_optional_field(&mut object, "caption_entities")?,
            link_preview_options: take_optional_field(&mut object, "link_preview_options")?,
            suggested_post_info: take_optional_field(&mut object, "suggested_post_info")?,
            effect_id: take_optional_field(&mut object, "effect_id")?,
            animation: take_optional_field(&mut object, "animation")?,
            audio: take_optional_field(&mut object, "audio")?,
            contact: take_optional_field(&mut object, "contact")?,
            dice: take_optional_field(&mut object, "dice")?,
            document: take_optional_field(&mut object, "document")?,
            paid_media: take_optional_field(&mut object, "paid_media")?,
            location: take_optional_field(&mut object, "location")?,
            photo: take_optional_field(&mut object, "photo")?,
            sticker: take_optional_field(&mut object, "sticker")?,
            story: take_optional_field(&mut object, "story")?,
            venue: take_optional_field(&mut object, "venue")?,
            video: take_optional_field(&mut object, "video")?,
            video_note: take_optional_field(&mut object, "video_note")?,
            voice: take_optional_field(&mut object, "voice")?,
            poll: take_optional_field(&mut object, "poll")?,
            show_caption_above_media: take_bool_field(&mut object, "show_caption_above_media")?,
            has_media_spoiler: take_bool_field(&mut object, "has_media_spoiler")?,
            checklist: take_optional_field(&mut object, "checklist")?,
            game: take_optional_field(&mut object, "game")?,
            web_app_data: take_optional_field(&mut object, "web_app_data")?,
            write_access_allowed: take_optional_field(&mut object, "write_access_allowed")?,
            new_chat_members: take_optional_field(&mut object, "new_chat_members")?,
            left_chat_member: take_optional_field(&mut object, "left_chat_member")?,
            chat_owner_left: take_optional_field(&mut object, "chat_owner_left")?,
            chat_owner_changed: take_optional_field(&mut object, "chat_owner_changed")?,
            new_chat_title: take_optional_field(&mut object, "new_chat_title")?,
            new_chat_photo: take_optional_field(&mut object, "new_chat_photo")?,
            delete_chat_photo: take_bool_field(&mut object, "delete_chat_photo")?,
            group_chat_created: take_bool_field(&mut object, "group_chat_created")?,
            supergroup_chat_created: take_bool_field(&mut object, "supergroup_chat_created")?,
            channel_chat_created: take_bool_field(&mut object, "channel_chat_created")?,
            pinned_message: take_optional_field(&mut object, "pinned_message")?,
            message_auto_delete_timer_changed: take_optional_field(
                &mut object,
                "message_auto_delete_timer_changed",
            )?,
            migrate_to_chat_id: take_optional_field(&mut object, "migrate_to_chat_id")?,
            migrate_from_chat_id: take_optional_field(&mut object, "migrate_from_chat_id")?,
            invoice: take_optional_field(&mut object, "invoice")?,
            successful_payment: take_optional_field(&mut object, "successful_payment")?,
            refunded_payment: take_optional_field(&mut object, "refunded_payment")?,
            users_shared: take_optional_field(&mut object, "users_shared")?,
            chat_shared: take_optional_field(&mut object, "chat_shared")?,
            connected_website: take_optional_field(&mut object, "connected_website")?,
            proximity_alert_triggered: take_optional_field(
                &mut object,
                "proximity_alert_triggered",
            )?,
            boost_added: take_optional_field(&mut object, "boost_added")?,
            checklist_tasks_done: take_optional_field(&mut object, "checklist_tasks_done")?,
            checklist_tasks_added: take_optional_field(&mut object, "checklist_tasks_added")?,
            direct_message_price_changed: take_optional_field(
                &mut object,
                "direct_message_price_changed",
            )?,
            forum_topic_created: take_optional_field(&mut object, "forum_topic_created")?,
            forum_topic_edited: take_optional_field(&mut object, "forum_topic_edited")?,
            forum_topic_closed: take_optional_field(&mut object, "forum_topic_closed")?,
            forum_topic_reopened: take_optional_field(&mut object, "forum_topic_reopened")?,
            general_forum_topic_hidden: take_optional_field(
                &mut object,
                "general_forum_topic_hidden",
            )?,
            general_forum_topic_unhidden: take_optional_field(
                &mut object,
                "general_forum_topic_unhidden",
            )?,
            giveaway_created: take_optional_field(&mut object, "giveaway_created")?,
            giveaway: take_optional_field(&mut object, "giveaway")?,
            giveaway_winners: take_optional_field(&mut object, "giveaway_winners")?,
            giveaway_completed: take_optional_field(&mut object, "giveaway_completed")?,
            paid_message_price_changed: take_optional_field(
                &mut object,
                "paid_message_price_changed",
            )?,
            suggested_post_approved: take_optional_field(&mut object, "suggested_post_approved")?,
            suggested_post_approval_failed: take_optional_field(
                &mut object,
                "suggested_post_approval_failed",
            )?,
            suggested_post_declined: take_optional_field(&mut object, "suggested_post_declined")?,
            suggested_post_paid: take_optional_field(&mut object, "suggested_post_paid")?,
            suggested_post_refunded: take_optional_field(&mut object, "suggested_post_refunded")?,
            video_chat_scheduled: take_optional_field(&mut object, "video_chat_scheduled")?,
            video_chat_started: take_optional_field(&mut object, "video_chat_started")?,
            video_chat_ended: take_optional_field(&mut object, "video_chat_ended")?,
            video_chat_participants_invited: take_optional_field(
                &mut object,
                "video_chat_participants_invited",
            )?,
            reply_markup: take_optional_field(&mut object, "reply_markup")?,
            extra: object.into_iter().collect(),
        })
    }
}

fn is_unmodeled_message_content_key(key: &str) -> bool {
    matches!(
        key,
        "gift" | "unique_gift" | "gift_upgrade_sent" | "passport_data" | "chat_background_set"
    )
}

impl Message {
    pub fn chat(&self) -> &Chat {
        &self.chat
    }

    pub fn from_user(&self) -> Option<&User> {
        self.from.as_ref()
    }

    pub fn sender_chat(&self) -> Option<&Chat> {
        self.sender_chat.as_ref()
    }

    pub fn reply_to_message(&self) -> Option<&MaybeInaccessibleMessage> {
        self.reply_to_message.as_deref()
    }

    pub fn pinned_message(&self) -> Option<&MaybeInaccessibleMessage> {
        self.pinned_message.as_deref()
    }

    fn has_modeled_kind(&self) -> bool {
        self.write_access_allowed.is_some()
            || self.web_app_data.is_some()
            || self.connected_website.is_some()
            || self.poll.is_some()
            || self.paid_media.is_some()
            || self.checklist.is_some()
            || self.game.is_some()
            || self.invoice.is_some()
            || self.successful_payment.is_some()
            || self.refunded_payment.is_some()
            || self.new_chat_members.is_some()
            || self.left_chat_member.is_some()
            || self.chat_owner_left.is_some()
            || self.chat_owner_changed.is_some()
            || self.new_chat_title.is_some()
            || self.new_chat_photo.is_some()
            || self.delete_chat_photo
            || self.group_chat_created
            || self.supergroup_chat_created
            || self.channel_chat_created
            || self.pinned_message.is_some()
            || self.message_auto_delete_timer_changed.is_some()
            || self.migrate_to_chat_id.is_some()
            || self.migrate_from_chat_id.is_some()
            || self.users_shared.is_some()
            || self.chat_shared.is_some()
            || self.proximity_alert_triggered.is_some()
            || self.boost_added.is_some()
            || self.checklist_tasks_done.is_some()
            || self.checklist_tasks_added.is_some()
            || self.direct_message_price_changed.is_some()
            || self.forum_topic_created.is_some()
            || self.forum_topic_edited.is_some()
            || self.forum_topic_closed.is_some()
            || self.forum_topic_reopened.is_some()
            || self.general_forum_topic_hidden.is_some()
            || self.general_forum_topic_unhidden.is_some()
            || self.giveaway_created.is_some()
            || self.giveaway.is_some()
            || self.giveaway_winners.is_some()
            || self.giveaway_completed.is_some()
            || self.paid_message_price_changed.is_some()
            || self.suggested_post_approved.is_some()
            || self.suggested_post_approval_failed.is_some()
            || self.suggested_post_declined.is_some()
            || self.suggested_post_paid.is_some()
            || self.suggested_post_refunded.is_some()
            || self.video_chat_scheduled.is_some()
            || self.video_chat_started.is_some()
            || self.video_chat_ended.is_some()
            || self.video_chat_participants_invited.is_some()
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
        for &kind in KNOWN_MESSAGE_KINDS {
            if self.has_kind(kind) {
                return kind;
            }
        }

        MessageKind::Unknown
    }

    /// Returns all detected kinds for this message.
    pub fn kinds(&self) -> Vec<MessageKind> {
        let mut kinds = Vec::with_capacity(KNOWN_MESSAGE_KINDS.len() + 1);
        for &kind in KNOWN_MESSAGE_KINDS {
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
            MessageKind::ConnectedWebsite => self.connected_website.is_some(),
            MessageKind::Poll => self.poll.is_some(),
            MessageKind::PaidMedia => self.paid_media.is_some(),
            MessageKind::Checklist => self.checklist.is_some(),
            MessageKind::Game => self.game.is_some(),
            MessageKind::Invoice => self.invoice.is_some(),
            MessageKind::SuccessfulPayment => self.successful_payment.is_some(),
            MessageKind::RefundedPayment => self.refunded_payment.is_some(),
            MessageKind::NewChatMembers => self.new_chat_members.is_some(),
            MessageKind::LeftChatMember => self.left_chat_member.is_some(),
            MessageKind::ChatOwnerLeft => self.chat_owner_left.is_some(),
            MessageKind::ChatOwnerChanged => self.chat_owner_changed.is_some(),
            MessageKind::NewChatTitle => self.new_chat_title.is_some(),
            MessageKind::NewChatPhoto => self.new_chat_photo.is_some(),
            MessageKind::DeleteChatPhoto => self.delete_chat_photo,
            MessageKind::GroupChatCreated => self.group_chat_created,
            MessageKind::SupergroupChatCreated => self.supergroup_chat_created,
            MessageKind::ChannelChatCreated => self.channel_chat_created,
            MessageKind::PinnedMessage => self.pinned_message.is_some(),
            MessageKind::MessageAutoDeleteTimerChanged => {
                self.message_auto_delete_timer_changed.is_some()
            }
            MessageKind::MigrateToChat => self.migrate_to_chat_id.is_some(),
            MessageKind::MigrateFromChat => self.migrate_from_chat_id.is_some(),
            MessageKind::UsersShared => self.users_shared.is_some(),
            MessageKind::ChatShared => self.chat_shared.is_some(),
            MessageKind::ProximityAlertTriggered => self.proximity_alert_triggered.is_some(),
            MessageKind::BoostAdded => self.boost_added.is_some(),
            MessageKind::ChecklistTasksDone => self.checklist_tasks_done.is_some(),
            MessageKind::ChecklistTasksAdded => self.checklist_tasks_added.is_some(),
            MessageKind::DirectMessagePriceChanged => self.direct_message_price_changed.is_some(),
            MessageKind::ForumTopicCreated => self.forum_topic_created.is_some(),
            MessageKind::ForumTopicEdited => self.forum_topic_edited.is_some(),
            MessageKind::ForumTopicClosed => self.forum_topic_closed.is_some(),
            MessageKind::ForumTopicReopened => self.forum_topic_reopened.is_some(),
            MessageKind::GeneralForumTopicHidden => self.general_forum_topic_hidden.is_some(),
            MessageKind::GeneralForumTopicUnhidden => self.general_forum_topic_unhidden.is_some(),
            MessageKind::GiveawayCreated => self.giveaway_created.is_some(),
            MessageKind::Giveaway => self.giveaway.is_some(),
            MessageKind::GiveawayWinners => self.giveaway_winners.is_some(),
            MessageKind::GiveawayCompleted => self.giveaway_completed.is_some(),
            MessageKind::PaidMessagePriceChanged => self.paid_message_price_changed.is_some(),
            MessageKind::SuggestedPostApproved => self.suggested_post_approved.is_some(),
            MessageKind::SuggestedPostApprovalFailed => {
                self.suggested_post_approval_failed.is_some()
            }
            MessageKind::SuggestedPostDeclined => self.suggested_post_declined.is_some(),
            MessageKind::SuggestedPostPaid => self.suggested_post_paid.is_some(),
            MessageKind::SuggestedPostRefunded => self.suggested_post_refunded.is_some(),
            MessageKind::VideoChatScheduled => self.video_chat_scheduled.is_some(),
            MessageKind::VideoChatStarted => self.video_chat_started.is_some(),
            MessageKind::VideoChatEnded => self.video_chat_ended.is_some(),
            MessageKind::VideoChatParticipantsInvited => {
                self.video_chat_participants_invited.is_some()
            }
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

impl MaybeInaccessibleMessage {
    pub fn is_accessible(&self) -> bool {
        matches!(self, Self::Accessible(_))
    }

    pub fn accessible(&self) -> Option<&Message> {
        match self {
            Self::Accessible(message) => Some(message.as_ref()),
            Self::Inaccessible(_) => None,
        }
    }

    pub fn inaccessible(&self) -> Option<&InaccessibleMessage> {
        match self {
            Self::Accessible(_) => None,
            Self::Inaccessible(message) => Some(message),
        }
    }

    pub fn chat(&self) -> &Chat {
        match self {
            Self::Accessible(message) => &message.chat,
            Self::Inaccessible(message) => &message.chat,
        }
    }

    pub fn message_id(&self) -> MessageId {
        match self {
            Self::Accessible(message) => message.message_id,
            Self::Inaccessible(message) => message.message_id,
        }
    }

    pub fn date(&self) -> i64 {
        match self {
            Self::Accessible(message) => message.date,
            Self::Inaccessible(message) => message.date,
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
            "gift": {"kind": "mystery"}
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
            "gift": {"kind": "mystery"}
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
            MessageKind::ConnectedWebsite => {
                object.insert("connected_website".to_owned(), json!("example.com"));
            }
            MessageKind::Poll => {
                object.insert(
                    "poll".to_owned(),
                    json!({
                        "id": "poll-1",
                        "question": "q?",
                        "question_entities": [{"type": "custom_emoji", "offset": 0, "length": 1, "custom_emoji_id": "ce-1"}],
                        "options": [{"text": "a", "voter_count": 1}],
                        "total_voter_count": 1,
                        "is_closed": true,
                        "is_anonymous": false,
                        "type": "regular",
                        "allows_multiple_answers": false,
                        "correct_option_id": 0,
                        "explanation": "ok",
                        "explanation_entities": [{"type": "bold", "offset": 0, "length": 2}],
                        "open_period": 30,
                        "close_date": 1700001000
                    }),
                );
            }
            MessageKind::PaidMedia => {
                object.insert(
                    "paid_media".to_owned(),
                    json!({
                        "star_count": 5,
                        "paid_media": [{
                            "type": "preview",
                            "width": 640,
                            "height": 480
                        }]
                    }),
                );
            }
            MessageKind::Checklist => {
                object.insert(
                    "checklist".to_owned(),
                    json!({
                        "title": "ops",
                        "tasks": [{
                            "id": 1,
                            "text": "triage"
                        }],
                        "others_can_add_tasks": true
                    }),
                );
            }
            MessageKind::Game => {
                object.insert(
                    "game".to_owned(),
                    json!({
                        "title": "Demo",
                        "description": "Fun",
                        "photo": [{
                            "file_id": "g-p-1",
                            "file_unique_id": "g-pu-1",
                            "width": 32,
                            "height": 32
                        }]
                    }),
                );
            }
            MessageKind::Invoice => {
                object.insert(
                    "invoice".to_owned(),
                    json!({
                        "title": "Premium",
                        "description": "Subscription",
                        "start_parameter": "start",
                        "currency": "USD",
                        "total_amount": 999
                    }),
                );
            }
            MessageKind::SuccessfulPayment => {
                object.insert(
                    "successful_payment".to_owned(),
                    json!({
                        "currency": "USD",
                        "total_amount": 999,
                        "invoice_payload": "inv-1",
                        "telegram_payment_charge_id": "tg-1",
                        "provider_payment_charge_id": "prov-1"
                    }),
                );
            }
            MessageKind::RefundedPayment => {
                object.insert(
                    "refunded_payment".to_owned(),
                    json!({
                        "currency": "XTR",
                        "total_amount": 100,
                        "invoice_payload": "inv-1",
                        "telegram_payment_charge_id": "tg-1"
                    }),
                );
            }
            MessageKind::NewChatMembers => {
                object.insert(
                    "new_chat_members".to_owned(),
                    json!([{
                        "id": 7,
                        "is_bot": false,
                        "first_name": "newbie"
                    }]),
                );
            }
            MessageKind::LeftChatMember => {
                object.insert(
                    "left_chat_member".to_owned(),
                    json!({
                        "id": 8,
                        "is_bot": false,
                        "first_name": "departed"
                    }),
                );
            }
            MessageKind::ChatOwnerLeft => {
                object.insert(
                    "chat_owner_left".to_owned(),
                    json!({
                        "new_owner": {
                            "id": 9,
                            "is_bot": false,
                            "first_name": "owner-next"
                        }
                    }),
                );
            }
            MessageKind::ChatOwnerChanged => {
                object.insert(
                    "chat_owner_changed".to_owned(),
                    json!({
                        "new_owner": {
                            "id": 10,
                            "is_bot": false,
                            "first_name": "owner-new"
                        }
                    }),
                );
            }
            MessageKind::NewChatTitle => {
                object.insert("new_chat_title".to_owned(), json!("ops"));
            }
            MessageKind::NewChatPhoto => {
                object.insert(
                    "new_chat_photo".to_owned(),
                    json!([{
                        "file_id": "cp-1",
                        "file_unique_id": "cpu-1",
                        "width": 64,
                        "height": 64
                    }]),
                );
            }
            MessageKind::DeleteChatPhoto => {
                object.insert("delete_chat_photo".to_owned(), json!(true));
            }
            MessageKind::GroupChatCreated => {
                object.insert("group_chat_created".to_owned(), json!(true));
            }
            MessageKind::SupergroupChatCreated => {
                object.insert("supergroup_chat_created".to_owned(), json!(true));
            }
            MessageKind::ChannelChatCreated => {
                object.insert("channel_chat_created".to_owned(), json!(true));
            }
            MessageKind::PinnedMessage => {
                object.insert(
                    "pinned_message".to_owned(),
                    json!({
                        "message_id": 500,
                        "date": 0,
                        "chat": {"id": -1001, "type": "supergroup", "title": "mods"}
                    }),
                );
            }
            MessageKind::MessageAutoDeleteTimerChanged => {
                object.insert(
                    "message_auto_delete_timer_changed".to_owned(),
                    json!({"message_auto_delete_time": 60}),
                );
            }
            MessageKind::MigrateToChat => {
                object.insert("migrate_to_chat_id".to_owned(), json!(-1001001001_i64));
            }
            MessageKind::MigrateFromChat => {
                object.insert("migrate_from_chat_id".to_owned(), json!(-1001001002_i64));
            }
            MessageKind::UsersShared => {
                object.insert(
                    "users_shared".to_owned(),
                    json!({
                        "request_id": 3,
                        "users": [{
                            "user_id": 7,
                            "first_name": "shared"
                        }]
                    }),
                );
            }
            MessageKind::ChatShared => {
                object.insert(
                    "chat_shared".to_owned(),
                    json!({
                        "request_id": 4,
                        "chat_id": -1002,
                        "title": "shared-chat"
                    }),
                );
            }
            MessageKind::ProximityAlertTriggered => {
                object.insert(
                    "proximity_alert_triggered".to_owned(),
                    json!({
                        "traveler": {
                            "id": 11,
                            "is_bot": false,
                            "first_name": "traveler"
                        },
                        "watcher": {
                            "id": 12,
                            "is_bot": false,
                            "first_name": "watcher"
                        },
                        "distance": 42
                    }),
                );
            }
            MessageKind::BoostAdded => {
                object.insert("boost_added".to_owned(), json!({"boost_count": 2}));
            }
            MessageKind::ChecklistTasksDone => {
                object.insert(
                    "checklist_tasks_done".to_owned(),
                    json!({
                        "checklist_message": {
                            "message_id": 300,
                            "date": 1700000000,
                            "chat": {"id": -1001, "type": "supergroup", "title": "mods"},
                            "checklist": {
                                "title": "ops",
                                "tasks": [{"id": 1, "text": "triage"}]
                            }
                        },
                        "marked_as_done_task_ids": [1]
                    }),
                );
            }
            MessageKind::ChecklistTasksAdded => {
                object.insert(
                    "checklist_tasks_added".to_owned(),
                    json!({
                        "tasks": [{
                            "id": 2,
                            "text": "review"
                        }]
                    }),
                );
            }
            MessageKind::DirectMessagePriceChanged => {
                object.insert(
                    "direct_message_price_changed".to_owned(),
                    json!({
                        "are_direct_messages_enabled": true,
                        "direct_message_star_count": 5
                    }),
                );
            }
            MessageKind::ForumTopicCreated => {
                object.insert(
                    "forum_topic_created".to_owned(),
                    json!({
                        "name": "ops",
                        "icon_color": 7322096
                    }),
                );
            }
            MessageKind::ForumTopicEdited => {
                object.insert(
                    "forum_topic_edited".to_owned(),
                    json!({
                        "name": "ops-renamed"
                    }),
                );
            }
            MessageKind::ForumTopicClosed => {
                object.insert("forum_topic_closed".to_owned(), json!({}));
            }
            MessageKind::ForumTopicReopened => {
                object.insert("forum_topic_reopened".to_owned(), json!({}));
            }
            MessageKind::GeneralForumTopicHidden => {
                object.insert("general_forum_topic_hidden".to_owned(), json!({}));
            }
            MessageKind::GeneralForumTopicUnhidden => {
                object.insert("general_forum_topic_unhidden".to_owned(), json!({}));
            }
            MessageKind::GiveawayCreated => {
                object.insert(
                    "giveaway_created".to_owned(),
                    json!({
                        "prize_star_count": 100
                    }),
                );
            }
            MessageKind::Giveaway => {
                object.insert(
                    "giveaway".to_owned(),
                    json!({
                        "chats": [{
                            "id": -1001,
                            "type": "supergroup",
                            "title": "mods"
                        }],
                        "winners_selection_date": 1700000200,
                        "winner_count": 2,
                        "has_public_winners": true
                    }),
                );
            }
            MessageKind::GiveawayWinners => {
                object.insert(
                    "giveaway_winners".to_owned(),
                    json!({
                        "chat": {"id": -1001, "type": "supergroup", "title": "mods"},
                        "giveaway_message_id": 401,
                        "winners_selection_date": 1700000201,
                        "winner_count": 1,
                        "winners": [{
                            "id": 13,
                            "is_bot": false,
                            "first_name": "winner"
                        }]
                    }),
                );
            }
            MessageKind::GiveawayCompleted => {
                object.insert(
                    "giveaway_completed".to_owned(),
                    json!({
                        "winner_count": 3,
                        "is_star_giveaway": true
                    }),
                );
            }
            MessageKind::PaidMessagePriceChanged => {
                object.insert(
                    "paid_message_price_changed".to_owned(),
                    json!({
                        "paid_message_star_count": 7
                    }),
                );
            }
            MessageKind::SuggestedPostApproved => {
                object.insert(
                    "suggested_post_approved".to_owned(),
                    json!({
                        "price": {
                            "currency": "XTR",
                            "amount": 50
                        },
                        "send_date": 1700000300
                    }),
                );
            }
            MessageKind::SuggestedPostApprovalFailed => {
                object.insert(
                    "suggested_post_approval_failed".to_owned(),
                    json!({
                        "price": {
                            "currency": "XTR",
                            "amount": 50
                        }
                    }),
                );
            }
            MessageKind::SuggestedPostDeclined => {
                object.insert(
                    "suggested_post_declined".to_owned(),
                    json!({
                        "comment": "no"
                    }),
                );
            }
            MessageKind::SuggestedPostPaid => {
                object.insert(
                    "suggested_post_paid".to_owned(),
                    json!({
                        "currency": "XTR",
                        "star_amount": {
                            "amount": 10
                        }
                    }),
                );
            }
            MessageKind::SuggestedPostRefunded => {
                object.insert(
                    "suggested_post_refunded".to_owned(),
                    json!({
                        "reason": "post_deleted"
                    }),
                );
            }
            MessageKind::VideoChatScheduled => {
                object.insert(
                    "video_chat_scheduled".to_owned(),
                    json!({
                        "start_date": 1700000400
                    }),
                );
            }
            MessageKind::VideoChatStarted => {
                object.insert("video_chat_started".to_owned(), json!({}));
            }
            MessageKind::VideoChatEnded => {
                object.insert(
                    "video_chat_ended".to_owned(),
                    json!({
                        "duration": 120
                    }),
                );
            }
            MessageKind::VideoChatParticipantsInvited => {
                object.insert(
                    "video_chat_participants_invited".to_owned(),
                    json!({
                        "users": [{
                            "id": 14,
                            "is_bot": false,
                            "first_name": "invitee"
                        }]
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
                object.insert("gift".to_owned(), json!({"kind": "mystery"}));
            }
        }

        Ok(serde_json::from_value(Value::Object(object))?)
    }

    #[test]
    fn message_kind_matrix_stays_in_sync() -> std::result::Result<(), Box<dyn StdError>> {
        for &kind in KNOWN_MESSAGE_KINDS {
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
                {
                    "type": "date_time",
                    "offset": 7,
                    "length": 5,
                    "unix_time": 1700000000,
                    "date_time_format": "ddd"
                },
                {"type": "mystery_entity", "offset": 13, "length": 5}
            ],
            "poll": {
                "id": "poll-2",
                "question": "q?",
                "question_entities": [{"type": "custom_emoji", "offset": 0, "length": 1, "custom_emoji_id": "ce-2"}],
                "options": [{"text": "a", "voter_count": 1}],
                "total_voter_count": 1,
                "is_closed": true,
                "is_anonymous": false,
                "type": "quiz",
                "allows_multiple_answers": false,
                "correct_option_id": 0,
                "explanation": "because",
                "explanation_entities": [{"type": "italic", "offset": 0, "length": 7}],
                "close_date": 1700000044
            }
        }))?;

        let entities = message.entities.as_ref().ok_or("missing entities")?;
        assert_eq!(entities[0].kind, MessageEntityKind::BotCommand);
        assert_eq!(entities[1].kind, MessageEntityKind::DateTime);
        assert_eq!(entities[1].unix_time, Some(1_700_000_000));
        assert_eq!(entities[1].date_time_format.as_deref(), Some("ddd"));
        assert_eq!(
            entities[2].kind,
            MessageEntityKind::Unknown("mystery_entity".to_owned())
        );

        let poll = message.poll.as_ref().ok_or("missing poll")?;
        assert_eq!(poll.kind, PollKind::Quiz);
        assert_eq!(poll.correct_option_id, Some(0));
        assert_eq!(poll.explanation.as_deref(), Some("because"));
        assert_eq!(poll.close_date, Some(1_700_000_044));

        Ok(())
    }

    #[test]
    fn parses_service_message_metadata_and_references() -> std::result::Result<(), Box<dyn StdError>>
    {
        let message: Message = serde_json::from_value(json!({
            "message_id": 44,
            "date": 1700000044,
            "chat": {"id": -1001, "type": "supergroup", "title": "mods"},
            "sender_chat": {"id": -1002, "type": "channel", "title": "announcements"},
            "author_signature": "anonymous admin",
            "sender_tag": "ops",
            "message_thread_id": 77,
            "is_topic_message": true,
            "via_bot": {"id": 99, "is_bot": true, "first_name": "relay"},
            "has_protected_content": true,
            "is_from_offline": true,
            "is_paid_post": true,
            "media_group_id": "album-1",
            "paid_star_count": 5,
            "quote": {
                "text": "quoted",
                "position": 3,
                "is_manual": true
            },
            "external_reply": {
                "origin": {
                    "type": "user",
                    "date": 1700000000,
                    "sender_user": {"id": 2, "is_bot": false, "first_name": "alice"}
                },
                "message_id": 123
            },
            "reply_to_story": {
                "chat": {"id": -1002, "type": "channel", "title": "announcements"},
                "id": 77
            },
            "reply_to_checklist_task_id": 9,
            "reply_to_message": {
                "message_id": 10,
                "date": 0,
                "chat": {"id": -1001, "type": "supergroup", "title": "mods"}
            },
            "pinned_message": {
                "message_id": 11,
                "date": 1700000000,
                "chat": {"id": -1001, "type": "supergroup", "title": "mods"},
                "text": "hello"
            },
            "link_preview_options": {"is_disabled": true},
            "reply_markup": {
                "inline_keyboard": [[{"text": "Open", "url": "https://example.com"}]]
            }
        }))?;

        assert_eq!(message.sender_chat().map(|chat| chat.id), Some(-1002));
        assert_eq!(message.author_signature.as_deref(), Some("anonymous admin"));
        assert_eq!(message.sender_tag.as_deref(), Some("ops"));
        assert_eq!(message.message_thread_id, Some(77));
        assert!(message.is_topic_message);
        assert_eq!(
            message.via_bot.as_ref().map(|user| user.id),
            Some(UserId(99))
        );
        assert!(message.has_protected_content);
        assert!(message.is_from_offline);
        assert!(message.is_paid_post);
        assert_eq!(message.media_group_id.as_deref(), Some("album-1"));
        assert_eq!(message.paid_star_count, Some(5));
        assert_eq!(
            message.quote.as_ref().map(|quote| quote.text.as_str()),
            Some("quoted")
        );
        assert_eq!(
            message
                .external_reply
                .as_ref()
                .and_then(|reply| reply.message_id),
            Some(MessageId(123))
        );
        assert_eq!(
            message.reply_to_story.as_ref().map(|story| story.id),
            Some(77)
        );
        assert_eq!(message.reply_to_checklist_task_id, Some(9));
        assert_eq!(
            message
                .link_preview_options
                .as_ref()
                .map(|options| options.is_disabled),
            Some(Some(true))
        );
        assert!(message.reply_markup.is_some());

        let reply = message.reply_to_message().ok_or("missing reply")?;
        assert!(!reply.is_accessible());
        assert_eq!(reply.message_id(), MessageId(10));

        let pinned = message.pinned_message().ok_or("missing pinned message")?;
        assert!(pinned.is_accessible());
        assert_eq!(
            pinned
                .accessible()
                .and_then(|message| message.text.as_deref()),
            Some("hello")
        );

        Ok(())
    }

    #[test]
    fn parses_paid_media_and_suggested_post_payloads() -> std::result::Result<(), Box<dyn StdError>>
    {
        let message: Message = serde_json::from_value(json!({
            "message_id": 45,
            "date": 1700000045,
            "chat": {"id": -1003, "type": "channel", "title": "channel"},
            "paid_media": {
                "star_count": 5,
                "paid_media": [{
                    "type": "photo",
                    "photo": [{
                        "file_id": "pm-1",
                        "file_unique_id": "pmu-1",
                        "width": 32,
                        "height": 32
                    }]
                }]
            },
            "checklist": {
                "title": "ops",
                "tasks": [{
                    "id": 1,
                    "text": "triage"
                }]
            },
            "invoice": {
                "title": "Premium",
                "description": "Subscription",
                "start_parameter": "start",
                "currency": "USD",
                "total_amount": 999
            },
            "successful_payment": {
                "currency": "USD",
                "total_amount": 999,
                "invoice_payload": "inv-1",
                "telegram_payment_charge_id": "tg-1",
                "provider_payment_charge_id": "prov-1"
            },
            "suggested_post_info": {
                "state": "approved",
                "price": {
                    "currency": "XTR",
                    "amount": 50
                },
                "send_date": 1700000800
            },
            "suggested_post_refunded": {
                "reason": "payment_refunded"
            }
        }))?;

        assert_eq!(message.kind(), MessageKind::PaidMedia);
        let paid_media = message.paid_media.as_ref().ok_or("missing paid media")?;
        assert_eq!(paid_media.star_count, 5);
        assert!(matches!(
            paid_media.paid_media.first(),
            Some(PaidMedia::Photo { .. })
        ));

        let checklist = message.checklist.as_ref().ok_or("missing checklist")?;
        assert_eq!(checklist.tasks.len(), 1);

        let invoice = message.invoice.as_ref().ok_or("missing invoice")?;
        assert_eq!(invoice.total_amount, 999);

        let payment = message
            .successful_payment
            .as_ref()
            .ok_or("missing successful payment")?;
        assert_eq!(payment.invoice_payload, "inv-1");

        let suggested = message
            .suggested_post_info
            .as_ref()
            .ok_or("missing suggested post info")?;
        assert_eq!(suggested.state, SuggestedPostState::Approved);
        assert_eq!(
            message
                .suggested_post_refunded
                .as_ref()
                .map(|value| &value.reason),
            Some(&SuggestedPostRefundReason::PaymentRefunded)
        );

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

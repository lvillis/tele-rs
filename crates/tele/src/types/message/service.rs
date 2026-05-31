use std::collections::BTreeMap;

use serde::Deserialize;
use serde_json::Value;

use crate::types::bot::User;
use crate::types::common::{MessageId, UserId};
use crate::types::tagged::{strip_type, tagged_kind};

use super::common::{Chat, MessageEntity, PhotoSize};
use super::content::ChecklistTask;
use super::media::Document;
use super::model::Message;
use super::reply::MaybeInaccessibleMessage;

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct DirectMessagesTopic {
    pub topic_id: i64,
    #[serde(default)]
    pub user: Option<User>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct ChecklistTasksDone {
    #[serde(default)]
    pub checklist_message: Option<Box<Message>>,
    #[serde(default)]
    pub marked_as_done_task_ids: Option<Vec<i64>>,
    #[serde(default)]
    pub marked_as_not_done_task_ids: Option<Vec<i64>>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct ChecklistTasksAdded {
    #[serde(default)]
    pub checklist_message: Option<Box<Message>>,
    pub tasks: Vec<ChecklistTask>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct ChatOwnerLeft {
    #[serde(default)]
    pub new_owner: Option<User>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct ChatOwnerChanged {
    pub new_owner: User,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct ProximityAlertTriggered {
    pub traveler: User,
    pub watcher: User,
    pub distance: u32,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct ChatBoostAdded {
    pub boost_count: u32,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct BackgroundFillSolid {
    pub color: u32,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct BackgroundFillGradient {
    pub top_color: u32,
    pub bottom_color: u32,
    pub rotation_angle: u32,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct BackgroundFillFreeformGradient {
    pub colors: Vec<u32>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum BackgroundFill {
    Solid(BackgroundFillSolid),
    Gradient(BackgroundFillGradient),
    FreeformGradient(BackgroundFillFreeformGradient),
    Unknown(Value),
}

impl BackgroundFill {
    pub fn kind(&self) -> Option<&str> {
        match self {
            Self::Solid(_) => Some("solid"),
            Self::Gradient(_) => Some("gradient"),
            Self::FreeformGradient(_) => Some("freeform_gradient"),
            Self::Unknown(value) => tagged_kind(value),
        }
    }

    pub fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown(_))
    }

    pub fn as_solid(&self) -> Option<&BackgroundFillSolid> {
        match self {
            Self::Solid(value) => Some(value),
            Self::Gradient(_) | Self::FreeformGradient(_) | Self::Unknown(_) => None,
        }
    }

    pub fn as_gradient(&self) -> Option<&BackgroundFillGradient> {
        match self {
            Self::Gradient(value) => Some(value),
            Self::Solid(_) | Self::FreeformGradient(_) | Self::Unknown(_) => None,
        }
    }

    pub fn as_freeform_gradient(&self) -> Option<&BackgroundFillFreeformGradient> {
        match self {
            Self::FreeformGradient(value) => Some(value),
            Self::Solid(_) | Self::Gradient(_) | Self::Unknown(_) => None,
        }
    }

    pub fn as_unknown_value(&self) -> Option<&Value> {
        match self {
            Self::Unknown(value) => Some(value),
            Self::Solid(_) | Self::Gradient(_) | Self::FreeformGradient(_) => None,
        }
    }

    pub fn into_unknown_value(self) -> Option<Value> {
        match self {
            Self::Unknown(value) => Some(value),
            Self::Solid(_) | Self::Gradient(_) | Self::FreeformGradient(_) => None,
        }
    }
}

impl<'de> Deserialize<'de> for BackgroundFill {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        match tagged_kind(&value) {
            Some("solid") => serde_json::from_value(strip_type(value))
                .map(Self::Solid)
                .map_err(serde::de::Error::custom),
            Some("gradient") => serde_json::from_value(strip_type(value))
                .map(Self::Gradient)
                .map_err(serde::de::Error::custom),
            Some("freeform_gradient") => serde_json::from_value(strip_type(value))
                .map(Self::FreeformGradient)
                .map_err(serde::de::Error::custom),
            Some(_) | None => Ok(Self::Unknown(value)),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct BackgroundTypeFill {
    pub fill: BackgroundFill,
    pub dark_theme_dimming: u32,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct BackgroundTypeWallpaper {
    pub document: Document,
    pub dark_theme_dimming: u32,
    #[serde(default)]
    pub is_blurred: bool,
    #[serde(default)]
    pub is_moving: bool,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct BackgroundTypePattern {
    pub document: Document,
    pub fill: BackgroundFill,
    pub intensity: u32,
    #[serde(default)]
    pub is_inverted: bool,
    #[serde(default)]
    pub is_moving: bool,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct BackgroundTypeChatTheme {
    pub theme_name: String,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum BackgroundType {
    Fill(BackgroundTypeFill),
    Wallpaper(Box<BackgroundTypeWallpaper>),
    Pattern(Box<BackgroundTypePattern>),
    ChatTheme(BackgroundTypeChatTheme),
    Unknown(Value),
}

impl BackgroundType {
    pub fn kind(&self) -> Option<&str> {
        match self {
            Self::Fill(_) => Some("fill"),
            Self::Wallpaper(_) => Some("wallpaper"),
            Self::Pattern(_) => Some("pattern"),
            Self::ChatTheme(_) => Some("chat_theme"),
            Self::Unknown(value) => tagged_kind(value),
        }
    }

    pub fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown(_))
    }

    pub fn as_fill(&self) -> Option<&BackgroundTypeFill> {
        match self {
            Self::Fill(value) => Some(value),
            Self::Wallpaper(_) | Self::Pattern(_) | Self::ChatTheme(_) | Self::Unknown(_) => None,
        }
    }

    pub fn as_wallpaper(&self) -> Option<&BackgroundTypeWallpaper> {
        match self {
            Self::Wallpaper(value) => Some(value),
            Self::Fill(_) | Self::Pattern(_) | Self::ChatTheme(_) | Self::Unknown(_) => None,
        }
    }

    pub fn as_pattern(&self) -> Option<&BackgroundTypePattern> {
        match self {
            Self::Pattern(value) => Some(value),
            Self::Fill(_) | Self::Wallpaper(_) | Self::ChatTheme(_) | Self::Unknown(_) => None,
        }
    }

    pub fn as_chat_theme(&self) -> Option<&BackgroundTypeChatTheme> {
        match self {
            Self::ChatTheme(value) => Some(value),
            Self::Fill(_) | Self::Wallpaper(_) | Self::Pattern(_) | Self::Unknown(_) => None,
        }
    }

    pub fn as_unknown_value(&self) -> Option<&Value> {
        match self {
            Self::Unknown(value) => Some(value),
            Self::Fill(_) | Self::Wallpaper(_) | Self::Pattern(_) | Self::ChatTheme(_) => None,
        }
    }

    pub fn into_unknown_value(self) -> Option<Value> {
        match self {
            Self::Unknown(value) => Some(value),
            Self::Fill(_) | Self::Wallpaper(_) | Self::Pattern(_) | Self::ChatTheme(_) => None,
        }
    }
}

impl<'de> Deserialize<'de> for BackgroundType {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        match tagged_kind(&value) {
            Some("fill") => serde_json::from_value(strip_type(value))
                .map(Self::Fill)
                .map_err(serde::de::Error::custom),
            Some("wallpaper") => serde_json::from_value(strip_type(value))
                .map(Box::new)
                .map(Self::Wallpaper)
                .map_err(serde::de::Error::custom),
            Some("pattern") => serde_json::from_value(strip_type(value))
                .map(Box::new)
                .map(Self::Pattern)
                .map_err(serde::de::Error::custom),
            Some("chat_theme") => serde_json::from_value(strip_type(value))
                .map(Self::ChatTheme)
                .map_err(serde::de::Error::custom),
            Some(_) | None => Ok(Self::Unknown(value)),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct ChatBackground {
    #[serde(rename = "type")]
    pub background_type: BackgroundType,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct VideoChatScheduled {
    pub start_date: i64,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[non_exhaustive]
pub struct VideoChatStarted {
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct VideoChatEnded {
    pub duration: u32,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct VideoChatParticipantsInvited {
    pub users: Vec<User>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct PaidMessagePriceChanged {
    pub paid_message_star_count: u64,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct DirectMessagePriceChanged {
    pub are_direct_messages_enabled: bool,
    #[serde(default)]
    pub direct_message_star_count: Option<u64>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct GiveawayCreated {
    #[serde(default)]
    pub prize_star_count: Option<u64>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct Giveaway {
    pub chats: Vec<Chat>,
    pub winners_selection_date: i64,
    pub winner_count: u32,
    #[serde(default)]
    pub only_new_members: bool,
    #[serde(default)]
    pub has_public_winners: bool,
    #[serde(default)]
    pub prize_description: Option<String>,
    #[serde(default)]
    pub country_codes: Option<Vec<String>>,
    #[serde(default)]
    pub prize_star_count: Option<u64>,
    #[serde(default)]
    pub premium_subscription_month_count: Option<u32>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct GiveawayWinners {
    pub chat: Chat,
    pub giveaway_message_id: MessageId,
    pub winners_selection_date: i64,
    pub winner_count: u32,
    pub winners: Vec<User>,
    #[serde(default)]
    pub additional_chat_count: Option<u32>,
    #[serde(default)]
    pub prize_star_count: Option<u64>,
    #[serde(default)]
    pub premium_subscription_month_count: Option<u32>,
    #[serde(default)]
    pub unclaimed_prize_count: Option<u32>,
    #[serde(default)]
    pub only_new_members: bool,
    #[serde(default)]
    pub was_refunded: bool,
    #[serde(default)]
    pub prize_description: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct GiveawayCompleted {
    pub winner_count: u32,
    #[serde(default)]
    pub unclaimed_prize_count: Option<u32>,
    #[serde(default)]
    pub giveaway_message: Option<Box<Message>>,
    #[serde(default)]
    pub is_star_giveaway: bool,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct ManagedBotCreated {
    pub bot: User,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct PollOptionAdded {
    #[serde(default)]
    pub poll_message: Option<Box<MaybeInaccessibleMessage>>,
    pub option_persistent_id: String,
    pub option_text: String,
    #[serde(default)]
    pub option_text_entities: Option<Vec<MessageEntity>>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct PollOptionDeleted {
    #[serde(default)]
    pub poll_message: Option<Box<MaybeInaccessibleMessage>>,
    pub option_persistent_id: String,
    pub option_text: String,
    #[serde(default)]
    pub option_text_entities: Option<Vec<MessageEntity>>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct WriteAccessAllowed {
    #[serde(default)]
    pub from_request: Option<bool>,
    #[serde(default)]
    pub web_app_name: Option<String>,
    #[serde(default)]
    pub from_attachment_menu: Option<bool>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct MessageAutoDeleteTimerChanged {
    pub message_auto_delete_time: u32,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct SharedUser {
    pub user_id: UserId,
    #[serde(default)]
    pub first_name: Option<String>,
    #[serde(default)]
    pub last_name: Option<String>,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub photo: Option<Vec<PhotoSize>>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct UsersShared {
    pub request_id: i64,
    pub users: Vec<SharedUser>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct ChatShared {
    pub request_id: i64,
    pub chat_id: i64,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub photo: Option<Vec<PhotoSize>>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

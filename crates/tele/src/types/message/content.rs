use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::bot::User;
use crate::types::common::UserId;

use super::common::{Chat, MessageEntity, PhotoSize};
use super::is_false;
use super::media::Animation;

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

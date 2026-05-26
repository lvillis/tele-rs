use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::bot::User;
use crate::types::common::UserId;

use super::common::{Chat, MessageEntity, PhotoSize};
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
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct PollOption {
    #[serde(default)]
    pub persistent_id: Option<String>,
    pub text: String,
    #[serde(default)]
    pub text_entities: Option<Vec<MessageEntity>>,
    #[serde(default)]
    pub media: Option<Value>,
    pub voter_count: u64,
    #[serde(default)]
    pub added_by_user: Option<User>,
    #[serde(default)]
    pub added_by_chat: Option<Chat>,
    #[serde(default)]
    pub addition_date: Option<i64>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram poll object.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct Poll {
    pub id: String,
    pub question: String,
    #[serde(default)]
    pub question_entities: Option<Vec<MessageEntity>>,
    pub options: Vec<PollOption>,
    pub total_voter_count: u64,
    pub is_closed: bool,
    pub is_anonymous: bool,
    #[serde(rename = "type")]
    pub kind: PollKind,
    pub allows_multiple_answers: bool,
    #[serde(default)]
    pub correct_option_ids: Option<Vec<u32>>,
    #[serde(default)]
    pub explanation: Option<String>,
    #[serde(default)]
    pub explanation_entities: Option<Vec<MessageEntity>>,
    #[serde(default)]
    pub open_period: Option<u32>,
    #[serde(default)]
    pub close_date: Option<i64>,
    #[serde(default)]
    pub allows_revoting: Option<bool>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram phone contact object.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct Contact {
    pub phone_number: String,
    pub first_name: String,
    #[serde(default)]
    pub last_name: Option<String>,
    #[serde(default)]
    pub user_id: Option<UserId>,
    #[serde(default)]
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
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct Dice {
    pub emoji: DiceEmoji,
    pub value: u8,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram geographic location object.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct Location {
    pub latitude: f64,
    pub longitude: f64,
    #[serde(default)]
    pub horizontal_accuracy: Option<f64>,
    #[serde(default)]
    pub live_period: Option<u32>,
    #[serde(default)]
    pub heading: Option<u16>,
    #[serde(default)]
    pub proximity_alert_radius: Option<u32>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram venue object.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct Venue {
    pub location: Location,
    pub title: String,
    pub address: String,
    #[serde(default)]
    pub foursquare_id: Option<String>,
    #[serde(default)]
    pub foursquare_type: Option<String>,
    #[serde(default)]
    pub google_place_id: Option<String>,
    #[serde(default)]
    pub google_place_type: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram checklist task.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct ChecklistTask {
    pub id: i64,
    pub text: String,
    #[serde(default)]
    pub text_entities: Option<Vec<MessageEntity>>,
    #[serde(default)]
    pub completed_by_user: Option<User>,
    #[serde(default)]
    pub completed_by_chat: Option<Chat>,
    #[serde(default)]
    pub completion_date: Option<i64>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram checklist payload.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct Checklist {
    pub title: String,
    #[serde(default)]
    pub title_entities: Option<Vec<MessageEntity>>,
    pub tasks: Vec<ChecklistTask>,
    #[serde(default)]
    pub others_can_add_tasks: bool,
    #[serde(default)]
    pub others_can_mark_tasks_as_done: bool,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram game payload.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct Game {
    pub title: String,
    pub description: String,
    pub photo: Vec<PhotoSize>,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub text_entities: Option<Vec<MessageEntity>>,
    #[serde(default)]
    pub animation: Option<Animation>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram game high score entry.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct GameHighScore {
    pub position: i64,
    pub user: User,
    pub score: i64,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn poll_preserves_future_fields() -> Result<(), Box<dyn std::error::Error>> {
        let option: PollOption = serde_json::from_value(json!({
            "text": "Choice",
            "voter_count": 3,
            "future": {"kept": true}
        }))?;
        assert_eq!(option.text, "Choice");
        assert_eq!(option.voter_count, 3);
        assert!(option.media.is_none());
        assert_eq!(option.extra["future"], json!({"kept": true}));

        let poll: Poll = serde_json::from_value(json!({
            "id": "poll-id",
            "question": "Question?",
            "options": [{"text": "Choice", "voter_count": 3}],
            "total_voter_count": 3,
            "is_closed": false,
            "is_anonymous": true,
            "type": "quiz",
            "allows_multiple_answers": false,
            "future": {"kept": true}
        }))?;
        assert_eq!(poll.id, "poll-id");
        assert_eq!(poll.question, "Question?");
        assert_eq!(poll.kind, PollKind::Quiz);
        assert!(poll.allows_revoting.is_none());
        assert_eq!(poll.extra["future"], json!({"kept": true}));

        Ok(())
    }

    #[test]
    fn contact_location_and_venue_preserve_future_fields() -> Result<(), Box<dyn std::error::Error>>
    {
        let contact: Contact = serde_json::from_value(json!({
            "phone_number": "+10000000000",
            "first_name": "Alice",
            "future": {"kept": true}
        }))?;
        assert_eq!(contact.phone_number, "+10000000000");
        assert_eq!(contact.first_name, "Alice");
        assert!(contact.vcard.is_none());
        assert_eq!(contact.extra["future"], json!({"kept": true}));

        let dice: Dice = serde_json::from_value(json!({
            "emoji": "🎲",
            "value": 6,
            "future": "kept"
        }))?;
        assert!(matches!(dice.emoji, DiceEmoji::Dice));
        assert_eq!(dice.value, 6);
        assert_eq!(dice.extra["future"], "kept");

        let location: Location = serde_json::from_value(json!({
            "latitude": 1.5,
            "longitude": 2.5,
            "future": {"kept": true}
        }))?;
        assert_eq!(location.latitude, 1.5);
        assert_eq!(location.longitude, 2.5);
        assert!(location.horizontal_accuracy.is_none());
        assert_eq!(location.extra["future"], json!({"kept": true}));

        let venue: Venue = serde_json::from_value(json!({
            "location": {"latitude": 1.5, "longitude": 2.5},
            "title": "Venue",
            "address": "Address",
            "future": {"kept": true}
        }))?;
        assert_eq!(venue.location.latitude, 1.5);
        assert_eq!(venue.title, "Venue");
        assert_eq!(venue.address, "Address");
        assert!(venue.foursquare_id.is_none());
        assert_eq!(venue.extra["future"], json!({"kept": true}));

        Ok(())
    }

    #[test]
    fn checklist_and_game_score_preserve_future_fields() -> Result<(), Box<dyn std::error::Error>> {
        let task: ChecklistTask = serde_json::from_value(json!({
            "id": 1,
            "text": "Task",
            "future": {"kept": true}
        }))?;
        assert_eq!(task.id, 1);
        assert_eq!(task.text, "Task");
        assert!(task.completion_date.is_none());
        assert_eq!(task.extra["future"], json!({"kept": true}));

        let checklist: Checklist = serde_json::from_value(json!({
            "title": "Checklist",
            "tasks": [{"id": 1, "text": "Task"}],
            "others_can_add_tasks": true,
            "future": {"kept": true}
        }))?;
        assert_eq!(checklist.title, "Checklist");
        assert_eq!(checklist.tasks.len(), 1);
        assert!(checklist.others_can_add_tasks);
        assert!(!checklist.others_can_mark_tasks_as_done);
        assert_eq!(checklist.extra["future"], json!({"kept": true}));

        let score: GameHighScore = serde_json::from_value(json!({
            "position": 1,
            "user": {"id": 1, "is_bot": false, "first_name": "Alice"},
            "score": 100,
            "future": {"kept": true}
        }))?;
        assert_eq!(score.position, 1);
        assert_eq!(score.user.id, UserId(1));
        assert_eq!(score.score, 100);
        assert_eq!(score.extra["future"], json!({"kept": true}));

        Ok(())
    }
}

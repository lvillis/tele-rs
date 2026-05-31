use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::bot::User;
use crate::types::common::UserId;
use crate::types::sticker::Sticker;

use super::common::{Chat, MessageEntity, PhotoSize};
use super::media::{Animation, Audio, Document, LivePhoto, Video};

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
    pub media: Option<PollMedia>,
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

/// Telegram media embedded in a poll description, quiz explanation, or poll option.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum PollMedia {
    Animation {
        animation: Animation,
        extra: BTreeMap<String, Value>,
    },
    Audio {
        audio: Audio,
        extra: BTreeMap<String, Value>,
    },
    Document {
        document: Document,
        extra: BTreeMap<String, Value>,
    },
    LivePhoto {
        live_photo: LivePhoto,
        extra: BTreeMap<String, Value>,
    },
    Location {
        location: Location,
        extra: BTreeMap<String, Value>,
    },
    Photo {
        photo: Vec<PhotoSize>,
        extra: BTreeMap<String, Value>,
    },
    Sticker {
        sticker: Sticker,
        extra: BTreeMap<String, Value>,
    },
    Venue {
        venue: Venue,
        extra: BTreeMap<String, Value>,
    },
    Video {
        video: Video,
        extra: BTreeMap<String, Value>,
    },
    Unknown(Value),
}

impl PollMedia {
    pub fn kind(&self) -> Option<&str> {
        match self {
            Self::Animation { .. } => Some("animation"),
            Self::Audio { .. } => Some("audio"),
            Self::Document { .. } => Some("document"),
            Self::LivePhoto { .. } => Some("live_photo"),
            Self::Location { .. } => Some("location"),
            Self::Photo { .. } => Some("photo"),
            Self::Sticker { .. } => Some("sticker"),
            Self::Venue { .. } => Some("venue"),
            Self::Video { .. } => Some("video"),
            Self::Unknown(value) => unknown_poll_media_kind(value),
        }
    }

    pub fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown(_))
    }

    pub fn as_animation(&self) -> Option<&Animation> {
        match self {
            Self::Animation { animation, .. } => Some(animation),
            _ => None,
        }
    }

    pub fn as_audio(&self) -> Option<&Audio> {
        match self {
            Self::Audio { audio, .. } => Some(audio),
            _ => None,
        }
    }

    pub fn as_document(&self) -> Option<&Document> {
        match self {
            Self::Document { document, .. } => Some(document),
            _ => None,
        }
    }

    pub fn as_live_photo(&self) -> Option<&LivePhoto> {
        match self {
            Self::LivePhoto { live_photo, .. } => Some(live_photo),
            _ => None,
        }
    }

    pub fn as_location(&self) -> Option<&Location> {
        match self {
            Self::Location { location, .. } => Some(location),
            _ => None,
        }
    }

    pub fn as_photo(&self) -> Option<&[PhotoSize]> {
        match self {
            Self::Photo { photo, .. } => Some(photo),
            _ => None,
        }
    }

    pub fn as_sticker(&self) -> Option<&Sticker> {
        match self {
            Self::Sticker { sticker, .. } => Some(sticker),
            _ => None,
        }
    }

    pub fn as_venue(&self) -> Option<&Venue> {
        match self {
            Self::Venue { venue, .. } => Some(venue),
            _ => None,
        }
    }

    pub fn as_video(&self) -> Option<&Video> {
        match self {
            Self::Video { video, .. } => Some(video),
            _ => None,
        }
    }

    pub fn extra(&self) -> Option<&BTreeMap<String, Value>> {
        match self {
            Self::Animation { extra, .. }
            | Self::Audio { extra, .. }
            | Self::Document { extra, .. }
            | Self::LivePhoto { extra, .. }
            | Self::Location { extra, .. }
            | Self::Photo { extra, .. }
            | Self::Sticker { extra, .. }
            | Self::Venue { extra, .. }
            | Self::Video { extra, .. } => Some(extra),
            Self::Unknown(_) => None,
        }
    }

    pub fn as_unknown_value(&self) -> Option<&Value> {
        match self {
            Self::Unknown(value) => Some(value),
            _ => None,
        }
    }

    pub fn into_unknown_value(self) -> Option<Value> {
        match self {
            Self::Unknown(value) => Some(value),
            _ => None,
        }
    }
}

impl<'de> Deserialize<'de> for PollMedia {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        let Some(object) = value.as_object() else {
            return Ok(Self::Unknown(value));
        };

        if let Some(payload) = object.get("animation") {
            return serde_json::from_value(payload.clone())
                .map(|animation| Self::Animation {
                    animation,
                    extra: poll_media_extra(object, "animation"),
                })
                .map_err(serde::de::Error::custom);
        }
        if let Some(payload) = object.get("audio") {
            return serde_json::from_value(payload.clone())
                .map(|audio| Self::Audio {
                    audio,
                    extra: poll_media_extra(object, "audio"),
                })
                .map_err(serde::de::Error::custom);
        }
        if let Some(payload) = object.get("document") {
            return serde_json::from_value(payload.clone())
                .map(|document| Self::Document {
                    document,
                    extra: poll_media_extra(object, "document"),
                })
                .map_err(serde::de::Error::custom);
        }
        if let Some(payload) = object.get("live_photo") {
            return serde_json::from_value(payload.clone())
                .map(|live_photo| Self::LivePhoto {
                    live_photo,
                    extra: poll_media_extra(object, "live_photo"),
                })
                .map_err(serde::de::Error::custom);
        }
        if let Some(payload) = object.get("location") {
            return serde_json::from_value(payload.clone())
                .map(|location| Self::Location {
                    location,
                    extra: poll_media_extra(object, "location"),
                })
                .map_err(serde::de::Error::custom);
        }
        if let Some(payload) = object.get("photo") {
            return serde_json::from_value(payload.clone())
                .map(|photo| Self::Photo {
                    photo,
                    extra: poll_media_extra(object, "photo"),
                })
                .map_err(serde::de::Error::custom);
        }
        if let Some(payload) = object.get("sticker") {
            return serde_json::from_value(payload.clone())
                .map(|sticker| Self::Sticker {
                    sticker,
                    extra: poll_media_extra(object, "sticker"),
                })
                .map_err(serde::de::Error::custom);
        }
        if let Some(payload) = object.get("venue") {
            return serde_json::from_value(payload.clone())
                .map(|venue| Self::Venue {
                    venue,
                    extra: poll_media_extra(object, "venue"),
                })
                .map_err(serde::de::Error::custom);
        }
        if let Some(payload) = object.get("video") {
            return serde_json::from_value(payload.clone())
                .map(|video| Self::Video {
                    video,
                    extra: poll_media_extra(object, "video"),
                })
                .map_err(serde::de::Error::custom);
        }

        Ok(Self::Unknown(value))
    }
}

fn poll_media_extra(
    object: &serde_json::Map<String, Value>,
    media_key: &str,
) -> BTreeMap<String, Value> {
    object
        .iter()
        .filter(|(key, _)| key.as_str() != media_key)
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect()
}

fn unknown_poll_media_kind(value: &Value) -> Option<&str> {
    let object = value.as_object()?;
    if object.len() == 1 {
        object.keys().next().map(String::as_str)
    } else {
        None
    }
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
    pub explanation_media: Option<PollMedia>,
    #[serde(default)]
    pub open_period: Option<u32>,
    #[serde(default)]
    pub close_date: Option<i64>,
    #[serde(default)]
    pub allows_revoting: Option<bool>,
    #[serde(default)]
    pub members_only: Option<bool>,
    #[serde(default)]
    pub country_codes: Option<Vec<String>>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub description_entities: Option<Vec<MessageEntity>>,
    #[serde(default)]
    pub media: Option<PollMedia>,
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
            "media": {
                "sticker": {
                    "file_id": "sticker-1",
                    "file_unique_id": "sticker-u-1",
                    "type": "regular",
                    "width": 64,
                    "height": 64,
                    "is_animated": false,
                    "is_video": false
                },
                "future": {"kept": true}
            },
            "voter_count": 3,
            "future": {"kept": true}
        }))?;
        assert_eq!(option.text, "Choice");
        assert_eq!(option.voter_count, 3);
        let option_media = option.media.as_ref().ok_or("missing option media")?;
        assert_eq!(option_media.kind(), Some("sticker"));
        assert!(option_media.as_sticker().is_some());
        assert_eq!(
            option_media.extra().and_then(|extra| extra.get("future")),
            Some(&json!({"kept": true}))
        );
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
            "members_only": true,
            "country_codes": ["US", "FT"],
            "description": "Choose carefully",
            "description_entities": [{"type": "bold", "offset": 0, "length": 6}],
            "media": {
                "location": {
                    "latitude": 1.5,
                    "longitude": 2.5
                }
            },
            "explanation_media": {
                "photo": [{
                    "file_id": "photo-1",
                    "file_unique_id": "photo-u-1",
                    "width": 16,
                    "height": 16
                }]
            },
            "future": {"kept": true}
        }))?;
        assert_eq!(poll.id, "poll-id");
        assert_eq!(poll.question, "Question?");
        assert_eq!(poll.kind, PollKind::Quiz);
        assert!(poll.allows_revoting.is_none());
        assert_eq!(poll.members_only, Some(true));
        assert_eq!(
            poll.country_codes.as_deref(),
            Some(&["US".to_owned(), "FT".to_owned()][..])
        );
        assert_eq!(poll.description.as_deref(), Some("Choose carefully"));
        assert_eq!(
            poll.media.as_ref().and_then(PollMedia::kind),
            Some("location")
        );
        assert_eq!(
            poll.explanation_media.as_ref().and_then(PollMedia::kind),
            Some("photo")
        );
        assert_eq!(poll.extra["future"], json!({"kept": true}));

        let unknown_media: PollMedia = serde_json::from_value(json!({
            "future_media": {"kept": true}
        }))?;
        assert!(unknown_media.is_unknown());
        assert_eq!(unknown_media.kind(), Some("future_media"));
        assert_eq!(
            unknown_media
                .as_unknown_value()
                .and_then(|value| value.get("future_media")),
            Some(&json!({"kept": true}))
        );

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

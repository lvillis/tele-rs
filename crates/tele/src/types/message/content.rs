use std::collections::BTreeMap;

use serde::ser::SerializeMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::bot::User;
use crate::types::common::UserId;
use crate::types::extra::{
    field_len as extra_field_len, serialize_fields as serialize_extra_fields,
    serialize_optional_field,
};

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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub persistent_id: Option<String>,
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_entities: Option<Vec<MessageEntity>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub media: Option<Value>,
    pub voter_count: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub added_by_user: Option<User>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub added_by_chat: Option<Chat>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub addition_date: Option<i64>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl Serialize for PollOption {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = [
            "persistent_id",
            "text",
            "text_entities",
            "media",
            "voter_count",
            "added_by_user",
            "added_by_chat",
            "addition_date",
        ];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let optional_len = usize::from(self.persistent_id.is_some())
            + usize::from(self.text_entities.is_some())
            + usize::from(self.media.is_some())
            + usize::from(self.added_by_user.is_some())
            + usize::from(self.added_by_chat.is_some())
            + usize::from(self.addition_date.is_some());
        let mut object = serializer.serialize_map(Some(extra_len + optional_len + 2))?;
        serialize_optional_field(&mut object, "persistent_id", &self.persistent_id)?;
        object.serialize_entry("text", &self.text)?;
        serialize_optional_field(&mut object, "text_entities", &self.text_entities)?;
        serialize_optional_field(&mut object, "media", &self.media)?;
        object.serialize_entry("voter_count", &self.voter_count)?;
        serialize_optional_field(&mut object, "added_by_user", &self.added_by_user)?;
        serialize_optional_field(&mut object, "added_by_chat", &self.added_by_chat)?;
        serialize_optional_field(&mut object, "addition_date", &self.addition_date)?;
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
}

/// Telegram poll object.
#[derive(Clone, Debug, Deserialize)]
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
    pub correct_option_ids: Option<Vec<u32>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub explanation: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub explanation_entities: Option<Vec<MessageEntity>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub open_period: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub close_date: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allows_revoting: Option<bool>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl Serialize for Poll {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = [
            "id",
            "question",
            "question_entities",
            "options",
            "total_voter_count",
            "is_closed",
            "is_anonymous",
            "type",
            "allows_multiple_answers",
            "correct_option_ids",
            "explanation",
            "explanation_entities",
            "open_period",
            "close_date",
            "allows_revoting",
        ];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let optional_len = usize::from(self.question_entities.is_some())
            + usize::from(self.correct_option_ids.is_some())
            + usize::from(self.explanation.is_some())
            + usize::from(self.explanation_entities.is_some())
            + usize::from(self.open_period.is_some())
            + usize::from(self.close_date.is_some())
            + usize::from(self.allows_revoting.is_some());
        let mut object = serializer.serialize_map(Some(extra_len + optional_len + 8))?;
        object.serialize_entry("id", &self.id)?;
        object.serialize_entry("question", &self.question)?;
        serialize_optional_field(&mut object, "question_entities", &self.question_entities)?;
        object.serialize_entry("options", &self.options)?;
        object.serialize_entry("total_voter_count", &self.total_voter_count)?;
        object.serialize_entry("is_closed", &self.is_closed)?;
        object.serialize_entry("is_anonymous", &self.is_anonymous)?;
        object.serialize_entry("type", &self.kind)?;
        object.serialize_entry("allows_multiple_answers", &self.allows_multiple_answers)?;
        serialize_optional_field(&mut object, "correct_option_ids", &self.correct_option_ids)?;
        serialize_optional_field(&mut object, "explanation", &self.explanation)?;
        serialize_optional_field(
            &mut object,
            "explanation_entities",
            &self.explanation_entities,
        )?;
        serialize_optional_field(&mut object, "open_period", &self.open_period)?;
        serialize_optional_field(&mut object, "close_date", &self.close_date)?;
        serialize_optional_field(&mut object, "allows_revoting", &self.allows_revoting)?;
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
}

/// Telegram phone contact object.
#[derive(Clone, Debug, Deserialize)]
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

impl Serialize for Contact {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = [
            "phone_number",
            "first_name",
            "last_name",
            "user_id",
            "vcard",
        ];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let optional_len = usize::from(self.last_name.is_some())
            + usize::from(self.user_id.is_some())
            + usize::from(self.vcard.is_some());
        let mut object = serializer.serialize_map(Some(extra_len + optional_len + 2))?;
        object.serialize_entry("phone_number", &self.phone_number)?;
        object.serialize_entry("first_name", &self.first_name)?;
        serialize_optional_field(&mut object, "last_name", &self.last_name)?;
        serialize_optional_field(&mut object, "user_id", &self.user_id)?;
        serialize_optional_field(&mut object, "vcard", &self.vcard)?;
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
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

/// Telegram animated dice object.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct Dice {
    pub emoji: DiceEmoji,
    pub value: u8,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl Serialize for Dice {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = ["emoji", "value"];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let mut object = serializer.serialize_map(Some(extra_len + 2))?;
        object.serialize_entry("emoji", &self.emoji)?;
        object.serialize_entry("value", &self.value)?;
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
}

/// Telegram geographic location object.
#[derive(Clone, Debug, Deserialize)]
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

impl Serialize for Location {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = [
            "latitude",
            "longitude",
            "horizontal_accuracy",
            "live_period",
            "heading",
            "proximity_alert_radius",
        ];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let optional_len = usize::from(self.horizontal_accuracy.is_some())
            + usize::from(self.live_period.is_some())
            + usize::from(self.heading.is_some())
            + usize::from(self.proximity_alert_radius.is_some());
        let mut object = serializer.serialize_map(Some(extra_len + optional_len + 2))?;
        object.serialize_entry("latitude", &self.latitude)?;
        object.serialize_entry("longitude", &self.longitude)?;
        serialize_optional_field(
            &mut object,
            "horizontal_accuracy",
            &self.horizontal_accuracy,
        )?;
        serialize_optional_field(&mut object, "live_period", &self.live_period)?;
        serialize_optional_field(&mut object, "heading", &self.heading)?;
        serialize_optional_field(
            &mut object,
            "proximity_alert_radius",
            &self.proximity_alert_radius,
        )?;
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
}

/// Telegram venue object.
#[derive(Clone, Debug, Deserialize)]
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

impl Serialize for Venue {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = [
            "location",
            "title",
            "address",
            "foursquare_id",
            "foursquare_type",
            "google_place_id",
            "google_place_type",
        ];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let optional_len = usize::from(self.foursquare_id.is_some())
            + usize::from(self.foursquare_type.is_some())
            + usize::from(self.google_place_id.is_some())
            + usize::from(self.google_place_type.is_some());
        let mut object = serializer.serialize_map(Some(extra_len + optional_len + 3))?;
        object.serialize_entry("location", &self.location)?;
        object.serialize_entry("title", &self.title)?;
        object.serialize_entry("address", &self.address)?;
        serialize_optional_field(&mut object, "foursquare_id", &self.foursquare_id)?;
        serialize_optional_field(&mut object, "foursquare_type", &self.foursquare_type)?;
        serialize_optional_field(&mut object, "google_place_id", &self.google_place_id)?;
        serialize_optional_field(&mut object, "google_place_type", &self.google_place_type)?;
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
}

/// Telegram checklist task.
#[derive(Clone, Debug, Deserialize)]
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

impl Serialize for ChecklistTask {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = [
            "id",
            "text",
            "text_entities",
            "completed_by_user",
            "completed_by_chat",
            "completion_date",
        ];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let optional_len = usize::from(self.text_entities.is_some())
            + usize::from(self.completed_by_user.is_some())
            + usize::from(self.completed_by_chat.is_some())
            + usize::from(self.completion_date.is_some());
        let mut object = serializer.serialize_map(Some(extra_len + optional_len + 2))?;
        object.serialize_entry("id", &self.id)?;
        object.serialize_entry("text", &self.text)?;
        serialize_optional_field(&mut object, "text_entities", &self.text_entities)?;
        serialize_optional_field(&mut object, "completed_by_user", &self.completed_by_user)?;
        serialize_optional_field(&mut object, "completed_by_chat", &self.completed_by_chat)?;
        serialize_optional_field(&mut object, "completion_date", &self.completion_date)?;
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
}

/// Telegram checklist payload.
#[derive(Clone, Debug, Deserialize)]
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

impl Serialize for Checklist {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = [
            "title",
            "title_entities",
            "tasks",
            "others_can_add_tasks",
            "others_can_mark_tasks_as_done",
        ];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let optional_len = usize::from(self.title_entities.is_some())
            + usize::from(self.others_can_add_tasks)
            + usize::from(self.others_can_mark_tasks_as_done);
        let mut object = serializer.serialize_map(Some(extra_len + optional_len + 2))?;
        object.serialize_entry("title", &self.title)?;
        serialize_optional_field(&mut object, "title_entities", &self.title_entities)?;
        object.serialize_entry("tasks", &self.tasks)?;
        if self.others_can_add_tasks {
            object.serialize_entry("others_can_add_tasks", &self.others_can_add_tasks)?;
        }
        if self.others_can_mark_tasks_as_done {
            object.serialize_entry(
                "others_can_mark_tasks_as_done",
                &self.others_can_mark_tasks_as_done,
            )?;
        }
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
}

/// Telegram game payload.
#[derive(Clone, Debug, Deserialize)]
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

impl Serialize for Game {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = [
            "title",
            "description",
            "photo",
            "text",
            "text_entities",
            "animation",
        ];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let optional_len = usize::from(self.text.is_some())
            + usize::from(self.text_entities.is_some())
            + usize::from(self.animation.is_some());
        let mut object = serializer.serialize_map(Some(extra_len + optional_len + 3))?;
        object.serialize_entry("title", &self.title)?;
        object.serialize_entry("description", &self.description)?;
        object.serialize_entry("photo", &self.photo)?;
        serialize_optional_field(&mut object, "text", &self.text)?;
        serialize_optional_field(&mut object, "text_entities", &self.text_entities)?;
        serialize_optional_field(&mut object, "animation", &self.animation)?;
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
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

impl Serialize for GameHighScore {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = ["position", "user", "score"];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let mut object = serializer.serialize_map(Some(extra_len + 3))?;
        object.serialize_entry("position", &self.position)?;
        object.serialize_entry("user", &self.user)?;
        object.serialize_entry("score", &self.score)?;
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use serde_json::json;

    use super::*;

    #[test]
    fn poll_extra_cannot_override_reserved_fields() -> Result<(), Box<dyn std::error::Error>> {
        let mut option: PollOption = serde_json::from_value(json!({
            "text": "Choice",
            "voter_count": 3,
            "future": {"kept": true}
        }))?;
        option.extra.insert("text".to_owned(), json!("spoofed"));
        option.extra.insert("voter_count".to_owned(), json!(0));
        option.extra.insert("media".to_owned(), json!("spoofed"));
        option
            .extra
            .insert("another_future".to_owned(), json!("kept"));

        let option_value = serde_json::to_value(option)?;
        assert_eq!(option_value["text"], "Choice");
        assert_eq!(option_value["voter_count"], 3);
        assert!(option_value.get("media").is_none());
        assert_eq!(option_value["future"], json!({"kept": true}));
        assert_eq!(option_value["another_future"], "kept");

        let mut poll: Poll = serde_json::from_value(json!({
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
        poll.extra.insert("id".to_owned(), json!("spoofed"));
        poll.extra.insert("question".to_owned(), json!("spoofed"));
        poll.extra.insert("type".to_owned(), json!("regular"));
        poll.extra.insert("allows_revoting".to_owned(), json!(true));
        poll.extra
            .insert("another_future".to_owned(), json!("kept"));

        let poll_value = serde_json::to_value(poll)?;
        assert_eq!(poll_value["id"], "poll-id");
        assert_eq!(poll_value["question"], "Question?");
        assert_eq!(poll_value["type"], "quiz");
        assert!(poll_value.get("allows_revoting").is_none());
        assert_eq!(poll_value["future"], json!({"kept": true}));
        assert_eq!(poll_value["another_future"], "kept");

        Ok(())
    }

    #[test]
    fn contact_location_and_venue_extra_cannot_override_reserved_fields()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut contact: Contact = serde_json::from_value(json!({
            "phone_number": "+10000000000",
            "first_name": "Alice",
            "future": {"kept": true}
        }))?;
        contact
            .extra
            .insert("phone_number".to_owned(), json!("spoofed"));
        contact
            .extra
            .insert("first_name".to_owned(), json!("spoofed"));
        contact.extra.insert("vcard".to_owned(), json!("spoofed"));
        contact
            .extra
            .insert("another_future".to_owned(), json!("kept"));

        let contact_value = serde_json::to_value(contact)?;
        assert_eq!(contact_value["phone_number"], "+10000000000");
        assert_eq!(contact_value["first_name"], "Alice");
        assert!(contact_value.get("vcard").is_none());
        assert_eq!(contact_value["future"], json!({"kept": true}));
        assert_eq!(contact_value["another_future"], "kept");

        let mut dice = Dice {
            emoji: DiceEmoji::Dice,
            value: 6,
            extra: BTreeMap::from([
                ("emoji".to_owned(), json!("spoofed")),
                ("value".to_owned(), json!(1)),
                ("future".to_owned(), json!("kept")),
            ]),
        };
        dice.extra
            .insert("another_future".to_owned(), json!("kept"));

        let dice_value = serde_json::to_value(dice)?;
        assert_eq!(dice_value["emoji"], serde_json::to_value(DiceEmoji::Dice)?);
        assert_eq!(dice_value["value"], 6);
        assert_eq!(dice_value["future"], "kept");
        assert_eq!(dice_value["another_future"], "kept");

        let mut location: Location = serde_json::from_value(json!({
            "latitude": 1.5,
            "longitude": 2.5,
            "future": {"kept": true}
        }))?;
        location.extra.insert("latitude".to_owned(), json!(9.0));
        location.extra.insert("longitude".to_owned(), json!(9.0));
        location
            .extra
            .insert("horizontal_accuracy".to_owned(), json!(1.0));
        location
            .extra
            .insert("another_future".to_owned(), json!("kept"));

        let location_value = serde_json::to_value(location)?;
        assert_eq!(location_value["latitude"], 1.5);
        assert_eq!(location_value["longitude"], 2.5);
        assert!(location_value.get("horizontal_accuracy").is_none());
        assert_eq!(location_value["future"], json!({"kept": true}));
        assert_eq!(location_value["another_future"], "kept");

        let mut venue: Venue = serde_json::from_value(json!({
            "location": {"latitude": 1.5, "longitude": 2.5},
            "title": "Venue",
            "address": "Address",
            "future": {"kept": true}
        }))?;
        venue.extra.insert(
            "location".to_owned(),
            json!({"latitude": 9.0, "longitude": 9.0}),
        );
        venue.extra.insert("title".to_owned(), json!("spoofed"));
        venue.extra.insert("address".to_owned(), json!("spoofed"));
        venue
            .extra
            .insert("foursquare_id".to_owned(), json!("spoofed"));
        venue
            .extra
            .insert("another_future".to_owned(), json!("kept"));

        let venue_value = serde_json::to_value(venue)?;
        assert_eq!(venue_value["location"]["latitude"], 1.5);
        assert_eq!(venue_value["title"], "Venue");
        assert_eq!(venue_value["address"], "Address");
        assert!(venue_value.get("foursquare_id").is_none());
        assert_eq!(venue_value["future"], json!({"kept": true}));
        assert_eq!(venue_value["another_future"], "kept");

        Ok(())
    }

    #[test]
    fn checklist_and_game_extra_cannot_override_reserved_fields()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut task: ChecklistTask = serde_json::from_value(json!({
            "id": 1,
            "text": "Task",
            "future": {"kept": true}
        }))?;
        task.extra.insert("id".to_owned(), json!(2));
        task.extra.insert("text".to_owned(), json!("spoofed"));
        task.extra.insert("completion_date".to_owned(), json!(10));
        task.extra
            .insert("another_future".to_owned(), json!("kept"));

        let task_value = serde_json::to_value(task)?;
        assert_eq!(task_value["id"], 1);
        assert_eq!(task_value["text"], "Task");
        assert!(task_value.get("completion_date").is_none());
        assert_eq!(task_value["future"], json!({"kept": true}));
        assert_eq!(task_value["another_future"], "kept");

        let mut checklist: Checklist = serde_json::from_value(json!({
            "title": "Checklist",
            "tasks": [{"id": 1, "text": "Task"}],
            "others_can_add_tasks": true,
            "future": {"kept": true}
        }))?;
        checklist.extra.insert("title".to_owned(), json!("spoofed"));
        checklist.extra.insert("tasks".to_owned(), json!([]));
        checklist
            .extra
            .insert("others_can_add_tasks".to_owned(), json!(false));
        checklist
            .extra
            .insert("others_can_mark_tasks_as_done".to_owned(), json!(true));
        checklist
            .extra
            .insert("another_future".to_owned(), json!("kept"));

        let checklist_value = serde_json::to_value(checklist)?;
        assert_eq!(checklist_value["title"], "Checklist");
        assert_eq!(checklist_value["tasks"].as_array().map(Vec::len), Some(1));
        assert_eq!(checklist_value["others_can_add_tasks"], true);
        assert!(
            checklist_value
                .get("others_can_mark_tasks_as_done")
                .is_none()
        );
        assert_eq!(checklist_value["future"], json!({"kept": true}));
        assert_eq!(checklist_value["another_future"], "kept");

        let mut game: Game = serde_json::from_value(json!({
            "title": "Game",
            "description": "Description",
            "photo": [{
                "file_id": "file",
                "file_unique_id": "unique",
                "width": 1,
                "height": 1
            }],
            "future": {"kept": true}
        }))?;
        game.extra.insert("title".to_owned(), json!("spoofed"));
        game.extra
            .insert("description".to_owned(), json!("spoofed"));
        game.extra.insert("photo".to_owned(), json!([]));
        game.extra.insert("text".to_owned(), json!("spoofed"));
        game.extra
            .insert("another_future".to_owned(), json!("kept"));

        let game_value = serde_json::to_value(game)?;
        assert_eq!(game_value["title"], "Game");
        assert_eq!(game_value["description"], "Description");
        assert_eq!(game_value["photo"].as_array().map(Vec::len), Some(1));
        assert!(game_value.get("text").is_none());
        assert_eq!(game_value["future"], json!({"kept": true}));
        assert_eq!(game_value["another_future"], "kept");

        let mut score: GameHighScore = serde_json::from_value(json!({
            "position": 1,
            "user": {"id": 1, "is_bot": false, "first_name": "Alice"},
            "score": 100,
            "future": {"kept": true}
        }))?;
        score.extra.insert("position".to_owned(), json!(2));
        score.extra.insert(
            "user".to_owned(),
            json!({"id": 2, "is_bot": true, "first_name": "spoofed"}),
        );
        score.extra.insert("score".to_owned(), json!(1));
        score
            .extra
            .insert("another_future".to_owned(), json!("kept"));

        let score_value = serde_json::to_value(score)?;
        assert_eq!(score_value["position"], 1);
        assert_eq!(score_value["user"]["id"], 1);
        assert_eq!(score_value["score"], 100);
        assert_eq!(score_value["future"], json!({"kept": true}));
        assert_eq!(score_value["another_future"], "kept");

        Ok(())
    }
}

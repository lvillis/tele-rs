use std::collections::BTreeMap;

use serde::ser::SerializeMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::common::MessageId;
use crate::types::extra::{
    field_len as extra_field_len, serialize_fields as serialize_extra_fields,
    serialize_optional_field,
};
use crate::types::sticker::Sticker;
use crate::types::telegram::LinkPreviewOptions;

use super::common::{Chat, MessageEntity, MessageOrigin, PhotoSize};
use super::content::{Checklist, Contact, Dice, Game, Location, Poll, Venue};
use super::media::{Animation, Audio, Document, PaidMediaInfo, Story, Video, VideoNote, Voice};
use super::model::Message;
use super::payments::Invoice;
use super::service::{Giveaway, GiveawayWinners};

#[derive(Clone, Debug, Deserialize)]
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

impl Serialize for TextQuote {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = ["text", "entities", "position", "is_manual"];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let optional_len = usize::from(self.entities.is_some()) + usize::from(self.is_manual);
        let mut object = serializer.serialize_map(Some(extra_len + optional_len + 2))?;
        object.serialize_entry("text", &self.text)?;
        serialize_optional_field(&mut object, "entities", &self.entities)?;
        object.serialize_entry("position", &self.position)?;
        if self.is_manual {
            object.serialize_entry("is_manual", &self.is_manual)?;
        }
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
}

#[derive(Clone, Debug, Deserialize)]
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

impl Serialize for ExternalReplyInfo {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = [
            "origin",
            "chat",
            "message_id",
            "link_preview_options",
            "animation",
            "audio",
            "document",
            "paid_media",
            "photo",
            "sticker",
            "story",
            "video",
            "video_note",
            "voice",
            "has_media_spoiler",
            "checklist",
            "contact",
            "dice",
            "game",
            "giveaway",
            "giveaway_winners",
            "invoice",
            "location",
            "poll",
            "venue",
        ];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let optional_len = usize::from(self.chat.is_some())
            + usize::from(self.message_id.is_some())
            + usize::from(self.link_preview_options.is_some())
            + usize::from(self.animation.is_some())
            + usize::from(self.audio.is_some())
            + usize::from(self.document.is_some())
            + usize::from(self.paid_media.is_some())
            + usize::from(self.photo.is_some())
            + usize::from(self.sticker.is_some())
            + usize::from(self.story.is_some())
            + usize::from(self.video.is_some())
            + usize::from(self.video_note.is_some())
            + usize::from(self.voice.is_some())
            + usize::from(self.has_media_spoiler)
            + usize::from(self.checklist.is_some())
            + usize::from(self.contact.is_some())
            + usize::from(self.dice.is_some())
            + usize::from(self.game.is_some())
            + usize::from(self.giveaway.is_some())
            + usize::from(self.giveaway_winners.is_some())
            + usize::from(self.invoice.is_some())
            + usize::from(self.location.is_some())
            + usize::from(self.poll.is_some())
            + usize::from(self.venue.is_some());
        let mut object = serializer.serialize_map(Some(extra_len + optional_len + 1))?;
        object.serialize_entry("origin", &self.origin)?;
        serialize_optional_field(&mut object, "chat", &self.chat)?;
        serialize_optional_field(&mut object, "message_id", &self.message_id)?;
        serialize_optional_field(
            &mut object,
            "link_preview_options",
            &self.link_preview_options,
        )?;
        serialize_optional_field(&mut object, "animation", &self.animation)?;
        serialize_optional_field(&mut object, "audio", &self.audio)?;
        serialize_optional_field(&mut object, "document", &self.document)?;
        serialize_optional_field(&mut object, "paid_media", &self.paid_media)?;
        serialize_optional_field(&mut object, "photo", &self.photo)?;
        serialize_optional_field(&mut object, "sticker", &self.sticker)?;
        serialize_optional_field(&mut object, "story", &self.story)?;
        serialize_optional_field(&mut object, "video", &self.video)?;
        serialize_optional_field(&mut object, "video_note", &self.video_note)?;
        serialize_optional_field(&mut object, "voice", &self.voice)?;
        if self.has_media_spoiler {
            object.serialize_entry("has_media_spoiler", &self.has_media_spoiler)?;
        }
        serialize_optional_field(&mut object, "checklist", &self.checklist)?;
        serialize_optional_field(&mut object, "contact", &self.contact)?;
        serialize_optional_field(&mut object, "dice", &self.dice)?;
        serialize_optional_field(&mut object, "game", &self.game)?;
        serialize_optional_field(&mut object, "giveaway", &self.giveaway)?;
        serialize_optional_field(&mut object, "giveaway_winners", &self.giveaway_winners)?;
        serialize_optional_field(&mut object, "invoice", &self.invoice)?;
        serialize_optional_field(&mut object, "location", &self.location)?;
        serialize_optional_field(&mut object, "poll", &self.poll)?;
        serialize_optional_field(&mut object, "venue", &self.venue)?;
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
}

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct InaccessibleMessage {
    pub chat: Chat,
    pub message_id: MessageId,
    pub date: i64,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl Serialize for InaccessibleMessage {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = ["chat", "message_id", "date"];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let mut object = serializer.serialize_map(Some(extra_len + 3))?;
        object.serialize_entry("chat", &self.chat)?;
        object.serialize_entry("message_id", &self.message_id)?;
        object.serialize_entry("date", &self.date)?;
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
}

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum MaybeInaccessibleMessage {
    Accessible(Box<Message>),
    Inaccessible(InaccessibleMessage),
}

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

impl MaybeInaccessibleMessage {
    pub fn is_accessible(&self) -> bool {
        matches!(self, Self::Accessible(_))
    }

    pub fn is_inaccessible(&self) -> bool {
        matches!(self, Self::Inaccessible(_))
    }

    pub fn as_accessible(&self) -> Option<&Message> {
        match self {
            Self::Accessible(message) => Some(message.as_ref()),
            Self::Inaccessible(_) => None,
        }
    }

    pub fn into_accessible(self) -> Option<Message> {
        match self {
            Self::Accessible(message) => Some(*message),
            Self::Inaccessible(_) => None,
        }
    }

    pub fn as_inaccessible(&self) -> Option<&InaccessibleMessage> {
        match self {
            Self::Accessible(_) => None,
            Self::Inaccessible(message) => Some(message),
        }
    }

    pub fn into_inaccessible(self) -> Option<InaccessibleMessage> {
        match self {
            Self::Accessible(_) => None,
            Self::Inaccessible(message) => Some(message),
        }
    }

    pub fn accessible(&self) -> Option<&Message> {
        self.as_accessible()
    }

    pub fn inaccessible(&self) -> Option<&InaccessibleMessage> {
        self.as_inaccessible()
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

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{ExternalReplyInfo, InaccessibleMessage, MaybeInaccessibleMessage, TextQuote};

    #[test]
    fn text_quote_extra_cannot_override_reserved_fields() -> Result<(), Box<dyn std::error::Error>>
    {
        let mut quote: TextQuote = serde_json::from_value(json!({
            "text": "quoted",
            "position": 4,
            "future": {"kept": true}
        }))?;
        quote.extra.insert("text".to_owned(), json!("spoofed"));
        quote.extra.insert("position".to_owned(), json!(0));
        quote.extra.insert("is_manual".to_owned(), json!(true));
        quote.extra.insert("entities".to_owned(), json!([]));
        quote
            .extra
            .insert("another_future".to_owned(), json!("kept"));

        let value = serde_json::to_value(quote)?;
        assert_eq!(value["text"], "quoted");
        assert_eq!(value["position"], 4);
        assert!(value.get("is_manual").is_none());
        assert!(value.get("entities").is_none());
        assert_eq!(value["future"], json!({"kept": true}));
        assert_eq!(value["another_future"], "kept");
        Ok(())
    }

    #[test]
    fn external_reply_info_extra_cannot_override_reserved_fields()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut reply: ExternalReplyInfo = serde_json::from_value(json!({
            "origin": {
                "type": "user",
                "date": 10,
                "sender_user": {"id": 1, "is_bot": false, "first_name": "Alice"}
            },
            "message_id": 55,
            "future": {"kept": true}
        }))?;
        reply.extra.insert(
            "origin".to_owned(),
            json!({
                "type": "hidden_user",
                "date": 1,
                "sender_user_name": "spoofed"
            }),
        );
        reply.extra.insert("message_id".to_owned(), json!(1));
        reply
            .extra
            .insert("has_media_spoiler".to_owned(), json!(true));
        reply.extra.insert("photo".to_owned(), json!([]));
        reply
            .extra
            .insert("another_future".to_owned(), json!("kept"));

        let value = serde_json::to_value(reply)?;
        assert_eq!(value["origin"]["type"], "user");
        assert_eq!(value["origin"]["date"], 10);
        assert_eq!(value["origin"]["sender_user"]["id"], 1);
        assert_eq!(value["message_id"], 55);
        assert!(value.get("has_media_spoiler").is_none());
        assert!(value.get("photo").is_none());
        assert_eq!(value["future"], json!({"kept": true}));
        assert_eq!(value["another_future"], "kept");
        Ok(())
    }

    #[test]
    fn inaccessible_message_preserves_future_fields() -> Result<(), Box<dyn std::error::Error>> {
        let input = json!({
            "message_id": 55,
            "date": 0,
            "chat": {"id": -10010, "type": "supergroup", "title": "mods"},
            "future": {"kept": true}
        });

        let message: MaybeInaccessibleMessage = serde_json::from_value(input.clone())?;

        assert!(!message.is_accessible());
        assert!(message.is_inaccessible());
        assert!(message.as_accessible().is_none());
        assert_eq!(
            message
                .as_inaccessible()
                .and_then(|message| message.extra.get("future")),
            Some(&json!({"kept": true}))
        );
        assert_eq!(serde_json::to_value(message.clone())?, input);
        assert_eq!(
            message
                .into_inaccessible()
                .and_then(|message| message.extra.get("future").cloned()),
            Some(json!({"kept": true}))
        );
        Ok(())
    }

    #[test]
    fn inaccessible_message_extra_cannot_override_reserved_fields()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut message: InaccessibleMessage = serde_json::from_value(json!({
            "message_id": 55,
            "date": 0,
            "chat": {"id": -10010, "type": "supergroup", "title": "mods"},
            "future": {"kept": true}
        }))?;
        message.extra.insert(
            "chat".to_owned(),
            json!({"id": 1, "type": "private", "first_name": "spoofed"}),
        );
        message.extra.insert("message_id".to_owned(), json!(1));
        message.extra.insert("date".to_owned(), json!(123));
        message
            .extra
            .insert("another_future".to_owned(), json!("kept"));

        let value = serde_json::to_value(message)?;
        assert_eq!(value["chat"]["id"], -10010);
        assert_eq!(value["chat"]["type"], "supergroup");
        assert_eq!(value["message_id"], 55);
        assert_eq!(value["date"], 0);
        assert_eq!(value["future"], json!({"kept": true}));
        assert_eq!(value["another_future"], "kept");
        Ok(())
    }

    #[test]
    fn accessible_message_can_be_consumed() -> Result<(), Box<dyn std::error::Error>> {
        let message: MaybeInaccessibleMessage = serde_json::from_value(json!({
            "message_id": 56,
            "date": 1700000000,
            "chat": {"id": 1, "type": "private"},
            "text": "hello"
        }))?;

        assert!(message.is_accessible());
        assert!(!message.is_inaccessible());
        assert!(message.as_inaccessible().is_none());
        assert_eq!(
            message
                .into_accessible()
                .and_then(|message| message.text)
                .as_deref(),
            Some("hello")
        );
        Ok(())
    }
}

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::common::MessageId;
use crate::types::sticker::Sticker;
use crate::types::telegram::LinkPreviewOptions;

use super::common::{Chat, MessageEntity, MessageOrigin, PhotoSize};
use super::content::{Checklist, Contact, Dice, Game, Location, Poll, Venue};
use super::is_false;
use super::media::{Animation, Audio, Document, PaidMediaInfo, Story, Video, VideoNote, Voice};
use super::model::Message;
use super::payments::Invoice;
use super::service::{Giveaway, GiveawayWinners};

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

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct InaccessibleMessage {
    pub chat: Chat,
    pub message_id: MessageId,
    pub date: i64,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
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

    use super::MaybeInaccessibleMessage;

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

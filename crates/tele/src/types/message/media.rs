use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::tagged::{serialize_tagged, strip_type, tagged_kind};

use super::common::{Chat, PhotoSize};

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

/// Telegram paid media preview payload.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct PaidMediaPreview {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<u32>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram paid media photo payload.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct PaidMediaPhoto {
    pub photo: Vec<PhotoSize>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram paid media video payload.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct PaidMediaVideo {
    pub video: Box<Video>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram paid media payload.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum PaidMedia {
    Preview(PaidMediaPreview),
    Photo(PaidMediaPhoto),
    Video(Box<PaidMediaVideo>),
    Unknown(Value),
}

impl PaidMedia {
    pub fn kind(&self) -> Option<&str> {
        match self {
            Self::Preview(_) => Some("preview"),
            Self::Photo(_) => Some("photo"),
            Self::Video(_) => Some("video"),
            Self::Unknown(value) => tagged_kind(value),
        }
    }

    pub fn is_preview(&self) -> bool {
        matches!(self, Self::Preview(_))
    }

    pub fn is_photo(&self) -> bool {
        matches!(self, Self::Photo(_))
    }

    pub fn is_video(&self) -> bool {
        matches!(self, Self::Video(_))
    }

    pub fn as_preview(&self) -> Option<&PaidMediaPreview> {
        match self {
            Self::Preview(value) => Some(value),
            Self::Photo(_) | Self::Video(_) | Self::Unknown(_) => None,
        }
    }

    pub fn into_preview(self) -> Option<PaidMediaPreview> {
        match self {
            Self::Preview(value) => Some(value),
            Self::Photo(_) | Self::Video(_) | Self::Unknown(_) => None,
        }
    }

    pub fn as_photo(&self) -> Option<&PaidMediaPhoto> {
        match self {
            Self::Photo(value) => Some(value),
            Self::Preview(_) | Self::Video(_) | Self::Unknown(_) => None,
        }
    }

    pub fn into_photo(self) -> Option<PaidMediaPhoto> {
        match self {
            Self::Photo(value) => Some(value),
            Self::Preview(_) | Self::Video(_) | Self::Unknown(_) => None,
        }
    }

    pub fn as_video(&self) -> Option<&PaidMediaVideo> {
        match self {
            Self::Video(value) => Some(value),
            Self::Preview(_) | Self::Photo(_) | Self::Unknown(_) => None,
        }
    }

    pub fn into_video(self) -> Option<PaidMediaVideo> {
        match self {
            Self::Video(value) => Some(*value),
            Self::Preview(_) | Self::Photo(_) | Self::Unknown(_) => None,
        }
    }

    pub fn as_unknown_value(&self) -> Option<&Value> {
        match self {
            Self::Unknown(value) => Some(value),
            Self::Preview(_) | Self::Photo(_) | Self::Video(_) => None,
        }
    }

    pub fn into_unknown_value(self) -> Option<Value> {
        match self {
            Self::Unknown(value) => Some(value),
            Self::Preview(_) | Self::Photo(_) | Self::Video(_) => None,
        }
    }
}

impl<'de> Deserialize<'de> for PaidMedia {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        match tagged_kind(&value) {
            Some("preview") => serde_json::from_value(strip_type(value))
                .map(Self::Preview)
                .map_err(serde::de::Error::custom),
            Some("photo") => serde_json::from_value(strip_type(value))
                .map(Self::Photo)
                .map_err(serde::de::Error::custom),
            Some("video") => serde_json::from_value(strip_type(value))
                .map(Box::new)
                .map(Self::Video)
                .map_err(serde::de::Error::custom),
            Some(_) | None => Ok(Self::Unknown(value)),
        }
    }
}

impl Serialize for PaidMedia {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Preview(value) => serialize_tagged(serializer, "preview", value),
            Self::Photo(value) => serialize_tagged(serializer, "photo", value),
            Self::Video(value) => serialize_tagged(serializer, "video", value),
            Self::Unknown(value) => value.serialize(serializer),
        }
    }
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

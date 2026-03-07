use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

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

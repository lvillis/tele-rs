use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::common::UserId;
use crate::types::message::PhotoSize;

/// Telegram user object.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct User {
    pub id: UserId,
    pub is_bot: bool,
    pub first_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language_code: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram user profile photos object.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct UserProfilePhotos {
    pub total_count: u64,
    pub photos: Vec<Vec<PhotoSize>>,
}

/// `getUserProfilePhotos` request.
#[derive(Clone, Debug, Serialize)]
pub struct GetUserProfilePhotosRequest {
    pub user_id: UserId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offset: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u8>,
}

impl GetUserProfilePhotosRequest {
    pub fn new(user_id: UserId) -> Self {
        Self {
            user_id,
            offset: None,
            limit: None,
        }
    }
}

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::common::UserId;
use crate::types::message::{Audio, PhotoSize};
use crate::{Error, Result};

const MAX_USER_PROFILE_PHOTOS_LIMIT: u8 = 100;

/// Telegram user object.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct User {
    pub id: UserId,
    pub is_bot: bool,
    pub first_name: String,
    #[serde(default)]
    pub last_name: Option<String>,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub language_code: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram user profile photos object.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct UserProfilePhotos {
    pub total_count: u64,
    pub photos: Vec<Vec<PhotoSize>>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram user profile audios object.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct UserProfileAudios {
    pub total_count: u64,
    pub audios: Vec<Audio>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
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

    pub fn validate(&self) -> Result<()> {
        self.user_id.validate()?;
        if self
            .limit
            .is_some_and(|limit| limit == 0 || limit > MAX_USER_PROFILE_PHOTOS_LIMIT)
        {
            return Err(Error::InvalidRequest {
                reason: format!(
                    "getUserProfilePhotos limit must be 1-{MAX_USER_PROFILE_PHOTOS_LIMIT}"
                ),
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_profile_photo_limit() {
        let mut request = GetUserProfilePhotosRequest::new(UserId(1));
        request.limit = Some(100);
        assert!(request.validate().is_ok());

        request.limit = Some(0);
        assert!(matches!(
            request.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let request = GetUserProfilePhotosRequest::new(UserId(0));
        assert!(matches!(
            request.validate(),
            Err(Error::InvalidRequest { .. })
        ));
    }

    #[test]
    fn user_preserves_future_fields() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let user: User = serde_json::from_value(serde_json::json!({
            "id": 42,
            "is_bot": true,
            "first_name": "Tele",
            "last_name": "Bot",
            "username": "telebot",
            "language_code": "en",
            "future_field": "kept"
        }))?;

        assert_eq!(user.id, UserId(42));
        assert!(user.is_bot);
        assert_eq!(user.first_name, "Tele");
        assert_eq!(user.last_name.as_deref(), Some("Bot"));
        assert_eq!(user.username.as_deref(), Some("telebot"));
        assert_eq!(user.language_code.as_deref(), Some("en"));
        assert_eq!(user.extra["future_field"], "kept");

        Ok(())
    }

    #[test]
    fn user_profile_photos_preserve_future_fields()
    -> std::result::Result<(), Box<dyn std::error::Error>> {
        let photos: UserProfilePhotos = serde_json::from_value(serde_json::json!({
            "total_count": 1,
            "photos": [[{
                "file_id": "photo-id",
                "file_unique_id": "unique-id",
                "width": 320,
                "height": 240,
                "future_photo_field": "kept"
            }]],
            "future_album_field": true
        }))?;

        assert_eq!(photos.total_count, 1);
        assert_eq!(photos.photos[0][0].file_id, "photo-id");
        assert_eq!(photos.photos[0][0].extra["future_photo_field"], "kept");
        assert_eq!(photos.extra["future_album_field"], true);

        Ok(())
    }
}

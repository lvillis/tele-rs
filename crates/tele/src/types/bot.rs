use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::common::UserId;
use crate::types::message::{Audio, PhotoSize};
use crate::{Error, Result};

const MAX_USER_PROFILE_PHOTOS_LIMIT: u8 = 100;

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

/// Telegram user profile audios object.
#[derive(Clone, Debug, Deserialize, Serialize)]
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
}

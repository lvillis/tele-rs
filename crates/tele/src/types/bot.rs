use std::collections::BTreeMap;

use serde::ser::SerializeMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::common::UserId;
use crate::types::extra::{
    field_len as extra_field_len, serialize_fields as serialize_extra_fields,
};
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language_code: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl Serialize for User {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = [
            "id",
            "is_bot",
            "first_name",
            "last_name",
            "username",
            "language_code",
        ];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let optional_len = usize::from(self.last_name.is_some())
            + usize::from(self.username.is_some())
            + usize::from(self.language_code.is_some());
        let mut object = serializer.serialize_map(Some(extra_len + optional_len + 3))?;
        object.serialize_entry("id", &self.id)?;
        object.serialize_entry("is_bot", &self.is_bot)?;
        object.serialize_entry("first_name", &self.first_name)?;
        if let Some(last_name) = self.last_name.as_ref() {
            object.serialize_entry("last_name", last_name)?;
        }
        if let Some(username) = self.username.as_ref() {
            object.serialize_entry("username", username)?;
        }
        if let Some(language_code) = self.language_code.as_ref() {
            object.serialize_entry("language_code", language_code)?;
        }
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
}

/// Telegram user profile photos object.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct UserProfilePhotos {
    pub total_count: u64,
    pub photos: Vec<Vec<PhotoSize>>,
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

impl Serialize for UserProfileAudios {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = ["total_count", "audios"];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let mut object = serializer.serialize_map(Some(extra_len + 2))?;
        object.serialize_entry("total_count", &self.total_count)?;
        object.serialize_entry("audios", &self.audios)?;
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
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
    fn user_extra_cannot_override_reserved_fields()
    -> std::result::Result<(), Box<dyn std::error::Error>> {
        let mut user = User {
            id: UserId(42),
            is_bot: true,
            first_name: "Tele".to_owned(),
            last_name: Some("Bot".to_owned()),
            username: Some("telebot".to_owned()),
            language_code: Some("en".to_owned()),
            extra: BTreeMap::new(),
        };
        user.extra.insert("id".to_owned(), serde_json::json!(1));
        user.extra
            .insert("is_bot".to_owned(), serde_json::json!(false));
        user.extra
            .insert("first_name".to_owned(), serde_json::json!("Overridden"));
        user.extra
            .insert("language_code".to_owned(), serde_json::json!("zh"));
        user.extra
            .insert("future_field".to_owned(), serde_json::json!("kept"));

        let value = serde_json::to_value(&user)?;
        assert_eq!(value["id"], 42);
        assert_eq!(value["is_bot"], true);
        assert_eq!(value["first_name"], "Tele");
        assert_eq!(value["last_name"], "Bot");
        assert_eq!(value["username"], "telebot");
        assert_eq!(value["language_code"], "en");
        assert_eq!(value["future_field"], "kept");

        Ok(())
    }

    #[test]
    fn user_profile_audios_extra_cannot_override_reserved_fields()
    -> std::result::Result<(), Box<dyn std::error::Error>> {
        let mut profile_audios = UserProfileAudios {
            total_count: 7,
            audios: Vec::new(),
            extra: BTreeMap::new(),
        };
        profile_audios
            .extra
            .insert("total_count".to_owned(), serde_json::json!(1));
        profile_audios
            .extra
            .insert("audios".to_owned(), serde_json::json!(["overridden"]));
        profile_audios
            .extra
            .insert("future_field".to_owned(), serde_json::json!("kept"));

        let value = serde_json::to_value(&profile_audios)?;
        assert_eq!(value["total_count"], 7);
        assert_eq!(value["audios"], serde_json::json!([]));
        assert_eq!(value["future_field"], "kept");

        Ok(())
    }
}

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::Error;
use crate::types::common::{ChatId, UserId};
use crate::types::message::PhotoSize;

fn ensure_non_empty(method: &str, field: &str, value: &str) -> Result<(), Error> {
    if value.trim().is_empty() {
        return Err(Error::InvalidRequest {
            reason: format!("{method} requires non-empty `{field}`"),
        });
    }

    Ok(())
}

/// Sticker media format used by upload and set methods.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StickerFormat {
    Static,
    Animated,
    Video,
}

/// Sticker set type.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StickerType {
    Regular,
    Mask,
    CustomEmoji,
}

/// Telegram sticker object.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Sticker {
    pub file_id: String,
    pub file_unique_id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub width: u32,
    pub height: u32,
    pub is_animated: bool,
    pub is_video: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<PhotoSize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub emoji: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub set_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_emoji_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub needs_repainting: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_size: Option<u64>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram sticker set object.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct StickerSet {
    pub name: String,
    pub title: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub stickers: Vec<Sticker>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<PhotoSize>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram input sticker descriptor.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InputSticker {
    pub sticker: String,
    pub format: StickerFormat,
    pub emoji_list: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mask_position: Option<MaskPosition>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keywords: Option<Vec<String>>,
}

impl InputSticker {
    pub fn new(
        sticker: impl Into<String>,
        format: StickerFormat,
        emoji_list: Vec<String>,
    ) -> Result<Self, Error> {
        let sticker = sticker.into();
        ensure_non_empty("inputSticker", "sticker", &sticker)?;

        if emoji_list.is_empty() {
            return Err(Error::InvalidRequest {
                reason: "input sticker requires at least one emoji".to_owned(),
            });
        }
        if emoji_list.iter().any(|emoji| emoji.trim().is_empty()) {
            return Err(Error::InvalidRequest {
                reason: "input sticker requires non-empty emoji entries".to_owned(),
            });
        }

        Ok(Self {
            sticker,
            format,
            emoji_list,
            mask_position: None,
            keywords: None,
        })
    }
}

/// Sticker mask position descriptor.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MaskPosition {
    pub point: String,
    pub x_shift: f64,
    pub y_shift: f64,
    pub scale: f64,
}

impl MaskPosition {
    pub fn new(
        point: impl Into<String>,
        x_shift: f64,
        y_shift: f64,
        scale: f64,
    ) -> Result<Self, Error> {
        let point = point.into();
        ensure_non_empty("maskPosition", "point", &point)?;

        Ok(Self {
            point,
            x_shift,
            y_shift,
            scale,
        })
    }
}

/// `sendSticker` request.
#[derive(Clone, Debug, Serialize)]
pub struct SendStickerRequest {
    pub chat_id: ChatId,
    pub sticker: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub business_connection_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_thread_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub direct_messages_topic_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub emoji: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_notification: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protect_content: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_paid_broadcast: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_effect_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suggested_post_parameters: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_parameters: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_markup: Option<Value>,
}

impl SendStickerRequest {
    pub fn new(chat_id: impl Into<ChatId>, sticker: impl Into<String>) -> Self {
        Self {
            chat_id: chat_id.into(),
            sticker: sticker.into(),
            business_connection_id: None,
            message_thread_id: None,
            direct_messages_topic_id: None,
            emoji: None,
            disable_notification: None,
            protect_content: None,
            allow_paid_broadcast: None,
            message_effect_id: None,
            suggested_post_parameters: None,
            reply_parameters: None,
            reply_markup: None,
        }
    }

    pub fn validate(&self) -> Result<(), Error> {
        ensure_non_empty("sendSticker", "sticker", &self.sticker)
    }
}

/// `getStickerSet` request.
#[derive(Clone, Debug, Serialize)]
pub struct GetStickerSetRequest {
    pub name: String,
}

impl GetStickerSetRequest {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    pub fn validate(&self) -> Result<(), Error> {
        ensure_non_empty("getStickerSet", "name", &self.name)
    }
}

/// `getCustomEmojiStickers` request.
#[derive(Clone, Debug, Serialize)]
pub struct GetCustomEmojiStickersRequest {
    pub custom_emoji_ids: Vec<String>,
}

impl GetCustomEmojiStickersRequest {
    pub fn new(custom_emoji_ids: Vec<String>) -> Result<Self, Error> {
        if custom_emoji_ids.is_empty() {
            return Err(Error::InvalidRequest {
                reason: "getCustomEmojiStickers requires at least one custom emoji id".to_owned(),
            });
        }
        if custom_emoji_ids
            .iter()
            .any(|emoji_id| emoji_id.trim().is_empty())
        {
            return Err(Error::InvalidRequest {
                reason: "getCustomEmojiStickers requires non-empty custom emoji ids".to_owned(),
            });
        }

        Ok(Self { custom_emoji_ids })
    }

    pub fn validate(&self) -> Result<(), Error> {
        if self.custom_emoji_ids.is_empty() {
            return Err(Error::InvalidRequest {
                reason: "getCustomEmojiStickers requires at least one custom emoji id".to_owned(),
            });
        }
        if self
            .custom_emoji_ids
            .iter()
            .any(|emoji_id| emoji_id.trim().is_empty())
        {
            return Err(Error::InvalidRequest {
                reason: "getCustomEmojiStickers requires non-empty custom emoji ids".to_owned(),
            });
        }
        Ok(())
    }
}

/// `uploadStickerFile` request.
#[derive(Clone, Debug, Serialize)]
pub struct UploadStickerFileRequest {
    pub user_id: UserId,
    pub sticker: String,
    pub sticker_format: StickerFormat,
}

impl UploadStickerFileRequest {
    pub fn new(user_id: UserId, sticker: impl Into<String>, sticker_format: StickerFormat) -> Self {
        Self {
            user_id,
            sticker: sticker.into(),
            sticker_format,
        }
    }

    pub fn validate(&self) -> Result<(), Error> {
        ensure_non_empty("uploadStickerFile", "sticker", &self.sticker)
    }
}

/// `createNewStickerSet` request.
#[derive(Clone, Debug, Serialize)]
pub struct CreateNewStickerSetRequest {
    pub user_id: UserId,
    pub name: String,
    pub title: String,
    pub stickers: Vec<InputSticker>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sticker_type: Option<StickerType>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub needs_repainting: Option<bool>,
}

impl CreateNewStickerSetRequest {
    pub fn new(
        user_id: UserId,
        name: impl Into<String>,
        title: impl Into<String>,
        stickers: Vec<InputSticker>,
    ) -> Result<Self, Error> {
        let name = name.into();
        let title = title.into();
        ensure_non_empty("createNewStickerSet", "name", &name)?;
        ensure_non_empty("createNewStickerSet", "title", &title)?;

        if stickers.is_empty() {
            return Err(Error::InvalidRequest {
                reason: "createNewStickerSet requires at least one input sticker".to_owned(),
            });
        }
        if stickers.len() > 50 {
            return Err(Error::InvalidRequest {
                reason: "createNewStickerSet supports at most 50 input stickers".to_owned(),
            });
        }

        let request = Self {
            user_id,
            name,
            title,
            stickers,
            sticker_type: None,
            needs_repainting: None,
        };
        request.validate()?;
        Ok(request)
    }

    pub fn validate(&self) -> Result<(), Error> {
        ensure_non_empty("createNewStickerSet", "name", &self.name)?;
        ensure_non_empty("createNewStickerSet", "title", &self.title)?;
        if self.stickers.is_empty() {
            return Err(Error::InvalidRequest {
                reason: "createNewStickerSet requires at least one input sticker".to_owned(),
            });
        }
        if self.stickers.len() > 50 {
            return Err(Error::InvalidRequest {
                reason: "createNewStickerSet supports at most 50 input stickers".to_owned(),
            });
        }
        Ok(())
    }
}

/// `addStickerToSet` request.
#[derive(Clone, Debug, Serialize)]
pub struct AddStickerToSetRequest {
    pub user_id: UserId,
    pub name: String,
    pub sticker: InputSticker,
}

impl AddStickerToSetRequest {
    pub fn new(user_id: UserId, name: impl Into<String>, sticker: InputSticker) -> Self {
        Self {
            user_id,
            name: name.into(),
            sticker,
        }
    }

    pub fn validate(&self) -> Result<(), Error> {
        ensure_non_empty("addStickerToSet", "name", &self.name)
    }
}

/// `setStickerPositionInSet` request.
#[derive(Clone, Debug, Serialize)]
pub struct SetStickerPositionInSetRequest {
    pub sticker: String,
    pub position: u16,
}

impl SetStickerPositionInSetRequest {
    pub fn new(sticker: impl Into<String>, position: u16) -> Self {
        Self {
            sticker: sticker.into(),
            position,
        }
    }

    pub fn validate(&self) -> Result<(), Error> {
        ensure_non_empty("setStickerPositionInSet", "sticker", &self.sticker)
    }
}

/// `deleteStickerFromSet` request.
#[derive(Clone, Debug, Serialize)]
pub struct DeleteStickerFromSetRequest {
    pub sticker: String,
}

impl DeleteStickerFromSetRequest {
    pub fn new(sticker: impl Into<String>) -> Self {
        Self {
            sticker: sticker.into(),
        }
    }

    pub fn validate(&self) -> Result<(), Error> {
        ensure_non_empty("deleteStickerFromSet", "sticker", &self.sticker)
    }
}

/// `replaceStickerInSet` request.
#[derive(Clone, Debug, Serialize)]
pub struct ReplaceStickerInSetRequest {
    pub user_id: UserId,
    pub name: String,
    pub old_sticker: String,
    pub sticker: InputSticker,
}

impl ReplaceStickerInSetRequest {
    pub fn new(
        user_id: UserId,
        name: impl Into<String>,
        old_sticker: impl Into<String>,
        sticker: InputSticker,
    ) -> Self {
        Self {
            user_id,
            name: name.into(),
            old_sticker: old_sticker.into(),
            sticker,
        }
    }

    pub fn validate(&self) -> Result<(), Error> {
        ensure_non_empty("replaceStickerInSet", "name", &self.name)?;
        ensure_non_empty("replaceStickerInSet", "old_sticker", &self.old_sticker)?;
        Ok(())
    }
}

/// `setStickerEmojiList` request.
#[derive(Clone, Debug, Serialize)]
pub struct SetStickerEmojiListRequest {
    pub sticker: String,
    pub emoji_list: Vec<String>,
}

impl SetStickerEmojiListRequest {
    pub fn new(sticker: impl Into<String>, emoji_list: Vec<String>) -> Result<Self, Error> {
        let sticker = sticker.into();
        ensure_non_empty("setStickerEmojiList", "sticker", &sticker)?;

        if emoji_list.is_empty() {
            return Err(Error::InvalidRequest {
                reason: "setStickerEmojiList requires at least one emoji".to_owned(),
            });
        }
        if emoji_list.iter().any(|emoji| emoji.trim().is_empty()) {
            return Err(Error::InvalidRequest {
                reason: "setStickerEmojiList requires non-empty emoji entries".to_owned(),
            });
        }

        Ok(Self {
            sticker,
            emoji_list,
        })
    }

    pub fn validate(&self) -> Result<(), Error> {
        ensure_non_empty("setStickerEmojiList", "sticker", &self.sticker)?;
        if self.emoji_list.is_empty() {
            return Err(Error::InvalidRequest {
                reason: "setStickerEmojiList requires at least one emoji".to_owned(),
            });
        }
        if self.emoji_list.iter().any(|emoji| emoji.trim().is_empty()) {
            return Err(Error::InvalidRequest {
                reason: "setStickerEmojiList requires non-empty emoji entries".to_owned(),
            });
        }
        Ok(())
    }
}

/// `setStickerKeywords` request.
#[derive(Clone, Debug, Serialize)]
pub struct SetStickerKeywordsRequest {
    pub sticker: String,
    pub keywords: Vec<String>,
}

impl SetStickerKeywordsRequest {
    pub fn new(sticker: impl Into<String>, keywords: Vec<String>) -> Self {
        Self {
            sticker: sticker.into(),
            keywords,
        }
    }

    pub fn validate(&self) -> Result<(), Error> {
        ensure_non_empty("setStickerKeywords", "sticker", &self.sticker)?;
        if self.keywords.len() > 20 {
            return Err(Error::InvalidRequest {
                reason: "setStickerKeywords supports at most 20 keywords".to_owned(),
            });
        }
        if self
            .keywords
            .iter()
            .any(|keyword| keyword.trim().is_empty())
        {
            return Err(Error::InvalidRequest {
                reason: "setStickerKeywords requires non-empty keyword entries".to_owned(),
            });
        }
        Ok(())
    }
}

/// `setStickerMaskPosition` request.
#[derive(Clone, Debug, Serialize)]
pub struct SetStickerMaskPositionRequest {
    pub sticker: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mask_position: Option<MaskPosition>,
}

impl SetStickerMaskPositionRequest {
    pub fn new(sticker: impl Into<String>) -> Self {
        Self {
            sticker: sticker.into(),
            mask_position: None,
        }
    }

    pub fn validate(&self) -> Result<(), Error> {
        ensure_non_empty("setStickerMaskPosition", "sticker", &self.sticker)
    }
}

/// `setStickerSetTitle` request.
#[derive(Clone, Debug, Serialize)]
pub struct SetStickerSetTitleRequest {
    pub name: String,
    pub title: String,
}

impl SetStickerSetTitleRequest {
    pub fn new(name: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            title: title.into(),
        }
    }

    pub fn validate(&self) -> Result<(), Error> {
        ensure_non_empty("setStickerSetTitle", "name", &self.name)?;
        ensure_non_empty("setStickerSetTitle", "title", &self.title)?;
        Ok(())
    }
}

/// `setStickerSetThumbnail` request.
#[derive(Clone, Debug, Serialize)]
pub struct SetStickerSetThumbnailRequest {
    pub name: String,
    pub user_id: UserId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<String>,
    pub format: StickerFormat,
}

impl SetStickerSetThumbnailRequest {
    pub fn new(name: impl Into<String>, user_id: UserId, format: StickerFormat) -> Self {
        Self {
            name: name.into(),
            user_id,
            thumbnail: None,
            format,
        }
    }

    pub fn validate(&self) -> Result<(), Error> {
        ensure_non_empty("setStickerSetThumbnail", "name", &self.name)
    }
}

/// `setCustomEmojiStickerSetThumbnail` request.
#[derive(Clone, Debug, Serialize)]
pub struct SetCustomEmojiStickerSetThumbnailRequest {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_emoji_id: Option<String>,
}

impl SetCustomEmojiStickerSetThumbnailRequest {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            custom_emoji_id: None,
        }
    }

    pub fn validate(&self) -> Result<(), Error> {
        ensure_non_empty("setCustomEmojiStickerSetThumbnail", "name", &self.name)
    }
}

/// `deleteStickerSet` request.
#[derive(Clone, Debug, Serialize)]
pub struct DeleteStickerSetRequest {
    pub name: String,
}

impl DeleteStickerSetRequest {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    pub fn validate(&self) -> Result<(), Error> {
        ensure_non_empty("deleteStickerSet", "name", &self.name)
    }
}

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::Error;
use crate::types::common::{ChatId, UserId};
use crate::types::message::PhotoSize;
use crate::types::telegram::{ReplyMarkup, ReplyParameters, SuggestedPostParameters};
use crate::types::validation::{
    control_free_string as validate_control_free_string,
    optional_request_positive_i64 as validate_optional_positive_i64,
    optional_request_string_id as validate_optional_string_id,
    reply_markup as validate_reply_markup, reply_parameters as validate_reply_parameters,
    request_non_empty as ensure_non_empty, request_string_id as validate_required_string_id,
    suggested_post_parameters as validate_suggested_post_parameters,
};

pub(crate) const MAX_CUSTOM_EMOJI_IDS: usize = 200;
pub(crate) const MAX_STICKER_EMOJIS: usize = 20;
pub(crate) const MAX_STICKER_KEYWORDS: usize = 20;

fn validate_optional_non_empty(
    method: &str,
    field: &str,
    value: Option<&str>,
) -> Result<(), Error> {
    if let Some(value) = value {
        validate_non_empty_control_free(method, field, value)?;
    }

    Ok(())
}

fn validate_non_empty_control_free(method: &str, field: &str, value: &str) -> Result<(), Error> {
    ensure_non_empty(method, field, value)?;
    validate_control_free_string(field, value)
}

fn validate_json_file_reference(method: &str, field: &str, value: &str) -> Result<(), Error> {
    validate_non_empty_control_free(method, field, value)?;
    if value.starts_with("attach://") {
        return Err(Error::InvalidRequest {
            reason: format!(
                "{method} {field} cannot use attach:// outside multipart upload requests"
            ),
        });
    }

    Ok(())
}

fn validate_optional_json_file_reference(
    method: &str,
    field: &str,
    value: Option<&str>,
) -> Result<(), Error> {
    if let Some(value) = value {
        validate_json_file_reference(method, field, value)?;
    }

    Ok(())
}

fn validate_business_connection_id(method: &str, value: Option<&str>) -> Result<(), Error> {
    validate_optional_string_id(method, "business_connection_id", value)
}

fn validate_message_effect_id(method: &str, value: Option<&str>) -> Result<(), Error> {
    validate_optional_string_id(method, "message_effect_id", value)
}

fn validate_custom_emoji_ids(method: &str, custom_emoji_ids: &[String]) -> Result<(), Error> {
    if custom_emoji_ids.is_empty() {
        return Err(Error::InvalidRequest {
            reason: format!("{method} requires at least one custom emoji id"),
        });
    }
    if custom_emoji_ids.len() > MAX_CUSTOM_EMOJI_IDS {
        return Err(Error::InvalidRequest {
            reason: format!("{method} supports at most {MAX_CUSTOM_EMOJI_IDS} custom emoji ids"),
        });
    }
    for (index, custom_emoji_id) in custom_emoji_ids.iter().enumerate() {
        validate_required_string_id(
            method,
            &format!("custom_emoji_ids[{index}]"),
            custom_emoji_id,
        )?;
    }

    Ok(())
}

fn validate_input_stickers(
    method: &str,
    stickers: &[InputSticker],
    max_items: usize,
) -> Result<(), Error> {
    if stickers.is_empty() {
        return Err(Error::InvalidRequest {
            reason: format!("{method} requires at least one input sticker"),
        });
    }
    if stickers.len() > max_items {
        return Err(Error::InvalidRequest {
            reason: format!("{method} supports at most {max_items} input stickers"),
        });
    }

    for sticker in stickers {
        sticker.validate()?;
    }

    Ok(())
}

fn validate_sticker_emoji_list(method: &str, emoji_list: &[String]) -> Result<(), Error> {
    if emoji_list.is_empty() {
        return Err(Error::InvalidRequest {
            reason: format!("{method} requires at least one emoji"),
        });
    }
    if emoji_list.len() > MAX_STICKER_EMOJIS {
        return Err(Error::InvalidRequest {
            reason: format!("{method} supports at most {MAX_STICKER_EMOJIS} emoji entries"),
        });
    }
    validate_sticker_string_items(method, "emoji", emoji_list)
}

fn validate_sticker_keywords(method: &str, keywords: &[String]) -> Result<(), Error> {
    if keywords.len() > MAX_STICKER_KEYWORDS {
        return Err(Error::InvalidRequest {
            reason: format!("{method} supports at most {MAX_STICKER_KEYWORDS} keywords"),
        });
    }
    validate_sticker_string_items(method, "keyword", keywords)
}

fn validate_sticker_string_items(
    method: &str,
    item_name: &str,
    values: &[String],
) -> Result<(), Error> {
    for (index, value) in values.iter().enumerate() {
        validate_non_empty_control_free(method, &format!("{item_name}[{index}]"), value)?;
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

/// Sticker type returned by Telegram.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum StickerKind {
    Regular,
    Mask,
    CustomEmoji,
    Unknown(String),
}

impl StickerKind {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Regular => "regular",
            Self::Mask => "mask",
            Self::CustomEmoji => "custom_emoji",
            Self::Unknown(kind) => kind.as_str(),
        }
    }
}

impl From<StickerType> for StickerKind {
    fn from(value: StickerType) -> Self {
        match value {
            StickerType::Regular => Self::Regular,
            StickerType::Mask => Self::Mask,
            StickerType::CustomEmoji => Self::CustomEmoji,
        }
    }
}

impl<'de> Deserialize<'de> for StickerKind {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let kind = String::deserialize(deserializer)?;
        Ok(match kind.as_str() {
            "regular" => Self::Regular,
            "mask" => Self::Mask,
            "custom_emoji" => Self::CustomEmoji,
            _ => Self::Unknown(kind),
        })
    }
}

/// Face point used by a sticker mask.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaskPositionPoint {
    Forehead,
    Eyes,
    Mouth,
    Chin,
}

/// Telegram sticker object.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct Sticker {
    pub file_id: String,
    pub file_unique_id: String,
    #[serde(rename = "type")]
    pub kind: StickerKind,
    pub width: u32,
    pub height: u32,
    pub is_animated: bool,
    pub is_video: bool,
    #[serde(default)]
    pub thumbnail: Option<PhotoSize>,
    #[serde(default)]
    pub emoji: Option<String>,
    #[serde(default)]
    pub set_name: Option<String>,
    #[serde(default)]
    pub custom_emoji_id: Option<String>,
    #[serde(default)]
    pub needs_repainting: Option<bool>,
    #[serde(default)]
    pub file_size: Option<u64>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram sticker set object.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct StickerSet {
    pub name: String,
    pub title: String,
    #[serde(rename = "type")]
    pub kind: StickerKind,
    pub stickers: Vec<Sticker>,
    #[serde(default)]
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
        let request = Self {
            sticker: sticker.into(),
            format,
            emoji_list,
            mask_position: None,
            keywords: None,
        };
        request.validate()?;
        Ok(request)
    }

    pub fn validate(&self) -> Result<(), Error> {
        validate_json_file_reference("inputSticker", "sticker", &self.sticker)?;
        validate_sticker_emoji_list("input sticker", &self.emoji_list)?;
        if let Some(mask_position) = self.mask_position.as_ref() {
            mask_position.validate()?;
        }
        if let Some(keywords) = self.keywords.as_ref() {
            validate_sticker_keywords("input sticker", keywords)?;
        }

        Ok(())
    }
}

/// Sticker mask position descriptor.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MaskPosition {
    pub point: MaskPositionPoint,
    pub x_shift: f64,
    pub y_shift: f64,
    pub scale: f64,
}

impl MaskPosition {
    pub fn new(
        point: MaskPositionPoint,
        x_shift: f64,
        y_shift: f64,
        scale: f64,
    ) -> Result<Self, Error> {
        let request = Self {
            point,
            x_shift,
            y_shift,
            scale,
        };
        request.validate()?;
        Ok(request)
    }

    pub fn validate(&self) -> Result<(), Error> {
        if !(self.x_shift.is_finite() && self.y_shift.is_finite() && self.scale.is_finite()) {
            return Err(Error::InvalidRequest {
                reason: "maskPosition coordinates and scale must be finite".to_owned(),
            });
        }
        if self.scale <= 0.0 {
            return Err(Error::InvalidRequest {
                reason: "maskPosition scale must be greater than 0".to_owned(),
            });
        }

        Ok(())
    }
}

/// `sendSticker` request.
#[derive(Clone, Debug, Serialize)]
pub struct SendStickerRequest {
    pub chat_id: ChatId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sticker: Option<String>,
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
    pub suggested_post_parameters: Option<SuggestedPostParameters>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_parameters: Option<ReplyParameters>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_markup: Option<ReplyMarkup>,
}

impl SendStickerRequest {
    pub fn new(chat_id: impl Into<ChatId>, sticker: impl Into<String>) -> Self {
        Self {
            chat_id: chat_id.into(),
            sticker: Some(sticker.into()),
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

    pub fn for_upload(chat_id: impl Into<ChatId>) -> Self {
        Self {
            chat_id: chat_id.into(),
            sticker: None,
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
        validate_business_connection_id("sendSticker", self.business_connection_id.as_deref())?;
        self.chat_id.validate()?;
        validate_optional_positive_i64("sendSticker", "message_thread_id", self.message_thread_id)?;
        validate_optional_positive_i64(
            "sendSticker",
            "direct_messages_topic_id",
            self.direct_messages_topic_id,
        )?;
        validate_reply_parameters(self.reply_parameters.as_ref())?;
        validate_reply_markup(self.reply_markup.as_ref())?;
        validate_suggested_post_parameters(self.suggested_post_parameters.as_ref())?;
        validate_message_effect_id("sendSticker", self.message_effect_id.as_deref())?;
        let Some(sticker) = self.sticker.as_deref() else {
            return Err(Error::InvalidRequest {
                reason: "sendSticker requires non-empty `sticker`".to_owned(),
            });
        };
        validate_json_file_reference("sendSticker", "sticker", sticker)?;
        validate_optional_non_empty("sendSticker", "emoji", self.emoji.as_deref())
    }

    pub(crate) fn validate_upload(&self) -> Result<(), Error> {
        validate_business_connection_id("sendSticker", self.business_connection_id.as_deref())?;
        self.chat_id.validate()?;
        validate_optional_positive_i64("sendSticker", "message_thread_id", self.message_thread_id)?;
        validate_optional_positive_i64(
            "sendSticker",
            "direct_messages_topic_id",
            self.direct_messages_topic_id,
        )?;
        validate_message_effect_id("sendSticker", self.message_effect_id.as_deref())?;
        validate_optional_non_empty("sendSticker", "emoji", self.emoji.as_deref())?;
        if self.sticker.is_some() {
            return Err(Error::InvalidRequest {
                reason: "sticker must be omitted for multipart upload requests".to_owned(),
            });
        }
        validate_reply_parameters(self.reply_parameters.as_ref())?;
        validate_reply_markup(self.reply_markup.as_ref())?;
        validate_suggested_post_parameters(self.suggested_post_parameters.as_ref())
    }

    pub fn suggested_post_parameters(
        mut self,
        suggested_post_parameters: SuggestedPostParameters,
    ) -> Self {
        self.suggested_post_parameters = Some(suggested_post_parameters);
        self
    }

    pub fn business_connection_id(mut self, business_connection_id: impl Into<String>) -> Self {
        self.business_connection_id = Some(business_connection_id.into());
        self
    }

    pub fn message_thread_id(mut self, message_thread_id: i64) -> Self {
        self.message_thread_id = Some(message_thread_id);
        self
    }

    pub fn direct_messages_topic_id(mut self, direct_messages_topic_id: i64) -> Self {
        self.direct_messages_topic_id = Some(direct_messages_topic_id);
        self
    }

    pub fn allow_paid_broadcast(mut self, enabled: bool) -> Self {
        self.allow_paid_broadcast = enabled.then_some(true);
        self
    }

    pub fn message_effect_id(mut self, message_effect_id: impl Into<String>) -> Self {
        self.message_effect_id = Some(message_effect_id.into());
        self
    }

    pub fn disable_notification(mut self, enabled: bool) -> Self {
        self.disable_notification = enabled.then_some(true);
        self
    }

    pub fn protect_content(mut self, enabled: bool) -> Self {
        self.protect_content = enabled.then_some(true);
        self
    }

    pub fn reply_parameters(mut self, reply_parameters: ReplyParameters) -> Self {
        self.reply_parameters = Some(reply_parameters);
        self
    }

    pub fn reply_to_message(mut self, message_id: crate::types::common::MessageId) -> Self {
        self.reply_parameters = Some(ReplyParameters::new(message_id));
        self
    }

    pub fn reply_markup(mut self, reply_markup: impl Into<ReplyMarkup>) -> Self {
        self.reply_markup = Some(reply_markup.into());
        self
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
        validate_non_empty_control_free("getStickerSet", "name", &self.name)
    }
}

/// `getCustomEmojiStickers` request.
#[derive(Clone, Debug, Serialize)]
pub struct GetCustomEmojiStickersRequest {
    pub custom_emoji_ids: Vec<String>,
}

impl GetCustomEmojiStickersRequest {
    pub fn new(custom_emoji_ids: Vec<String>) -> Result<Self, Error> {
        validate_custom_emoji_ids("getCustomEmojiStickers", &custom_emoji_ids)?;
        Ok(Self { custom_emoji_ids })
    }

    pub fn validate(&self) -> Result<(), Error> {
        validate_custom_emoji_ids("getCustomEmojiStickers", &self.custom_emoji_ids)
    }
}

/// `uploadStickerFile` request.
#[derive(Clone, Debug, Serialize)]
pub struct UploadStickerFileRequest {
    pub user_id: UserId,
    pub sticker_format: StickerFormat,
}

impl UploadStickerFileRequest {
    pub fn new(user_id: UserId, sticker_format: StickerFormat) -> Self {
        Self {
            user_id,
            sticker_format,
        }
    }

    pub fn validate(&self) -> Result<(), Error> {
        self.user_id.validate()
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
        validate_non_empty_control_free("createNewStickerSet", "name", &name)?;
        validate_non_empty_control_free("createNewStickerSet", "title", &title)?;

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
        self.user_id.validate()?;
        validate_non_empty_control_free("createNewStickerSet", "name", &self.name)?;
        validate_non_empty_control_free("createNewStickerSet", "title", &self.title)?;
        validate_input_stickers("createNewStickerSet", &self.stickers, 50)
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
        self.user_id.validate()?;
        validate_non_empty_control_free("addStickerToSet", "name", &self.name)?;
        self.sticker.validate()
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
        validate_non_empty_control_free("setStickerPositionInSet", "sticker", &self.sticker)
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
        validate_non_empty_control_free("deleteStickerFromSet", "sticker", &self.sticker)
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
        self.user_id.validate()?;
        validate_non_empty_control_free("replaceStickerInSet", "name", &self.name)?;
        validate_non_empty_control_free("replaceStickerInSet", "old_sticker", &self.old_sticker)?;
        self.sticker.validate()
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
        validate_non_empty_control_free("setStickerEmojiList", "sticker", &sticker)?;
        validate_sticker_emoji_list("setStickerEmojiList", &emoji_list)?;

        Ok(Self {
            sticker,
            emoji_list,
        })
    }

    pub fn validate(&self) -> Result<(), Error> {
        validate_non_empty_control_free("setStickerEmojiList", "sticker", &self.sticker)?;
        validate_sticker_emoji_list("setStickerEmojiList", &self.emoji_list)
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
        validate_non_empty_control_free("setStickerKeywords", "sticker", &self.sticker)?;
        validate_sticker_keywords("setStickerKeywords", &self.keywords)
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
        validate_non_empty_control_free("setStickerMaskPosition", "sticker", &self.sticker)?;
        if let Some(mask_position) = self.mask_position.as_ref() {
            mask_position.validate()?;
        }
        Ok(())
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
        validate_non_empty_control_free("setStickerSetTitle", "name", &self.name)?;
        validate_non_empty_control_free("setStickerSetTitle", "title", &self.title)?;
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
        validate_non_empty_control_free("setStickerSetThumbnail", "name", &self.name)?;
        self.user_id.validate()?;
        validate_optional_json_file_reference(
            "setStickerSetThumbnail",
            "thumbnail",
            self.thumbnail.as_deref(),
        )
    }

    pub(crate) fn validate_upload(&self) -> Result<(), Error> {
        validate_non_empty_control_free("setStickerSetThumbnail", "name", &self.name)?;
        self.user_id.validate()?;
        if self.thumbnail.is_some() {
            return Err(Error::InvalidRequest {
                reason: "thumbnail must be omitted for multipart upload requests".to_owned(),
            });
        }
        Ok(())
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
        validate_non_empty_control_free("setCustomEmojiStickerSetThumbnail", "name", &self.name)?;
        validate_optional_non_empty(
            "setCustomEmojiStickerSetThumbnail",
            "custom_emoji_id",
            self.custom_emoji_id.as_deref(),
        )
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
        validate_non_empty_control_free("deleteStickerSet", "name", &self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::common::ChatId;

    #[test]
    fn validates_sticker_request_ids_and_nested_input() -> Result<(), Error> {
        let send = SendStickerRequest::new(ChatId::from(0), "sticker-file-id");
        assert!(matches!(send.validate(), Err(Error::InvalidRequest { .. })));

        let send = SendStickerRequest::new(ChatId::from(1), "sticker-file-id")
            .reply_markup(crate::types::telegram::InlineKeyboardMarkup::new(Vec::new()));
        assert!(matches!(send.validate(), Err(Error::InvalidRequest { .. })));

        let attach_send = SendStickerRequest::new(ChatId::from(1), "attach://sticker0");
        assert!(matches!(
            attach_send.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let mut threaded = SendStickerRequest::new(ChatId::from(1), "sticker-file-id");
        threaded.message_thread_id = Some(0);
        assert!(matches!(
            threaded.validate(),
            Err(Error::InvalidRequest { .. })
        ));
        threaded.message_thread_id = None;
        threaded.direct_messages_topic_id = Some(-1);
        assert!(matches!(
            threaded.validate(),
            Err(Error::InvalidRequest { .. })
        ));
        threaded.direct_messages_topic_id = None;
        threaded.message_effect_id = Some("bad\nid".to_owned());
        assert!(matches!(
            threaded.validate(),
            Err(Error::InvalidRequest { .. })
        ));
        threaded.message_effect_id = None;
        threaded.emoji = Some("bad\nemoji".to_owned());
        assert!(matches!(
            threaded.validate(),
            Err(Error::InvalidRequest { .. })
        ));
        assert!(matches!(
            threaded.validate_upload(),
            Err(Error::InvalidRequest { .. })
        ));

        let upload = UploadStickerFileRequest::new(UserId(0), StickerFormat::Static);
        assert!(matches!(
            upload.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let mut input = InputSticker::new(
            "sticker-file-id",
            StickerFormat::Static,
            vec!["😀".to_owned()],
        )?;
        assert!(matches!(
            InputSticker::new(
                "attach://sticker0",
                StickerFormat::Static,
                vec!["😀".to_owned()]
            ),
            Err(Error::InvalidRequest { .. })
        ));
        input.emoji_list = vec!["😀".to_owned(); MAX_STICKER_EMOJIS + 1];
        assert!(matches!(
            input.validate(),
            Err(Error::InvalidRequest { .. })
        ));
        input.emoji_list = vec!["😀".to_owned()];
        input.emoji_list = vec!["bad\nemoji".to_owned()];
        assert!(matches!(
            input.validate(),
            Err(Error::InvalidRequest { .. })
        ));
        input.emoji_list = vec!["😀".to_owned()];
        input.keywords = Some(vec!["tag".to_owned(); MAX_STICKER_KEYWORDS + 1]);
        assert!(matches!(
            input.validate(),
            Err(Error::InvalidRequest { .. })
        ));
        input.keywords = Some(vec!["bad\nkeyword".to_owned()]);
        assert!(matches!(
            input.validate(),
            Err(Error::InvalidRequest { .. })
        ));
        input.keywords = None;
        input.mask_position = Some(MaskPosition {
            point: MaskPositionPoint::Forehead,
            x_shift: 0.0,
            y_shift: 0.0,
            scale: 0.0,
        });
        let add = AddStickerToSetRequest::new(UserId(1), "set_name", input);
        assert!(matches!(add.validate(), Err(Error::InvalidRequest { .. })));

        let mut custom = SetCustomEmojiStickerSetThumbnailRequest::new("set_name");
        custom.custom_emoji_id = Some("  ".to_owned());
        assert!(matches!(
            custom.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let mut thumbnail_upload =
            SetStickerSetThumbnailRequest::new("set_name", UserId(1), StickerFormat::Static);
        thumbnail_upload.thumbnail = Some("bad\nthumbnail".to_owned());
        assert!(matches!(
            thumbnail_upload.validate(),
            Err(Error::InvalidRequest { .. })
        ));
        thumbnail_upload.thumbnail = Some("attach://thumb0".to_owned());
        assert!(matches!(
            thumbnail_upload.validate(),
            Err(Error::InvalidRequest { .. })
        ));
        thumbnail_upload.thumbnail = Some("existing-thumbnail-file-id".to_owned());
        assert!(matches!(
            thumbnail_upload.validate_upload(),
            Err(Error::InvalidRequest { .. })
        ));
        thumbnail_upload.thumbnail = None;
        thumbnail_upload.validate_upload()?;

        let custom_emoji_ids = vec!["emoji-id".to_owned(); MAX_CUSTOM_EMOJI_IDS + 1];
        assert!(matches!(
            GetCustomEmojiStickersRequest::new(custom_emoji_ids),
            Err(Error::InvalidRequest { .. })
        ));
        assert!(matches!(
            GetCustomEmojiStickersRequest::new(vec!["bad\nid".to_owned()]),
            Err(Error::InvalidRequest { .. })
        ));

        let invalid_custom_emoji_ids = GetCustomEmojiStickersRequest {
            custom_emoji_ids: vec!["bad\nid".to_owned()],
        };
        assert!(matches!(
            invalid_custom_emoji_ids.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let emoji_list = vec!["😀".to_owned(); MAX_STICKER_EMOJIS + 1];
        assert!(matches!(
            SetStickerEmojiListRequest::new("sticker-file-id", emoji_list),
            Err(Error::InvalidRequest { .. })
        ));
        assert!(matches!(
            SetStickerEmojiListRequest::new("sticker-file-id", vec!["bad\nemoji".to_owned()]),
            Err(Error::InvalidRequest { .. })
        ));
        assert!(matches!(
            SetStickerKeywordsRequest::new("sticker-file-id", vec!["bad\nkeyword".to_owned()])
                .validate(),
            Err(Error::InvalidRequest { .. })
        ));

        Ok(())
    }

    #[test]
    fn sticker_response_types_are_typed_and_forward_compatible()
    -> std::result::Result<(), Box<dyn std::error::Error>> {
        let known: Sticker = serde_json::from_value(serde_json::json!({
            "file_id": "file",
            "file_unique_id": "unique",
            "type": "custom_emoji",
            "width": 512,
            "height": 512,
            "is_animated": false,
            "is_video": false,
            "future_field": "kept"
        }))?;
        assert_eq!(known.kind, StickerKind::CustomEmoji);
        assert_eq!(known.file_id, "file");
        assert_eq!(known.kind.as_str(), "custom_emoji");
        assert_eq!(known.width, 512);
        assert!(known.thumbnail.is_none());
        assert_eq!(known.extra["future_field"], "kept");

        let unknown: StickerSet = serde_json::from_value(serde_json::json!({
            "name": "set",
            "title": "Set",
            "type": "future_kind",
            "stickers": [],
            "future_field": "kept"
        }))?;
        assert_eq!(unknown.kind, StickerKind::Unknown("future_kind".to_owned()));
        assert_eq!(unknown.kind.as_str(), "future_kind");
        assert!(unknown.stickers.is_empty());
        assert_eq!(unknown.extra["future_field"], "kept");

        Ok(())
    }
}

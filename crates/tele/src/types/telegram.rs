use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Display, Write as _};
use std::str::FromStr;

use serde::de::DeserializeOwned;
use serde::ser::SerializeMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::common::{ChatId, MessageId, NumericChatId, ParseMode};
use crate::types::extra::{
    field_len as extra_field_len, serialize_fields as serialize_extra_fields,
};
use crate::types::message::MessageEntity;
#[cfg(test)]
use crate::types::message::MessageEntityKind;
use crate::types::tagged::{strip_type, tagged_kind};
use crate::types::validation::{
    display_text_length_range as validate_display_text_length_range,
    https_url as validate_https_url,
    optional_rich_text_formatting as validate_optional_rich_text_formatting,
    optional_text_formatting as validate_optional_text_formatting,
    required_display_text as validate_required_display_text,
    rich_text_formatting as validate_rich_text_formatting,
    suggested_post_parameters_send_date as validate_suggested_post_parameters_send_date,
    url as validate_url,
};
use crate::{Error, Result};

pub const MAX_CALLBACK_DATA_BYTES: usize = 64;

fn invalid_request(reason: impl Into<String>) -> Error {
    Error::InvalidRequest {
        reason: reason.into(),
    }
}

fn validate_callback_data(data: impl Into<String>) -> Result<String> {
    let data = data.into();
    if data.trim().is_empty() {
        return Err(invalid_request("callback_data cannot be empty"));
    }
    if data.len() > MAX_CALLBACK_DATA_BYTES {
        return Err(invalid_request(format!(
            "callback_data exceeds Telegram's 64-byte limit ({})",
            data.len()
        )));
    }
    Ok(data)
}

fn validate_required_visible_text(field: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(invalid_request(format!("{field} cannot be empty")));
    }
    if value.chars().any(char::is_control) {
        return Err(invalid_request(format!(
            "{field} must not contain control characters"
        )));
    }

    Ok(())
}

fn validate_string_without_control_chars(field: &str, value: &str) -> Result<()> {
    if value.chars().any(char::is_control) {
        return Err(invalid_request(format!(
            "{field} must not contain control characters"
        )));
    }

    Ok(())
}

fn value_as_object<'a>(
    field: &str,
    value: &'a Value,
) -> Result<&'a serde_json::Map<String, Value>> {
    value
        .as_object()
        .ok_or_else(|| invalid_request(format!("{field} must be a JSON object")))
}

fn value_as_str<'a>(field: &str, value: &'a Value) -> Result<&'a str> {
    value
        .as_str()
        .ok_or_else(|| invalid_request(format!("{field} must be a string")))
}

fn value_as_bool(field: &str, value: &Value) -> Result<bool> {
    value
        .as_bool()
        .ok_or_else(|| invalid_request(format!("{field} must be a boolean")))
}

fn validate_true_value(field: &str, value: &Value) -> Result<()> {
    if !value_as_bool(field, value)? {
        return Err(invalid_request(format!("{field} must be true")));
    }

    Ok(())
}

fn validate_optional_bool_field(
    object: &serde_json::Map<String, Value>,
    object_name: &str,
    field: &str,
) -> Result<()> {
    if let Some(value) = object.get(field) {
        value_as_bool(&format!("{object_name}.{field}"), value)?;
    }

    Ok(())
}

fn validate_required_i64_field(
    object: &serde_json::Map<String, Value>,
    object_name: &str,
    field: &str,
) -> Result<()> {
    let value = object
        .get(field)
        .and_then(Value::as_i64)
        .ok_or_else(|| invalid_request(format!("{object_name}.{field} must be an integer")))?;
    if value <= 0 {
        return Err(invalid_request(format!(
            "{object_name}.{field} must be greater than 0"
        )));
    }

    Ok(())
}

fn validate_login_url_payload(value: &Value) -> Result<()> {
    let object = value_as_object("login_url", value)?;
    let url = object
        .get("url")
        .map(|value| value_as_str("login_url.url", value))
        .transpose()?
        .ok_or_else(|| invalid_request("login_url.url is required"))?;
    validate_https_url("login_url.url", url)
}

fn validate_switch_inline_query_chosen_chat(value: &Value) -> Result<()> {
    let object = value_as_object("switch_inline_query_chosen_chat", value)?;
    if let Some(query) = object.get("query") {
        validate_string_without_control_chars(
            "switch_inline_query_chosen_chat.query",
            value_as_str("switch_inline_query_chosen_chat.query", query)?,
        )?;
    }
    for field in [
        "allow_user_chats",
        "allow_bot_chats",
        "allow_group_chats",
        "allow_channel_chats",
    ] {
        validate_optional_bool_field(object, "switch_inline_query_chosen_chat", field)?;
    }

    Ok(())
}

fn validate_copy_text_button(value: &Value) -> Result<()> {
    let object = value_as_object("copy_text", value)?;
    let text = object
        .get("text")
        .map(|value| value_as_str("copy_text.text", value))
        .transpose()?
        .ok_or_else(|| invalid_request("copy_text.text is required"))?;
    validate_required_visible_text("copy_text.text", text)
}

fn validate_keyboard_button_request_users(value: &Value) -> Result<()> {
    let object = value_as_object("request_users", value)?;
    validate_required_i64_field(object, "request_users", "request_id")?;
    for field in [
        "user_is_bot",
        "user_is_premium",
        "request_name",
        "request_username",
        "request_photo",
    ] {
        validate_optional_bool_field(object, "request_users", field)?;
    }

    Ok(())
}

fn validate_keyboard_button_request_chat(value: &Value) -> Result<()> {
    let object = value_as_object("request_chat", value)?;
    validate_required_i64_field(object, "request_chat", "request_id")?;
    object
        .get("chat_is_channel")
        .map(|value| value_as_bool("request_chat.chat_is_channel", value))
        .transpose()?
        .ok_or_else(|| invalid_request("request_chat.chat_is_channel is required"))?;
    for field in [
        "chat_is_forum",
        "chat_has_username",
        "chat_is_created",
        "bot_is_member",
        "request_title",
        "request_username",
        "request_photo",
    ] {
        validate_optional_bool_field(object, "request_chat", field)?;
    }

    Ok(())
}

fn validate_keyboard_button_request_poll(value: &Value) -> Result<()> {
    let object = value_as_object("request_poll", value)?;
    if let Some(poll_type) = object.get("type") {
        let poll_type = value_as_str("request_poll.type", poll_type)?;
        validate_required_visible_text("request_poll.type", poll_type)?;
        if !matches!(poll_type, "quiz" | "regular") {
            return Err(invalid_request(
                "request_poll.type must be `quiz` or `regular`",
            ));
        }
    }

    Ok(())
}

fn validate_typed_object_payload(field: &str, value: &Value) -> Result<()> {
    let Some(object) = value.as_object() else {
        return Err(invalid_request(format!("{field} must be a JSON object")));
    };
    let Some(kind) = object.get("type").and_then(Value::as_str) else {
        return Err(invalid_request(format!(
            "{field} requires a string `type` field"
        )));
    };
    validate_required_visible_text(&format!("{field}.type"), kind)
}

fn validate_suggested_post_parameters_payload(field: &str, value: &Value) -> Result<()> {
    let object = value_as_object(field, value)?;
    if object.is_empty() {
        return Err(invalid_request(format!("{field} cannot be empty")));
    }

    if let Some(price) = object.get("price") {
        validate_suggested_post_price(&format!("{field}.price"), price)?;
    }
    if let Some(send_date) = object.get("send_date") {
        let send_date = send_date
            .as_i64()
            .ok_or_else(|| invalid_request(format!("{field}.send_date must be an integer")))?;
        validate_suggested_post_parameters_send_date(&format!("{field}.send_date"), send_date)?;
    }

    Ok(())
}

fn validate_suggested_post_price(field: &str, value: &Value) -> Result<()> {
    let object = value_as_object(field, value)?;
    let currency = object
        .get("currency")
        .map(|value| value_as_str(&format!("{field}.currency"), value))
        .transpose()?
        .ok_or_else(|| invalid_request(format!("{field}.currency is required")))?;
    let amount = object
        .get("amount")
        .and_then(Value::as_i64)
        .ok_or_else(|| invalid_request(format!("{field}.amount must be an integer")))?;

    match currency {
        "XTR" if (5..=100_000).contains(&amount) => Ok(()),
        "TON" if (10_000_000..=10_000_000_000_000).contains(&amount) => Ok(()),
        "XTR" => Err(invalid_request(format!(
            "{field}.amount must be 5-100000 for XTR"
        ))),
        "TON" => Err(invalid_request(format!(
            "{field}.amount must be 10000000-10000000000000 for TON"
        ))),
        _ => Err(invalid_request(format!(
            "{field}.currency must be `XTR` or `TON`"
        ))),
    }
}

fn validate_source_object_payload(field: &str, value: &Value) -> Result<()> {
    let Some(object) = value.as_object() else {
        return Err(invalid_request(format!("{field} must be a JSON object")));
    };
    let Some(source) = object.get("source").and_then(Value::as_str) else {
        return Err(invalid_request(format!(
            "{field} requires a string `source` field"
        )));
    };
    validate_required_visible_text(&format!("{field}.source"), source)
}

fn validate_accepted_gift_types_payload(field: &str, value: &Value) -> Result<()> {
    let Some(object) = value.as_object() else {
        return Err(invalid_request(format!("{field} must be a JSON object")));
    };
    if object.is_empty() {
        return Err(invalid_request(format!("{field} cannot be empty")));
    }
    for key in [
        "unlimited_gifts",
        "limited_gifts",
        "unique_gifts",
        "premium_subscription",
        "gifts_from_channels",
    ] {
        if object.get(key).is_some_and(|value| !value.is_boolean()) {
            return Err(invalid_request(format!("{field}.{key} must be a boolean")));
        }
    }

    Ok(())
}

fn validate_finite_float(field: &str, value: f64) -> Result<()> {
    if !value.is_finite() {
        return Err(invalid_request(format!("{field} must be finite")));
    }

    Ok(())
}

fn validate_float_range(field: &str, value: f64, min: f64, max: f64) -> Result<()> {
    validate_finite_float(field, value)?;
    if !(min..=max).contains(&value) {
        return Err(invalid_request(format!(
            "{field} must be between {min} and {max}"
        )));
    }

    Ok(())
}

fn validate_positive_percentage(field: &str, value: f64) -> Result<()> {
    validate_float_range(field, value, 0.0, 100.0)?;
    if value <= 0.0 {
        return Err(invalid_request(format!("{field} must be greater than 0")));
    }

    Ok(())
}

fn validate_optional_visible_text(field: &str, value: Option<&str>) -> Result<()> {
    if let Some(value) = value {
        validate_string_without_control_chars(field, value)?;
    }

    Ok(())
}

macro_rules! json_payload_wrapper {
    ($(#[$meta:meta])* $name:ident, $label:literal, $validator:ident) => {
        $(#[$meta])*
        #[derive(Clone, Debug, Serialize)]
        #[serde(transparent)]
        pub struct $name(Value);

        impl $name {
            pub fn new(value: Value) -> Result<Self> {
                $validator($label, &value)?;
                Ok(Self(value))
            }

            pub fn try_from_typed<T>(value: T) -> Result<Self>
            where
                T: Serialize,
            {
                let value = serde_json::to_value(value)
                    .map_err(|source| Error::SerializeRequest { source })?;
                Self::new(value)
            }

            pub fn validate(&self) -> Result<()> {
                $validator($label, &self.0)
            }

            pub fn as_value(&self) -> &Value {
                &self.0
            }

            pub fn into_value(self) -> Value {
                self.0
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let value = Value::deserialize(deserializer)?;
                Self::new(value).map_err(serde::de::Error::custom)
            }
        }

        impl TryFrom<Value> for $name {
            type Error = Error;

            fn try_from(value: Value) -> std::result::Result<Self, Self::Error> {
                Self::new(value)
            }
        }

        impl From<$name> for Value {
            fn from(value: $name) -> Self {
                value.0
            }
        }
    };
}

/// Photo content for a story to post.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[non_exhaustive]
pub struct InputStoryContentPhoto {
    pub photo: String,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl InputStoryContentPhoto {
    pub fn new(photo: impl Into<String>) -> Self {
        Self {
            photo: photo.into(),
            extra: BTreeMap::new(),
        }
    }

    pub fn validate(&self) -> Result<()> {
        validate_required_visible_text("input_story_content.photo", &self.photo)
    }
}

impl Serialize for InputStoryContentPhoto {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = ["photo"];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let mut object = serializer.serialize_map(Some(extra_len + 1))?;
        object.serialize_entry("photo", &self.photo)?;
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
}

/// Video content for a story to post.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[non_exhaustive]
pub struct InputStoryContentVideo {
    pub video: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cover_frame_timestamp: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_animation: Option<bool>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl InputStoryContentVideo {
    pub fn new(video: impl Into<String>) -> Self {
        Self {
            video: video.into(),
            duration: None,
            cover_frame_timestamp: None,
            is_animation: None,
            extra: BTreeMap::new(),
        }
    }

    pub fn with_duration(mut self, duration: f64) -> Self {
        self.duration = Some(duration);
        self
    }

    pub fn with_cover_frame_timestamp(mut self, cover_frame_timestamp: f64) -> Self {
        self.cover_frame_timestamp = Some(cover_frame_timestamp);
        self
    }

    pub fn with_animation(mut self, is_animation: bool) -> Self {
        self.is_animation = Some(is_animation);
        self
    }

    pub fn validate(&self) -> Result<()> {
        validate_required_visible_text("input_story_content.video", &self.video)?;
        if let Some(duration) = self.duration {
            validate_float_range("input_story_content.duration", duration, 0.0, 60.0)?;
        }
        if let Some(cover_frame_timestamp) = self.cover_frame_timestamp {
            validate_float_range(
                "input_story_content.cover_frame_timestamp",
                cover_frame_timestamp,
                0.0,
                f64::MAX,
            )?;
        }

        Ok(())
    }
}

impl Serialize for InputStoryContentVideo {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = ["video", "duration", "cover_frame_timestamp", "is_animation"];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let optional_len = usize::from(self.duration.is_some())
            + usize::from(self.cover_frame_timestamp.is_some())
            + usize::from(self.is_animation.is_some());
        let mut object = serializer.serialize_map(Some(extra_len + optional_len + 1))?;
        object.serialize_entry("video", &self.video)?;
        if let Some(duration) = self.duration {
            object.serialize_entry("duration", &duration)?;
        }
        if let Some(cover_frame_timestamp) = self.cover_frame_timestamp {
            object.serialize_entry("cover_frame_timestamp", &cover_frame_timestamp)?;
        }
        if let Some(is_animation) = self.is_animation {
            object.serialize_entry("is_animation", &is_animation)?;
        }
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
}

/// Content of a story to post.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum InputStoryContent {
    Photo(InputStoryContentPhoto),
    Video(InputStoryContentVideo),
    Unknown(Value),
}

impl InputStoryContent {
    pub fn photo(photo: impl Into<String>) -> Self {
        Self::Photo(InputStoryContentPhoto::new(photo))
    }

    pub fn video(video: impl Into<String>) -> Self {
        Self::Video(InputStoryContentVideo::new(video))
    }

    pub fn new(value: Value) -> Result<Self> {
        Self::try_from_value(value)
    }

    pub fn try_from_value(value: Value) -> Result<Self> {
        let content = serde_json::from_value::<Self>(value).map_err(|source| {
            invalid_request(format!("invalid input_story_content payload: {source}"))
        })?;
        content.validate()?;
        Ok(content)
    }

    pub fn try_from_typed<T>(value: T) -> Result<Self>
    where
        T: Serialize,
    {
        let value =
            serde_json::to_value(value).map_err(|source| Error::SerializeRequest { source })?;
        Self::try_from_value(value)
    }

    pub fn kind(&self) -> Option<&str> {
        match self {
            Self::Photo(_) => Some("photo"),
            Self::Video(_) => Some("video"),
            Self::Unknown(value) => tagged_kind(value),
        }
    }

    pub fn validate(&self) -> Result<()> {
        match self {
            Self::Photo(value) => value.validate(),
            Self::Video(value) => value.validate(),
            Self::Unknown(value) => validate_typed_object_payload("input_story_content", value),
        }
    }
}

impl From<InputStoryContentPhoto> for InputStoryContent {
    fn from(value: InputStoryContentPhoto) -> Self {
        Self::Photo(value)
    }
}

impl From<InputStoryContentVideo> for InputStoryContent {
    fn from(value: InputStoryContentVideo) -> Self {
        Self::Video(value)
    }
}

impl<'de> Deserialize<'de> for InputStoryContent {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        Self::try_from(value).map_err(serde::de::Error::custom)
    }
}

impl TryFrom<Value> for InputStoryContent {
    type Error = Error;

    fn try_from(value: Value) -> std::result::Result<Self, Self::Error> {
        match tagged_kind(&value) {
            Some("photo") => deserialize_input_story_content_known(value, Self::Photo),
            Some("video") => deserialize_input_story_content_known(value, Self::Video),
            Some(_) => {
                validate_typed_object_payload("input_story_content", &value)?;
                Ok(Self::Unknown(value))
            }
            None => Err(invalid_request(
                "input_story_content requires a string `type` field",
            )),
        }
    }
}

fn deserialize_input_story_content_known<T>(
    value: Value,
    constructor: impl FnOnce(T) -> InputStoryContent,
) -> Result<InputStoryContent>
where
    T: DeserializeOwned,
{
    let payload = serde_json::from_value::<T>(strip_type(value)).map_err(|source| {
        invalid_request(format!("invalid input_story_content payload: {source}"))
    })?;
    let content = constructor(payload);
    content.validate()?;
    Ok(content)
}

impl Serialize for InputStoryContent {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Photo(value) => serialize_typed_payload(serializer, "photo", value),
            Self::Video(value) => serialize_typed_payload(serializer, "video", value),
            Self::Unknown(value) => value.serialize(serializer),
        }
    }
}

/// Paid live photo media item.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[non_exhaustive]
pub struct InputPaidMediaLivePhoto {
    pub media: String,
    pub photo: String,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl InputPaidMediaLivePhoto {
    pub fn new(media: impl Into<String>, photo: impl Into<String>) -> Self {
        Self {
            media: media.into(),
            photo: photo.into(),
            extra: BTreeMap::new(),
        }
    }

    pub fn validate(&self) -> Result<()> {
        validate_required_visible_text("input_paid_media.media", &self.media)?;
        validate_required_visible_text("input_paid_media.photo", &self.photo)
    }
}

impl Serialize for InputPaidMediaLivePhoto {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = ["media", "photo"];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let mut object = serializer.serialize_map(Some(extra_len + 2))?;
        object.serialize_entry("media", &self.media)?;
        object.serialize_entry("photo", &self.photo)?;
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
}

/// Paid photo media item.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[non_exhaustive]
pub struct InputPaidMediaPhoto {
    pub media: String,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl InputPaidMediaPhoto {
    pub fn new(media: impl Into<String>) -> Self {
        Self {
            media: media.into(),
            extra: BTreeMap::new(),
        }
    }

    pub fn validate(&self) -> Result<()> {
        validate_required_visible_text("input_paid_media.media", &self.media)
    }
}

impl Serialize for InputPaidMediaPhoto {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = ["media"];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let mut object = serializer.serialize_map(Some(extra_len + 1))?;
        object.serialize_entry("media", &self.media)?;
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
}

/// Paid video media item.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[non_exhaustive]
pub struct InputPaidMediaVideo {
    pub media: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cover: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_timestamp: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supports_streaming: Option<bool>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl InputPaidMediaVideo {
    pub fn new(media: impl Into<String>) -> Self {
        Self {
            media: media.into(),
            thumbnail: None,
            cover: None,
            start_timestamp: None,
            width: None,
            height: None,
            duration: None,
            supports_streaming: None,
            extra: BTreeMap::new(),
        }
    }

    pub fn with_thumbnail(mut self, thumbnail: impl Into<String>) -> Self {
        self.thumbnail = Some(thumbnail.into());
        self
    }

    pub fn with_cover(mut self, cover: impl Into<String>) -> Self {
        self.cover = Some(cover.into());
        self
    }

    pub fn with_start_timestamp(mut self, start_timestamp: u32) -> Self {
        self.start_timestamp = Some(start_timestamp);
        self
    }

    pub fn with_dimensions(mut self, width: u32, height: u32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    pub fn with_duration(mut self, duration: u32) -> Self {
        self.duration = Some(duration);
        self
    }

    pub fn with_streaming(mut self, supports_streaming: bool) -> Self {
        self.supports_streaming = Some(supports_streaming);
        self
    }

    pub fn validate(&self) -> Result<()> {
        validate_required_visible_text("input_paid_media.media", &self.media)?;
        if let Some(thumbnail) = self.thumbnail.as_deref() {
            validate_required_visible_text("input_paid_media.thumbnail", thumbnail)?;
        }
        if let Some(cover) = self.cover.as_deref() {
            validate_required_visible_text("input_paid_media.cover", cover)?;
        }
        validate_positive_optional_u32("input_paid_media.width", self.width)?;
        validate_positive_optional_u32("input_paid_media.height", self.height)?;
        validate_positive_optional_u32("input_paid_media.duration", self.duration)
    }
}

impl Serialize for InputPaidMediaVideo {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = [
            "media",
            "thumbnail",
            "cover",
            "start_timestamp",
            "width",
            "height",
            "duration",
            "supports_streaming",
        ];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let optional_len = usize::from(self.thumbnail.is_some())
            + usize::from(self.cover.is_some())
            + usize::from(self.start_timestamp.is_some())
            + usize::from(self.width.is_some())
            + usize::from(self.height.is_some())
            + usize::from(self.duration.is_some())
            + usize::from(self.supports_streaming.is_some());
        let mut object = serializer.serialize_map(Some(extra_len + optional_len + 1))?;
        object.serialize_entry("media", &self.media)?;
        if let Some(thumbnail) = self.thumbnail.as_ref() {
            object.serialize_entry("thumbnail", thumbnail)?;
        }
        if let Some(cover) = self.cover.as_ref() {
            object.serialize_entry("cover", cover)?;
        }
        if let Some(start_timestamp) = self.start_timestamp {
            object.serialize_entry("start_timestamp", &start_timestamp)?;
        }
        if let Some(width) = self.width {
            object.serialize_entry("width", &width)?;
        }
        if let Some(height) = self.height {
            object.serialize_entry("height", &height)?;
        }
        if let Some(duration) = self.duration {
            object.serialize_entry("duration", &duration)?;
        }
        if let Some(supports_streaming) = self.supports_streaming {
            object.serialize_entry("supports_streaming", &supports_streaming)?;
        }
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
}

fn validate_positive_optional_u32(field: &str, value: Option<u32>) -> Result<()> {
    if value == Some(0) {
        return Err(invalid_request(format!("{field} must be greater than 0")));
    }

    Ok(())
}

/// Paid media item for `sendPaidMedia`.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum InputPaidMedia {
    LivePhoto(InputPaidMediaLivePhoto),
    Photo(InputPaidMediaPhoto),
    Video(InputPaidMediaVideo),
    Unknown(Value),
}

impl InputPaidMedia {
    pub fn live_photo(media: impl Into<String>, photo: impl Into<String>) -> Self {
        Self::LivePhoto(InputPaidMediaLivePhoto::new(media, photo))
    }

    pub fn photo(media: impl Into<String>) -> Self {
        Self::Photo(InputPaidMediaPhoto::new(media))
    }

    pub fn video(media: impl Into<String>) -> Self {
        Self::Video(InputPaidMediaVideo::new(media))
    }

    pub fn new(value: Value) -> Result<Self> {
        Self::try_from_value(value)
    }

    pub fn try_from_value(value: Value) -> Result<Self> {
        let media = serde_json::from_value::<Self>(value).map_err(|source| {
            invalid_request(format!("invalid input_paid_media payload: {source}"))
        })?;
        media.validate()?;
        Ok(media)
    }

    pub fn try_from_typed<T>(value: T) -> Result<Self>
    where
        T: Serialize,
    {
        let value =
            serde_json::to_value(value).map_err(|source| Error::SerializeRequest { source })?;
        Self::try_from_value(value)
    }

    pub fn kind(&self) -> Option<&str> {
        match self {
            Self::LivePhoto(_) => Some("live_photo"),
            Self::Photo(_) => Some("photo"),
            Self::Video(_) => Some("video"),
            Self::Unknown(value) => tagged_kind(value),
        }
    }

    pub fn validate(&self) -> Result<()> {
        match self {
            Self::LivePhoto(value) => value.validate(),
            Self::Photo(value) => value.validate(),
            Self::Video(value) => value.validate(),
            Self::Unknown(value) => validate_typed_object_payload("input_paid_media", value),
        }
    }
}

impl From<InputPaidMediaLivePhoto> for InputPaidMedia {
    fn from(value: InputPaidMediaLivePhoto) -> Self {
        Self::LivePhoto(value)
    }
}

impl From<InputPaidMediaPhoto> for InputPaidMedia {
    fn from(value: InputPaidMediaPhoto) -> Self {
        Self::Photo(value)
    }
}

impl From<InputPaidMediaVideo> for InputPaidMedia {
    fn from(value: InputPaidMediaVideo) -> Self {
        Self::Video(value)
    }
}

impl<'de> Deserialize<'de> for InputPaidMedia {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        Self::try_from(value).map_err(serde::de::Error::custom)
    }
}

impl TryFrom<Value> for InputPaidMedia {
    type Error = Error;

    fn try_from(value: Value) -> std::result::Result<Self, Self::Error> {
        match tagged_kind(&value) {
            Some("live_photo") => deserialize_input_paid_media_known(value, Self::LivePhoto),
            Some("photo") => deserialize_input_paid_media_known(value, Self::Photo),
            Some("video") => deserialize_input_paid_media_known(value, Self::Video),
            Some(_) => {
                validate_typed_object_payload("input_paid_media", &value)?;
                Ok(Self::Unknown(value))
            }
            None => Err(invalid_request(
                "input_paid_media requires a string `type` field",
            )),
        }
    }
}

fn deserialize_input_paid_media_known<T>(
    value: Value,
    constructor: impl FnOnce(T) -> InputPaidMedia,
) -> Result<InputPaidMedia>
where
    T: DeserializeOwned,
{
    let payload = serde_json::from_value::<T>(strip_type(value))
        .map_err(|source| invalid_request(format!("invalid input_paid_media payload: {source}")))?;
    let media = constructor(payload);
    media.validate()?;
    Ok(media)
}

impl Serialize for InputPaidMedia {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::LivePhoto(value) => serialize_typed_payload(serializer, "live_photo", value),
            Self::Photo(value) => serialize_typed_payload(serializer, "photo", value),
            Self::Video(value) => serialize_typed_payload(serializer, "video", value),
            Self::Unknown(value) => value.serialize(serializer),
        }
    }
}

/// Emoji reaction payload.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[non_exhaustive]
pub struct ReactionTypeEmoji {
    pub emoji: String,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl ReactionTypeEmoji {
    pub fn new(emoji: impl Into<String>) -> Self {
        Self {
            emoji: emoji.into(),
            extra: BTreeMap::new(),
        }
    }

    pub fn validate(&self) -> Result<()> {
        validate_required_visible_text("reaction_type.emoji", &self.emoji)
    }
}

impl Serialize for ReactionTypeEmoji {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = ["emoji"];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let mut object = serializer.serialize_map(Some(extra_len + 1))?;
        object.serialize_entry("emoji", &self.emoji)?;
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
}

/// Custom emoji reaction payload.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[non_exhaustive]
pub struct ReactionTypeCustomEmoji {
    pub custom_emoji_id: String,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl ReactionTypeCustomEmoji {
    pub fn new(custom_emoji_id: impl Into<String>) -> Self {
        Self {
            custom_emoji_id: custom_emoji_id.into(),
            extra: BTreeMap::new(),
        }
    }

    pub fn validate(&self) -> Result<()> {
        validate_required_visible_text("reaction_type.custom_emoji_id", &self.custom_emoji_id)
    }
}

impl Serialize for ReactionTypeCustomEmoji {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = ["custom_emoji_id"];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let mut object = serializer.serialize_map(Some(extra_len + 1))?;
        object.serialize_entry("custom_emoji_id", &self.custom_emoji_id)?;
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
}

/// Paid reaction payload.
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[non_exhaustive]
pub struct ReactionTypePaid {
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl ReactionTypePaid {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn validate(&self) -> Result<()> {
        Ok(())
    }
}

impl Serialize for ReactionTypePaid {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = ["type"];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let mut object = serializer.serialize_map(Some(extra_len))?;
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
}

/// Reaction type payload.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum ReactionType {
    Emoji(ReactionTypeEmoji),
    CustomEmoji(ReactionTypeCustomEmoji),
    Paid(ReactionTypePaid),
    Unknown(Value),
}

impl ReactionType {
    pub fn emoji(emoji: impl Into<String>) -> Self {
        Self::Emoji(ReactionTypeEmoji::new(emoji))
    }

    pub fn custom_emoji(custom_emoji_id: impl Into<String>) -> Self {
        Self::CustomEmoji(ReactionTypeCustomEmoji::new(custom_emoji_id))
    }

    pub fn paid() -> Self {
        Self::Paid(ReactionTypePaid::new())
    }

    pub fn new(value: Value) -> Result<Self> {
        Self::try_from_value(value)
    }

    pub fn try_from_value(value: Value) -> Result<Self> {
        let reaction = serde_json::from_value::<Self>(value).map_err(|source| {
            invalid_request(format!("invalid reaction_type payload: {source}"))
        })?;
        reaction.validate()?;
        Ok(reaction)
    }

    pub fn try_from_typed<T>(value: T) -> Result<Self>
    where
        T: Serialize,
    {
        let value =
            serde_json::to_value(value).map_err(|source| Error::SerializeRequest { source })?;
        Self::try_from_value(value)
    }

    pub fn kind(&self) -> Option<&str> {
        match self {
            Self::Emoji(_) => Some("emoji"),
            Self::CustomEmoji(_) => Some("custom_emoji"),
            Self::Paid(_) => Some("paid"),
            Self::Unknown(value) => tagged_kind(value),
        }
    }

    pub fn validate(&self) -> Result<()> {
        match self {
            Self::Emoji(value) => value.validate(),
            Self::CustomEmoji(value) => value.validate(),
            Self::Paid(value) => value.validate(),
            Self::Unknown(value) => validate_typed_object_payload("reaction_type", value),
        }
    }
}

impl From<ReactionTypeEmoji> for ReactionType {
    fn from(value: ReactionTypeEmoji) -> Self {
        Self::Emoji(value)
    }
}

impl From<ReactionTypeCustomEmoji> for ReactionType {
    fn from(value: ReactionTypeCustomEmoji) -> Self {
        Self::CustomEmoji(value)
    }
}

impl From<ReactionTypePaid> for ReactionType {
    fn from(value: ReactionTypePaid) -> Self {
        Self::Paid(value)
    }
}

impl<'de> Deserialize<'de> for ReactionType {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        Self::try_from(value).map_err(serde::de::Error::custom)
    }
}

impl TryFrom<Value> for ReactionType {
    type Error = Error;

    fn try_from(value: Value) -> std::result::Result<Self, Self::Error> {
        match tagged_kind(&value) {
            Some("emoji") => deserialize_reaction_type_known(value, Self::Emoji),
            Some("custom_emoji") => deserialize_reaction_type_known(value, Self::CustomEmoji),
            Some("paid") => deserialize_reaction_type_known(value, Self::Paid),
            Some(_) => {
                validate_typed_object_payload("reaction_type", &value)?;
                Ok(Self::Unknown(value))
            }
            None => Err(invalid_request(
                "reaction_type requires a string `type` field",
            )),
        }
    }
}

fn deserialize_reaction_type_known<T>(
    value: Value,
    constructor: impl FnOnce(T) -> ReactionType,
) -> Result<ReactionType>
where
    T: DeserializeOwned,
{
    let payload = serde_json::from_value::<T>(strip_type(value))
        .map_err(|source| invalid_request(format!("invalid reaction_type payload: {source}")))?;
    let reaction = constructor(payload);
    reaction.validate()?;
    Ok(reaction)
}

impl Serialize for ReactionType {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Emoji(value) => serialize_typed_payload(serializer, "emoji", value),
            Self::CustomEmoji(value) => serialize_typed_payload(serializer, "custom_emoji", value),
            Self::Paid(value) => serialize_typed_payload(serializer, "paid", value),
            Self::Unknown(value) => value.serialize(serializer),
        }
    }
}

/// Position and shape of a clickable story area, expressed in percentages.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[non_exhaustive]
pub struct StoryAreaPosition {
    pub x_percentage: f64,
    pub y_percentage: f64,
    pub width_percentage: f64,
    pub height_percentage: f64,
    pub rotation_angle: f64,
    pub corner_radius_percentage: f64,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl StoryAreaPosition {
    pub fn new(
        x_percentage: f64,
        y_percentage: f64,
        width_percentage: f64,
        height_percentage: f64,
        rotation_angle: f64,
        corner_radius_percentage: f64,
    ) -> Self {
        Self {
            x_percentage,
            y_percentage,
            width_percentage,
            height_percentage,
            rotation_angle,
            corner_radius_percentage,
            extra: BTreeMap::new(),
        }
    }

    pub fn validate(&self) -> Result<()> {
        validate_float_range(
            "story_area.position.x_percentage",
            self.x_percentage,
            0.0,
            100.0,
        )?;
        validate_float_range(
            "story_area.position.y_percentage",
            self.y_percentage,
            0.0,
            100.0,
        )?;
        validate_positive_percentage(
            "story_area.position.width_percentage",
            self.width_percentage,
        )?;
        validate_positive_percentage(
            "story_area.position.height_percentage",
            self.height_percentage,
        )?;
        validate_float_range(
            "story_area.position.rotation_angle",
            self.rotation_angle,
            0.0,
            360.0,
        )?;
        validate_float_range(
            "story_area.position.corner_radius_percentage",
            self.corner_radius_percentage,
            0.0,
            100.0,
        )
    }
}

impl Serialize for StoryAreaPosition {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = [
            "x_percentage",
            "y_percentage",
            "width_percentage",
            "height_percentage",
            "rotation_angle",
            "corner_radius_percentage",
        ];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let mut object = serializer.serialize_map(Some(extra_len + 6))?;
        object.serialize_entry("x_percentage", &self.x_percentage)?;
        object.serialize_entry("y_percentage", &self.y_percentage)?;
        object.serialize_entry("width_percentage", &self.width_percentage)?;
        object.serialize_entry("height_percentage", &self.height_percentage)?;
        object.serialize_entry("rotation_angle", &self.rotation_angle)?;
        object.serialize_entry("corner_radius_percentage", &self.corner_radius_percentage)?;
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
}

/// Physical address attached to a location story area.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[non_exhaustive]
pub struct LocationAddress {
    pub country_code: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub street: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl LocationAddress {
    pub fn new(country_code: impl Into<String>) -> Self {
        Self {
            country_code: country_code.into(),
            state: None,
            city: None,
            street: None,
            extra: BTreeMap::new(),
        }
    }

    pub fn validate(&self) -> Result<()> {
        if self.country_code.len() != 2
            || !self
                .country_code
                .bytes()
                .all(|byte| byte.is_ascii_alphabetic())
        {
            return Err(invalid_request(
                "location_address.country_code must be a two-letter ISO country code",
            ));
        }
        validate_optional_visible_text("location_address.state", self.state.as_deref())?;
        validate_optional_visible_text("location_address.city", self.city.as_deref())?;
        validate_optional_visible_text("location_address.street", self.street.as_deref())
    }
}

impl Serialize for LocationAddress {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = ["country_code", "state", "city", "street"];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let optional_len = usize::from(self.state.is_some())
            + usize::from(self.city.is_some())
            + usize::from(self.street.is_some());
        let mut object = serializer.serialize_map(Some(extra_len + optional_len + 1))?;
        object.serialize_entry("country_code", &self.country_code)?;
        if let Some(state) = self.state.as_ref() {
            object.serialize_entry("state", state)?;
        }
        if let Some(city) = self.city.as_ref() {
            object.serialize_entry("city", city)?;
        }
        if let Some(street) = self.street.as_ref() {
            object.serialize_entry("street", street)?;
        }
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
}

/// Story area type that points to a location.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[non_exhaustive]
pub struct StoryAreaTypeLocation {
    pub latitude: f64,
    pub longitude: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub address: Option<LocationAddress>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl StoryAreaTypeLocation {
    pub fn new(latitude: f64, longitude: f64) -> Self {
        Self {
            latitude,
            longitude,
            address: None,
            extra: BTreeMap::new(),
        }
    }

    pub fn with_address(mut self, address: impl Into<LocationAddress>) -> Self {
        self.address = Some(address.into());
        self
    }

    pub fn validate(&self) -> Result<()> {
        validate_float_range("story_area.type.latitude", self.latitude, -90.0, 90.0)?;
        validate_float_range("story_area.type.longitude", self.longitude, -180.0, 180.0)?;
        if let Some(address) = self.address.as_ref() {
            address.validate()?;
        }

        Ok(())
    }
}

/// Story area type that suggests a reaction.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct StoryAreaTypeSuggestedReaction {
    pub reaction_type: ReactionType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_dark: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_flipped: Option<bool>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl StoryAreaTypeSuggestedReaction {
    pub fn new(reaction_type: ReactionType) -> Self {
        Self {
            reaction_type,
            is_dark: None,
            is_flipped: None,
            extra: BTreeMap::new(),
        }
    }

    pub fn validate(&self) -> Result<()> {
        self.reaction_type.validate()
    }
}

impl Serialize for StoryAreaTypeSuggestedReaction {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = ["reaction_type", "is_dark", "is_flipped"];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let optional_len =
            usize::from(self.is_dark.is_some()) + usize::from(self.is_flipped.is_some());
        let mut object = serializer.serialize_map(Some(extra_len + optional_len + 1))?;
        object.serialize_entry("reaction_type", &self.reaction_type)?;
        if let Some(is_dark) = self.is_dark {
            object.serialize_entry("is_dark", &is_dark)?;
        }
        if let Some(is_flipped) = self.is_flipped {
            object.serialize_entry("is_flipped", &is_flipped)?;
        }
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
}

/// Story area type that opens an HTTP, HTTPS, or tg:// URL.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[non_exhaustive]
pub struct StoryAreaTypeLink {
    pub url: String,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl StoryAreaTypeLink {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            extra: BTreeMap::new(),
        }
    }

    pub fn validate(&self) -> Result<()> {
        validate_url("story_area.type.url", &self.url)
    }
}

/// Story area type that displays weather information.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[non_exhaustive]
pub struct StoryAreaTypeWeather {
    pub temperature: f64,
    pub emoji: String,
    pub background_color: u32,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl StoryAreaTypeWeather {
    pub fn new(temperature: f64, emoji: impl Into<String>, background_color: u32) -> Self {
        Self {
            temperature,
            emoji: emoji.into(),
            background_color,
            extra: BTreeMap::new(),
        }
    }

    pub fn validate(&self) -> Result<()> {
        validate_finite_float("story_area.type.temperature", self.temperature)?;
        validate_required_visible_text("story_area.type.emoji", &self.emoji)
    }
}

/// Story area type that points to a unique gift.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[non_exhaustive]
pub struct StoryAreaTypeUniqueGift {
    pub name: String,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl StoryAreaTypeUniqueGift {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            extra: BTreeMap::new(),
        }
    }

    pub fn validate(&self) -> Result<()> {
        validate_required_visible_text("story_area.type.name", &self.name)
    }
}

/// Clickable story area type.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum StoryAreaType {
    Location(StoryAreaTypeLocation),
    SuggestedReaction(StoryAreaTypeSuggestedReaction),
    Link(StoryAreaTypeLink),
    Weather(StoryAreaTypeWeather),
    UniqueGift(StoryAreaTypeUniqueGift),
    Unknown(Value),
}

impl StoryAreaType {
    pub fn location(latitude: f64, longitude: f64) -> Self {
        Self::Location(StoryAreaTypeLocation::new(latitude, longitude))
    }

    pub fn suggested_reaction(reaction_type: ReactionType) -> Self {
        Self::SuggestedReaction(StoryAreaTypeSuggestedReaction::new(reaction_type))
    }

    pub fn link(url: impl Into<String>) -> Self {
        Self::Link(StoryAreaTypeLink::new(url))
    }

    pub fn weather(temperature: f64, emoji: impl Into<String>, background_color: u32) -> Self {
        Self::Weather(StoryAreaTypeWeather::new(
            temperature,
            emoji,
            background_color,
        ))
    }

    pub fn unique_gift(name: impl Into<String>) -> Self {
        Self::UniqueGift(StoryAreaTypeUniqueGift::new(name))
    }

    pub fn kind(&self) -> Option<&str> {
        match self {
            Self::Location(_) => Some("location"),
            Self::SuggestedReaction(_) => Some("suggested_reaction"),
            Self::Link(_) => Some("link"),
            Self::Weather(_) => Some("weather"),
            Self::UniqueGift(_) => Some("unique_gift"),
            Self::Unknown(value) => tagged_kind(value),
        }
    }

    pub fn validate(&self) -> Result<()> {
        match self {
            Self::Location(value) => value.validate(),
            Self::SuggestedReaction(value) => value.validate(),
            Self::Link(value) => value.validate(),
            Self::Weather(value) => value.validate(),
            Self::UniqueGift(value) => value.validate(),
            Self::Unknown(value) => validate_typed_object_payload("story_area.type", value),
        }
    }
}

impl From<StoryAreaTypeLocation> for StoryAreaType {
    fn from(value: StoryAreaTypeLocation) -> Self {
        Self::Location(value)
    }
}

impl From<StoryAreaTypeSuggestedReaction> for StoryAreaType {
    fn from(value: StoryAreaTypeSuggestedReaction) -> Self {
        Self::SuggestedReaction(value)
    }
}

impl From<StoryAreaTypeLink> for StoryAreaType {
    fn from(value: StoryAreaTypeLink) -> Self {
        Self::Link(value)
    }
}

impl From<StoryAreaTypeWeather> for StoryAreaType {
    fn from(value: StoryAreaTypeWeather) -> Self {
        Self::Weather(value)
    }
}

impl From<StoryAreaTypeUniqueGift> for StoryAreaType {
    fn from(value: StoryAreaTypeUniqueGift) -> Self {
        Self::UniqueGift(value)
    }
}

impl<'de> Deserialize<'de> for StoryAreaType {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        Self::try_from(value).map_err(serde::de::Error::custom)
    }
}

impl TryFrom<Value> for StoryAreaType {
    type Error = Error;

    fn try_from(value: Value) -> std::result::Result<Self, Self::Error> {
        match tagged_kind(&value) {
            Some("location") => deserialize_story_area_type_known(value, Self::Location),
            Some("suggested_reaction") => {
                deserialize_story_area_type_known(value, Self::SuggestedReaction)
            }
            Some("link") => deserialize_story_area_type_known(value, Self::Link),
            Some("weather") => deserialize_story_area_type_known(value, Self::Weather),
            Some("unique_gift") => deserialize_story_area_type_known(value, Self::UniqueGift),
            Some(_) => {
                validate_typed_object_payload("story_area.type", &value)?;
                Ok(Self::Unknown(value))
            }
            None => Err(invalid_request(
                "story_area.type requires a string `type` field",
            )),
        }
    }
}

impl Serialize for StoryAreaType {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Location(value) => serialize_typed_payload(serializer, "location", value),
            Self::SuggestedReaction(value) => {
                serialize_typed_payload(serializer, "suggested_reaction", value)
            }
            Self::Link(value) => serialize_typed_payload(serializer, "link", value),
            Self::Weather(value) => serialize_typed_payload(serializer, "weather", value),
            Self::UniqueGift(value) => serialize_typed_payload(serializer, "unique_gift", value),
            Self::Unknown(value) => value.serialize(serializer),
        }
    }
}

fn serialize_typed_payload<S, T>(
    serializer: S,
    kind: &str,
    payload: &T,
) -> std::result::Result<S::Ok, S::Error>
where
    S: serde::Serializer,
    T: Serialize,
{
    let mut value = serde_json::to_value(payload).map_err(serde::ser::Error::custom)?;
    let object = value.as_object_mut().ok_or_else(|| {
        serde::ser::Error::custom("story area type payload must serialize to a JSON object")
    })?;
    object.insert("type".to_owned(), Value::String(kind.to_owned()));
    value.serialize(serializer)
}

fn deserialize_story_area_type_known<T>(
    value: Value,
    constructor: impl FnOnce(T) -> StoryAreaType,
) -> Result<StoryAreaType>
where
    T: DeserializeOwned,
{
    let payload = serde_json::from_value::<T>(strip_type(value))
        .map_err(|source| invalid_request(format!("invalid story_area.type payload: {source}")))?;
    let area_type = constructor(payload);
    area_type.validate()?;
    Ok(area_type)
}

impl Serialize for StoryAreaTypeLocation {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = ["latitude", "longitude", "address"];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let optional_len = usize::from(self.address.is_some());
        let mut object = serializer.serialize_map(Some(extra_len + optional_len + 2))?;
        object.serialize_entry("latitude", &self.latitude)?;
        object.serialize_entry("longitude", &self.longitude)?;
        if let Some(address) = self.address.as_ref() {
            object.serialize_entry("address", address)?;
        }
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
}

impl Serialize for StoryAreaTypeLink {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = ["url"];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let mut object = serializer.serialize_map(Some(extra_len + 1))?;
        object.serialize_entry("url", &self.url)?;
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
}

impl Serialize for StoryAreaTypeWeather {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = ["temperature", "emoji", "background_color"];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let mut object = serializer.serialize_map(Some(extra_len + 3))?;
        object.serialize_entry("temperature", &self.temperature)?;
        object.serialize_entry("emoji", &self.emoji)?;
        object.serialize_entry("background_color", &self.background_color)?;
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
}

impl Serialize for StoryAreaTypeUniqueGift {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = ["name"];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let mut object = serializer.serialize_map(Some(extra_len + 1))?;
        object.serialize_entry("name", &self.name)?;
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
}

/// Clickable area on a story media.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct StoryArea {
    pub position: StoryAreaPosition,
    pub kind: StoryAreaType,
    pub extra: BTreeMap<String, Value>,
}

impl StoryArea {
    pub fn new(position: StoryAreaPosition, kind: impl Into<StoryAreaType>) -> Self {
        Self {
            position,
            kind: kind.into(),
            extra: BTreeMap::new(),
        }
    }

    pub fn location(position: StoryAreaPosition, latitude: f64, longitude: f64) -> Self {
        Self::new(position, StoryAreaTypeLocation::new(latitude, longitude))
    }

    pub fn link(position: StoryAreaPosition, url: impl Into<String>) -> Self {
        Self::new(position, StoryAreaTypeLink::new(url))
    }

    pub fn weather(
        position: StoryAreaPosition,
        temperature: f64,
        emoji: impl Into<String>,
        background_color: u32,
    ) -> Self {
        Self::new(
            position,
            StoryAreaTypeWeather::new(temperature, emoji, background_color),
        )
    }

    pub fn unique_gift(position: StoryAreaPosition, name: impl Into<String>) -> Self {
        Self::new(position, StoryAreaTypeUniqueGift::new(name))
    }

    pub fn try_from_value(value: Value) -> Result<Self> {
        let story_area = serde_json::from_value::<Self>(value)
            .map_err(|source| invalid_request(format!("invalid story_area payload: {source}")))?;
        story_area.validate()?;
        Ok(story_area)
    }

    pub fn try_from_typed<T>(value: T) -> Result<Self>
    where
        T: Serialize,
    {
        let value =
            serde_json::to_value(value).map_err(|source| Error::SerializeRequest { source })?;
        Self::try_from_value(value)
    }

    pub fn validate(&self) -> Result<()> {
        self.position.validate()?;
        self.kind.validate()
    }
}

impl<'de> Deserialize<'de> for StoryArea {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawStoryArea {
            position: StoryAreaPosition,
            #[serde(rename = "type")]
            kind: StoryAreaType,
            #[serde(flatten)]
            extra: BTreeMap<String, Value>,
        }

        let raw = RawStoryArea::deserialize(deserializer)?;
        let story_area = Self {
            position: raw.position,
            kind: raw.kind,
            extra: raw.extra,
        };
        story_area.validate().map_err(serde::de::Error::custom)?;
        Ok(story_area)
    }
}

impl TryFrom<Value> for StoryArea {
    type Error = Error;

    fn try_from(value: Value) -> std::result::Result<Self, Self::Error> {
        Self::try_from_value(value)
    }
}

impl Serialize for StoryArea {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = ["position", "type"];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let mut object = serializer.serialize_map(Some(extra_len + 2))?;
        object.serialize_entry("position", &self.position)?;
        object.serialize_entry("type", &self.kind)?;
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
}

pub(crate) fn validate_story_areas(field: &str, values: &[StoryArea]) -> Result<()> {
    let mut location_count = 0;
    let mut suggested_reaction_count = 0;
    let mut link_count = 0;
    let mut weather_count = 0;
    let mut unique_gift_count = 0;

    for value in values {
        value.validate()?;
        match value.kind.kind() {
            Some("location") => location_count += 1,
            Some("suggested_reaction") => suggested_reaction_count += 1,
            Some("link") => link_count += 1,
            Some("weather") => weather_count += 1,
            Some("unique_gift") => unique_gift_count += 1,
            Some(_) | None => {}
        }
    }

    validate_story_area_type_count(field, "location", location_count, 10)?;
    validate_story_area_type_count(field, "suggested_reaction", suggested_reaction_count, 5)?;
    validate_story_area_type_count(field, "link", link_count, 3)?;
    validate_story_area_type_count(field, "weather", weather_count, 3)?;
    validate_story_area_type_count(field, "unique_gift", unique_gift_count, 1)
}

fn validate_story_area_type_count(field: &str, kind: &str, count: usize, max: usize) -> Result<()> {
    if count > max {
        return Err(invalid_request(format!(
            "{field} accepts at most {max} {kind} story areas"
        )));
    }

    Ok(())
}

fn is_compact_callback_safe(byte: u8) -> bool {
    matches!(
        byte,
        b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~'
    )
}

fn encode_compact_callback_segment(segment: &str) -> String {
    let mut encoded = String::with_capacity(segment.len());
    for byte in segment.as_bytes() {
        if is_compact_callback_safe(*byte) {
            encoded.push(*byte as char);
        } else {
            let _ = write!(&mut encoded, "%{byte:02X}");
        }
    }
    encoded
}

fn decode_hex_digit(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn decode_compact_callback_segment(segment: &str) -> Result<String> {
    let mut bytes = Vec::with_capacity(segment.len());
    let raw = segment.as_bytes();
    let mut index = 0;
    while index < raw.len() {
        if raw[index] == b'%' {
            let hi = *raw
                .get(index + 1)
                .ok_or_else(|| invalid_request("compact callback segment has truncated escape"))?;
            let lo = *raw
                .get(index + 2)
                .ok_or_else(|| invalid_request("compact callback segment has truncated escape"))?;
            let hi = decode_hex_digit(hi).ok_or_else(|| {
                invalid_request("compact callback segment contains invalid escape")
            })?;
            let lo = decode_hex_digit(lo).ok_or_else(|| {
                invalid_request("compact callback segment contains invalid escape")
            })?;
            bytes.push((hi << 4) | lo);
            index += 3;
        } else {
            bytes.push(raw[index]);
            index += 1;
        }
    }

    String::from_utf8(bytes)
        .map_err(|_| invalid_request("compact callback segment is not valid UTF-8"))
}

/// Pluggable callback payload codec for inline keyboard buttons and callback routers.
pub trait CallbackCodec<T>: Send + Sync + 'static {
    fn encode_callback_data(payload: &T) -> Result<String>;
    fn decode_callback_data(data: &str) -> Result<T>;
}

/// Adapter codec for payload types that implement [`CallbackPayload`].
#[derive(Clone, Copy, Debug, Default)]
pub struct CallbackPayloadCodec;

impl<T> CallbackCodec<T> for CallbackPayloadCodec
where
    T: CallbackPayload,
{
    fn encode_callback_data(payload: &T) -> Result<String> {
        payload.encode_callback_data()
    }

    fn decode_callback_data(data: &str) -> Result<T> {
        T::decode_callback_data(data)
    }
}

/// JSON callback codec for serde-serializable payloads.
#[derive(Clone, Copy, Debug, Default)]
pub struct JsonCallbackCodec;

impl<T> CallbackCodec<T> for JsonCallbackCodec
where
    T: Serialize + DeserializeOwned,
{
    fn encode_callback_data(payload: &T) -> Result<String> {
        let encoded =
            serde_json::to_string(payload).map_err(|source| Error::SerializeRequest { source })?;
        validate_callback_data(encoded)
    }

    fn decode_callback_data(data: &str) -> Result<T> {
        serde_json::from_str(data).map_err(|source| {
            invalid_request(format!("failed to decode callback payload: {source}"))
        })
    }
}

/// Builder for compact callback payload strings.
#[derive(Clone, Debug, Default)]
pub struct CompactCallbackEncoder {
    segments: Vec<String>,
}

impl CompactCallbackEncoder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn tag(&mut self, tag: impl AsRef<str>) -> Result<&mut Self> {
        let tag = tag.as_ref().trim();
        if tag.is_empty() {
            return Err(invalid_request("compact callback tag cannot be empty"));
        }
        self.segments.push(encode_compact_callback_segment(tag));
        Ok(self)
    }

    pub fn push(&mut self, value: impl AsRef<str>) -> Result<&mut Self> {
        self.segments
            .push(encode_compact_callback_segment(value.as_ref()));
        Ok(self)
    }

    pub fn push_display(&mut self, value: impl Display) -> Result<&mut Self> {
        self.push(value.to_string())
    }

    pub fn finish(self) -> Result<String> {
        if self.segments.is_empty() {
            return Err(invalid_request("compact callback payload cannot be empty"));
        }
        validate_callback_data(self.segments.join(":"))
    }
}

/// Decoder for compact callback payload strings.
#[derive(Clone, Debug)]
pub struct CompactCallbackDecoder {
    segments: Vec<String>,
    index: usize,
}

impl CompactCallbackDecoder {
    pub fn new(data: &str) -> Result<Self> {
        if data.is_empty() {
            return Err(invalid_request("compact callback payload cannot be empty"));
        }
        let segments = data
            .split(':')
            .map(decode_compact_callback_segment)
            .collect::<Result<Vec<_>>>()?;
        Ok(Self { segments, index: 0 })
    }

    pub fn expect_tag(&mut self, expected: &str) -> Result<&mut Self> {
        let actual = self.next_string("callback tag")?;
        if actual == expected {
            Ok(self)
        } else {
            Err(invalid_request(format!(
                "unexpected compact callback tag `{actual}`, expected `{expected}`"
            )))
        }
    }

    pub fn next_string(&mut self, field: &str) -> Result<String> {
        let value = self.segments.get(self.index).cloned().ok_or_else(|| {
            invalid_request(format!(
                "compact callback payload is missing required field `{field}`"
            ))
        })?;
        self.index += 1;
        Ok(value)
    }

    pub fn next_parse<T>(&mut self, field: &str) -> Result<T>
    where
        T: FromStr,
        T::Err: Display,
    {
        let raw = self.next_string(field)?;
        raw.parse().map_err(|source| {
            invalid_request(format!(
                "failed to parse compact callback field `{field}`: {source}"
            ))
        })
    }

    pub fn remaining(&self) -> usize {
        self.segments.len().saturating_sub(self.index)
    }

    pub fn finish(self) -> Result<()> {
        if self.remaining() == 0 {
            Ok(())
        } else {
            Err(invalid_request(format!(
                "compact callback payload has {} unexpected trailing field(s)",
                self.remaining()
            )))
        }
    }
}

/// Manual compact callback payload contract for 64-byte-friendly callback data.
pub trait CompactCallbackPayload: Sized {
    fn encode_compact(&self, encoder: &mut CompactCallbackEncoder) -> Result<()>;
    fn decode_compact(decoder: &mut CompactCallbackDecoder) -> Result<Self>;

    fn encode_compact_data(&self) -> Result<String> {
        let mut encoder = CompactCallbackEncoder::new();
        self.encode_compact(&mut encoder)?;
        encoder.finish()
    }

    fn decode_compact_data(data: &str) -> Result<Self> {
        let mut decoder = CompactCallbackDecoder::new(data)?;
        let payload = Self::decode_compact(&mut decoder)?;
        decoder.finish()?;
        Ok(payload)
    }
}

/// Compact callback codec backed by [`CompactCallbackPayload`].
#[derive(Clone, Copy, Debug, Default)]
pub struct CompactCallbackCodec;

impl<T> CallbackCodec<T> for CompactCallbackCodec
where
    T: CompactCallbackPayload,
{
    fn encode_callback_data(payload: &T) -> Result<String> {
        payload.encode_compact_data()
    }

    fn decode_callback_data(data: &str) -> Result<T> {
        T::decode_compact_data(data)
    }
}

/// Strongly-typed callback payload codec for inline keyboard buttons.
pub trait CallbackPayload: Sized {
    fn encode_callback_data(&self) -> Result<String>;
    fn decode_callback_data(data: &str) -> Result<Self>;
}

impl<T> CallbackPayload for T
where
    T: Serialize + DeserializeOwned,
{
    fn encode_callback_data(&self) -> Result<String> {
        JsonCallbackCodec::encode_callback_data(self)
    }

    fn decode_callback_data(data: &str) -> Result<Self> {
        JsonCallbackCodec::decode_callback_data(data)
    }
}

const MAX_INLINE_QUERY_RESULT_ID_BYTES: usize = 64;

fn validate_inline_query_result_value(value: &Value) -> Result<()> {
    let Some(object) = value.as_object() else {
        return Err(invalid_request("inline query result must be a JSON object"));
    };

    let Some(kind) = object.get("type").and_then(Value::as_str) else {
        return Err(invalid_request(
            "inline query result requires a string `type` field",
        ));
    };
    if kind.trim().is_empty() || kind.chars().any(char::is_control) {
        return Err(invalid_request(
            "inline query result `type` must be a non-empty visible string",
        ));
    }

    let Some(id) = object.get("id").and_then(Value::as_str) else {
        return Err(invalid_request(
            "inline query result requires a string `id` field",
        ));
    };
    if id.trim().is_empty() {
        return Err(invalid_request("inline query result `id` cannot be empty"));
    }
    if id.len() > MAX_INLINE_QUERY_RESULT_ID_BYTES {
        return Err(invalid_request(format!(
            "inline query result `id` exceeds {MAX_INLINE_QUERY_RESULT_ID_BYTES} bytes"
        )));
    }
    if id.chars().any(char::is_control) {
        return Err(invalid_request(
            "inline query result `id` must not contain control characters",
        ));
    }

    Ok(())
}

/// Generic inline query result payload.
#[derive(Clone, Debug, Serialize)]
#[serde(transparent)]
pub struct InlineQueryResult(Value);

impl InlineQueryResult {
    pub fn new(value: Value) -> Result<Self> {
        validate_inline_query_result_value(&value)?;
        Ok(Self(value))
    }

    pub fn try_from_typed<T>(value: T) -> Result<Self>
    where
        T: Serialize,
    {
        let value =
            serde_json::to_value(value).map_err(|source| Error::SerializeRequest { source })?;
        Self::new(value)
    }

    pub fn from_typed<T>(value: T) -> Result<Self>
    where
        T: Serialize,
    {
        Self::try_from_typed(value)
    }

    pub fn article(
        id: impl Into<String>,
        title: impl Into<String>,
        message_text: impl Into<String>,
    ) -> Result<Self> {
        InlineQueryResult::try_from(InlineQueryResultArticle::new(id, title, message_text))
    }

    pub fn validate(&self) -> Result<()> {
        validate_inline_query_result_value(&self.0)
    }

    pub fn as_value(&self) -> &Value {
        &self.0
    }

    pub fn into_value(self) -> Value {
        self.0
    }
}

impl<'de> Deserialize<'de> for InlineQueryResult {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

impl From<InlineQueryResult> for Value {
    fn from(value: InlineQueryResult) -> Self {
        value.0
    }
}

/// Input text content for inline query article results.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct InputTextMessageContent {
    pub message_text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parse_mode: Option<ParseMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entities: Option<Vec<MessageEntity>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub link_preview_options: Option<LinkPreviewOptions>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_web_page_preview: Option<bool>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl InputTextMessageContent {
    pub fn new(message_text: impl Into<String>) -> Self {
        Self {
            message_text: message_text.into(),
            parse_mode: None,
            entities: None,
            link_preview_options: None,
            disable_web_page_preview: None,
            extra: BTreeMap::new(),
        }
    }

    pub fn validate(&self) -> Result<()> {
        validate_required_display_text("input text message content text", &self.message_text)?;
        validate_optional_text_formatting(
            "input text message content text",
            Some(&self.message_text),
            self.parse_mode,
            self.entities.as_deref(),
        )?;
        if self.disable_web_page_preview.is_some() && self.link_preview_options.is_some() {
            return Err(invalid_request(
                "input text message content cannot combine disable_web_page_preview with link_preview_options",
            ));
        }
        if let Some(link_preview_options) = self.link_preview_options.as_ref() {
            link_preview_options.validate()?;
        }

        Ok(())
    }
}

impl Serialize for InputTextMessageContent {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = [
            "message_text",
            "parse_mode",
            "entities",
            "link_preview_options",
            "disable_web_page_preview",
        ];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let optional_len = usize::from(self.parse_mode.is_some())
            + usize::from(self.entities.is_some())
            + usize::from(self.link_preview_options.is_some())
            + usize::from(self.disable_web_page_preview.is_some());
        let mut object = serializer.serialize_map(Some(extra_len + optional_len + 1))?;
        object.serialize_entry("message_text", &self.message_text)?;
        if let Some(parse_mode) = self.parse_mode {
            object.serialize_entry("parse_mode", &parse_mode)?;
        }
        if let Some(entities) = self.entities.as_ref() {
            object.serialize_entry("entities", entities)?;
        }
        if let Some(link_preview_options) = self.link_preview_options.as_ref() {
            object.serialize_entry("link_preview_options", link_preview_options)?;
        }
        if let Some(disable_web_page_preview) = self.disable_web_page_preview {
            object.serialize_entry("disable_web_page_preview", &disable_web_page_preview)?;
        }
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
}

/// Typed inline query article result.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum InlineQueryResultArticleKind {
    #[default]
    #[serde(rename = "article")]
    Article,
}

/// Typed inline query article result.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct InlineQueryResultArticle {
    #[serde(rename = "type")]
    pub kind: InlineQueryResultArticleKind,
    pub id: String,
    pub title: String,
    pub input_message_content: InputTextMessageContent,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_markup: Option<InlineKeyboardMarkup>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hide_url: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thumbnail_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thumbnail_width: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thumbnail_height: Option<u32>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl InlineQueryResultArticle {
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        message_text: impl Into<String>,
    ) -> Self {
        Self {
            kind: InlineQueryResultArticleKind::Article,
            id: id.into(),
            title: title.into(),
            input_message_content: InputTextMessageContent::new(message_text),
            reply_markup: None,
            url: None,
            hide_url: None,
            description: None,
            thumbnail_url: None,
            thumbnail_width: None,
            thumbnail_height: None,
            extra: BTreeMap::new(),
        }
    }

    pub fn validate(&self) -> Result<()> {
        let value =
            serde_json::to_value(self).map_err(|source| Error::SerializeRequest { source })?;
        validate_inline_query_result_value(&value)?;
        validate_required_visible_text("inline query article title", &self.title)?;
        self.input_message_content.validate()?;
        if let Some(reply_markup) = self.reply_markup.as_ref() {
            reply_markup.validate()?;
        }
        if let Some(url) = self.url.as_deref() {
            validate_url("inline query article url", url)?;
        }
        if let Some(thumbnail_url) = self.thumbnail_url.as_deref() {
            validate_url("inline query article thumbnail_url", thumbnail_url)?;
        }

        Ok(())
    }
}

impl TryFrom<InlineQueryResultArticle> for InlineQueryResult {
    type Error = Error;

    fn try_from(value: InlineQueryResultArticle) -> std::result::Result<Self, Self::Error> {
        value.validate()?;
        Self::try_from_typed(value)
    }
}

impl Serialize for InlineQueryResultArticle {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = [
            "type",
            "id",
            "title",
            "input_message_content",
            "reply_markup",
            "url",
            "hide_url",
            "description",
            "thumbnail_url",
            "thumbnail_width",
            "thumbnail_height",
        ];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let optional_len = usize::from(self.reply_markup.is_some())
            + usize::from(self.url.is_some())
            + usize::from(self.hide_url.is_some())
            + usize::from(self.description.is_some())
            + usize::from(self.thumbnail_url.is_some())
            + usize::from(self.thumbnail_width.is_some())
            + usize::from(self.thumbnail_height.is_some());
        let mut object = serializer.serialize_map(Some(extra_len + optional_len + 4))?;
        object.serialize_entry("type", &self.kind)?;
        object.serialize_entry("id", &self.id)?;
        object.serialize_entry("title", &self.title)?;
        object.serialize_entry("input_message_content", &self.input_message_content)?;
        if let Some(reply_markup) = self.reply_markup.as_ref() {
            object.serialize_entry("reply_markup", reply_markup)?;
        }
        if let Some(url) = self.url.as_ref() {
            object.serialize_entry("url", url)?;
        }
        if let Some(hide_url) = self.hide_url {
            object.serialize_entry("hide_url", &hide_url)?;
        }
        if let Some(description) = self.description.as_ref() {
            object.serialize_entry("description", description)?;
        }
        if let Some(thumbnail_url) = self.thumbnail_url.as_ref() {
            object.serialize_entry("thumbnail_url", thumbnail_url)?;
        }
        if let Some(thumbnail_width) = self.thumbnail_width {
            object.serialize_entry("thumbnail_width", &thumbnail_width)?;
        }
        if let Some(thumbnail_height) = self.thumbnail_height {
            object.serialize_entry("thumbnail_height", &thumbnail_height)?;
        }
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
}

/// Task to add to a checklist.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct InputChecklistTask {
    pub id: i64,
    pub text: String,
    pub parse_mode: Option<ParseMode>,
    pub text_entities: Option<Vec<MessageEntity>>,
    pub extra: BTreeMap<String, Value>,
}

impl InputChecklistTask {
    pub fn new(id: i64, text: impl Into<String>) -> Self {
        Self {
            id,
            text: text.into(),
            parse_mode: None,
            text_entities: None,
            extra: BTreeMap::new(),
        }
    }

    pub fn parse_mode(mut self, parse_mode: ParseMode) -> Self {
        self.parse_mode = Some(parse_mode);
        self
    }

    pub fn text_entities(mut self, entities: Vec<MessageEntity>) -> Self {
        self.text_entities = Some(entities);
        self
    }

    pub fn validate(&self) -> Result<()> {
        if self.id <= 0 {
            return Err(invalid_request("input_checklist_task.id must be positive"));
        }
        validate_display_text_length_range("input_checklist_task.text", &self.text, 1, 100)?;
        validate_rich_text_formatting(
            "input_checklist_task.text",
            &self.text,
            self.parse_mode,
            self.text_entities.as_deref(),
        )
    }
}

impl<'de> Deserialize<'de> for InputChecklistTask {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawInputChecklistTask {
            id: i64,
            text: String,
            #[serde(default)]
            parse_mode: Option<ParseMode>,
            #[serde(default)]
            text_entities: Option<Vec<MessageEntity>>,
            #[serde(flatten)]
            extra: BTreeMap<String, Value>,
        }

        let raw = RawInputChecklistTask::deserialize(deserializer)?;
        let task = Self {
            id: raw.id,
            text: raw.text,
            parse_mode: raw.parse_mode,
            text_entities: raw.text_entities,
            extra: raw.extra,
        };
        task.validate().map_err(serde::de::Error::custom)?;
        Ok(task)
    }
}

impl Serialize for InputChecklistTask {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = ["id", "text", "parse_mode", "text_entities"];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let optional_len =
            usize::from(self.parse_mode.is_some()) + usize::from(self.text_entities.is_some());
        let mut object = serializer.serialize_map(Some(extra_len + optional_len + 2))?;
        object.serialize_entry("id", &self.id)?;
        object.serialize_entry("text", &self.text)?;
        if let Some(parse_mode) = self.parse_mode {
            object.serialize_entry("parse_mode", &parse_mode)?;
        }
        if let Some(text_entities) = self.text_entities.as_ref() {
            object.serialize_entry("text_entities", text_entities)?;
        }
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
}

/// Checklist payload to send or edit.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct InputChecklist {
    pub title: String,
    pub parse_mode: Option<ParseMode>,
    pub title_entities: Option<Vec<MessageEntity>>,
    pub tasks: Vec<InputChecklistTask>,
    pub others_can_add_tasks: Option<bool>,
    pub others_can_mark_tasks_as_done: Option<bool>,
    pub extra: BTreeMap<String, Value>,
}

impl InputChecklist {
    pub fn new(
        title: impl Into<String>,
        tasks: impl IntoIterator<Item = InputChecklistTask>,
    ) -> Self {
        Self {
            title: title.into(),
            parse_mode: None,
            title_entities: None,
            tasks: tasks.into_iter().collect(),
            others_can_add_tasks: None,
            others_can_mark_tasks_as_done: None,
            extra: BTreeMap::new(),
        }
    }

    pub fn try_from_value(value: Value) -> Result<Self> {
        let checklist = serde_json::from_value::<Self>(value).map_err(|source| {
            invalid_request(format!("invalid input_checklist payload: {source}"))
        })?;
        checklist.validate()?;
        Ok(checklist)
    }

    pub fn try_from_typed<T>(value: T) -> Result<Self>
    where
        T: Serialize,
    {
        let value =
            serde_json::to_value(value).map_err(|source| Error::SerializeRequest { source })?;
        Self::try_from_value(value)
    }

    pub fn parse_mode(mut self, parse_mode: ParseMode) -> Self {
        self.parse_mode = Some(parse_mode);
        self
    }

    pub fn title_entities(mut self, entities: Vec<MessageEntity>) -> Self {
        self.title_entities = Some(entities);
        self
    }

    pub fn others_can_add_tasks(mut self, value: bool) -> Self {
        self.others_can_add_tasks = Some(value);
        self
    }

    pub fn others_can_mark_tasks_as_done(mut self, value: bool) -> Self {
        self.others_can_mark_tasks_as_done = Some(value);
        self
    }

    pub fn validate(&self) -> Result<()> {
        validate_display_text_length_range("input_checklist.title", &self.title, 1, 255)?;
        validate_rich_text_formatting(
            "input_checklist.title",
            &self.title,
            self.parse_mode,
            self.title_entities.as_deref(),
        )?;
        if !(1..=30).contains(&self.tasks.len()) {
            return Err(invalid_request(
                "input_checklist.tasks must contain 1-30 tasks",
            ));
        }

        let mut task_ids = BTreeSet::new();
        for task in &self.tasks {
            task.validate()?;
            if !task_ids.insert(task.id) {
                return Err(invalid_request(format!(
                    "input_checklist.tasks contains duplicate id {}",
                    task.id
                )));
            }
        }

        Ok(())
    }
}

impl<'de> Deserialize<'de> for InputChecklist {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawInputChecklist {
            title: String,
            #[serde(default)]
            parse_mode: Option<ParseMode>,
            #[serde(default)]
            title_entities: Option<Vec<MessageEntity>>,
            tasks: Vec<InputChecklistTask>,
            #[serde(default)]
            others_can_add_tasks: Option<bool>,
            #[serde(default)]
            others_can_mark_tasks_as_done: Option<bool>,
            #[serde(flatten)]
            extra: BTreeMap<String, Value>,
        }

        let raw = RawInputChecklist::deserialize(deserializer)?;
        let checklist = Self {
            title: raw.title,
            parse_mode: raw.parse_mode,
            title_entities: raw.title_entities,
            tasks: raw.tasks,
            others_can_add_tasks: raw.others_can_add_tasks,
            others_can_mark_tasks_as_done: raw.others_can_mark_tasks_as_done,
            extra: raw.extra,
        };
        checklist.validate().map_err(serde::de::Error::custom)?;
        Ok(checklist)
    }
}

impl TryFrom<Value> for InputChecklist {
    type Error = Error;

    fn try_from(value: Value) -> std::result::Result<Self, Self::Error> {
        Self::try_from_value(value)
    }
}

impl Serialize for InputChecklist {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = [
            "title",
            "parse_mode",
            "title_entities",
            "tasks",
            "others_can_add_tasks",
            "others_can_mark_tasks_as_done",
        ];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let optional_len = usize::from(self.parse_mode.is_some())
            + usize::from(self.title_entities.is_some())
            + usize::from(self.others_can_add_tasks.is_some())
            + usize::from(self.others_can_mark_tasks_as_done.is_some());
        let mut object = serializer.serialize_map(Some(extra_len + optional_len + 2))?;
        object.serialize_entry("title", &self.title)?;
        if let Some(parse_mode) = self.parse_mode {
            object.serialize_entry("parse_mode", &parse_mode)?;
        }
        if let Some(title_entities) = self.title_entities.as_ref() {
            object.serialize_entry("title_entities", title_entities)?;
        }
        object.serialize_entry("tasks", &self.tasks)?;
        if let Some(others_can_add_tasks) = self.others_can_add_tasks {
            object.serialize_entry("others_can_add_tasks", &others_can_add_tasks)?;
        }
        if let Some(others_can_mark_tasks_as_done) = self.others_can_mark_tasks_as_done {
            object.serialize_entry(
                "others_can_mark_tasks_as_done",
                &others_can_mark_tasks_as_done,
            )?;
        }
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
}

json_payload_wrapper!(
    /// Generic suggested-post payload.
    SuggestedPostParameters,
    "suggested_post_parameters",
    validate_suggested_post_parameters_payload
);

json_payload_wrapper!(
    /// Generic accepted-gift-types payload.
    AcceptedGiftTypes,
    "accepted_gift_types",
    validate_accepted_gift_types_payload
);

/// Static profile photo input payload.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[non_exhaustive]
pub struct InputProfilePhotoStatic {
    pub photo: String,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl InputProfilePhotoStatic {
    pub fn new(photo: impl Into<String>) -> Self {
        Self {
            photo: photo.into(),
            extra: BTreeMap::new(),
        }
    }

    pub fn validate(&self) -> Result<()> {
        validate_required_visible_text("input_profile_photo.photo", &self.photo)
    }
}

impl Serialize for InputProfilePhotoStatic {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = ["photo"];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let mut object = serializer.serialize_map(Some(extra_len + 1))?;
        object.serialize_entry("photo", &self.photo)?;
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
}

/// Animated profile photo input payload.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[non_exhaustive]
pub struct InputProfilePhotoAnimated {
    pub animation: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub main_frame_timestamp: Option<f64>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl InputProfilePhotoAnimated {
    pub fn new(animation: impl Into<String>) -> Self {
        Self {
            animation: animation.into(),
            main_frame_timestamp: None,
            extra: BTreeMap::new(),
        }
    }

    pub fn with_main_frame_timestamp(mut self, main_frame_timestamp: f64) -> Self {
        self.main_frame_timestamp = Some(main_frame_timestamp);
        self
    }

    pub fn validate(&self) -> Result<()> {
        validate_required_visible_text("input_profile_photo.animation", &self.animation)?;
        if let Some(main_frame_timestamp) = self.main_frame_timestamp {
            validate_float_range(
                "input_profile_photo.main_frame_timestamp",
                main_frame_timestamp,
                0.0,
                f64::MAX,
            )?;
        }

        Ok(())
    }
}

impl Serialize for InputProfilePhotoAnimated {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = ["animation", "main_frame_timestamp"];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let optional_len = usize::from(self.main_frame_timestamp.is_some());
        let mut object = serializer.serialize_map(Some(extra_len + optional_len + 1))?;
        object.serialize_entry("animation", &self.animation)?;
        if let Some(main_frame_timestamp) = self.main_frame_timestamp {
            object.serialize_entry("main_frame_timestamp", &main_frame_timestamp)?;
        }
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
}

/// Profile photo input payload.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum InputProfilePhoto {
    Static(InputProfilePhotoStatic),
    Animated(InputProfilePhotoAnimated),
    Unknown(Value),
}

impl InputProfilePhoto {
    pub fn static_photo(photo: impl Into<String>) -> Self {
        Self::Static(InputProfilePhotoStatic::new(photo))
    }

    pub fn animated(animation: impl Into<String>) -> Self {
        Self::Animated(InputProfilePhotoAnimated::new(animation))
    }

    pub fn new(value: Value) -> Result<Self> {
        Self::try_from_value(value)
    }

    pub fn try_from_value(value: Value) -> Result<Self> {
        let photo = serde_json::from_value::<Self>(value).map_err(|source| {
            invalid_request(format!("invalid input_profile_photo payload: {source}"))
        })?;
        photo.validate()?;
        Ok(photo)
    }

    pub fn try_from_typed<T>(value: T) -> Result<Self>
    where
        T: Serialize,
    {
        let value =
            serde_json::to_value(value).map_err(|source| Error::SerializeRequest { source })?;
        Self::try_from_value(value)
    }

    pub fn kind(&self) -> Option<&str> {
        match self {
            Self::Static(_) => Some("static"),
            Self::Animated(_) => Some("animated"),
            Self::Unknown(value) => tagged_kind(value),
        }
    }

    pub fn validate(&self) -> Result<()> {
        match self {
            Self::Static(value) => value.validate(),
            Self::Animated(value) => value.validate(),
            Self::Unknown(value) => validate_typed_object_payload("input_profile_photo", value),
        }
    }
}

impl From<InputProfilePhotoStatic> for InputProfilePhoto {
    fn from(value: InputProfilePhotoStatic) -> Self {
        Self::Static(value)
    }
}

impl From<InputProfilePhotoAnimated> for InputProfilePhoto {
    fn from(value: InputProfilePhotoAnimated) -> Self {
        Self::Animated(value)
    }
}

impl<'de> Deserialize<'de> for InputProfilePhoto {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        Self::try_from(value).map_err(serde::de::Error::custom)
    }
}

impl TryFrom<Value> for InputProfilePhoto {
    type Error = Error;

    fn try_from(value: Value) -> std::result::Result<Self, Self::Error> {
        match tagged_kind(&value) {
            Some("static") => deserialize_input_profile_photo_known(value, Self::Static),
            Some("animated") => deserialize_input_profile_photo_known(value, Self::Animated),
            Some(_) => {
                validate_typed_object_payload("input_profile_photo", &value)?;
                Ok(Self::Unknown(value))
            }
            None => Err(invalid_request(
                "input_profile_photo requires a string `type` field",
            )),
        }
    }
}

fn deserialize_input_profile_photo_known<T>(
    value: Value,
    constructor: impl FnOnce(T) -> InputProfilePhoto,
) -> Result<InputProfilePhoto>
where
    T: DeserializeOwned,
{
    let payload = serde_json::from_value::<T>(strip_type(value)).map_err(|source| {
        invalid_request(format!("invalid input_profile_photo payload: {source}"))
    })?;
    let photo = constructor(payload);
    photo.validate()?;
    Ok(photo)
}

impl Serialize for InputProfilePhoto {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Static(value) => serialize_typed_payload(serializer, "static", value),
            Self::Animated(value) => serialize_typed_payload(serializer, "animated", value),
            Self::Unknown(value) => value.serialize(serializer),
        }
    }
}

/// Prepared inline message that can be sent by a Mini App user.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct PreparedInlineMessage {
    pub id: String,
    pub expiration_date: i64,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Prepared keyboard button that can be used by a Mini App user.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct PreparedKeyboardButton {
    pub id: String,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Menu button returned by Telegram or sent through `setChatMenuButton`.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum MenuButton {
    Commands(MenuButtonCommands),
    Default(MenuButtonDefault),
    WebApp(MenuButtonWebApp),
    Unknown(Value),
}

/// `commands` menu button payload.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct MenuButtonCommands {
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// `default` menu button payload.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct MenuButtonDefault {
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Known menu button variants for constructing `MenuButton` values.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[non_exhaustive]
pub enum MenuButtonKind {
    Commands,
    Default,
    WebApp(MenuButtonWebApp),
}

impl MenuButton {
    pub fn new(value: Value) -> Self {
        Self::from(value)
    }

    pub fn commands() -> Self {
        Self::Commands(MenuButtonCommands::default())
    }

    pub fn default_button() -> Self {
        Self::Default(MenuButtonDefault::default())
    }

    pub fn web_app(text: impl Into<String>, web_app: impl Into<WebAppInfo>) -> Self {
        Self::WebApp(MenuButtonWebApp::new(text, web_app))
    }

    pub fn kind(&self) -> Option<&str> {
        match self {
            Self::Commands(_) => Some("commands"),
            Self::Default(_) => Some("default"),
            Self::WebApp(_) => Some("web_app"),
            Self::Unknown(value) => tagged_kind(value),
        }
    }

    pub fn is_commands(&self) -> bool {
        matches!(self, Self::Commands(_))
    }

    pub fn as_commands(&self) -> Option<&MenuButtonCommands> {
        match self {
            Self::Commands(value) => Some(value),
            Self::Default(_) | Self::WebApp(_) | Self::Unknown(_) => None,
        }
    }

    pub fn into_commands(self) -> Option<MenuButtonCommands> {
        match self {
            Self::Commands(value) => Some(value),
            Self::Default(_) | Self::WebApp(_) | Self::Unknown(_) => None,
        }
    }

    pub fn is_default(&self) -> bool {
        matches!(self, Self::Default(_))
    }

    pub fn as_default(&self) -> Option<&MenuButtonDefault> {
        match self {
            Self::Default(value) => Some(value),
            Self::Commands(_) | Self::WebApp(_) | Self::Unknown(_) => None,
        }
    }

    pub fn into_default(self) -> Option<MenuButtonDefault> {
        match self {
            Self::Default(value) => Some(value),
            Self::Commands(_) | Self::WebApp(_) | Self::Unknown(_) => None,
        }
    }

    pub fn is_web_app(&self) -> bool {
        matches!(self, Self::WebApp(_))
    }

    pub fn as_web_app(&self) -> Option<&MenuButtonWebApp> {
        match self {
            Self::WebApp(value) => Some(value),
            Self::Commands(_) | Self::Default(_) | Self::Unknown(_) => None,
        }
    }

    pub fn into_web_app(self) -> Option<MenuButtonWebApp> {
        match self {
            Self::WebApp(value) => Some(value),
            Self::Commands(_) | Self::Default(_) | Self::Unknown(_) => None,
        }
    }

    pub fn as_unknown_value(&self) -> Option<&Value> {
        match self {
            Self::Unknown(value) => Some(value),
            Self::Commands(_) | Self::Default(_) | Self::WebApp(_) => None,
        }
    }

    pub fn into_unknown_value(self) -> Option<Value> {
        match self {
            Self::Unknown(value) => Some(value),
            Self::Commands(_) | Self::Default(_) | Self::WebApp(_) => None,
        }
    }

    pub fn validate(&self) -> Result<()> {
        match self {
            Self::Commands(_) | Self::Default(_) => Ok(()),
            Self::WebApp(value) => value.validate(),
            Self::Unknown(value) => validate_unknown_menu_button(value),
        }
    }
}

fn deserialize_menu_button_known<T>(
    value: Value,
    constructor: impl FnOnce(T) -> MenuButton,
) -> MenuButton
where
    T: DeserializeOwned,
{
    match serde_json::from_value::<T>(strip_type(value.clone())) {
        Ok(payload) => constructor(payload),
        Err(_error) => MenuButton::Unknown(value),
    }
}

fn validate_unknown_menu_button(value: &Value) -> Result<()> {
    match tagged_kind(value) {
        Some("web_app") => {
            match serde_json::from_value::<MenuButtonWebApp>(strip_type(value.clone())) {
                Ok(payload) => payload.validate(),
                Err(error) => Err(invalid_request(format!(
                    "invalid menu_button web_app payload: {error}"
                ))),
            }
        }
        Some("commands" | "default") => Ok(()),
        Some(_) | None => validate_typed_object_payload("menu_button", value),
    }
}

fn menu_button_object(kind: &str, extra: BTreeMap<String, Value>) -> Value {
    let mut object = serde_json::Map::new();
    for (key, value) in extra {
        object.insert(key, value);
    }
    object.insert("type".to_owned(), Value::String(kind.to_owned()));
    Value::Object(object)
}

fn web_app_info_object(value: WebAppInfo) -> Value {
    let mut object = serde_json::Map::new();
    for (key, extra_value) in value.extra {
        object.insert(key, extra_value);
    }
    object.insert("url".to_owned(), Value::String(value.url));
    Value::Object(object)
}

impl Serialize for MenuButton {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Commands(value) => {
                serialize_menu_button_object(serializer, "commands", &value.extra)
            }
            Self::Default(value) => {
                serialize_menu_button_object(serializer, "default", &value.extra)
            }
            Self::WebApp(value) => serialize_menu_button_web_app(serializer, value),
            Self::Unknown(value) => value.serialize(serializer),
        }
    }
}

fn serialize_menu_button_object<S>(
    serializer: S,
    kind: &str,
    extra: &BTreeMap<String, Value>,
) -> std::result::Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let reserved = ["type"];
    let extra_len = extra_field_len(extra, &reserved);
    let mut object = serializer.serialize_map(Some(extra_len + 1))?;
    object.serialize_entry("type", kind)?;
    serialize_extra_fields(&mut object, extra, &reserved)?;
    object.end()
}

fn serialize_menu_button_web_app<S>(
    serializer: S,
    value: &MenuButtonWebApp,
) -> std::result::Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let reserved = ["type", "text", "web_app"];
    let extra_len = extra_field_len(&value.extra, &reserved);
    let mut object = serializer.serialize_map(Some(extra_len + 3))?;
    object.serialize_entry("type", "web_app")?;
    object.serialize_entry("text", &value.text)?;
    object.serialize_entry("web_app", &value.web_app)?;
    serialize_extra_fields(&mut object, &value.extra, &reserved)?;
    object.end()
}

impl<'de> Deserialize<'de> for MenuButton {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        Ok(Self::from(value))
    }
}

impl Default for MenuButton {
    fn default() -> Self {
        Self::default_button()
    }
}

impl From<Value> for MenuButton {
    fn from(value: Value) -> Self {
        match tagged_kind(&value) {
            Some("commands") => deserialize_menu_button_known(value, Self::Commands),
            Some("default") => deserialize_menu_button_known(value, Self::Default),
            Some("web_app") => deserialize_menu_button_known(value, Self::WebApp),
            Some(_) | None => Self::Unknown(value),
        }
    }
}

impl From<MenuButtonKind> for MenuButton {
    fn from(value: MenuButtonKind) -> Self {
        match value {
            MenuButtonKind::Commands => Self::commands(),
            MenuButtonKind::Default => Self::default_button(),
            MenuButtonKind::WebApp(value) => Self::WebApp(value),
        }
    }
}

impl From<MenuButton> for Value {
    fn from(value: MenuButton) -> Self {
        match value {
            MenuButton::Commands(value) => menu_button_object("commands", value.extra),
            MenuButton::Default(value) => menu_button_object("default", value.extra),
            MenuButton::WebApp(value) => {
                let mut object = serde_json::Map::new();
                for (key, extra_value) in value.extra {
                    object.insert(key, extra_value);
                }
                object.insert("type".to_owned(), Value::String("web_app".to_owned()));
                object.insert("text".to_owned(), Value::String(value.text));
                object.insert("web_app".to_owned(), web_app_info_object(value.web_app));
                Value::Object(object)
            }
            MenuButton::Unknown(value) => value,
        }
    }
}

/// Mini App Web App descriptor.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[non_exhaustive]
pub struct WebAppInfo {
    pub url: String,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl WebAppInfo {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            extra: BTreeMap::new(),
        }
    }

    pub fn validate(&self) -> Result<()> {
        validate_https_url("web_app.url", &self.url)
    }
}

impl From<String> for WebAppInfo {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for WebAppInfo {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl Serialize for WebAppInfo {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = ["url"];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let mut object = serializer.serialize_map(Some(extra_len + 1))?;
        object.serialize_entry("url", &self.url)?;
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
}

/// Button shown above inline query results.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[non_exhaustive]
pub struct InlineQueryResultsButton {
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub web_app: Option<WebAppInfo>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_parameter: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl InlineQueryResultsButton {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            web_app: None,
            start_parameter: None,
            extra: BTreeMap::new(),
        }
    }

    pub fn web_app(text: impl Into<String>, web_app: impl Into<WebAppInfo>) -> Self {
        Self::new(text).with_web_app(web_app)
    }

    pub fn start_parameter(text: impl Into<String>, start_parameter: impl Into<String>) -> Self {
        Self::new(text).with_start_parameter(start_parameter)
    }

    pub fn with_web_app(mut self, web_app: impl Into<WebAppInfo>) -> Self {
        self.web_app = Some(web_app.into());
        self.start_parameter = None;
        self
    }

    pub fn with_start_parameter(mut self, start_parameter: impl Into<String>) -> Self {
        self.start_parameter = Some(start_parameter.into());
        self.web_app = None;
        self
    }

    pub fn validate(&self) -> Result<()> {
        validate_required_visible_text("inline query results button text", &self.text)?;
        if let Some(web_app) = self.web_app.as_ref() {
            web_app.validate()?;
        }
        if let Some(start_parameter) = self.start_parameter.as_deref() {
            validate_required_visible_text(
                "inline query results button start_parameter",
                start_parameter,
            )?;
        }
        if self.web_app.is_some() && self.start_parameter.is_some() {
            return Err(invalid_request(
                "inline query results button cannot set both web_app and start_parameter",
            ));
        }
        let reserved = ["text", "web_app", "start_parameter"];
        if self.web_app.is_none()
            && self.start_parameter.is_none()
            && extra_field_len(&self.extra, &reserved) == 0
        {
            return Err(invalid_request(
                "inline query results button must define an action",
            ));
        }

        Ok(())
    }
}

impl Serialize for InlineQueryResultsButton {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = ["text", "web_app", "start_parameter"];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let optional_len =
            usize::from(self.web_app.is_some()) + usize::from(self.start_parameter.is_some());
        let mut object = serializer.serialize_map(Some(extra_len + optional_len + 1))?;
        object.serialize_entry("text", &self.text)?;
        if let Some(web_app) = self.web_app.as_ref() {
            object.serialize_entry("web_app", web_app)?;
        }
        if let Some(start_parameter) = self.start_parameter.as_ref() {
            object.serialize_entry("start_parameter", start_parameter)?;
        }
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
}

/// Menu button launching a Mini App.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[non_exhaustive]
pub struct MenuButtonWebApp {
    pub text: String,
    pub web_app: WebAppInfo,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl MenuButtonWebApp {
    pub fn new(text: impl Into<String>, web_app: impl Into<WebAppInfo>) -> Self {
        Self {
            text: text.into(),
            web_app: web_app.into(),
            extra: BTreeMap::new(),
        }
    }

    pub fn validate(&self) -> Result<()> {
        validate_required_visible_text("menu button text", &self.text)?;
        self.web_app.validate()
    }
}

impl Serialize for MenuButtonWebApp {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = ["text", "web_app"];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let mut object = serializer.serialize_map(Some(extra_len + 2))?;
        object.serialize_entry("text", &self.text)?;
        object.serialize_entry("web_app", &self.web_app)?;
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
}

impl From<MenuButtonWebApp> for MenuButton {
    fn from(value: MenuButtonWebApp) -> Self {
        Self::WebApp(value)
    }
}

/// Data sent from Mini App via `Telegram.WebApp.sendData`.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[non_exhaustive]
pub struct WebAppData {
    pub data: String,
    pub button_text: String,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl WebAppData {
    pub fn new(data: impl Into<String>, button_text: impl Into<String>) -> Self {
        Self {
            data: data.into(),
            button_text: button_text.into(),
            extra: BTreeMap::new(),
        }
    }
}

impl crate::types::advanced::AdvancedSetChatMenuButtonRequest {
    pub fn chat_id(mut self, chat_id: impl Into<NumericChatId>) -> Self {
        self.chat_id = Some(chat_id.into());
        self
    }

    pub fn menu_button(mut self, menu_button: impl Into<MenuButton>) -> Self {
        self.menu_button = Some(menu_button.into());
        self
    }

    pub fn menu_button_default(mut self) -> Self {
        self.menu_button = Some(MenuButton::default_button());
        self
    }

    pub fn menu_button_commands(mut self) -> Self {
        self.menu_button = Some(MenuButton::commands());
        self
    }

    pub fn menu_button_web_app(
        mut self,
        text: impl Into<String>,
        web_app: impl Into<WebAppInfo>,
    ) -> Self {
        self.menu_button = Some(MenuButton::web_app(text, web_app));
        self
    }
}

json_payload_wrapper!(
    /// Generic passport element error payload.
    PassportElementError,
    "passport_element_error",
    validate_source_object_payload
);

/// Inline keyboard button.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct InlineKeyboardButton {
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub web_app: Option<WebAppInfo>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl InlineKeyboardButton {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            web_app: None,
            extra: BTreeMap::new(),
        }
    }

    pub fn callback(text: impl Into<String>, data: impl Into<String>) -> Result<Self> {
        Self::new(text).with_callback_data(data)
    }

    pub fn pay(text: impl Into<String>) -> Self {
        Self::new(text).with_pay()
    }

    pub fn typed_callback<T>(text: impl Into<String>, payload: &T) -> Result<Self>
    where
        T: CallbackPayload,
    {
        Self::new(text).with_typed_callback(payload)
    }

    pub fn typed_callback_with_codec<T, C>(text: impl Into<String>, payload: &T) -> Result<Self>
    where
        C: CallbackCodec<T>,
    {
        Self::new(text).with_typed_callback_with_codec::<T, C>(payload)
    }

    pub fn compact_callback<T>(text: impl Into<String>, payload: &T) -> Result<Self>
    where
        T: CompactCallbackPayload,
    {
        Self::typed_callback_with_codec::<T, CompactCallbackCodec>(text, payload)
    }

    pub fn web_app(mut self, web_app: impl Into<WebAppInfo>) -> Self {
        self.web_app = Some(web_app.into());
        self
    }

    pub fn with_callback_data(mut self, data: impl Into<String>) -> Result<Self> {
        self.extra.insert(
            "callback_data".to_owned(),
            Value::String(validate_callback_data(data)?),
        );
        Ok(self)
    }

    pub fn with_pay(mut self) -> Self {
        self.extra.insert("pay".to_owned(), Value::Bool(true));
        self
    }

    pub fn with_typed_callback<T>(self, payload: &T) -> Result<Self>
    where
        T: CallbackPayload,
    {
        self.with_callback_data(payload.encode_callback_data()?)
    }

    pub fn with_typed_callback_with_codec<T, C>(self, payload: &T) -> Result<Self>
    where
        C: CallbackCodec<T>,
    {
        self.with_callback_data(C::encode_callback_data(payload)?)
    }

    pub fn with_compact_callback<T>(self, payload: &T) -> Result<Self>
    where
        T: CompactCallbackPayload,
    {
        self.with_typed_callback_with_codec::<T, CompactCallbackCodec>(payload)
    }

    pub fn callback_data(&self) -> Option<&str> {
        self.extra.get("callback_data").and_then(Value::as_str)
    }

    pub fn is_pay_button(&self) -> bool {
        matches!(self.extra.get("pay"), Some(Value::Bool(true)))
    }

    pub fn decode_callback<T>(&self) -> Result<Option<T>>
    where
        T: CallbackPayload,
    {
        self.callback_data()
            .map(T::decode_callback_data)
            .transpose()
    }

    pub fn decode_callback_with_codec<T, C>(&self) -> Result<Option<T>>
    where
        C: CallbackCodec<T>,
    {
        self.callback_data()
            .map(C::decode_callback_data)
            .transpose()
    }

    pub fn decode_compact_callback<T>(&self) -> Result<Option<T>>
    where
        T: CompactCallbackPayload,
    {
        self.decode_callback_with_codec::<T, CompactCallbackCodec>()
    }

    pub fn validate(&self) -> Result<()> {
        validate_required_visible_text("inline keyboard button text", &self.text)?;

        let mut known_actions = usize::from(self.web_app.is_some());
        if let Some(web_app) = self.web_app.as_ref() {
            web_app.validate()?;
        }

        for (key, value) in &self.extra {
            match key.as_str() {
                "callback_data" => {
                    let data = value_as_str("callback_data", value)?;
                    validate_callback_data(data)?;
                    known_actions += 1;
                }
                "url" => {
                    validate_url("url", value_as_str("url", value)?)?;
                    known_actions += 1;
                }
                "login_url" => {
                    validate_login_url_payload(value)?;
                    known_actions += 1;
                }
                "switch_inline_query" | "switch_inline_query_current_chat" => {
                    validate_string_without_control_chars(key, value_as_str(key, value)?)?;
                    known_actions += 1;
                }
                "switch_inline_query_chosen_chat" => {
                    validate_switch_inline_query_chosen_chat(value)?;
                    known_actions += 1;
                }
                "callback_game" => {
                    value_as_object("callback_game", value)?;
                    known_actions += 1;
                }
                "pay" => {
                    validate_true_value("pay", value)?;
                    known_actions += 1;
                }
                "copy_text" => {
                    validate_copy_text_button(value)?;
                    known_actions += 1;
                }
                _ => {}
            }
        }

        if known_actions > 1 {
            return Err(invalid_request(
                "inline keyboard button must define exactly one known action",
            ));
        }
        let reserved = ["text", "web_app"];
        if known_actions == 0 && extra_field_len(&self.extra, &reserved) == 0 {
            return Err(invalid_request(
                "inline keyboard button must define an action",
            ));
        }

        Ok(())
    }
}

impl Serialize for InlineKeyboardButton {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = ["text", "web_app"];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let optional_len = usize::from(self.web_app.is_some());
        let mut object = serializer.serialize_map(Some(extra_len + optional_len + 1))?;
        object.serialize_entry("text", &self.text)?;
        if let Some(web_app) = self.web_app.as_ref() {
            object.serialize_entry("web_app", web_app)?;
        }
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
}

/// Inline keyboard markup.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct InlineKeyboardMarkup {
    pub inline_keyboard: Vec<Vec<InlineKeyboardButton>>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl InlineKeyboardMarkup {
    pub fn new(inline_keyboard: Vec<Vec<InlineKeyboardButton>>) -> Self {
        Self {
            inline_keyboard,
            extra: BTreeMap::new(),
        }
    }

    pub fn single_row(row: Vec<InlineKeyboardButton>) -> Self {
        Self::new(vec![row])
    }

    pub fn push_row(mut self, row: Vec<InlineKeyboardButton>) -> Self {
        self.inline_keyboard.push(row);
        self
    }

    pub fn validate(&self) -> Result<()> {
        if self.inline_keyboard.is_empty() {
            return Err(invalid_request("inline_keyboard cannot be empty"));
        }
        for row in &self.inline_keyboard {
            if row.is_empty() {
                return Err(invalid_request("inline_keyboard rows cannot be empty"));
            }
            for button in row {
                button.validate()?;
            }
        }

        Ok(())
    }
}

impl Serialize for InlineKeyboardMarkup {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = ["inline_keyboard"];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let mut object = serializer.serialize_map(Some(extra_len + 1))?;
        object.serialize_entry("inline_keyboard", &self.inline_keyboard)?;
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
}

/// Reply keyboard button.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct KeyboardButton {
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub web_app: Option<WebAppInfo>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl KeyboardButton {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            web_app: None,
            extra: BTreeMap::new(),
        }
    }

    pub fn web_app(mut self, web_app: impl Into<WebAppInfo>) -> Self {
        self.web_app = Some(web_app.into());
        self
    }

    pub fn validate(&self) -> Result<()> {
        validate_required_visible_text("keyboard button text", &self.text)?;
        let mut known_actions = usize::from(self.web_app.is_some());
        if let Some(web_app) = self.web_app.as_ref() {
            web_app.validate()?;
        }
        for (key, value) in &self.extra {
            match key.as_str() {
                "request_contact" | "request_location" => {
                    validate_true_value(key, value)?;
                    known_actions += 1;
                }
                "request_users" => {
                    validate_keyboard_button_request_users(value)?;
                    known_actions += 1;
                }
                "request_chat" => {
                    validate_keyboard_button_request_chat(value)?;
                    known_actions += 1;
                }
                "request_poll" => {
                    validate_keyboard_button_request_poll(value)?;
                    known_actions += 1;
                }
                _ => {}
            }
        }
        if known_actions > 1 {
            return Err(invalid_request(
                "keyboard button must define at most one known optional action",
            ));
        }

        Ok(())
    }
}

impl Serialize for KeyboardButton {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = ["text", "web_app"];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let optional_len = usize::from(self.web_app.is_some());
        let mut object = serializer.serialize_map(Some(extra_len + optional_len + 1))?;
        object.serialize_entry("text", &self.text)?;
        if let Some(web_app) = self.web_app.as_ref() {
            object.serialize_entry("web_app", web_app)?;
        }
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
}

/// Reply keyboard markup.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct ReplyKeyboardMarkup {
    pub keyboard: Vec<Vec<KeyboardButton>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_persistent: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resize_keyboard: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub one_time_keyboard: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_field_placeholder: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selective: Option<bool>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl ReplyKeyboardMarkup {
    pub fn new(keyboard: Vec<Vec<KeyboardButton>>) -> Self {
        Self {
            keyboard,
            is_persistent: None,
            resize_keyboard: None,
            one_time_keyboard: None,
            input_field_placeholder: None,
            selective: None,
            extra: BTreeMap::new(),
        }
    }

    pub fn validate(&self) -> Result<()> {
        if self.keyboard.is_empty() {
            return Err(invalid_request("keyboard cannot be empty"));
        }
        for row in &self.keyboard {
            if row.is_empty() {
                return Err(invalid_request("keyboard rows cannot be empty"));
            }
            for button in row {
                button.validate()?;
            }
        }
        if let Some(placeholder) = self.input_field_placeholder.as_deref() {
            validate_required_visible_text("input_field_placeholder", placeholder)?;
        }

        Ok(())
    }
}

impl Serialize for ReplyKeyboardMarkup {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = [
            "keyboard",
            "is_persistent",
            "resize_keyboard",
            "one_time_keyboard",
            "input_field_placeholder",
            "selective",
        ];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let optional_len = usize::from(self.is_persistent.is_some())
            + usize::from(self.resize_keyboard.is_some())
            + usize::from(self.one_time_keyboard.is_some())
            + usize::from(self.input_field_placeholder.is_some())
            + usize::from(self.selective.is_some());
        let mut object = serializer.serialize_map(Some(extra_len + optional_len + 1))?;
        object.serialize_entry("keyboard", &self.keyboard)?;
        if let Some(is_persistent) = self.is_persistent {
            object.serialize_entry("is_persistent", &is_persistent)?;
        }
        if let Some(resize_keyboard) = self.resize_keyboard {
            object.serialize_entry("resize_keyboard", &resize_keyboard)?;
        }
        if let Some(one_time_keyboard) = self.one_time_keyboard {
            object.serialize_entry("one_time_keyboard", &one_time_keyboard)?;
        }
        if let Some(input_field_placeholder) = self.input_field_placeholder.as_ref() {
            object.serialize_entry("input_field_placeholder", input_field_placeholder)?;
        }
        if let Some(selective) = self.selective {
            object.serialize_entry("selective", &selective)?;
        }
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
}

/// Remove reply keyboard marker.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct ReplyKeyboardRemove {
    pub remove_keyboard: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selective: Option<bool>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl Default for ReplyKeyboardRemove {
    fn default() -> Self {
        Self {
            remove_keyboard: true,
            selective: None,
            extra: BTreeMap::new(),
        }
    }
}

impl ReplyKeyboardRemove {
    pub fn validate(&self) -> Result<()> {
        if !self.remove_keyboard {
            return Err(invalid_request("remove_keyboard must be true"));
        }

        Ok(())
    }
}

impl Serialize for ReplyKeyboardRemove {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = ["remove_keyboard", "selective"];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let optional_len = usize::from(self.selective.is_some());
        let mut object = serializer.serialize_map(Some(extra_len + optional_len + 1))?;
        object.serialize_entry("remove_keyboard", &self.remove_keyboard)?;
        if let Some(selective) = self.selective {
            object.serialize_entry("selective", &selective)?;
        }
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
}

/// Force reply marker.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct ForceReply {
    pub force_reply: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_field_placeholder: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selective: Option<bool>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl Default for ForceReply {
    fn default() -> Self {
        Self {
            force_reply: true,
            input_field_placeholder: None,
            selective: None,
            extra: BTreeMap::new(),
        }
    }
}

impl ForceReply {
    pub fn validate(&self) -> Result<()> {
        if !self.force_reply {
            return Err(invalid_request("force_reply must be true"));
        }
        if let Some(placeholder) = self.input_field_placeholder.as_deref() {
            validate_required_visible_text("input_field_placeholder", placeholder)?;
        }

        Ok(())
    }
}

impl Serialize for ForceReply {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = ["force_reply", "input_field_placeholder", "selective"];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let optional_len = usize::from(self.input_field_placeholder.is_some())
            + usize::from(self.selective.is_some());
        let mut object = serializer.serialize_map(Some(extra_len + optional_len + 1))?;
        object.serialize_entry("force_reply", &self.force_reply)?;
        if let Some(input_field_placeholder) = self.input_field_placeholder.as_ref() {
            object.serialize_entry("input_field_placeholder", input_field_placeholder)?;
        }
        if let Some(selective) = self.selective {
            object.serialize_entry("selective", &selective)?;
        }
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
}

/// Reply markup union accepted by Telegram send/edit methods.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ReplyMarkup {
    InlineKeyboardMarkup(InlineKeyboardMarkup),
    ReplyKeyboardMarkup(ReplyKeyboardMarkup),
    ReplyKeyboardRemove(ReplyKeyboardRemove),
    ForceReply(ForceReply),
}

impl From<InlineKeyboardMarkup> for ReplyMarkup {
    fn from(value: InlineKeyboardMarkup) -> Self {
        Self::InlineKeyboardMarkup(value)
    }
}

impl From<ReplyKeyboardMarkup> for ReplyMarkup {
    fn from(value: ReplyKeyboardMarkup) -> Self {
        Self::ReplyKeyboardMarkup(value)
    }
}

impl From<ReplyKeyboardRemove> for ReplyMarkup {
    fn from(value: ReplyKeyboardRemove) -> Self {
        Self::ReplyKeyboardRemove(value)
    }
}

impl From<ForceReply> for ReplyMarkup {
    fn from(value: ForceReply) -> Self {
        Self::ForceReply(value)
    }
}

impl ReplyMarkup {
    pub fn validate(&self) -> Result<()> {
        match self {
            Self::InlineKeyboardMarkup(value) => value.validate(),
            Self::ReplyKeyboardMarkup(value) => value.validate(),
            Self::ReplyKeyboardRemove(value) => value.validate(),
            Self::ForceReply(value) => value.validate(),
        }
    }
}

/// Reply-to reference parameters.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct ReplyParameters {
    pub message_id: MessageId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chat_id: Option<ChatId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_sending_without_reply: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quote: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quote_parse_mode: Option<ParseMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quote_entities: Option<Vec<MessageEntity>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quote_position: Option<u32>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl ReplyParameters {
    pub fn new(message_id: MessageId) -> Self {
        Self {
            message_id,
            chat_id: None,
            allow_sending_without_reply: None,
            quote: None,
            quote_parse_mode: None,
            quote_entities: None,
            quote_position: None,
            extra: BTreeMap::new(),
        }
    }

    pub fn validate(&self) -> Result<()> {
        self.message_id.validate()?;
        if let Some(chat_id) = self.chat_id.as_ref() {
            chat_id.validate()?;
        }
        if let Some(quote) = self.quote.as_deref() {
            validate_required_display_text("reply quote", quote)?;
        }
        if self.quote.is_none()
            && (self.quote_parse_mode.is_some()
                || self.quote_entities.is_some()
                || self.quote_position.is_some())
        {
            return Err(invalid_request(
                "quote formatting options require reply quote",
            ));
        }
        validate_optional_rich_text_formatting(
            "reply quote",
            self.quote.as_deref(),
            self.quote_parse_mode,
            self.quote_entities.as_deref(),
        )?;

        Ok(())
    }
}

impl Serialize for ReplyParameters {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = [
            "message_id",
            "chat_id",
            "allow_sending_without_reply",
            "quote",
            "quote_parse_mode",
            "quote_entities",
            "quote_position",
        ];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let optional_len = usize::from(self.chat_id.is_some())
            + usize::from(self.allow_sending_without_reply.is_some())
            + usize::from(self.quote.is_some())
            + usize::from(self.quote_parse_mode.is_some())
            + usize::from(self.quote_entities.is_some())
            + usize::from(self.quote_position.is_some());
        let mut object = serializer.serialize_map(Some(extra_len + optional_len + 1))?;
        object.serialize_entry("message_id", &self.message_id)?;
        if let Some(chat_id) = self.chat_id.as_ref() {
            object.serialize_entry("chat_id", chat_id)?;
        }
        if let Some(allow_sending_without_reply) = self.allow_sending_without_reply {
            object.serialize_entry("allow_sending_without_reply", &allow_sending_without_reply)?;
        }
        if let Some(quote) = self.quote.as_ref() {
            object.serialize_entry("quote", quote)?;
        }
        if let Some(quote_parse_mode) = self.quote_parse_mode {
            object.serialize_entry("quote_parse_mode", &quote_parse_mode)?;
        }
        if let Some(quote_entities) = self.quote_entities.as_ref() {
            object.serialize_entry("quote_entities", quote_entities)?;
        }
        if let Some(quote_position) = self.quote_position {
            object.serialize_entry("quote_position", &quote_position)?;
        }
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
}

/// Link preview options for text messages.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct LinkPreviewOptions {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_disabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prefer_small_media: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prefer_large_media: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub show_above_text: Option<bool>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl LinkPreviewOptions {
    pub fn new() -> Self {
        Self {
            is_disabled: None,
            url: None,
            prefer_small_media: None,
            prefer_large_media: None,
            show_above_text: None,
            extra: BTreeMap::new(),
        }
    }

    pub fn disabled() -> Self {
        let mut options = Self::new();
        options.is_disabled = Some(true);
        options
    }

    pub fn validate(&self) -> Result<()> {
        if let Some(url) = self.url.as_deref() {
            validate_url("link_preview_options.url", url)?;
        }
        if self.prefer_small_media == Some(true) && self.prefer_large_media == Some(true) {
            return Err(invalid_request(
                "link_preview_options cannot prefer both small and large media",
            ));
        }
        if self.is_disabled == Some(true)
            && (self.url.is_some()
                || self.prefer_small_media == Some(true)
                || self.prefer_large_media == Some(true)
                || self.show_above_text == Some(true))
        {
            return Err(invalid_request(
                "disabled link preview cannot set preview customization options",
            ));
        }

        Ok(())
    }
}

impl Default for LinkPreviewOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl Serialize for LinkPreviewOptions {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = [
            "is_disabled",
            "url",
            "prefer_small_media",
            "prefer_large_media",
            "show_above_text",
        ];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let optional_len = usize::from(self.is_disabled.is_some())
            + usize::from(self.url.is_some())
            + usize::from(self.prefer_small_media.is_some())
            + usize::from(self.prefer_large_media.is_some())
            + usize::from(self.show_above_text.is_some());
        let mut object = serializer.serialize_map(Some(extra_len + optional_len))?;
        if let Some(is_disabled) = self.is_disabled {
            object.serialize_entry("is_disabled", &is_disabled)?;
        }
        if let Some(url) = self.url.as_ref() {
            object.serialize_entry("url", url)?;
        }
        if let Some(prefer_small_media) = self.prefer_small_media {
            object.serialize_entry("prefer_small_media", &prefer_small_media)?;
        }
        if let Some(prefer_large_media) = self.prefer_large_media {
            object.serialize_entry("prefer_large_media", &prefer_large_media)?;
        }
        if let Some(show_above_text) = self.show_above_text {
            object.serialize_entry("show_above_text", &show_above_text)?;
        }
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_entity(kind: MessageEntityKind, length: u32) -> MessageEntity {
        MessageEntity {
            kind,
            offset: 0,
            length,
            url: None,
            user: None,
            language: None,
            custom_emoji_id: None,
            unix_time: None,
            date_time_format: None,
            extra: BTreeMap::new(),
        }
    }

    #[test]
    fn inline_query_article_kind_is_fixed() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let article = InlineQueryResultArticle::new("article-id", "Title", "hello");
        assert_eq!(article.kind, InlineQueryResultArticleKind::Article);

        let value = serde_json::to_value(&article)?;
        assert_eq!(value["type"], "article");

        let parsed: InlineQueryResultArticle = serde_json::from_value(value)?;
        assert_eq!(parsed.kind, InlineQueryResultArticleKind::Article);

        let mut reserved_extra = InlineQueryResultArticle::new("article-id", "Title", "hello");
        reserved_extra
            .extra
            .insert("type".to_owned(), serde_json::json!("photo"));
        reserved_extra
            .extra
            .insert("id".to_owned(), serde_json::json!("overridden-id"));
        reserved_extra.input_message_content.extra.insert(
            "message_text".to_owned(),
            serde_json::json!("overridden text"),
        );
        let reserved_extra_value = serde_json::to_value(&reserved_extra)?;
        assert_eq!(reserved_extra_value["type"], "article");
        assert_eq!(reserved_extra_value["id"], "article-id");
        assert_eq!(
            reserved_extra_value["input_message_content"]["message_text"],
            "hello"
        );

        let invalid = serde_json::json!({
            "type": "photo",
            "id": "article-id",
            "title": "Title",
            "input_message_content": {
                "message_text": "hello"
            }
        });
        assert!(serde_json::from_value::<InlineQueryResultArticle>(invalid).is_err());

        let mut conflicting_link_preview =
            InlineQueryResultArticle::new("article-id", "Title", "hello");
        conflicting_link_preview
            .input_message_content
            .disable_web_page_preview = Some(true);
        conflicting_link_preview
            .input_message_content
            .link_preview_options = Some(LinkPreviewOptions::disabled());
        assert!(matches!(
            InlineQueryResult::try_from(conflicting_link_preview),
            Err(Error::InvalidRequest { .. })
        ));

        Ok(())
    }

    #[test]
    fn inline_query_result_rejects_invalid_payloads()
    -> std::result::Result<(), Box<dyn std::error::Error>> {
        let valid = InlineQueryResult::new(serde_json::json!({
            "type": "article",
            "id": "result-id",
            "title": "Title",
        }))?;
        assert_eq!(valid.as_value()["type"], "article");

        assert!(matches!(
            InlineQueryResult::new(serde_json::json!({"id": "result-id"})),
            Err(Error::InvalidRequest { .. })
        ));
        assert!(matches!(
            InlineQueryResult::new(serde_json::json!({"type": "article", "id": 1})),
            Err(Error::InvalidRequest { .. })
        ));
        assert!(matches!(
            InlineQueryResult::new(serde_json::json!({"type": "article", "id": "x".repeat(65)})),
            Err(Error::InvalidRequest { .. })
        ));

        let decoded = serde_json::from_value::<InlineQueryResult>(serde_json::json!({
            "type": "article",
            "id": "\n"
        }));
        assert!(decoded.is_err());

        Ok(())
    }

    #[test]
    fn markup_extra_cannot_override_reserved_fields()
    -> std::result::Result<(), Box<dyn std::error::Error>> {
        let mut inline_button = InlineKeyboardButton::callback("Open", "real-action")?;
        inline_button
            .extra
            .insert("text".to_owned(), serde_json::json!("overridden"));
        inline_button.extra.insert(
            "web_app".to_owned(),
            serde_json::json!({"url": "https://example.com/overridden"}),
        );
        let inline_button_json = serde_json::to_value(&inline_button)?;
        assert_eq!(inline_button_json["text"], "Open");
        assert!(inline_button_json.get("web_app").is_none());
        assert_eq!(inline_button_json["callback_data"], "real-action");

        let mut missing_action = InlineKeyboardButton::new("No action");
        missing_action
            .extra
            .insert("text".to_owned(), serde_json::json!("still no action"));
        assert!(matches!(
            missing_action.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let mut inline_markup = InlineKeyboardMarkup::single_row(vec![inline_button]);
        inline_markup
            .extra
            .insert("inline_keyboard".to_owned(), serde_json::json!([]));
        let inline_markup_json = serde_json::to_value(&inline_markup)?;
        assert_eq!(
            inline_markup_json["inline_keyboard"][0][0]["callback_data"],
            "real-action"
        );

        let mut keyboard_button = KeyboardButton::new("Share");
        keyboard_button
            .extra
            .insert("text".to_owned(), serde_json::json!("overridden"));
        keyboard_button.extra.insert(
            "web_app".to_owned(),
            serde_json::json!({"url": "https://example.com/overridden"}),
        );
        let keyboard_button_json = serde_json::to_value(&keyboard_button)?;
        assert_eq!(keyboard_button_json["text"], "Share");
        assert!(keyboard_button_json.get("web_app").is_none());

        let mut reply_keyboard = ReplyKeyboardMarkup::new(vec![vec![keyboard_button]]);
        reply_keyboard.selective = Some(true);
        reply_keyboard
            .extra
            .insert("keyboard".to_owned(), serde_json::json!([]));
        reply_keyboard
            .extra
            .insert("selective".to_owned(), serde_json::json!(false));
        let reply_keyboard_json = serde_json::to_value(&reply_keyboard)?;
        assert_eq!(reply_keyboard_json["keyboard"][0][0]["text"], "Share");
        assert_eq!(reply_keyboard_json["selective"], true);

        let mut remove = ReplyKeyboardRemove {
            selective: Some(true),
            ..ReplyKeyboardRemove::default()
        };
        remove
            .extra
            .insert("remove_keyboard".to_owned(), serde_json::json!(false));
        remove
            .extra
            .insert("selective".to_owned(), serde_json::json!(false));
        let remove_json = serde_json::to_value(&remove)?;
        assert_eq!(remove_json["remove_keyboard"], true);
        assert_eq!(remove_json["selective"], true);

        let mut force_reply = ForceReply {
            selective: Some(true),
            ..ForceReply::default()
        };
        force_reply
            .extra
            .insert("force_reply".to_owned(), serde_json::json!(false));
        force_reply
            .extra
            .insert("selective".to_owned(), serde_json::json!(false));
        let force_reply_json = serde_json::to_value(&force_reply)?;
        assert_eq!(force_reply_json["force_reply"], true);
        assert_eq!(force_reply_json["selective"], true);

        let mut link_preview = LinkPreviewOptions::disabled();
        link_preview
            .extra
            .insert("is_disabled".to_owned(), serde_json::json!(false));
        link_preview.extra.insert(
            "url".to_owned(),
            serde_json::json!("https://example.com/overridden"),
        );
        let link_preview_json = serde_json::to_value(&link_preview)?;
        assert_eq!(link_preview_json["is_disabled"], true);
        assert!(link_preview_json.get("url").is_none());

        let mut inline_results_button =
            InlineQueryResultsButton::web_app("Open results", "https://example.com/app");
        inline_results_button
            .extra
            .insert("text".to_owned(), serde_json::json!("overridden"));
        inline_results_button.extra.insert(
            "web_app".to_owned(),
            serde_json::json!({"url": "https://example.com/overridden"}),
        );
        let inline_results_button_json = serde_json::to_value(&inline_results_button)?;
        assert_eq!(inline_results_button_json["text"], "Open results");
        assert_eq!(
            inline_results_button_json["web_app"]["url"],
            "https://example.com/app"
        );

        Ok(())
    }

    #[test]
    fn reply_parameters_extra_cannot_override_reserved_fields()
    -> std::result::Result<(), Box<dyn std::error::Error>> {
        let mut reply = ReplyParameters::new(MessageId(42));
        reply.chat_id = Some(ChatId::from(1001));
        reply.allow_sending_without_reply = Some(true);
        reply.quote = Some("quoted text".to_owned());
        reply.quote_parse_mode = Some(ParseMode::MarkdownV2);
        reply.quote_position = Some(3);
        reply
            .extra
            .insert("message_id".to_owned(), serde_json::json!(1));
        reply
            .extra
            .insert("chat_id".to_owned(), serde_json::json!(2));
        reply.extra.insert(
            "allow_sending_without_reply".to_owned(),
            serde_json::json!(false),
        );
        reply
            .extra
            .insert("quote".to_owned(), serde_json::json!("overridden"));
        reply
            .extra
            .insert("quote_position".to_owned(), serde_json::json!(99));
        reply
            .extra
            .insert("future_field".to_owned(), serde_json::json!("kept"));

        let value = serde_json::to_value(&reply)?;
        assert_eq!(value["message_id"], 42);
        assert_eq!(value["chat_id"], 1001);
        assert_eq!(value["allow_sending_without_reply"], true);
        assert_eq!(value["quote"], "quoted text");
        assert_eq!(value["quote_parse_mode"], "MarkdownV2");
        assert_eq!(value["quote_position"], 3);
        assert_eq!(value["future_field"], "kept");

        Ok(())
    }

    #[test]
    fn web_app_data_preserves_future_fields() -> std::result::Result<(), Box<dyn std::error::Error>>
    {
        let parsed = serde_json::from_value::<WebAppData>(serde_json::json!({
            "data": r#"{"query_id":"q-1","item":"coffee"}"#,
            "button_text": "Open",
            "future_field": "kept"
        }))?;
        assert_eq!(parsed.data, r#"{"query_id":"q-1","item":"coffee"}"#);
        assert_eq!(parsed.button_text, "Open");
        assert_eq!(
            parsed.extra.get("future_field"),
            Some(&serde_json::json!("kept"))
        );

        Ok(())
    }

    #[test]
    fn menu_button_preserves_known_and_unknown_payloads()
    -> std::result::Result<(), Box<dyn std::error::Error>> {
        let future = serde_json::json!({"kept": true});
        let commands = MenuButton::new(serde_json::json!({
            "type": "commands",
            "future": future,
        }));
        assert_eq!(commands.kind(), Some("commands"));
        assert!(commands.is_commands());
        assert_eq!(
            commands
                .as_commands()
                .and_then(|value| value.extra.get("future")),
            Some(&future)
        );
        let commands_json = serde_json::to_value(&commands)?;
        assert_eq!(commands_json["type"], "commands");
        assert_eq!(commands_json["future"], future);
        assert_eq!(
            commands
                .clone()
                .into_commands()
                .and_then(|value| value.extra.get("future").cloned()),
            Some(future.clone())
        );

        let default_button = MenuButton::new(serde_json::json!({
            "type": "default",
            "future": "value",
        }));
        assert_eq!(default_button.kind(), Some("default"));
        assert!(default_button.is_default());
        let default_json = serde_json::to_value(&default_button)?;
        assert_eq!(default_json["future"], "value");

        let nested_future = serde_json::json!({"nested": true});
        let web_app = MenuButton::new(serde_json::json!({
            "type": "web_app",
            "text": "Open",
            "web_app": {
                "url": "https://example.com/app",
                "future": nested_future,
            },
            "future": "outer",
        }));
        assert_eq!(web_app.kind(), Some("web_app"));
        assert!(web_app.is_web_app());
        assert_eq!(
            web_app
                .as_web_app()
                .and_then(|value| value.web_app.extra.get("future")),
            Some(&nested_future)
        );
        let web_app_json = serde_json::to_value(&web_app)?;
        assert_eq!(web_app_json["future"], "outer");
        assert_eq!(web_app_json["web_app"]["future"], nested_future);

        let raw_unknown = serde_json::json!({
            "type": "custom_menu_button",
            "raw_field": "raw_value",
        });
        let unknown = MenuButton::new(raw_unknown.clone());
        assert_eq!(unknown.kind(), Some("custom_menu_button"));
        assert_eq!(unknown.as_unknown_value(), Some(&raw_unknown));
        assert_eq!(serde_json::to_value(&unknown)?, raw_unknown);
        assert_eq!(unknown.into_unknown_value(), Some(raw_unknown));

        assert!(matches!(
            MenuButton::new(serde_json::json!({"type": "web_app"})).validate(),
            Err(Error::InvalidRequest { .. })
        ));

        Ok(())
    }

    #[test]
    fn validates_markup_payloads() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let valid_inline = InlineKeyboardMarkup::single_row(vec![InlineKeyboardButton::callback(
            "Open", "open:1",
        )?]);
        assert!(valid_inline.validate().is_ok());

        let pay_button = InlineKeyboardButton::pay("Pay");
        assert!(pay_button.is_pay_button());
        assert!(pay_button.validate().is_ok());

        assert!(matches!(
            InlineKeyboardMarkup::new(Vec::new()).validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let mut invalid_callback = InlineKeyboardButton::new("Open");
        invalid_callback.extra.insert(
            "callback_data".to_owned(),
            Value::String("x".repeat(MAX_CALLBACK_DATA_BYTES + 1)),
        );
        assert!(matches!(
            invalid_callback.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let mut invalid_pay = InlineKeyboardButton::new("Pay");
        invalid_pay
            .extra
            .insert("pay".to_owned(), Value::String("true".to_owned()));
        assert!(matches!(
            invalid_pay.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let mut invalid_login_url = InlineKeyboardButton::new("Login");
        invalid_login_url.extra.insert(
            "login_url".to_owned(),
            Value::String("https://example.com".to_owned()),
        );
        assert!(matches!(
            invalid_login_url.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let mut valid_url = InlineKeyboardButton::new("Docs");
        valid_url.extra.insert(
            "url".to_owned(),
            Value::String("https://example.com/docs".to_owned()),
        );
        assert!(valid_url.validate().is_ok());

        let mut valid_copy_text = InlineKeyboardButton::new("Copy");
        valid_copy_text.extra.insert(
            "copy_text".to_owned(),
            serde_json::json!({"text": "copy me"}),
        );
        assert!(valid_copy_text.validate().is_ok());

        let mut invalid_copy_text = InlineKeyboardButton::new("Copy");
        invalid_copy_text
            .extra
            .insert("copy_text".to_owned(), serde_json::json!({}));
        assert!(matches!(
            invalid_copy_text.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let invalid_web_app = InlineKeyboardButton::new("Open").web_app("http://example.com/app");
        assert!(matches!(
            invalid_web_app.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let mut conflicting_inline_action =
            InlineKeyboardButton::new("Open").web_app("https://example.com/app");
        conflicting_inline_action
            .extra
            .insert("pay".to_owned(), Value::Bool(true));
        assert!(matches!(
            conflicting_inline_action.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let valid_keyboard = ReplyKeyboardMarkup::new(vec![vec![
            KeyboardButton::new("Open").web_app("https://example.com/app"),
        ]]);
        assert!(ReplyMarkup::from(valid_keyboard).validate().is_ok());

        let mut invalid_request_contact = KeyboardButton::new("Share contact");
        invalid_request_contact.extra.insert(
            "request_contact".to_owned(),
            Value::String("yes".to_owned()),
        );
        assert!(matches!(
            invalid_request_contact.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let mut conflicting_keyboard_action =
            KeyboardButton::new("Open").web_app("https://example.com/app");
        conflicting_keyboard_action
            .extra
            .insert("request_location".to_owned(), Value::Bool(true));
        assert!(matches!(
            conflicting_keyboard_action.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let mut invalid_request_users = KeyboardButton::new("Pick user");
        invalid_request_users
            .extra
            .insert("request_users".to_owned(), serde_json::json!({}));
        assert!(matches!(
            invalid_request_users.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let mut valid_request_poll = KeyboardButton::new("Create quiz");
        valid_request_poll.extra.insert(
            "request_poll".to_owned(),
            serde_json::json!({"type": "quiz"}),
        );
        assert!(valid_request_poll.validate().is_ok());

        let invalid_remove = ReplyKeyboardRemove {
            remove_keyboard: false,
            ..ReplyKeyboardRemove::default()
        };
        assert!(matches!(
            invalid_remove.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        assert!(
            MenuButton::new(serde_json::json!({
                "type": "custom_menu_button",
                "raw_field": "raw_value"
            }))
            .validate()
            .is_ok()
        );
        assert!(matches!(
            MenuButton::new(serde_json::json!({"raw_field": "raw_value"})).validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let mut conflicting_inline_button =
            InlineQueryResultsButton::web_app("Open", "https://example.com/app");
        conflicting_inline_button.start_parameter = Some("start".to_owned());
        assert!(matches!(
            conflicting_inline_button.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        Ok(())
    }

    #[test]
    fn validates_reply_and_link_preview_options()
    -> std::result::Result<(), Box<dyn std::error::Error>> {
        let mut reply = ReplyParameters::new(MessageId(1));
        reply.quote = Some("quoted\ntext".to_owned());
        reply.quote_parse_mode = Some(ParseMode::MarkdownV2);
        assert!(reply.validate().is_ok());

        let mut date_time_entity = test_entity(MessageEntityKind::DateTime, 4);
        date_time_entity.unix_time = Some(1_700_000_000);
        let mut reply_with_date_time = ReplyParameters::new(MessageId(1));
        reply_with_date_time.quote = Some("date".to_owned());
        reply_with_date_time.quote_entities = Some(vec![date_time_entity]);
        assert!(reply_with_date_time.validate().is_ok());

        let mut reply_with_unsupported_entity = ReplyParameters::new(MessageId(1));
        reply_with_unsupported_entity.quote = Some("https://example.com".to_owned());
        reply_with_unsupported_entity.quote_entities =
            Some(vec![test_entity(MessageEntityKind::Url, 19)]);
        assert!(matches!(
            reply_with_unsupported_entity.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        reply.quote_entities = Some(Vec::new());
        assert!(matches!(
            reply.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let mut reply_without_quote = ReplyParameters::new(MessageId(1));
        reply_without_quote.quote_parse_mode = Some(ParseMode::MarkdownV2);
        assert!(matches!(
            reply_without_quote.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let mut link_preview = LinkPreviewOptions::new();
        link_preview.url = Some("https://example.com/article".to_owned());
        assert!(link_preview.validate().is_ok());

        link_preview.prefer_small_media = Some(true);
        link_preview.prefer_large_media = Some(true);
        assert!(matches!(
            link_preview.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let mut invalid_disabled = LinkPreviewOptions::disabled();
        invalid_disabled.url = Some("https://example.com/article".to_owned());
        assert!(matches!(
            invalid_disabled.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let mut invalid_url = LinkPreviewOptions::new();
        invalid_url.url = Some("not a url".to_owned());
        assert!(matches!(
            invalid_url.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        Ok(())
    }

    #[test]
    fn validates_generic_json_payload_wrappers()
    -> std::result::Result<(), Box<dyn std::error::Error>> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() as i64;

        let mut date_time_entity = test_entity(MessageEntityKind::DateTime, 4);
        date_time_entity.unix_time = Some(1_700_000_000);
        date_time_entity.date_time_format = Some("MMM d".to_owned());
        let checklist = InputChecklist::new(
            "Deploy",
            [InputChecklistTask::new(1, "Ship").text_entities(vec![date_time_entity.clone()])],
        )
        .title_entities(vec![date_time_entity]);
        checklist.validate()?;
        let checklist_json = serde_json::to_value(&checklist)?;
        assert_eq!(checklist_json["title"], "Deploy");
        assert_eq!(checklist_json["tasks"][0]["id"], 1);
        assert_eq!(
            checklist_json["tasks"][0]["text_entities"][0]["type"],
            "date_time"
        );

        let story_content = InputStoryContent::photo("file-id");
        assert_eq!(story_content.kind(), Some("photo"));
        let story_content_json = serde_json::to_value(&story_content)?;
        assert_eq!(story_content_json["type"], "photo");
        assert_eq!(story_content_json["photo"], "file-id");

        let story_video = InputStoryContent::video("video-file-id");
        assert_eq!(story_video.kind(), Some("video"));
        InputStoryContent::try_from_value(serde_json::json!({
            "type": "video",
            "video": "video-file-id",
            "duration": 60.0
        }))?;

        let paid_media = InputPaidMedia::photo("file-id");
        assert_eq!(paid_media.kind(), Some("photo"));
        let paid_media_json = serde_json::to_value(&paid_media)?;
        assert_eq!(paid_media_json["type"], "photo");
        assert_eq!(paid_media_json["media"], "file-id");

        let paid_video = InputPaidMedia::try_from_value(serde_json::json!({
            "type": "video",
            "media": "video-file-id",
            "duration": 1
        }))?;
        assert_eq!(paid_video.kind(), Some("video"));
        let paid_live_photo = InputPaidMedia::live_photo("video-file-id", "photo-file-id");
        paid_live_photo.validate()?;

        let profile_photo = InputProfilePhoto::static_photo("file-id");
        assert_eq!(profile_photo.kind(), Some("static"));
        let profile_photo_json = serde_json::to_value(&profile_photo)?;
        assert_eq!(profile_photo_json["type"], "static");
        assert_eq!(profile_photo_json["photo"], "file-id");
        let animated_profile_photo = InputProfilePhoto::try_from_value(serde_json::json!({
            "type": "animated",
            "animation": "animation-file-id",
            "main_frame_timestamp": 0.5
        }))?;
        assert_eq!(animated_profile_photo.kind(), Some("animated"));

        let reaction = ReactionType::emoji("👍");
        assert_eq!(reaction.kind(), Some("emoji"));
        let reaction_json = serde_json::to_value(&reaction)?;
        assert_eq!(reaction_json["type"], "emoji");
        assert_eq!(reaction_json["emoji"], "👍");
        let custom_reaction = ReactionType::try_from_value(serde_json::json!({
            "type": "custom_emoji",
            "custom_emoji_id": "custom-emoji-id"
        }))?;
        assert_eq!(custom_reaction.kind(), Some("custom_emoji"));
        let paid_reaction_json = serde_json::to_value(ReactionType::paid())?;
        assert_eq!(paid_reaction_json["type"], "paid");

        let passport_error = PassportElementError::new(serde_json::json!({
            "source": "data",
            "type": "passport",
            "message": "invalid"
        }))?;
        assert_eq!(passport_error.as_value()["source"], "data");

        assert!(
            AcceptedGiftTypes::new(serde_json::json!({
                "unique_gifts": true,
                "gifts_from_channels": false
            }))
            .is_ok()
        );
        assert!(
            SuggestedPostParameters::new(serde_json::json!({
                "send_date": now + 600,
                "price": {
                    "currency": "XTR",
                    "amount": 5
                }
            }))
            .is_ok()
        );
        assert!(
            SuggestedPostParameters::new(serde_json::json!({
                "price": {
                    "currency": "TON",
                    "amount": 10_000_000
                }
            }))
            .is_ok()
        );
        let position = StoryAreaPosition::new(50.0, 50.0, 20.0, 10.0, 0.0, 4.0);
        let story_area = StoryArea::try_from_value(serde_json::json!({
            "position": {
                "x_percentage": 50.0,
                "y_percentage": 50.0,
                "width_percentage": 20.0,
                "height_percentage": 10.0,
                "rotation_angle": 0.0,
                "corner_radius_percentage": 4.0
            },
            "type": {
                "type": "location",
                "latitude": 1.0,
                "longitude": 2.0
            }
        }))?;
        assert_eq!(story_area.kind.kind(), Some("location"));

        let link_area = StoryArea::link(position.clone(), "https://example.com");
        link_area.validate()?;
        let link_area_value = serde_json::to_value(&link_area)?;
        assert_eq!(link_area_value["type"]["type"], "link");
        assert_eq!(link_area_value["type"]["url"], "https://example.com");

        assert!(matches!(
            InputChecklist::try_from_value(Value::Null),
            Err(Error::InvalidRequest { .. })
        ));
        assert!(matches!(
            InputChecklist::new("Deploy", Vec::<InputChecklistTask>::new()).validate(),
            Err(Error::InvalidRequest { .. })
        ));
        assert!(matches!(
            InputChecklist::new(
                "Deploy",
                [
                    InputChecklistTask::new(1, "Ship"),
                    InputChecklistTask::new(1, "Verify")
                ],
            )
            .validate(),
            Err(Error::InvalidRequest { .. })
        ));
        assert!(matches!(
            InputChecklistTask::new(1, "https://example.com")
                .text_entities(vec![test_entity(MessageEntityKind::Url, 19)])
                .validate(),
            Err(Error::InvalidRequest { .. })
        ));
        assert!(matches!(
            InputPaidMedia::new(serde_json::json!({"media": "file-id"})),
            Err(Error::InvalidRequest { .. })
        ));
        assert!(matches!(
            InputPaidMedia::try_from_value(serde_json::json!({"type": "photo"})),
            Err(Error::InvalidRequest { .. })
        ));
        assert!(matches!(
            InputPaidMedia::try_from_value(serde_json::json!({
                "type": "live_photo",
                "media": "video-file-id"
            })),
            Err(Error::InvalidRequest { .. })
        ));
        assert!(matches!(
            InputPaidMedia::try_from_value(serde_json::json!({
                "type": "video",
                "media": "video-file-id",
                "duration": 0
            })),
            Err(Error::InvalidRequest { .. })
        ));
        assert!(matches!(
            InputProfilePhoto::new(serde_json::json!({"photo": "file-id"})),
            Err(Error::InvalidRequest { .. })
        ));
        assert!(matches!(
            InputProfilePhoto::try_from_value(serde_json::json!({"type": "static"})),
            Err(Error::InvalidRequest { .. })
        ));
        assert!(matches!(
            InputProfilePhoto::try_from_value(serde_json::json!({
                "type": "animated",
                "animation": "animation-file-id",
                "main_frame_timestamp": -0.1
            })),
            Err(Error::InvalidRequest { .. })
        ));
        assert!(matches!(
            ReactionType::new(serde_json::json!({"type": ""})),
            Err(Error::InvalidRequest { .. })
        ));
        assert!(matches!(
            ReactionType::try_from_value(serde_json::json!({"type": "emoji"})),
            Err(Error::InvalidRequest { .. })
        ));
        assert!(matches!(
            ReactionType::try_from_value(serde_json::json!({
                "type": "custom_emoji"
            })),
            Err(Error::InvalidRequest { .. })
        ));
        assert!(matches!(
            PassportElementError::new(serde_json::json!({"type": "passport"})),
            Err(Error::InvalidRequest { .. })
        ));
        assert!(matches!(
            AcceptedGiftTypes::new(serde_json::json!({"unique_gifts": "yes"})),
            Err(Error::InvalidRequest { .. })
        ));
        assert!(matches!(
            AcceptedGiftTypes::new(serde_json::json!({"gifts_from_channels": "yes"})),
            Err(Error::InvalidRequest { .. })
        ));
        assert!(matches!(
            SuggestedPostParameters::new(serde_json::json!({"send_date": "soon"})),
            Err(Error::InvalidRequest { .. })
        ));
        assert!(matches!(
            InputStoryContent::try_from_value(serde_json::json!({"type": "photo"})),
            Err(Error::InvalidRequest { .. })
        ));
        assert!(matches!(
            InputStoryContent::try_from_value(serde_json::json!({
                "type": "video",
                "video": "file-id",
                "duration": 60.1
            })),
            Err(Error::InvalidRequest { .. })
        ));
        assert!(matches!(
            StoryArea::try_from_value(serde_json::json!({
                "type": "location",
                "position": {}
            })),
            Err(Error::InvalidRequest { .. })
        ));
        assert!(matches!(
            StoryArea::try_from_value(serde_json::json!({
                "position": {
                    "x_percentage": 50.0,
                    "y_percentage": 50.0,
                    "width_percentage": 20.0,
                    "height_percentage": 10.0,
                    "rotation_angle": 0.0,
                    "corner_radius_percentage": 4.0
                },
                "type": {
                    "type": "location"
                }
            })),
            Err(Error::InvalidRequest { .. })
        ));
        assert!(matches!(
            validate_story_areas("areas", &vec![StoryArea::location(position, 1.0, 2.0); 11]),
            Err(Error::InvalidRequest { .. })
        ));
        assert!(matches!(
            SuggestedPostParameters::new(serde_json::json!({"send_date": now + 299})),
            Err(Error::InvalidRequest { .. })
        ));
        assert!(matches!(
            SuggestedPostParameters::new(serde_json::json!({"send_date": now + 2_679_000})),
            Err(Error::InvalidRequest { .. })
        ));
        assert!(matches!(
            SuggestedPostParameters::new(serde_json::json!({
                "price": {
                    "currency": "USD",
                    "amount": 5
                }
            })),
            Err(Error::InvalidRequest { .. })
        ));
        assert!(matches!(
            SuggestedPostParameters::new(serde_json::json!({
                "price": {
                    "currency": "XTR",
                    "amount": 4
                }
            })),
            Err(Error::InvalidRequest { .. })
        ));

        let decoded = serde_json::from_value::<InputStoryContent>(serde_json::json!({
            "photo": "file-id"
        }));
        assert!(decoded.is_err());

        Ok(())
    }
}

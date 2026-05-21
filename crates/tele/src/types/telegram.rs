use std::collections::BTreeMap;
use std::fmt::{Display, Write as _};
use std::str::FromStr;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use url::Url;

use crate::types::common::{ChatId, MessageId, NumericChatId, ParseMode};
use crate::types::message::MessageEntity;
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

fn is_disallowed_display_control(character: char) -> bool {
    character.is_control() && !matches!(character, '\n' | '\r' | '\t')
}

fn validate_required_display_text(field: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(invalid_request(format!("{field} cannot be empty")));
    }
    if value.chars().any(is_disallowed_display_control) {
        return Err(invalid_request(format!(
            "{field} must not contain non-whitespace control characters"
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

fn validate_url(field: &str, value: &str) -> Result<()> {
    let parsed = Url::parse(value)
        .map_err(|source| invalid_request(format!("{field} must be a valid URL: {source}")))?;
    match parsed.scheme() {
        "http" | "https" | "tg" => {}
        scheme => {
            return Err(invalid_request(format!(
                "{field} must use http, https, or tg scheme, got `{scheme}`"
            )));
        }
    }
    if matches!(parsed.scheme(), "http" | "https") && parsed.host_str().is_none() {
        return Err(invalid_request(format!("{field} must include a host")));
    }

    Ok(())
}

fn validate_https_url(field: &str, value: &str) -> Result<()> {
    let parsed = Url::parse(value)
        .map_err(|source| invalid_request(format!("{field} must be a valid URL: {source}")))?;
    if parsed.scheme() != "https" {
        return Err(invalid_request(format!("{field} must use HTTPS")));
    }
    if parsed.host_str().is_none() {
        return Err(invalid_request(format!("{field} must include a host")));
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

fn validate_object_payload(field: &str, value: &Value) -> Result<()> {
    let Some(object) = value.as_object() else {
        return Err(invalid_request(format!("{field} must be a JSON object")));
    };
    if object.is_empty() {
        return Err(invalid_request(format!("{field} cannot be empty")));
    }

    Ok(())
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
    ] {
        if object.get(key).is_some_and(|value| !value.is_boolean()) {
            return Err(invalid_request(format!("{field}.{key} must be a boolean")));
        }
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
#[derive(Clone, Debug, Serialize, Deserialize)]
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
}

/// Typed inline query article result.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum InlineQueryResultArticleKind {
    #[default]
    #[serde(rename = "article")]
    Article,
}

/// Typed inline query article result.
#[derive(Clone, Debug, Serialize, Deserialize)]
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
}

impl TryFrom<InlineQueryResultArticle> for InlineQueryResult {
    type Error = Error;

    fn try_from(value: InlineQueryResultArticle) -> std::result::Result<Self, Self::Error> {
        Self::try_from_typed(value)
    }
}

json_payload_wrapper!(
    /// Generic checklist input payload.
    InputChecklist,
    "input_checklist",
    validate_object_payload
);

json_payload_wrapper!(
    /// Generic story content payload.
    InputStoryContent,
    "input_story_content",
    validate_typed_object_payload
);

json_payload_wrapper!(
    /// Generic story area payload.
    StoryArea,
    "story_area",
    validate_typed_object_payload
);

json_payload_wrapper!(
    /// Generic paid media item payload.
    InputPaidMedia,
    "input_paid_media",
    validate_typed_object_payload
);

json_payload_wrapper!(
    /// Generic suggested-post payload.
    SuggestedPostParameters,
    "suggested_post_parameters",
    validate_object_payload
);

json_payload_wrapper!(
    /// Generic accepted-gift-types payload.
    AcceptedGiftTypes,
    "accepted_gift_types",
    validate_accepted_gift_types_payload
);

json_payload_wrapper!(
    /// Generic profile photo input payload.
    InputProfilePhoto,
    "input_profile_photo",
    validate_typed_object_payload
);

/// Typed menu button union.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MenuButton {
    Typed(MenuButtonKind),
    Other(Value),
}

/// Known menu button variants.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
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
        Self::Typed(MenuButtonKind::Commands)
    }

    pub fn default_button() -> Self {
        Self::Typed(MenuButtonKind::Default)
    }

    pub fn web_app(text: impl Into<String>, web_app: impl Into<WebAppInfo>) -> Self {
        Self::Typed(MenuButtonKind::WebApp(MenuButtonWebApp::new(text, web_app)))
    }

    pub fn as_web_app(&self) -> Option<&MenuButtonWebApp> {
        match self {
            Self::Typed(MenuButtonKind::WebApp(value)) => Some(value),
            Self::Typed(_) | Self::Other(_) => None,
        }
    }

    pub fn validate(&self) -> Result<()> {
        match self {
            Self::Typed(MenuButtonKind::Commands | MenuButtonKind::Default) => Ok(()),
            Self::Typed(MenuButtonKind::WebApp(value)) => value.validate(),
            Self::Other(value) => validate_typed_object_payload("menu_button", value),
        }
    }
}

impl Default for MenuButton {
    fn default() -> Self {
        Self::default_button()
    }
}

impl From<Value> for MenuButton {
    fn from(value: Value) -> Self {
        match serde_json::from_value::<MenuButtonKind>(value.clone()) {
            Ok(known) => Self::Typed(known),
            Err(_error) => Self::Other(value),
        }
    }
}

impl From<MenuButtonKind> for MenuButton {
    fn from(value: MenuButtonKind) -> Self {
        Self::Typed(value)
    }
}

impl From<MenuButton> for Value {
    fn from(value: MenuButton) -> Self {
        match value {
            MenuButton::Typed(known) => match known {
                MenuButtonKind::Commands => serde_json::json!({"type": "commands"}),
                MenuButtonKind::Default => serde_json::json!({"type": "default"}),
                MenuButtonKind::WebApp(mut value) => {
                    let mut object = serde_json::Map::new();
                    let mut web_app = serde_json::Map::new();
                    web_app.insert("url".to_owned(), Value::String(value.web_app.url));
                    object.insert("type".to_owned(), Value::String("web_app".to_owned()));
                    object.insert("text".to_owned(), Value::String(value.text));
                    object.insert("web_app".to_owned(), Value::Object(web_app));
                    for (key, extra_value) in std::mem::take(&mut value.extra) {
                        object.insert(key, extra_value);
                    }
                    Value::Object(object)
                }
            },
            MenuButton::Other(value) => value,
        }
    }
}

/// Mini App Web App descriptor.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct WebAppInfo {
    pub url: String,
}

impl WebAppInfo {
    pub fn new(url: impl Into<String>) -> Self {
        Self { url: url.into() }
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

/// Button shown above inline query results.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
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
        if self.web_app.is_none() && self.start_parameter.is_none() && self.extra.is_empty() {
            return Err(invalid_request(
                "inline query results button must define an action",
            ));
        }

        Ok(())
    }
}

/// Menu button launching a Mini App.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
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

impl From<MenuButtonWebApp> for MenuButton {
    fn from(value: MenuButtonWebApp) -> Self {
        Self::Typed(MenuButtonKind::WebApp(value))
    }
}

/// Data sent from Mini App via `Telegram.WebApp.sendData`.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
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
    /// Generic reaction type payload.
    ReactionType,
    "reaction_type",
    validate_typed_object_payload
);

json_payload_wrapper!(
    /// Generic passport element error payload.
    PassportElementError,
    "passport_element_error",
    validate_source_object_payload
);

/// Inline keyboard button.
#[derive(Clone, Debug, Serialize, Deserialize)]
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
        if known_actions == 0 && self.extra.is_empty() {
            return Err(invalid_request(
                "inline keyboard button must define an action",
            ));
        }

        Ok(())
    }
}

/// Inline keyboard markup.
#[derive(Clone, Debug, Serialize, Deserialize)]
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

/// Reply keyboard button.
#[derive(Clone, Debug, Serialize, Deserialize)]
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

/// Reply keyboard markup.
#[derive(Clone, Debug, Serialize, Deserialize)]
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

/// Remove reply keyboard marker.
#[derive(Clone, Debug, Serialize, Deserialize)]
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

/// Force reply marker.
#[derive(Clone, Debug, Serialize, Deserialize)]
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
#[derive(Clone, Debug, Serialize, Deserialize)]
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
        if self.quote_parse_mode.is_some() && self.quote_entities.is_some() {
            return Err(invalid_request(
                "reply quote cannot set both quote_parse_mode and quote_entities",
            ));
        }
        if let Some(entities) = self.quote_entities.as_ref() {
            if entities.is_empty() {
                return Err(invalid_request("quote_entities cannot be empty"));
            }
            for entity in entities {
                if entity.length == 0 {
                    return Err(invalid_request(
                        "quote_entities length must be greater than 0",
                    ));
                }
            }
        }

        Ok(())
    }
}

/// Link preview options for text messages.
#[derive(Clone, Debug, Serialize, Deserialize)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inline_query_article_kind_is_fixed() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let article = InlineQueryResultArticle::new("article-id", "Title", "hello");
        assert_eq!(article.kind, InlineQueryResultArticleKind::Article);

        let value = serde_json::to_value(&article)?;
        assert_eq!(value["type"], "article");

        let parsed: InlineQueryResultArticle = serde_json::from_value(value)?;
        assert_eq!(parsed.kind, InlineQueryResultArticleKind::Article);

        let invalid = serde_json::json!({
            "type": "photo",
            "id": "article-id",
            "title": "Title",
            "input_message_content": {
                "message_text": "hello"
            }
        });
        assert!(serde_json::from_value::<InlineQueryResultArticle>(invalid).is_err());

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
    fn validates_markup_payloads() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let valid_inline = InlineKeyboardMarkup::single_row(vec![InlineKeyboardButton::callback(
            "Open", "open:1",
        )?]);
        assert!(valid_inline.validate().is_ok());

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
        let checklist = InputChecklist::new(serde_json::json!({
            "title": "Deploy",
            "tasks": []
        }))?;
        assert_eq!(checklist.as_value()["title"], "Deploy");

        let story_content = InputStoryContent::new(serde_json::json!({
            "type": "photo",
            "photo": "file-id"
        }))?;
        assert_eq!(story_content.as_value()["type"], "photo");

        let paid_media = InputPaidMedia::new(serde_json::json!({
            "type": "photo",
            "media": "file-id"
        }))?;
        assert_eq!(paid_media.as_value()["type"], "photo");

        let profile_photo = InputProfilePhoto::new(serde_json::json!({
            "type": "static",
            "photo": "file-id"
        }))?;
        assert_eq!(profile_photo.as_value()["type"], "static");

        let reaction = ReactionType::new(serde_json::json!({"type": "emoji", "emoji": "ok"}))?;
        assert_eq!(reaction.as_value()["type"], "emoji");

        let passport_error = PassportElementError::new(serde_json::json!({
            "source": "data",
            "type": "passport",
            "message": "invalid"
        }))?;
        assert_eq!(passport_error.as_value()["source"], "data");

        assert!(
            AcceptedGiftTypes::new(serde_json::json!({
                "unique_gifts": true
            }))
            .is_ok()
        );
        assert!(
            SuggestedPostParameters::new(serde_json::json!({
                "send_date": 1
            }))
            .is_ok()
        );
        assert!(
            StoryArea::new(serde_json::json!({
                "type": "location",
                "position": {}
            }))
            .is_ok()
        );

        assert!(matches!(
            InputChecklist::new(Value::Null),
            Err(Error::InvalidRequest { .. })
        ));
        assert!(matches!(
            InputPaidMedia::new(serde_json::json!({"media": "file-id"})),
            Err(Error::InvalidRequest { .. })
        ));
        assert!(matches!(
            InputProfilePhoto::new(serde_json::json!({"photo": "file-id"})),
            Err(Error::InvalidRequest { .. })
        ));
        assert!(matches!(
            ReactionType::new(serde_json::json!({"type": ""})),
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

        let decoded = serde_json::from_value::<InputStoryContent>(serde_json::json!({
            "photo": "file-id"
        }));
        assert!(decoded.is_err());

        Ok(())
    }
}

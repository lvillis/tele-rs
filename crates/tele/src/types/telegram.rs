use std::collections::BTreeMap;
use std::fmt::{Display, Write as _};
use std::str::FromStr;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::common::{ChatId, MessageId, ParseMode};
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

/// Generic inline query result payload.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct InlineQueryResult(pub Value);

impl InlineQueryResult {
    pub fn new(value: Value) -> Self {
        Self(value)
    }

    pub fn try_from_typed<T>(value: T) -> std::result::Result<Self, serde_json::Error>
    where
        T: Serialize,
    {
        serde_json::to_value(value).map(Self)
    }

    pub fn from_typed<T>(value: T) -> std::result::Result<Self, serde_json::Error>
    where
        T: Serialize,
    {
        Self::try_from_typed(value)
    }

    pub fn article(
        id: impl Into<String>,
        title: impl Into<String>,
        message_text: impl Into<String>,
    ) -> std::result::Result<Self, serde_json::Error> {
        InlineQueryResult::try_from(InlineQueryResultArticle::new(id, title, message_text))
    }

    pub fn as_value(&self) -> &Value {
        &self.0
    }

    pub fn into_value(self) -> Value {
        self.0
    }
}

impl From<Value> for InlineQueryResult {
    fn from(value: Value) -> Self {
        Self(value)
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
#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub struct InlineQueryResultArticle {
    #[serde(rename = "type")]
    pub kind: String,
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
            kind: "article".to_owned(),
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
    type Error = serde_json::Error;

    fn try_from(value: InlineQueryResultArticle) -> std::result::Result<Self, Self::Error> {
        Self::try_from_typed(value)
    }
}

/// Generic checklist input payload.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct InputChecklist(pub Value);

impl InputChecklist {
    pub fn new(value: Value) -> Self {
        Self(value)
    }
}

impl From<Value> for InputChecklist {
    fn from(value: Value) -> Self {
        Self(value)
    }
}

impl From<InputChecklist> for Value {
    fn from(value: InputChecklist) -> Self {
        value.0
    }
}

/// Generic story content payload.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct InputStoryContent(pub Value);

impl InputStoryContent {
    pub fn new(value: Value) -> Self {
        Self(value)
    }
}

impl From<Value> for InputStoryContent {
    fn from(value: Value) -> Self {
        Self(value)
    }
}

impl From<InputStoryContent> for Value {
    fn from(value: InputStoryContent) -> Self {
        value.0
    }
}

/// Generic story area payload.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct StoryArea(pub Value);

impl StoryArea {
    pub fn new(value: Value) -> Self {
        Self(value)
    }
}

impl From<Value> for StoryArea {
    fn from(value: Value) -> Self {
        Self(value)
    }
}

impl From<StoryArea> for Value {
    fn from(value: StoryArea) -> Self {
        value.0
    }
}

/// Generic paid media item payload.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct InputPaidMedia(pub Value);

impl InputPaidMedia {
    pub fn new(value: Value) -> Self {
        Self(value)
    }
}

impl From<Value> for InputPaidMedia {
    fn from(value: Value) -> Self {
        Self(value)
    }
}

impl From<InputPaidMedia> for Value {
    fn from(value: InputPaidMedia) -> Self {
        value.0
    }
}

/// Generic suggested-post payload.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SuggestedPostParameters(pub Value);

impl SuggestedPostParameters {
    pub fn new(value: Value) -> Self {
        Self(value)
    }
}

impl From<Value> for SuggestedPostParameters {
    fn from(value: Value) -> Self {
        Self(value)
    }
}

impl From<SuggestedPostParameters> for Value {
    fn from(value: SuggestedPostParameters) -> Self {
        value.0
    }
}

/// Generic accepted-gift-types payload.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AcceptedGiftTypes(pub Value);

impl AcceptedGiftTypes {
    pub fn new(value: Value) -> Self {
        Self(value)
    }
}

impl From<Value> for AcceptedGiftTypes {
    fn from(value: Value) -> Self {
        Self(value)
    }
}

impl From<AcceptedGiftTypes> for Value {
    fn from(value: AcceptedGiftTypes) -> Self {
        value.0
    }
}

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
    pub fn chat_id(mut self, chat_id: i64) -> Self {
        self.chat_id = Some(chat_id);
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

/// Generic reaction type payload.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ReactionType(pub Value);

impl ReactionType {
    pub fn new(value: Value) -> Self {
        Self(value)
    }
}

impl From<Value> for ReactionType {
    fn from(value: Value) -> Self {
        Self(value)
    }
}

impl From<ReactionType> for Value {
    fn from(value: ReactionType) -> Self {
        value.0
    }
}

/// Generic passport element error payload.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PassportElementError(pub Value);

impl PassportElementError {
    pub fn new(value: Value) -> Self {
        Self(value)
    }
}

impl From<Value> for PassportElementError {
    fn from(value: Value) -> Self {
        Self(value)
    }
}

impl From<PassportElementError> for Value {
    fn from(value: PassportElementError) -> Self {
        value.0
    }
}

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
}

impl Default for LinkPreviewOptions {
    fn default() -> Self {
        Self::new()
    }
}

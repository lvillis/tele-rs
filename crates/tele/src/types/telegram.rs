use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::common::{ChatId, MessageId, ParseMode};
use crate::types::message::MessageEntity;

/// Generic inline query result payload.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct InlineQueryResult(pub Value);

impl InlineQueryResult {
    pub fn new(value: Value) -> Self {
        Self(value)
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

/// Generic menu button payload.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MenuButton(pub Value);

impl MenuButton {
    pub fn new(value: Value) -> Self {
        Self(value)
    }
}

impl From<Value> for MenuButton {
    fn from(value: Value) -> Self {
        Self(value)
    }
}

impl From<MenuButton> for Value {
    fn from(value: MenuButton) -> Self {
        value.0
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
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl InlineKeyboardButton {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            extra: BTreeMap::new(),
        }
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
}

/// Reply keyboard button.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub struct KeyboardButton {
    pub text: String,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl KeyboardButton {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            extra: BTreeMap::new(),
        }
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

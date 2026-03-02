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

    pub fn from_typed<T>(value: T) -> Self
    where
        T: Serialize,
    {
        match serde_json::to_value(value) {
            Ok(value) => Self(value),
            Err(_error) => Self(Value::Null),
        }
    }

    pub fn article(
        id: impl Into<String>,
        title: impl Into<String>,
        message_text: impl Into<String>,
    ) -> Self {
        InlineQueryResultArticle::new(id, title, message_text).into()
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

impl From<InlineQueryResultArticle> for InlineQueryResult {
    fn from(value: InlineQueryResultArticle) -> Self {
        Self::from_typed(value)
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
            MenuButton::Typed(known) => match serde_json::to_value(known) {
                Ok(value) => value,
                Err(_error) => Value::Null,
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

    pub fn web_app(mut self, web_app: impl Into<WebAppInfo>) -> Self {
        self.web_app = Some(web_app.into());
        self
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

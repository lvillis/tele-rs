use serde::{Deserialize, Serialize};

use crate::Error;
use crate::types::common::{ChatId, MessageId, ParseMode};
use crate::types::telegram::{LinkPreviewOptions, ReplyMarkup, ReplyParameters};

use super::content::DiceEmoji;
use super::model::Message;

const MAX_MESSAGE_TEXT_CHARS: usize = 4096;
const MAX_CAPTION_CHARS: usize = 1024;
const MIN_MEDIA_GROUP_ITEMS: usize = 2;
const MAX_MEDIA_GROUP_ITEMS: usize = 10;
const MAX_BULK_MESSAGE_IDS: usize = 100;
const MAX_POLL_OPTIONS: usize = 12;
const MAX_POLL_QUESTION_CHARS: usize = 300;
const MAX_POLL_OPTION_CHARS: usize = 100;
const MIN_POLL_OPEN_PERIOD_SECONDS: u16 = 5;
const MAX_POLL_OPEN_PERIOD_SECONDS: u16 = 600;

#[derive(Clone, Debug, Serialize)]
pub struct SendMessageRequest {
    pub chat_id: ChatId,
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parse_mode: Option<ParseMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_web_page_preview: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_notification: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protect_content: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_thread_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_parameters: Option<ReplyParameters>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_markup: Option<ReplyMarkup>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub link_preview_options: Option<LinkPreviewOptions>,
}

impl SendMessageRequest {
    pub fn new(chat_id: impl Into<ChatId>, text: impl Into<String>) -> Result<Self, Error> {
        let chat_id = chat_id.into();
        chat_id.validate()?;
        let text = text.into();
        validate_message_text("sendMessage", &text)?;

        Ok(Self {
            chat_id,
            text,
            parse_mode: None,
            disable_web_page_preview: None,
            disable_notification: None,
            protect_content: None,
            message_thread_id: None,
            reply_parameters: None,
            reply_markup: None,
            link_preview_options: None,
        })
    }

    pub fn parse_mode(mut self, parse_mode: ParseMode) -> Self {
        self.parse_mode = Some(parse_mode);
        self
    }

    pub fn validate(&self) -> Result<(), Error> {
        self.chat_id.validate()?;
        validate_message_text("sendMessage", &self.text)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ForwardMessageRequest {
    pub chat_id: ChatId,
    pub from_chat_id: ChatId,
    pub message_id: MessageId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_thread_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_notification: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protect_content: Option<bool>,
}

impl ForwardMessageRequest {
    pub fn new(
        chat_id: impl Into<ChatId>,
        from_chat_id: impl Into<ChatId>,
        message_id: MessageId,
    ) -> Self {
        Self {
            chat_id: chat_id.into(),
            from_chat_id: from_chat_id.into(),
            message_id,
            message_thread_id: None,
            disable_notification: None,
            protect_content: None,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct CopyMessageRequest {
    pub chat_id: ChatId,
    pub from_chat_id: ChatId,
    pub message_id: MessageId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_thread_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parse_mode: Option<ParseMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_notification: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protect_content: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_parameters: Option<ReplyParameters>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_markup: Option<ReplyMarkup>,
}

impl CopyMessageRequest {
    pub fn new(
        chat_id: impl Into<ChatId>,
        from_chat_id: impl Into<ChatId>,
        message_id: MessageId,
    ) -> Self {
        Self {
            chat_id: chat_id.into(),
            from_chat_id: from_chat_id.into(),
            message_id,
            message_thread_id: None,
            caption: None,
            parse_mode: None,
            disable_notification: None,
            protect_content: None,
            reply_parameters: None,
            reply_markup: None,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct CopyMessagesRequest {
    pub chat_id: ChatId,
    pub from_chat_id: ChatId,
    pub message_ids: Vec<MessageId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_thread_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_notification: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protect_content: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remove_caption: Option<bool>,
}

impl CopyMessagesRequest {
    pub fn new(
        chat_id: impl Into<ChatId>,
        from_chat_id: impl Into<ChatId>,
        message_ids: Vec<MessageId>,
    ) -> Result<Self, Error> {
        if message_ids.is_empty() {
            return Err(Error::InvalidRequest {
                reason: "copyMessages requires at least one message id".to_owned(),
            });
        }
        if message_ids.len() > MAX_BULK_MESSAGE_IDS {
            return Err(Error::InvalidRequest {
                reason: format!("copyMessages accepts at most {MAX_BULK_MESSAGE_IDS} message ids"),
            });
        }

        Ok(Self {
            chat_id: chat_id.into(),
            from_chat_id: from_chat_id.into(),
            message_ids,
            message_thread_id: None,
            disable_notification: None,
            protect_content: None,
            remove_caption: None,
        })
    }

    pub fn validate(&self) -> Result<(), Error> {
        validate_bulk_message_ids("copyMessages", &self.message_ids)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct MessageIdObject {
    pub message_id: MessageId,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct SentWebAppMessage {
    pub inline_message_id: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct SendPhotoRequest {
    pub chat_id: ChatId,
    pub photo: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parse_mode: Option<ParseMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_spoiler: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_notification: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protect_content: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_thread_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_parameters: Option<ReplyParameters>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_markup: Option<ReplyMarkup>,
}

impl SendPhotoRequest {
    pub fn new(chat_id: impl Into<ChatId>, photo: impl Into<String>) -> Self {
        Self {
            chat_id: chat_id.into(),
            photo: photo.into(),
            caption: None,
            parse_mode: None,
            has_spoiler: None,
            disable_notification: None,
            protect_content: None,
            message_thread_id: None,
            reply_parameters: None,
            reply_markup: None,
        }
    }

    pub fn validate(&self) -> Result<(), Error> {
        validate_file_reference("photo", &self.photo)?;
        validate_optional_caption(self.caption.as_deref())
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct SendAudioRequest {
    pub chat_id: ChatId,
    pub audio: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parse_mode: Option<ParseMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub performer: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_notification: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protect_content: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_thread_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_parameters: Option<ReplyParameters>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_markup: Option<ReplyMarkup>,
}

impl SendAudioRequest {
    pub fn new(chat_id: impl Into<ChatId>, audio: impl Into<String>) -> Self {
        Self {
            chat_id: chat_id.into(),
            audio: audio.into(),
            caption: None,
            parse_mode: None,
            duration: None,
            performer: None,
            title: None,
            thumbnail: None,
            disable_notification: None,
            protect_content: None,
            message_thread_id: None,
            reply_parameters: None,
            reply_markup: None,
        }
    }

    pub fn validate(&self) -> Result<(), Error> {
        validate_file_reference("audio", &self.audio)?;
        validate_optional_caption(self.caption.as_deref())?;
        validate_optional_file_reference("thumbnail", self.thumbnail.as_deref())
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct SendDocumentRequest {
    pub chat_id: ChatId,
    pub document: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parse_mode: Option<ParseMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_content_type_detection: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_notification: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protect_content: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_thread_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_parameters: Option<ReplyParameters>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_markup: Option<ReplyMarkup>,
}

impl SendDocumentRequest {
    pub fn new(chat_id: impl Into<ChatId>, document: impl Into<String>) -> Self {
        Self {
            chat_id: chat_id.into(),
            document: document.into(),
            thumbnail: None,
            caption: None,
            parse_mode: None,
            disable_content_type_detection: None,
            disable_notification: None,
            protect_content: None,
            message_thread_id: None,
            reply_parameters: None,
            reply_markup: None,
        }
    }

    pub fn validate(&self) -> Result<(), Error> {
        validate_file_reference("document", &self.document)?;
        validate_optional_caption(self.caption.as_deref())?;
        validate_optional_file_reference("thumbnail", self.thumbnail.as_deref())
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct SendVideoRequest {
    pub chat_id: ChatId,
    pub video: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parse_mode: Option<ParseMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supports_streaming: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_spoiler: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_notification: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protect_content: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_thread_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_parameters: Option<ReplyParameters>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_markup: Option<ReplyMarkup>,
}

impl SendVideoRequest {
    pub fn new(chat_id: impl Into<ChatId>, video: impl Into<String>) -> Self {
        Self {
            chat_id: chat_id.into(),
            video: video.into(),
            duration: None,
            width: None,
            height: None,
            thumbnail: None,
            caption: None,
            parse_mode: None,
            supports_streaming: None,
            has_spoiler: None,
            disable_notification: None,
            protect_content: None,
            message_thread_id: None,
            reply_parameters: None,
            reply_markup: None,
        }
    }

    pub fn validate(&self) -> Result<(), Error> {
        validate_file_reference("video", &self.video)?;
        validate_optional_caption(self.caption.as_deref())?;
        validate_optional_file_reference("thumbnail", self.thumbnail.as_deref())
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct SendAnimationRequest {
    pub chat_id: ChatId,
    pub animation: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parse_mode: Option<ParseMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_spoiler: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_notification: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protect_content: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_thread_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_parameters: Option<ReplyParameters>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_markup: Option<ReplyMarkup>,
}

impl SendAnimationRequest {
    pub fn new(chat_id: impl Into<ChatId>, animation: impl Into<String>) -> Self {
        Self {
            chat_id: chat_id.into(),
            animation: animation.into(),
            duration: None,
            width: None,
            height: None,
            thumbnail: None,
            caption: None,
            parse_mode: None,
            has_spoiler: None,
            disable_notification: None,
            protect_content: None,
            message_thread_id: None,
            reply_parameters: None,
            reply_markup: None,
        }
    }

    pub fn validate(&self) -> Result<(), Error> {
        validate_file_reference("animation", &self.animation)?;
        validate_optional_caption(self.caption.as_deref())?;
        validate_optional_file_reference("thumbnail", self.thumbnail.as_deref())
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct SendVoiceRequest {
    pub chat_id: ChatId,
    pub voice: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parse_mode: Option<ParseMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_notification: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protect_content: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_thread_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_parameters: Option<ReplyParameters>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_markup: Option<ReplyMarkup>,
}

impl SendVoiceRequest {
    pub fn new(chat_id: impl Into<ChatId>, voice: impl Into<String>) -> Self {
        Self {
            chat_id: chat_id.into(),
            voice: voice.into(),
            caption: None,
            parse_mode: None,
            duration: None,
            disable_notification: None,
            protect_content: None,
            message_thread_id: None,
            reply_parameters: None,
            reply_markup: None,
        }
    }

    pub fn validate(&self) -> Result<(), Error> {
        validate_file_reference("voice", &self.voice)?;
        validate_optional_caption(self.caption.as_deref())
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct SendVideoNoteRequest {
    pub chat_id: ChatId,
    pub video_note: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub length: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_notification: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protect_content: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_thread_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_parameters: Option<ReplyParameters>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_markup: Option<ReplyMarkup>,
}

impl SendVideoNoteRequest {
    pub fn new(chat_id: impl Into<ChatId>, video_note: impl Into<String>) -> Self {
        Self {
            chat_id: chat_id.into(),
            video_note: video_note.into(),
            duration: None,
            length: None,
            thumbnail: None,
            disable_notification: None,
            protect_content: None,
            message_thread_id: None,
            reply_parameters: None,
            reply_markup: None,
        }
    }

    pub fn validate(&self) -> Result<(), Error> {
        validate_file_reference("video_note", &self.video_note)?;
        validate_optional_file_reference("thumbnail", self.thumbnail.as_deref())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub struct InputMediaPhoto {
    pub media: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parse_mode: Option<ParseMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_spoiler: Option<bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub struct InputMediaVideo {
    pub media: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parse_mode: Option<ParseMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supports_streaming: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_spoiler: Option<bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub struct InputMediaAnimation {
    pub media: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parse_mode: Option<ParseMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_spoiler: Option<bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub struct InputMediaAudio {
    pub media: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parse_mode: Option<ParseMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub performer: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub struct InputMediaDocument {
    pub media: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parse_mode: Option<ParseMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_content_type_detection: Option<bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InputMedia {
    Photo(Box<InputMediaPhoto>),
    Video(Box<InputMediaVideo>),
    Animation(Box<InputMediaAnimation>),
    Audio(Box<InputMediaAudio>),
    Document(Box<InputMediaDocument>),
}

impl From<InputMediaPhoto> for InputMedia {
    fn from(value: InputMediaPhoto) -> Self {
        Self::Photo(Box::new(value))
    }
}

impl From<InputMediaVideo> for InputMedia {
    fn from(value: InputMediaVideo) -> Self {
        Self::Video(Box::new(value))
    }
}

impl From<InputMediaAnimation> for InputMedia {
    fn from(value: InputMediaAnimation) -> Self {
        Self::Animation(Box::new(value))
    }
}

impl From<InputMediaAudio> for InputMedia {
    fn from(value: InputMediaAudio) -> Self {
        Self::Audio(Box::new(value))
    }
}

impl From<InputMediaDocument> for InputMedia {
    fn from(value: InputMediaDocument) -> Self {
        Self::Document(Box::new(value))
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct SendMediaGroupRequest {
    pub chat_id: ChatId,
    pub media: Vec<InputMedia>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_notification: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protect_content: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_thread_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_parameters: Option<ReplyParameters>,
}

impl SendMediaGroupRequest {
    pub fn new(chat_id: impl Into<ChatId>, media: Vec<InputMedia>) -> Result<Self, Error> {
        if media.len() < MIN_MEDIA_GROUP_ITEMS {
            return Err(Error::InvalidRequest {
                reason: format!(
                    "sendMediaGroup requires at least {MIN_MEDIA_GROUP_ITEMS} media items"
                ),
            });
        }
        validate_media_group_items(&media)?;

        Ok(Self {
            chat_id: chat_id.into(),
            media,
            disable_notification: None,
            protect_content: None,
            message_thread_id: None,
            reply_parameters: None,
        })
    }

    pub fn validate(&self) -> Result<(), Error> {
        validate_media_group_items(&self.media)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct SendLocationRequest {
    pub chat_id: ChatId,
    pub latitude: f64,
    pub longitude: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub horizontal_accuracy: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub live_period: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub heading: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proximity_alert_radius: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_notification: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protect_content: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_thread_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_parameters: Option<ReplyParameters>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_markup: Option<ReplyMarkup>,
}

impl SendLocationRequest {
    pub fn new(chat_id: impl Into<ChatId>, latitude: f64, longitude: f64) -> Self {
        Self {
            chat_id: chat_id.into(),
            latitude,
            longitude,
            horizontal_accuracy: None,
            live_period: None,
            heading: None,
            proximity_alert_radius: None,
            disable_notification: None,
            protect_content: None,
            message_thread_id: None,
            reply_parameters: None,
            reply_markup: None,
        }
    }

    pub fn validate(&self) -> Result<(), Error> {
        validate_coordinates(self.latitude, self.longitude)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct SendVenueRequest {
    pub chat_id: ChatId,
    pub latitude: f64,
    pub longitude: f64,
    pub title: String,
    pub address: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub foursquare_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub foursquare_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub google_place_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub google_place_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_notification: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protect_content: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_thread_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_parameters: Option<ReplyParameters>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_markup: Option<ReplyMarkup>,
}

impl SendVenueRequest {
    pub fn new(
        chat_id: impl Into<ChatId>,
        latitude: f64,
        longitude: f64,
        title: impl Into<String>,
        address: impl Into<String>,
    ) -> Self {
        Self {
            chat_id: chat_id.into(),
            latitude,
            longitude,
            title: title.into(),
            address: address.into(),
            foursquare_id: None,
            foursquare_type: None,
            google_place_id: None,
            google_place_type: None,
            disable_notification: None,
            protect_content: None,
            message_thread_id: None,
            reply_parameters: None,
            reply_markup: None,
        }
    }

    pub fn validate(&self) -> Result<(), Error> {
        validate_coordinates(self.latitude, self.longitude)?;
        validate_required_text("venue title", &self.title)?;
        validate_required_text("venue address", &self.address)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct SendContactRequest {
    pub chat_id: ChatId,
    pub phone_number: String,
    pub first_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vcard: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_notification: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protect_content: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_thread_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_parameters: Option<ReplyParameters>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_markup: Option<ReplyMarkup>,
}

impl SendContactRequest {
    pub fn new(
        chat_id: impl Into<ChatId>,
        phone_number: impl Into<String>,
        first_name: impl Into<String>,
    ) -> Self {
        Self {
            chat_id: chat_id.into(),
            phone_number: phone_number.into(),
            first_name: first_name.into(),
            last_name: None,
            vcard: None,
            disable_notification: None,
            protect_content: None,
            message_thread_id: None,
            reply_parameters: None,
            reply_markup: None,
        }
    }

    pub fn validate(&self) -> Result<(), Error> {
        validate_required_text("phone_number", &self.phone_number)?;
        validate_required_text("first_name", &self.first_name)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct SendPollRequest {
    pub chat_id: ChatId,
    pub question: String,
    pub options: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_anonymous: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allows_multiple_answers: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub correct_option_id: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub explanation: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub explanation_parse_mode: Option<ParseMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub open_period: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub close_date: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_closed: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_notification: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protect_content: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_thread_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_parameters: Option<ReplyParameters>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_markup: Option<ReplyMarkup>,
}

impl SendPollRequest {
    pub fn new(
        chat_id: impl Into<ChatId>,
        question: impl Into<String>,
        options: Vec<String>,
    ) -> Result<Self, Error> {
        let question = question.into();
        validate_poll(&question, &options, None, None)?;

        Ok(Self {
            chat_id: chat_id.into(),
            question,
            options,
            is_anonymous: None,
            r#type: None,
            allows_multiple_answers: None,
            correct_option_id: None,
            explanation: None,
            explanation_parse_mode: None,
            open_period: None,
            close_date: None,
            is_closed: None,
            disable_notification: None,
            protect_content: None,
            message_thread_id: None,
            reply_parameters: None,
            reply_markup: None,
        })
    }

    pub fn validate(&self) -> Result<(), Error> {
        validate_poll(
            &self.question,
            &self.options,
            self.correct_option_id,
            self.open_period,
        )
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct StopPollRequest {
    pub chat_id: ChatId,
    pub message_id: MessageId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_markup: Option<ReplyMarkup>,
}

impl StopPollRequest {
    pub fn new(chat_id: impl Into<ChatId>, message_id: MessageId) -> Self {
        Self {
            chat_id: chat_id.into(),
            message_id,
            reply_markup: None,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct SendDiceRequest {
    pub chat_id: ChatId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub emoji: Option<DiceEmoji>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_notification: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protect_content: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_thread_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_parameters: Option<ReplyParameters>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_markup: Option<ReplyMarkup>,
}

impl SendDiceRequest {
    pub fn new(chat_id: impl Into<ChatId>) -> Self {
        Self {
            chat_id: chat_id.into(),
            emoji: None,
            disable_notification: None,
            protect_content: None,
            message_thread_id: None,
            reply_parameters: None,
            reply_markup: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ChatAction {
    Typing,
    UploadPhoto,
    RecordVideo,
    UploadVideo,
    RecordVoice,
    UploadVoice,
    UploadDocument,
    ChooseSticker,
    FindLocation,
    RecordVideoNote,
    UploadVideoNote,
}

#[derive(Clone, Debug, Serialize)]
pub struct SendChatActionRequest {
    pub chat_id: ChatId,
    pub action: ChatAction,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_thread_id: Option<i64>,
}

impl SendChatActionRequest {
    pub fn new(chat_id: impl Into<ChatId>, action: ChatAction) -> Self {
        Self {
            chat_id: chat_id.into(),
            action,
            message_thread_id: None,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct EditMessageTextRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chat_id: Option<ChatId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_id: Option<MessageId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inline_message_id: Option<String>,
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parse_mode: Option<ParseMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_markup: Option<ReplyMarkup>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub link_preview_options: Option<LinkPreviewOptions>,
}

impl EditMessageTextRequest {
    pub fn for_chat_message(
        chat_id: impl Into<ChatId>,
        message_id: MessageId,
        text: impl Into<String>,
    ) -> Result<Self, Error> {
        let text = text.into();
        validate_message_text("editMessageText", &text)?;

        Ok(Self {
            chat_id: Some(chat_id.into()),
            message_id: Some(message_id),
            inline_message_id: None,
            text,
            parse_mode: None,
            reply_markup: None,
            link_preview_options: None,
        })
    }

    pub fn for_inline_message(
        inline_message_id: impl Into<String>,
        text: impl Into<String>,
    ) -> Result<Self, Error> {
        let inline_message_id = inline_message_id.into();
        if inline_message_id.trim().is_empty() {
            return Err(Error::InvalidRequest {
                reason: "inline_message_id cannot be empty".to_owned(),
            });
        }

        let text = text.into();
        validate_message_text("editMessageText", &text)?;

        Ok(Self {
            chat_id: None,
            message_id: None,
            inline_message_id: Some(inline_message_id),
            text,
            parse_mode: None,
            reply_markup: None,
            link_preview_options: None,
        })
    }

    pub fn validate(&self) -> Result<(), Error> {
        validate_edit_target(
            self.chat_id.is_some() && self.message_id.is_some(),
            &self.inline_message_id,
        )?;

        validate_message_text("editMessageText", &self.text)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct EditMessageCaptionRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chat_id: Option<ChatId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_id: Option<MessageId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inline_message_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parse_mode: Option<ParseMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_markup: Option<ReplyMarkup>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub show_caption_above_media: Option<bool>,
}

impl EditMessageCaptionRequest {
    pub fn validate(&self) -> Result<(), Error> {
        validate_edit_target(
            self.chat_id.is_some() && self.message_id.is_some(),
            &self.inline_message_id,
        )
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct EditMessageReplyMarkupRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chat_id: Option<ChatId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_id: Option<MessageId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inline_message_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_markup: Option<ReplyMarkup>,
}

impl EditMessageReplyMarkupRequest {
    pub fn validate(&self) -> Result<(), Error> {
        validate_edit_target(
            self.chat_id.is_some() && self.message_id.is_some(),
            &self.inline_message_id,
        )
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct EditMessageLiveLocationRequest {
    pub latitude: f64,
    pub longitude: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chat_id: Option<ChatId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_id: Option<MessageId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inline_message_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub live_period: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub horizontal_accuracy: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub heading: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proximity_alert_radius: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_markup: Option<ReplyMarkup>,
}

impl EditMessageLiveLocationRequest {
    pub fn validate(&self) -> Result<(), Error> {
        validate_edit_target(
            self.chat_id.is_some() && self.message_id.is_some(),
            &self.inline_message_id,
        )
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct StopMessageLiveLocationRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chat_id: Option<ChatId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_id: Option<MessageId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inline_message_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_markup: Option<ReplyMarkup>,
}

impl StopMessageLiveLocationRequest {
    pub fn validate(&self) -> Result<(), Error> {
        validate_edit_target(
            self.chat_id.is_some() && self.message_id.is_some(),
            &self.inline_message_id,
        )
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum EditMessageResult {
    Message(Box<Message>),
    Success(bool),
}

impl EditMessageResult {
    pub fn message(&self) -> Option<&Message> {
        match self {
            Self::Message(message) => Some(message.as_ref()),
            Self::Success(_) => None,
        }
    }

    pub fn into_message(self) -> Option<Message> {
        match self {
            Self::Message(message) => Some(*message),
            Self::Success(_) => None,
        }
    }

    pub fn success(&self) -> Option<bool> {
        match self {
            Self::Message(_) => None,
            Self::Success(success) => Some(*success),
        }
    }
}

impl From<Message> for EditMessageResult {
    fn from(value: Message) -> Self {
        Self::Message(Box::new(value))
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct DeleteMessageRequest {
    pub chat_id: ChatId,
    pub message_id: MessageId,
}

impl DeleteMessageRequest {
    pub fn new(chat_id: impl Into<ChatId>, message_id: MessageId) -> Self {
        Self {
            chat_id: chat_id.into(),
            message_id,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct DeleteMessagesRequest {
    pub chat_id: ChatId,
    pub message_ids: Vec<MessageId>,
}

impl DeleteMessagesRequest {
    pub fn new(chat_id: impl Into<ChatId>, message_ids: Vec<MessageId>) -> Result<Self, Error> {
        if message_ids.is_empty() {
            return Err(Error::InvalidRequest {
                reason: "deleteMessages requires at least one message id".to_owned(),
            });
        }
        if message_ids.len() > MAX_BULK_MESSAGE_IDS {
            return Err(Error::InvalidRequest {
                reason: format!(
                    "deleteMessages accepts at most {MAX_BULK_MESSAGE_IDS} message ids"
                ),
            });
        }

        Ok(Self {
            chat_id: chat_id.into(),
            message_ids,
        })
    }

    pub fn validate(&self) -> Result<(), Error> {
        validate_bulk_message_ids("deleteMessages", &self.message_ids)
    }
}

fn validate_edit_target(
    has_chat_target: bool,
    inline_message_id: &Option<String>,
) -> Result<(), Error> {
    let has_inline_target = inline_message_id
        .as_ref()
        .is_some_and(|inline_message_id| !inline_message_id.trim().is_empty());

    if has_chat_target ^ has_inline_target {
        return Ok(());
    }

    Err(Error::InvalidRequest {
        reason: "method requires either chat_id+message_id or inline_message_id".to_owned(),
    })
}

fn validate_message_text(method: &str, text: &str) -> Result<(), Error> {
    if text.trim().is_empty() {
        return Err(Error::InvalidRequest {
            reason: format!("{method} requires non-empty text"),
        });
    }

    let length = text.chars().count();
    if length > MAX_MESSAGE_TEXT_CHARS {
        return Err(Error::InvalidRequest {
            reason: format!("{method} text exceeds {MAX_MESSAGE_TEXT_CHARS} characters"),
        });
    }

    Ok(())
}

fn validate_file_reference(label: &str, value: &str) -> Result<(), Error> {
    if value.trim().is_empty() {
        return Err(Error::InvalidRequest {
            reason: format!("{label} cannot be empty"),
        });
    }
    if value.chars().any(char::is_control) {
        return Err(Error::InvalidRequest {
            reason: format!("{label} must not contain control characters"),
        });
    }

    Ok(())
}

fn validate_optional_file_reference(label: &str, value: Option<&str>) -> Result<(), Error> {
    if let Some(value) = value {
        validate_file_reference(label, value)?;
    }
    Ok(())
}

fn validate_optional_caption(caption: Option<&str>) -> Result<(), Error> {
    let Some(caption) = caption else {
        return Ok(());
    };
    if caption.chars().count() > MAX_CAPTION_CHARS {
        return Err(Error::InvalidRequest {
            reason: format!("caption exceeds {MAX_CAPTION_CHARS} characters"),
        });
    }
    if caption.chars().any(is_disallowed_display_control) {
        return Err(Error::InvalidRequest {
            reason: "caption must not contain non-whitespace control characters".to_owned(),
        });
    }

    Ok(())
}

fn validate_media(media: &InputMedia) -> Result<(), Error> {
    match media {
        InputMedia::Photo(media) => {
            validate_file_reference("media", &media.media)?;
            validate_optional_caption(media.caption.as_deref())
        }
        InputMedia::Video(media) => {
            validate_file_reference("media", &media.media)?;
            validate_optional_caption(media.caption.as_deref())
        }
        InputMedia::Animation(media) => {
            validate_file_reference("media", &media.media)?;
            validate_optional_caption(media.caption.as_deref())
        }
        InputMedia::Audio(media) => {
            validate_file_reference("media", &media.media)?;
            validate_optional_caption(media.caption.as_deref())
        }
        InputMedia::Document(media) => {
            validate_file_reference("media", &media.media)?;
            validate_optional_caption(media.caption.as_deref())
        }
    }
}

fn validate_media_group_items(media: &[InputMedia]) -> Result<(), Error> {
    if !(MIN_MEDIA_GROUP_ITEMS..=MAX_MEDIA_GROUP_ITEMS).contains(&media.len()) {
        return Err(Error::InvalidRequest {
            reason: format!(
                "sendMediaGroup requires {MIN_MEDIA_GROUP_ITEMS}-{MAX_MEDIA_GROUP_ITEMS} media items"
            ),
        });
    }
    for item in media {
        validate_media(item)?;
    }

    Ok(())
}

fn validate_coordinates(latitude: f64, longitude: f64) -> Result<(), Error> {
    if !latitude.is_finite() || !(-90.0..=90.0).contains(&latitude) {
        return Err(Error::InvalidRequest {
            reason: "latitude must be finite and between -90 and 90".to_owned(),
        });
    }
    if !longitude.is_finite() || !(-180.0..=180.0).contains(&longitude) {
        return Err(Error::InvalidRequest {
            reason: "longitude must be finite and between -180 and 180".to_owned(),
        });
    }

    Ok(())
}

fn validate_required_text(label: &str, value: &str) -> Result<(), Error> {
    if value.trim().is_empty() {
        return Err(Error::InvalidRequest {
            reason: format!("{label} cannot be empty"),
        });
    }
    if value.chars().any(char::is_control) {
        return Err(Error::InvalidRequest {
            reason: format!("{label} must not contain control characters"),
        });
    }

    Ok(())
}

fn is_disallowed_display_control(character: char) -> bool {
    character.is_control() && !matches!(character, '\n' | '\r' | '\t')
}

fn validate_poll(
    question: &str,
    options: &[String],
    correct_option_id: Option<u8>,
    open_period: Option<u16>,
) -> Result<(), Error> {
    validate_required_text("poll question", question)?;
    if question.chars().count() > MAX_POLL_QUESTION_CHARS {
        return Err(Error::InvalidRequest {
            reason: format!("poll question exceeds {MAX_POLL_QUESTION_CHARS} characters"),
        });
    }
    if !(2..=MAX_POLL_OPTIONS).contains(&options.len()) {
        return Err(Error::InvalidRequest {
            reason: format!("sendPoll requires 2-{MAX_POLL_OPTIONS} options"),
        });
    }
    for option in options {
        validate_required_text("poll option", option)?;
        if option.chars().count() > MAX_POLL_OPTION_CHARS {
            return Err(Error::InvalidRequest {
                reason: format!("poll option exceeds {MAX_POLL_OPTION_CHARS} characters"),
            });
        }
    }
    if let Some(correct_option_id) = correct_option_id
        && usize::from(correct_option_id) >= options.len()
    {
        return Err(Error::InvalidRequest {
            reason: "correct_option_id must point to an existing poll option".to_owned(),
        });
    }
    if let Some(open_period) = open_period
        && !(MIN_POLL_OPEN_PERIOD_SECONDS..=MAX_POLL_OPEN_PERIOD_SECONDS).contains(&open_period)
    {
        return Err(Error::InvalidRequest {
            reason: format!(
                "open_period must be {MIN_POLL_OPEN_PERIOD_SECONDS}-{MAX_POLL_OPEN_PERIOD_SECONDS} seconds"
            ),
        });
    }

    Ok(())
}

fn validate_bulk_message_ids(method: &str, message_ids: &[MessageId]) -> Result<(), Error> {
    if message_ids.is_empty() {
        return Err(Error::InvalidRequest {
            reason: format!("{method} requires at least one message id"),
        });
    }
    if message_ids.len() > MAX_BULK_MESSAGE_IDS {
        return Err(Error::InvalidRequest {
            reason: format!("{method} accepts at most {MAX_BULK_MESSAGE_IDS} message ids"),
        });
    }

    Ok(())
}

macro_rules! impl_reply_markup_setter {
    ($($ty:ty),* $(,)?) => {
        $(
            impl $ty {
                pub fn reply_markup(mut self, reply_markup: impl Into<ReplyMarkup>) -> Self {
                    self.reply_markup = Some(reply_markup.into());
                    self
                }
            }
        )*
    };
}

macro_rules! impl_reply_parameters_setter {
    ($($ty:ty),* $(,)?) => {
        $(
            impl $ty {
                pub fn reply_parameters(mut self, reply_parameters: ReplyParameters) -> Self {
                    self.reply_parameters = Some(reply_parameters);
                    self
                }

                pub fn reply_to_message(mut self, message_id: MessageId) -> Self {
                    self.reply_parameters = Some(ReplyParameters::new(message_id));
                    self
                }
            }
        )*
    };
}

macro_rules! impl_link_preview_setter {
    ($($ty:ty),* $(,)?) => {
        $(
            impl $ty {
                pub fn link_preview_options(
                    mut self,
                    link_preview_options: LinkPreviewOptions,
                ) -> Self {
                    self.link_preview_options = Some(link_preview_options);
                    self
                }

                pub fn disable_link_preview(mut self) -> Self {
                    self.link_preview_options = Some(LinkPreviewOptions::disabled());
                    self
                }
            }
        )*
    };
}

impl_reply_markup_setter!(
    SendMessageRequest,
    CopyMessageRequest,
    SendPhotoRequest,
    SendAudioRequest,
    SendDocumentRequest,
    SendVideoRequest,
    SendAnimationRequest,
    SendVoiceRequest,
    SendVideoNoteRequest,
    SendLocationRequest,
    SendVenueRequest,
    SendContactRequest,
    SendPollRequest,
    SendDiceRequest,
    StopPollRequest,
    EditMessageTextRequest,
    EditMessageCaptionRequest,
    EditMessageReplyMarkupRequest,
    EditMessageLiveLocationRequest,
    StopMessageLiveLocationRequest
);

impl_reply_parameters_setter!(
    SendMessageRequest,
    CopyMessageRequest,
    SendPhotoRequest,
    SendAudioRequest,
    SendDocumentRequest,
    SendVideoRequest,
    SendAnimationRequest,
    SendVoiceRequest,
    SendVideoNoteRequest,
    SendMediaGroupRequest,
    SendLocationRequest,
    SendVenueRequest,
    SendContactRequest,
    SendPollRequest,
    SendDiceRequest
);

impl_link_preview_setter!(SendMessageRequest, EditMessageTextRequest);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_send_message_text_bounds() {
        assert!(SendMessageRequest::new(1_i64, "hello").is_ok());

        for text in ["", "   "] {
            assert!(matches!(
                SendMessageRequest::new(1_i64, text),
                Err(Error::InvalidRequest { .. })
            ));
        }

        let too_long = "a".repeat(MAX_MESSAGE_TEXT_CHARS + 1);
        assert!(matches!(
            SendMessageRequest::new(1_i64, too_long),
            Err(Error::InvalidRequest { .. })
        ));
    }

    #[test]
    fn validates_mutated_message_text_on_send_path() -> Result<(), Error> {
        let mut request = SendMessageRequest::new(1_i64, "hello")?;
        request.text.clear();
        assert!(matches!(
            request.validate(),
            Err(Error::InvalidRequest { .. })
        ));
        Ok(())
    }

    #[test]
    fn validates_edit_message_text_bounds() {
        assert!(EditMessageTextRequest::for_chat_message(1_i64, MessageId(1), "hello").is_ok());

        let too_long = "a".repeat(MAX_MESSAGE_TEXT_CHARS + 1);
        assert!(matches!(
            EditMessageTextRequest::for_chat_message(1_i64, MessageId(1), too_long),
            Err(Error::InvalidRequest { .. })
        ));
    }

    #[test]
    fn validates_media_requests() -> Result<(), Error> {
        let mut photo = SendPhotoRequest::new(1_i64, "photo-file-id");
        photo.caption = Some("a".repeat(MAX_CAPTION_CHARS + 1));
        assert!(matches!(
            photo.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let empty_photo = SendPhotoRequest::new(1_i64, "");
        assert!(matches!(
            empty_photo.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let mut multiline_caption = SendPhotoRequest::new(1_i64, "photo-file-id");
        multiline_caption.caption = Some("hello\nworld".to_owned());
        assert!(multiline_caption.validate().is_ok());

        let photo_media = InputMediaPhoto {
            media: "photo-file-id".to_owned(),
            caption: None,
            parse_mode: None,
            has_spoiler: None,
        };
        let video_media = InputMediaVideo {
            media: "video-file-id".to_owned(),
            caption: None,
            parse_mode: None,
            width: None,
            height: None,
            duration: None,
            supports_streaming: None,
            has_spoiler: None,
        };
        let group =
            SendMediaGroupRequest::new(1_i64, vec![photo_media.into(), video_media.into()])?;
        assert!(group.validate().is_ok());

        let single_item = SendMediaGroupRequest::new(
            1_i64,
            vec![
                InputMediaPhoto {
                    media: "photo-file-id".to_owned(),
                    caption: None,
                    parse_mode: None,
                    has_spoiler: None,
                }
                .into(),
            ],
        );
        assert!(matches!(single_item, Err(Error::InvalidRequest { .. })));
        Ok(())
    }

    #[test]
    fn validates_location_contact_poll_and_bulk_ids() -> Result<(), Error> {
        let invalid_location = SendLocationRequest::new(1_i64, f64::NAN, 0.0);
        assert!(matches!(
            invalid_location.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let invalid_contact = SendContactRequest::new(1_i64, "", "Alice");
        assert!(matches!(
            invalid_contact.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let invalid_poll = SendPollRequest::new(1_i64, "question", vec!["only".to_owned()]);
        assert!(matches!(invalid_poll, Err(Error::InvalidRequest { .. })));

        let mut poll =
            SendPollRequest::new(1_i64, "question", vec!["one".to_owned(), "two".to_owned()])?;
        poll.correct_option_id = Some(2);
        assert!(matches!(poll.validate(), Err(Error::InvalidRequest { .. })));

        let too_many_ids = (0..=MAX_BULK_MESSAGE_IDS)
            .map(|id| MessageId(id as i64))
            .collect::<Vec<_>>();
        assert!(matches!(
            DeleteMessagesRequest::new(1_i64, too_many_ids),
            Err(Error::InvalidRequest { .. })
        ));
        Ok(())
    }
}

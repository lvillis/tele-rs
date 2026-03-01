use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::Error;
use crate::types::bot::User;
use crate::types::common::{ChatId, MessageId, ParseMode};
use crate::types::telegram::{LinkPreviewOptions, ReplyMarkup, ReplyParameters};

/// Telegram chat type.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChatType {
    Private,
    Group,
    Supergroup,
    Channel,
}

/// Telegram chat object.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Chat {
    pub id: i64,
    #[serde(rename = "type")]
    pub kind: ChatType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_forum: Option<bool>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram message entity.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct MessageEntity {
    #[serde(rename = "type")]
    pub kind: String,
    pub offset: u32,
    pub length: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<User>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_emoji_id: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram photo size object.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct PhotoSize {
    pub file_id: String,
    pub file_unique_id: String,
    pub width: u32,
    pub height: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_size: Option<u64>,
}

/// Telegram poll option.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct PollOption {
    pub text: String,
    pub voter_count: u64,
}

/// Telegram poll object.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Poll {
    pub id: String,
    pub question: String,
    pub options: Vec<PollOption>,
    pub total_voter_count: u64,
    pub is_closed: bool,
    pub is_anonymous: bool,
    #[serde(rename = "type")]
    pub kind: String,
    pub allows_multiple_answers: bool,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram message object.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Message {
    pub message_id: MessageId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from: Option<User>,
    pub chat: Chat,
    pub date: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entities: Option<Vec<MessageEntity>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caption_entities: Option<Vec<MessageEntity>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub photo: Option<Vec<PhotoSize>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub poll: Option<Poll>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub edit_date: Option<i64>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// `sendMessage` request.
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
        let text = text.into();
        if text.trim().is_empty() {
            return Err(Error::InvalidRequest {
                reason: "sendMessage requires non-empty text".to_owned(),
            });
        }

        Ok(Self {
            chat_id: chat_id.into(),
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
}

/// `forwardMessage` request.
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

/// `copyMessage` request.
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

/// `copyMessages` request.
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
}

/// Telegram message id response object.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct MessageIdObject {
    pub message_id: MessageId,
}

/// `sendPhoto` request.
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
}

/// `sendAudio` request.
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
}

/// `sendDocument` request.
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
}

/// `sendVideo` request.
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
}

/// `sendAnimation` request.
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
}

/// `sendVoice` request.
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
}

/// `sendVideoNote` request.
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

/// Input media objects for `sendMediaGroup`.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[allow(clippy::large_enum_variant)]
pub enum InputMedia {
    Photo(InputMediaPhoto),
    Video(InputMediaVideo),
    Animation(InputMediaAnimation),
    Audio(InputMediaAudio),
    Document(InputMediaDocument),
}

/// `sendMediaGroup` request.
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
        if media.is_empty() {
            return Err(Error::InvalidRequest {
                reason: "sendMediaGroup requires at least one media item".to_owned(),
            });
        }

        Ok(Self {
            chat_id: chat_id.into(),
            media,
            disable_notification: None,
            protect_content: None,
            message_thread_id: None,
            reply_parameters: None,
        })
    }
}

/// `sendLocation` request.
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
}

/// `sendVenue` request.
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
}

/// `sendContact` request.
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
}

/// `sendPoll` request.
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
        if options.len() < 2 {
            return Err(Error::InvalidRequest {
                reason: "sendPoll requires at least two options".to_owned(),
            });
        }

        Ok(Self {
            chat_id: chat_id.into(),
            question: question.into(),
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
}

/// `stopPoll` request.
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

/// Dice emoji.
#[derive(Clone, Copy, Debug, Serialize)]
pub enum DiceEmoji {
    #[serde(rename = "🎲")]
    Dice,
    #[serde(rename = "🎯")]
    Darts,
    #[serde(rename = "🏀")]
    Basketball,
    #[serde(rename = "⚽")]
    Football,
    #[serde(rename = "🎳")]
    Bowling,
    #[serde(rename = "🎰")]
    SlotMachine,
}

/// `sendDice` request.
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

/// Chat action values.
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

/// `sendChatAction` request.
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

/// `editMessageText` request.
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
        if text.trim().is_empty() {
            return Err(Error::InvalidRequest {
                reason: "editMessageText requires non-empty text".to_owned(),
            });
        }

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
        if text.trim().is_empty() {
            return Err(Error::InvalidRequest {
                reason: "editMessageText requires non-empty text".to_owned(),
            });
        }

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

        if self.text.trim().is_empty() {
            return Err(Error::InvalidRequest {
                reason: "editMessageText requires non-empty text".to_owned(),
            });
        }

        Ok(())
    }
}

/// `editMessageCaption` request.
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

/// `editMessageReplyMarkup` request.
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

/// `editMessageLiveLocation` request.
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

/// `stopMessageLiveLocation` request.
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

/// Return type for edit message methods.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
#[allow(clippy::large_enum_variant)]
pub enum EditMessageResult {
    Message(Message),
    Success(bool),
}

/// `deleteMessage` request.
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

/// `deleteMessages` request.
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

        Ok(Self {
            chat_id: chat_id.into(),
            message_ids,
        })
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

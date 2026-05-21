use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::Error;
use crate::types::common::{ChatId, MessageId, ParseMode};
use crate::types::telegram::{
    LinkPreviewOptions, ReplyMarkup, ReplyParameters, SuggestedPostParameters,
};
use crate::types::upload::{UploadPart, validate_upload_part_name};
use crate::types::validation::{
    optional_positive_i64 as validate_optional_positive_i64, reply_markup as validate_reply_markup,
    reply_parameters as validate_reply_parameters, required_text as validate_required_text,
    suggested_post_parameters as validate_suggested_post_parameters,
};

use super::content::{DiceEmoji, PollKind};
use super::model::Message;

const MAX_MESSAGE_TEXT_CHARS: usize = 4096;
const MAX_CAPTION_CHARS: usize = 1024;
const MIN_MEDIA_GROUP_ITEMS: usize = 2;
const MAX_MEDIA_GROUP_ITEMS: usize = 10;
const MAX_BULK_MESSAGE_IDS: usize = 100;
const MAX_POLL_OPTIONS: usize = 12;
const MAX_POLL_QUESTION_CHARS: usize = 300;
const MAX_POLL_OPTION_CHARS: usize = 100;
const MIN_POLL_OPEN_PERIOD_SECONDS: u32 = 5;
const MAX_POLL_OPEN_PERIOD_SECONDS: u32 = 2_628_000;
const MIN_LIVE_LOCATION_PERIOD_SECONDS: u32 = 60;
const MAX_LIVE_LOCATION_PERIOD_SECONDS: u32 = 86_400;
const INDEFINITE_LIVE_LOCATION_PERIOD_SECONDS: u32 = 0x7FFF_FFFF;
const MAX_HORIZONTAL_ACCURACY_METERS: f64 = 1500.0;
const MIN_LOCATION_HEADING_DEGREES: u16 = 1;
const MAX_LOCATION_HEADING_DEGREES: u16 = 360;
const MIN_PROXIMITY_ALERT_RADIUS_METERS: u32 = 1;
const MAX_PROXIMITY_ALERT_RADIUS_METERS: u32 = 100_000;
const ATTACH_URI_PREFIX: &str = "attach://";

#[derive(Clone, Debug, Serialize)]
pub struct SendMessageRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub business_connection_id: Option<String>,
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
            business_connection_id: None,
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

    pub fn business_connection_id(mut self, business_connection_id: impl Into<String>) -> Self {
        self.business_connection_id = Some(business_connection_id.into());
        self
    }

    pub fn validate(&self) -> Result<(), Error> {
        validate_business_connection_id(self.business_connection_id.as_deref())?;
        self.chat_id.validate()?;
        validate_message_thread_id(self.message_thread_id)?;
        validate_reply_parameters(self.reply_parameters.as_ref())?;
        validate_reply_markup(self.reply_markup.as_ref())?;
        validate_link_preview_fields(
            self.disable_web_page_preview,
            self.link_preview_options.as_ref(),
        )?;
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

    pub fn validate(&self) -> Result<(), Error> {
        self.chat_id.validate()?;
        self.from_chat_id.validate()?;
        self.message_id.validate()?;
        validate_message_thread_id(self.message_thread_id)
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

    pub fn validate(&self) -> Result<(), Error> {
        self.chat_id.validate()?;
        self.from_chat_id.validate()?;
        self.message_id.validate()?;
        validate_message_thread_id(self.message_thread_id)?;
        validate_reply_parameters(self.reply_parameters.as_ref())?;
        validate_reply_markup(self.reply_markup.as_ref())?;
        validate_optional_caption(self.caption.as_deref())
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
        let chat_id = chat_id.into();
        chat_id.validate()?;
        let from_chat_id = from_chat_id.into();
        from_chat_id.validate()?;
        validate_bulk_message_ids("copyMessages", &message_ids)?;

        Ok(Self {
            chat_id,
            from_chat_id,
            message_ids,
            message_thread_id: None,
            disable_notification: None,
            protect_content: None,
            remove_caption: None,
        })
    }

    pub fn validate(&self) -> Result<(), Error> {
        self.chat_id.validate()?;
        self.from_chat_id.validate()?;
        validate_message_thread_id(self.message_thread_id)?;
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub business_connection_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub photo: Option<String>,
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
            business_connection_id: None,
            photo: Some(photo.into()),
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

    pub fn for_upload(chat_id: impl Into<ChatId>) -> Self {
        Self {
            chat_id: chat_id.into(),
            business_connection_id: None,
            photo: None,
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
        validate_business_connection_id(self.business_connection_id.as_deref())?;
        self.chat_id.validate()?;
        validate_message_thread_id(self.message_thread_id)?;
        validate_reply_parameters(self.reply_parameters.as_ref())?;
        validate_reply_markup(self.reply_markup.as_ref())?;
        validate_required_file_reference("photo", self.photo.as_deref())?;
        validate_optional_caption(self.caption.as_deref())
    }

    pub(crate) fn validate_upload(&self) -> Result<(), Error> {
        validate_absent_upload_field("photo", self.photo.as_deref())?;
        validate_business_connection_id(self.business_connection_id.as_deref())?;
        self.chat_id.validate()?;
        validate_message_thread_id(self.message_thread_id)?;
        validate_reply_parameters(self.reply_parameters.as_ref())?;
        validate_reply_markup(self.reply_markup.as_ref())?;
        validate_optional_caption(self.caption.as_deref())
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct SendAudioRequest {
    pub chat_id: ChatId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub business_connection_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio: Option<String>,
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
            business_connection_id: None,
            audio: Some(audio.into()),
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

    pub fn for_upload(chat_id: impl Into<ChatId>) -> Self {
        Self {
            chat_id: chat_id.into(),
            business_connection_id: None,
            audio: None,
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
        validate_business_connection_id(self.business_connection_id.as_deref())?;
        self.chat_id.validate()?;
        validate_message_thread_id(self.message_thread_id)?;
        validate_reply_parameters(self.reply_parameters.as_ref())?;
        validate_reply_markup(self.reply_markup.as_ref())?;
        validate_required_file_reference("audio", self.audio.as_deref())?;
        validate_optional_caption(self.caption.as_deref())?;
        validate_optional_file_reference("thumbnail", self.thumbnail.as_deref())?;
        validate_positive_u32("duration", self.duration)
    }

    pub(crate) fn validate_upload_parts(&self, files: &[UploadPart]) -> Result<(), Error> {
        validate_absent_upload_field("audio", self.audio.as_deref())?;
        validate_business_connection_id(self.business_connection_id.as_deref())?;
        self.chat_id.validate()?;
        validate_message_thread_id(self.message_thread_id)?;
        validate_reply_parameters(self.reply_parameters.as_ref())?;
        validate_reply_markup(self.reply_markup.as_ref())?;
        validate_optional_caption(self.caption.as_deref())?;
        validate_optional_file_reference("thumbnail", self.thumbnail.as_deref())?;
        validate_positive_u32("duration", self.duration)?;
        validate_attach_upload_parts("sendAudio", [self.thumbnail.as_deref()], files)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct SendDocumentRequest {
    pub chat_id: ChatId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub business_connection_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub document: Option<String>,
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
            business_connection_id: None,
            document: Some(document.into()),
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

    pub fn for_upload(chat_id: impl Into<ChatId>) -> Self {
        Self {
            chat_id: chat_id.into(),
            business_connection_id: None,
            document: None,
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
        validate_business_connection_id(self.business_connection_id.as_deref())?;
        self.chat_id.validate()?;
        validate_message_thread_id(self.message_thread_id)?;
        validate_reply_parameters(self.reply_parameters.as_ref())?;
        validate_reply_markup(self.reply_markup.as_ref())?;
        validate_required_file_reference("document", self.document.as_deref())?;
        validate_optional_caption(self.caption.as_deref())?;
        validate_optional_file_reference("thumbnail", self.thumbnail.as_deref())
    }

    pub(crate) fn validate_upload_parts(&self, files: &[UploadPart]) -> Result<(), Error> {
        validate_absent_upload_field("document", self.document.as_deref())?;
        validate_business_connection_id(self.business_connection_id.as_deref())?;
        self.chat_id.validate()?;
        validate_message_thread_id(self.message_thread_id)?;
        validate_reply_parameters(self.reply_parameters.as_ref())?;
        validate_reply_markup(self.reply_markup.as_ref())?;
        validate_optional_caption(self.caption.as_deref())?;
        validate_optional_file_reference("thumbnail", self.thumbnail.as_deref())?;
        validate_attach_upload_parts("sendDocument", [self.thumbnail.as_deref()], files)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct SendVideoRequest {
    pub chat_id: ChatId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub business_connection_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub video: Option<String>,
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
            business_connection_id: None,
            video: Some(video.into()),
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

    pub fn for_upload(chat_id: impl Into<ChatId>) -> Self {
        Self {
            chat_id: chat_id.into(),
            business_connection_id: None,
            video: None,
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
        validate_business_connection_id(self.business_connection_id.as_deref())?;
        self.chat_id.validate()?;
        validate_message_thread_id(self.message_thread_id)?;
        validate_reply_parameters(self.reply_parameters.as_ref())?;
        validate_reply_markup(self.reply_markup.as_ref())?;
        validate_required_file_reference("video", self.video.as_deref())?;
        validate_optional_caption(self.caption.as_deref())?;
        validate_optional_file_reference("thumbnail", self.thumbnail.as_deref())?;
        validate_positive_u32("duration", self.duration)?;
        validate_positive_u32("width", self.width)?;
        validate_positive_u32("height", self.height)
    }

    pub(crate) fn validate_upload_parts(&self, files: &[UploadPart]) -> Result<(), Error> {
        validate_absent_upload_field("video", self.video.as_deref())?;
        validate_business_connection_id(self.business_connection_id.as_deref())?;
        self.chat_id.validate()?;
        validate_message_thread_id(self.message_thread_id)?;
        validate_reply_parameters(self.reply_parameters.as_ref())?;
        validate_reply_markup(self.reply_markup.as_ref())?;
        validate_optional_caption(self.caption.as_deref())?;
        validate_optional_file_reference("thumbnail", self.thumbnail.as_deref())?;
        validate_positive_u32("duration", self.duration)?;
        validate_positive_u32("width", self.width)?;
        validate_positive_u32("height", self.height)?;
        validate_attach_upload_parts("sendVideo", [self.thumbnail.as_deref()], files)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct SendAnimationRequest {
    pub chat_id: ChatId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub business_connection_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub animation: Option<String>,
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
            business_connection_id: None,
            animation: Some(animation.into()),
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

    pub fn for_upload(chat_id: impl Into<ChatId>) -> Self {
        Self {
            chat_id: chat_id.into(),
            business_connection_id: None,
            animation: None,
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
        validate_business_connection_id(self.business_connection_id.as_deref())?;
        self.chat_id.validate()?;
        validate_message_thread_id(self.message_thread_id)?;
        validate_reply_parameters(self.reply_parameters.as_ref())?;
        validate_reply_markup(self.reply_markup.as_ref())?;
        validate_required_file_reference("animation", self.animation.as_deref())?;
        validate_optional_caption(self.caption.as_deref())?;
        validate_optional_file_reference("thumbnail", self.thumbnail.as_deref())?;
        validate_positive_u32("duration", self.duration)?;
        validate_positive_u32("width", self.width)?;
        validate_positive_u32("height", self.height)
    }

    pub(crate) fn validate_upload_parts(&self, files: &[UploadPart]) -> Result<(), Error> {
        validate_absent_upload_field("animation", self.animation.as_deref())?;
        validate_business_connection_id(self.business_connection_id.as_deref())?;
        self.chat_id.validate()?;
        validate_message_thread_id(self.message_thread_id)?;
        validate_reply_parameters(self.reply_parameters.as_ref())?;
        validate_reply_markup(self.reply_markup.as_ref())?;
        validate_optional_caption(self.caption.as_deref())?;
        validate_optional_file_reference("thumbnail", self.thumbnail.as_deref())?;
        validate_positive_u32("duration", self.duration)?;
        validate_positive_u32("width", self.width)?;
        validate_positive_u32("height", self.height)?;
        validate_attach_upload_parts("sendAnimation", [self.thumbnail.as_deref()], files)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct SendVoiceRequest {
    pub chat_id: ChatId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub business_connection_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub voice: Option<String>,
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
            business_connection_id: None,
            voice: Some(voice.into()),
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

    pub fn for_upload(chat_id: impl Into<ChatId>) -> Self {
        Self {
            chat_id: chat_id.into(),
            business_connection_id: None,
            voice: None,
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
        validate_business_connection_id(self.business_connection_id.as_deref())?;
        self.chat_id.validate()?;
        validate_message_thread_id(self.message_thread_id)?;
        validate_reply_parameters(self.reply_parameters.as_ref())?;
        validate_reply_markup(self.reply_markup.as_ref())?;
        validate_required_file_reference("voice", self.voice.as_deref())?;
        validate_optional_caption(self.caption.as_deref())?;
        validate_positive_u32("duration", self.duration)
    }

    pub(crate) fn validate_upload(&self) -> Result<(), Error> {
        validate_absent_upload_field("voice", self.voice.as_deref())?;
        validate_business_connection_id(self.business_connection_id.as_deref())?;
        self.chat_id.validate()?;
        validate_message_thread_id(self.message_thread_id)?;
        validate_reply_parameters(self.reply_parameters.as_ref())?;
        validate_reply_markup(self.reply_markup.as_ref())?;
        validate_optional_caption(self.caption.as_deref())?;
        validate_positive_u32("duration", self.duration)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct SendVideoNoteRequest {
    pub chat_id: ChatId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub business_connection_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub video_note: Option<String>,
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
    pub direct_messages_topic_id: Option<i64>,
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

impl SendVideoNoteRequest {
    pub fn new(chat_id: impl Into<ChatId>, video_note: impl Into<String>) -> Self {
        Self {
            chat_id: chat_id.into(),
            business_connection_id: None,
            video_note: Some(video_note.into()),
            duration: None,
            length: None,
            thumbnail: None,
            disable_notification: None,
            protect_content: None,
            message_thread_id: None,
            direct_messages_topic_id: None,
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
            business_connection_id: None,
            video_note: None,
            duration: None,
            length: None,
            thumbnail: None,
            disable_notification: None,
            protect_content: None,
            message_thread_id: None,
            direct_messages_topic_id: None,
            allow_paid_broadcast: None,
            message_effect_id: None,
            suggested_post_parameters: None,
            reply_parameters: None,
            reply_markup: None,
        }
    }

    pub fn validate(&self) -> Result<(), Error> {
        validate_business_connection_id(self.business_connection_id.as_deref())?;
        self.chat_id.validate()?;
        validate_message_thread_id(self.message_thread_id)?;
        validate_direct_messages_topic_id(self.direct_messages_topic_id)?;
        validate_message_effect_id(self.message_effect_id.as_deref())?;
        validate_suggested_post_parameters(self.suggested_post_parameters.as_ref())?;
        validate_reply_parameters(self.reply_parameters.as_ref())?;
        validate_reply_markup(self.reply_markup.as_ref())?;
        validate_required_file_reference("video_note", self.video_note.as_deref())?;
        validate_optional_file_reference("thumbnail", self.thumbnail.as_deref())?;
        validate_positive_u32("duration", self.duration)?;
        validate_positive_u32("length", self.length)
    }

    pub(crate) fn validate_upload_parts(&self, files: &[UploadPart]) -> Result<(), Error> {
        validate_absent_upload_field("video_note", self.video_note.as_deref())?;
        validate_business_connection_id(self.business_connection_id.as_deref())?;
        self.chat_id.validate()?;
        validate_message_thread_id(self.message_thread_id)?;
        validate_direct_messages_topic_id(self.direct_messages_topic_id)?;
        validate_message_effect_id(self.message_effect_id.as_deref())?;
        validate_suggested_post_parameters(self.suggested_post_parameters.as_ref())?;
        validate_reply_parameters(self.reply_parameters.as_ref())?;
        validate_reply_markup(self.reply_markup.as_ref())?;
        validate_optional_file_reference("thumbnail", self.thumbnail.as_deref())?;
        validate_positive_u32("duration", self.duration)?;
        validate_positive_u32("length", self.length)?;
        validate_attach_upload_parts("sendVideoNote", [self.thumbnail.as_deref()], files)
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

impl InputMediaPhoto {
    pub fn new(media: impl Into<String>) -> Self {
        Self {
            media: media.into(),
            caption: None,
            parse_mode: None,
            has_spoiler: None,
        }
    }

    pub fn caption(mut self, caption: impl Into<String>) -> Self {
        self.caption = Some(caption.into());
        self
    }

    pub fn parse_mode(mut self, parse_mode: ParseMode) -> Self {
        self.parse_mode = Some(parse_mode);
        self
    }

    pub fn has_spoiler(mut self, enabled: bool) -> Self {
        self.has_spoiler = enabled.then_some(true);
        self
    }
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

impl InputMediaVideo {
    pub fn new(media: impl Into<String>) -> Self {
        Self {
            media: media.into(),
            caption: None,
            parse_mode: None,
            width: None,
            height: None,
            duration: None,
            supports_streaming: None,
            has_spoiler: None,
        }
    }

    pub fn caption(mut self, caption: impl Into<String>) -> Self {
        self.caption = Some(caption.into());
        self
    }

    pub fn parse_mode(mut self, parse_mode: ParseMode) -> Self {
        self.parse_mode = Some(parse_mode);
        self
    }

    pub fn width(mut self, width: u32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn height(mut self, height: u32) -> Self {
        self.height = Some(height);
        self
    }

    pub fn duration(mut self, duration: u32) -> Self {
        self.duration = Some(duration);
        self
    }

    pub fn supports_streaming(mut self, enabled: bool) -> Self {
        self.supports_streaming = enabled.then_some(true);
        self
    }

    pub fn has_spoiler(mut self, enabled: bool) -> Self {
        self.has_spoiler = enabled.then_some(true);
        self
    }
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

impl InputMediaAnimation {
    pub fn new(media: impl Into<String>) -> Self {
        Self {
            media: media.into(),
            caption: None,
            parse_mode: None,
            width: None,
            height: None,
            duration: None,
            has_spoiler: None,
        }
    }

    pub fn caption(mut self, caption: impl Into<String>) -> Self {
        self.caption = Some(caption.into());
        self
    }

    pub fn parse_mode(mut self, parse_mode: ParseMode) -> Self {
        self.parse_mode = Some(parse_mode);
        self
    }

    pub fn width(mut self, width: u32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn height(mut self, height: u32) -> Self {
        self.height = Some(height);
        self
    }

    pub fn duration(mut self, duration: u32) -> Self {
        self.duration = Some(duration);
        self
    }

    pub fn has_spoiler(mut self, enabled: bool) -> Self {
        self.has_spoiler = enabled.then_some(true);
        self
    }
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

impl InputMediaAudio {
    pub fn new(media: impl Into<String>) -> Self {
        Self {
            media: media.into(),
            caption: None,
            parse_mode: None,
            duration: None,
            performer: None,
            title: None,
        }
    }

    pub fn caption(mut self, caption: impl Into<String>) -> Self {
        self.caption = Some(caption.into());
        self
    }

    pub fn parse_mode(mut self, parse_mode: ParseMode) -> Self {
        self.parse_mode = Some(parse_mode);
        self
    }

    pub fn duration(mut self, duration: u32) -> Self {
        self.duration = Some(duration);
        self
    }

    pub fn performer(mut self, performer: impl Into<String>) -> Self {
        self.performer = Some(performer.into());
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }
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

impl InputMediaDocument {
    pub fn new(media: impl Into<String>) -> Self {
        Self {
            media: media.into(),
            caption: None,
            parse_mode: None,
            disable_content_type_detection: None,
        }
    }

    pub fn caption(mut self, caption: impl Into<String>) -> Self {
        self.caption = Some(caption.into());
        self
    }

    pub fn parse_mode(mut self, parse_mode: ParseMode) -> Self {
        self.parse_mode = Some(parse_mode);
        self
    }

    pub fn disable_content_type_detection(mut self, enabled: bool) -> Self {
        self.disable_content_type_detection = enabled.then_some(true);
        self
    }
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub business_connection_id: Option<String>,
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
        let chat_id = chat_id.into();
        chat_id.validate()?;
        if media.len() < MIN_MEDIA_GROUP_ITEMS {
            return Err(Error::InvalidRequest {
                reason: format!(
                    "sendMediaGroup requires at least {MIN_MEDIA_GROUP_ITEMS} media items"
                ),
            });
        }
        validate_media_group_items(&media)?;

        Ok(Self {
            chat_id,
            business_connection_id: None,
            media,
            disable_notification: None,
            protect_content: None,
            message_thread_id: None,
            reply_parameters: None,
        })
    }

    pub fn validate(&self) -> Result<(), Error> {
        validate_business_connection_id(self.business_connection_id.as_deref())?;
        self.chat_id.validate()?;
        validate_message_thread_id(self.message_thread_id)?;
        validate_reply_parameters(self.reply_parameters.as_ref())?;
        validate_media_group_items(&self.media)?;
        validate_no_multipart_attach_references(&self.media)
    }

    pub(crate) fn validate_upload(&self, files: &[UploadPart]) -> Result<(), Error> {
        validate_business_connection_id(self.business_connection_id.as_deref())?;
        self.chat_id.validate()?;
        validate_message_thread_id(self.message_thread_id)?;
        validate_reply_parameters(self.reply_parameters.as_ref())?;
        validate_media_group_items(&self.media)?;
        validate_media_group_upload_parts(&self.media, files)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct SendLocationRequest {
    pub chat_id: ChatId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub business_connection_id: Option<String>,
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
            business_connection_id: None,
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
        validate_business_connection_id(self.business_connection_id.as_deref())?;
        self.chat_id.validate()?;
        validate_message_thread_id(self.message_thread_id)?;
        validate_reply_parameters(self.reply_parameters.as_ref())?;
        validate_reply_markup(self.reply_markup.as_ref())?;
        validate_coordinates(self.latitude, self.longitude)?;
        validate_location_options(
            self.horizontal_accuracy,
            self.live_period,
            self.heading,
            self.proximity_alert_radius,
        )
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct SendVenueRequest {
    pub chat_id: ChatId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub business_connection_id: Option<String>,
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
            business_connection_id: None,
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
        validate_business_connection_id(self.business_connection_id.as_deref())?;
        self.chat_id.validate()?;
        validate_message_thread_id(self.message_thread_id)?;
        validate_reply_parameters(self.reply_parameters.as_ref())?;
        validate_reply_markup(self.reply_markup.as_ref())?;
        validate_coordinates(self.latitude, self.longitude)?;
        validate_required_text("venue title", &self.title)?;
        validate_required_text("venue address", &self.address)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct SendContactRequest {
    pub chat_id: ChatId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub business_connection_id: Option<String>,
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
            business_connection_id: None,
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
        validate_business_connection_id(self.business_connection_id.as_deref())?;
        self.chat_id.validate()?;
        validate_message_thread_id(self.message_thread_id)?;
        validate_reply_parameters(self.reply_parameters.as_ref())?;
        validate_reply_markup(self.reply_markup.as_ref())?;
        validate_required_text("phone_number", &self.phone_number)?;
        validate_required_text("first_name", &self.first_name)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct SendPollRequest {
    pub chat_id: ChatId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub business_connection_id: Option<String>,
    pub question: String,
    pub options: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_anonymous: Option<bool>,
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<PollKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allows_multiple_answers: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub correct_option_id: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub explanation: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub explanation_parse_mode: Option<ParseMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub open_period: Option<u32>,
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
        let chat_id = chat_id.into();
        chat_id.validate()?;
        let question = question.into();
        validate_poll(&question, &options, None, None, None)?;

        Ok(Self {
            chat_id,
            business_connection_id: None,
            question,
            options,
            is_anonymous: None,
            kind: None,
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
        validate_business_connection_id(self.business_connection_id.as_deref())?;
        self.chat_id.validate()?;
        validate_message_thread_id(self.message_thread_id)?;
        validate_reply_parameters(self.reply_parameters.as_ref())?;
        validate_reply_markup(self.reply_markup.as_ref())?;
        validate_poll(
            &self.question,
            &self.options,
            self.correct_option_id,
            self.open_period,
            self.kind.as_ref(),
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

    pub fn validate(&self) -> Result<(), Error> {
        self.chat_id.validate()?;
        self.message_id.validate()?;
        validate_reply_markup(self.reply_markup.as_ref())
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct SendDiceRequest {
    pub chat_id: ChatId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub business_connection_id: Option<String>,
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
            business_connection_id: None,
            emoji: None,
            disable_notification: None,
            protect_content: None,
            message_thread_id: None,
            reply_parameters: None,
            reply_markup: None,
        }
    }

    pub fn validate(&self) -> Result<(), Error> {
        validate_business_connection_id(self.business_connection_id.as_deref())?;
        self.chat_id.validate()?;
        validate_message_thread_id(self.message_thread_id)?;
        validate_reply_parameters(self.reply_parameters.as_ref())?;
        validate_reply_markup(self.reply_markup.as_ref())
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub business_connection_id: Option<String>,
    pub action: ChatAction,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_thread_id: Option<i64>,
}

impl SendChatActionRequest {
    pub fn new(chat_id: impl Into<ChatId>, action: ChatAction) -> Self {
        Self {
            chat_id: chat_id.into(),
            business_connection_id: None,
            action,
            message_thread_id: None,
        }
    }

    pub fn validate(&self) -> Result<(), Error> {
        validate_business_connection_id(self.business_connection_id.as_deref())?;
        self.chat_id.validate()?;
        validate_message_thread_id(self.message_thread_id)
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
        let chat_id = chat_id.into();
        chat_id.validate()?;
        message_id.validate()?;
        let text = text.into();
        validate_message_text("editMessageText", &text)?;

        Ok(Self {
            chat_id: Some(chat_id),
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
            self.chat_id.as_ref(),
            self.message_id,
            &self.inline_message_id,
        )?;

        validate_reply_markup(self.reply_markup.as_ref())?;
        validate_link_preview_options(self.link_preview_options.as_ref())?;
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
            self.chat_id.as_ref(),
            self.message_id,
            &self.inline_message_id,
        )?;
        validate_reply_markup(self.reply_markup.as_ref())?;
        validate_optional_caption(self.caption.as_deref())
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
            self.chat_id.as_ref(),
            self.message_id,
            &self.inline_message_id,
        )?;
        validate_reply_markup(self.reply_markup.as_ref())
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
            self.chat_id.as_ref(),
            self.message_id,
            &self.inline_message_id,
        )?;
        validate_reply_markup(self.reply_markup.as_ref())?;
        validate_coordinates(self.latitude, self.longitude)?;
        validate_location_options(
            self.horizontal_accuracy,
            self.live_period,
            self.heading,
            self.proximity_alert_radius,
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
            self.chat_id.as_ref(),
            self.message_id,
            &self.inline_message_id,
        )?;
        validate_reply_markup(self.reply_markup.as_ref())
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

    pub fn validate(&self) -> Result<(), Error> {
        self.chat_id.validate()?;
        self.message_id.validate()
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct DeleteMessagesRequest {
    pub chat_id: ChatId,
    pub message_ids: Vec<MessageId>,
}

impl DeleteMessagesRequest {
    pub fn new(chat_id: impl Into<ChatId>, message_ids: Vec<MessageId>) -> Result<Self, Error> {
        let chat_id = chat_id.into();
        chat_id.validate()?;
        validate_bulk_message_ids("deleteMessages", &message_ids)?;

        Ok(Self {
            chat_id,
            message_ids,
        })
    }

    pub fn validate(&self) -> Result<(), Error> {
        self.chat_id.validate()?;
        validate_bulk_message_ids("deleteMessages", &self.message_ids)
    }
}

fn validate_edit_target(
    chat_id: Option<&ChatId>,
    message_id: Option<MessageId>,
    inline_message_id: &Option<String>,
) -> Result<(), Error> {
    if let Some(chat_id) = chat_id {
        chat_id.validate()?;
    }
    if let Some(message_id) = message_id {
        message_id.validate()?;
    }

    let has_chat_target = chat_id.is_some() && message_id.is_some();
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

fn validate_absent_upload_field(label: &str, value: Option<&str>) -> Result<(), Error> {
    if value.is_some() {
        return Err(Error::InvalidRequest {
            reason: format!("{label} must be omitted for multipart upload requests"),
        });
    }

    Ok(())
}

fn validate_required_file_reference(label: &str, value: Option<&str>) -> Result<(), Error> {
    let Some(value) = value else {
        return Err(Error::InvalidRequest {
            reason: format!("{label} is required"),
        });
    };

    validate_file_reference(label, value)
}

fn validate_optional_file_reference(label: &str, value: Option<&str>) -> Result<(), Error> {
    if let Some(value) = value {
        validate_file_reference(label, value)?;
    }
    Ok(())
}

fn validate_attach_upload_parts<'a>(
    method: &str,
    references: impl IntoIterator<Item = Option<&'a str>>,
    files: &[UploadPart],
) -> Result<(), Error> {
    let mut attach_names = BTreeSet::new();
    for reference in references.into_iter().flatten() {
        if let Some(name) = attach_name(reference) {
            validate_attach_name("upload attach name", name)?;
            attach_names.insert(name.to_owned());
        }
    }

    if attach_names.is_empty() {
        if files.is_empty() {
            return Ok(());
        }
        return Err(Error::InvalidRequest {
            reason: format!("{method} upload has file parts that are not referenced by attach://"),
        });
    }

    let mut file_names = BTreeSet::new();
    for file in files {
        if !file_names.insert(file.field_name().to_owned()) {
            return Err(Error::InvalidRequest {
                reason: format!("duplicate multipart file field `{}`", file.field_name()),
            });
        }
    }

    for name in &attach_names {
        if !file_names.contains(name) {
            return Err(Error::InvalidRequest {
                reason: format!("missing multipart file part for attach://{name}"),
            });
        }
    }

    for name in &file_names {
        if !attach_names.contains(name) {
            return Err(Error::InvalidRequest {
                reason: format!("multipart file part `{name}` is not referenced by attach://"),
            });
        }
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
            validate_optional_caption(media.caption.as_deref())?;
            validate_positive_u32("width", media.width)?;
            validate_positive_u32("height", media.height)?;
            validate_positive_u32("duration", media.duration)
        }
        InputMedia::Animation(media) => {
            validate_file_reference("media", &media.media)?;
            validate_optional_caption(media.caption.as_deref())?;
            validate_positive_u32("width", media.width)?;
            validate_positive_u32("height", media.height)?;
            validate_positive_u32("duration", media.duration)
        }
        InputMedia::Audio(media) => {
            validate_file_reference("media", &media.media)?;
            validate_optional_caption(media.caption.as_deref())?;
            validate_positive_u32("duration", media.duration)
        }
        InputMedia::Document(media) => {
            validate_file_reference("media", &media.media)?;
            validate_optional_caption(media.caption.as_deref())
        }
    }
}

fn media_file_reference(media: &InputMedia) -> &str {
    match media {
        InputMedia::Photo(media) => &media.media,
        InputMedia::Video(media) => &media.media,
        InputMedia::Animation(media) => &media.media,
        InputMedia::Audio(media) => &media.media,
        InputMedia::Document(media) => &media.media,
    }
}

fn attach_name(value: &str) -> Option<&str> {
    value.strip_prefix(ATTACH_URI_PREFIX)
}

fn validate_attach_name(field: &str, name: &str) -> Result<(), Error> {
    validate_upload_part_name(field, name)
}

fn media_attach_names(media: &[InputMedia]) -> Result<BTreeSet<String>, Error> {
    let mut names = BTreeSet::new();
    for item in media {
        if let Some(name) = attach_name(media_file_reference(item)) {
            validate_attach_name("media attach name", name)?;
            names.insert(name.to_owned());
        }
    }

    Ok(names)
}

fn validate_no_multipart_attach_references(media: &[InputMedia]) -> Result<(), Error> {
    for item in media {
        if attach_name(media_file_reference(item)).is_some() {
            return Err(Error::InvalidRequest {
                reason: "sendMediaGroup JSON requests cannot use attach:// media; use send_media_group_upload".to_owned(),
            });
        }
    }

    Ok(())
}

fn validate_media_group_upload_parts(
    media: &[InputMedia],
    files: &[UploadPart],
) -> Result<(), Error> {
    if files.is_empty() {
        return Err(Error::InvalidRequest {
            reason: "sendMediaGroup upload requires at least one file part".to_owned(),
        });
    }

    let attach_names = media_attach_names(media)?;
    if attach_names.is_empty() {
        return Err(Error::InvalidRequest {
            reason: "sendMediaGroup upload requires at least one attach:// media reference"
                .to_owned(),
        });
    }

    let mut file_names = BTreeSet::new();
    for file in files {
        if !file_names.insert(file.field_name().to_owned()) {
            return Err(Error::InvalidRequest {
                reason: format!("duplicate multipart file field `{}`", file.field_name()),
            });
        }
    }

    for name in &attach_names {
        if !file_names.contains(name) {
            return Err(Error::InvalidRequest {
                reason: format!("missing multipart file part for attach://{name}"),
            });
        }
    }

    for name in &file_names {
        if !attach_names.contains(name) {
            return Err(Error::InvalidRequest {
                reason: format!("multipart file part `{name}` is not referenced by media"),
            });
        }
    }

    Ok(())
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

fn validate_location_options(
    horizontal_accuracy: Option<f64>,
    live_period: Option<u32>,
    heading: Option<u16>,
    proximity_alert_radius: Option<u32>,
) -> Result<(), Error> {
    if let Some(horizontal_accuracy) = horizontal_accuracy
        && (!horizontal_accuracy.is_finite()
            || !(0.0..=MAX_HORIZONTAL_ACCURACY_METERS).contains(&horizontal_accuracy))
    {
        return Err(Error::InvalidRequest {
            reason: format!(
                "horizontal_accuracy must be finite and between 0 and {MAX_HORIZONTAL_ACCURACY_METERS} meters"
            ),
        });
    }

    if let Some(live_period) = live_period
        && live_period != INDEFINITE_LIVE_LOCATION_PERIOD_SECONDS
        && !(MIN_LIVE_LOCATION_PERIOD_SECONDS..=MAX_LIVE_LOCATION_PERIOD_SECONDS)
            .contains(&live_period)
    {
        return Err(Error::InvalidRequest {
            reason: format!(
                "live_period must be {MIN_LIVE_LOCATION_PERIOD_SECONDS}-{MAX_LIVE_LOCATION_PERIOD_SECONDS} seconds or {INDEFINITE_LIVE_LOCATION_PERIOD_SECONDS}"
            ),
        });
    }

    if let Some(heading) = heading
        && !(MIN_LOCATION_HEADING_DEGREES..=MAX_LOCATION_HEADING_DEGREES).contains(&heading)
    {
        return Err(Error::InvalidRequest {
            reason: format!(
                "heading must be {MIN_LOCATION_HEADING_DEGREES}-{MAX_LOCATION_HEADING_DEGREES} degrees"
            ),
        });
    }

    if let Some(proximity_alert_radius) = proximity_alert_radius
        && !(MIN_PROXIMITY_ALERT_RADIUS_METERS..=MAX_PROXIMITY_ALERT_RADIUS_METERS)
            .contains(&proximity_alert_radius)
    {
        return Err(Error::InvalidRequest {
            reason: format!(
                "proximity_alert_radius must be {MIN_PROXIMITY_ALERT_RADIUS_METERS}-{MAX_PROXIMITY_ALERT_RADIUS_METERS} meters"
            ),
        });
    }

    Ok(())
}

fn validate_positive_u32(field: &str, value: Option<u32>) -> Result<(), Error> {
    if matches!(value, Some(0)) {
        return Err(Error::InvalidRequest {
            reason: format!("{field} must be greater than 0"),
        });
    }

    Ok(())
}

fn validate_message_thread_id(value: Option<i64>) -> Result<(), Error> {
    validate_optional_positive_i64("message_thread_id", value)
}

fn validate_direct_messages_topic_id(value: Option<i64>) -> Result<(), Error> {
    validate_optional_positive_i64("direct_messages_topic_id", value)
}

fn validate_business_connection_id(value: Option<&str>) -> Result<(), Error> {
    if let Some(value) = value {
        validate_required_text("business_connection_id", value)?;
    }

    Ok(())
}

fn validate_message_effect_id(value: Option<&str>) -> Result<(), Error> {
    if let Some(value) = value {
        validate_required_text("message_effect_id", value)?;
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
    open_period: Option<u32>,
    kind: Option<&PollKind>,
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
    if let Some(PollKind::Unknown(kind)) = kind {
        return Err(Error::InvalidRequest {
            reason: format!("unsupported poll type `{kind}`"),
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
    for message_id in message_ids {
        message_id.validate()?;
    }

    Ok(())
}

fn validate_link_preview_options(
    link_preview_options: Option<&LinkPreviewOptions>,
) -> Result<(), Error> {
    if let Some(link_preview_options) = link_preview_options {
        link_preview_options.validate()?;
    }

    Ok(())
}

fn validate_link_preview_fields(
    disable_web_page_preview: Option<bool>,
    link_preview_options: Option<&LinkPreviewOptions>,
) -> Result<(), Error> {
    if disable_web_page_preview.is_some() && link_preview_options.is_some() {
        return Err(Error::InvalidRequest {
            reason: "disable_web_page_preview cannot be combined with link_preview_options"
                .to_owned(),
        });
    }

    validate_link_preview_options(link_preview_options)
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

macro_rules! impl_business_connection_id_setter {
    ($($ty:ty),* $(,)?) => {
        $(
            impl $ty {
                pub fn business_connection_id(
                    mut self,
                    business_connection_id: impl Into<String>,
                ) -> Self {
                    self.business_connection_id = Some(business_connection_id.into());
                    self
                }
            }
        )*
    };
}

macro_rules! impl_direct_messages_topic_id_setter {
    ($($ty:ty),* $(,)?) => {
        $(
            impl $ty {
                pub fn direct_messages_topic_id(mut self, direct_messages_topic_id: i64) -> Self {
                    self.direct_messages_topic_id = Some(direct_messages_topic_id);
                    self
                }
            }
        )*
    };
}

macro_rules! impl_allow_paid_broadcast_setter {
    ($($ty:ty),* $(,)?) => {
        $(
            impl $ty {
                pub fn allow_paid_broadcast(mut self, enabled: bool) -> Self {
                    self.allow_paid_broadcast = enabled.then_some(true);
                    self
                }
            }
        )*
    };
}

macro_rules! impl_message_effect_id_setter {
    ($($ty:ty),* $(,)?) => {
        $(
            impl $ty {
                pub fn message_effect_id(mut self, message_effect_id: impl Into<String>) -> Self {
                    self.message_effect_id = Some(message_effect_id.into());
                    self
                }
            }
        )*
    };
}

macro_rules! impl_suggested_post_parameters_setter {
    ($($ty:ty),* $(,)?) => {
        $(
            impl $ty {
                pub fn suggested_post_parameters(
                    mut self,
                    suggested_post_parameters: SuggestedPostParameters,
                ) -> Self {
                    self.suggested_post_parameters = Some(suggested_post_parameters);
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

impl_business_connection_id_setter!(
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
    SendDiceRequest,
    SendChatActionRequest
);

impl_direct_messages_topic_id_setter!(SendVideoNoteRequest);
impl_allow_paid_broadcast_setter!(SendVideoNoteRequest);
impl_message_effect_id_setter!(SendVideoNoteRequest);
impl_suggested_post_parameters_setter!(SendVideoNoteRequest);

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
    fn validates_reply_markup_on_message_requests() -> Result<(), Error> {
        let mut invalid_callback = crate::types::telegram::InlineKeyboardButton::new("Open");
        invalid_callback.extra.insert(
            "callback_data".to_owned(),
            serde_json::json!("x".repeat(crate::types::telegram::MAX_CALLBACK_DATA_BYTES + 1)),
        );
        let invalid_inline_markup =
            crate::types::telegram::InlineKeyboardMarkup::single_row(vec![invalid_callback]);
        let request = SendMessageRequest::new(1_i64, "hello")?.reply_markup(invalid_inline_markup);
        assert!(matches!(
            request.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let invalid_keyboard = crate::types::telegram::ReplyKeyboardMarkup::new(vec![vec![
            crate::types::telegram::KeyboardButton::new("Open").web_app("http://example.com/app"),
        ]]);
        let request = SendPhotoRequest::new(1_i64, "photo-file-id").reply_markup(invalid_keyboard);
        assert!(matches!(
            request.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let edit = EditMessageReplyMarkupRequest {
            chat_id: Some(1_i64.into()),
            message_id: Some(MessageId(1)),
            inline_message_id: None,
            reply_markup: Some(
                crate::types::telegram::InlineKeyboardMarkup::new(Vec::new()).into(),
            ),
        };
        assert!(matches!(edit.validate(), Err(Error::InvalidRequest { .. })));

        let mut invalid_link_preview = LinkPreviewOptions::disabled();
        invalid_link_preview.url = Some("https://example.com/article".to_owned());
        let request =
            SendMessageRequest::new(1_i64, "hello")?.link_preview_options(invalid_link_preview);
        assert!(matches!(
            request.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let mut conflicting_link_preview =
            SendMessageRequest::new(1_i64, "hello")?.disable_link_preview();
        conflicting_link_preview.disable_web_page_preview = Some(true);
        assert!(matches!(
            conflicting_link_preview.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let mut invalid_edit_link_preview = LinkPreviewOptions::new();
        invalid_edit_link_preview.prefer_small_media = Some(true);
        invalid_edit_link_preview.prefer_large_media = Some(true);
        let edit = EditMessageTextRequest::for_chat_message(1_i64, MessageId(1), "hello")?
            .link_preview_options(invalid_edit_link_preview);
        assert!(matches!(edit.validate(), Err(Error::InvalidRequest { .. })));

        Ok(())
    }

    #[test]
    fn validates_chat_id_on_message_requests() -> Result<(), Error> {
        assert!(matches!(
            SendPhotoRequest::new(0_i64, "photo-file-id").validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let forward = ForwardMessageRequest::new(0_i64, 1_i64, MessageId(1));
        assert!(matches!(
            forward.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let forward = ForwardMessageRequest::new(1_i64, 1_i64, MessageId(0));
        assert!(matches!(
            forward.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        assert!(matches!(
            CopyMessagesRequest::new(1_i64, "channel", vec![MessageId(1)]),
            Err(Error::InvalidRequest { .. })
        ));

        assert!(matches!(
            CopyMessagesRequest::new(1_i64, 1_i64, vec![MessageId(0)]),
            Err(Error::InvalidRequest { .. })
        ));

        assert!(matches!(
            EditMessageTextRequest::for_chat_message(0_i64, MessageId(1), "hello"),
            Err(Error::InvalidRequest { .. })
        ));

        assert!(matches!(
            EditMessageTextRequest::for_chat_message(1_i64, MessageId(0), "hello"),
            Err(Error::InvalidRequest { .. })
        ));

        let delete = DeleteMessageRequest::new("channel", MessageId(1));
        assert!(matches!(
            delete.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let delete = DeleteMessageRequest::new(1_i64, MessageId(0));
        assert!(matches!(
            delete.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let action = SendChatActionRequest::new(0_i64, ChatAction::Typing);
        assert!(matches!(
            action.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let mut threaded = SendMessageRequest::new(1_i64, "hello")?;
        threaded.message_thread_id = Some(0);
        assert!(matches!(
            threaded.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let mut reply = SendMessageRequest::new(1_i64, "hello")?;
        reply.reply_parameters = Some(ReplyParameters::new(MessageId(0)));
        assert!(matches!(
            reply.validate(),
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

        let mut invalid_audio = SendAudioRequest::new(1_i64, "audio-file-id");
        invalid_audio.duration = Some(0);
        assert!(matches!(
            invalid_audio.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let mut invalid_video = SendVideoRequest::new(1_i64, "video-file-id");
        invalid_video.width = Some(0);
        assert!(matches!(
            invalid_video.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let mut invalid_video_note = SendVideoNoteRequest::new(1_i64, "video-note-file-id");
        invalid_video_note.direct_messages_topic_id = Some(0);
        assert!(matches!(
            invalid_video_note.validate(),
            Err(Error::InvalidRequest { .. })
        ));
        invalid_video_note.direct_messages_topic_id = None;
        invalid_video_note.message_effect_id = Some("bad\nid".to_owned());
        assert!(matches!(
            invalid_video_note.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let video_note = SendVideoNoteRequest::new(1_i64, "video-note-file-id")
            .direct_messages_topic_id(7)
            .allow_paid_broadcast(true)
            .message_effect_id("effect-1")
            .suggested_post_parameters(SuggestedPostParameters::new(serde_json::json!({
                "send_date": 1
            }))?);
        video_note.validate()?;
        let video_note_json = serde_json::to_value(&video_note)
            .map_err(|source| Error::SerializeRequest { source })?;
        assert_eq!(video_note_json["direct_messages_topic_id"], 7);
        assert_eq!(video_note_json["allow_paid_broadcast"], true);
        assert_eq!(video_note_json["message_effect_id"], "effect-1");
        assert!(video_note_json.get("suggested_post_parameters").is_some());

        let mut multiline_caption = SendPhotoRequest::new(1_i64, "photo-file-id");
        multiline_caption.caption = Some("hello\nworld".to_owned());
        assert!(multiline_caption.validate().is_ok());

        let old_style_upload = SendPhotoRequest::new(1_i64, "photo-file-id");
        assert!(matches!(
            old_style_upload.validate_upload(),
            Err(Error::InvalidRequest { .. })
        ));

        let mut document_upload = SendDocumentRequest::for_upload(1_i64);
        document_upload.thumbnail = Some("attach://thumb0".to_owned());
        assert!(matches!(
            document_upload.validate_upload_parts(&[]),
            Err(Error::InvalidRequest { .. })
        ));
        let thumbnail = UploadPart::from_bytes("thumb0", "thumb.jpg", b"thumb".to_vec())?;
        assert!(
            document_upload
                .validate_upload_parts(std::slice::from_ref(&thumbnail))
                .is_ok()
        );
        assert!(matches!(
            document_upload.validate_upload_parts(&[UploadPart::from_bytes(
                "extra",
                "extra.jpg",
                b"extra".to_vec()
            )?]),
            Err(Error::InvalidRequest { .. })
        ));

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

        let invalid_group = SendMediaGroupRequest::new(
            1_i64,
            vec![
                InputMediaPhoto::new("photo-file-id").into(),
                InputMediaVideo::new("video-file-id").duration(0).into(),
            ],
        );
        assert!(matches!(invalid_group, Err(Error::InvalidRequest { .. })));

        let upload_group = SendMediaGroupRequest::new(
            1_i64,
            vec![
                InputMediaPhoto {
                    media: "attach://photo0".to_owned(),
                    caption: None,
                    parse_mode: None,
                    has_spoiler: None,
                }
                .into(),
                InputMediaVideo {
                    media: "video-file-id".to_owned(),
                    caption: None,
                    parse_mode: None,
                    width: None,
                    height: None,
                    duration: None,
                    supports_streaming: None,
                    has_spoiler: None,
                }
                .into(),
            ],
        )?;
        assert!(matches!(
            upload_group.validate(),
            Err(Error::InvalidRequest { .. })
        ));
        let upload_file =
            crate::types::upload::UploadFile::from_bytes("photo.jpg", b"photo-data".to_vec())?;
        let upload_part = UploadPart::new("photo0", upload_file)?;
        assert!(
            upload_group
                .validate_upload(std::slice::from_ref(&upload_part))
                .is_ok()
        );
        assert!(matches!(
            upload_group.validate_upload(&[]),
            Err(Error::InvalidRequest { .. })
        ));
        assert!(matches!(
            upload_group.validate_upload(&[UploadPart::from_bytes(
                "unused",
                "unused.jpg",
                b"unused".to_vec()
            )?]),
            Err(Error::InvalidRequest { .. })
        ));

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

        let mut invalid_live_location = SendLocationRequest::new(1_i64, 1.0, 2.0);
        invalid_live_location.live_period = Some(59);
        assert!(matches!(
            invalid_live_location.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let mut invalid_heading = EditMessageLiveLocationRequest {
            latitude: 1.0,
            longitude: 2.0,
            chat_id: Some(1_i64.into()),
            message_id: Some(MessageId(1)),
            inline_message_id: None,
            live_period: None,
            horizontal_accuracy: None,
            heading: Some(0),
            proximity_alert_radius: None,
            reply_markup: None,
        };
        assert!(matches!(
            invalid_heading.validate(),
            Err(Error::InvalidRequest { .. })
        ));
        invalid_heading.heading = Some(360);
        assert!(invalid_heading.validate().is_ok());

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
        poll.correct_option_id = None;
        poll.open_period = Some(MAX_POLL_OPEN_PERIOD_SECONDS);
        assert!(poll.validate().is_ok());
        poll.open_period = Some(MAX_POLL_OPEN_PERIOD_SECONDS + 1);
        assert!(matches!(poll.validate(), Err(Error::InvalidRequest { .. })));

        let mut unsupported_kind =
            SendPollRequest::new(1_i64, "question", vec!["one".to_owned(), "two".to_owned()])?;
        unsupported_kind.kind = Some(PollKind::Unknown("future".to_owned()));
        assert!(matches!(
            unsupported_kind.validate(),
            Err(Error::InvalidRequest { .. })
        ));

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

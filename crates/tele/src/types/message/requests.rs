use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::Error;
use crate::types::common::{ChatId, MessageId, ParseMode};
use crate::types::telegram::{
    InlineKeyboardMarkup, LinkPreviewOptions, ReplyMarkup, ReplyParameters, SuggestedPostParameters,
};
use crate::types::upload::{UploadPart, validate_upload_part_name};
use crate::types::validation::{
    message_text as validate_message_text, optional_caption as validate_optional_caption,
    optional_positive_i64 as validate_optional_positive_i64,
    optional_text_formatting as validate_optional_text_formatting,
    reply_markup as validate_reply_markup, reply_parameters as validate_reply_parameters,
    required_text as validate_required_text, string_id as validate_string_id,
    suggested_post_parameters as validate_suggested_post_parameters,
    text_formatting as validate_text_formatting,
};

use super::common::MessageEntity;
use super::content::{DiceEmoji, PollKind};
use super::model::Message;

const MIN_MEDIA_GROUP_ITEMS: usize = 2;
const MAX_MEDIA_GROUP_ITEMS: usize = 10;
const MAX_BULK_MESSAGE_IDS: usize = 100;
const MIN_POLL_OPTIONS: usize = 1;
const MAX_POLL_OPTIONS: usize = 12;
const MAX_POLL_QUESTION_CHARS: usize = 300;
const MAX_POLL_OPTION_CHARS: usize = 100;
const MAX_POLL_EXPLANATION_CHARS: usize = 200;
const MAX_POLL_EXPLANATION_LINE_FEEDS: usize = 2;
const MAX_POLL_DESCRIPTION_CHARS: usize = 1024;
const MAX_POLL_COUNTRY_CODES: usize = 12;
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
    pub entities: Option<Vec<MessageEntity>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_web_page_preview: Option<bool>,
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
            entities: None,
            disable_web_page_preview: None,
            disable_notification: None,
            protect_content: None,
            message_thread_id: None,
            direct_messages_topic_id: None,
            allow_paid_broadcast: None,
            message_effect_id: None,
            suggested_post_parameters: None,
            reply_parameters: None,
            reply_markup: None,
            link_preview_options: None,
        })
    }

    pub fn parse_mode(mut self, parse_mode: ParseMode) -> Self {
        self.parse_mode = Some(parse_mode);
        self
    }

    pub fn entities(mut self, entities: Vec<MessageEntity>) -> Self {
        self.entities = Some(entities);
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
        validate_direct_messages_topic_id(self.direct_messages_topic_id)?;
        validate_message_effect_id(self.message_effect_id.as_deref())?;
        validate_suggested_post_parameters(self.suggested_post_parameters.as_ref())?;
        validate_reply_parameters(self.reply_parameters.as_ref())?;
        validate_reply_markup(self.reply_markup.as_ref())?;
        validate_link_preview_fields(
            self.disable_web_page_preview,
            self.link_preview_options.as_ref(),
        )?;
        validate_message_text("sendMessage", &self.text)?;
        validate_text_formatting(
            "sendMessage text",
            &self.text,
            self.parse_mode,
            self.entities.as_deref(),
        )
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ForwardMessageRequest {
    pub chat_id: ChatId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_thread_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub direct_messages_topic_id: Option<i64>,
    pub from_chat_id: ChatId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub video_start_timestamp: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_notification: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protect_content: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_effect_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suggested_post_parameters: Option<SuggestedPostParameters>,
    pub message_id: MessageId,
}

impl ForwardMessageRequest {
    pub fn new(
        chat_id: impl Into<ChatId>,
        from_chat_id: impl Into<ChatId>,
        message_id: MessageId,
    ) -> Self {
        Self {
            chat_id: chat_id.into(),
            message_thread_id: None,
            direct_messages_topic_id: None,
            from_chat_id: from_chat_id.into(),
            video_start_timestamp: None,
            disable_notification: None,
            protect_content: None,
            message_effect_id: None,
            suggested_post_parameters: None,
            message_id,
        }
    }

    pub fn validate(&self) -> Result<(), Error> {
        self.chat_id.validate()?;
        self.from_chat_id.validate()?;
        self.message_id.validate()?;
        validate_message_thread_id(self.message_thread_id)?;
        validate_direct_messages_topic_id(self.direct_messages_topic_id)?;
        validate_message_effect_id(self.message_effect_id.as_deref())?;
        validate_suggested_post_parameters(self.suggested_post_parameters.as_ref())
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct CopyMessageRequest {
    pub chat_id: ChatId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_thread_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub direct_messages_topic_id: Option<i64>,
    pub from_chat_id: ChatId,
    pub message_id: MessageId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub video_start_timestamp: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parse_mode: Option<ParseMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caption_entities: Option<Vec<MessageEntity>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub show_caption_above_media: Option<bool>,
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

impl CopyMessageRequest {
    pub fn new(
        chat_id: impl Into<ChatId>,
        from_chat_id: impl Into<ChatId>,
        message_id: MessageId,
    ) -> Self {
        Self {
            chat_id: chat_id.into(),
            message_thread_id: None,
            direct_messages_topic_id: None,
            from_chat_id: from_chat_id.into(),
            message_id,
            video_start_timestamp: None,
            caption: None,
            parse_mode: None,
            caption_entities: None,
            show_caption_above_media: None,
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
        self.chat_id.validate()?;
        self.from_chat_id.validate()?;
        self.message_id.validate()?;
        validate_message_thread_id(self.message_thread_id)?;
        validate_direct_messages_topic_id(self.direct_messages_topic_id)?;
        validate_message_effect_id(self.message_effect_id.as_deref())?;
        validate_suggested_post_parameters(self.suggested_post_parameters.as_ref())?;
        validate_reply_parameters(self.reply_parameters.as_ref())?;
        validate_reply_markup(self.reply_markup.as_ref())?;
        validate_caption_fields(
            "copy caption",
            self.caption.as_deref(),
            self.parse_mode,
            self.caption_entities.as_deref(),
        )
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct CopyMessagesRequest {
    pub chat_id: ChatId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_thread_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub direct_messages_topic_id: Option<i64>,
    pub from_chat_id: ChatId,
    pub message_ids: Vec<MessageId>,
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
        validate_ordered_bulk_message_ids("copyMessages", &message_ids)?;

        Ok(Self {
            chat_id,
            message_thread_id: None,
            direct_messages_topic_id: None,
            from_chat_id,
            message_ids,
            disable_notification: None,
            protect_content: None,
            remove_caption: None,
        })
    }

    pub fn validate(&self) -> Result<(), Error> {
        self.chat_id.validate()?;
        self.from_chat_id.validate()?;
        validate_message_thread_id(self.message_thread_id)?;
        validate_direct_messages_topic_id(self.direct_messages_topic_id)?;
        validate_ordered_bulk_message_ids("copyMessages", &self.message_ids)
    }
}

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct MessageIdObject {
    pub message_id: MessageId,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct SentWebAppMessage {
    pub inline_message_id: String,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
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
    pub caption_entities: Option<Vec<MessageEntity>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_spoiler: Option<bool>,
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

impl SendPhotoRequest {
    pub fn new(chat_id: impl Into<ChatId>, photo: impl Into<String>) -> Self {
        Self {
            chat_id: chat_id.into(),
            business_connection_id: None,
            photo: Some(photo.into()),
            caption: None,
            parse_mode: None,
            caption_entities: None,
            has_spoiler: None,
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
            photo: None,
            caption: None,
            parse_mode: None,
            caption_entities: None,
            has_spoiler: None,
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
        validate_required_file_reference("photo", self.photo.as_deref())?;
        validate_caption_fields(
            "sendPhoto caption",
            self.caption.as_deref(),
            self.parse_mode,
            self.caption_entities.as_deref(),
        )
    }

    pub(crate) fn validate_upload(&self) -> Result<(), Error> {
        validate_absent_upload_field("photo", self.photo.as_deref())?;
        validate_business_connection_id(self.business_connection_id.as_deref())?;
        self.chat_id.validate()?;
        validate_message_thread_id(self.message_thread_id)?;
        validate_direct_messages_topic_id(self.direct_messages_topic_id)?;
        validate_message_effect_id(self.message_effect_id.as_deref())?;
        validate_suggested_post_parameters(self.suggested_post_parameters.as_ref())?;
        validate_reply_parameters(self.reply_parameters.as_ref())?;
        validate_reply_markup(self.reply_markup.as_ref())?;
        validate_caption_fields(
            "sendPhoto caption",
            self.caption.as_deref(),
            self.parse_mode,
            self.caption_entities.as_deref(),
        )
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
    pub caption_entities: Option<Vec<MessageEntity>>,
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

impl SendAudioRequest {
    pub fn new(chat_id: impl Into<ChatId>, audio: impl Into<String>) -> Self {
        Self {
            chat_id: chat_id.into(),
            business_connection_id: None,
            audio: Some(audio.into()),
            caption: None,
            parse_mode: None,
            caption_entities: None,
            duration: None,
            performer: None,
            title: None,
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
            audio: None,
            caption: None,
            parse_mode: None,
            caption_entities: None,
            duration: None,
            performer: None,
            title: None,
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
        validate_required_file_reference("audio", self.audio.as_deref())?;
        validate_caption_fields(
            "sendAudio caption",
            self.caption.as_deref(),
            self.parse_mode,
            self.caption_entities.as_deref(),
        )?;
        validate_optional_file_reference("thumbnail", self.thumbnail.as_deref())?;
        validate_positive_u32("duration", self.duration)
    }

    pub(crate) fn validate_upload_parts(&self, files: &[UploadPart]) -> Result<(), Error> {
        validate_absent_upload_field("audio", self.audio.as_deref())?;
        validate_business_connection_id(self.business_connection_id.as_deref())?;
        self.chat_id.validate()?;
        validate_message_thread_id(self.message_thread_id)?;
        validate_direct_messages_topic_id(self.direct_messages_topic_id)?;
        validate_message_effect_id(self.message_effect_id.as_deref())?;
        validate_suggested_post_parameters(self.suggested_post_parameters.as_ref())?;
        validate_reply_parameters(self.reply_parameters.as_ref())?;
        validate_reply_markup(self.reply_markup.as_ref())?;
        validate_caption_fields(
            "sendAudio caption",
            self.caption.as_deref(),
            self.parse_mode,
            self.caption_entities.as_deref(),
        )?;
        validate_optional_upload_file_reference("thumbnail", self.thumbnail.as_deref())?;
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
    pub caption_entities: Option<Vec<MessageEntity>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_content_type_detection: Option<bool>,
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

impl SendDocumentRequest {
    pub fn new(chat_id: impl Into<ChatId>, document: impl Into<String>) -> Self {
        Self {
            chat_id: chat_id.into(),
            business_connection_id: None,
            document: Some(document.into()),
            thumbnail: None,
            caption: None,
            parse_mode: None,
            caption_entities: None,
            disable_content_type_detection: None,
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
            document: None,
            thumbnail: None,
            caption: None,
            parse_mode: None,
            caption_entities: None,
            disable_content_type_detection: None,
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
        validate_required_file_reference("document", self.document.as_deref())?;
        validate_caption_fields(
            "sendDocument caption",
            self.caption.as_deref(),
            self.parse_mode,
            self.caption_entities.as_deref(),
        )?;
        validate_optional_file_reference("thumbnail", self.thumbnail.as_deref())
    }

    pub(crate) fn validate_upload_parts(&self, files: &[UploadPart]) -> Result<(), Error> {
        validate_absent_upload_field("document", self.document.as_deref())?;
        validate_business_connection_id(self.business_connection_id.as_deref())?;
        self.chat_id.validate()?;
        validate_message_thread_id(self.message_thread_id)?;
        validate_direct_messages_topic_id(self.direct_messages_topic_id)?;
        validate_message_effect_id(self.message_effect_id.as_deref())?;
        validate_suggested_post_parameters(self.suggested_post_parameters.as_ref())?;
        validate_reply_parameters(self.reply_parameters.as_ref())?;
        validate_reply_markup(self.reply_markup.as_ref())?;
        validate_caption_fields(
            "sendDocument caption",
            self.caption.as_deref(),
            self.parse_mode,
            self.caption_entities.as_deref(),
        )?;
        validate_optional_upload_file_reference("thumbnail", self.thumbnail.as_deref())?;
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
    pub caption_entities: Option<Vec<MessageEntity>>,
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
            caption_entities: None,
            supports_streaming: None,
            has_spoiler: None,
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
            video: None,
            duration: None,
            width: None,
            height: None,
            thumbnail: None,
            caption: None,
            parse_mode: None,
            caption_entities: None,
            supports_streaming: None,
            has_spoiler: None,
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
        validate_required_file_reference("video", self.video.as_deref())?;
        validate_caption_fields(
            "sendVideo caption",
            self.caption.as_deref(),
            self.parse_mode,
            self.caption_entities.as_deref(),
        )?;
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
        validate_direct_messages_topic_id(self.direct_messages_topic_id)?;
        validate_message_effect_id(self.message_effect_id.as_deref())?;
        validate_suggested_post_parameters(self.suggested_post_parameters.as_ref())?;
        validate_reply_parameters(self.reply_parameters.as_ref())?;
        validate_reply_markup(self.reply_markup.as_ref())?;
        validate_caption_fields(
            "sendVideo caption",
            self.caption.as_deref(),
            self.parse_mode,
            self.caption_entities.as_deref(),
        )?;
        validate_optional_upload_file_reference("thumbnail", self.thumbnail.as_deref())?;
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
    pub caption_entities: Option<Vec<MessageEntity>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_spoiler: Option<bool>,
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
            caption_entities: None,
            has_spoiler: None,
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
            animation: None,
            duration: None,
            width: None,
            height: None,
            thumbnail: None,
            caption: None,
            parse_mode: None,
            caption_entities: None,
            has_spoiler: None,
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
        validate_required_file_reference("animation", self.animation.as_deref())?;
        validate_caption_fields(
            "sendAnimation caption",
            self.caption.as_deref(),
            self.parse_mode,
            self.caption_entities.as_deref(),
        )?;
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
        validate_direct_messages_topic_id(self.direct_messages_topic_id)?;
        validate_message_effect_id(self.message_effect_id.as_deref())?;
        validate_suggested_post_parameters(self.suggested_post_parameters.as_ref())?;
        validate_reply_parameters(self.reply_parameters.as_ref())?;
        validate_reply_markup(self.reply_markup.as_ref())?;
        validate_caption_fields(
            "sendAnimation caption",
            self.caption.as_deref(),
            self.parse_mode,
            self.caption_entities.as_deref(),
        )?;
        validate_optional_upload_file_reference("thumbnail", self.thumbnail.as_deref())?;
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
    pub caption_entities: Option<Vec<MessageEntity>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<u32>,
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

impl SendVoiceRequest {
    pub fn new(chat_id: impl Into<ChatId>, voice: impl Into<String>) -> Self {
        Self {
            chat_id: chat_id.into(),
            business_connection_id: None,
            voice: Some(voice.into()),
            caption: None,
            parse_mode: None,
            caption_entities: None,
            duration: None,
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
            voice: None,
            caption: None,
            parse_mode: None,
            caption_entities: None,
            duration: None,
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
        validate_required_file_reference("voice", self.voice.as_deref())?;
        validate_caption_fields(
            "sendVoice caption",
            self.caption.as_deref(),
            self.parse_mode,
            self.caption_entities.as_deref(),
        )?;
        validate_positive_u32("duration", self.duration)
    }

    pub(crate) fn validate_upload(&self) -> Result<(), Error> {
        validate_absent_upload_field("voice", self.voice.as_deref())?;
        validate_business_connection_id(self.business_connection_id.as_deref())?;
        self.chat_id.validate()?;
        validate_message_thread_id(self.message_thread_id)?;
        validate_direct_messages_topic_id(self.direct_messages_topic_id)?;
        validate_message_effect_id(self.message_effect_id.as_deref())?;
        validate_suggested_post_parameters(self.suggested_post_parameters.as_ref())?;
        validate_reply_parameters(self.reply_parameters.as_ref())?;
        validate_reply_markup(self.reply_markup.as_ref())?;
        validate_caption_fields(
            "sendVoice caption",
            self.caption.as_deref(),
            self.parse_mode,
            self.caption_entities.as_deref(),
        )?;
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
        validate_optional_upload_file_reference("thumbnail", self.thumbnail.as_deref())?;
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
    pub caption_entities: Option<Vec<MessageEntity>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_spoiler: Option<bool>,
}

impl InputMediaPhoto {
    pub fn new(media: impl Into<String>) -> Self {
        Self {
            media: media.into(),
            caption: None,
            parse_mode: None,
            caption_entities: None,
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

    pub fn caption_entities(mut self, entities: Vec<MessageEntity>) -> Self {
        self.caption_entities = Some(entities);
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
    pub thumbnail: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parse_mode: Option<ParseMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caption_entities: Option<Vec<MessageEntity>>,
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
            thumbnail: None,
            caption: None,
            parse_mode: None,
            caption_entities: None,
            width: None,
            height: None,
            duration: None,
            supports_streaming: None,
            has_spoiler: None,
        }
    }

    pub fn thumbnail(mut self, thumbnail: impl Into<String>) -> Self {
        self.thumbnail = Some(thumbnail.into());
        self
    }

    pub fn caption(mut self, caption: impl Into<String>) -> Self {
        self.caption = Some(caption.into());
        self
    }

    pub fn parse_mode(mut self, parse_mode: ParseMode) -> Self {
        self.parse_mode = Some(parse_mode);
        self
    }

    pub fn caption_entities(mut self, entities: Vec<MessageEntity>) -> Self {
        self.caption_entities = Some(entities);
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
    pub thumbnail: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parse_mode: Option<ParseMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caption_entities: Option<Vec<MessageEntity>>,
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
            thumbnail: None,
            caption: None,
            parse_mode: None,
            caption_entities: None,
            width: None,
            height: None,
            duration: None,
            has_spoiler: None,
        }
    }

    pub fn thumbnail(mut self, thumbnail: impl Into<String>) -> Self {
        self.thumbnail = Some(thumbnail.into());
        self
    }

    pub fn caption(mut self, caption: impl Into<String>) -> Self {
        self.caption = Some(caption.into());
        self
    }

    pub fn parse_mode(mut self, parse_mode: ParseMode) -> Self {
        self.parse_mode = Some(parse_mode);
        self
    }

    pub fn caption_entities(mut self, entities: Vec<MessageEntity>) -> Self {
        self.caption_entities = Some(entities);
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
    pub thumbnail: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parse_mode: Option<ParseMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caption_entities: Option<Vec<MessageEntity>>,
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
            thumbnail: None,
            caption: None,
            parse_mode: None,
            caption_entities: None,
            duration: None,
            performer: None,
            title: None,
        }
    }

    pub fn thumbnail(mut self, thumbnail: impl Into<String>) -> Self {
        self.thumbnail = Some(thumbnail.into());
        self
    }

    pub fn caption(mut self, caption: impl Into<String>) -> Self {
        self.caption = Some(caption.into());
        self
    }

    pub fn parse_mode(mut self, parse_mode: ParseMode) -> Self {
        self.parse_mode = Some(parse_mode);
        self
    }

    pub fn caption_entities(mut self, entities: Vec<MessageEntity>) -> Self {
        self.caption_entities = Some(entities);
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
    pub thumbnail: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parse_mode: Option<ParseMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caption_entities: Option<Vec<MessageEntity>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_content_type_detection: Option<bool>,
}

impl InputMediaDocument {
    pub fn new(media: impl Into<String>) -> Self {
        Self {
            media: media.into(),
            thumbnail: None,
            caption: None,
            parse_mode: None,
            caption_entities: None,
            disable_content_type_detection: None,
        }
    }

    pub fn thumbnail(mut self, thumbnail: impl Into<String>) -> Self {
        self.thumbnail = Some(thumbnail.into());
        self
    }

    pub fn caption(mut self, caption: impl Into<String>) -> Self {
        self.caption = Some(caption.into());
        self
    }

    pub fn parse_mode(mut self, parse_mode: ParseMode) -> Self {
        self.parse_mode = Some(parse_mode);
        self
    }

    pub fn caption_entities(mut self, entities: Vec<MessageEntity>) -> Self {
        self.caption_entities = Some(entities);
        self
    }

    pub fn disable_content_type_detection(mut self, enabled: bool) -> Self {
        self.disable_content_type_detection = enabled.then_some(true);
        self
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub struct InputMediaLivePhoto {
    pub media: String,
    pub photo: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parse_mode: Option<ParseMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caption_entities: Option<Vec<MessageEntity>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub show_caption_above_media: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_spoiler: Option<bool>,
}

impl InputMediaLivePhoto {
    pub fn new(media: impl Into<String>, photo: impl Into<String>) -> Self {
        Self {
            media: media.into(),
            photo: photo.into(),
            caption: None,
            parse_mode: None,
            caption_entities: None,
            show_caption_above_media: None,
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

    pub fn caption_entities(mut self, entities: Vec<MessageEntity>) -> Self {
        self.caption_entities = Some(entities);
        self
    }

    pub fn show_caption_above_media(mut self, enabled: bool) -> Self {
        self.show_caption_above_media = enabled.then_some(true);
        self
    }

    pub fn has_spoiler(mut self, enabled: bool) -> Self {
        self.has_spoiler = enabled.then_some(true);
        self
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub struct InputMediaLocation {
    pub latitude: f64,
    pub longitude: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub horizontal_accuracy: Option<f64>,
}

impl InputMediaLocation {
    pub fn new(latitude: f64, longitude: f64) -> Self {
        Self {
            latitude,
            longitude,
            horizontal_accuracy: None,
        }
    }

    pub fn horizontal_accuracy(mut self, horizontal_accuracy: f64) -> Self {
        self.horizontal_accuracy = Some(horizontal_accuracy);
        self
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub struct InputMediaSticker {
    pub media: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub emoji: Option<String>,
}

impl InputMediaSticker {
    pub fn new(media: impl Into<String>) -> Self {
        Self {
            media: media.into(),
            emoji: None,
        }
    }

    pub fn emoji(mut self, emoji: impl Into<String>) -> Self {
        self.emoji = Some(emoji.into());
        self
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub struct InputMediaVenue {
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
}

impl InputMediaVenue {
    pub fn new(
        latitude: f64,
        longitude: f64,
        title: impl Into<String>,
        address: impl Into<String>,
    ) -> Self {
        Self {
            latitude,
            longitude,
            title: title.into(),
            address: address.into(),
            foursquare_id: None,
            foursquare_type: None,
            google_place_id: None,
            google_place_type: None,
        }
    }

    pub fn foursquare_id(mut self, foursquare_id: impl Into<String>) -> Self {
        self.foursquare_id = Some(foursquare_id.into());
        self
    }

    pub fn foursquare_type(mut self, foursquare_type: impl Into<String>) -> Self {
        self.foursquare_type = Some(foursquare_type.into());
        self
    }

    pub fn google_place_id(mut self, google_place_id: impl Into<String>) -> Self {
        self.google_place_id = Some(google_place_id.into());
        self
    }

    pub fn google_place_type(mut self, google_place_type: impl Into<String>) -> Self {
        self.google_place_type = Some(google_place_type.into());
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
    LivePhoto(Box<InputMediaLivePhoto>),
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

impl From<InputMediaLivePhoto> for InputMedia {
    fn from(value: InputMediaLivePhoto) -> Self {
        Self::LivePhoto(Box::new(value))
    }
}

impl InputMedia {
    pub fn validate(&self) -> Result<(), Error> {
        validate_media(self)
    }
}

/// Media item accepted by `sendMediaGroup`.
///
/// Telegram media groups do not accept animations. Use [`InputMedia`] for APIs that support
/// animations, such as editing a message's media.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InputMediaGroupItem {
    Photo(Box<InputMediaPhoto>),
    Video(Box<InputMediaVideo>),
    Audio(Box<InputMediaAudio>),
    Document(Box<InputMediaDocument>),
    LivePhoto(Box<InputMediaLivePhoto>),
}

impl From<InputMediaPhoto> for InputMediaGroupItem {
    fn from(value: InputMediaPhoto) -> Self {
        Self::Photo(Box::new(value))
    }
}

impl From<InputMediaVideo> for InputMediaGroupItem {
    fn from(value: InputMediaVideo) -> Self {
        Self::Video(Box::new(value))
    }
}

impl From<InputMediaAudio> for InputMediaGroupItem {
    fn from(value: InputMediaAudio) -> Self {
        Self::Audio(Box::new(value))
    }
}

impl From<InputMediaDocument> for InputMediaGroupItem {
    fn from(value: InputMediaDocument) -> Self {
        Self::Document(Box::new(value))
    }
}

impl From<InputMediaLivePhoto> for InputMediaGroupItem {
    fn from(value: InputMediaLivePhoto) -> Self {
        Self::LivePhoto(Box::new(value))
    }
}

impl InputMediaGroupItem {
    pub fn validate(&self) -> Result<(), Error> {
        validate_media_group_item(self)
    }
}

impl From<InputMediaGroupItem> for InputMedia {
    fn from(value: InputMediaGroupItem) -> Self {
        match value {
            InputMediaGroupItem::Photo(value) => Self::Photo(value),
            InputMediaGroupItem::Video(value) => Self::Video(value),
            InputMediaGroupItem::Audio(value) => Self::Audio(value),
            InputMediaGroupItem::Document(value) => Self::Document(value),
            InputMediaGroupItem::LivePhoto(value) => Self::LivePhoto(value),
        }
    }
}

impl TryFrom<InputMedia> for InputMediaGroupItem {
    type Error = Error;

    fn try_from(value: InputMedia) -> Result<Self, Self::Error> {
        match value {
            InputMedia::Photo(value) => Ok(Self::Photo(value)),
            InputMedia::Video(value) => Ok(Self::Video(value)),
            InputMedia::Audio(value) => Ok(Self::Audio(value)),
            InputMedia::Document(value) => Ok(Self::Document(value)),
            InputMedia::LivePhoto(value) => Ok(Self::LivePhoto(value)),
            InputMedia::Animation(_) => Err(Error::InvalidRequest {
                reason: "sendMediaGroup does not support animation media".to_owned(),
            }),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InputPollMedia {
    Animation(Box<InputMediaAnimation>),
    Audio(Box<InputMediaAudio>),
    Document(Box<InputMediaDocument>),
    LivePhoto(Box<InputMediaLivePhoto>),
    Location(Box<InputMediaLocation>),
    Photo(Box<InputMediaPhoto>),
    Venue(Box<InputMediaVenue>),
    Video(Box<InputMediaVideo>),
}

impl InputPollMedia {
    pub fn animation(media: impl Into<String>) -> Self {
        InputMediaAnimation::new(media).into()
    }

    pub fn audio(media: impl Into<String>) -> Self {
        InputMediaAudio::new(media).into()
    }

    pub fn document(media: impl Into<String>) -> Self {
        InputMediaDocument::new(media).into()
    }

    pub fn live_photo(media: impl Into<String>, photo: impl Into<String>) -> Self {
        InputMediaLivePhoto::new(media, photo).into()
    }

    pub fn location(latitude: f64, longitude: f64) -> Self {
        InputMediaLocation::new(latitude, longitude).into()
    }

    pub fn photo(media: impl Into<String>) -> Self {
        InputMediaPhoto::new(media).into()
    }

    pub fn venue(
        latitude: f64,
        longitude: f64,
        title: impl Into<String>,
        address: impl Into<String>,
    ) -> Self {
        InputMediaVenue::new(latitude, longitude, title, address).into()
    }

    pub fn video(media: impl Into<String>) -> Self {
        InputMediaVideo::new(media).into()
    }

    pub fn validate(&self) -> Result<(), Error> {
        validate_poll_media(self)
    }
}

impl From<InputMediaAnimation> for InputPollMedia {
    fn from(value: InputMediaAnimation) -> Self {
        Self::Animation(Box::new(value))
    }
}

impl From<InputMediaAudio> for InputPollMedia {
    fn from(value: InputMediaAudio) -> Self {
        Self::Audio(Box::new(value))
    }
}

impl From<InputMediaDocument> for InputPollMedia {
    fn from(value: InputMediaDocument) -> Self {
        Self::Document(Box::new(value))
    }
}

impl From<InputMediaLivePhoto> for InputPollMedia {
    fn from(value: InputMediaLivePhoto) -> Self {
        Self::LivePhoto(Box::new(value))
    }
}

impl From<InputMediaLocation> for InputPollMedia {
    fn from(value: InputMediaLocation) -> Self {
        Self::Location(Box::new(value))
    }
}

impl From<InputMediaPhoto> for InputPollMedia {
    fn from(value: InputMediaPhoto) -> Self {
        Self::Photo(Box::new(value))
    }
}

impl From<InputMediaVenue> for InputPollMedia {
    fn from(value: InputMediaVenue) -> Self {
        Self::Venue(Box::new(value))
    }
}

impl From<InputMediaVideo> for InputPollMedia {
    fn from(value: InputMediaVideo) -> Self {
        Self::Video(Box::new(value))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InputPollOptionMedia {
    Animation(Box<InputMediaAnimation>),
    LivePhoto(Box<InputMediaLivePhoto>),
    Location(Box<InputMediaLocation>),
    Photo(Box<InputMediaPhoto>),
    Sticker(Box<InputMediaSticker>),
    Venue(Box<InputMediaVenue>),
    Video(Box<InputMediaVideo>),
}

impl InputPollOptionMedia {
    pub fn animation(media: impl Into<String>) -> Self {
        InputMediaAnimation::new(media).into()
    }

    pub fn live_photo(media: impl Into<String>, photo: impl Into<String>) -> Self {
        InputMediaLivePhoto::new(media, photo).into()
    }

    pub fn location(latitude: f64, longitude: f64) -> Self {
        InputMediaLocation::new(latitude, longitude).into()
    }

    pub fn photo(media: impl Into<String>) -> Self {
        InputMediaPhoto::new(media).into()
    }

    pub fn sticker(media: impl Into<String>) -> Self {
        InputMediaSticker::new(media).into()
    }

    pub fn venue(
        latitude: f64,
        longitude: f64,
        title: impl Into<String>,
        address: impl Into<String>,
    ) -> Self {
        InputMediaVenue::new(latitude, longitude, title, address).into()
    }

    pub fn video(media: impl Into<String>) -> Self {
        InputMediaVideo::new(media).into()
    }

    pub fn validate(&self) -> Result<(), Error> {
        validate_poll_option_media(self)
    }
}

impl From<InputMediaAnimation> for InputPollOptionMedia {
    fn from(value: InputMediaAnimation) -> Self {
        Self::Animation(Box::new(value))
    }
}

impl From<InputMediaLivePhoto> for InputPollOptionMedia {
    fn from(value: InputMediaLivePhoto) -> Self {
        Self::LivePhoto(Box::new(value))
    }
}

impl From<InputMediaLocation> for InputPollOptionMedia {
    fn from(value: InputMediaLocation) -> Self {
        Self::Location(Box::new(value))
    }
}

impl From<InputMediaPhoto> for InputPollOptionMedia {
    fn from(value: InputMediaPhoto) -> Self {
        Self::Photo(Box::new(value))
    }
}

impl From<InputMediaSticker> for InputPollOptionMedia {
    fn from(value: InputMediaSticker) -> Self {
        Self::Sticker(Box::new(value))
    }
}

impl From<InputMediaVenue> for InputPollOptionMedia {
    fn from(value: InputMediaVenue) -> Self {
        Self::Venue(Box::new(value))
    }
}

impl From<InputMediaVideo> for InputPollOptionMedia {
    fn from(value: InputMediaVideo) -> Self {
        Self::Video(Box::new(value))
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct SendMediaGroupRequest {
    pub chat_id: ChatId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub business_connection_id: Option<String>,
    pub media: Vec<InputMediaGroupItem>,
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
    pub reply_parameters: Option<ReplyParameters>,
}

impl SendMediaGroupRequest {
    pub fn new(chat_id: impl Into<ChatId>, media: Vec<InputMediaGroupItem>) -> Result<Self, Error> {
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
            direct_messages_topic_id: None,
            allow_paid_broadcast: None,
            message_effect_id: None,
            reply_parameters: None,
        })
    }

    pub fn validate(&self) -> Result<(), Error> {
        validate_business_connection_id(self.business_connection_id.as_deref())?;
        self.chat_id.validate()?;
        validate_message_thread_id(self.message_thread_id)?;
        validate_direct_messages_topic_id(self.direct_messages_topic_id)?;
        validate_message_effect_id(self.message_effect_id.as_deref())?;
        validate_reply_parameters(self.reply_parameters.as_ref())?;
        validate_media_group_items(&self.media)?;
        validate_no_multipart_attach_references(&self.media)
    }

    pub(crate) fn validate_upload(&self, files: &[UploadPart]) -> Result<(), Error> {
        validate_business_connection_id(self.business_connection_id.as_deref())?;
        self.chat_id.validate()?;
        validate_message_thread_id(self.message_thread_id)?;
        validate_direct_messages_topic_id(self.direct_messages_topic_id)?;
        validate_message_effect_id(self.message_effect_id.as_deref())?;
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
        validate_required_text("phone_number", &self.phone_number)?;
        validate_required_text("first_name", &self.first_name)
    }
}

/// Poll answer option sent by [`SendPollRequest`].
#[derive(Clone, Debug, Serialize)]
#[non_exhaustive]
pub struct InputPollOption {
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_parse_mode: Option<ParseMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_entities: Option<Vec<MessageEntity>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub media: Option<InputPollOptionMedia>,
}

impl InputPollOption {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            text_parse_mode: None,
            text_entities: None,
            media: None,
        }
    }

    pub fn text_parse_mode(mut self, parse_mode: ParseMode) -> Self {
        self.text_parse_mode = Some(parse_mode);
        self
    }

    pub fn text_entities(mut self, entities: Vec<MessageEntity>) -> Self {
        self.text_entities = Some(entities);
        self
    }

    pub fn media(mut self, media: impl Into<InputPollOptionMedia>) -> Self {
        self.media = Some(media.into());
        self
    }

    pub fn validate(&self) -> Result<(), Error> {
        validate_poll_option(self)?;
        validate_text_formatting(
            "poll option text",
            &self.text,
            self.text_parse_mode,
            self.text_entities.as_deref(),
        )
    }
}

impl From<String> for InputPollOption {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for InputPollOption {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct SendPollRequest {
    pub chat_id: ChatId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub business_connection_id: Option<String>,
    pub question: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub question_parse_mode: Option<ParseMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub question_entities: Option<Vec<MessageEntity>>,
    pub options: Vec<InputPollOption>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_anonymous: Option<bool>,
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<PollKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allows_multiple_answers: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allows_revoting: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shuffle_options: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_adding_options: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hide_results_until_closes: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub members_only: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub country_codes: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub correct_option_ids: Option<Vec<u8>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub explanation: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub explanation_parse_mode: Option<ParseMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub explanation_entities: Option<Vec<MessageEntity>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub explanation_media: Option<InputPollMedia>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub open_period: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub close_date: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description_parse_mode: Option<ParseMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description_entities: Option<Vec<MessageEntity>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub media: Option<InputPollMedia>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_closed: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_notification: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protect_content: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_paid_broadcast: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_thread_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_effect_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_parameters: Option<ReplyParameters>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_markup: Option<ReplyMarkup>,
}

impl SendPollRequest {
    pub fn new(
        chat_id: impl Into<ChatId>,
        question: impl Into<String>,
        options: impl IntoIterator<Item = impl Into<InputPollOption>>,
    ) -> Result<Self, Error> {
        let chat_id = chat_id.into();
        chat_id.validate()?;
        let question = question.into();
        let options = options.into_iter().map(Into::into).collect::<Vec<_>>();
        validate_poll(&PollValidation {
            question: &question,
            question_parse_mode: None,
            question_entities: None,
            options: &options,
            is_anonymous: None,
            correct_option_ids: None,
            open_period: None,
            close_date: None,
            kind: None,
            allow_adding_options: None,
            explanation: None,
            explanation_parse_mode: None,
            explanation_entities: None,
            explanation_media: None,
            description: None,
            description_parse_mode: None,
            description_entities: None,
            media: None,
            country_codes: None,
        })?;

        Ok(Self {
            chat_id,
            business_connection_id: None,
            question,
            question_parse_mode: None,
            question_entities: None,
            options,
            is_anonymous: None,
            kind: None,
            allows_multiple_answers: None,
            allows_revoting: None,
            shuffle_options: None,
            allow_adding_options: None,
            hide_results_until_closes: None,
            members_only: None,
            country_codes: None,
            correct_option_ids: None,
            explanation: None,
            explanation_parse_mode: None,
            explanation_entities: None,
            explanation_media: None,
            open_period: None,
            close_date: None,
            description: None,
            description_parse_mode: None,
            description_entities: None,
            media: None,
            is_closed: None,
            disable_notification: None,
            protect_content: None,
            allow_paid_broadcast: None,
            message_thread_id: None,
            message_effect_id: None,
            reply_parameters: None,
            reply_markup: None,
        })
    }

    pub fn validate(&self) -> Result<(), Error> {
        validate_business_connection_id(self.business_connection_id.as_deref())?;
        self.chat_id.validate()?;
        validate_message_thread_id(self.message_thread_id)?;
        validate_message_effect_id(self.message_effect_id.as_deref())?;
        validate_reply_parameters(self.reply_parameters.as_ref())?;
        validate_reply_markup(self.reply_markup.as_ref())?;
        validate_poll(&PollValidation {
            question: &self.question,
            question_parse_mode: self.question_parse_mode,
            question_entities: self.question_entities.as_deref(),
            options: &self.options,
            is_anonymous: self.is_anonymous,
            correct_option_ids: self.correct_option_ids.as_deref(),
            open_period: self.open_period,
            close_date: self.close_date,
            kind: self.kind.as_ref(),
            allow_adding_options: self.allow_adding_options,
            explanation: self.explanation.as_deref(),
            explanation_parse_mode: self.explanation_parse_mode,
            explanation_entities: self.explanation_entities.as_deref(),
            explanation_media: self.explanation_media.as_ref(),
            description: self.description.as_deref(),
            description_parse_mode: self.description_parse_mode,
            description_entities: self.description_entities.as_deref(),
            media: self.media.as_ref(),
            country_codes: self.country_codes.as_deref(),
        })
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct StopPollRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub business_connection_id: Option<String>,
    pub chat_id: ChatId,
    pub message_id: MessageId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_markup: Option<InlineKeyboardMarkup>,
}

impl StopPollRequest {
    pub fn new(chat_id: impl Into<ChatId>, message_id: MessageId) -> Self {
        Self {
            business_connection_id: None,
            chat_id: chat_id.into(),
            message_id,
            reply_markup: None,
        }
    }

    pub fn validate(&self) -> Result<(), Error> {
        validate_business_connection_id(self.business_connection_id.as_deref())?;
        self.chat_id.validate()?;
        self.message_id.validate()?;
        validate_inline_keyboard_markup(self.reply_markup.as_ref())
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

impl SendDiceRequest {
    pub fn new(chat_id: impl Into<ChatId>) -> Self {
        Self {
            chat_id: chat_id.into(),
            business_connection_id: None,
            emoji: None,
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
    pub business_connection_id: Option<String>,
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
    pub entities: Option<Vec<MessageEntity>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_markup: Option<InlineKeyboardMarkup>,
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
            business_connection_id: None,
            chat_id: Some(chat_id),
            message_id: Some(message_id),
            inline_message_id: None,
            text,
            parse_mode: None,
            entities: None,
            reply_markup: None,
            link_preview_options: None,
        })
    }

    pub fn for_inline_message(
        inline_message_id: impl Into<String>,
        text: impl Into<String>,
    ) -> Result<Self, Error> {
        let inline_message_id = inline_message_id.into();
        validate_string_id("inline_message_id", &inline_message_id)?;

        let text = text.into();
        validate_message_text("editMessageText", &text)?;

        Ok(Self {
            business_connection_id: None,
            chat_id: None,
            message_id: None,
            inline_message_id: Some(inline_message_id),
            text,
            parse_mode: None,
            entities: None,
            reply_markup: None,
            link_preview_options: None,
        })
    }

    pub fn validate(&self) -> Result<(), Error> {
        validate_business_connection_id(self.business_connection_id.as_deref())?;
        validate_edit_target(
            self.chat_id.as_ref(),
            self.message_id,
            &self.inline_message_id,
        )?;

        validate_inline_keyboard_markup(self.reply_markup.as_ref())?;
        validate_link_preview_options(self.link_preview_options.as_ref())?;
        validate_message_text("editMessageText", &self.text)?;
        validate_text_formatting(
            "editMessageText text",
            &self.text,
            self.parse_mode,
            self.entities.as_deref(),
        )
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct EditMessageCaptionRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub business_connection_id: Option<String>,
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
    pub caption_entities: Option<Vec<MessageEntity>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_markup: Option<InlineKeyboardMarkup>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub show_caption_above_media: Option<bool>,
}

impl EditMessageCaptionRequest {
    pub fn validate(&self) -> Result<(), Error> {
        validate_business_connection_id(self.business_connection_id.as_deref())?;
        validate_edit_target(
            self.chat_id.as_ref(),
            self.message_id,
            &self.inline_message_id,
        )?;
        validate_inline_keyboard_markup(self.reply_markup.as_ref())?;
        validate_caption_fields(
            "editMessageCaption caption",
            self.caption.as_deref(),
            self.parse_mode,
            self.caption_entities.as_deref(),
        )
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct EditMessageReplyMarkupRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub business_connection_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chat_id: Option<ChatId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_id: Option<MessageId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inline_message_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_markup: Option<InlineKeyboardMarkup>,
}

impl EditMessageReplyMarkupRequest {
    pub fn validate(&self) -> Result<(), Error> {
        validate_business_connection_id(self.business_connection_id.as_deref())?;
        validate_edit_target(
            self.chat_id.as_ref(),
            self.message_id,
            &self.inline_message_id,
        )?;
        validate_inline_keyboard_markup(self.reply_markup.as_ref())
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct EditMessageLiveLocationRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub business_connection_id: Option<String>,
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
    pub reply_markup: Option<InlineKeyboardMarkup>,
}

impl EditMessageLiveLocationRequest {
    pub fn validate(&self) -> Result<(), Error> {
        validate_business_connection_id(self.business_connection_id.as_deref())?;
        validate_edit_target(
            self.chat_id.as_ref(),
            self.message_id,
            &self.inline_message_id,
        )?;
        validate_inline_keyboard_markup(self.reply_markup.as_ref())?;
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
    pub business_connection_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chat_id: Option<ChatId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_id: Option<MessageId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inline_message_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_markup: Option<InlineKeyboardMarkup>,
}

impl StopMessageLiveLocationRequest {
    pub fn validate(&self) -> Result<(), Error> {
        validate_business_connection_id(self.business_connection_id.as_deref())?;
        validate_edit_target(
            self.chat_id.as_ref(),
            self.message_id,
            &self.inline_message_id,
        )?;
        validate_inline_keyboard_markup(self.reply_markup.as_ref())
    }
}

#[derive(Clone, Debug, Deserialize)]
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
    if let Some(inline_message_id) = inline_message_id.as_deref() {
        validate_string_id("inline_message_id", inline_message_id)?;
    }

    let has_complete_chat_target = chat_id.is_some() && message_id.is_some();
    let has_any_chat_target_field = chat_id.is_some() || message_id.is_some();
    let has_inline_target = inline_message_id.is_some();

    if has_inline_target {
        if has_any_chat_target_field {
            return Err(Error::InvalidRequest {
                reason:
                    "method accepts either chat_id with message_id or inline_message_id, not both"
                        .to_owned(),
            });
        }

        return Ok(());
    }

    if has_complete_chat_target {
        return Ok(());
    }

    Err(Error::InvalidRequest {
        reason: "method requires either chat_id with message_id or inline_message_id".to_owned(),
    })
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

fn validate_json_file_reference(label: &str, value: &str) -> Result<(), Error> {
    validate_file_reference(label, value)?;
    if attach_name(value).is_some() {
        return Err(Error::InvalidRequest {
            reason: format!("{label} cannot use attach:// outside multipart upload requests"),
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

    validate_json_file_reference(label, value)
}

fn validate_optional_file_reference(label: &str, value: Option<&str>) -> Result<(), Error> {
    if let Some(value) = value {
        validate_json_file_reference(label, value)?;
    }
    Ok(())
}

fn validate_optional_upload_file_reference(label: &str, value: Option<&str>) -> Result<(), Error> {
    if let Some(value) = value {
        validate_file_reference(label, value)?;
    }
    Ok(())
}

fn validate_file_reference_with_attach_mode(
    label: &str,
    value: &str,
    allow_multipart_attach: bool,
) -> Result<(), Error> {
    if allow_multipart_attach {
        validate_file_reference(label, value)
    } else {
        validate_json_file_reference(label, value)
    }
}

fn validate_optional_file_reference_with_attach_mode(
    label: &str,
    value: Option<&str>,
    allow_multipart_attach: bool,
) -> Result<(), Error> {
    if let Some(value) = value {
        validate_file_reference_with_attach_mode(label, value, allow_multipart_attach)?;
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

fn validate_caption_fields(
    field: &str,
    caption: Option<&str>,
    parse_mode: Option<ParseMode>,
    entities: Option<&[MessageEntity]>,
) -> Result<(), Error> {
    validate_optional_caption(caption)?;
    validate_optional_text_formatting(field, caption, parse_mode, entities)
}

fn validate_input_media_photo(
    media: &InputMediaPhoto,
    allow_multipart_attach: bool,
) -> Result<(), Error> {
    validate_file_reference_with_attach_mode("media", &media.media, allow_multipart_attach)?;
    validate_caption_fields(
        "input media caption",
        media.caption.as_deref(),
        media.parse_mode,
        media.caption_entities.as_deref(),
    )
}

fn validate_input_media_video(
    media: &InputMediaVideo,
    allow_multipart_attach: bool,
) -> Result<(), Error> {
    validate_file_reference_with_attach_mode("media", &media.media, allow_multipart_attach)?;
    validate_optional_file_reference_with_attach_mode(
        "thumbnail",
        media.thumbnail.as_deref(),
        allow_multipart_attach,
    )?;
    validate_caption_fields(
        "input media caption",
        media.caption.as_deref(),
        media.parse_mode,
        media.caption_entities.as_deref(),
    )?;
    validate_positive_u32("width", media.width)?;
    validate_positive_u32("height", media.height)?;
    validate_positive_u32("duration", media.duration)
}

fn validate_input_media_animation(
    media: &InputMediaAnimation,
    allow_multipart_attach: bool,
) -> Result<(), Error> {
    validate_file_reference_with_attach_mode("media", &media.media, allow_multipart_attach)?;
    validate_optional_file_reference_with_attach_mode(
        "thumbnail",
        media.thumbnail.as_deref(),
        allow_multipart_attach,
    )?;
    validate_caption_fields(
        "input media caption",
        media.caption.as_deref(),
        media.parse_mode,
        media.caption_entities.as_deref(),
    )?;
    validate_positive_u32("width", media.width)?;
    validate_positive_u32("height", media.height)?;
    validate_positive_u32("duration", media.duration)
}

fn validate_input_media_audio(
    media: &InputMediaAudio,
    allow_multipart_attach: bool,
) -> Result<(), Error> {
    validate_file_reference_with_attach_mode("media", &media.media, allow_multipart_attach)?;
    validate_optional_file_reference_with_attach_mode(
        "thumbnail",
        media.thumbnail.as_deref(),
        allow_multipart_attach,
    )?;
    validate_caption_fields(
        "input media caption",
        media.caption.as_deref(),
        media.parse_mode,
        media.caption_entities.as_deref(),
    )?;
    validate_positive_u32("duration", media.duration)
}

fn validate_input_media_document(
    media: &InputMediaDocument,
    allow_multipart_attach: bool,
) -> Result<(), Error> {
    validate_file_reference_with_attach_mode("media", &media.media, allow_multipart_attach)?;
    validate_optional_file_reference_with_attach_mode(
        "thumbnail",
        media.thumbnail.as_deref(),
        allow_multipart_attach,
    )?;
    validate_caption_fields(
        "input media caption",
        media.caption.as_deref(),
        media.parse_mode,
        media.caption_entities.as_deref(),
    )
}

fn validate_input_media_live_photo(
    media: &InputMediaLivePhoto,
    allow_multipart_attach: bool,
) -> Result<(), Error> {
    validate_file_reference_with_attach_mode("media", &media.media, allow_multipart_attach)?;
    validate_file_reference_with_attach_mode("photo", &media.photo, allow_multipart_attach)?;
    validate_caption_fields(
        "input media caption",
        media.caption.as_deref(),
        media.parse_mode,
        media.caption_entities.as_deref(),
    )
}

fn validate_input_media_location(media: &InputMediaLocation) -> Result<(), Error> {
    validate_coordinates(media.latitude, media.longitude)?;
    if let Some(horizontal_accuracy) = media.horizontal_accuracy
        && !(0.0..=MAX_HORIZONTAL_ACCURACY_METERS).contains(&horizontal_accuracy)
    {
        return Err(Error::InvalidRequest {
            reason: format!(
                "horizontal_accuracy must be 0-{MAX_HORIZONTAL_ACCURACY_METERS} meters"
            ),
        });
    }

    Ok(())
}

fn validate_input_media_sticker(
    media: &InputMediaSticker,
    allow_multipart_attach: bool,
) -> Result<(), Error> {
    validate_file_reference_with_attach_mode("media", &media.media, allow_multipart_attach)?;
    if let Some(emoji) = media.emoji.as_deref() {
        validate_required_text("sticker emoji", emoji)?;
    }

    Ok(())
}

fn validate_input_media_venue(media: &InputMediaVenue) -> Result<(), Error> {
    validate_coordinates(media.latitude, media.longitude)?;
    validate_required_text("venue title", &media.title)?;
    validate_required_text("venue address", &media.address)
}

fn validate_media(media: &InputMedia) -> Result<(), Error> {
    match media {
        InputMedia::Photo(media) => validate_input_media_photo(media, false),
        InputMedia::Video(media) => validate_input_media_video(media, false),
        InputMedia::Animation(media) => validate_input_media_animation(media, false),
        InputMedia::Audio(media) => validate_input_media_audio(media, false),
        InputMedia::Document(media) => validate_input_media_document(media, false),
        InputMedia::LivePhoto(media) => validate_input_media_live_photo(media, false),
    }
}

fn validate_poll_media(media: &InputPollMedia) -> Result<(), Error> {
    match media {
        InputPollMedia::Animation(media) => validate_input_media_animation(media, false),
        InputPollMedia::Audio(media) => validate_input_media_audio(media, false),
        InputPollMedia::Document(media) => validate_input_media_document(media, false),
        InputPollMedia::LivePhoto(media) => validate_input_media_live_photo(media, false),
        InputPollMedia::Location(media) => validate_input_media_location(media),
        InputPollMedia::Photo(media) => validate_input_media_photo(media, false),
        InputPollMedia::Venue(media) => validate_input_media_venue(media),
        InputPollMedia::Video(media) => validate_input_media_video(media, false),
    }
}

fn validate_optional_poll_media(media: Option<&InputPollMedia>) -> Result<(), Error> {
    if let Some(media) = media {
        validate_poll_media(media)?;
    }

    Ok(())
}

fn validate_poll_option_media(media: &InputPollOptionMedia) -> Result<(), Error> {
    match media {
        InputPollOptionMedia::Animation(media) => validate_input_media_animation(media, false),
        InputPollOptionMedia::LivePhoto(media) => validate_input_media_live_photo(media, false),
        InputPollOptionMedia::Location(media) => validate_input_media_location(media),
        InputPollOptionMedia::Photo(media) => validate_input_media_photo(media, false),
        InputPollOptionMedia::Sticker(media) => validate_input_media_sticker(media, false),
        InputPollOptionMedia::Venue(media) => validate_input_media_venue(media),
        InputPollOptionMedia::Video(media) => validate_input_media_video(media, false),
    }
}

fn validate_media_group_item(media: &InputMediaGroupItem) -> Result<(), Error> {
    match media {
        InputMediaGroupItem::Photo(media) => validate_input_media_photo(media, true),
        InputMediaGroupItem::Video(media) => validate_input_media_video(media, true),
        InputMediaGroupItem::Audio(media) => validate_input_media_audio(media, true),
        InputMediaGroupItem::Document(media) => validate_input_media_document(media, true),
        InputMediaGroupItem::LivePhoto(media) => validate_input_media_live_photo(media, true),
    }
}

struct MediaGroupFileReferences<'a> {
    media: &'a str,
    extra_file: Option<&'a str>,
}

impl<'a> MediaGroupFileReferences<'a> {
    fn iter(&self) -> impl Iterator<Item = &'a str> + '_ {
        std::iter::once(self.media).chain(self.extra_file)
    }
}

fn media_group_file_references(media: &InputMediaGroupItem) -> MediaGroupFileReferences<'_> {
    match media {
        InputMediaGroupItem::Photo(media) => MediaGroupFileReferences {
            media: &media.media,
            extra_file: None,
        },
        InputMediaGroupItem::Video(media) => MediaGroupFileReferences {
            media: &media.media,
            extra_file: media.thumbnail.as_deref(),
        },
        InputMediaGroupItem::Audio(media) => MediaGroupFileReferences {
            media: &media.media,
            extra_file: media.thumbnail.as_deref(),
        },
        InputMediaGroupItem::Document(media) => MediaGroupFileReferences {
            media: &media.media,
            extra_file: media.thumbnail.as_deref(),
        },
        InputMediaGroupItem::LivePhoto(media) => MediaGroupFileReferences {
            media: &media.media,
            extra_file: Some(&media.photo),
        },
    }
}

fn attach_name(value: &str) -> Option<&str> {
    value.strip_prefix(ATTACH_URI_PREFIX)
}

fn validate_attach_name(field: &str, name: &str) -> Result<(), Error> {
    validate_upload_part_name(field, name)
}

fn media_attach_names(media: &[InputMediaGroupItem]) -> Result<BTreeSet<String>, Error> {
    let mut names = BTreeSet::new();
    for item in media {
        for reference in media_group_file_references(item).iter() {
            if let Some(name) = attach_name(reference) {
                validate_attach_name("media attach name", name)?;
                names.insert(name.to_owned());
            }
        }
    }

    Ok(names)
}

fn validate_no_multipart_attach_references(media: &[InputMediaGroupItem]) -> Result<(), Error> {
    for item in media {
        if media_group_file_references(item)
            .iter()
            .any(|reference| attach_name(reference).is_some())
        {
            return Err(Error::InvalidRequest {
                reason: "sendMediaGroup JSON requests cannot use attach:// file references; use send_media_group_upload".to_owned(),
            });
        }
    }

    Ok(())
}

fn validate_media_group_upload_parts(
    media: &[InputMediaGroupItem],
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
            reason:
                "sendMediaGroup upload requires at least one attach:// media or thumbnail reference"
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
                reason: format!("multipart file part `{name}` is not referenced by attach://"),
            });
        }
    }

    Ok(())
}

fn validate_media_group_items(media: &[InputMediaGroupItem]) -> Result<(), Error> {
    if !(MIN_MEDIA_GROUP_ITEMS..=MAX_MEDIA_GROUP_ITEMS).contains(&media.len()) {
        return Err(Error::InvalidRequest {
            reason: format!(
                "sendMediaGroup requires {MIN_MEDIA_GROUP_ITEMS}-{MAX_MEDIA_GROUP_ITEMS} media items"
            ),
        });
    }
    for item in media {
        validate_media_group_item(item)?;
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

struct PollValidation<'a> {
    question: &'a str,
    question_parse_mode: Option<ParseMode>,
    question_entities: Option<&'a [MessageEntity]>,
    options: &'a [InputPollOption],
    is_anonymous: Option<bool>,
    correct_option_ids: Option<&'a [u8]>,
    open_period: Option<u32>,
    close_date: Option<i64>,
    kind: Option<&'a PollKind>,
    allow_adding_options: Option<bool>,
    explanation: Option<&'a str>,
    explanation_parse_mode: Option<ParseMode>,
    explanation_entities: Option<&'a [MessageEntity]>,
    explanation_media: Option<&'a InputPollMedia>,
    description: Option<&'a str>,
    description_parse_mode: Option<ParseMode>,
    description_entities: Option<&'a [MessageEntity]>,
    media: Option<&'a InputPollMedia>,
    country_codes: Option<&'a [String]>,
}

fn validate_poll(input: &PollValidation<'_>) -> Result<(), Error> {
    validate_required_text("poll question", input.question)?;
    if input.question.chars().count() > MAX_POLL_QUESTION_CHARS {
        return Err(Error::InvalidRequest {
            reason: format!("poll question exceeds {MAX_POLL_QUESTION_CHARS} characters"),
        });
    }
    validate_text_formatting(
        "poll question",
        input.question,
        input.question_parse_mode,
        input.question_entities,
    )?;
    if !(MIN_POLL_OPTIONS..=MAX_POLL_OPTIONS).contains(&input.options.len()) {
        return Err(Error::InvalidRequest {
            reason: format!("sendPoll requires {MIN_POLL_OPTIONS}-{MAX_POLL_OPTIONS} options"),
        });
    }
    for option in input.options {
        option.validate()?;
    }

    validate_correct_option_ids(input.kind, input.correct_option_ids, input.options.len())?;
    validate_poll_option_additions(input.kind, input.is_anonymous, input.allow_adding_options)?;

    if input.open_period.is_some() && input.close_date.is_some() {
        return Err(Error::InvalidRequest {
            reason: "open_period cannot be combined with close_date".to_owned(),
        });
    }
    if let Some(open_period) = input.open_period
        && !(MIN_POLL_OPEN_PERIOD_SECONDS..=MAX_POLL_OPEN_PERIOD_SECONDS).contains(&open_period)
    {
        return Err(Error::InvalidRequest {
            reason: format!(
                "open_period must be {MIN_POLL_OPEN_PERIOD_SECONDS}-{MAX_POLL_OPEN_PERIOD_SECONDS} seconds"
            ),
        });
    }
    if let Some(close_date) = input.close_date
        && close_date <= 0
    {
        return Err(Error::InvalidRequest {
            reason: "close_date must be a positive Unix timestamp".to_owned(),
        });
    }
    if let Some(PollKind::Unknown(kind)) = input.kind {
        return Err(Error::InvalidRequest {
            reason: format!("unsupported poll type `{kind}`"),
        });
    }

    validate_poll_explanation(input.explanation)?;
    validate_optional_text_formatting(
        "poll explanation",
        input.explanation,
        input.explanation_parse_mode,
        input.explanation_entities,
    )?;
    validate_optional_poll_media(input.explanation_media)?;
    validate_poll_description(input.description)?;
    validate_optional_text_formatting(
        "poll description",
        input.description,
        input.description_parse_mode,
        input.description_entities,
    )?;
    validate_optional_poll_media(input.media)?;
    validate_poll_country_codes(input.country_codes)?;

    Ok(())
}

fn validate_poll_option_additions(
    kind: Option<&PollKind>,
    is_anonymous: Option<bool>,
    allow_adding_options: Option<bool>,
) -> Result<(), Error> {
    if allow_adding_options != Some(true) {
        return Ok(());
    }

    if is_anonymous.unwrap_or(true) {
        return Err(Error::InvalidRequest {
            reason: "allow_adding_options requires a non-anonymous poll".to_owned(),
        });
    }
    if matches!(kind, Some(PollKind::Quiz)) {
        return Err(Error::InvalidRequest {
            reason: "allow_adding_options is not supported for quiz polls".to_owned(),
        });
    }

    Ok(())
}

fn validate_poll_option(option: &InputPollOption) -> Result<(), Error> {
    validate_required_text("poll option", &option.text)?;
    if option.text.chars().count() > MAX_POLL_OPTION_CHARS {
        return Err(Error::InvalidRequest {
            reason: format!("poll option exceeds {MAX_POLL_OPTION_CHARS} characters"),
        });
    }
    if let Some(media) = option.media.as_ref() {
        validate_poll_option_media(media)?;
    }

    Ok(())
}

fn validate_correct_option_ids(
    kind: Option<&PollKind>,
    correct_option_ids: Option<&[u8]>,
    option_count: usize,
) -> Result<(), Error> {
    let is_quiz = matches!(kind, Some(PollKind::Quiz));
    let ids = correct_option_ids.unwrap_or_default();

    if !is_quiz && !ids.is_empty() {
        return Err(Error::InvalidRequest {
            reason: "correct_option_ids can only be used for quiz polls".to_owned(),
        });
    }
    if is_quiz && ids.is_empty() {
        return Err(Error::InvalidRequest {
            reason: "quiz polls require at least one correct_option_ids entry".to_owned(),
        });
    }

    let mut previous = None;
    for id in ids {
        let id = usize::from(*id);
        if id >= option_count {
            return Err(Error::InvalidRequest {
                reason: "correct_option_ids must point to existing poll options".to_owned(),
            });
        }
        if previous.is_some_and(|previous| id <= previous) {
            return Err(Error::InvalidRequest {
                reason: "correct_option_ids must be strictly increasing".to_owned(),
            });
        }
        previous = Some(id);
    }

    Ok(())
}

fn validate_poll_explanation(explanation: Option<&str>) -> Result<(), Error> {
    let Some(explanation) = explanation else {
        return Ok(());
    };
    if explanation.chars().count() > MAX_POLL_EXPLANATION_CHARS {
        return Err(Error::InvalidRequest {
            reason: format!("poll explanation exceeds {MAX_POLL_EXPLANATION_CHARS} characters"),
        });
    }
    if explanation.chars().filter(|ch| *ch == '\n').count() > MAX_POLL_EXPLANATION_LINE_FEEDS {
        return Err(Error::InvalidRequest {
            reason: format!(
                "poll explanation must contain at most {MAX_POLL_EXPLANATION_LINE_FEEDS} line feeds"
            ),
        });
    }
    if explanation.chars().any(is_disallowed_display_control) {
        return Err(Error::InvalidRequest {
            reason: "poll explanation must not contain non-whitespace control characters"
                .to_owned(),
        });
    }

    Ok(())
}

fn validate_poll_description(description: Option<&str>) -> Result<(), Error> {
    let Some(description) = description else {
        return Ok(());
    };
    if description.chars().count() > MAX_POLL_DESCRIPTION_CHARS {
        return Err(Error::InvalidRequest {
            reason: format!("poll description exceeds {MAX_POLL_DESCRIPTION_CHARS} characters"),
        });
    }
    if description.chars().any(is_disallowed_display_control) {
        return Err(Error::InvalidRequest {
            reason: "poll description must not contain non-whitespace control characters"
                .to_owned(),
        });
    }

    Ok(())
}

fn validate_poll_country_codes(country_codes: Option<&[String]>) -> Result<(), Error> {
    let Some(country_codes) = country_codes else {
        return Ok(());
    };
    if country_codes.len() > MAX_POLL_COUNTRY_CODES {
        return Err(Error::InvalidRequest {
            reason: format!("sendPoll accepts at most {MAX_POLL_COUNTRY_CODES} country codes"),
        });
    }
    for country_code in country_codes {
        if country_code.len() != 2 || !country_code.bytes().all(|byte| byte.is_ascii_uppercase()) {
            return Err(Error::InvalidRequest {
                reason: "poll country codes must be two uppercase ASCII letters".to_owned(),
            });
        }
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
    let mut unique = BTreeSet::new();
    for message_id in message_ids {
        message_id.validate()?;
        if !unique.insert(*message_id) {
            return Err(Error::InvalidRequest {
                reason: format!("{method} message ids must be unique"),
            });
        }
    }

    Ok(())
}

fn validate_ordered_bulk_message_ids(method: &str, message_ids: &[MessageId]) -> Result<(), Error> {
    validate_bulk_message_ids(method, message_ids)?;

    let mut previous = None;
    for message_id in message_ids {
        if previous.is_some_and(|previous| message_id.0 <= previous) {
            return Err(Error::InvalidRequest {
                reason: format!("{method} message ids must be strictly increasing"),
            });
        }
        previous = Some(message_id.0);
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

fn validate_inline_keyboard_markup(
    reply_markup: Option<&InlineKeyboardMarkup>,
) -> Result<(), Error> {
    if let Some(reply_markup) = reply_markup {
        reply_markup.validate()?;
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

macro_rules! impl_inline_keyboard_markup_setter {
    ($($ty:ty),* $(,)?) => {
        $(
            impl $ty {
                pub fn reply_markup(mut self, reply_markup: impl Into<InlineKeyboardMarkup>) -> Self {
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

macro_rules! impl_caption_setter {
    ($($ty:ty),* $(,)?) => {
        $(
            impl $ty {
                pub fn caption(mut self, caption: impl Into<String>) -> Self {
                    self.caption = Some(caption.into());
                    self
                }
            }
        )*
    };
}

macro_rules! impl_parse_mode_setter {
    ($($ty:ty),* $(,)?) => {
        $(
            impl $ty {
                pub fn parse_mode(mut self, parse_mode: ParseMode) -> Self {
                    self.parse_mode = Some(parse_mode);
                    self
                }
            }
        )*
    };
}

macro_rules! impl_entities_setter {
    ($($ty:ty),* $(,)?) => {
        $(
            impl $ty {
                pub fn entities(mut self, entities: Vec<MessageEntity>) -> Self {
                    self.entities = Some(entities);
                    self
                }
            }
        )*
    };
}

macro_rules! impl_caption_entities_setter {
    ($($ty:ty),* $(,)?) => {
        $(
            impl $ty {
                pub fn caption_entities(mut self, entities: Vec<MessageEntity>) -> Self {
                    self.caption_entities = Some(entities);
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
    SendDiceRequest
);

impl_inline_keyboard_markup_setter!(
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
    SendChatActionRequest,
    StopPollRequest,
    EditMessageTextRequest,
    EditMessageCaptionRequest,
    EditMessageReplyMarkupRequest,
    EditMessageLiveLocationRequest,
    StopMessageLiveLocationRequest
);

impl_direct_messages_topic_id_setter!(
    SendMessageRequest,
    ForwardMessageRequest,
    CopyMessageRequest,
    CopyMessagesRequest,
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
    SendDiceRequest
);
impl_allow_paid_broadcast_setter!(
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
impl_message_effect_id_setter!(
    SendMessageRequest,
    ForwardMessageRequest,
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
impl_suggested_post_parameters_setter!(
    SendMessageRequest,
    ForwardMessageRequest,
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
    SendDiceRequest
);

impl_caption_setter!(
    CopyMessageRequest,
    SendPhotoRequest,
    SendAudioRequest,
    SendDocumentRequest,
    SendVideoRequest,
    SendAnimationRequest,
    SendVoiceRequest,
    EditMessageCaptionRequest
);

impl_parse_mode_setter!(
    CopyMessageRequest,
    SendPhotoRequest,
    SendAudioRequest,
    SendDocumentRequest,
    SendVideoRequest,
    SendAnimationRequest,
    SendVoiceRequest,
    EditMessageTextRequest,
    EditMessageCaptionRequest
);

impl_entities_setter!(EditMessageTextRequest);

impl_caption_entities_setter!(
    CopyMessageRequest,
    SendPhotoRequest,
    SendAudioRequest,
    SendDocumentRequest,
    SendVideoRequest,
    SendAnimationRequest,
    SendVoiceRequest,
    EditMessageCaptionRequest
);

impl_link_preview_setter!(SendMessageRequest, EditMessageTextRequest);

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::types::validation::{MAX_CAPTION_CHARS, MAX_MESSAGE_TEXT_CHARS};

    fn valid_suggested_post_send_date() -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(600, |duration| duration.as_secs() as i64 + 600)
    }

    fn bold_entity(length: u32) -> MessageEntity {
        entity(crate::types::message::MessageEntityKind::Bold, 0, length)
    }

    fn entity(
        kind: crate::types::message::MessageEntityKind,
        offset: u32,
        length: u32,
    ) -> MessageEntity {
        MessageEntity {
            kind,
            offset,
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
    fn response_objects_preserve_future_fields()
    -> std::result::Result<(), Box<dyn std::error::Error>> {
        let copied: MessageIdObject = serde_json::from_value(serde_json::json!({
            "message_id": 42,
            "future_copy_field": "kept"
        }))?;
        let sent: SentWebAppMessage = serde_json::from_value(serde_json::json!({
            "inline_message_id": "inline-id",
            "future_web_app_field": true
        }))?;

        assert_eq!(copied.message_id, MessageId(42));
        assert_eq!(copied.extra["future_copy_field"], "kept");
        assert_eq!(sent.inline_message_id, "inline-id");
        assert_eq!(sent.extra["future_web_app_field"], true);

        Ok(())
    }

    #[test]
    fn validates_send_message_text_bounds() {
        assert!(SendMessageRequest::new(1_i64, "hello").is_ok());
        assert!(SendMessageRequest::new(1_i64, "hello\nworld\tagain").is_ok());

        for text in ["", "   ", "hello\u{0000}", "hello\u{0007}"] {
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
    fn serializes_explicit_text_and_caption_entities() -> Result<(), Error> {
        let message = SendMessageRequest::new(1_i64, "hello")?.entities(vec![bold_entity(5)]);
        message.validate()?;
        let message_json =
            serde_json::to_value(&message).map_err(|source| Error::SerializeRequest { source })?;
        assert_eq!(message_json["entities"][0]["type"], "bold");

        let emoji_message = SendMessageRequest::new(1_i64, "💣a")?.entities(vec![entity(
            crate::types::message::MessageEntityKind::Bold,
            2,
            1,
        )]);
        assert!(emoji_message.validate().is_ok());

        let out_of_range_message =
            SendMessageRequest::new(1_i64, "hello")?.entities(vec![bold_entity(6)]);
        assert!(matches!(
            out_of_range_message.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let unknown_entity_message =
            SendMessageRequest::new(1_i64, "hello")?.entities(vec![entity(
                crate::types::message::MessageEntityKind::Unknown("future".to_owned()),
                0,
                5,
            )]);
        assert!(matches!(
            unknown_entity_message.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let mut invalid_mention =
            entity(crate::types::message::MessageEntityKind::TextMention, 0, 5);
        invalid_mention.user = Some(crate::types::bot::User {
            id: crate::types::common::UserId(0),
            is_bot: false,
            first_name: "Alice".to_owned(),
            last_name: None,
            username: None,
            language_code: None,
            is_premium: false,
            added_to_attachment_menu: false,
            can_join_groups: None,
            can_read_all_group_messages: None,
            supports_guest_queries: None,
            supports_inline_queries: None,
            can_connect_to_business: None,
            has_main_web_app: None,
            has_topics_enabled: None,
            allows_users_to_create_topics: None,
            can_manage_bots: None,
            extra: BTreeMap::new(),
        });
        let invalid_mention_message =
            SendMessageRequest::new(1_i64, "hello")?.entities(vec![invalid_mention]);
        assert!(matches!(
            invalid_mention_message.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let mut invalid_text_link =
            entity(crate::types::message::MessageEntityKind::TextLink, 0, 5);
        invalid_text_link.url = Some("https://exa\nmple.com".to_owned());
        let invalid_text_link_message =
            SendMessageRequest::new(1_i64, "hello")?.entities(vec![invalid_text_link]);
        assert!(matches!(
            invalid_text_link_message.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let mut valid_pre = entity(crate::types::message::MessageEntityKind::Pre, 0, 5);
        valid_pre.language = Some("rust".to_owned());
        let valid_pre_message = SendMessageRequest::new(1_i64, "hello")?.entities(vec![valid_pre]);
        assert!(valid_pre_message.validate().is_ok());

        let mut invalid_pre = entity(crate::types::message::MessageEntityKind::Pre, 0, 5);
        invalid_pre.language = Some("rust\nlang".to_owned());
        let invalid_pre_message =
            SendMessageRequest::new(1_i64, "hello")?.entities(vec![invalid_pre]);
        assert!(matches!(
            invalid_pre_message.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let mut bold_with_url = bold_entity(5);
        bold_with_url.url = Some("https://example.com".to_owned());
        let bold_with_url_message =
            SendMessageRequest::new(1_i64, "hello")?.entities(vec![bold_with_url]);
        assert!(matches!(
            bold_with_url_message.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let mut text_link_with_user =
            entity(crate::types::message::MessageEntityKind::TextLink, 0, 5);
        text_link_with_user.url = Some("https://example.com".to_owned());
        text_link_with_user.user = Some(crate::types::bot::User {
            id: crate::types::common::UserId(1),
            is_bot: false,
            first_name: "Alice".to_owned(),
            last_name: None,
            username: None,
            language_code: None,
            is_premium: false,
            added_to_attachment_menu: false,
            can_join_groups: None,
            can_read_all_group_messages: None,
            supports_guest_queries: None,
            supports_inline_queries: None,
            can_connect_to_business: None,
            has_main_web_app: None,
            has_topics_enabled: None,
            allows_users_to_create_topics: None,
            can_manage_bots: None,
            extra: BTreeMap::new(),
        });
        let text_link_with_user_message =
            SendMessageRequest::new(1_i64, "hello")?.entities(vec![text_link_with_user]);
        assert!(matches!(
            text_link_with_user_message.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let mut custom_emoji_with_language =
            entity(crate::types::message::MessageEntityKind::CustomEmoji, 0, 5);
        custom_emoji_with_language.custom_emoji_id = Some("emoji-id".to_owned());
        custom_emoji_with_language.language = Some("rust".to_owned());
        let custom_emoji_with_language_message =
            SendMessageRequest::new(1_i64, "hello")?.entities(vec![custom_emoji_with_language]);
        assert!(matches!(
            custom_emoji_with_language_message.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let mut invalid_message = SendMessageRequest::new(1_i64, "hello")?
            .parse_mode(ParseMode::Html)
            .entities(vec![bold_entity(5)]);
        assert!(matches!(
            invalid_message.validate(),
            Err(Error::InvalidRequest { .. })
        ));
        invalid_message.entities = None;
        assert!(invalid_message.validate().is_ok());

        let photo = SendPhotoRequest::new(1_i64, "photo-file-id")
            .caption("hello")
            .caption_entities(vec![bold_entity(5)]);
        photo.validate()?;
        let photo_json =
            serde_json::to_value(&photo).map_err(|source| Error::SerializeRequest { source })?;
        assert_eq!(photo_json["caption_entities"][0]["type"], "bold");

        let missing_caption =
            SendPhotoRequest::new(1_i64, "photo-file-id").caption_entities(vec![bold_entity(5)]);
        assert!(matches!(
            missing_caption.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let invalid_photo = SendPhotoRequest::new(1_i64, "photo-file-id")
            .caption("hello")
            .parse_mode(ParseMode::Html)
            .caption_entities(vec![bold_entity(5)]);
        assert!(matches!(
            invalid_photo.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let media_group = SendMediaGroupRequest::new(
            1_i64,
            vec![
                InputMediaPhoto::new("photo-file-id")
                    .caption("hello")
                    .caption_entities(vec![bold_entity(5)])
                    .into(),
                InputMediaVideo::new("video-file-id").into(),
            ],
        )?;
        media_group.validate()?;
        let media_group_json = serde_json::to_value(&media_group)
            .map_err(|source| Error::SerializeRequest { source })?;
        assert_eq!(
            media_group_json["media"][0]["caption_entities"][0]["type"],
            "bold"
        );

        let animation_media = InputMedia::from(InputMediaAnimation::new("animation-file-id"));
        assert!(animation_media.validate().is_ok());
        assert!(matches!(
            InputMediaGroupItem::try_from(animation_media),
            Err(Error::InvalidRequest { reason })
                if reason.contains("does not support animation")
        ));

        let invalid_media_group = SendMediaGroupRequest::new(
            1_i64,
            vec![
                InputMediaPhoto::new("photo-file-id")
                    .caption("hello")
                    .parse_mode(ParseMode::Html)
                    .caption_entities(vec![bold_entity(5)])
                    .into(),
                InputMediaVideo::new("video-file-id").into(),
            ],
        );
        assert!(matches!(
            invalid_media_group,
            Err(Error::InvalidRequest { .. })
        ));

        let invalid_edit_text =
            EditMessageTextRequest::for_chat_message(1_i64, MessageId(10), "hello")?
                .entities(vec![bold_entity(5)]);
        let mut invalid_edit_text = invalid_edit_text;
        invalid_edit_text.parse_mode = Some(ParseMode::Html);
        assert!(matches!(
            invalid_edit_text.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let invalid_edit_caption = EditMessageCaptionRequest {
            business_connection_id: None,
            chat_id: Some(1_i64.into()),
            message_id: Some(MessageId(10)),
            inline_message_id: None,
            caption: Some("hello".to_owned()),
            parse_mode: Some(ParseMode::Html),
            caption_entities: Some(vec![bold_entity(5)]),
            reply_markup: None,
            show_caption_above_media: None,
        };
        assert!(matches!(
            invalid_edit_caption.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        Ok(())
    }

    #[test]
    fn serializes_common_send_options() -> Result<(), Error> {
        let send_date = valid_suggested_post_send_date();
        let suggested_post_parameters = SuggestedPostParameters::new(serde_json::json!({
            "send_date": send_date
        }))?;
        let message = SendMessageRequest::new(1_i64, "hello")?
            .direct_messages_topic_id(7)
            .allow_paid_broadcast(true)
            .message_effect_id("effect-1")
            .suggested_post_parameters(suggested_post_parameters);
        message.validate()?;
        let message_json =
            serde_json::to_value(&message).map_err(|source| Error::SerializeRequest { source })?;
        assert_eq!(message_json["direct_messages_topic_id"], 7);
        assert_eq!(message_json["allow_paid_broadcast"], true);
        assert_eq!(message_json["message_effect_id"], "effect-1");
        assert!(message_json.get("suggested_post_parameters").is_some());

        let forward = ForwardMessageRequest::new(1_i64, 2_i64, MessageId(10))
            .direct_messages_topic_id(6)
            .message_effect_id("forward-effect")
            .suggested_post_parameters(SuggestedPostParameters::new(serde_json::json!({
                "send_date": send_date + 1
            }))?);
        forward.validate()?;
        let forward_json =
            serde_json::to_value(&forward).map_err(|source| Error::SerializeRequest { source })?;
        assert_eq!(forward_json["direct_messages_topic_id"], 6);
        assert_eq!(forward_json["message_effect_id"], "forward-effect");
        assert!(forward_json.get("suggested_post_parameters").is_some());

        let copy = CopyMessageRequest::new(1_i64, 2_i64, MessageId(10))
            .direct_messages_topic_id(7)
            .allow_paid_broadcast(true)
            .message_effect_id("copy-effect")
            .suggested_post_parameters(SuggestedPostParameters::new(serde_json::json!({
                "send_date": send_date + 2
            }))?);
        copy.validate()?;
        let copy_json =
            serde_json::to_value(&copy).map_err(|source| Error::SerializeRequest { source })?;
        assert_eq!(copy_json["direct_messages_topic_id"], 7);
        assert_eq!(copy_json["allow_paid_broadcast"], true);
        assert_eq!(copy_json["message_effect_id"], "copy-effect");
        assert!(copy_json.get("suggested_post_parameters").is_some());

        let copy_messages =
            CopyMessagesRequest::new(1_i64, 2_i64, vec![MessageId(10), MessageId(11)])?
                .direct_messages_topic_id(8);
        copy_messages.validate()?;
        let copy_messages_json = serde_json::to_value(&copy_messages)
            .map_err(|source| Error::SerializeRequest { source })?;
        assert_eq!(copy_messages_json["direct_messages_topic_id"], 8);
        assert!(
            copy_messages_json
                .get("suggested_post_parameters")
                .is_none()
        );

        let media_group = SendMediaGroupRequest::new(
            1_i64,
            vec![
                InputMediaPhoto::new("photo-file-id").into(),
                InputMediaVideo::new("video-file-id").into(),
            ],
        )?
        .direct_messages_topic_id(8)
        .allow_paid_broadcast(true)
        .message_effect_id("effect-2");
        media_group.validate()?;
        let media_group_json = serde_json::to_value(&media_group)
            .map_err(|source| Error::SerializeRequest { source })?;
        assert_eq!(media_group_json["direct_messages_topic_id"], 8);
        assert_eq!(media_group_json["allow_paid_broadcast"], true);
        assert_eq!(media_group_json["message_effect_id"], "effect-2");
        assert!(media_group_json.get("suggested_post_parameters").is_none());

        let mut invalid_direct_topic = SendLocationRequest::new(1_i64, 1.0, 2.0);
        invalid_direct_topic.direct_messages_topic_id = Some(0);
        assert!(matches!(
            invalid_direct_topic.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let mut invalid_effect = SendPhotoRequest::new(1_i64, "photo-file-id");
        invalid_effect.message_effect_id = Some("bad\nid".to_owned());
        assert!(matches!(
            invalid_effect.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let mut invalid_copy_caption_entities =
            CopyMessageRequest::new(1_i64, 2_i64, MessageId(10));
        invalid_copy_caption_entities.parse_mode = Some(ParseMode::Html);
        invalid_copy_caption_entities.caption_entities = Some(Vec::new());
        assert!(matches!(
            invalid_copy_caption_entities.validate(),
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
            business_connection_id: None,
            chat_id: Some(1_i64.into()),
            message_id: Some(MessageId(1)),
            inline_message_id: None,
            reply_markup: Some(crate::types::telegram::InlineKeyboardMarkup::new(Vec::new())),
        };
        assert!(matches!(edit.validate(), Err(Error::InvalidRequest { .. })));

        let stop_poll = StopPollRequest::new(1_i64, MessageId(1))
            .reply_markup(InlineKeyboardMarkup::new(Vec::new()));
        assert!(matches!(
            stop_poll.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let mut stop_poll = StopPollRequest::new(1_i64, MessageId(1));
        stop_poll.business_connection_id = Some("bad\nbusiness".to_owned());
        assert!(matches!(
            stop_poll.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let invalid_business_edit =
            EditMessageTextRequest::for_chat_message(1_i64, MessageId(1), "hello")?
                .business_connection_id("bad\nbusiness");
        assert!(matches!(
            invalid_business_edit.validate(),
            Err(Error::InvalidRequest { .. })
        ));

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

        assert!(matches!(
            EditMessageTextRequest::for_inline_message("bad\nid", "hello"),
            Err(Error::InvalidRequest { .. })
        ));

        let mut partial_edit_target =
            EditMessageTextRequest::for_chat_message(1_i64, MessageId(1), "hello")?;
        partial_edit_target.message_id = None;
        assert!(matches!(
            partial_edit_target.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let mut ambiguous_edit_target =
            EditMessageTextRequest::for_inline_message("inline-id", "hello")?;
        ambiguous_edit_target.chat_id = Some(1_i64.into());
        assert!(matches!(
            ambiguous_edit_target.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let invalid_inline_caption_target = EditMessageCaptionRequest {
            business_connection_id: None,
            chat_id: None,
            message_id: None,
            inline_message_id: Some("bad\nid".to_owned()),
            caption: Some("hello".to_owned()),
            parse_mode: None,
            caption_entities: None,
            reply_markup: None,
            show_caption_above_media: None,
        };
        assert!(matches!(
            invalid_inline_caption_target.validate(),
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

        let attach_photo = SendPhotoRequest::new(1_i64, "attach://photo0");
        assert!(matches!(
            attach_photo.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let mut attach_thumbnail = SendDocumentRequest::new(1_i64, "document-file-id");
        attach_thumbnail.thumbnail = Some("attach://thumb0".to_owned());
        assert!(matches!(
            attach_thumbnail.validate(),
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
                "send_date": valid_suggested_post_send_date()
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

        let input_video = InputMediaVideo::new("video-file-id").thumbnail("thumb-file-id");
        InputMedia::from(input_video.clone()).validate()?;
        let input_video_json = serde_json::to_value(&input_video)
            .map_err(|source| Error::SerializeRequest { source })?;
        assert_eq!(input_video_json["thumbnail"], "thumb-file-id");

        let input_live_photo =
            InputMediaLivePhoto::new("live-photo-file-id", "cover-photo-file-id");
        InputMedia::from(input_live_photo.clone()).validate()?;
        InputMediaGroupItem::from(input_live_photo.clone()).validate()?;
        let input_live_photo_json = serde_json::to_value(InputMedia::from(input_live_photo))
            .map_err(|source| Error::SerializeRequest { source })?;
        assert_eq!(input_live_photo_json["type"], "live_photo");
        assert_eq!(input_live_photo_json["media"], "live-photo-file-id");
        assert_eq!(input_live_photo_json["photo"], "cover-photo-file-id");

        let attach_input_media = InputMedia::from(InputMediaPhoto::new("attach://photo0"));
        assert!(matches!(
            attach_input_media.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let invalid_input_document = InputMedia::from(
            InputMediaDocument::new("document-file-id").thumbnail("bad\nthumbnail"),
        );
        assert!(matches!(
            invalid_input_document.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let photo_media = InputMediaPhoto {
            media: "photo-file-id".to_owned(),
            caption: None,
            parse_mode: None,
            caption_entities: None,
            has_spoiler: None,
        };
        let video_media = InputMediaVideo {
            media: "video-file-id".to_owned(),
            thumbnail: None,
            caption: None,
            parse_mode: None,
            caption_entities: None,
            width: None,
            height: None,
            duration: None,
            supports_streaming: None,
            has_spoiler: None,
        };
        let group =
            SendMediaGroupRequest::new(1_i64, vec![photo_media.into(), video_media.into()])?;
        assert!(group.validate().is_ok());

        let live_photo_group = SendMediaGroupRequest::new(
            1_i64,
            vec![
                InputMediaPhoto::new("photo-file-id").into(),
                InputMediaLivePhoto::new("live-photo-file-id", "cover-photo-file-id").into(),
            ],
        )?;
        assert!(live_photo_group.validate().is_ok());

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
                    caption_entities: None,
                    has_spoiler: None,
                }
                .into(),
                InputMediaVideo {
                    media: "video-file-id".to_owned(),
                    thumbnail: None,
                    caption: None,
                    parse_mode: None,
                    caption_entities: None,
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
        let attach_live_photo_group = SendMediaGroupRequest::new(
            1_i64,
            vec![
                InputMediaPhoto::new("photo-file-id").into(),
                InputMediaLivePhoto::new("live-photo-file-id", "attach://cover0").into(),
            ],
        )?;
        assert!(matches!(
            attach_live_photo_group.validate(),
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

        let thumbnail_upload_group = SendMediaGroupRequest::new(
            1_i64,
            vec![
                InputMediaPhoto::new("photo-file-id").into(),
                InputMediaVideo::new("video-file-id")
                    .thumbnail("attach://thumb0")
                    .into(),
            ],
        )?;
        assert!(matches!(
            thumbnail_upload_group.validate(),
            Err(Error::InvalidRequest { .. })
        ));
        assert!(matches!(
            thumbnail_upload_group.validate_upload(&[]),
            Err(Error::InvalidRequest { .. })
        ));
        assert!(
            thumbnail_upload_group
                .validate_upload(std::slice::from_ref(&thumbnail))
                .is_ok()
        );

        let single_item = SendMediaGroupRequest::new(
            1_i64,
            vec![
                InputMediaPhoto {
                    media: "photo-file-id".to_owned(),
                    caption: None,
                    parse_mode: None,
                    caption_entities: None,
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
            business_connection_id: None,
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

        let invalid_poll = SendPollRequest::new(1_i64, "question", Vec::<&str>::new());
        assert!(matches!(invalid_poll, Err(Error::InvalidRequest { .. })));

        let single_option_poll = SendPollRequest::new(1_i64, "question", ["only"])?;
        assert_eq!(single_option_poll.options.len(), 1);

        let array_poll = SendPollRequest::new(1_i64, "question", ["one", "two"])?;
        assert_eq!(array_poll.options.len(), 2);

        let mut poll =
            SendPollRequest::new(1_i64, "question", vec!["one".to_owned(), "two".to_owned()])?;
        poll.kind = Some(PollKind::Quiz);
        poll.correct_option_ids = Some(vec![2]);
        assert!(matches!(poll.validate(), Err(Error::InvalidRequest { .. })));
        poll.correct_option_ids = Some(vec![0]);
        poll.open_period = Some(MAX_POLL_OPEN_PERIOD_SECONDS);
        assert!(poll.validate().is_ok());
        poll.allow_paid_broadcast = Some(true);
        poll.message_effect_id = Some("effect-1".to_owned());
        poll.description = Some("choose carefully".to_owned());
        poll.media = Some(InputPollMedia::location(37.5, -122.25));
        poll.explanation_media = Some(InputPollMedia::photo("photo-file-id"));
        poll.options[0] = poll.options[0]
            .clone()
            .media(InputPollOptionMedia::sticker("sticker-file-id"));
        poll.country_codes = Some(vec!["US".to_owned(), "FT".to_owned()]);
        poll.allows_revoting = Some(true);
        poll.shuffle_options = Some(true);
        poll.hide_results_until_closes = Some(true);
        poll.members_only = Some(true);
        let poll_json =
            serde_json::to_value(&poll).map_err(|source| Error::SerializeRequest { source })?;
        assert_eq!(poll_json["options"][0]["text"], "one");
        assert_eq!(poll_json["correct_option_ids"], serde_json::json!([0]));
        assert_eq!(poll_json["allow_paid_broadcast"], true);
        assert_eq!(poll_json["message_effect_id"], "effect-1");
        assert_eq!(poll_json["description"], "choose carefully");
        assert_eq!(poll_json["media"]["type"], "location");
        assert_eq!(poll_json["explanation_media"]["type"], "photo");
        assert_eq!(poll_json["explanation_media"]["media"], "photo-file-id");
        assert_eq!(poll_json["options"][0]["media"]["type"], "sticker");
        assert_eq!(poll_json["options"][0]["media"]["media"], "sticker-file-id");
        assert_eq!(poll_json["country_codes"], serde_json::json!(["US", "FT"]));
        assert_eq!(poll_json["allows_revoting"], true);
        assert_eq!(poll_json["shuffle_options"], true);
        assert_eq!(poll_json["hide_results_until_closes"], true);
        assert_eq!(poll_json["members_only"], true);

        poll.open_period = Some(MAX_POLL_OPEN_PERIOD_SECONDS + 1);
        assert!(matches!(poll.validate(), Err(Error::InvalidRequest { .. })));
        poll.open_period = None;
        poll.close_date = Some(1);
        assert!(poll.validate().is_ok());
        poll.open_period = Some(MIN_POLL_OPEN_PERIOD_SECONDS);
        assert!(matches!(poll.validate(), Err(Error::InvalidRequest { .. })));
        poll.open_period = None;
        poll.close_date = None;
        poll.country_codes = Some(vec!["usa".to_owned()]);
        assert!(matches!(poll.validate(), Err(Error::InvalidRequest { .. })));
        poll.country_codes = None;
        poll.media = Some(InputPollMedia::photo("attach://photo0"));
        assert!(matches!(poll.validate(), Err(Error::InvalidRequest { .. })));

        let mut unsupported_kind =
            SendPollRequest::new(1_i64, "question", vec!["one".to_owned(), "two".to_owned()])?;
        unsupported_kind.kind = Some(PollKind::Unknown("future".to_owned()));
        assert!(matches!(
            unsupported_kind.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let mut regular_with_answer =
            SendPollRequest::new(1_i64, "question", vec!["one".to_owned(), "two".to_owned()])?;
        regular_with_answer.correct_option_ids = Some(vec![0]);
        assert!(matches!(
            regular_with_answer.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let mut quiz_without_answer =
            SendPollRequest::new(1_i64, "question", vec!["one".to_owned(), "two".to_owned()])?;
        quiz_without_answer.kind = Some(PollKind::Quiz);
        assert!(matches!(
            quiz_without_answer.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let mut add_options_poll =
            SendPollRequest::new(1_i64, "question", vec!["one".to_owned(), "two".to_owned()])?;
        add_options_poll.allow_adding_options = Some(true);
        assert!(matches!(
            add_options_poll.validate(),
            Err(Error::InvalidRequest { .. })
        ));
        add_options_poll.is_anonymous = Some(false);
        assert!(add_options_poll.validate().is_ok());
        let add_options_json = serde_json::to_value(&add_options_poll)
            .map_err(|source| Error::SerializeRequest { source })?;
        assert_eq!(add_options_json["allow_adding_options"], true);
        add_options_poll.kind = Some(PollKind::Quiz);
        add_options_poll.correct_option_ids = Some(vec![0]);
        assert!(matches!(
            add_options_poll.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let mut quiz_with_unsorted_answers = quiz_without_answer;
        quiz_with_unsorted_answers.correct_option_ids = Some(vec![1, 0]);
        assert!(matches!(
            quiz_with_unsorted_answers.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let mut poll_with_bad_explanation =
            SendPollRequest::new(1_i64, "question", vec!["one".to_owned(), "two".to_owned()])?;
        poll_with_bad_explanation.explanation = Some("a\nb\nc\nd".to_owned());
        assert!(matches!(
            poll_with_bad_explanation.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let mut poll_with_entity_conflict =
            SendPollRequest::new(1_i64, "question", vec!["one".to_owned(), "two".to_owned()])?;
        poll_with_entity_conflict.question_parse_mode = Some(ParseMode::Html);
        poll_with_entity_conflict.question_entities = Some(Vec::new());
        assert!(matches!(
            poll_with_entity_conflict.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let mut poll_with_option_entity_conflict =
            SendPollRequest::new(1_i64, "question", vec!["one".to_owned(), "two".to_owned()])?;
        poll_with_option_entity_conflict.options[0] = InputPollOption::new("one")
            .text_parse_mode(ParseMode::Html)
            .text_entities(Vec::new());
        assert!(matches!(
            poll_with_option_entity_conflict.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let too_many_ids = (0..=MAX_BULK_MESSAGE_IDS)
            .map(|id| MessageId(id as i64))
            .collect::<Vec<_>>();
        assert!(matches!(
            DeleteMessagesRequest::new(1_i64, too_many_ids),
            Err(Error::InvalidRequest { .. })
        ));
        assert!(matches!(
            CopyMessagesRequest::new(1_i64, 2_i64, vec![MessageId(10), MessageId(10)]),
            Err(Error::InvalidRequest { .. })
        ));
        assert!(matches!(
            CopyMessagesRequest::new(1_i64, 2_i64, vec![MessageId(11), MessageId(10)]),
            Err(Error::InvalidRequest { .. })
        ));
        assert!(matches!(
            DeleteMessagesRequest::new(1_i64, vec![MessageId(20), MessageId(20)]),
            Err(Error::InvalidRequest { .. })
        ));
        Ok(())
    }
}

use super::*;
use crate::types::InputMedia;

/// Request-scoped runtime facade for handler code.
///
/// This is the preferred entry point inside handlers. It mirrors the stable runtime surface of
/// [`crate::client::AppApi`], but keeps the call path centered on `context.app()` so handler code
/// reads naturally and stays on the business/runtime plane.
#[derive(Clone)]
pub struct ContextAppApi {
    client: Client,
}

impl ContextAppApi {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Returns the governance-oriented moderation facade for handler-side actions.
    pub fn moderation(&self) -> crate::client::ModerationApi {
        self.client.app().moderation()
    }

    /// Returns the membership/capability facade used by install and bind pre-check flows.
    pub fn membership(&self) -> crate::client::MembershipApi {
        self.client.app().membership()
    }

    /// Returns the dedicated Web App facade for runtime query handling.
    pub fn web_app(&self) -> crate::client::WebAppApi {
        self.client.app().web_app()
    }

    /// Starts a callback-answer builder.
    ///
    /// Prefer this when callback replies need options such as `show_alert`, `url`, or
    /// `cache_time`.
    pub fn callback_answer(
        &self,
        callback_query_id: impl Into<String>,
    ) -> crate::client::CallbackAnswerBuilder {
        self.client.app().callback_answer(callback_query_id)
    }

    /// Starts a callback-answer builder using the callback id extracted from an update.
    pub fn callback_answer_from_update(
        &self,
        update: &Update,
    ) -> Result<crate::client::CallbackAnswerBuilder> {
        self.client.app().callback_answer_from_update(update)
    }

    /// Starts a text-send builder for a target chat.
    pub fn text(
        &self,
        chat_id: impl Into<ChatId>,
        text: impl Into<String>,
    ) -> Result<crate::client::TextSendBuilder> {
        self.client.app().text(chat_id, text)
    }

    /// Starts a text-send builder using the canonical reply chat derived from an update.
    pub fn reply(
        &self,
        update: &Update,
        text: impl Into<String>,
    ) -> Result<crate::client::TextSendBuilder> {
        self.client.app().reply(update, text)
    }

    /// Starts a photo-send builder for a target chat.
    pub fn photo(
        &self,
        chat_id: impl Into<ChatId>,
        photo: impl Into<String>,
    ) -> crate::client::PhotoSendBuilder {
        self.client.app().photo(chat_id, photo)
    }

    /// Starts a photo-send builder using the canonical reply chat derived from an update.
    pub fn reply_photo(
        &self,
        update: &Update,
        photo: impl Into<String>,
    ) -> Result<crate::client::PhotoSendBuilder> {
        self.client.app().reply_photo(update, photo)
    }

    /// Starts a document-send builder for a target chat.
    pub fn document(
        &self,
        chat_id: impl Into<ChatId>,
        document: impl Into<String>,
    ) -> crate::client::DocumentSendBuilder {
        self.client.app().document(chat_id, document)
    }

    /// Starts a document-send builder using the canonical reply chat derived from an update.
    pub fn reply_document(
        &self,
        update: &Update,
        document: impl Into<String>,
    ) -> Result<crate::client::DocumentSendBuilder> {
        self.client.app().reply_document(update, document)
    }

    /// Starts a video-send builder for a target chat.
    pub fn video(
        &self,
        chat_id: impl Into<ChatId>,
        video: impl Into<String>,
    ) -> crate::client::VideoSendBuilder {
        self.client.app().video(chat_id, video)
    }

    /// Starts a video-send builder using the canonical reply chat derived from an update.
    pub fn reply_video(
        &self,
        update: &Update,
        video: impl Into<String>,
    ) -> Result<crate::client::VideoSendBuilder> {
        self.client.app().reply_video(update, video)
    }

    /// Starts an audio-send builder for a target chat.
    pub fn audio(
        &self,
        chat_id: impl Into<ChatId>,
        audio: impl Into<String>,
    ) -> crate::client::AudioSendBuilder {
        self.client.app().audio(chat_id, audio)
    }

    /// Starts an audio-send builder using the canonical reply chat derived from an update.
    pub fn reply_audio(
        &self,
        update: &Update,
        audio: impl Into<String>,
    ) -> Result<crate::client::AudioSendBuilder> {
        self.client.app().reply_audio(update, audio)
    }

    /// Starts an animation-send builder for a target chat.
    pub fn animation(
        &self,
        chat_id: impl Into<ChatId>,
        animation: impl Into<String>,
    ) -> crate::client::AnimationSendBuilder {
        self.client.app().animation(chat_id, animation)
    }

    /// Starts an animation-send builder using the canonical reply chat derived from an update.
    pub fn reply_animation(
        &self,
        update: &Update,
        animation: impl Into<String>,
    ) -> Result<crate::client::AnimationSendBuilder> {
        self.client.app().reply_animation(update, animation)
    }

    /// Starts a voice-send builder for a target chat.
    pub fn voice(
        &self,
        chat_id: impl Into<ChatId>,
        voice: impl Into<String>,
    ) -> crate::client::VoiceSendBuilder {
        self.client.app().voice(chat_id, voice)
    }

    /// Starts a voice-send builder using the canonical reply chat derived from an update.
    pub fn reply_voice(
        &self,
        update: &Update,
        voice: impl Into<String>,
    ) -> Result<crate::client::VoiceSendBuilder> {
        self.client.app().reply_voice(update, voice)
    }

    /// Starts a sticker-send builder for a target chat.
    pub fn sticker(
        &self,
        chat_id: impl Into<ChatId>,
        sticker: impl Into<String>,
    ) -> crate::client::StickerSendBuilder {
        self.client.app().sticker(chat_id, sticker)
    }

    /// Starts a sticker-send builder using the canonical reply chat derived from an update.
    pub fn reply_sticker(
        &self,
        update: &Update,
        sticker: impl Into<String>,
    ) -> Result<crate::client::StickerSendBuilder> {
        self.client.app().reply_sticker(update, sticker)
    }

    /// Starts a media-group builder for a target chat.
    pub fn media_group<I, M>(
        &self,
        chat_id: impl Into<ChatId>,
        media: I,
    ) -> Result<crate::client::MediaGroupSendBuilder>
    where
        I: IntoIterator<Item = M>,
        M: Into<InputMedia>,
    {
        self.client.app().media_group(chat_id, media)
    }

    /// Starts a media-group builder using the canonical reply chat derived from an update.
    pub fn reply_media_group<I, M>(
        &self,
        update: &Update,
        media: I,
    ) -> Result<crate::client::MediaGroupSendBuilder>
    where
        I: IntoIterator<Item = M>,
        M: Into<InputMedia>,
    {
        self.client.app().reply_media_group(update, media)
    }

    /// Shortcut for `text(...).send().await`.
    pub async fn send_text(
        &self,
        chat_id: impl Into<ChatId>,
        text: impl Into<String>,
    ) -> Result<Message> {
        self.client.app().send_text(chat_id, text).await
    }

    /// Shortcut for `reply(...).send().await`.
    pub async fn reply_text(&self, update: &Update, text: impl Into<String>) -> Result<Message> {
        self.client.app().reply_text(update, text).await
    }

    /// Shortcut for `callback_answer(...).text_optional(...).send().await`.
    pub async fn answer_callback(
        &self,
        callback_query_id: impl Into<String>,
        text: Option<String>,
    ) -> Result<bool> {
        self.client
            .app()
            .answer_callback(callback_query_id, text)
            .await
    }

    /// Shortcut for `callback_answer_from_update(...).text_optional(...).send().await`.
    pub async fn answer_callback_from_update(
        &self,
        update: &Update,
        text: Option<String>,
    ) -> Result<bool> {
        self.client
            .app()
            .answer_callback_from_update(update, text)
            .await
    }
}

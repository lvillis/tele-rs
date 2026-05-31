use super::*;
use crate::types::{ChatAction, InputMediaGroupItem, InputPollOption, MessageId};

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

    /// Starts a text-send builder using the update reply target and quoting its source message when present.
    pub fn reply(
        &self,
        update: &Update,
        text: impl Into<String>,
    ) -> Result<crate::client::TextSendBuilder> {
        self.client.app().reply(update, text)
    }

    /// Starts a location-send builder for a target chat.
    pub fn location(
        &self,
        chat_id: impl Into<ChatId>,
        latitude: f64,
        longitude: f64,
    ) -> crate::client::LocationSendBuilder {
        self.client.app().location(chat_id, latitude, longitude)
    }

    /// Starts a location-send builder using the update reply target and quoting its source message when present.
    pub fn reply_location(
        &self,
        update: &Update,
        latitude: f64,
        longitude: f64,
    ) -> Result<crate::client::LocationSendBuilder> {
        self.client
            .app()
            .reply_location(update, latitude, longitude)
    }

    /// Starts a venue-send builder for a target chat.
    pub fn venue(
        &self,
        chat_id: impl Into<ChatId>,
        latitude: f64,
        longitude: f64,
        title: impl Into<String>,
        address: impl Into<String>,
    ) -> crate::client::VenueSendBuilder {
        self.client
            .app()
            .venue(chat_id, latitude, longitude, title, address)
    }

    /// Starts a venue-send builder using the update reply target and quoting its source message when present.
    pub fn reply_venue(
        &self,
        update: &Update,
        latitude: f64,
        longitude: f64,
        title: impl Into<String>,
        address: impl Into<String>,
    ) -> Result<crate::client::VenueSendBuilder> {
        self.client
            .app()
            .reply_venue(update, latitude, longitude, title, address)
    }

    /// Starts a contact-send builder for a target chat.
    pub fn contact(
        &self,
        chat_id: impl Into<ChatId>,
        phone_number: impl Into<String>,
        first_name: impl Into<String>,
    ) -> crate::client::ContactSendBuilder {
        self.client.app().contact(chat_id, phone_number, first_name)
    }

    /// Starts a contact-send builder using the update reply target and quoting its source message when present.
    pub fn reply_contact(
        &self,
        update: &Update,
        phone_number: impl Into<String>,
        first_name: impl Into<String>,
    ) -> Result<crate::client::ContactSendBuilder> {
        self.client
            .app()
            .reply_contact(update, phone_number, first_name)
    }

    /// Starts a poll-send builder for a target chat.
    pub fn poll(
        &self,
        chat_id: impl Into<ChatId>,
        question: impl Into<String>,
        options: impl IntoIterator<Item = impl Into<InputPollOption>>,
    ) -> Result<crate::client::PollSendBuilder> {
        self.client.app().poll(chat_id, question, options)
    }

    /// Starts a poll-send builder using the update reply target and quoting its source message when present.
    pub fn reply_poll(
        &self,
        update: &Update,
        question: impl Into<String>,
        options: impl IntoIterator<Item = impl Into<InputPollOption>>,
    ) -> Result<crate::client::PollSendBuilder> {
        self.client.app().reply_poll(update, question, options)
    }

    /// Starts a stop-poll builder for a target chat and message.
    pub fn stop_poll(
        &self,
        chat_id: impl Into<ChatId>,
        message_id: MessageId,
    ) -> crate::client::StopPollBuilder {
        self.client.app().stop_poll(chat_id, message_id)
    }

    /// Starts a dice-send builder for a target chat.
    pub fn dice(&self, chat_id: impl Into<ChatId>) -> crate::client::DiceSendBuilder {
        self.client.app().dice(chat_id)
    }

    /// Starts a dice-send builder using the update reply target and quoting its source message when present.
    pub fn reply_dice(&self, update: &Update) -> Result<crate::client::DiceSendBuilder> {
        self.client.app().reply_dice(update)
    }

    /// Starts a chat-action builder for a target chat.
    pub fn chat_action(
        &self,
        chat_id: impl Into<ChatId>,
        action: ChatAction,
    ) -> crate::client::ChatActionBuilder {
        self.client.app().chat_action(chat_id, action)
    }

    /// Starts a chat-action builder using the update reply target.
    pub fn chat_action_for_update(
        &self,
        update: &Update,
        action: ChatAction,
    ) -> Result<crate::client::ChatActionBuilder> {
        self.client.app().chat_action_for_update(update, action)
    }

    /// Starts a photo-send builder for a target chat.
    pub fn photo(
        &self,
        chat_id: impl Into<ChatId>,
        photo: impl Into<String>,
    ) -> crate::client::PhotoSendBuilder {
        self.client.app().photo(chat_id, photo)
    }

    /// Starts a photo-upload builder for a target chat.
    pub fn photo_upload(&self, chat_id: impl Into<ChatId>) -> crate::client::PhotoUploadBuilder {
        self.client.app().photo_upload(chat_id)
    }

    /// Starts a photo-send builder using the update reply target and quoting its source message when present.
    pub fn reply_photo(
        &self,
        update: &Update,
        photo: impl Into<String>,
    ) -> Result<crate::client::PhotoSendBuilder> {
        self.client.app().reply_photo(update, photo)
    }

    /// Starts a photo-upload builder using the update reply target and quoting its source message when present.
    pub fn reply_photo_upload(&self, update: &Update) -> Result<crate::client::PhotoUploadBuilder> {
        self.client.app().reply_photo_upload(update)
    }

    /// Starts a document-send builder for a target chat.
    pub fn document(
        &self,
        chat_id: impl Into<ChatId>,
        document: impl Into<String>,
    ) -> crate::client::DocumentSendBuilder {
        self.client.app().document(chat_id, document)
    }

    /// Starts a document-upload builder for a target chat.
    pub fn document_upload(
        &self,
        chat_id: impl Into<ChatId>,
    ) -> crate::client::DocumentUploadBuilder {
        self.client.app().document_upload(chat_id)
    }

    /// Starts a document-send builder using the update reply target and quoting its source message when present.
    pub fn reply_document(
        &self,
        update: &Update,
        document: impl Into<String>,
    ) -> Result<crate::client::DocumentSendBuilder> {
        self.client.app().reply_document(update, document)
    }

    /// Starts a document-upload builder using the update reply target and quoting its source message when present.
    pub fn reply_document_upload(
        &self,
        update: &Update,
    ) -> Result<crate::client::DocumentUploadBuilder> {
        self.client.app().reply_document_upload(update)
    }

    /// Starts a video-send builder for a target chat.
    pub fn video(
        &self,
        chat_id: impl Into<ChatId>,
        video: impl Into<String>,
    ) -> crate::client::VideoSendBuilder {
        self.client.app().video(chat_id, video)
    }

    /// Starts a video-upload builder for a target chat.
    pub fn video_upload(&self, chat_id: impl Into<ChatId>) -> crate::client::VideoUploadBuilder {
        self.client.app().video_upload(chat_id)
    }

    /// Starts a video-send builder using the update reply target and quoting its source message when present.
    pub fn reply_video(
        &self,
        update: &Update,
        video: impl Into<String>,
    ) -> Result<crate::client::VideoSendBuilder> {
        self.client.app().reply_video(update, video)
    }

    /// Starts a video-upload builder using the update reply target and quoting its source message when present.
    pub fn reply_video_upload(&self, update: &Update) -> Result<crate::client::VideoUploadBuilder> {
        self.client.app().reply_video_upload(update)
    }

    /// Starts an audio-send builder for a target chat.
    pub fn audio(
        &self,
        chat_id: impl Into<ChatId>,
        audio: impl Into<String>,
    ) -> crate::client::AudioSendBuilder {
        self.client.app().audio(chat_id, audio)
    }

    /// Starts an audio-upload builder for a target chat.
    pub fn audio_upload(&self, chat_id: impl Into<ChatId>) -> crate::client::AudioUploadBuilder {
        self.client.app().audio_upload(chat_id)
    }

    /// Starts an audio-send builder using the update reply target and quoting its source message when present.
    pub fn reply_audio(
        &self,
        update: &Update,
        audio: impl Into<String>,
    ) -> Result<crate::client::AudioSendBuilder> {
        self.client.app().reply_audio(update, audio)
    }

    /// Starts an audio-upload builder using the update reply target and quoting its source message when present.
    pub fn reply_audio_upload(&self, update: &Update) -> Result<crate::client::AudioUploadBuilder> {
        self.client.app().reply_audio_upload(update)
    }

    /// Starts an animation-send builder for a target chat.
    pub fn animation(
        &self,
        chat_id: impl Into<ChatId>,
        animation: impl Into<String>,
    ) -> crate::client::AnimationSendBuilder {
        self.client.app().animation(chat_id, animation)
    }

    /// Starts an animation-upload builder for a target chat.
    pub fn animation_upload(
        &self,
        chat_id: impl Into<ChatId>,
    ) -> crate::client::AnimationUploadBuilder {
        self.client.app().animation_upload(chat_id)
    }

    /// Starts an animation-send builder using the update reply target and quoting its source message when present.
    pub fn reply_animation(
        &self,
        update: &Update,
        animation: impl Into<String>,
    ) -> Result<crate::client::AnimationSendBuilder> {
        self.client.app().reply_animation(update, animation)
    }

    /// Starts an animation-upload builder using the update reply target and quoting its source message when present.
    pub fn reply_animation_upload(
        &self,
        update: &Update,
    ) -> Result<crate::client::AnimationUploadBuilder> {
        self.client.app().reply_animation_upload(update)
    }

    /// Starts a voice-send builder for a target chat.
    pub fn voice(
        &self,
        chat_id: impl Into<ChatId>,
        voice: impl Into<String>,
    ) -> crate::client::VoiceSendBuilder {
        self.client.app().voice(chat_id, voice)
    }

    /// Starts a voice-upload builder for a target chat.
    pub fn voice_upload(&self, chat_id: impl Into<ChatId>) -> crate::client::VoiceUploadBuilder {
        self.client.app().voice_upload(chat_id)
    }

    /// Starts a voice-send builder using the update reply target and quoting its source message when present.
    pub fn reply_voice(
        &self,
        update: &Update,
        voice: impl Into<String>,
    ) -> Result<crate::client::VoiceSendBuilder> {
        self.client.app().reply_voice(update, voice)
    }

    /// Starts a voice-upload builder using the update reply target and quoting its source message when present.
    pub fn reply_voice_upload(&self, update: &Update) -> Result<crate::client::VoiceUploadBuilder> {
        self.client.app().reply_voice_upload(update)
    }

    /// Starts a video-note-send builder for a target chat.
    pub fn video_note(
        &self,
        chat_id: impl Into<ChatId>,
        video_note: impl Into<String>,
    ) -> crate::client::VideoNoteSendBuilder {
        self.client.app().video_note(chat_id, video_note)
    }

    /// Starts a video-note-upload builder for a target chat.
    pub fn video_note_upload(
        &self,
        chat_id: impl Into<ChatId>,
    ) -> crate::client::VideoNoteUploadBuilder {
        self.client.app().video_note_upload(chat_id)
    }

    /// Starts a video-note-send builder using the update reply target and quoting its source message when present.
    pub fn reply_video_note(
        &self,
        update: &Update,
        video_note: impl Into<String>,
    ) -> Result<crate::client::VideoNoteSendBuilder> {
        self.client.app().reply_video_note(update, video_note)
    }

    /// Starts a video-note-upload builder using the update reply target and quoting its source message when present.
    pub fn reply_video_note_upload(
        &self,
        update: &Update,
    ) -> Result<crate::client::VideoNoteUploadBuilder> {
        self.client.app().reply_video_note_upload(update)
    }

    /// Starts a sticker-send builder for a target chat.
    pub fn sticker(
        &self,
        chat_id: impl Into<ChatId>,
        sticker: impl Into<String>,
    ) -> crate::client::StickerSendBuilder {
        self.client.app().sticker(chat_id, sticker)
    }

    /// Starts a sticker-upload builder for a target chat.
    pub fn sticker_upload(
        &self,
        chat_id: impl Into<ChatId>,
    ) -> crate::client::StickerUploadBuilder {
        self.client.app().sticker_upload(chat_id)
    }

    /// Starts a sticker-send builder using the update reply target and quoting its source message when present.
    pub fn reply_sticker(
        &self,
        update: &Update,
        sticker: impl Into<String>,
    ) -> Result<crate::client::StickerSendBuilder> {
        self.client.app().reply_sticker(update, sticker)
    }

    /// Starts a sticker-upload builder using the update reply target and quoting its source message when present.
    pub fn reply_sticker_upload(
        &self,
        update: &Update,
    ) -> Result<crate::client::StickerUploadBuilder> {
        self.client.app().reply_sticker_upload(update)
    }

    /// Starts a media-group builder for a target chat.
    pub fn media_group<I, M>(
        &self,
        chat_id: impl Into<ChatId>,
        media: I,
    ) -> Result<crate::client::MediaGroupSendBuilder>
    where
        I: IntoIterator<Item = M>,
        M: Into<InputMediaGroupItem>,
    {
        self.client.app().media_group(chat_id, media)
    }

    /// Starts a media-group upload builder for a target chat.
    pub fn media_group_upload<I, M>(
        &self,
        chat_id: impl Into<ChatId>,
        media: I,
    ) -> Result<crate::client::MediaGroupUploadBuilder>
    where
        I: IntoIterator<Item = M>,
        M: Into<InputMediaGroupItem>,
    {
        self.client.app().media_group_upload(chat_id, media)
    }

    /// Starts a media-group builder using the update reply target and quoting its source message when present.
    pub fn reply_media_group<I, M>(
        &self,
        update: &Update,
        media: I,
    ) -> Result<crate::client::MediaGroupSendBuilder>
    where
        I: IntoIterator<Item = M>,
        M: Into<InputMediaGroupItem>,
    {
        self.client.app().reply_media_group(update, media)
    }

    /// Starts a media-group upload builder using the update reply target and quoting its source message when present.
    pub fn reply_media_group_upload<I, M>(
        &self,
        update: &Update,
        media: I,
    ) -> Result<crate::client::MediaGroupUploadBuilder>
    where
        I: IntoIterator<Item = M>,
        M: Into<InputMediaGroupItem>,
    {
        self.client.app().reply_media_group_upload(update, media)
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

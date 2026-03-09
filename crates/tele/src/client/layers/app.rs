use super::support::{callback_query_id, reply_chat_id, serialize_request_value};
use super::*;

fn text_send_request(
    chat_id: impl Into<ChatId>,
    text: impl Into<String>,
) -> Result<SendMessageRequest> {
    SendMessageRequest::new(chat_id, text)
}

fn reply_text_request(update: &Update, text: impl Into<String>) -> Result<SendMessageRequest> {
    let chat_id = reply_chat_id(update)?;
    text_send_request(chat_id, text)
}

fn photo_send_request(chat_id: impl Into<ChatId>, photo: impl Into<String>) -> SendPhotoRequest {
    SendPhotoRequest::new(chat_id, photo)
}

fn reply_photo_request(update: &Update, photo: impl Into<String>) -> Result<SendPhotoRequest> {
    let chat_id = reply_chat_id(update)?;
    Ok(photo_send_request(chat_id, photo))
}

fn document_send_request(
    chat_id: impl Into<ChatId>,
    document: impl Into<String>,
) -> SendDocumentRequest {
    SendDocumentRequest::new(chat_id, document)
}

fn reply_document_request(
    update: &Update,
    document: impl Into<String>,
) -> Result<SendDocumentRequest> {
    let chat_id = reply_chat_id(update)?;
    Ok(document_send_request(chat_id, document))
}

fn video_send_request(chat_id: impl Into<ChatId>, video: impl Into<String>) -> SendVideoRequest {
    SendVideoRequest::new(chat_id, video)
}

fn reply_video_request(update: &Update, video: impl Into<String>) -> Result<SendVideoRequest> {
    let chat_id = reply_chat_id(update)?;
    Ok(video_send_request(chat_id, video))
}

fn audio_send_request(chat_id: impl Into<ChatId>, audio: impl Into<String>) -> SendAudioRequest {
    SendAudioRequest::new(chat_id, audio)
}

fn reply_audio_request(update: &Update, audio: impl Into<String>) -> Result<SendAudioRequest> {
    let chat_id = reply_chat_id(update)?;
    Ok(audio_send_request(chat_id, audio))
}

fn animation_send_request(
    chat_id: impl Into<ChatId>,
    animation: impl Into<String>,
) -> SendAnimationRequest {
    SendAnimationRequest::new(chat_id, animation)
}

fn reply_animation_request(
    update: &Update,
    animation: impl Into<String>,
) -> Result<SendAnimationRequest> {
    let chat_id = reply_chat_id(update)?;
    Ok(animation_send_request(chat_id, animation))
}

fn voice_send_request(chat_id: impl Into<ChatId>, voice: impl Into<String>) -> SendVoiceRequest {
    SendVoiceRequest::new(chat_id, voice)
}

fn reply_voice_request(update: &Update, voice: impl Into<String>) -> Result<SendVoiceRequest> {
    let chat_id = reply_chat_id(update)?;
    Ok(voice_send_request(chat_id, voice))
}

fn sticker_send_request(
    chat_id: impl Into<ChatId>,
    sticker: impl Into<String>,
) -> SendStickerRequest {
    SendStickerRequest::new(chat_id, sticker)
}

fn reply_sticker_request(
    update: &Update,
    sticker: impl Into<String>,
) -> Result<SendStickerRequest> {
    let chat_id = reply_chat_id(update)?;
    Ok(sticker_send_request(chat_id, sticker))
}

fn media_group_send_request<I, M>(
    chat_id: impl Into<ChatId>,
    media: I,
) -> Result<SendMediaGroupRequest>
where
    I: IntoIterator<Item = M>,
    M: Into<InputMedia>,
{
    SendMediaGroupRequest::new(chat_id, media.into_iter().map(Into::into).collect())
}

fn reply_media_group_request<I, M>(update: &Update, media: I) -> Result<SendMediaGroupRequest>
where
    I: IntoIterator<Item = M>,
    M: Into<InputMedia>,
{
    let chat_id = reply_chat_id(update)?;
    media_group_send_request(chat_id, media)
}

fn callback_answer_request(
    callback_query_id: impl Into<String>,
    text: Option<String>,
) -> AnswerCallbackQueryRequest {
    AnswerCallbackQueryRequest {
        callback_query_id: callback_query_id.into(),
        text,
        show_alert: None,
        url: None,
        cache_time: None,
    }
}

macro_rules! impl_common_callback_answer_builder_methods {
    ($builder:ident, $request_ty:ty) => {
        impl $builder {
            /// Sets the callback answer text shown to the user.
            pub fn text(mut self, text: impl Into<String>) -> Self {
                self.request.text = Some(text.into());
                self
            }

            /// Sets the callback answer text, or clears it when `None`.
            pub fn text_optional(mut self, text: Option<String>) -> Self {
                self.request.text = text;
                self
            }

            /// Shows the callback answer as an alert dialog instead of a toast.
            pub fn show_alert(mut self, enabled: bool) -> Self {
                self.request.show_alert = enabled.then_some(true);
                self
            }

            /// Redirects the user to a URL after the callback answer is acknowledged.
            pub fn url(mut self, url: impl Into<String>) -> Self {
                self.request.url = Some(url.into());
                self
            }

            /// Sets Telegram-side caching for identical callback answers.
            pub fn cache_time(mut self, cache_time: u32) -> Self {
                self.request.cache_time = Some(cache_time);
                self
            }

            /// Returns the typed request for lower-level reuse or inspection.
            pub fn into_request(self) -> $request_ty {
                self.request
            }
        }
    };
}

macro_rules! impl_common_media_builder_methods {
    ($builder:ident, $request_ty:ty) => {
        impl $builder {
            /// Sets the media caption.
            pub fn caption(mut self, caption: impl Into<String>) -> Self {
                self.request.caption = Some(caption.into());
                self
            }

            /// Sets caption parse mode.
            pub fn parse_mode(mut self, parse_mode: ParseMode) -> Self {
                self.request.parse_mode = Some(parse_mode);
                self
            }

            /// Attaches reply markup such as an inline keyboard.
            pub fn reply_markup(mut self, reply_markup: impl Into<ReplyMarkup>) -> Self {
                self.request.reply_markup = Some(reply_markup.into());
                self
            }

            /// Sets explicit reply parameters.
            pub fn reply_parameters(mut self, reply_parameters: ReplyParameters) -> Self {
                self.request.reply_parameters = Some(reply_parameters);
                self
            }

            /// Replies to a concrete message by id.
            pub fn reply_to_message(mut self, message_id: MessageId) -> Self {
                self.request.reply_parameters = Some(ReplyParameters::new(message_id));
                self
            }

            /// Targets a forum topic / message thread when applicable.
            pub fn message_thread_id(mut self, message_thread_id: i64) -> Self {
                self.request.message_thread_id = Some(message_thread_id);
                self
            }

            /// Sends silently when `true`.
            pub fn disable_notification(mut self, enabled: bool) -> Self {
                self.request.disable_notification = enabled.then_some(true);
                self
            }

            /// Protects the sent message from forwarding and saving when `true`.
            pub fn protect_content(mut self, enabled: bool) -> Self {
                self.request.protect_content = enabled.then_some(true);
                self
            }

            /// Returns the typed request for lower-level reuse or inspection.
            pub fn into_request(self) -> $request_ty {
                self.request
            }
        }
    };
}

macro_rules! impl_common_media_group_builder_methods {
    ($builder:ident, $request_ty:ty) => {
        impl $builder {
            /// Appends one more media item to the group.
            pub fn add_media(mut self, media: impl Into<InputMedia>) -> Self {
                self.request.media.push(media.into());
                self
            }

            /// Sets explicit reply parameters for the whole media group.
            pub fn reply_parameters(mut self, reply_parameters: ReplyParameters) -> Self {
                self.request.reply_parameters = Some(reply_parameters);
                self
            }

            /// Replies to a concrete message by id.
            pub fn reply_to_message(mut self, message_id: MessageId) -> Self {
                self.request.reply_parameters = Some(ReplyParameters::new(message_id));
                self
            }

            /// Targets a forum topic / message thread when applicable.
            pub fn message_thread_id(mut self, message_thread_id: i64) -> Self {
                self.request.message_thread_id = Some(message_thread_id);
                self
            }

            /// Sends silently when `true`.
            pub fn disable_notification(mut self, enabled: bool) -> Self {
                self.request.disable_notification = enabled.then_some(true);
                self
            }

            /// Protects the sent media group from forwarding and saving when `true`.
            pub fn protect_content(mut self, enabled: bool) -> Self {
                self.request.protect_content = enabled.then_some(true);
                self
            }

            /// Returns the typed request for lower-level reuse or inspection.
            pub fn into_request(self) -> $request_ty {
                self.request
            }
        }
    };
}

macro_rules! impl_common_sticker_builder_methods {
    ($builder:ident, $request_ty:ty) => {
        impl $builder {
            /// Sets the optional emoji associated with the sticker send.
            pub fn emoji(mut self, emoji: impl Into<String>) -> Self {
                self.request.emoji = Some(emoji.into());
                self
            }

            /// Attaches reply markup such as an inline keyboard.
            pub fn reply_markup(mut self, reply_markup: impl Into<ReplyMarkup>) -> Result<Self> {
                self.request.reply_markup = Some(serialize_request_value(reply_markup.into())?);
                Ok(self)
            }

            /// Sets explicit reply parameters.
            pub fn reply_parameters(mut self, reply_parameters: ReplyParameters) -> Result<Self> {
                self.request.reply_parameters = Some(serialize_request_value(reply_parameters)?);
                Ok(self)
            }

            /// Replies to a concrete message by id.
            pub fn reply_to_message(mut self, message_id: MessageId) -> Result<Self> {
                self.request.reply_parameters =
                    Some(serialize_request_value(ReplyParameters::new(message_id))?);
                Ok(self)
            }

            /// Targets a forum topic / message thread when applicable.
            pub fn message_thread_id(mut self, message_thread_id: i64) -> Self {
                self.request.message_thread_id = Some(message_thread_id);
                self
            }

            /// Sends silently when `true`.
            pub fn disable_notification(mut self, enabled: bool) -> Self {
                self.request.disable_notification = enabled.then_some(true);
                self
            }

            /// Protects the sent sticker from forwarding and saving when `true`.
            pub fn protect_content(mut self, enabled: bool) -> Self {
                self.request.protect_content = enabled.then_some(true);
                self
            }

            /// Returns the typed request for lower-level reuse or inspection.
            pub fn into_request(self) -> $request_ty {
                self.request
            }
        }
    };
}

/// Stable builder for high-level callback answers on the async app facade.
///
/// Start this from [`AppApi::callback_answer`] or [`AppApi::callback_answer_from_update`] when
/// you need more than the shortcut `answer_callback(...)` helpers expose.
#[cfg(feature = "_async")]
#[derive(Clone)]
#[must_use = "call `.send().await` or `.into_request()` to finish the callback answer"]
pub struct CallbackAnswerBuilder {
    client: Client,
    request: AnswerCallbackQueryRequest,
}

#[cfg(feature = "_async")]
impl CallbackAnswerBuilder {
    fn new(client: Client, request: AnswerCallbackQueryRequest) -> Self {
        Self { client, request }
    }

    /// Sends the callback answer request.
    pub async fn send(self) -> Result<bool> {
        self.client
            .updates()
            .answer_callback_query(&self.request)
            .await
    }
}

#[cfg(feature = "_async")]
impl_common_callback_answer_builder_methods!(CallbackAnswerBuilder, AnswerCallbackQueryRequest);

/// Stable builder for high-level text sends on the async app facade.
///
/// Start this from [`AppApi::text`] or [`AppApi::reply`] for the common message-send path.
#[cfg(feature = "_async")]
#[derive(Clone)]
#[must_use = "call `.send().await` or `.into_request()` to finish the message send"]
pub struct TextSendBuilder {
    client: Client,
    request: SendMessageRequest,
}

#[cfg(feature = "_async")]
impl TextSendBuilder {
    fn new(client: Client, request: SendMessageRequest) -> Self {
        Self { client, request }
    }

    /// Sets text parse mode.
    pub fn parse_mode(mut self, parse_mode: ParseMode) -> Self {
        self.request = self.request.parse_mode(parse_mode);
        self
    }

    /// Attaches reply markup such as an inline keyboard.
    pub fn reply_markup(mut self, reply_markup: impl Into<ReplyMarkup>) -> Self {
        self.request = self.request.reply_markup(reply_markup);
        self
    }

    /// Sets explicit reply parameters.
    pub fn reply_parameters(mut self, reply_parameters: ReplyParameters) -> Self {
        self.request = self.request.reply_parameters(reply_parameters);
        self
    }

    /// Replies to a concrete message by id.
    pub fn reply_to_message(mut self, message_id: MessageId) -> Self {
        self.request = self.request.reply_to_message(message_id);
        self
    }

    /// Targets a forum topic / message thread when applicable.
    pub fn message_thread_id(mut self, message_thread_id: i64) -> Self {
        self.request.message_thread_id = Some(message_thread_id);
        self
    }

    /// Sends silently when `true`.
    pub fn disable_notification(mut self, enabled: bool) -> Self {
        self.request.disable_notification = enabled.then_some(true);
        self
    }

    /// Protects the sent message from forwarding and saving when `true`.
    pub fn protect_content(mut self, enabled: bool) -> Self {
        self.request.protect_content = enabled.then_some(true);
        self
    }

    /// Sets Telegram link preview behavior explicitly.
    pub fn link_preview_options(mut self, link_preview_options: LinkPreviewOptions) -> Self {
        self.request = self.request.link_preview_options(link_preview_options);
        self
    }

    /// Disables link previews for the text message.
    pub fn disable_link_preview(mut self) -> Self {
        self.request = self.request.disable_link_preview();
        self
    }

    /// Returns the typed request for lower-level reuse or inspection.
    pub fn into_request(self) -> SendMessageRequest {
        self.request
    }

    /// Sends the message.
    pub async fn send(self) -> Result<Message> {
        self.client.messages().send_message(&self.request).await
    }
}

/// Stable builder for high-level photo sends on the async app facade.
///
/// Start this from [`AppApi::photo`] or [`AppApi::reply_photo`].
#[cfg(feature = "_async")]
#[derive(Clone)]
#[must_use = "call `.send().await`, `.send_upload(...)`, or `.into_request()` to finish the send"]
pub struct PhotoSendBuilder {
    client: Client,
    request: SendPhotoRequest,
}

#[cfg(feature = "_async")]
impl PhotoSendBuilder {
    fn new(client: Client, request: SendPhotoRequest) -> Self {
        Self { client, request }
    }

    /// Marks the photo as spoiler media when `true`.
    pub fn has_spoiler(mut self, enabled: bool) -> Self {
        self.request.has_spoiler = enabled.then_some(true);
        self
    }

    /// Sends the photo using a Telegram file id / URL / attach reference already in the request.
    pub async fn send(self) -> Result<Message> {
        self.client.messages().send_photo(&self.request).await
    }

    /// Uploads local bytes as the photo payload.
    pub async fn send_upload(self, file: &UploadFile) -> Result<Message> {
        self.client
            .messages()
            .send_photo_upload(&self.request, file)
            .await
    }
}

#[cfg(feature = "_async")]
impl_common_media_builder_methods!(PhotoSendBuilder, SendPhotoRequest);

/// Stable builder for high-level document sends on the async app facade.
///
/// Start this from [`AppApi::document`] or [`AppApi::reply_document`].
#[cfg(feature = "_async")]
#[derive(Clone)]
#[must_use = "call `.send().await`, `.send_upload(...)`, or `.into_request()` to finish the send"]
pub struct DocumentSendBuilder {
    client: Client,
    request: SendDocumentRequest,
}

#[cfg(feature = "_async")]
impl DocumentSendBuilder {
    fn new(client: Client, request: SendDocumentRequest) -> Self {
        Self { client, request }
    }

    /// Sets a document thumbnail by file id / URL / attach reference.
    pub fn thumbnail(mut self, thumbnail: impl Into<String>) -> Self {
        self.request.thumbnail = Some(thumbnail.into());
        self
    }

    /// Disables Telegram content-type detection when `true`.
    pub fn disable_content_type_detection(mut self, enabled: bool) -> Self {
        self.request.disable_content_type_detection = enabled.then_some(true);
        self
    }

    /// Sends the document using a Telegram file id / URL / attach reference already in the request.
    pub async fn send(self) -> Result<Message> {
        self.client.messages().send_document(&self.request).await
    }

    /// Uploads local bytes as the document payload.
    pub async fn send_upload(self, file: &UploadFile) -> Result<Message> {
        self.client
            .messages()
            .send_document_upload(&self.request, file)
            .await
    }
}

#[cfg(feature = "_async")]
impl_common_media_builder_methods!(DocumentSendBuilder, SendDocumentRequest);

/// Stable builder for high-level video sends on the async app facade.
///
/// Start this from [`AppApi::video`] or [`AppApi::reply_video`].
#[cfg(feature = "_async")]
#[derive(Clone)]
#[must_use = "call `.send().await`, `.send_upload(...)`, or `.into_request()` to finish the send"]
pub struct VideoSendBuilder {
    client: Client,
    request: SendVideoRequest,
}

#[cfg(feature = "_async")]
impl VideoSendBuilder {
    fn new(client: Client, request: SendVideoRequest) -> Self {
        Self { client, request }
    }

    /// Sets video duration in seconds.
    pub fn duration(mut self, duration: u32) -> Self {
        self.request.duration = Some(duration);
        self
    }

    /// Sets video width in pixels.
    pub fn width(mut self, width: u32) -> Self {
        self.request.width = Some(width);
        self
    }

    /// Sets video height in pixels.
    pub fn height(mut self, height: u32) -> Self {
        self.request.height = Some(height);
        self
    }

    /// Sets a video thumbnail by file id / URL / attach reference.
    pub fn thumbnail(mut self, thumbnail: impl Into<String>) -> Self {
        self.request.thumbnail = Some(thumbnail.into());
        self
    }

    /// Marks the video as streamable when `true`.
    pub fn supports_streaming(mut self, enabled: bool) -> Self {
        self.request.supports_streaming = enabled.then_some(true);
        self
    }

    /// Marks the video as spoiler media when `true`.
    pub fn has_spoiler(mut self, enabled: bool) -> Self {
        self.request.has_spoiler = enabled.then_some(true);
        self
    }

    /// Sends the video using a Telegram file id / URL / attach reference already in the request.
    pub async fn send(self) -> Result<Message> {
        self.client.messages().send_video(&self.request).await
    }

    /// Uploads local bytes as the video payload.
    pub async fn send_upload(self, file: &UploadFile) -> Result<Message> {
        self.client
            .messages()
            .send_video_upload(&self.request, file)
            .await
    }
}

#[cfg(feature = "_async")]
impl_common_media_builder_methods!(VideoSendBuilder, SendVideoRequest);

/// Stable builder for high-level audio sends on the async app facade.
///
/// Start this from [`AppApi::audio`] or [`AppApi::reply_audio`].
#[cfg(feature = "_async")]
#[derive(Clone)]
#[must_use = "call `.send().await`, `.send_upload(...)`, or `.into_request()` to finish the send"]
pub struct AudioSendBuilder {
    client: Client,
    request: SendAudioRequest,
}

#[cfg(feature = "_async")]
impl AudioSendBuilder {
    fn new(client: Client, request: SendAudioRequest) -> Self {
        Self { client, request }
    }

    /// Sets audio duration in seconds.
    pub fn duration(mut self, duration: u32) -> Self {
        self.request.duration = Some(duration);
        self
    }

    /// Sets the displayed performer.
    pub fn performer(mut self, performer: impl Into<String>) -> Self {
        self.request.performer = Some(performer.into());
        self
    }

    /// Sets the displayed title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.request.title = Some(title.into());
        self
    }

    /// Sets an audio thumbnail by file id / URL / attach reference.
    pub fn thumbnail(mut self, thumbnail: impl Into<String>) -> Self {
        self.request.thumbnail = Some(thumbnail.into());
        self
    }

    /// Sends the audio using a Telegram file id / URL / attach reference already in the request.
    pub async fn send(self) -> Result<Message> {
        self.client.messages().send_audio(&self.request).await
    }

    /// Uploads local bytes as the audio payload.
    pub async fn send_upload(self, file: &UploadFile) -> Result<Message> {
        self.client
            .messages()
            .send_audio_upload(&self.request, file)
            .await
    }
}

#[cfg(feature = "_async")]
impl_common_media_builder_methods!(AudioSendBuilder, SendAudioRequest);

/// Stable builder for high-level animation sends on the async app facade.
///
/// Start this from [`AppApi::animation`] or [`AppApi::reply_animation`].
#[cfg(feature = "_async")]
#[derive(Clone)]
#[must_use = "call `.send().await`, `.send_upload(...)`, or `.into_request()` to finish the send"]
pub struct AnimationSendBuilder {
    client: Client,
    request: SendAnimationRequest,
}

#[cfg(feature = "_async")]
impl AnimationSendBuilder {
    fn new(client: Client, request: SendAnimationRequest) -> Self {
        Self { client, request }
    }

    /// Sets animation duration in seconds.
    pub fn duration(mut self, duration: u32) -> Self {
        self.request.duration = Some(duration);
        self
    }

    /// Sets animation width in pixels.
    pub fn width(mut self, width: u32) -> Self {
        self.request.width = Some(width);
        self
    }

    /// Sets animation height in pixels.
    pub fn height(mut self, height: u32) -> Self {
        self.request.height = Some(height);
        self
    }

    /// Sets an animation thumbnail by file id / URL / attach reference.
    pub fn thumbnail(mut self, thumbnail: impl Into<String>) -> Self {
        self.request.thumbnail = Some(thumbnail.into());
        self
    }

    /// Marks the animation as spoiler media when `true`.
    pub fn has_spoiler(mut self, enabled: bool) -> Self {
        self.request.has_spoiler = enabled.then_some(true);
        self
    }

    /// Sends the animation using a Telegram file id / URL / attach reference already in the request.
    pub async fn send(self) -> Result<Message> {
        self.client.messages().send_animation(&self.request).await
    }

    /// Uploads local bytes as the animation payload.
    pub async fn send_upload(self, file: &UploadFile) -> Result<Message> {
        self.client
            .messages()
            .send_animation_upload(&self.request, file)
            .await
    }
}

#[cfg(feature = "_async")]
impl_common_media_builder_methods!(AnimationSendBuilder, SendAnimationRequest);

/// Stable builder for high-level voice sends on the async app facade.
///
/// Start this from [`AppApi::voice`] or [`AppApi::reply_voice`].
#[cfg(feature = "_async")]
#[derive(Clone)]
#[must_use = "call `.send().await`, `.send_upload(...)`, or `.into_request()` to finish the send"]
pub struct VoiceSendBuilder {
    client: Client,
    request: SendVoiceRequest,
}

#[cfg(feature = "_async")]
impl VoiceSendBuilder {
    fn new(client: Client, request: SendVoiceRequest) -> Self {
        Self { client, request }
    }

    /// Sets voice duration in seconds.
    pub fn duration(mut self, duration: u32) -> Self {
        self.request.duration = Some(duration);
        self
    }

    /// Sends the voice message using a Telegram file id / URL / attach reference already in the request.
    pub async fn send(self) -> Result<Message> {
        self.client.messages().send_voice(&self.request).await
    }

    /// Uploads local bytes as the voice payload.
    pub async fn send_upload(self, file: &UploadFile) -> Result<Message> {
        self.client
            .messages()
            .send_voice_upload(&self.request, file)
            .await
    }
}

#[cfg(feature = "_async")]
impl_common_media_builder_methods!(VoiceSendBuilder, SendVoiceRequest);

/// Stable builder for high-level sticker sends on the async app facade.
///
/// Start this from [`AppApi::sticker`] or [`AppApi::reply_sticker`].
#[cfg(feature = "_async")]
#[derive(Clone)]
#[must_use = "call `.send().await`, `.send_upload(...)`, or `.into_request()` to finish the send"]
pub struct StickerSendBuilder {
    client: Client,
    request: SendStickerRequest,
}

#[cfg(feature = "_async")]
impl StickerSendBuilder {
    fn new(client: Client, request: SendStickerRequest) -> Self {
        Self { client, request }
    }

    /// Sends the sticker using a Telegram file id / URL / attach reference already in the request.
    pub async fn send(self) -> Result<Message> {
        self.client.stickers().send_sticker(&self.request).await
    }

    /// Uploads local bytes as the sticker payload.
    pub async fn send_upload(self, file: &UploadFile) -> Result<Message> {
        self.client
            .stickers()
            .send_sticker_upload(&self.request, file)
            .await
    }
}

#[cfg(feature = "_async")]
impl_common_sticker_builder_methods!(StickerSendBuilder, SendStickerRequest);

/// Stable builder for high-level media group sends on the async app facade.
///
/// Start this from [`AppApi::media_group`] or [`AppApi::reply_media_group`].
#[cfg(feature = "_async")]
#[derive(Clone)]
#[must_use = "call `.send().await` or `.into_request()` to finish the send"]
pub struct MediaGroupSendBuilder {
    client: Client,
    request: SendMediaGroupRequest,
}

#[cfg(feature = "_async")]
impl MediaGroupSendBuilder {
    fn new(client: Client, request: SendMediaGroupRequest) -> Self {
        Self { client, request }
    }

    /// Sends the media group.
    pub async fn send(self) -> Result<Vec<Message>> {
        self.client.messages().send_media_group(&self.request).await
    }
}

#[cfg(feature = "_async")]
impl_common_media_group_builder_methods!(MediaGroupSendBuilder, SendMediaGroupRequest);

/// Stable app-facing runtime facade for business code.
///
/// Prefer this facade inside handlers through `context.app()`, or directly through
/// `client.app()` in application code that is still part of the runtime plane.
///
/// Use this layer for:
///
/// - text and media sends via builder-style helpers
/// - callback answers
/// - moderation and governance notices
/// - membership / capability checks
/// - Web App runtime interactions
///
/// Use `client.control()` for startup, bootstrap, router preparation, and other orchestration
/// concerns.
#[cfg(feature = "_async")]
#[derive(Clone)]
pub struct AppApi {
    client: Client,
}

#[cfg(feature = "_async")]
impl AppApi {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Returns the governance-oriented moderation facade.
    pub fn moderation(&self) -> ModerationApi {
        ModerationApi::new(self.client.clone())
    }

    /// Returns the membership/capability facade used by install and bind pre-check flows.
    pub fn membership(&self) -> MembershipApi {
        MembershipApi::new(self.client.clone())
    }

    /// Returns the dedicated Web App runtime facade.
    pub fn web_app(&self) -> WebAppApi {
        WebAppApi::new(self.client.clone())
    }

    /// Starts a callback-answer builder.
    ///
    /// Prefer this when you need richer callback options such as `show_alert`, `url`, or
    /// `cache_time`. For simple text-only answers, `answer_callback(...)` remains the short path.
    pub fn callback_answer(&self, callback_query_id: impl Into<String>) -> CallbackAnswerBuilder {
        let request = callback_answer_request(callback_query_id, None);
        CallbackAnswerBuilder::new(self.client.clone(), request)
    }

    /// Starts a callback-answer builder using the callback id extracted from an update.
    pub fn callback_answer_from_update(&self, update: &Update) -> Result<CallbackAnswerBuilder> {
        let Some(callback_query_id) = callback_query_id(update) else {
            return Err(super::support::invalid_request(
                "update does not contain callback query for answerCallbackQuery",
            ));
        };
        Ok(self.callback_answer(callback_query_id))
    }

    /// Starts a text-send builder for a target chat.
    pub fn text(
        &self,
        chat_id: impl Into<ChatId>,
        text: impl Into<String>,
    ) -> Result<TextSendBuilder> {
        let request = text_send_request(chat_id, text)?;
        Ok(TextSendBuilder::new(self.client.clone(), request))
    }

    /// Starts a text-send builder using the canonical reply chat derived from an update.
    pub fn reply(&self, update: &Update, text: impl Into<String>) -> Result<TextSendBuilder> {
        let request = reply_text_request(update, text)?;
        Ok(TextSendBuilder::new(self.client.clone(), request))
    }

    /// Starts a photo-send builder for a target chat.
    pub fn photo(&self, chat_id: impl Into<ChatId>, photo: impl Into<String>) -> PhotoSendBuilder {
        let request = photo_send_request(chat_id, photo);
        PhotoSendBuilder::new(self.client.clone(), request)
    }

    /// Starts a photo-send builder using the canonical reply chat derived from an update.
    pub fn reply_photo(
        &self,
        update: &Update,
        photo: impl Into<String>,
    ) -> Result<PhotoSendBuilder> {
        let request = reply_photo_request(update, photo)?;
        Ok(PhotoSendBuilder::new(self.client.clone(), request))
    }

    /// Starts a document-send builder for a target chat.
    pub fn document(
        &self,
        chat_id: impl Into<ChatId>,
        document: impl Into<String>,
    ) -> DocumentSendBuilder {
        let request = document_send_request(chat_id, document);
        DocumentSendBuilder::new(self.client.clone(), request)
    }

    /// Starts a document-send builder using the canonical reply chat derived from an update.
    pub fn reply_document(
        &self,
        update: &Update,
        document: impl Into<String>,
    ) -> Result<DocumentSendBuilder> {
        let request = reply_document_request(update, document)?;
        Ok(DocumentSendBuilder::new(self.client.clone(), request))
    }

    /// Starts a video-send builder for a target chat.
    pub fn video(&self, chat_id: impl Into<ChatId>, video: impl Into<String>) -> VideoSendBuilder {
        let request = video_send_request(chat_id, video);
        VideoSendBuilder::new(self.client.clone(), request)
    }

    /// Starts a video-send builder using the canonical reply chat derived from an update.
    pub fn reply_video(
        &self,
        update: &Update,
        video: impl Into<String>,
    ) -> Result<VideoSendBuilder> {
        let request = reply_video_request(update, video)?;
        Ok(VideoSendBuilder::new(self.client.clone(), request))
    }

    /// Starts an audio-send builder for a target chat.
    pub fn audio(&self, chat_id: impl Into<ChatId>, audio: impl Into<String>) -> AudioSendBuilder {
        let request = audio_send_request(chat_id, audio);
        AudioSendBuilder::new(self.client.clone(), request)
    }

    /// Starts an audio-send builder using the canonical reply chat derived from an update.
    pub fn reply_audio(
        &self,
        update: &Update,
        audio: impl Into<String>,
    ) -> Result<AudioSendBuilder> {
        let request = reply_audio_request(update, audio)?;
        Ok(AudioSendBuilder::new(self.client.clone(), request))
    }

    /// Starts an animation-send builder for a target chat.
    pub fn animation(
        &self,
        chat_id: impl Into<ChatId>,
        animation: impl Into<String>,
    ) -> AnimationSendBuilder {
        let request = animation_send_request(chat_id, animation);
        AnimationSendBuilder::new(self.client.clone(), request)
    }

    /// Starts an animation-send builder using the canonical reply chat derived from an update.
    pub fn reply_animation(
        &self,
        update: &Update,
        animation: impl Into<String>,
    ) -> Result<AnimationSendBuilder> {
        let request = reply_animation_request(update, animation)?;
        Ok(AnimationSendBuilder::new(self.client.clone(), request))
    }

    /// Starts a voice-send builder for a target chat.
    pub fn voice(&self, chat_id: impl Into<ChatId>, voice: impl Into<String>) -> VoiceSendBuilder {
        let request = voice_send_request(chat_id, voice);
        VoiceSendBuilder::new(self.client.clone(), request)
    }

    /// Starts a voice-send builder using the canonical reply chat derived from an update.
    pub fn reply_voice(
        &self,
        update: &Update,
        voice: impl Into<String>,
    ) -> Result<VoiceSendBuilder> {
        let request = reply_voice_request(update, voice)?;
        Ok(VoiceSendBuilder::new(self.client.clone(), request))
    }

    /// Starts a sticker-send builder for a target chat.
    pub fn sticker(
        &self,
        chat_id: impl Into<ChatId>,
        sticker: impl Into<String>,
    ) -> StickerSendBuilder {
        let request = sticker_send_request(chat_id, sticker);
        StickerSendBuilder::new(self.client.clone(), request)
    }

    /// Starts a sticker-send builder using the canonical reply chat derived from an update.
    pub fn reply_sticker(
        &self,
        update: &Update,
        sticker: impl Into<String>,
    ) -> Result<StickerSendBuilder> {
        let request = reply_sticker_request(update, sticker)?;
        Ok(StickerSendBuilder::new(self.client.clone(), request))
    }

    /// Starts a media-group builder for a target chat.
    ///
    /// `media` must contain at least one item. Build items with the typed `InputMedia` models.
    pub fn media_group<I, M>(
        &self,
        chat_id: impl Into<ChatId>,
        media: I,
    ) -> Result<MediaGroupSendBuilder>
    where
        I: IntoIterator<Item = M>,
        M: Into<InputMedia>,
    {
        let request = media_group_send_request(chat_id, media)?;
        Ok(MediaGroupSendBuilder::new(self.client.clone(), request))
    }

    /// Starts a media-group builder using the canonical reply chat derived from an update.
    pub fn reply_media_group<I, M>(
        &self,
        update: &Update,
        media: I,
    ) -> Result<MediaGroupSendBuilder>
    where
        I: IntoIterator<Item = M>,
        M: Into<InputMedia>,
    {
        let request = reply_media_group_request(update, media)?;
        Ok(MediaGroupSendBuilder::new(self.client.clone(), request))
    }

    /// Shortcut for `text(...).send().await`.
    pub async fn send_text(
        &self,
        chat_id: impl Into<ChatId>,
        text: impl Into<String>,
    ) -> Result<Message> {
        self.text(chat_id, text)?.send().await
    }

    /// Shortcut for `reply(...).send().await`.
    pub async fn reply_text(&self, update: &Update, text: impl Into<String>) -> Result<Message> {
        self.reply(update, text)?.send().await
    }

    /// Shortcut for `callback_answer(...).text_optional(...).send().await`.
    pub async fn answer_callback(
        &self,
        callback_query_id: impl Into<String>,
        text: Option<String>,
    ) -> Result<bool> {
        self.callback_answer(callback_query_id)
            .text_optional(text)
            .send()
            .await
    }

    /// Shortcut for `callback_answer_from_update(...).text_optional(...).send().await`.
    pub async fn answer_callback_from_update(
        &self,
        update: &Update,
        text: Option<String>,
    ) -> Result<bool> {
        self.callback_answer_from_update(update)?
            .text_optional(text)
            .send()
            .await
    }
}

/// Stable builder for high-level callback answers on the blocking app facade.
///
/// Blocking mirror of [`CallbackAnswerBuilder`].
#[cfg(feature = "_blocking")]
#[derive(Clone)]
#[must_use = "call `.send()` or `.into_request()` to finish the callback answer"]
pub struct BlockingCallbackAnswerBuilder {
    client: BlockingClient,
    request: AnswerCallbackQueryRequest,
}

#[cfg(feature = "_blocking")]
impl BlockingCallbackAnswerBuilder {
    fn new(client: BlockingClient, request: AnswerCallbackQueryRequest) -> Self {
        Self { client, request }
    }

    /// Sends the callback answer request.
    pub fn send(self) -> Result<bool> {
        self.client.updates().answer_callback_query(&self.request)
    }
}

#[cfg(feature = "_blocking")]
impl_common_callback_answer_builder_methods!(
    BlockingCallbackAnswerBuilder,
    AnswerCallbackQueryRequest
);

/// Stable builder for high-level text sends on the blocking app facade.
///
/// Blocking mirror of [`TextSendBuilder`].
#[cfg(feature = "_blocking")]
#[derive(Clone)]
#[must_use = "call `.send()` or `.into_request()` to finish the message send"]
pub struct BlockingTextSendBuilder {
    client: BlockingClient,
    request: SendMessageRequest,
}

#[cfg(feature = "_blocking")]
impl BlockingTextSendBuilder {
    fn new(client: BlockingClient, request: SendMessageRequest) -> Self {
        Self { client, request }
    }

    /// Sets text parse mode.
    pub fn parse_mode(mut self, parse_mode: ParseMode) -> Self {
        self.request = self.request.parse_mode(parse_mode);
        self
    }

    /// Attaches reply markup such as an inline keyboard.
    pub fn reply_markup(mut self, reply_markup: impl Into<ReplyMarkup>) -> Self {
        self.request = self.request.reply_markup(reply_markup);
        self
    }

    /// Sets explicit reply parameters.
    pub fn reply_parameters(mut self, reply_parameters: ReplyParameters) -> Self {
        self.request = self.request.reply_parameters(reply_parameters);
        self
    }

    /// Replies to a concrete message by id.
    pub fn reply_to_message(mut self, message_id: MessageId) -> Self {
        self.request = self.request.reply_to_message(message_id);
        self
    }

    /// Targets a forum topic / message thread when applicable.
    pub fn message_thread_id(mut self, message_thread_id: i64) -> Self {
        self.request.message_thread_id = Some(message_thread_id);
        self
    }

    /// Sends silently when `true`.
    pub fn disable_notification(mut self, enabled: bool) -> Self {
        self.request.disable_notification = enabled.then_some(true);
        self
    }

    /// Protects the sent message from forwarding and saving when `true`.
    pub fn protect_content(mut self, enabled: bool) -> Self {
        self.request.protect_content = enabled.then_some(true);
        self
    }

    /// Sets Telegram link preview behavior explicitly.
    pub fn link_preview_options(mut self, link_preview_options: LinkPreviewOptions) -> Self {
        self.request = self.request.link_preview_options(link_preview_options);
        self
    }

    /// Disables link previews for the text message.
    pub fn disable_link_preview(mut self) -> Self {
        self.request = self.request.disable_link_preview();
        self
    }

    /// Returns the typed request for lower-level reuse or inspection.
    pub fn into_request(self) -> SendMessageRequest {
        self.request
    }

    /// Sends the message.
    pub fn send(self) -> Result<Message> {
        self.client.messages().send_message(&self.request)
    }
}

/// Stable builder for high-level photo sends on the blocking app facade.
///
/// Blocking mirror of [`PhotoSendBuilder`].
#[cfg(feature = "_blocking")]
#[derive(Clone)]
#[must_use = "call `.send()`, `.send_upload(...)`, or `.into_request()` to finish the send"]
pub struct BlockingPhotoSendBuilder {
    client: BlockingClient,
    request: SendPhotoRequest,
}

#[cfg(feature = "_blocking")]
impl BlockingPhotoSendBuilder {
    fn new(client: BlockingClient, request: SendPhotoRequest) -> Self {
        Self { client, request }
    }

    /// Marks the photo as spoiler media when `true`.
    pub fn has_spoiler(mut self, enabled: bool) -> Self {
        self.request.has_spoiler = enabled.then_some(true);
        self
    }

    /// Sends the photo using a Telegram file id / URL / attach reference already in the request.
    pub fn send(self) -> Result<Message> {
        self.client.messages().send_photo(&self.request)
    }

    /// Uploads local bytes as the photo payload.
    pub fn send_upload(self, file: &UploadFile) -> Result<Message> {
        self.client
            .messages()
            .send_photo_upload(&self.request, file)
    }
}

#[cfg(feature = "_blocking")]
impl_common_media_builder_methods!(BlockingPhotoSendBuilder, SendPhotoRequest);

/// Stable builder for high-level document sends on the blocking app facade.
///
/// Blocking mirror of [`DocumentSendBuilder`].
#[cfg(feature = "_blocking")]
#[derive(Clone)]
#[must_use = "call `.send()`, `.send_upload(...)`, or `.into_request()` to finish the send"]
pub struct BlockingDocumentSendBuilder {
    client: BlockingClient,
    request: SendDocumentRequest,
}

#[cfg(feature = "_blocking")]
impl BlockingDocumentSendBuilder {
    fn new(client: BlockingClient, request: SendDocumentRequest) -> Self {
        Self { client, request }
    }

    /// Sets a document thumbnail by file id / URL / attach reference.
    pub fn thumbnail(mut self, thumbnail: impl Into<String>) -> Self {
        self.request.thumbnail = Some(thumbnail.into());
        self
    }

    /// Disables Telegram content-type detection when `true`.
    pub fn disable_content_type_detection(mut self, enabled: bool) -> Self {
        self.request.disable_content_type_detection = enabled.then_some(true);
        self
    }

    /// Sends the document using a Telegram file id / URL / attach reference already in the request.
    pub fn send(self) -> Result<Message> {
        self.client.messages().send_document(&self.request)
    }

    /// Uploads local bytes as the document payload.
    pub fn send_upload(self, file: &UploadFile) -> Result<Message> {
        self.client
            .messages()
            .send_document_upload(&self.request, file)
    }
}

#[cfg(feature = "_blocking")]
impl_common_media_builder_methods!(BlockingDocumentSendBuilder, SendDocumentRequest);

/// Stable builder for high-level video sends on the blocking app facade.
///
/// Blocking mirror of [`VideoSendBuilder`].
#[cfg(feature = "_blocking")]
#[derive(Clone)]
#[must_use = "call `.send()`, `.send_upload(...)`, or `.into_request()` to finish the send"]
pub struct BlockingVideoSendBuilder {
    client: BlockingClient,
    request: SendVideoRequest,
}

#[cfg(feature = "_blocking")]
impl BlockingVideoSendBuilder {
    fn new(client: BlockingClient, request: SendVideoRequest) -> Self {
        Self { client, request }
    }

    /// Sets video duration in seconds.
    pub fn duration(mut self, duration: u32) -> Self {
        self.request.duration = Some(duration);
        self
    }

    /// Sets video width in pixels.
    pub fn width(mut self, width: u32) -> Self {
        self.request.width = Some(width);
        self
    }

    /// Sets video height in pixels.
    pub fn height(mut self, height: u32) -> Self {
        self.request.height = Some(height);
        self
    }

    /// Sets a video thumbnail by file id / URL / attach reference.
    pub fn thumbnail(mut self, thumbnail: impl Into<String>) -> Self {
        self.request.thumbnail = Some(thumbnail.into());
        self
    }

    /// Marks the video as streamable when `true`.
    pub fn supports_streaming(mut self, enabled: bool) -> Self {
        self.request.supports_streaming = enabled.then_some(true);
        self
    }

    /// Marks the video as spoiler media when `true`.
    pub fn has_spoiler(mut self, enabled: bool) -> Self {
        self.request.has_spoiler = enabled.then_some(true);
        self
    }

    /// Sends the video using a Telegram file id / URL / attach reference already in the request.
    pub fn send(self) -> Result<Message> {
        self.client.messages().send_video(&self.request)
    }

    /// Uploads local bytes as the video payload.
    pub fn send_upload(self, file: &UploadFile) -> Result<Message> {
        self.client
            .messages()
            .send_video_upload(&self.request, file)
    }
}

#[cfg(feature = "_blocking")]
impl_common_media_builder_methods!(BlockingVideoSendBuilder, SendVideoRequest);

/// Stable builder for high-level audio sends on the blocking app facade.
///
/// Blocking mirror of [`AudioSendBuilder`].
#[cfg(feature = "_blocking")]
#[derive(Clone)]
#[must_use = "call `.send()`, `.send_upload(...)`, or `.into_request()` to finish the send"]
pub struct BlockingAudioSendBuilder {
    client: BlockingClient,
    request: SendAudioRequest,
}

#[cfg(feature = "_blocking")]
impl BlockingAudioSendBuilder {
    fn new(client: BlockingClient, request: SendAudioRequest) -> Self {
        Self { client, request }
    }

    /// Sets audio duration in seconds.
    pub fn duration(mut self, duration: u32) -> Self {
        self.request.duration = Some(duration);
        self
    }

    /// Sets the displayed performer.
    pub fn performer(mut self, performer: impl Into<String>) -> Self {
        self.request.performer = Some(performer.into());
        self
    }

    /// Sets the displayed title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.request.title = Some(title.into());
        self
    }

    /// Sets an audio thumbnail by file id / URL / attach reference.
    pub fn thumbnail(mut self, thumbnail: impl Into<String>) -> Self {
        self.request.thumbnail = Some(thumbnail.into());
        self
    }

    /// Sends the audio using a Telegram file id / URL / attach reference already in the request.
    pub fn send(self) -> Result<Message> {
        self.client.messages().send_audio(&self.request)
    }

    /// Uploads local bytes as the audio payload.
    pub fn send_upload(self, file: &UploadFile) -> Result<Message> {
        self.client
            .messages()
            .send_audio_upload(&self.request, file)
    }
}

#[cfg(feature = "_blocking")]
impl_common_media_builder_methods!(BlockingAudioSendBuilder, SendAudioRequest);

/// Stable builder for high-level animation sends on the blocking app facade.
///
/// Blocking mirror of [`AnimationSendBuilder`].
#[cfg(feature = "_blocking")]
#[derive(Clone)]
#[must_use = "call `.send()`, `.send_upload(...)`, or `.into_request()` to finish the send"]
pub struct BlockingAnimationSendBuilder {
    client: BlockingClient,
    request: SendAnimationRequest,
}

#[cfg(feature = "_blocking")]
impl BlockingAnimationSendBuilder {
    fn new(client: BlockingClient, request: SendAnimationRequest) -> Self {
        Self { client, request }
    }

    /// Sets animation duration in seconds.
    pub fn duration(mut self, duration: u32) -> Self {
        self.request.duration = Some(duration);
        self
    }

    /// Sets animation width in pixels.
    pub fn width(mut self, width: u32) -> Self {
        self.request.width = Some(width);
        self
    }

    /// Sets animation height in pixels.
    pub fn height(mut self, height: u32) -> Self {
        self.request.height = Some(height);
        self
    }

    /// Sets an animation thumbnail by file id / URL / attach reference.
    pub fn thumbnail(mut self, thumbnail: impl Into<String>) -> Self {
        self.request.thumbnail = Some(thumbnail.into());
        self
    }

    /// Marks the animation as spoiler media when `true`.
    pub fn has_spoiler(mut self, enabled: bool) -> Self {
        self.request.has_spoiler = enabled.then_some(true);
        self
    }

    /// Sends the animation using a Telegram file id / URL / attach reference already in the request.
    pub fn send(self) -> Result<Message> {
        self.client.messages().send_animation(&self.request)
    }

    /// Uploads local bytes as the animation payload.
    pub fn send_upload(self, file: &UploadFile) -> Result<Message> {
        self.client
            .messages()
            .send_animation_upload(&self.request, file)
    }
}

#[cfg(feature = "_blocking")]
impl_common_media_builder_methods!(BlockingAnimationSendBuilder, SendAnimationRequest);

/// Stable builder for high-level voice sends on the blocking app facade.
///
/// Blocking mirror of [`VoiceSendBuilder`].
#[cfg(feature = "_blocking")]
#[derive(Clone)]
#[must_use = "call `.send()`, `.send_upload(...)`, or `.into_request()` to finish the send"]
pub struct BlockingVoiceSendBuilder {
    client: BlockingClient,
    request: SendVoiceRequest,
}

#[cfg(feature = "_blocking")]
impl BlockingVoiceSendBuilder {
    fn new(client: BlockingClient, request: SendVoiceRequest) -> Self {
        Self { client, request }
    }

    /// Sets voice duration in seconds.
    pub fn duration(mut self, duration: u32) -> Self {
        self.request.duration = Some(duration);
        self
    }

    /// Sends the voice message using a Telegram file id / URL / attach reference already in the request.
    pub fn send(self) -> Result<Message> {
        self.client.messages().send_voice(&self.request)
    }

    /// Uploads local bytes as the voice payload.
    pub fn send_upload(self, file: &UploadFile) -> Result<Message> {
        self.client
            .messages()
            .send_voice_upload(&self.request, file)
    }
}

#[cfg(feature = "_blocking")]
impl_common_media_builder_methods!(BlockingVoiceSendBuilder, SendVoiceRequest);

/// Stable builder for high-level sticker sends on the blocking app facade.
///
/// Blocking mirror of [`StickerSendBuilder`].
#[cfg(feature = "_blocking")]
#[derive(Clone)]
#[must_use = "call `.send()`, `.send_upload(...)`, or `.into_request()` to finish the send"]
pub struct BlockingStickerSendBuilder {
    client: BlockingClient,
    request: SendStickerRequest,
}

#[cfg(feature = "_blocking")]
impl BlockingStickerSendBuilder {
    fn new(client: BlockingClient, request: SendStickerRequest) -> Self {
        Self { client, request }
    }

    /// Sends the sticker using a Telegram file id / URL / attach reference already in the request.
    pub fn send(self) -> Result<Message> {
        self.client.stickers().send_sticker(&self.request)
    }

    /// Uploads local bytes as the sticker payload.
    pub fn send_upload(self, file: &UploadFile) -> Result<Message> {
        self.client
            .stickers()
            .send_sticker_upload(&self.request, file)
    }
}

#[cfg(feature = "_blocking")]
impl_common_sticker_builder_methods!(BlockingStickerSendBuilder, SendStickerRequest);

/// Stable builder for high-level media group sends on the blocking app facade.
///
/// Blocking mirror of [`MediaGroupSendBuilder`].
#[cfg(feature = "_blocking")]
#[derive(Clone)]
#[must_use = "call `.send()` or `.into_request()` to finish the send"]
pub struct BlockingMediaGroupSendBuilder {
    client: BlockingClient,
    request: SendMediaGroupRequest,
}

#[cfg(feature = "_blocking")]
impl BlockingMediaGroupSendBuilder {
    fn new(client: BlockingClient, request: SendMediaGroupRequest) -> Self {
        Self { client, request }
    }

    /// Sends the media group.
    pub fn send(self) -> Result<Vec<Message>> {
        self.client.messages().send_media_group(&self.request)
    }
}

#[cfg(feature = "_blocking")]
impl_common_media_group_builder_methods!(BlockingMediaGroupSendBuilder, SendMediaGroupRequest);

/// Stable app-facing runtime facade for blocking workflows.
///
/// Blocking mirror of [`AppApi`]. Prefer this layer for runtime/business code and keep
/// `client.control()` for startup and orchestration concerns.
#[cfg(feature = "_blocking")]
#[derive(Clone)]
pub struct BlockingAppApi {
    client: BlockingClient,
}

#[cfg(feature = "_blocking")]
impl BlockingAppApi {
    pub(crate) fn new(client: BlockingClient) -> Self {
        Self { client }
    }

    /// Returns the governance-oriented moderation facade.
    pub fn moderation(&self) -> BlockingModerationApi {
        BlockingModerationApi::new(self.client.clone())
    }

    /// Returns the membership/capability facade used by install and bind pre-check flows.
    pub fn membership(&self) -> BlockingMembershipApi {
        BlockingMembershipApi::new(self.client.clone())
    }

    /// Returns the dedicated Web App runtime facade.
    pub fn web_app(&self) -> BlockingWebAppApi {
        BlockingWebAppApi::new(self.client.clone())
    }

    /// Starts a callback-answer builder.
    ///
    /// Prefer this when callback replies need options such as `show_alert`, `url`, or
    /// `cache_time`.
    pub fn callback_answer(
        &self,
        callback_query_id: impl Into<String>,
    ) -> BlockingCallbackAnswerBuilder {
        let request = callback_answer_request(callback_query_id, None);
        BlockingCallbackAnswerBuilder::new(self.client.clone(), request)
    }

    /// Starts a callback-answer builder using the callback id extracted from an update.
    pub fn callback_answer_from_update(
        &self,
        update: &Update,
    ) -> Result<BlockingCallbackAnswerBuilder> {
        let Some(callback_query_id) = callback_query_id(update) else {
            return Err(super::support::invalid_request(
                "update does not contain callback query for answerCallbackQuery",
            ));
        };
        Ok(self.callback_answer(callback_query_id))
    }

    /// Starts a text-send builder for a target chat.
    pub fn text(
        &self,
        chat_id: impl Into<ChatId>,
        text: impl Into<String>,
    ) -> Result<BlockingTextSendBuilder> {
        let request = text_send_request(chat_id, text)?;
        Ok(BlockingTextSendBuilder::new(self.client.clone(), request))
    }

    /// Starts a text-send builder using the canonical reply chat derived from an update.
    pub fn reply(
        &self,
        update: &Update,
        text: impl Into<String>,
    ) -> Result<BlockingTextSendBuilder> {
        let request = reply_text_request(update, text)?;
        Ok(BlockingTextSendBuilder::new(self.client.clone(), request))
    }

    /// Starts a photo-send builder for a target chat.
    pub fn photo(
        &self,
        chat_id: impl Into<ChatId>,
        photo: impl Into<String>,
    ) -> BlockingPhotoSendBuilder {
        let request = photo_send_request(chat_id, photo);
        BlockingPhotoSendBuilder::new(self.client.clone(), request)
    }

    /// Starts a photo-send builder using the canonical reply chat derived from an update.
    pub fn reply_photo(
        &self,
        update: &Update,
        photo: impl Into<String>,
    ) -> Result<BlockingPhotoSendBuilder> {
        let request = reply_photo_request(update, photo)?;
        Ok(BlockingPhotoSendBuilder::new(self.client.clone(), request))
    }

    /// Starts a document-send builder for a target chat.
    pub fn document(
        &self,
        chat_id: impl Into<ChatId>,
        document: impl Into<String>,
    ) -> BlockingDocumentSendBuilder {
        let request = document_send_request(chat_id, document);
        BlockingDocumentSendBuilder::new(self.client.clone(), request)
    }

    /// Starts a document-send builder using the canonical reply chat derived from an update.
    pub fn reply_document(
        &self,
        update: &Update,
        document: impl Into<String>,
    ) -> Result<BlockingDocumentSendBuilder> {
        let request = reply_document_request(update, document)?;
        Ok(BlockingDocumentSendBuilder::new(
            self.client.clone(),
            request,
        ))
    }

    /// Starts a video-send builder for a target chat.
    pub fn video(
        &self,
        chat_id: impl Into<ChatId>,
        video: impl Into<String>,
    ) -> BlockingVideoSendBuilder {
        let request = video_send_request(chat_id, video);
        BlockingVideoSendBuilder::new(self.client.clone(), request)
    }

    /// Starts a video-send builder using the canonical reply chat derived from an update.
    pub fn reply_video(
        &self,
        update: &Update,
        video: impl Into<String>,
    ) -> Result<BlockingVideoSendBuilder> {
        let request = reply_video_request(update, video)?;
        Ok(BlockingVideoSendBuilder::new(self.client.clone(), request))
    }

    /// Starts an audio-send builder for a target chat.
    pub fn audio(
        &self,
        chat_id: impl Into<ChatId>,
        audio: impl Into<String>,
    ) -> BlockingAudioSendBuilder {
        let request = audio_send_request(chat_id, audio);
        BlockingAudioSendBuilder::new(self.client.clone(), request)
    }

    /// Starts an audio-send builder using the canonical reply chat derived from an update.
    pub fn reply_audio(
        &self,
        update: &Update,
        audio: impl Into<String>,
    ) -> Result<BlockingAudioSendBuilder> {
        let request = reply_audio_request(update, audio)?;
        Ok(BlockingAudioSendBuilder::new(self.client.clone(), request))
    }

    /// Starts an animation-send builder for a target chat.
    pub fn animation(
        &self,
        chat_id: impl Into<ChatId>,
        animation: impl Into<String>,
    ) -> BlockingAnimationSendBuilder {
        let request = animation_send_request(chat_id, animation);
        BlockingAnimationSendBuilder::new(self.client.clone(), request)
    }

    /// Starts an animation-send builder using the canonical reply chat derived from an update.
    pub fn reply_animation(
        &self,
        update: &Update,
        animation: impl Into<String>,
    ) -> Result<BlockingAnimationSendBuilder> {
        let request = reply_animation_request(update, animation)?;
        Ok(BlockingAnimationSendBuilder::new(
            self.client.clone(),
            request,
        ))
    }

    /// Starts a voice-send builder for a target chat.
    pub fn voice(
        &self,
        chat_id: impl Into<ChatId>,
        voice: impl Into<String>,
    ) -> BlockingVoiceSendBuilder {
        let request = voice_send_request(chat_id, voice);
        BlockingVoiceSendBuilder::new(self.client.clone(), request)
    }

    /// Starts a voice-send builder using the canonical reply chat derived from an update.
    pub fn reply_voice(
        &self,
        update: &Update,
        voice: impl Into<String>,
    ) -> Result<BlockingVoiceSendBuilder> {
        let request = reply_voice_request(update, voice)?;
        Ok(BlockingVoiceSendBuilder::new(self.client.clone(), request))
    }

    /// Starts a sticker-send builder for a target chat.
    pub fn sticker(
        &self,
        chat_id: impl Into<ChatId>,
        sticker: impl Into<String>,
    ) -> BlockingStickerSendBuilder {
        let request = sticker_send_request(chat_id, sticker);
        BlockingStickerSendBuilder::new(self.client.clone(), request)
    }

    /// Starts a sticker-send builder using the canonical reply chat derived from an update.
    pub fn reply_sticker(
        &self,
        update: &Update,
        sticker: impl Into<String>,
    ) -> Result<BlockingStickerSendBuilder> {
        let request = reply_sticker_request(update, sticker)?;
        Ok(BlockingStickerSendBuilder::new(
            self.client.clone(),
            request,
        ))
    }

    /// Starts a media-group builder for a target chat.
    ///
    /// `media` must contain at least one item. Build items with the typed `InputMedia` models.
    pub fn media_group<I, M>(
        &self,
        chat_id: impl Into<ChatId>,
        media: I,
    ) -> Result<BlockingMediaGroupSendBuilder>
    where
        I: IntoIterator<Item = M>,
        M: Into<InputMedia>,
    {
        let request = media_group_send_request(chat_id, media)?;
        Ok(BlockingMediaGroupSendBuilder::new(
            self.client.clone(),
            request,
        ))
    }

    /// Starts a media-group builder using the canonical reply chat derived from an update.
    pub fn reply_media_group<I, M>(
        &self,
        update: &Update,
        media: I,
    ) -> Result<BlockingMediaGroupSendBuilder>
    where
        I: IntoIterator<Item = M>,
        M: Into<InputMedia>,
    {
        let request = reply_media_group_request(update, media)?;
        Ok(BlockingMediaGroupSendBuilder::new(
            self.client.clone(),
            request,
        ))
    }

    /// Shortcut for `text(...).send()`.
    pub fn send_text(
        &self,
        chat_id: impl Into<ChatId>,
        text: impl Into<String>,
    ) -> Result<Message> {
        self.text(chat_id, text)?.send()
    }

    /// Shortcut for `reply(...).send()`.
    pub fn reply_text(&self, update: &Update, text: impl Into<String>) -> Result<Message> {
        self.reply(update, text)?.send()
    }

    /// Shortcut for `callback_answer(...).text_optional(...).send()`.
    pub fn answer_callback(
        &self,
        callback_query_id: impl Into<String>,
        text: Option<String>,
    ) -> Result<bool> {
        self.callback_answer(callback_query_id)
            .text_optional(text)
            .send()
    }

    /// Shortcut for `callback_answer_from_update(...).text_optional(...).send()`.
    pub fn answer_callback_from_update(
        &self,
        update: &Update,
        text: Option<String>,
    ) -> Result<bool> {
        self.callback_answer_from_update(update)?
            .text_optional(text)
            .send()
    }
}

use super::support::{callback_query_id, reply_chat_id};
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

macro_rules! impl_common_media_builder_methods {
    ($builder:ident, $request_ty:ty) => {
        impl $builder {
            pub fn caption(mut self, caption: impl Into<String>) -> Self {
                self.request.caption = Some(caption.into());
                self
            }

            pub fn parse_mode(mut self, parse_mode: ParseMode) -> Self {
                self.request.parse_mode = Some(parse_mode);
                self
            }

            pub fn reply_markup(mut self, reply_markup: impl Into<ReplyMarkup>) -> Self {
                self.request.reply_markup = Some(reply_markup.into());
                self
            }

            pub fn reply_parameters(mut self, reply_parameters: ReplyParameters) -> Self {
                self.request.reply_parameters = Some(reply_parameters);
                self
            }

            pub fn reply_to_message(mut self, message_id: MessageId) -> Self {
                self.request.reply_parameters = Some(ReplyParameters::new(message_id));
                self
            }

            pub fn message_thread_id(mut self, message_thread_id: i64) -> Self {
                self.request.message_thread_id = Some(message_thread_id);
                self
            }

            pub fn disable_notification(mut self, enabled: bool) -> Self {
                self.request.disable_notification = enabled.then_some(true);
                self
            }

            pub fn protect_content(mut self, enabled: bool) -> Self {
                self.request.protect_content = enabled.then_some(true);
                self
            }

            pub fn into_request(self) -> $request_ty {
                self.request
            }
        }
    };
}

/// Stable builder for high-level text sends on the async app facade.
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

    pub fn parse_mode(mut self, parse_mode: ParseMode) -> Self {
        self.request = self.request.parse_mode(parse_mode);
        self
    }

    pub fn reply_markup(mut self, reply_markup: impl Into<ReplyMarkup>) -> Self {
        self.request = self.request.reply_markup(reply_markup);
        self
    }

    pub fn reply_parameters(mut self, reply_parameters: ReplyParameters) -> Self {
        self.request = self.request.reply_parameters(reply_parameters);
        self
    }

    pub fn reply_to_message(mut self, message_id: MessageId) -> Self {
        self.request = self.request.reply_to_message(message_id);
        self
    }

    pub fn message_thread_id(mut self, message_thread_id: i64) -> Self {
        self.request.message_thread_id = Some(message_thread_id);
        self
    }

    pub fn disable_notification(mut self, enabled: bool) -> Self {
        self.request.disable_notification = enabled.then_some(true);
        self
    }

    pub fn protect_content(mut self, enabled: bool) -> Self {
        self.request.protect_content = enabled.then_some(true);
        self
    }

    pub fn link_preview_options(mut self, link_preview_options: LinkPreviewOptions) -> Self {
        self.request = self.request.link_preview_options(link_preview_options);
        self
    }

    pub fn disable_link_preview(mut self) -> Self {
        self.request = self.request.disable_link_preview();
        self
    }

    pub fn into_request(self) -> SendMessageRequest {
        self.request
    }

    pub async fn send(self) -> Result<Message> {
        self.client.messages().send_message(&self.request).await
    }
}

/// Stable builder for high-level photo sends on the async app facade.
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

    pub fn has_spoiler(mut self, enabled: bool) -> Self {
        self.request.has_spoiler = enabled.then_some(true);
        self
    }

    pub async fn send(self) -> Result<Message> {
        self.client.messages().send_photo(&self.request).await
    }

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

    pub fn thumbnail(mut self, thumbnail: impl Into<String>) -> Self {
        self.request.thumbnail = Some(thumbnail.into());
        self
    }

    pub fn disable_content_type_detection(mut self, enabled: bool) -> Self {
        self.request.disable_content_type_detection = enabled.then_some(true);
        self
    }

    pub async fn send(self) -> Result<Message> {
        self.client.messages().send_document(&self.request).await
    }

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

    pub fn duration(mut self, duration: u32) -> Self {
        self.request.duration = Some(duration);
        self
    }

    pub fn width(mut self, width: u32) -> Self {
        self.request.width = Some(width);
        self
    }

    pub fn height(mut self, height: u32) -> Self {
        self.request.height = Some(height);
        self
    }

    pub fn thumbnail(mut self, thumbnail: impl Into<String>) -> Self {
        self.request.thumbnail = Some(thumbnail.into());
        self
    }

    pub fn supports_streaming(mut self, enabled: bool) -> Self {
        self.request.supports_streaming = enabled.then_some(true);
        self
    }

    pub fn has_spoiler(mut self, enabled: bool) -> Self {
        self.request.has_spoiler = enabled.then_some(true);
        self
    }

    pub async fn send(self) -> Result<Message> {
        self.client.messages().send_video(&self.request).await
    }

    pub async fn send_upload(self, file: &UploadFile) -> Result<Message> {
        self.client
            .messages()
            .send_video_upload(&self.request, file)
            .await
    }
}

#[cfg(feature = "_async")]
impl_common_media_builder_methods!(VideoSendBuilder, SendVideoRequest);

/// Stable app-facing high-level facade for common bot workflows.
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

    /// Dedicated moderation/admin facade for governance actions.
    pub fn moderation(&self) -> ModerationApi {
        ModerationApi::new(self.client.clone())
    }

    /// Dedicated membership/capability facade for install/bind pre-check flows.
    pub fn membership(&self) -> MembershipApi {
        MembershipApi::new(self.client.clone())
    }

    /// Dedicated Web App runtime facade.
    pub fn web_app(&self) -> WebAppApi {
        WebAppApi::new(self.client.clone())
    }

    /// Starts a high-level text send to a target chat.
    pub fn text(
        &self,
        chat_id: impl Into<ChatId>,
        text: impl Into<String>,
    ) -> Result<TextSendBuilder> {
        let request = text_send_request(chat_id, text)?;
        Ok(TextSendBuilder::new(self.client.clone(), request))
    }

    /// Starts a high-level text send using the canonical reply chat derived from an update.
    pub fn reply(&self, update: &Update, text: impl Into<String>) -> Result<TextSendBuilder> {
        let request = reply_text_request(update, text)?;
        Ok(TextSendBuilder::new(self.client.clone(), request))
    }

    /// Starts a high-level photo send to a target chat.
    pub fn photo(&self, chat_id: impl Into<ChatId>, photo: impl Into<String>) -> PhotoSendBuilder {
        let request = photo_send_request(chat_id, photo);
        PhotoSendBuilder::new(self.client.clone(), request)
    }

    /// Starts a high-level photo send using the canonical reply chat derived from an update.
    pub fn reply_photo(
        &self,
        update: &Update,
        photo: impl Into<String>,
    ) -> Result<PhotoSendBuilder> {
        let request = reply_photo_request(update, photo)?;
        Ok(PhotoSendBuilder::new(self.client.clone(), request))
    }

    /// Starts a high-level document send to a target chat.
    pub fn document(
        &self,
        chat_id: impl Into<ChatId>,
        document: impl Into<String>,
    ) -> DocumentSendBuilder {
        let request = document_send_request(chat_id, document);
        DocumentSendBuilder::new(self.client.clone(), request)
    }

    /// Starts a high-level document send using the canonical reply chat derived from an update.
    pub fn reply_document(
        &self,
        update: &Update,
        document: impl Into<String>,
    ) -> Result<DocumentSendBuilder> {
        let request = reply_document_request(update, document)?;
        Ok(DocumentSendBuilder::new(self.client.clone(), request))
    }

    /// Starts a high-level video send to a target chat.
    pub fn video(&self, chat_id: impl Into<ChatId>, video: impl Into<String>) -> VideoSendBuilder {
        let request = video_send_request(chat_id, video);
        VideoSendBuilder::new(self.client.clone(), request)
    }

    /// Starts a high-level video send using the canonical reply chat derived from an update.
    pub fn reply_video(
        &self,
        update: &Update,
        video: impl Into<String>,
    ) -> Result<VideoSendBuilder> {
        let request = reply_video_request(update, video)?;
        Ok(VideoSendBuilder::new(self.client.clone(), request))
    }

    /// Sends plain text to a target chat.
    pub async fn send_text(
        &self,
        chat_id: impl Into<ChatId>,
        text: impl Into<String>,
    ) -> Result<Message> {
        self.text(chat_id, text)?.send().await
    }

    /// Replies to a chat derived from an incoming update.
    pub async fn reply_text(&self, update: &Update, text: impl Into<String>) -> Result<Message> {
        self.reply(update, text)?.send().await
    }

    /// Answers callback query with optional message text.
    pub async fn answer_callback(
        &self,
        callback_query_id: impl Into<String>,
        text: Option<String>,
    ) -> Result<bool> {
        let request = callback_answer_request(callback_query_id, text);
        self.client.updates().answer_callback_query(&request).await
    }

    /// Answers callback query from update payload.
    pub async fn answer_callback_from_update(
        &self,
        update: &Update,
        text: Option<String>,
    ) -> Result<bool> {
        let Some(callback_query_id) = callback_query_id(update) else {
            return Err(super::support::invalid_request(
                "update does not contain callback query for answerCallbackQuery",
            ));
        };
        self.answer_callback(callback_query_id, text).await
    }
}

/// Stable builder for high-level text sends on the blocking app facade.
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

    pub fn parse_mode(mut self, parse_mode: ParseMode) -> Self {
        self.request = self.request.parse_mode(parse_mode);
        self
    }

    pub fn reply_markup(mut self, reply_markup: impl Into<ReplyMarkup>) -> Self {
        self.request = self.request.reply_markup(reply_markup);
        self
    }

    pub fn reply_parameters(mut self, reply_parameters: ReplyParameters) -> Self {
        self.request = self.request.reply_parameters(reply_parameters);
        self
    }

    pub fn reply_to_message(mut self, message_id: MessageId) -> Self {
        self.request = self.request.reply_to_message(message_id);
        self
    }

    pub fn message_thread_id(mut self, message_thread_id: i64) -> Self {
        self.request.message_thread_id = Some(message_thread_id);
        self
    }

    pub fn disable_notification(mut self, enabled: bool) -> Self {
        self.request.disable_notification = enabled.then_some(true);
        self
    }

    pub fn protect_content(mut self, enabled: bool) -> Self {
        self.request.protect_content = enabled.then_some(true);
        self
    }

    pub fn link_preview_options(mut self, link_preview_options: LinkPreviewOptions) -> Self {
        self.request = self.request.link_preview_options(link_preview_options);
        self
    }

    pub fn disable_link_preview(mut self) -> Self {
        self.request = self.request.disable_link_preview();
        self
    }

    pub fn into_request(self) -> SendMessageRequest {
        self.request
    }

    pub fn send(self) -> Result<Message> {
        self.client.messages().send_message(&self.request)
    }
}

/// Stable builder for high-level photo sends on the blocking app facade.
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

    pub fn has_spoiler(mut self, enabled: bool) -> Self {
        self.request.has_spoiler = enabled.then_some(true);
        self
    }

    pub fn send(self) -> Result<Message> {
        self.client.messages().send_photo(&self.request)
    }

    pub fn send_upload(self, file: &UploadFile) -> Result<Message> {
        self.client
            .messages()
            .send_photo_upload(&self.request, file)
    }
}

#[cfg(feature = "_blocking")]
impl_common_media_builder_methods!(BlockingPhotoSendBuilder, SendPhotoRequest);

/// Stable builder for high-level document sends on the blocking app facade.
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

    pub fn thumbnail(mut self, thumbnail: impl Into<String>) -> Self {
        self.request.thumbnail = Some(thumbnail.into());
        self
    }

    pub fn disable_content_type_detection(mut self, enabled: bool) -> Self {
        self.request.disable_content_type_detection = enabled.then_some(true);
        self
    }

    pub fn send(self) -> Result<Message> {
        self.client.messages().send_document(&self.request)
    }

    pub fn send_upload(self, file: &UploadFile) -> Result<Message> {
        self.client
            .messages()
            .send_document_upload(&self.request, file)
    }
}

#[cfg(feature = "_blocking")]
impl_common_media_builder_methods!(BlockingDocumentSendBuilder, SendDocumentRequest);

/// Stable builder for high-level video sends on the blocking app facade.
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

    pub fn duration(mut self, duration: u32) -> Self {
        self.request.duration = Some(duration);
        self
    }

    pub fn width(mut self, width: u32) -> Self {
        self.request.width = Some(width);
        self
    }

    pub fn height(mut self, height: u32) -> Self {
        self.request.height = Some(height);
        self
    }

    pub fn thumbnail(mut self, thumbnail: impl Into<String>) -> Self {
        self.request.thumbnail = Some(thumbnail.into());
        self
    }

    pub fn supports_streaming(mut self, enabled: bool) -> Self {
        self.request.supports_streaming = enabled.then_some(true);
        self
    }

    pub fn has_spoiler(mut self, enabled: bool) -> Self {
        self.request.has_spoiler = enabled.then_some(true);
        self
    }

    pub fn send(self) -> Result<Message> {
        self.client.messages().send_video(&self.request)
    }

    pub fn send_upload(self, file: &UploadFile) -> Result<Message> {
        self.client
            .messages()
            .send_video_upload(&self.request, file)
    }
}

#[cfg(feature = "_blocking")]
impl_common_media_builder_methods!(BlockingVideoSendBuilder, SendVideoRequest);

/// Stable app-facing high-level facade for blocking workflows.
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

    pub fn moderation(&self) -> BlockingModerationApi {
        BlockingModerationApi::new(self.client.clone())
    }

    pub fn membership(&self) -> BlockingMembershipApi {
        BlockingMembershipApi::new(self.client.clone())
    }

    pub fn web_app(&self) -> BlockingWebAppApi {
        BlockingWebAppApi::new(self.client.clone())
    }

    pub fn text(
        &self,
        chat_id: impl Into<ChatId>,
        text: impl Into<String>,
    ) -> Result<BlockingTextSendBuilder> {
        let request = text_send_request(chat_id, text)?;
        Ok(BlockingTextSendBuilder::new(self.client.clone(), request))
    }

    pub fn reply(
        &self,
        update: &Update,
        text: impl Into<String>,
    ) -> Result<BlockingTextSendBuilder> {
        let request = reply_text_request(update, text)?;
        Ok(BlockingTextSendBuilder::new(self.client.clone(), request))
    }

    pub fn photo(
        &self,
        chat_id: impl Into<ChatId>,
        photo: impl Into<String>,
    ) -> BlockingPhotoSendBuilder {
        let request = photo_send_request(chat_id, photo);
        BlockingPhotoSendBuilder::new(self.client.clone(), request)
    }

    pub fn reply_photo(
        &self,
        update: &Update,
        photo: impl Into<String>,
    ) -> Result<BlockingPhotoSendBuilder> {
        let request = reply_photo_request(update, photo)?;
        Ok(BlockingPhotoSendBuilder::new(self.client.clone(), request))
    }

    pub fn document(
        &self,
        chat_id: impl Into<ChatId>,
        document: impl Into<String>,
    ) -> BlockingDocumentSendBuilder {
        let request = document_send_request(chat_id, document);
        BlockingDocumentSendBuilder::new(self.client.clone(), request)
    }

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

    pub fn video(
        &self,
        chat_id: impl Into<ChatId>,
        video: impl Into<String>,
    ) -> BlockingVideoSendBuilder {
        let request = video_send_request(chat_id, video);
        BlockingVideoSendBuilder::new(self.client.clone(), request)
    }

    pub fn reply_video(
        &self,
        update: &Update,
        video: impl Into<String>,
    ) -> Result<BlockingVideoSendBuilder> {
        let request = reply_video_request(update, video)?;
        Ok(BlockingVideoSendBuilder::new(self.client.clone(), request))
    }

    pub fn send_text(
        &self,
        chat_id: impl Into<ChatId>,
        text: impl Into<String>,
    ) -> Result<Message> {
        self.text(chat_id, text)?.send()
    }

    pub fn reply_text(&self, update: &Update, text: impl Into<String>) -> Result<Message> {
        self.reply(update, text)?.send()
    }

    pub fn answer_callback(
        &self,
        callback_query_id: impl Into<String>,
        text: Option<String>,
    ) -> Result<bool> {
        let request = callback_answer_request(callback_query_id, text);
        self.client.updates().answer_callback_query(&request)
    }

    pub fn answer_callback_from_update(
        &self,
        update: &Update,
        text: Option<String>,
    ) -> Result<bool> {
        let Some(callback_query_id) = callback_query_id(update) else {
            return Err(super::support::invalid_request(
                "update does not contain callback query for answerCallbackQuery",
            ));
        };
        self.answer_callback(callback_query_id, text)
    }
}

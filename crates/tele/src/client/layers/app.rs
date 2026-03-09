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

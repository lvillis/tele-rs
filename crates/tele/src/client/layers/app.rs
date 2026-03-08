use super::support::{callback_query_id, reply_chat_id};
use super::*;

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

    /// App setup helpers for commands, menu buttons and bootstrap.
    pub fn setup(&self) -> SetupApi {
        SetupApi::new(self.client.clone())
    }

    /// Dedicated Web App facade.
    pub fn web_app(&self) -> WebAppApi {
        WebAppApi::new(self.client.clone())
    }

    /// Sends plain text to a target chat.
    pub async fn send_text(
        &self,
        chat_id: impl Into<ChatId>,
        text: impl Into<String>,
    ) -> Result<Message> {
        let request = SendMessageRequest::new(chat_id, text)?;
        self.client.messages().send_message(&request).await
    }

    /// Replies to a chat derived from an incoming update.
    pub async fn reply_text(&self, update: &Update, text: impl Into<String>) -> Result<Message> {
        let chat_id = reply_chat_id(update)?;
        self.send_text(chat_id, text).await
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

    pub fn setup(&self) -> BlockingSetupApi {
        BlockingSetupApi::new(self.client.clone())
    }

    pub fn web_app(&self) -> BlockingWebAppApi {
        BlockingWebAppApi::new(self.client.clone())
    }

    pub fn send_text(
        &self,
        chat_id: impl Into<ChatId>,
        text: impl Into<String>,
    ) -> Result<Message> {
        let request = SendMessageRequest::new(chat_id, text)?;
        self.client.messages().send_message(&request)
    }

    pub fn reply_text(&self, update: &Update, text: impl Into<String>) -> Result<Message> {
        let chat_id = reply_chat_id(update)?;
        self.send_text(chat_id, text)
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

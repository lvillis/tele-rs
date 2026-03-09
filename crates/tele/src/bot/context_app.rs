use super::*;

/// Request-scoped high-level facade for handler code.
#[derive(Clone)]
pub struct ContextAppApi {
    client: Client,
}

impl ContextAppApi {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Dedicated moderation/admin facade for handler-side governance actions.
    pub fn moderation(&self) -> crate::client::ModerationApi {
        self.client.app().moderation()
    }

    /// Dedicated Web App facade for runtime query handling.
    pub fn web_app(&self) -> crate::client::WebAppApi {
        self.client.app().web_app()
    }

    /// Starts a high-level text send to a target chat.
    pub fn text(
        &self,
        chat_id: impl Into<ChatId>,
        text: impl Into<String>,
    ) -> Result<crate::client::TextSendBuilder> {
        self.client.app().text(chat_id, text)
    }

    /// Starts a high-level text send using the canonical reply chat derived from an update.
    pub fn reply(
        &self,
        update: &Update,
        text: impl Into<String>,
    ) -> Result<crate::client::TextSendBuilder> {
        self.client.app().reply(update, text)
    }

    /// Sends plain text to a target chat.
    pub async fn send_text(
        &self,
        chat_id: impl Into<ChatId>,
        text: impl Into<String>,
    ) -> Result<Message> {
        self.client.app().send_text(chat_id, text).await
    }

    /// Replies with plain text using the canonical chat id extracted from update.
    pub async fn reply_text(&self, update: &Update, text: impl Into<String>) -> Result<Message> {
        self.client.app().reply_text(update, text).await
    }

    /// Answers callback query by id.
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

    /// Answers callback query using id extracted from update.
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

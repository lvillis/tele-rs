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

    /// Dedicated membership/capability facade for install/bind pre-check flows.
    pub fn membership(&self) -> crate::client::MembershipApi {
        self.client.app().membership()
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

    /// Starts a high-level photo send to a target chat.
    pub fn photo(
        &self,
        chat_id: impl Into<ChatId>,
        photo: impl Into<String>,
    ) -> crate::client::PhotoSendBuilder {
        self.client.app().photo(chat_id, photo)
    }

    /// Starts a high-level photo send using the canonical reply chat derived from an update.
    pub fn reply_photo(
        &self,
        update: &Update,
        photo: impl Into<String>,
    ) -> Result<crate::client::PhotoSendBuilder> {
        self.client.app().reply_photo(update, photo)
    }

    /// Starts a high-level document send to a target chat.
    pub fn document(
        &self,
        chat_id: impl Into<ChatId>,
        document: impl Into<String>,
    ) -> crate::client::DocumentSendBuilder {
        self.client.app().document(chat_id, document)
    }

    /// Starts a high-level document send using the canonical reply chat derived from an update.
    pub fn reply_document(
        &self,
        update: &Update,
        document: impl Into<String>,
    ) -> Result<crate::client::DocumentSendBuilder> {
        self.client.app().reply_document(update, document)
    }

    /// Starts a high-level video send to a target chat.
    pub fn video(
        &self,
        chat_id: impl Into<ChatId>,
        video: impl Into<String>,
    ) -> crate::client::VideoSendBuilder {
        self.client.app().video(chat_id, video)
    }

    /// Starts a high-level video send using the canonical reply chat derived from an update.
    pub fn reply_video(
        &self,
        update: &Update,
        video: impl Into<String>,
    ) -> Result<crate::client::VideoSendBuilder> {
        self.client.app().reply_video(update, video)
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

#[cfg(feature = "_async")]
use super::*;

/// Ergonomic high-level helpers for common async bot workflows.
#[cfg(feature = "_async")]
#[derive(Clone)]
pub struct ErgoApi {
    client: Client,
}

#[cfg(feature = "_async")]
impl ErgoApi {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
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
            return Err(super::super::support::invalid_request(
                "update does not contain callback query for answerCallbackQuery",
            ));
        };
        self.answer_callback(callback_query_id, text).await
    }
}

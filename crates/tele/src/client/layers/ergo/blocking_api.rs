#[cfg(feature = "_blocking")]
use super::*;

#[cfg(feature = "_blocking")]
#[derive(Clone)]
pub struct BlockingErgoApi {
    client: BlockingClient,
}

#[cfg(feature = "_blocking")]
impl BlockingErgoApi {
    pub(crate) fn new(client: BlockingClient) -> Self {
        Self { client }
    }

    /// Sends plain text to a target chat.
    pub fn send_text(
        &self,
        chat_id: impl Into<ChatId>,
        text: impl Into<String>,
    ) -> Result<Message> {
        let request = SendMessageRequest::new(chat_id, text)?;
        self.client.messages().send_message(&request)
    }

    /// Replies to a chat derived from an incoming update.
    pub fn reply_text(&self, update: &Update, text: impl Into<String>) -> Result<Message> {
        let chat_id = reply_chat_id(update)?;
        self.send_text(chat_id, text)
    }

    /// Answers callback query with optional message text.
    pub fn answer_callback(
        &self,
        callback_query_id: impl Into<String>,
        text: Option<String>,
    ) -> Result<bool> {
        let request = callback_answer_request(callback_query_id, text);
        self.client.updates().answer_callback_query(&request)
    }

    /// Answers callback query from update payload.
    pub fn answer_callback_from_update(
        &self,
        update: &Update,
        text: Option<String>,
    ) -> Result<bool> {
        let Some(callback_query_id) = callback_query_id(update) else {
            return Err(super::super::support::invalid_request(
                "update does not contain callback query for answerCallbackQuery",
            ));
        };
        self.answer_callback(callback_query_id, text)
    }
}

use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::types::advanced::AdvancedRequest;
use crate::types::command::{BotCommand, SetMyCommandsRequest};
use crate::types::common::ChatId;
use crate::types::message::{Message, SendMessageRequest};
use crate::types::update::{AnswerCallbackQueryRequest, Update};
use crate::types::upload::UploadFile;
use crate::{Error, Result};

#[cfg(feature = "_blocking")]
use crate::BlockingClient;
#[cfg(feature = "_async")]
use crate::Client;

fn invalid_request(reason: impl Into<String>) -> Error {
    Error::InvalidRequest {
        reason: reason.into(),
    }
}

fn update_chat_id(update: &Update) -> Option<i64> {
    if let Some(message) = update.message.as_ref() {
        return Some(message.chat.id);
    }
    if let Some(message) = update.edited_message.as_ref() {
        return Some(message.chat.id);
    }
    if let Some(message) = update.channel_post.as_ref() {
        return Some(message.chat.id);
    }
    if let Some(message) = update.edited_channel_post.as_ref() {
        return Some(message.chat.id);
    }

    update
        .callback_query
        .as_ref()
        .and_then(|query| query.message.as_ref())
        .map(|message| message.chat.id)
}

fn callback_query_id(update: &Update) -> Option<String> {
    update.callback_query.as_ref().map(|query| query.id.clone())
}

/// Raw Telegram API calling layer for async clients.
#[cfg(feature = "_async")]
#[derive(Clone)]
pub struct RawApi {
    client: Client,
}

#[cfg(feature = "_async")]
impl RawApi {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Calls any Telegram method with JSON payload.
    pub async fn call_json<R, P>(&self, method: &str, payload: &P) -> Result<R>
    where
        R: DeserializeOwned,
        P: Serialize + ?Sized,
    {
        self.client.call_method(method, payload).await
    }

    /// Calls any Telegram method without payload.
    pub async fn call_no_params<R>(&self, method: &str) -> Result<R>
    where
        R: DeserializeOwned,
    {
        self.client.call_method_no_params(method).await
    }

    /// Calls any Telegram method with a multipart file part.
    pub async fn call_multipart<R, P>(
        &self,
        method: &str,
        payload: &P,
        file_field_name: &str,
        file: &UploadFile,
    ) -> Result<R>
    where
        R: DeserializeOwned,
        P: Serialize + ?Sized,
    {
        self.client
            .call_method_multipart(method, payload, file_field_name, file)
            .await
    }
}

/// Typed Telegram API layer for async clients.
#[cfg(feature = "_async")]
#[derive(Clone)]
pub struct TypedApi {
    client: Client,
}

#[cfg(feature = "_async")]
impl TypedApi {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Calls a typed request that carries method name and response type.
    pub async fn call<Q>(&self, request: &Q) -> Result<Q::Response>
    where
        Q: AdvancedRequest,
    {
        self.client.call_method(Q::METHOD, request).await
    }
}

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
        let Some(chat_id) = update_chat_id(update) else {
            return Err(invalid_request(
                "update does not contain a chat id for reply",
            ));
        };
        self.send_text(chat_id, text).await
    }

    /// Answers callback query with optional message text.
    pub async fn answer_callback(
        &self,
        callback_query_id: impl Into<String>,
        text: Option<String>,
    ) -> Result<bool> {
        let request = AnswerCallbackQueryRequest {
            callback_query_id: callback_query_id.into(),
            text,
            show_alert: None,
            url: None,
            cache_time: None,
        };
        self.client.updates().answer_callback_query(&request).await
    }

    /// Answers callback query from update payload.
    pub async fn answer_callback_from_update(
        &self,
        update: &Update,
        text: Option<String>,
    ) -> Result<bool> {
        let Some(callback_query_id) = callback_query_id(update) else {
            return Err(invalid_request(
                "update does not contain callback query for answerCallbackQuery",
            ));
        };
        self.answer_callback(callback_query_id, text).await
    }

    /// Registers explicit command definitions.
    pub async fn set_commands(&self, commands: Vec<BotCommand>) -> Result<bool> {
        let request = SetMyCommandsRequest::new(commands)?;
        self.client.bot().set_my_commands(&request).await
    }

    /// Registers command definitions from a typed command enum.
    #[cfg(feature = "bot")]
    pub async fn set_typed_commands<C>(&self) -> Result<bool>
    where
        C: crate::bot::BotCommands,
    {
        self.set_commands(crate::bot::command_definitions::<C>())
            .await
    }
}

/// Raw Telegram API calling layer for blocking clients.
#[cfg(feature = "_blocking")]
#[derive(Clone)]
pub struct BlockingRawApi {
    client: BlockingClient,
}

#[cfg(feature = "_blocking")]
impl BlockingRawApi {
    pub(crate) fn new(client: BlockingClient) -> Self {
        Self { client }
    }

    /// Calls any Telegram method with JSON payload.
    pub fn call_json<R, P>(&self, method: &str, payload: &P) -> Result<R>
    where
        R: DeserializeOwned,
        P: Serialize + ?Sized,
    {
        self.client.call_method(method, payload)
    }

    /// Calls any Telegram method without payload.
    pub fn call_no_params<R>(&self, method: &str) -> Result<R>
    where
        R: DeserializeOwned,
    {
        self.client.call_method_no_params(method)
    }

    /// Calls any Telegram method with a multipart file part.
    pub fn call_multipart<R, P>(
        &self,
        method: &str,
        payload: &P,
        file_field_name: &str,
        file: &UploadFile,
    ) -> Result<R>
    where
        R: DeserializeOwned,
        P: Serialize + ?Sized,
    {
        self.client
            .call_method_multipart(method, payload, file_field_name, file)
    }
}

/// Typed Telegram API layer for blocking clients.
#[cfg(feature = "_blocking")]
#[derive(Clone)]
pub struct BlockingTypedApi {
    client: BlockingClient,
}

#[cfg(feature = "_blocking")]
impl BlockingTypedApi {
    pub(crate) fn new(client: BlockingClient) -> Self {
        Self { client }
    }

    /// Calls a typed request that carries method name and response type.
    pub fn call<Q>(&self, request: &Q) -> Result<Q::Response>
    where
        Q: AdvancedRequest,
    {
        self.client.call_method(Q::METHOD, request)
    }
}

/// Ergonomic high-level helpers for common blocking bot workflows.
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
        let Some(chat_id) = update_chat_id(update) else {
            return Err(invalid_request(
                "update does not contain a chat id for reply",
            ));
        };
        self.send_text(chat_id, text)
    }

    /// Answers callback query with optional message text.
    pub fn answer_callback(
        &self,
        callback_query_id: impl Into<String>,
        text: Option<String>,
    ) -> Result<bool> {
        let request = AnswerCallbackQueryRequest {
            callback_query_id: callback_query_id.into(),
            text,
            show_alert: None,
            url: None,
            cache_time: None,
        };
        self.client.updates().answer_callback_query(&request)
    }

    /// Answers callback query from update payload.
    pub fn answer_callback_from_update(
        &self,
        update: &Update,
        text: Option<String>,
    ) -> Result<bool> {
        let Some(callback_query_id) = callback_query_id(update) else {
            return Err(invalid_request(
                "update does not contain callback query for answerCallbackQuery",
            ));
        };
        self.answer_callback(callback_query_id, text)
    }

    /// Registers explicit command definitions.
    pub fn set_commands(&self, commands: Vec<BotCommand>) -> Result<bool> {
        let request = SetMyCommandsRequest::new(commands)?;
        self.client.bot().set_my_commands(&request)
    }

    /// Registers command definitions from a typed command enum.
    #[cfg(feature = "bot")]
    pub fn set_typed_commands<C>(&self) -> Result<bool>
    where
        C: crate::bot::BotCommands,
    {
        self.set_commands(crate::bot::command_definitions::<C>())
    }
}

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::client::RetryConfig;
use crate::types::advanced::{
    AdvancedAnswerWebAppQueryRequest, AdvancedRequest, AdvancedSetChatMenuButtonRequest,
};
use crate::types::bot::User;
use crate::types::command::{BotCommand, BotCommandScope, SetMyCommandsRequest};
use crate::types::common::ChatId;
use crate::types::message::{Message, SendMessageRequest, SentWebAppMessage};
use crate::types::telegram::{InlineQueryResult, WebAppData};
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

/// Retry policy for startup/bootstrap helper methods.
#[derive(Clone, Copy, Debug)]
pub struct BootstrapRetryPolicy {
    pub max_attempts: usize,
    pub base_backoff: Duration,
    pub max_backoff: Duration,
    pub jitter_ratio: f32,
    /// When true, exhausting retries returns `Ok(false)` instead of `Err(...)`.
    pub continue_on_failure: bool,
}

impl Default for BootstrapRetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_backoff: Duration::from_millis(200),
            max_backoff: Duration::from_secs(3),
            jitter_ratio: 0.1,
            continue_on_failure: false,
        }
    }
}

/// Startup bootstrap plan for common bot initialization actions.
#[derive(Clone, Debug, Default)]
pub struct BootstrapPlan {
    pub verify_get_me: bool,
    pub commands: Option<SetMyCommandsRequest>,
    pub menu_button: Option<AdvancedSetChatMenuButtonRequest>,
}

/// Result summary of a bootstrap run.
#[derive(Clone, Debug, Default)]
pub struct BootstrapReport {
    pub me: Option<User>,
    pub commands_applied: Option<bool>,
    pub menu_button_applied: Option<bool>,
}

fn normalize_language_code(language_code: Option<String>) -> Result<Option<String>> {
    let Some(language_code) = language_code else {
        return Ok(None);
    };
    if language_code.trim().is_empty() {
        return Err(invalid_request("language_code cannot be empty"));
    }
    Ok(Some(language_code))
}

#[cfg(feature = "bot")]
fn typed_commands_request<C>(
    scope: Option<BotCommandScope>,
    language_code: Option<String>,
) -> Result<SetMyCommandsRequest>
where
    C: crate::bot::BotCommands,
{
    let mut request = SetMyCommandsRequest::new(crate::bot::command_definitions::<C>())?;
    request.scope = scope;
    request.language_code = normalize_language_code(language_code)?;
    Ok(request)
}

fn backoff_delay(base: Duration, max: Duration, attempt: usize, jitter_ratio: f32) -> Duration {
    let exponent = attempt.saturating_sub(1).min(16);
    let factor = 2u32.saturating_pow(exponent as u32);
    let delay = base.saturating_mul(factor).min(max);
    if delay.is_zero() || jitter_ratio <= 0.0 {
        return delay;
    }

    let ratio = f64::from(jitter_ratio.clamp(0.0, 1.0));
    let now_nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0_u128, |value| value.as_nanos());
    let unit = (now_nanos % 10_000) as f64 / 10_000.0;
    let multiplier = (1.0 - ratio) + (2.0 * ratio * unit);
    let jittered = Duration::from_secs_f64(delay.as_secs_f64() * multiplier);
    jittered.min(max)
}

#[cfg(feature = "_async")]
async fn retry_with_config_async<T, F, Fut>(retry: &RetryConfig, mut op: F) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let max_attempts = retry.max_attempts.max(1);
    let mut attempt = 0;

    loop {
        attempt += 1;
        match op().await {
            Ok(value) => return Ok(value),
            Err(error) => {
                let should_retry = error.is_retryable() && attempt < max_attempts;
                if !should_retry {
                    return Err(error);
                }
                let delay = error.retry_after().unwrap_or_else(|| {
                    backoff_delay(
                        retry.base_backoff,
                        retry.max_backoff,
                        attempt,
                        retry.jitter_ratio as f32,
                    )
                });
                tokio::time::sleep(delay.min(retry.max_backoff)).await;
            }
        }
    }
}

#[cfg(feature = "_blocking")]
fn retry_with_config_blocking<T, F>(retry: &RetryConfig, mut op: F) -> Result<T>
where
    F: FnMut() -> Result<T>,
{
    let max_attempts = retry.max_attempts.max(1);
    let mut attempt = 0;

    loop {
        attempt += 1;
        match op() {
            Ok(value) => return Ok(value),
            Err(error) => {
                let should_retry = error.is_retryable() && attempt < max_attempts;
                if !should_retry {
                    return Err(error);
                }
                let delay = error.retry_after().unwrap_or_else(|| {
                    backoff_delay(
                        retry.base_backoff,
                        retry.max_backoff,
                        attempt,
                        retry.jitter_ratio as f32,
                    )
                });
                std::thread::sleep(delay.min(retry.max_backoff));
            }
        }
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

/// Parsed WebApp payload containing `query_id` and typed data.
#[derive(Clone, Debug)]
pub struct WebAppQueryPayload<T> {
    pub query_id: String,
    pub payload: T,
}

impl<T> WebAppQueryPayload<T>
where
    T: DeserializeOwned,
{
    pub fn parse(web_app_data: &WebAppData) -> Result<Self> {
        parse_web_app_query_payload(web_app_data)
    }
}

fn parse_web_app_query_payload<T>(web_app_data: &WebAppData) -> Result<WebAppQueryPayload<T>>
where
    T: DeserializeOwned,
{
    let mut value: serde_json::Value =
        serde_json::from_str(&web_app_data.data).map_err(|source| Error::InvalidRequest {
            reason: format!("invalid web_app_data JSON payload: {source}"),
        })?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| invalid_request("web_app_data payload must be a JSON object"))?;

    let query_id = object
        .remove("query_id")
        .and_then(|value| value.as_str().map(str::to_owned))
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| invalid_request("web_app_data payload is missing non-empty `query_id`"))?;

    let payload = serde_json::from_value::<T>(serde_json::Value::Object(object.clone())).map_err(
        |source| Error::InvalidRequest {
            reason: format!("failed to parse typed web_app_data payload: {source}"),
        },
    )?;

    Ok(WebAppQueryPayload { query_id, payload })
}

#[cfg(feature = "_async")]
async fn retry_async<F, Fut>(policy: BootstrapRetryPolicy, mut op: F) -> Result<bool>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<bool>>,
{
    let max_attempts = policy.max_attempts.max(1);
    let mut attempt = 0;

    loop {
        attempt += 1;
        match op().await {
            Ok(applied) => return Ok(applied),
            Err(error) => {
                let should_retry = error.is_retryable() && attempt < max_attempts;
                if should_retry {
                    let delay = backoff_delay(
                        policy.base_backoff,
                        policy.max_backoff,
                        attempt,
                        policy.jitter_ratio,
                    );
                    tokio::time::sleep(delay).await;
                    continue;
                }
                if policy.continue_on_failure {
                    return Ok(false);
                }
                return Err(error);
            }
        }
    }
}

#[cfg(feature = "_blocking")]
fn retry_blocking<F>(policy: BootstrapRetryPolicy, mut op: F) -> Result<bool>
where
    F: FnMut() -> Result<bool>,
{
    let max_attempts = policy.max_attempts.max(1);
    let mut attempt = 0;

    loop {
        attempt += 1;
        match op() {
            Ok(applied) => return Ok(applied),
            Err(error) => {
                let should_retry = error.is_retryable() && attempt < max_attempts;
                if should_retry {
                    let delay = backoff_delay(
                        policy.base_backoff,
                        policy.max_backoff,
                        attempt,
                        policy.jitter_ratio,
                    );
                    std::thread::sleep(delay);
                    continue;
                }
                if policy.continue_on_failure {
                    return Ok(false);
                }
                return Err(error);
            }
        }
    }
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

    /// Calls JSON method with method-scoped retry policy.
    pub async fn call_json_with_retry<R, P>(
        &self,
        method: &str,
        payload: &P,
        retry: RetryConfig,
    ) -> Result<R>
    where
        R: DeserializeOwned,
        P: Serialize + ?Sized,
    {
        retry_with_config_async(&retry, || async {
            self.client.call_method(method, payload).await
        })
        .await
    }

    /// Calls any Telegram method without payload.
    pub async fn call_no_params<R>(&self, method: &str) -> Result<R>
    where
        R: DeserializeOwned,
    {
        self.client.call_method_no_params(method).await
    }

    /// Calls no-params method with method-scoped retry policy.
    pub async fn call_no_params_with_retry<R>(&self, method: &str, retry: RetryConfig) -> Result<R>
    where
        R: DeserializeOwned,
    {
        retry_with_config_async(&retry, || async {
            self.client.call_method_no_params(method).await
        })
        .await
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

    /// Calls multipart method with method-scoped retry policy.
    pub async fn call_multipart_with_retry<R, P>(
        &self,
        method: &str,
        payload: &P,
        file_field_name: &str,
        file: &UploadFile,
        retry: RetryConfig,
    ) -> Result<R>
    where
        R: DeserializeOwned,
        P: Serialize + ?Sized,
    {
        retry_with_config_async(&retry, || async {
            self.client
                .call_method_multipart(method, payload, file_field_name, file)
                .await
        })
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

    /// Calls typed request with method-scoped retry policy.
    pub async fn call_with_retry<Q>(&self, request: &Q, retry: RetryConfig) -> Result<Q::Response>
    where
        Q: AdvancedRequest,
    {
        retry_with_config_async(&retry, || async {
            self.client.call_method(Q::METHOD, request).await
        })
        .await
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

    /// Registers explicit command definitions with retry/backoff.
    pub async fn set_commands_with_retry(
        &self,
        commands: Vec<BotCommand>,
        policy: BootstrapRetryPolicy,
    ) -> Result<bool> {
        let request = SetMyCommandsRequest::new(commands)?;
        retry_async(policy, || async {
            self.client.bot().set_my_commands(&request).await
        })
        .await
    }

    /// Registers typed command definitions with optional scope and language code.
    #[cfg(feature = "bot")]
    pub async fn set_typed_commands_with_options<C>(
        &self,
        scope: Option<BotCommandScope>,
        language_code: Option<String>,
    ) -> Result<bool>
    where
        C: crate::bot::BotCommands,
    {
        let request = typed_commands_request::<C>(scope, language_code)?;
        self.client.bot().set_my_commands(&request).await
    }

    /// Registers typed command definitions with options and retry/backoff.
    #[cfg(feature = "bot")]
    pub async fn set_typed_commands_with_options_retry<C>(
        &self,
        scope: Option<BotCommandScope>,
        language_code: Option<String>,
        policy: BootstrapRetryPolicy,
    ) -> Result<bool>
    where
        C: crate::bot::BotCommands,
    {
        let request = typed_commands_request::<C>(scope, language_code)?;
        retry_async(policy, || async {
            self.client.bot().set_my_commands(&request).await
        })
        .await
    }

    /// Applies a chat menu button with retry/backoff.
    pub async fn set_chat_menu_button_with_retry(
        &self,
        request: &AdvancedSetChatMenuButtonRequest,
        policy: BootstrapRetryPolicy,
    ) -> Result<bool> {
        retry_async(policy, || async {
            self.client
                .advanced()
                .set_chat_menu_button_typed(request)
                .await
        })
        .await
    }

    /// Runs common startup bootstrap in one call (`getMe`/commands/menu button).
    pub async fn bootstrap_with_retry(
        &self,
        plan: &BootstrapPlan,
        policy: BootstrapRetryPolicy,
    ) -> Result<BootstrapReport> {
        let mut report = BootstrapReport::default();

        if plan.verify_get_me {
            report.me = Some(self.client.bot().get_me().await?);
        }
        if let Some(commands) = plan.commands.as_ref() {
            report.commands_applied = Some(
                retry_async(policy, || async {
                    self.client.bot().set_my_commands(commands).await
                })
                .await?,
            );
        }
        if let Some(menu_button) = plan.menu_button.as_ref() {
            report.menu_button_applied = Some(
                retry_async(policy, || async {
                    self.client
                        .advanced()
                        .set_chat_menu_button_typed(menu_button)
                        .await
                })
                .await?,
            );
        }

        Ok(report)
    }

    /// Runs common startup bootstrap without retries.
    pub async fn bootstrap(&self, plan: &BootstrapPlan) -> Result<BootstrapReport> {
        self.bootstrap_with_retry(
            plan,
            BootstrapRetryPolicy {
                max_attempts: 1,
                continue_on_failure: false,
                ..BootstrapRetryPolicy::default()
            },
        )
        .await
    }

    /// Answers `answerWebAppQuery` with a typed inline result payload.
    pub async fn answer_web_app_query<T>(
        &self,
        web_app_query_id: impl Into<String>,
        result: T,
    ) -> Result<SentWebAppMessage>
    where
        T: Serialize,
    {
        let result =
            InlineQueryResult::from_typed(result).map_err(|source| Error::InvalidRequest {
                reason: format!("failed to serialize WebApp inline result: {source}"),
            })?;
        let request = AdvancedAnswerWebAppQueryRequest::new(web_app_query_id, result);
        self.client
            .advanced()
            .answer_web_app_query_typed(&request)
            .await
    }

    /// Parses WebApp JSON payload and answers `answerWebAppQuery` in one step.
    pub async fn answer_web_app_query_from_payload<T, R>(
        &self,
        web_app_data: &WebAppData,
        result: R,
    ) -> Result<SentWebAppMessage>
    where
        T: DeserializeOwned,
        R: Serialize,
    {
        let envelope = parse_web_app_query_payload::<T>(web_app_data)?;
        self.answer_web_app_query(envelope.query_id, result).await
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

    /// Calls JSON method with method-scoped retry policy.
    pub fn call_json_with_retry<R, P>(
        &self,
        method: &str,
        payload: &P,
        retry: RetryConfig,
    ) -> Result<R>
    where
        R: DeserializeOwned,
        P: Serialize + ?Sized,
    {
        retry_with_config_blocking(&retry, || self.client.call_method(method, payload))
    }

    /// Calls any Telegram method without payload.
    pub fn call_no_params<R>(&self, method: &str) -> Result<R>
    where
        R: DeserializeOwned,
    {
        self.client.call_method_no_params(method)
    }

    /// Calls no-params method with method-scoped retry policy.
    pub fn call_no_params_with_retry<R>(&self, method: &str, retry: RetryConfig) -> Result<R>
    where
        R: DeserializeOwned,
    {
        retry_with_config_blocking(&retry, || self.client.call_method_no_params(method))
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

    /// Calls multipart method with method-scoped retry policy.
    pub fn call_multipart_with_retry<R, P>(
        &self,
        method: &str,
        payload: &P,
        file_field_name: &str,
        file: &UploadFile,
        retry: RetryConfig,
    ) -> Result<R>
    where
        R: DeserializeOwned,
        P: Serialize + ?Sized,
    {
        retry_with_config_blocking(&retry, || {
            self.client
                .call_method_multipart(method, payload, file_field_name, file)
        })
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

    /// Calls typed request with method-scoped retry policy.
    pub fn call_with_retry<Q>(&self, request: &Q, retry: RetryConfig) -> Result<Q::Response>
    where
        Q: AdvancedRequest,
    {
        retry_with_config_blocking(&retry, || self.client.call_method(Q::METHOD, request))
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

    /// Registers explicit command definitions with retry/backoff.
    pub fn set_commands_with_retry(
        &self,
        commands: Vec<BotCommand>,
        policy: BootstrapRetryPolicy,
    ) -> Result<bool> {
        let request = SetMyCommandsRequest::new(commands)?;
        retry_blocking(policy, || self.client.bot().set_my_commands(&request))
    }

    /// Registers typed command definitions with optional scope and language code.
    #[cfg(feature = "bot")]
    pub fn set_typed_commands_with_options<C>(
        &self,
        scope: Option<BotCommandScope>,
        language_code: Option<String>,
    ) -> Result<bool>
    where
        C: crate::bot::BotCommands,
    {
        let request = typed_commands_request::<C>(scope, language_code)?;
        self.client.bot().set_my_commands(&request)
    }

    /// Registers typed command definitions with options and retry/backoff.
    #[cfg(feature = "bot")]
    pub fn set_typed_commands_with_options_retry<C>(
        &self,
        scope: Option<BotCommandScope>,
        language_code: Option<String>,
        policy: BootstrapRetryPolicy,
    ) -> Result<bool>
    where
        C: crate::bot::BotCommands,
    {
        let request = typed_commands_request::<C>(scope, language_code)?;
        retry_blocking(policy, || self.client.bot().set_my_commands(&request))
    }

    /// Applies a chat menu button with retry/backoff.
    pub fn set_chat_menu_button_with_retry(
        &self,
        request: &AdvancedSetChatMenuButtonRequest,
        policy: BootstrapRetryPolicy,
    ) -> Result<bool> {
        retry_blocking(policy, || {
            self.client.advanced().set_chat_menu_button_typed(request)
        })
    }

    /// Runs common startup bootstrap in one call (`getMe`/commands/menu button).
    pub fn bootstrap_with_retry(
        &self,
        plan: &BootstrapPlan,
        policy: BootstrapRetryPolicy,
    ) -> Result<BootstrapReport> {
        let mut report = BootstrapReport::default();

        if plan.verify_get_me {
            report.me = Some(self.client.bot().get_me()?);
        }
        if let Some(commands) = plan.commands.as_ref() {
            report.commands_applied = Some(retry_blocking(policy, || {
                self.client.bot().set_my_commands(commands)
            })?);
        }
        if let Some(menu_button) = plan.menu_button.as_ref() {
            report.menu_button_applied = Some(retry_blocking(policy, || {
                self.client
                    .advanced()
                    .set_chat_menu_button_typed(menu_button)
            })?);
        }

        Ok(report)
    }

    /// Runs common startup bootstrap without retries.
    pub fn bootstrap(&self, plan: &BootstrapPlan) -> Result<BootstrapReport> {
        self.bootstrap_with_retry(
            plan,
            BootstrapRetryPolicy {
                max_attempts: 1,
                continue_on_failure: false,
                ..BootstrapRetryPolicy::default()
            },
        )
    }

    /// Answers `answerWebAppQuery` with a typed inline result payload.
    pub fn answer_web_app_query<T>(
        &self,
        web_app_query_id: impl Into<String>,
        result: T,
    ) -> Result<SentWebAppMessage>
    where
        T: Serialize,
    {
        let result =
            InlineQueryResult::from_typed(result).map_err(|source| Error::InvalidRequest {
                reason: format!("failed to serialize WebApp inline result: {source}"),
            })?;
        let request = AdvancedAnswerWebAppQueryRequest::new(web_app_query_id, result);
        self.client.advanced().answer_web_app_query_typed(&request)
    }

    /// Parses WebApp JSON payload and answers `answerWebAppQuery` in one step.
    pub fn answer_web_app_query_from_payload<T, R>(
        &self,
        web_app_data: &WebAppData,
        result: R,
    ) -> Result<SentWebAppMessage>
    where
        T: DeserializeOwned,
        R: Serialize,
    {
        let envelope = parse_web_app_query_payload::<T>(web_app_data)?;
        self.answer_web_app_query(envelope.query_id, result)
    }
}

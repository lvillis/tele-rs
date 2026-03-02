use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::Serialize;
use serde::de::DeserializeOwned;
use tokio::sync::{RwLock, Semaphore, mpsc, oneshot};
use tokio::task::JoinSet;
use tokio::time::sleep;

#[cfg(feature = "axum")]
pub mod axum;

use crate::api::{
    AdvancedService, BotService, ChatsService, FilesService, MessagesService, PaymentsService,
    StickersService, UpdatesService,
};
use crate::types::command::SetMyCommandsRequest;
use crate::types::common::ChatId;
use crate::types::message::{Message, SendMessageRequest, WriteAccessAllowed};
use crate::types::telegram::WebAppData;
use crate::types::update::{AnswerCallbackQueryRequest, GetUpdatesRequest, Update};
use crate::types::webhook::{DeleteWebhookRequest, SetWebhookRequest};
use crate::{Client, Error, ErrorClass, Result};

type HandlerFuture = Pin<Box<dyn Future<Output = Result<()>> + Send + 'static>>;
type SessionFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T>> + Send + 'a>>;
type SourceFuture<'a> = Pin<Box<dyn Future<Output = Result<Vec<Update>>> + Send + 'a>>;

/// Shared async update handler function.
pub type HandlerFn = Arc<dyn Fn(BotContext, Update) -> HandlerFuture + Send + Sync + 'static>;

/// Shared async middleware function.
pub type MiddlewareFn =
    Arc<dyn Fn(BotContext, Update, HandlerFn) -> HandlerFuture + Send + Sync + 'static>;

/// Hook called whenever update source polling fails.
pub type SourceErrorHook = Arc<dyn Fn(&Error) + Send + Sync + 'static>;

/// Hook called when a handler fails. The first parameter is `update_id`.
pub type HandlerErrorHook = Arc<dyn Fn(i64, &Error) + Send + Sync + 'static>;

/// Hook called for high-level runtime events.
pub type EngineEventHook = Arc<dyn Fn(&EngineEvent) + Send + Sync + 'static>;

/// Runtime event payload for observability.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum EngineEvent {
    PollStarted,
    PollCompleted {
        update_count: usize,
    },
    PollFailed {
        classification: ErrorClass,
        retryable: bool,
    },
    DispatchStarted {
        update_id: i64,
    },
    DispatchCompleted {
        outcome: DispatchOutcome,
    },
    DispatchFailed {
        update_id: i64,
        classification: ErrorClass,
    },
}

type FilterFn = Arc<dyn Fn(&Update) -> bool + Send + Sync + 'static>;

#[derive(Clone)]
struct Route {
    filter: FilterFn,
    handler: HandlerFn,
}

fn invalid_request(reason: impl Into<String>) -> Error {
    Error::InvalidRequest {
        reason: reason.into(),
    }
}

/// Parsed slash command with command name and trailing arguments.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandData {
    pub name: String,
    pub args: String,
}

impl CommandData {
    pub fn args_trimmed(&self) -> &str {
        self.args.trim()
    }

    pub fn has_args(&self) -> bool {
        !self.args_trimmed().is_empty()
    }
}

/// Handler error type that separates user-facing errors from internal failures.
#[derive(Debug)]
pub enum HandlerError {
    UserFacing { message: String },
    Internal(Error),
}

impl HandlerError {
    pub fn user(message: impl Into<String>) -> Self {
        Self::UserFacing {
            message: message.into(),
        }
    }

    pub fn internal(error: Error) -> Self {
        Self::Internal(error)
    }
}

impl From<Error> for HandlerError {
    fn from(value: Error) -> Self {
        Self::Internal(value)
    }
}

/// Ergonomic result type for bot handlers.
pub type HandlerResult = std::result::Result<(), HandlerError>;

/// Command declaration metadata used for typed command registration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CommandDescription {
    pub command: &'static str,
    pub description: &'static str,
}

/// Typed command parser contract. Intended for use with `#[derive(tele::BotCommands)]`.
pub trait BotCommands: Sized {
    fn parse(command: &str, args: &str) -> Option<Self>;
    fn descriptions() -> &'static [CommandDescription];

    fn parse_text(text: &str) -> Option<Self> {
        let command = parse_command_text(text)?;
        Self::parse(&command.name, command.args_trimmed())
    }
}

/// Declarative route-level error strategy for business handlers.
#[derive(Clone, Debug, Default)]
#[non_exhaustive]
pub enum ErrorPolicy {
    /// Propagate the handler error to `Router::dispatch`.
    #[default]
    Propagate,
    /// Suppress handler errors and continue processing.
    Ignore,
    /// Reply a user-facing fallback message and suppress the original error.
    ReplyUser { fallback_message: String },
}

/// Typed extractor contract for business handlers.
pub trait UpdateExtractor: Sized {
    fn extract(update: &Update) -> Option<Self>;

    fn describe() -> &'static str {
        "required update payload"
    }
}

/// Plain text message extractor payload.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextInput(pub String);

impl TextInput {
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl UpdateExtractor for TextInput {
    fn extract(update: &Update) -> Option<Self> {
        extract_text(update).map(|text| Self(text.to_owned()))
    }

    fn describe() -> &'static str {
        "text message"
    }
}

/// Raw callback data extractor payload.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallbackInput(pub String);

impl CallbackInput {
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl UpdateExtractor for CallbackInput {
    fn extract(update: &Update) -> Option<Self> {
        extract_callback_data(update).map(|data| Self(data.to_owned()))
    }

    fn describe() -> &'static str {
        "callback data"
    }
}

/// Mini App payload extractor payload.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WebAppInput(pub WebAppData);

impl WebAppInput {
    pub fn into_inner(self) -> WebAppData {
        self.0
    }
}

impl UpdateExtractor for WebAppInput {
    fn extract(update: &Update) -> Option<Self> {
        extract_web_app_data(update).cloned().map(Self)
    }

    fn describe() -> &'static str {
        "web app data"
    }
}

/// Write-access service payload extractor.
#[derive(Clone, Debug)]
pub struct WriteAccessAllowedInput(pub WriteAccessAllowed);

impl WriteAccessAllowedInput {
    pub fn into_inner(self) -> WriteAccessAllowed {
        self.0
    }
}

impl UpdateExtractor for WriteAccessAllowedInput {
    fn extract(update: &Update) -> Option<Self> {
        extract_write_access_allowed(update).cloned().map(Self)
    }

    fn describe() -> &'static str {
        "write access allowed"
    }
}

/// JSON-decoded callback extractor payload.
#[derive(Clone, Debug)]
pub struct JsonCallback<T>(pub T);

impl<T> JsonCallback<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> UpdateExtractor for JsonCallback<T>
where
    T: DeserializeOwned,
{
    fn extract(update: &Update) -> Option<Self> {
        extract_callback_json(update).map(Self)
    }

    fn describe() -> &'static str {
        "json callback payload"
    }
}

/// Strongly-typed command extractor payload.
#[derive(Clone, Debug)]
pub struct TypedCommandInput<C> {
    pub command: C,
    pub raw: CommandData,
}

impl<C> UpdateExtractor for TypedCommandInput<C>
where
    C: BotCommands,
{
    fn extract(update: &Update) -> Option<Self> {
        let raw = extract_command_data(update)?;
        let command = C::parse(&raw.name, raw.args_trimmed())?;
        Some(Self { command, raw })
    }

    fn describe() -> &'static str {
        "typed command"
    }
}

fn user_message_for_error(error: &Error, fallback: &str) -> String {
    match error.classification() {
        ErrorClass::Authentication => "bot authentication failed, please contact admin".to_owned(),
        ErrorClass::RateLimited => "too many requests, please retry shortly".to_owned(),
        _ => fallback.to_owned(),
    }
}

/// Dispatch context passed to handlers and middlewares.
#[derive(Clone)]
pub struct BotContext {
    client: Client,
}

impl BotContext {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    pub fn bot(&self) -> BotService {
        self.client.bot()
    }

    pub fn messages(&self) -> MessagesService {
        self.client.messages()
    }

    pub fn chats(&self) -> ChatsService {
        self.client.chats()
    }

    pub fn files(&self) -> FilesService {
        self.client.files()
    }

    pub fn stickers(&self) -> StickersService {
        self.client.stickers()
    }

    pub fn payments(&self) -> PaymentsService {
        self.client.payments()
    }

    pub fn advanced(&self) -> AdvancedService {
        self.client.advanced()
    }

    pub fn updates(&self) -> UpdatesService {
        self.client.updates()
    }

    /// Spawns a reliable outbox worker for send-side retry, throttling and idempotency.
    pub fn spawn_outbox(&self, config: OutboxConfig) -> BotOutbox {
        BotOutbox::spawn(self.client.clone(), config)
    }

    /// Sends plain text to a target chat.
    pub async fn send_text(
        &self,
        chat_id: impl Into<ChatId>,
        text: impl Into<String>,
    ) -> Result<Message> {
        let request = SendMessageRequest::new(chat_id, text)?;
        self.messages().send_message(&request).await
    }

    /// Replies with plain text using the canonical chat id extracted from update.
    pub async fn reply_text(&self, update: &Update, text: impl Into<String>) -> Result<Message> {
        let Some(chat_id) = update_chat_id(update) else {
            return Err(invalid_request(
                "update does not contain a chat id for reply",
            ));
        };

        self.send_text(chat_id, text).await
    }

    /// Answers callback query by id.
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
        self.updates().answer_callback_query(&request).await
    }

    /// Answers callback query using id extracted from update.
    pub async fn answer_callback_from_update(
        &self,
        update: &Update,
        text: Option<String>,
    ) -> Result<bool> {
        let Some(callback_query_id) = update.callback_query.as_ref().map(|query| query.id.clone())
        else {
            return Err(invalid_request(
                "update does not contain callback query for answerCallbackQuery",
            ));
        };

        self.answer_callback(callback_query_id, text).await
    }

    /// Registers typed command definitions from a `BotCommands` enum.
    pub async fn set_typed_commands<C: BotCommands>(&self) -> Result<bool> {
        let request = SetMyCommandsRequest::new(command_definitions::<C>())?;
        self.bot().set_my_commands(&request).await
    }

    /// Converts high-level handler error into transportable SDK result.
    pub async fn resolve_handler_error(&self, update: &Update, error: HandlerError) -> Result<()> {
        match error {
            HandlerError::UserFacing { message } => {
                let _ = self.reply_text(update, message).await?;
                Ok(())
            }
            HandlerError::Internal(error) => Err(error),
        }
    }
}

/// Declarative update router with middleware support.
#[derive(Clone, Default)]
pub struct Router {
    routes: Vec<Route>,
    middlewares: Vec<MiddlewareFn>,
    fallback: Option<HandlerFn>,
}

impl Router {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn route<P, H, Fut>(&mut self, predicate: P, handler: H) -> &mut Self
    where
        P: Fn(&Update) -> bool + Send + Sync + 'static,
        H: Fn(BotContext, Update) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        self.routes.push(Route {
            filter: Arc::new(predicate),
            handler: to_handler_fn(handler),
        });
        self
    }

    /// Adds route with declarative error policy.
    pub fn route_with_policy<P, H, Fut>(
        &mut self,
        predicate: P,
        policy: ErrorPolicy,
        handler: H,
    ) -> &mut Self
    where
        P: Fn(&Update) -> bool + Send + Sync + 'static,
        H: Fn(BotContext, Update) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        let handler = Arc::new(handler);
        self.route(predicate, move |context, update| {
            let handler = Arc::clone(&handler);
            let context_for_policy = context.clone();
            let update_for_policy = update.clone();
            let policy = policy.clone();
            async move {
                match handler(context, update).await {
                    Ok(()) => Ok(()),
                    Err(error) => match policy {
                        ErrorPolicy::Propagate => Err(error),
                        ErrorPolicy::Ignore => Ok(()),
                        ErrorPolicy::ReplyUser { fallback_message } => {
                            let message = user_message_for_error(&error, &fallback_message);
                            let _ = context_for_policy
                                .reply_text(&update_for_policy, message)
                                .await?;
                            Ok(())
                        }
                    },
                }
            }
        })
    }

    /// Adds route with typed extractor so handlers can focus on business input.
    pub fn on_extracted<E, H, Fut>(&mut self, handler: H) -> &mut Self
    where
        E: UpdateExtractor + Send + 'static,
        H: Fn(BotContext, Update, E) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        let handler = Arc::new(handler);
        self.route(
            |update| E::extract(update).is_some(),
            move |context, update| {
                let handler = Arc::clone(&handler);
                let extracted = E::extract(&update);
                async move {
                    let Some(extracted) = extracted else {
                        return Err(invalid_request(format!(
                            "update does not contain {}",
                            E::describe()
                        )));
                    };
                    handler(context, update, extracted).await
                }
            },
        )
    }

    /// Adds extractor route with additional predicate over extracted payload.
    pub fn on_extracted_filter<E, P, H, Fut>(&mut self, predicate: P, handler: H) -> &mut Self
    where
        E: UpdateExtractor + Send + 'static,
        P: Fn(&E, &Update) -> bool + Send + Sync + 'static,
        H: Fn(BotContext, Update, E) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        let predicate = Arc::new(predicate);
        let handler = Arc::new(handler);
        self.route(
            {
                let predicate = Arc::clone(&predicate);
                move |update| {
                    let Some(extracted) = E::extract(update) else {
                        return false;
                    };
                    predicate(&extracted, update)
                }
            },
            move |context, update| {
                let predicate = Arc::clone(&predicate);
                let handler = Arc::clone(&handler);
                let extracted = E::extract(&update);
                async move {
                    let Some(extracted) = extracted else {
                        return Err(invalid_request(format!(
                            "update does not contain {}",
                            E::describe()
                        )));
                    };
                    if !predicate(&extracted, &update) {
                        return Ok(());
                    }
                    handler(context, update, extracted).await
                }
            },
        )
    }

    /// Adds extractor route with mapping step before invoking the handler.
    pub fn on_extracted_map<E, T, M, H, Fut>(&mut self, mapper: M, handler: H) -> &mut Self
    where
        E: UpdateExtractor + Send + 'static,
        T: Send + 'static,
        M: Fn(E, &Update) -> Option<T> + Send + Sync + 'static,
        H: Fn(BotContext, Update, T) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        let mapper = Arc::new(mapper);
        let handler = Arc::new(handler);
        self.route(
            {
                let mapper = Arc::clone(&mapper);
                move |update| {
                    let Some(extracted) = E::extract(update) else {
                        return false;
                    };
                    mapper(extracted, update).is_some()
                }
            },
            move |context, update| {
                let mapper = Arc::clone(&mapper);
                let handler = Arc::clone(&handler);
                let extracted = E::extract(&update);
                async move {
                    let Some(extracted) = extracted else {
                        return Err(invalid_request(format!(
                            "update does not contain {}",
                            E::describe()
                        )));
                    };
                    let Some(mapped) = mapper(extracted, &update) else {
                        return Ok(());
                    };
                    handler(context, update, mapped).await
                }
            },
        )
    }

    /// Adds extractor route with a guard check before invoking the handler.
    pub fn on_extracted_guard<E, G, H, Fut>(&mut self, guard: G, handler: H) -> &mut Self
    where
        E: UpdateExtractor + Send + 'static,
        G: Fn(&E, &Update) -> HandlerResult + Send + Sync + 'static,
        H: Fn(BotContext, Update, E) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        let guard = Arc::new(guard);
        let handler = Arc::new(handler);
        self.route_fallible(
            |update| E::extract(update).is_some(),
            move |context, update| {
                let guard = Arc::clone(&guard);
                let handler = Arc::clone(&handler);
                let extracted = E::extract(&update);
                async move {
                    let Some(extracted) = extracted else {
                        return Err(HandlerError::internal(invalid_request(format!(
                            "update does not contain {}",
                            E::describe()
                        ))));
                    };
                    guard(&extracted, &update)?;
                    handler(context, update, extracted)
                        .await
                        .map_err(HandlerError::from)
                }
            },
        )
    }

    /// Adds fallible extractor route with `HandlerError`.
    pub fn on_extracted_fallible<E, H, Fut>(&mut self, handler: H) -> &mut Self
    where
        E: UpdateExtractor + Send + 'static,
        H: Fn(BotContext, Update, E) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult> + Send + 'static,
    {
        let handler = Arc::new(handler);
        self.route_fallible(
            |update| E::extract(update).is_some(),
            move |context, update| {
                let handler = Arc::clone(&handler);
                let extracted = E::extract(&update);
                async move {
                    let Some(extracted) = extracted else {
                        return Err(HandlerError::internal(invalid_request(format!(
                            "update does not contain {}",
                            E::describe()
                        ))));
                    };
                    handler(context, update, extracted).await
                }
            },
        )
    }

    /// Adds extractor route with declarative error policy.
    pub fn on_extracted_with_policy<E, H, Fut>(
        &mut self,
        policy: ErrorPolicy,
        handler: H,
    ) -> &mut Self
    where
        E: UpdateExtractor + Send + 'static,
        H: Fn(BotContext, Update, E) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        let handler = Arc::new(handler);
        self.route_with_policy(
            |update| E::extract(update).is_some(),
            policy,
            move |context, update| {
                let handler = Arc::clone(&handler);
                let extracted = E::extract(&update);
                async move {
                    let Some(extracted) = extracted else {
                        return Err(invalid_request(format!(
                            "update does not contain {}",
                            E::describe()
                        )));
                    };
                    handler(context, update, extracted).await
                }
            },
        )
    }

    /// Adds route with fallible handler that can return user-facing errors.
    pub fn route_fallible<P, H, Fut>(&mut self, predicate: P, handler: H) -> &mut Self
    where
        P: Fn(&Update) -> bool + Send + Sync + 'static,
        H: Fn(BotContext, Update) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult> + Send + 'static,
    {
        let handler = Arc::new(handler);
        self.route(predicate, move |context, update| {
            let handler = Arc::clone(&handler);
            let context_for_error = context.clone();
            let update_for_error = update.clone();
            async move {
                match handler(context, update).await {
                    Ok(()) => Ok(()),
                    Err(error) => {
                        context_for_error
                            .resolve_handler_error(&update_for_error, error)
                            .await
                    }
                }
            }
        })
    }

    pub fn on_message<H, Fut>(&mut self, handler: H) -> &mut Self
    where
        H: Fn(BotContext, Update) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        self.route(|update| update.message.is_some(), handler)
    }

    pub fn on_message_fallible<H, Fut>(&mut self, handler: H) -> &mut Self
    where
        H: Fn(BotContext, Update) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult> + Send + 'static,
    {
        self.route_fallible(|update| update.message.is_some(), handler)
    }

    pub fn on_text<H, Fut>(&mut self, handler: H) -> &mut Self
    where
        H: Fn(BotContext, Update) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        self.route(|update| extract_text(update).is_some(), handler)
    }

    /// Routes text updates with extracted string payload.
    pub fn on_text_input<H, Fut>(&mut self, handler: H) -> &mut Self
    where
        H: Fn(BotContext, Update, TextInput) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        self.on_extracted::<TextInput, _, _>(handler)
    }

    pub fn on_callback_query<H, Fut>(&mut self, handler: H) -> &mut Self
    where
        H: Fn(BotContext, Update) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        self.route(|update| update.callback_query.is_some(), handler)
    }

    pub fn on_inline_query<H, Fut>(&mut self, handler: H) -> &mut Self
    where
        H: Fn(BotContext, Update) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        self.route(|update| update.inline_query.is_some(), handler)
    }

    /// Routes callback updates with extracted callback payload.
    pub fn on_callback_input<H, Fut>(&mut self, handler: H) -> &mut Self
    where
        H: Fn(BotContext, Update, CallbackInput) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        self.on_extracted::<CallbackInput, _, _>(handler)
    }

    /// Routes callback updates with JSON-decoded callback payload.
    pub fn on_callback_json<T, H, Fut>(&mut self, handler: H) -> &mut Self
    where
        T: DeserializeOwned + Send + 'static,
        H: Fn(BotContext, Update, JsonCallback<T>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        self.on_extracted::<JsonCallback<T>, _, _>(handler)
    }

    pub fn on_command<H, Fut>(&mut self, command: impl Into<String>, handler: H) -> &mut Self
    where
        H: Fn(BotContext, Update) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        let expected = command.into();
        self.route(
            move |update| extract_command(update).is_some_and(|command| command == expected),
            handler,
        )
    }

    /// Routes specific command with declarative error policy.
    pub fn on_command_with_policy<H, Fut>(
        &mut self,
        command: impl Into<String>,
        policy: ErrorPolicy,
        handler: H,
    ) -> &mut Self
    where
        H: Fn(BotContext, Update) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        let expected = command.into();
        self.route_with_policy(
            move |update| extract_command(update).is_some_and(|command| command == expected),
            policy,
            handler,
        )
    }

    pub fn on_command_fallible<H, Fut>(
        &mut self,
        command: impl Into<String>,
        handler: H,
    ) -> &mut Self
    where
        H: Fn(BotContext, Update) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult> + Send + 'static,
    {
        let expected = command.into();
        self.route_fallible(
            move |update| extract_command(update).is_some_and(|command| command == expected),
            handler,
        )
    }

    pub fn on_any_command<H, Fut>(&mut self, handler: H) -> &mut Self
    where
        H: Fn(BotContext, Update, CommandData) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        let handler = Arc::new(handler);
        self.route(
            |update| extract_command_data(update).is_some(),
            move |context, update| {
                let handler = Arc::clone(&handler);
                let command = extract_command_data(&update);
                async move {
                    let Some(command) = command else {
                        return Err(invalid_request("update does not contain a valid command"));
                    };
                    handler(context, update, command).await
                }
            },
        )
    }

    pub fn on_typed_command<C, H, Fut>(&mut self, handler: H) -> &mut Self
    where
        C: BotCommands + Send + 'static,
        H: Fn(BotContext, Update, C) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        let handler = Arc::new(handler);
        self.route(
            |update| parse_typed_command::<C>(update).is_some(),
            move |context, update| {
                let handler = Arc::clone(&handler);
                let parsed = parse_typed_command::<C>(&update);
                async move {
                    let Some(command) = parsed else {
                        return Err(invalid_request(
                            "update does not contain a valid typed command",
                        ));
                    };
                    handler(context, update, command).await
                }
            },
        )
    }

    /// Routes typed command updates with both parsed command and raw command data.
    pub fn on_typed_command_input<C, H, Fut>(&mut self, handler: H) -> &mut Self
    where
        C: BotCommands + Send + 'static,
        H: Fn(BotContext, Update, TypedCommandInput<C>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        self.on_extracted::<TypedCommandInput<C>, _, _>(handler)
    }

    pub fn on_typed_command_fallible<C, H, Fut>(&mut self, handler: H) -> &mut Self
    where
        C: BotCommands + Send + 'static,
        H: Fn(BotContext, Update, C) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult> + Send + 'static,
    {
        let handler = Arc::new(handler);
        self.route_fallible(
            |update| parse_typed_command::<C>(update).is_some(),
            move |context, update| {
                let handler = Arc::clone(&handler);
                let parsed = parse_typed_command::<C>(&update);
                async move {
                    let Some(command) = parsed else {
                        return Err(HandlerError::internal(invalid_request(
                            "update does not contain a valid typed command",
                        )));
                    };
                    handler(context, update, command).await
                }
            },
        )
    }

    pub fn fallback<H, Fut>(&mut self, handler: H) -> &mut Self
    where
        H: Fn(BotContext, Update) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        self.fallback = Some(to_handler_fn(handler));
        self
    }

    pub fn middleware<M, Fut>(&mut self, middleware: M) -> &mut Self
    where
        M: Fn(BotContext, Update, HandlerFn) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        self.middlewares.push(to_middleware_fn(middleware));
        self
    }

    /// Dispatches a single update to the first matching route.
    pub async fn dispatch(&self, context: BotContext, update: Update) -> Result<bool> {
        let handler = self
            .routes
            .iter()
            .find(|route| (route.filter)(&update))
            .map(|route| Arc::clone(&route.handler))
            .or_else(|| self.fallback.as_ref().map(Arc::clone));

        let Some(handler) = handler else {
            return Ok(false);
        };

        let wrapped = self.apply_middlewares(handler);
        wrapped(context, update).await?;
        Ok(true)
    }

    fn apply_middlewares(&self, handler: HandlerFn) -> HandlerFn {
        let mut wrapped = handler;

        for middleware in self.middlewares.iter().rev() {
            let middleware = Arc::clone(middleware);
            let next = Arc::clone(&wrapped);

            wrapped = Arc::new(move |context: BotContext, update: Update| {
                let next_handler = Arc::clone(&next);
                middleware(context, update, next_handler)
            });
        }

        wrapped
    }
}

fn to_handler_fn<H, Fut>(handler: H) -> HandlerFn
where
    H: Fn(BotContext, Update) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<()>> + Send + 'static,
{
    Arc::new(move |context, update| Box::pin(handler(context, update)))
}

fn to_middleware_fn<M, Fut>(middleware: M) -> MiddlewareFn
where
    M: Fn(BotContext, Update, HandlerFn) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<()>> + Send + 'static,
{
    Arc::new(move |context, update, next| Box::pin(middleware(context, update, next)))
}

/// Parses a slash command from raw message text.
pub fn parse_command_text(text: &str) -> Option<CommandData> {
    let token = text.split_whitespace().next()?;
    let command = token.strip_prefix('/')?;

    let name = match command.split_once('@') {
        Some((command, _bot_name)) => command,
        None => command,
    };

    if name.is_empty() {
        return None;
    }

    let args = text[token.len()..].trim().to_owned();
    Some(CommandData {
        name: name.to_owned(),
        args,
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum QuoteKind {
    Single,
    Double,
}

/// Tokenizes command arguments with quote and escape support.
///
/// Accepts single (`'...'`) and double (`"..."`) quotes and backslash escapes (`\`).
/// Returns `None` when input contains an unterminated quote or a dangling escape.
pub fn tokenize_command_args(args: &str) -> Option<Vec<String>> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut chars = args.chars().peekable();
    let mut quote = None;
    let mut token_started = false;

    while let Some(ch) = chars.next() {
        match quote {
            Some(QuoteKind::Single) => match ch {
                '\'' => quote = None,
                '\\' => {
                    let escaped = chars.next()?;
                    current.push(escaped);
                    token_started = true;
                }
                _ => {
                    current.push(ch);
                    token_started = true;
                }
            },
            Some(QuoteKind::Double) => match ch {
                '"' => quote = None,
                '\\' => {
                    let escaped = chars.next()?;
                    current.push(escaped);
                    token_started = true;
                }
                _ => {
                    current.push(ch);
                    token_started = true;
                }
            },
            None => match ch {
                '\'' => {
                    quote = Some(QuoteKind::Single);
                    token_started = true;
                }
                '"' => {
                    quote = Some(QuoteKind::Double);
                    token_started = true;
                }
                '\\' => {
                    let escaped = chars.next()?;
                    current.push(escaped);
                    token_started = true;
                }
                _ if ch.is_whitespace() => {
                    if token_started {
                        tokens.push(std::mem::take(&mut current));
                        token_started = false;
                    }

                    while chars.peek().is_some_and(|next| next.is_whitespace()) {
                        let _ = chars.next();
                    }
                }
                _ => {
                    current.push(ch);
                    token_started = true;
                }
            },
        }
    }

    if quote.is_some() {
        return None;
    }

    if token_started {
        tokens.push(current);
    }

    Some(tokens)
}

/// Returns canonical message object from update, prioritizing incoming message variants.
pub fn extract_message(update: &Update) -> Option<&Message> {
    if let Some(message) = update.message.as_ref() {
        return Some(message);
    }
    if let Some(message) = update.edited_message.as_ref() {
        return Some(message);
    }
    if let Some(message) = update.channel_post.as_ref() {
        return Some(message);
    }
    if let Some(message) = update.edited_channel_post.as_ref() {
        return Some(message);
    }

    update
        .callback_query
        .as_ref()
        .and_then(|query| query.message.as_ref())
}

/// Returns plain text from extracted message when available.
pub fn extract_text(update: &Update) -> Option<&str> {
    extract_message(update)?.text.as_deref()
}

/// Returns Mini App payload from extracted message when available.
pub fn extract_web_app_data(update: &Update) -> Option<&WebAppData> {
    extract_message(update)?.web_app_data()
}

/// Returns write-access service payload from extracted message when available.
pub fn extract_write_access_allowed(update: &Update) -> Option<&WriteAccessAllowed> {
    extract_message(update)?.write_access_allowed()
}

/// Returns callback query data payload from update.
pub fn extract_callback_data(update: &Update) -> Option<&str> {
    update.callback_query.as_ref()?.data.as_deref()
}

/// Returns JSON-decoded callback payload from update.
pub fn extract_callback_json<T>(update: &Update) -> Option<T>
where
    T: DeserializeOwned,
{
    let payload = extract_callback_data(update)?;
    serde_json::from_str(payload).ok()
}

/// Returns command name from message text (without leading `/` and optional `@botname`).
pub fn extract_command(update: &Update) -> Option<&str> {
    let text = update.message.as_ref()?.text.as_deref()?;
    let token = text.split_whitespace().next()?;
    let command = token.strip_prefix('/')?;

    let command = match command.split_once('@') {
        Some((command, _bot_name)) => command,
        None => command,
    };

    if command.is_empty() {
        return None;
    }

    Some(command)
}

/// Returns command arguments as a trimmed string slice.
pub fn extract_command_args(update: &Update) -> Option<&str> {
    let text = update.message.as_ref()?.text.as_deref()?;
    let token = text.split_whitespace().next()?;
    let _ = token.strip_prefix('/')?;
    Some(text[token.len()..].trim())
}

/// Returns parsed command with owned command name and args.
pub fn extract_command_data(update: &Update) -> Option<CommandData> {
    parse_command_text(update.message.as_ref()?.text.as_deref()?)
}

/// Parses typed command from incoming update using a `BotCommands` implementation.
pub fn parse_typed_command<C: BotCommands>(update: &Update) -> Option<C> {
    let command = extract_command_data(update)?;
    C::parse(&command.name, command.args_trimmed())
}

/// Builds Telegram command descriptors from a typed command enum.
pub fn command_definitions<C: BotCommands>() -> Vec<crate::types::command::BotCommand> {
    C::descriptions()
        .iter()
        .map(|description| crate::types::command::BotCommand {
            command: description.command.to_owned(),
            description: description.description.to_owned(),
        })
        .collect()
}

/// Convenience extractor trait for update handlers.
pub trait UpdateExt {
    fn message(&self) -> Option<&Message>;
    fn text(&self) -> Option<&str>;
    fn web_app_data(&self) -> Option<&WebAppData> {
        None
    }
    fn write_access_allowed(&self) -> Option<&WriteAccessAllowed> {
        None
    }
    fn callback_data(&self) -> Option<&str>;
    fn callback_json<T>(&self) -> Option<T>
    where
        T: DeserializeOwned;
    fn command(&self) -> Option<&str>;
    fn command_args(&self) -> Option<&str>;
    fn command_data(&self) -> Option<CommandData>;
    fn typed_command<C>(&self) -> Option<C>
    where
        C: BotCommands;
    fn chat_id(&self) -> Option<i64>;
}

impl UpdateExt for Update {
    fn message(&self) -> Option<&Message> {
        extract_message(self)
    }

    fn text(&self) -> Option<&str> {
        extract_text(self)
    }

    fn web_app_data(&self) -> Option<&WebAppData> {
        extract_web_app_data(self)
    }

    fn write_access_allowed(&self) -> Option<&WriteAccessAllowed> {
        extract_write_access_allowed(self)
    }

    fn callback_data(&self) -> Option<&str> {
        extract_callback_data(self)
    }

    fn callback_json<T>(&self) -> Option<T>
    where
        T: DeserializeOwned,
    {
        extract_callback_json(self)
    }

    fn command(&self) -> Option<&str> {
        extract_command(self)
    }

    fn command_args(&self) -> Option<&str> {
        extract_command_args(self)
    }

    fn command_data(&self) -> Option<CommandData> {
        extract_command_data(self)
    }

    fn typed_command<C>(&self) -> Option<C>
    where
        C: BotCommands,
    {
        parse_typed_command(self)
    }

    fn chat_id(&self) -> Option<i64> {
        update_chat_id(self)
    }
}

/// Tries to extract a canonical chat id from an incoming update.
pub fn update_chat_id(update: &Update) -> Option<i64> {
    extract_message(update).map(|message| message.chat.id)
}

/// State transition result for a finite-state machine.
#[derive(Clone, Debug)]
pub enum StateTransition<S> {
    Keep,
    Set(S),
    Clear,
}

/// Abstract async session-state store.
pub trait SessionStore<S>: Send + Sync + 'static
where
    S: Clone + Send + Sync + 'static,
{
    fn load<'a>(&'a self, chat_id: i64) -> SessionFuture<'a, Option<S>>;
    fn save<'a>(&'a self, chat_id: i64, state: S) -> SessionFuture<'a, ()>;
    fn clear<'a>(&'a self, chat_id: i64) -> SessionFuture<'a, ()>;
}

/// In-memory session store for prototyping and small bots.
pub struct InMemorySessionStore<S>
where
    S: Clone + Send + Sync + 'static,
{
    inner: Arc<RwLock<HashMap<i64, S>>>,
}

impl<S> Clone for InMemorySessionStore<S>
where
    S: Clone + Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<S> Default for InMemorySessionStore<S>
where
    S: Clone + Send + Sync + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<S> InMemorySessionStore<S>
where
    S: Clone + Send + Sync + 'static,
{
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl<S> SessionStore<S> for InMemorySessionStore<S>
where
    S: Clone + Send + Sync + 'static,
{
    fn load<'a>(&'a self, chat_id: i64) -> SessionFuture<'a, Option<S>> {
        Box::pin(async move {
            let guard = self.inner.read().await;
            Ok(guard.get(&chat_id).cloned())
        })
    }

    fn save<'a>(&'a self, chat_id: i64, state: S) -> SessionFuture<'a, ()> {
        Box::pin(async move {
            let mut guard = self.inner.write().await;
            guard.insert(chat_id, state);
            Ok(())
        })
    }

    fn clear<'a>(&'a self, chat_id: i64) -> SessionFuture<'a, ()> {
        Box::pin(async move {
            let mut guard = self.inner.write().await;
            guard.remove(&chat_id);
            Ok(())
        })
    }
}

/// JSON-file backed session store for bots that need process restart recovery.
pub struct JsonFileSessionStore<S>
where
    S: Clone + Send + Sync + Serialize + DeserializeOwned + 'static,
{
    path: PathBuf,
    inner: Arc<RwLock<HashMap<i64, S>>>,
}

impl<S> Clone for JsonFileSessionStore<S>
where
    S: Clone + Send + Sync + Serialize + DeserializeOwned + 'static,
{
    fn clone(&self) -> Self {
        Self {
            path: self.path.clone(),
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<S> JsonFileSessionStore<S>
where
    S: Clone + Send + Sync + Serialize + DeserializeOwned + 'static,
{
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let initial = load_session_snapshot::<S>(&path)?;
        Ok(Self {
            path,
            inner: Arc::new(RwLock::new(initial)),
        })
    }

    pub fn path(&self) -> &Path {
        self.path.as_path()
    }
}

impl<S> SessionStore<S> for JsonFileSessionStore<S>
where
    S: Clone + Send + Sync + Serialize + DeserializeOwned + 'static,
{
    fn load<'a>(&'a self, chat_id: i64) -> SessionFuture<'a, Option<S>> {
        Box::pin(async move {
            let guard = self.inner.read().await;
            Ok(guard.get(&chat_id).cloned())
        })
    }

    fn save<'a>(&'a self, chat_id: i64, state: S) -> SessionFuture<'a, ()> {
        Box::pin(async move {
            let snapshot = {
                let mut guard = self.inner.write().await;
                guard.insert(chat_id, state);
                guard.clone()
            };
            persist_session_snapshot(self.path.as_path(), &snapshot)?;
            Ok(())
        })
    }

    fn clear<'a>(&'a self, chat_id: i64) -> SessionFuture<'a, ()> {
        Box::pin(async move {
            let snapshot = {
                let mut guard = self.inner.write().await;
                guard.remove(&chat_id);
                guard.clone()
            };
            persist_session_snapshot(self.path.as_path(), &snapshot)?;
            Ok(())
        })
    }
}

fn load_session_snapshot<S>(path: &Path) -> Result<HashMap<i64, S>>
where
    S: Clone + Send + Sync + Serialize + DeserializeOwned + 'static,
{
    if !path.exists() {
        return Ok(HashMap::new());
    }

    let raw = fs::read(path).map_err(|source| Error::ReadLocalFile {
        path: path.display().to_string(),
        source,
    })?;

    if raw.is_empty() {
        return Ok(HashMap::new());
    }

    serde_json::from_slice(&raw).map_err(|source| Error::InvalidRequest {
        reason: format!(
            "failed to deserialize session store `{}`: {source}",
            path.display()
        ),
    })
}

fn persist_session_snapshot<S>(path: &Path, snapshot: &HashMap<i64, S>) -> Result<()>
where
    S: Clone + Send + Sync + Serialize + DeserializeOwned + 'static,
{
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|source| Error::InvalidRequest {
        reason: format!(
            "failed to create session store directory `{}`: {source}",
            parent.display()
        ),
    })?;

    let encoded =
        serde_json::to_vec(snapshot).map_err(|source| Error::SerializeRequest { source })?;
    fs::write(path, encoded).map_err(|source| Error::InvalidRequest {
        reason: format!(
            "failed to write session store `{}`: {source}",
            path.display()
        ),
    })?;
    Ok(())
}

#[cfg(feature = "redis-session")]
/// Redis-backed session store for distributed bot deployments.
pub struct RedisSessionStore<S>
where
    S: Clone + Send + Sync + Serialize + DeserializeOwned + 'static,
{
    client: redis::Client,
    namespace: String,
    _state: std::marker::PhantomData<S>,
}

#[cfg(feature = "redis-session")]
impl<S> Clone for RedisSessionStore<S>
where
    S: Clone + Send + Sync + Serialize + DeserializeOwned + 'static,
{
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            namespace: self.namespace.clone(),
            _state: std::marker::PhantomData,
        }
    }
}

#[cfg(feature = "redis-session")]
impl<S> RedisSessionStore<S>
where
    S: Clone + Send + Sync + Serialize + DeserializeOwned + 'static,
{
    pub fn new(redis_url: &str, namespace: impl Into<String>) -> Result<Self> {
        let namespace = namespace.into();
        if namespace.trim().is_empty() {
            return Err(invalid_request("redis session namespace cannot be empty"));
        }

        let client = redis::Client::open(redis_url).map_err(|source| {
            invalid_request(format!(
                "failed to create redis client `{redis_url}`: {source}"
            ))
        })?;

        Ok(Self {
            client,
            namespace,
            _state: std::marker::PhantomData,
        })
    }

    pub fn namespace(&self) -> &str {
        self.namespace.as_str()
    }

    fn session_key(&self, chat_id: i64) -> String {
        format!("{}:{chat_id}", self.namespace)
    }

    async fn connection(&self) -> Result<redis::aio::MultiplexedConnection> {
        self.client
            .get_multiplexed_async_connection()
            .await
            .map_err(|source| invalid_request(format!("failed to connect redis: {source}")))
    }
}

#[cfg(feature = "redis-session")]
impl<S> SessionStore<S> for RedisSessionStore<S>
where
    S: Clone + Send + Sync + Serialize + DeserializeOwned + 'static,
{
    fn load<'a>(&'a self, chat_id: i64) -> SessionFuture<'a, Option<S>> {
        Box::pin(async move {
            let key = self.session_key(chat_id);
            let mut connection = self.connection().await?;
            let payload: Option<String> = redis::cmd("GET")
                .arg(&key)
                .query_async(&mut connection)
                .await
                .map_err(|source| invalid_request(format!("redis GET `{key}` failed: {source}")))?;

            let Some(payload) = payload else {
                return Ok(None);
            };

            let state = serde_json::from_str::<S>(&payload).map_err(|source| {
                invalid_request(format!(
                    "redis state decode failed for key `{key}`: {source}"
                ))
            })?;
            Ok(Some(state))
        })
    }

    fn save<'a>(&'a self, chat_id: i64, state: S) -> SessionFuture<'a, ()> {
        Box::pin(async move {
            let key = self.session_key(chat_id);
            let payload = serde_json::to_string(&state)
                .map_err(|source| Error::SerializeRequest { source })?;
            let mut connection = self.connection().await?;
            let _: () = redis::cmd("SET")
                .arg(&key)
                .arg(&payload)
                .query_async(&mut connection)
                .await
                .map_err(|source| invalid_request(format!("redis SET `{key}` failed: {source}")))?;
            Ok(())
        })
    }

    fn clear<'a>(&'a self, chat_id: i64) -> SessionFuture<'a, ()> {
        Box::pin(async move {
            let key = self.session_key(chat_id);
            let mut connection = self.connection().await?;
            let _: i64 = redis::cmd("DEL")
                .arg(&key)
                .query_async(&mut connection)
                .await
                .map_err(|source| invalid_request(format!("redis DEL `{key}` failed: {source}")))?;
            Ok(())
        })
    }
}

#[cfg(feature = "postgres-session")]
/// Postgres-backed session store for durable multi-instance bots.
pub struct PostgresSessionStore<S>
where
    S: Clone + Send + Sync + Serialize + DeserializeOwned + 'static,
{
    client: Arc<tokio_postgres::Client>,
    table: String,
    _state: std::marker::PhantomData<S>,
}

#[cfg(feature = "postgres-session")]
impl<S> Clone for PostgresSessionStore<S>
where
    S: Clone + Send + Sync + Serialize + DeserializeOwned + 'static,
{
    fn clone(&self) -> Self {
        Self {
            client: Arc::clone(&self.client),
            table: self.table.clone(),
            _state: std::marker::PhantomData,
        }
    }
}

#[cfg(feature = "postgres-session")]
impl<S> PostgresSessionStore<S>
where
    S: Clone + Send + Sync + Serialize + DeserializeOwned + 'static,
{
    pub async fn connect(database_url: &str, table: impl Into<String>) -> Result<Self> {
        let table = table.into();
        validate_sql_identifier(&table)?;

        let (client, connection) = tokio_postgres::connect(database_url, tokio_postgres::NoTls)
            .await
            .map_err(|source| {
                invalid_request(format!(
                    "failed to connect postgres `{database_url}`: {source}"
                ))
            })?;

        tokio::spawn(async move {
            let _ = connection.await;
        });

        let create = format!(
            "CREATE TABLE IF NOT EXISTS {table} (chat_id BIGINT PRIMARY KEY, state TEXT NOT NULL)"
        );
        client.execute(&create, &[]).await.map_err(|source| {
            invalid_request(format!(
                "failed to create postgres session table `{table}`: {source}"
            ))
        })?;

        Ok(Self {
            client: Arc::new(client),
            table,
            _state: std::marker::PhantomData,
        })
    }

    pub fn table(&self) -> &str {
        self.table.as_str()
    }
}

#[cfg(feature = "postgres-session")]
impl<S> SessionStore<S> for PostgresSessionStore<S>
where
    S: Clone + Send + Sync + Serialize + DeserializeOwned + 'static,
{
    fn load<'a>(&'a self, chat_id: i64) -> SessionFuture<'a, Option<S>> {
        Box::pin(async move {
            let query = format!("SELECT state FROM {} WHERE chat_id = $1", self.table);
            let row = self
                .client
                .query_opt(&query, &[&chat_id])
                .await
                .map_err(|source| {
                    invalid_request(format!(
                        "postgres load failed for chat_id `{chat_id}`: {source}"
                    ))
                })?;

            let Some(row) = row else {
                return Ok(None);
            };

            let payload: String = row.try_get(0).map_err(|source| {
                invalid_request(format!(
                    "postgres session payload decode failed for chat_id `{chat_id}`: {source}"
                ))
            })?;
            let state = serde_json::from_str::<S>(&payload).map_err(|source| {
                invalid_request(format!(
                    "postgres session json decode failed for chat_id `{chat_id}`: {source}"
                ))
            })?;
            Ok(Some(state))
        })
    }

    fn save<'a>(&'a self, chat_id: i64, state: S) -> SessionFuture<'a, ()> {
        Box::pin(async move {
            let query = format!(
                "INSERT INTO {} (chat_id, state) VALUES ($1, $2) \
                 ON CONFLICT (chat_id) DO UPDATE SET state = EXCLUDED.state",
                self.table
            );
            let payload = serde_json::to_string(&state)
                .map_err(|source| Error::SerializeRequest { source })?;
            self.client
                .execute(&query, &[&chat_id, &payload])
                .await
                .map_err(|source| {
                    invalid_request(format!(
                        "postgres save failed for chat_id `{chat_id}`: {source}"
                    ))
                })?;
            Ok(())
        })
    }

    fn clear<'a>(&'a self, chat_id: i64) -> SessionFuture<'a, ()> {
        Box::pin(async move {
            let query = format!("DELETE FROM {} WHERE chat_id = $1", self.table);
            self.client
                .execute(&query, &[&chat_id])
                .await
                .map_err(|source| {
                    invalid_request(format!(
                        "postgres clear failed for chat_id `{chat_id}`: {source}"
                    ))
                })?;
            Ok(())
        })
    }
}

#[cfg(feature = "postgres-session")]
fn validate_sql_identifier(identifier: &str) -> Result<()> {
    let mut chars = identifier.chars();
    let Some(first) = chars.next() else {
        return Err(invalid_request("sql identifier cannot be empty"));
    };

    if !(first.is_ascii_alphabetic() || first == '_') {
        return Err(invalid_request(format!(
            "sql identifier `{identifier}` must start with [A-Za-z_]"
        )));
    }

    if !chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_') {
        return Err(invalid_request(format!(
            "sql identifier `{identifier}` contains invalid characters"
        )));
    }

    Ok(())
}

/// Loads chat-scoped state from a store.
pub async fn load_chat_state<S, Store>(store: &Store, update: &Update) -> Result<Option<S>>
where
    S: Clone + Send + Sync + 'static,
    Store: SessionStore<S> + ?Sized,
{
    let Some(chat_id) = update_chat_id(update) else {
        return Err(invalid_request(
            "update does not contain a chat id for state operations",
        ));
    };

    store.load(chat_id).await
}

/// Saves chat-scoped state into a store.
pub async fn save_chat_state<S, Store>(store: &Store, update: &Update, state: S) -> Result<()>
where
    S: Clone + Send + Sync + 'static,
    Store: SessionStore<S> + ?Sized,
{
    let Some(chat_id) = update_chat_id(update) else {
        return Err(invalid_request(
            "update does not contain a chat id for state operations",
        ));
    };

    store.save(chat_id, state).await
}

/// Clears chat-scoped state from a store.
pub async fn clear_chat_state<S, Store>(store: &Store, update: &Update) -> Result<()>
where
    S: Clone + Send + Sync + 'static,
    Store: SessionStore<S> + ?Sized,
{
    let Some(chat_id) = update_chat_id(update) else {
        return Err(invalid_request(
            "update does not contain a chat id for state operations",
        ));
    };

    store.clear(chat_id).await
}

/// Applies an FSM transition to chat-scoped state.
pub async fn apply_chat_state_transition<S, Store>(
    store: &Store,
    update: &Update,
    transition: StateTransition<S>,
) -> Result<()>
where
    S: Clone + Send + Sync + 'static,
    Store: SessionStore<S> + ?Sized,
{
    match transition {
        StateTransition::Keep => Ok(()),
        StateTransition::Set(state) => save_chat_state(store, update, state).await,
        StateTransition::Clear => clear_chat_state::<S, Store>(store, update).await,
    }
}

/// High-level chat-scoped session manager for FSM-style bots.
pub struct ChatSession<S, Store>
where
    S: Clone + Send + Sync + 'static,
    Store: SessionStore<S>,
{
    store: Arc<Store>,
    _state: std::marker::PhantomData<S>,
}

impl<S, Store> Clone for ChatSession<S, Store>
where
    S: Clone + Send + Sync + 'static,
    Store: SessionStore<S>,
{
    fn clone(&self) -> Self {
        Self {
            store: Arc::clone(&self.store),
            _state: std::marker::PhantomData,
        }
    }
}

impl<S, Store> ChatSession<S, Store>
where
    S: Clone + Send + Sync + 'static,
    Store: SessionStore<S>,
{
    pub fn new(store: Store) -> Self {
        Self {
            store: Arc::new(store),
            _state: std::marker::PhantomData,
        }
    }

    pub fn from_shared(store: Arc<Store>) -> Self {
        Self {
            store,
            _state: std::marker::PhantomData,
        }
    }

    pub fn store(&self) -> &Store {
        self.store.as_ref()
    }

    pub fn shared_store(&self) -> Arc<Store> {
        Arc::clone(&self.store)
    }

    pub async fn load(&self, update: &Update) -> Result<Option<S>> {
        load_chat_state(self.store(), update).await
    }

    pub async fn save(&self, update: &Update, state: S) -> Result<()> {
        save_chat_state(self.store(), update, state).await
    }

    pub async fn clear(&self, update: &Update) -> Result<()> {
        clear_chat_state::<S, Store>(self.store(), update).await
    }

    pub async fn apply(&self, update: &Update, transition: StateTransition<S>) -> Result<()> {
        apply_chat_state_transition(self.store(), update, transition).await
    }

    /// Loads state, runs transition function, then applies resulting state transition.
    pub async fn transition<R, F, Fut>(&self, update: &Update, f: F) -> Result<R>
    where
        F: FnOnce(Option<S>) -> Fut + Send,
        Fut: Future<Output = (R, StateTransition<S>)> + Send,
    {
        let current = self.load(update).await?;
        let (output, transition) = f(current).await;
        self.apply(update, transition).await?;
        Ok(output)
    }
}

/// Long-polling source configuration.
#[derive(Clone, Debug)]
pub struct PollingConfig {
    pub poll_timeout_seconds: u16,
    pub limit: Option<u8>,
    pub allowed_updates: Option<Vec<String>>,
    pub disable_webhook_on_start: bool,
    pub drop_pending_updates_on_start: bool,
    pub dedupe_window_size: usize,
    pub persist_offset_path: Option<PathBuf>,
}

impl Default for PollingConfig {
    fn default() -> Self {
        Self {
            poll_timeout_seconds: 30,
            limit: None,
            allowed_updates: None,
            disable_webhook_on_start: true,
            drop_pending_updates_on_start: false,
            dedupe_window_size: 2048,
            persist_offset_path: None,
        }
    }
}

/// Result of dispatching one update through router + middleware chain.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DispatchOutcome {
    Handled { update_id: i64 },
    Ignored { update_id: i64 },
}

impl DispatchOutcome {
    pub fn update_id(self) -> i64 {
        match self {
            Self::Handled { update_id } | Self::Ignored { update_id } => update_id,
        }
    }

    pub fn is_handled(self) -> bool {
        matches!(self, Self::Handled { .. })
    }
}

/// Pluggable update input source used by `BotEngine`.
pub trait UpdateSource: Send + 'static {
    fn poll<'a>(&'a mut self) -> SourceFuture<'a>;
}

/// Shared engine configuration independent from input source implementation.
#[derive(Clone, Debug)]
pub struct EngineConfig {
    pub idle_delay: Duration,
    pub error_delay: Duration,
    pub continue_on_source_error: bool,
    pub continue_on_handler_error: bool,
    pub max_handler_concurrency: usize,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            idle_delay: Duration::from_millis(100),
            error_delay: Duration::from_millis(500),
            continue_on_source_error: true,
            continue_on_handler_error: true,
            max_handler_concurrency: 1,
        }
    }
}

/// Long-polling update source that only fetches updates and tracks offsets.
#[derive(Clone)]
pub struct LongPollingSource {
    client: Client,
    config: PollingConfig,
    next_offset: Option<i64>,
    seen_update_ids: HashSet<i64>,
    seen_update_order: VecDeque<i64>,
    offset_loaded: bool,
    prepared: bool,
}

impl LongPollingSource {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            config: PollingConfig::default(),
            next_offset: None,
            seen_update_ids: HashSet::new(),
            seen_update_order: VecDeque::new(),
            offset_loaded: false,
            prepared: false,
        }
    }

    pub fn with_config(mut self, config: PollingConfig) -> Self {
        self.config = config;
        self
    }

    pub fn config_mut(&mut self) -> &mut PollingConfig {
        &mut self.config
    }

    pub fn next_offset(&self) -> Option<i64> {
        self.next_offset
    }

    pub fn set_next_offset(&mut self, offset: Option<i64>) -> &mut Self {
        self.next_offset = offset;
        self
    }

    pub fn with_offset_persistence_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.config.persist_offset_path = Some(path.into());
        self
    }

    pub fn clear_offset_persistence_path(mut self) -> Self {
        self.config.persist_offset_path = None;
        self
    }

    pub fn set_prepared(&mut self, prepared: bool) -> &mut Self {
        self.prepared = prepared;
        self
    }

    async fn ensure_prepared(&mut self) -> Result<()> {
        self.ensure_offset_loaded()?;

        if self.prepared {
            return Ok(());
        }

        if self.config.disable_webhook_on_start {
            let request = DeleteWebhookRequest {
                drop_pending_updates: self.config.drop_pending_updates_on_start.then_some(true),
            };
            self.client.updates().delete_webhook(&request).await?;
        }

        self.prepared = true;
        Ok(())
    }

    fn advance_next_offset(&mut self, update_id: i64) -> bool {
        let candidate = update_id.saturating_add(1);
        let next = Some(
            self.next_offset
                .map_or(candidate, |current| current.max(candidate)),
        );
        let changed = next != self.next_offset;
        self.next_offset = next;
        changed
    }

    fn ensure_offset_loaded(&mut self) -> Result<()> {
        if self.offset_loaded {
            return Ok(());
        }

        if self.next_offset.is_none()
            && let Some(path) = self.config.persist_offset_path.as_deref()
        {
            self.next_offset = load_persisted_polling_offset(path)?;
        }

        self.offset_loaded = true;
        Ok(())
    }

    fn persist_offset_if_configured(&self) -> Result<()> {
        let Some(path) = self.config.persist_offset_path.as_deref() else {
            return Ok(());
        };
        persist_polling_offset(path, self.next_offset)
    }

    fn is_duplicate_update(&self, update_id: i64) -> bool {
        if self.config.dedupe_window_size == 0 {
            return false;
        }
        self.seen_update_ids.contains(&update_id)
    }

    fn remember_update(&mut self, update_id: i64) {
        if self.config.dedupe_window_size == 0 {
            return;
        }

        if !self.seen_update_ids.insert(update_id) {
            return;
        }

        self.seen_update_order.push_back(update_id);
        while self.seen_update_order.len() > self.config.dedupe_window_size {
            if let Some(oldest) = self.seen_update_order.pop_front() {
                self.seen_update_ids.remove(&oldest);
            }
        }
    }
}

impl UpdateSource for LongPollingSource {
    fn poll<'a>(&'a mut self) -> SourceFuture<'a> {
        Box::pin(async move {
            self.ensure_prepared().await?;

            let mut request = GetUpdatesRequest::with_timeout(self.config.poll_timeout_seconds);
            request.offset = self.next_offset;
            request.limit = self.config.limit;
            request.allowed_updates = self.config.allowed_updates.clone();

            let updates = self.client.updates().get_updates(&request).await?;
            let mut offset_changed = false;
            for update in &updates {
                offset_changed |= self.advance_next_offset(update.update_id);
            }
            if offset_changed {
                self.persist_offset_if_configured()?;
            }

            let mut deduped = Vec::with_capacity(updates.len());
            for update in updates {
                if self.is_duplicate_update(update.update_id) {
                    continue;
                }
                self.remember_update(update.update_id);
                deduped.push(update);
            }

            Ok(deduped)
        })
    }
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
struct PollingOffsetSnapshot {
    #[serde(default = "default_polling_offset_snapshot_version")]
    version: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    next_offset: Option<i64>,
}

fn default_polling_offset_snapshot_version() -> u8 {
    1
}

fn load_persisted_polling_offset(path: &Path) -> Result<Option<i64>> {
    if !path.exists() {
        return Ok(None);
    }

    let raw = fs::read(path).map_err(|source| Error::ReadLocalFile {
        path: path.display().to_string(),
        source,
    })?;
    if raw.is_empty() {
        return Ok(None);
    }

    let snapshot: PollingOffsetSnapshot = serde_json::from_slice(&raw).map_err(|source| {
        invalid_request(format!(
            "failed to deserialize polling offset snapshot `{}`: {source}",
            path.display()
        ))
    })?;
    Ok(snapshot.next_offset)
}

fn persist_polling_offset(path: &Path, next_offset: Option<i64>) -> Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|source| {
        invalid_request(format!(
            "failed to create polling offset directory `{}`: {source}",
            parent.display()
        ))
    })?;

    let snapshot = PollingOffsetSnapshot {
        version: default_polling_offset_snapshot_version(),
        next_offset,
    };
    let encoded =
        serde_json::to_vec(&snapshot).map_err(|source| Error::SerializeRequest { source })?;
    fs::write(path, encoded).map_err(|source| {
        invalid_request(format!(
            "failed to write polling offset snapshot `{}`: {source}",
            path.display()
        ))
    })?;
    Ok(())
}

/// Sink side of a channel-backed update source.
#[derive(Clone)]
pub struct UpdateSink {
    sender: mpsc::Sender<Update>,
}

impl UpdateSink {
    pub fn new(sender: mpsc::Sender<Update>) -> Self {
        Self { sender }
    }

    pub async fn send(&self, update: Update) -> Result<()> {
        self.sender
            .send(update)
            .await
            .map_err(|_| invalid_request("update sink channel is closed"))?;
        Ok(())
    }
}

/// Source side of a channel-backed update source.
pub struct ChannelUpdateSource {
    receiver: mpsc::Receiver<Update>,
    max_batch: usize,
}

impl ChannelUpdateSource {
    pub fn new(receiver: mpsc::Receiver<Update>) -> Self {
        Self {
            receiver,
            max_batch: 32,
        }
    }

    pub fn with_max_batch(mut self, max_batch: usize) -> Self {
        self.max_batch = max_batch.max(1);
        self
    }
}

impl UpdateSource for ChannelUpdateSource {
    fn poll<'a>(&'a mut self) -> SourceFuture<'a> {
        Box::pin(async move {
            let Some(first) = self.receiver.recv().await else {
                return Err(invalid_request("update source channel is closed"));
            };

            let mut updates = Vec::with_capacity(self.max_batch.max(1));
            updates.push(first);

            while updates.len() < self.max_batch {
                match self.receiver.try_recv() {
                    Ok(update) => updates.push(update),
                    Err(mpsc::error::TryRecvError::Empty) => break,
                    Err(mpsc::error::TryRecvError::Disconnected) => break,
                }
            }

            Ok(updates)
        })
    }
}

/// Creates a webhook-friendly channel source pair.
pub fn channel_source(buffer: usize) -> (UpdateSink, ChannelUpdateSource) {
    let (sender, receiver) = mpsc::channel(buffer.max(1));
    (UpdateSink::new(sender), ChannelUpdateSource::new(receiver))
}

/// Reliable send-side outbox configuration.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct OutboxConfig {
    pub queue_capacity: usize,
    pub max_attempts: usize,
    pub base_backoff: Duration,
    pub max_backoff: Duration,
    pub dedupe_ttl: Duration,
    pub persistence_path: Option<PathBuf>,
    pub dead_letter_path: Option<PathBuf>,
    pub max_dead_letters: usize,
    pub max_message_age: Option<Duration>,
}

impl Default for OutboxConfig {
    fn default() -> Self {
        Self {
            queue_capacity: 256,
            max_attempts: 4,
            base_backoff: Duration::from_millis(200),
            max_backoff: Duration::from_secs(5),
            dedupe_ttl: Duration::from_secs(120),
            persistence_path: None,
            dead_letter_path: None,
            max_dead_letters: 1024,
            max_message_age: None,
        }
    }
}

impl OutboxConfig {
    pub fn with_persistence_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.persistence_path = Some(path.into());
        self
    }

    pub fn without_persistence(mut self) -> Self {
        self.persistence_path = None;
        self
    }

    pub fn with_dead_letter_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.dead_letter_path = Some(path.into());
        self
    }

    pub fn with_max_message_age(mut self, max_age: Option<Duration>) -> Self {
        self.max_message_age = max_age;
        self
    }
}

struct OutboxCommand {
    chat_id: ChatId,
    text: String,
    idempotency_key: Option<String>,
    responder: oneshot::Sender<Result<Message>>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct PersistedOutboxCommand {
    chat_id: ChatId,
    text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    idempotency_key: Option<String>,
    #[serde(default = "unix_timestamp_millis_now")]
    enqueued_at_unix_ms: i64,
    #[serde(default)]
    attempt: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    last_error: Option<String>,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
struct OutboxSnapshot {
    #[serde(default = "default_outbox_snapshot_version")]
    version: u8,
    #[serde(default)]
    queue: Vec<PersistedOutboxCommand>,
}

fn default_outbox_snapshot_version() -> u8 {
    1
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct DeadLetterEntry {
    chat_id: ChatId,
    text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    idempotency_key: Option<String>,
    attempts: usize,
    reason: String,
    enqueued_at_unix_ms: i64,
    failed_at_unix_ms: i64,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
struct DeadLetterSnapshot {
    #[serde(default = "default_dead_letter_snapshot_version")]
    version: u8,
    #[serde(default)]
    entries: Vec<DeadLetterEntry>,
}

fn default_dead_letter_snapshot_version() -> u8 {
    1
}

struct QueuedOutboxCommand {
    payload: PersistedOutboxCommand,
    responder: Option<oneshot::Sender<Result<Message>>>,
}

/// Asynchronous outbox handle for reliable message delivery.
#[derive(Clone)]
pub struct BotOutbox {
    sender: mpsc::Sender<OutboxCommand>,
}

impl BotOutbox {
    pub fn spawn(client: Client, config: OutboxConfig) -> Self {
        let (sender, receiver) = mpsc::channel(config.queue_capacity.max(1));
        tokio::spawn(run_outbox_worker(client, config, receiver));
        Self { sender }
    }

    pub async fn send_text(
        &self,
        chat_id: impl Into<ChatId>,
        text: impl Into<String>,
    ) -> Result<Message> {
        self.send_text_with_key(chat_id, text, None).await
    }

    pub async fn send_text_with_key(
        &self,
        chat_id: impl Into<ChatId>,
        text: impl Into<String>,
        idempotency_key: Option<String>,
    ) -> Result<Message> {
        let (responder, receiver) = oneshot::channel();
        let command = OutboxCommand {
            chat_id: chat_id.into(),
            text: text.into(),
            idempotency_key,
            responder,
        };

        self.sender
            .send(command)
            .await
            .map_err(|_| invalid_request("outbox worker is closed"))?;

        receiver
            .await
            .map_err(|_| invalid_request("outbox worker dropped response"))?
    }
}

async fn run_outbox_worker(
    client: Client,
    config: OutboxConfig,
    mut receiver: mpsc::Receiver<OutboxCommand>,
) {
    let mut dedupe: HashMap<String, (Message, Instant)> = HashMap::new();
    let mut queue = load_outbox_queue(config.persistence_path.as_deref())
        .unwrap_or_default()
        .into_iter()
        .map(|payload| QueuedOutboxCommand {
            payload,
            responder: None,
        })
        .collect::<VecDeque<_>>();

    let _ = persist_outbox_queue(config.persistence_path.as_deref(), &queue);

    loop {
        while let Ok(command) = receiver.try_recv() {
            queue.push_back(QueuedOutboxCommand {
                payload: PersistedOutboxCommand {
                    chat_id: command.chat_id,
                    text: command.text,
                    idempotency_key: command.idempotency_key,
                    enqueued_at_unix_ms: unix_timestamp_millis_now(),
                    attempt: 0,
                    last_error: None,
                },
                responder: Some(command.responder),
            });
        }

        if queue.is_empty() {
            let Some(command) = receiver.recv().await else {
                break;
            };

            queue.push_back(QueuedOutboxCommand {
                payload: PersistedOutboxCommand {
                    chat_id: command.chat_id,
                    text: command.text,
                    idempotency_key: command.idempotency_key,
                    enqueued_at_unix_ms: unix_timestamp_millis_now(),
                    attempt: 0,
                    last_error: None,
                },
                responder: Some(command.responder),
            });
        }

        if let Err(error) = persist_outbox_queue(config.persistence_path.as_deref(), &queue) {
            if let Some(entry) = queue.pop_front()
                && let Some(responder) = entry.responder
            {
                let _ = responder.send(Err(error));
            }
            continue;
        }

        let Some(front_payload) = queue.front().map(|entry| entry.payload.clone()) else {
            continue;
        };

        if is_outbox_message_expired(
            front_payload.enqueued_at_unix_ms,
            config.max_message_age,
            unix_timestamp_millis_now(),
        ) {
            let entry = queue.pop_front();
            let _ = persist_outbox_queue(config.persistence_path.as_deref(), &queue);
            if let Some(entry) = entry {
                let dead_letter = to_dead_letter(
                    &entry.payload,
                    "message expired in outbox before delivery".to_owned(),
                );
                let _ = append_dead_letter(
                    config.dead_letter_path.as_deref(),
                    config.max_dead_letters,
                    dead_letter,
                );
                if let Some(responder) = entry.responder {
                    let _ = responder.send(Err(invalid_request("message expired in outbox queue")));
                }
            }
            continue;
        }

        prune_dedupe_cache(&mut dedupe);

        if let Some(key) = front_payload.idempotency_key.as_deref()
            && let Some((cached, expires_at)) = dedupe.get(key)
            && *expires_at > Instant::now()
        {
            let entry = queue.pop_front();
            let _ = persist_outbox_queue(config.persistence_path.as_deref(), &queue);
            if let Some(entry) = entry
                && let Some(responder) = entry.responder
            {
                let _ = responder.send(Ok(cached.clone()));
            }
            continue;
        }

        let send_result = send_once(&client, &front_payload.chat_id, &front_payload.text).await;
        match send_result {
            Ok(message) => {
                if let Some(key) = front_payload.idempotency_key.clone() {
                    let expires_at = Instant::now() + config.dedupe_ttl;
                    dedupe.insert(key, (message.clone(), expires_at));
                }

                let entry = queue.pop_front();
                let _ = persist_outbox_queue(config.persistence_path.as_deref(), &queue);
                if let Some(entry) = entry
                    && let Some(responder) = entry.responder
                {
                    let _ = responder.send(Ok(message));
                }
            }
            Err(error) => {
                let max_attempts = config.max_attempts.max(1);
                let error_message = error.to_string();
                let attempt = if let Some(front) = queue.front_mut() {
                    front.payload.attempt = front.payload.attempt.saturating_add(1);
                    front.payload.last_error = Some(error_message.clone());
                    front.payload.attempt
                } else {
                    1
                };
                let should_retry = error.is_retryable() && attempt < max_attempts;
                if should_retry {
                    let delay = error.retry_after().unwrap_or_else(|| {
                        exponential_backoff(config.base_backoff, config.max_backoff, attempt)
                    });
                    let _ = persist_outbox_queue(config.persistence_path.as_deref(), &queue);
                    sleep(delay.min(config.max_backoff)).await;
                    continue;
                }

                let entry = queue.pop_front();
                let _ = persist_outbox_queue(config.persistence_path.as_deref(), &queue);
                if let Some(entry) = entry {
                    let dead_letter = to_dead_letter(&entry.payload, error_message);
                    let _ = append_dead_letter(
                        config.dead_letter_path.as_deref(),
                        config.max_dead_letters,
                        dead_letter,
                    );
                    if let Some(responder) = entry.responder {
                        let _ = responder.send(Err(error));
                    }
                }
            }
        }
    }
}

fn prune_dedupe_cache(dedupe: &mut HashMap<String, (Message, Instant)>) {
    let now = Instant::now();
    dedupe.retain(|_, (_message, expires_at)| *expires_at > now);
}

fn unix_timestamp_millis_now() -> i64 {
    let Ok(duration) = SystemTime::now().duration_since(UNIX_EPOCH) else {
        return 0;
    };
    i64::try_from(duration.as_millis()).unwrap_or(i64::MAX)
}

fn is_outbox_message_expired(
    enqueued_at_unix_ms: i64,
    max_message_age: Option<Duration>,
    now_unix_ms: i64,
) -> bool {
    let Some(max_message_age) = max_message_age else {
        return false;
    };

    let max_age_ms = i64::try_from(max_message_age.as_millis()).unwrap_or(i64::MAX);
    let elapsed = now_unix_ms.saturating_sub(enqueued_at_unix_ms);
    elapsed >= max_age_ms
}

fn to_dead_letter(payload: &PersistedOutboxCommand, reason: String) -> DeadLetterEntry {
    DeadLetterEntry {
        chat_id: payload.chat_id.clone(),
        text: payload.text.clone(),
        idempotency_key: payload.idempotency_key.clone(),
        attempts: payload.attempt,
        reason,
        enqueued_at_unix_ms: payload.enqueued_at_unix_ms,
        failed_at_unix_ms: unix_timestamp_millis_now(),
    }
}

fn append_dead_letter(
    path: Option<&Path>,
    max_dead_letters: usize,
    entry: DeadLetterEntry,
) -> Result<()> {
    let Some(path) = path else {
        return Ok(());
    };

    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|source| {
        invalid_request(format!(
            "failed to create dead-letter directory `{}`: {source}",
            parent.display()
        ))
    })?;

    let mut snapshot = load_dead_letter_snapshot(path)?;
    snapshot.entries.push(entry);
    let max_dead_letters = max_dead_letters.max(1);
    if snapshot.entries.len() > max_dead_letters {
        let overflow = snapshot.entries.len().saturating_sub(max_dead_letters);
        snapshot.entries.drain(0..overflow);
    }

    let encoded =
        serde_json::to_vec(&snapshot).map_err(|source| Error::SerializeRequest { source })?;
    fs::write(path, encoded).map_err(|source| {
        invalid_request(format!(
            "failed to write dead-letter snapshot `{}`: {source}",
            path.display()
        ))
    })?;
    Ok(())
}

fn load_dead_letter_snapshot(path: &Path) -> Result<DeadLetterSnapshot> {
    if !path.exists() {
        return Ok(DeadLetterSnapshot {
            version: default_dead_letter_snapshot_version(),
            entries: Vec::new(),
        });
    }

    let raw = fs::read(path).map_err(|source| Error::ReadLocalFile {
        path: path.display().to_string(),
        source,
    })?;
    if raw.is_empty() {
        return Ok(DeadLetterSnapshot {
            version: default_dead_letter_snapshot_version(),
            entries: Vec::new(),
        });
    }

    serde_json::from_slice(&raw).map_err(|source| {
        invalid_request(format!(
            "failed to deserialize dead-letter snapshot `{}`: {source}",
            path.display()
        ))
    })
}

fn load_outbox_queue(path: Option<&Path>) -> Result<Vec<PersistedOutboxCommand>> {
    let Some(path) = path else {
        return Ok(Vec::new());
    };

    if !path.exists() {
        return Ok(Vec::new());
    }

    let raw = fs::read(path).map_err(|source| Error::ReadLocalFile {
        path: path.display().to_string(),
        source,
    })?;
    if raw.is_empty() {
        return Ok(Vec::new());
    }

    let snapshot: OutboxSnapshot = serde_json::from_slice(&raw).map_err(|source| {
        invalid_request(format!(
            "failed to deserialize outbox snapshot `{}`: {source}",
            path.display()
        ))
    })?;
    Ok(snapshot.queue)
}

fn persist_outbox_queue(path: Option<&Path>, queue: &VecDeque<QueuedOutboxCommand>) -> Result<()> {
    let Some(path) = path else {
        return Ok(());
    };

    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|source| {
        invalid_request(format!(
            "failed to create outbox directory `{}`: {source}",
            parent.display()
        ))
    })?;

    let snapshot = OutboxSnapshot {
        version: default_outbox_snapshot_version(),
        queue: queue.iter().map(|entry| entry.payload.clone()).collect(),
    };
    let encoded =
        serde_json::to_vec(&snapshot).map_err(|source| Error::SerializeRequest { source })?;

    fs::write(path, encoded).map_err(|source| {
        invalid_request(format!(
            "failed to persist outbox snapshot `{}`: {source}",
            path.display()
        ))
    })?;
    Ok(())
}

async fn send_once(client: &Client, chat_id: &ChatId, text: &str) -> Result<Message> {
    let request = SendMessageRequest::new(chat_id.clone(), text.to_owned())?;
    client.messages().send_message(&request).await
}

fn exponential_backoff(base: Duration, max: Duration, attempt: usize) -> Duration {
    let exponent = attempt.saturating_sub(1).min(16);
    let factor = 2u32.saturating_pow(exponent as u32);
    let delay = base.saturating_mul(factor);
    delay.min(max)
}

/// Source-agnostic bot engine that handles dispatching, backpressure and error policy.
pub struct BotEngine<S>
where
    S: UpdateSource,
{
    client: Client,
    source: S,
    router: Router,
    config: EngineConfig,
    on_source_error: Option<SourceErrorHook>,
    on_handler_error: Option<HandlerErrorHook>,
    on_event: Option<EngineEventHook>,
}

impl<S> BotEngine<S>
where
    S: UpdateSource,
{
    pub fn new(client: Client, source: S, router: Router) -> Self {
        Self {
            client,
            source,
            router,
            config: EngineConfig::default(),
            on_source_error: None,
            on_handler_error: None,
            on_event: None,
        }
    }

    pub fn with_config(mut self, config: EngineConfig) -> Self {
        self.config = config;
        self
    }

    pub fn config_mut(&mut self) -> &mut EngineConfig {
        &mut self.config
    }

    pub fn source_mut(&mut self) -> &mut S {
        &mut self.source
    }

    pub fn on_source_error<F>(mut self, hook: F) -> Self
    where
        F: Fn(&Error) + Send + Sync + 'static,
    {
        self.on_source_error = Some(Arc::new(hook));
        self
    }

    pub fn on_handler_error<F>(mut self, hook: F) -> Self
    where
        F: Fn(i64, &Error) + Send + Sync + 'static,
    {
        self.on_handler_error = Some(Arc::new(hook));
        self
    }

    pub fn on_event<F>(mut self, hook: F) -> Self
    where
        F: Fn(&EngineEvent) + Send + Sync + 'static,
    {
        self.on_event = Some(Arc::new(hook));
        self
    }

    pub async fn poll_once(&mut self) -> Result<Vec<DispatchOutcome>> {
        self.notify_event(EngineEvent::PollStarted);

        let updates = match self.source.poll().await {
            Ok(updates) => updates,
            Err(error) => {
                self.notify_event(EngineEvent::PollFailed {
                    classification: error.classification(),
                    retryable: error.is_retryable(),
                });
                return Err(error);
            }
        };

        self.notify_event(EngineEvent::PollCompleted {
            update_count: updates.len(),
        });

        self.dispatch_updates(updates).await
    }

    pub async fn run(&mut self) -> Result<()> {
        loop {
            let poll_result = self.poll_once().await;
            let delay = self.handle_poll_result(poll_result)?;
            wait_if_needed(delay).await;
        }
    }

    pub async fn run_until<F>(&mut self, shutdown: F) -> Result<()>
    where
        F: Future<Output = ()> + Send,
    {
        tokio::pin!(shutdown);

        loop {
            tokio::select! {
                _ = &mut shutdown => return Ok(()),
                poll_result = self.poll_once() => {
                    let delay = self.handle_poll_result(poll_result)?;
                    if !delay.is_zero() {
                        tokio::select! {
                            _ = &mut shutdown => return Ok(()),
                            _ = sleep(delay) => {}
                        }
                    }
                }
            }
        }
    }

    async fn dispatch_updates(&self, updates: Vec<Update>) -> Result<Vec<DispatchOutcome>> {
        if self.config.max_handler_concurrency <= 1 {
            return self.dispatch_updates_sequential(updates).await;
        }
        self.dispatch_updates_concurrent(updates).await
    }

    async fn dispatch_updates_sequential(
        &self,
        updates: Vec<Update>,
    ) -> Result<Vec<DispatchOutcome>> {
        let mut outcomes = Vec::with_capacity(updates.len());

        for update in updates {
            let update_id = update.update_id;
            let context = BotContext::new(self.client.clone());
            self.notify_event(EngineEvent::DispatchStarted { update_id });
            match self.router.dispatch(context, update).await {
                Ok(true) => {
                    let outcome = DispatchOutcome::Handled { update_id };
                    self.notify_event(EngineEvent::DispatchCompleted { outcome });
                    outcomes.push(outcome);
                }
                Ok(false) => {
                    let outcome = DispatchOutcome::Ignored { update_id };
                    self.notify_event(EngineEvent::DispatchCompleted { outcome });
                    outcomes.push(outcome);
                }
                Err(error) => {
                    self.notify_handler_error(update_id, &error);
                    self.notify_event(EngineEvent::DispatchFailed {
                        update_id,
                        classification: error.classification(),
                    });
                    if !self.config.continue_on_handler_error {
                        return Err(error);
                    }
                    let outcome = DispatchOutcome::Ignored { update_id };
                    self.notify_event(EngineEvent::DispatchCompleted { outcome });
                    outcomes.push(outcome);
                }
            }
        }

        Ok(outcomes)
    }

    async fn dispatch_updates_concurrent(
        &self,
        updates: Vec<Update>,
    ) -> Result<Vec<DispatchOutcome>> {
        let max_concurrency = self.config.max_handler_concurrency.max(1);
        let semaphore = Arc::new(Semaphore::new(max_concurrency));
        let mut join_set = JoinSet::new();

        for update in updates {
            let update_id = update.update_id;
            self.notify_event(EngineEvent::DispatchStarted { update_id });

            let permit = semaphore
                .clone()
                .acquire_owned()
                .await
                .map_err(|_| invalid_request("handler semaphore closed unexpectedly"))?;

            let router = self.router.clone();
            let context = BotContext::new(self.client.clone());
            join_set.spawn(async move {
                let _permit = permit;
                let result = router.dispatch(context, update).await;
                (update_id, result)
            });
        }

        let mut outcomes = Vec::new();
        let mut first_error: Option<Error> = None;

        while let Some(join_result) = join_set.join_next().await {
            match join_result {
                Ok((update_id, Ok(true))) => {
                    let outcome = DispatchOutcome::Handled { update_id };
                    self.notify_event(EngineEvent::DispatchCompleted { outcome });
                    outcomes.push(outcome);
                }
                Ok((update_id, Ok(false))) => {
                    let outcome = DispatchOutcome::Ignored { update_id };
                    self.notify_event(EngineEvent::DispatchCompleted { outcome });
                    outcomes.push(outcome);
                }
                Ok((update_id, Err(error))) => {
                    self.notify_handler_error(update_id, &error);
                    self.notify_event(EngineEvent::DispatchFailed {
                        update_id,
                        classification: error.classification(),
                    });
                    if !self.config.continue_on_handler_error {
                        first_error = Some(error);
                        break;
                    }
                    let outcome = DispatchOutcome::Ignored { update_id };
                    self.notify_event(EngineEvent::DispatchCompleted { outcome });
                    outcomes.push(outcome);
                }
                Err(join_error) => {
                    let error = invalid_request(format!("bot handler task failed: {join_error}"));
                    self.notify_handler_error(-1, &error);
                    self.notify_event(EngineEvent::DispatchFailed {
                        update_id: -1,
                        classification: error.classification(),
                    });
                    if !self.config.continue_on_handler_error {
                        first_error = Some(error);
                        break;
                    }
                }
            }
        }

        if let Some(error) = first_error {
            join_set.abort_all();
            while join_set.join_next().await.is_some() {}
            return Err(error);
        }

        Ok(outcomes)
    }

    fn handle_poll_result(&self, poll_result: Result<Vec<DispatchOutcome>>) -> Result<Duration> {
        match poll_result {
            Ok(outcomes) if outcomes.is_empty() => Ok(self.config.idle_delay),
            Ok(_) => Ok(Duration::ZERO),
            Err(error) => {
                self.notify_source_error(&error);
                if !self.config.continue_on_source_error {
                    return Err(error);
                }
                Ok(self.config.error_delay)
            }
        }
    }

    fn notify_source_error(&self, error: &Error) {
        if let Some(hook) = self.on_source_error.as_ref() {
            hook(error);
        }
    }

    fn notify_handler_error(&self, update_id: i64, error: &Error) {
        if let Some(hook) = self.on_handler_error.as_ref() {
            hook(update_id, error);
        }
    }

    fn notify_event(&self, event: EngineEvent) {
        if let Some(hook) = self.on_event.as_ref() {
            hook(&event);
        }
    }
}

impl BotEngine<LongPollingSource> {
    /// Builds engine with default long polling source.
    pub fn with_long_polling(client: Client, router: Router) -> Self {
        let source = LongPollingSource::new(client.clone());
        Self::new(client, source, router)
    }
}

impl BotEngine<ChannelUpdateSource> {
    /// Builds engine backed by channel source and returns paired sink.
    pub fn with_channel(client: Client, router: Router, buffer: usize) -> (UpdateSink, Self) {
        let (sink, source) = channel_source(buffer);
        let engine = Self::new(client, source, router);
        (sink, engine)
    }
}

/// High-level app wrapper that keeps bot runtime setup short for downstream projects.
pub struct BotApp<S>
where
    S: UpdateSource,
{
    engine: BotEngine<S>,
}

impl BotApp<LongPollingSource> {
    pub fn long_polling(client: Client, router: Router) -> Self {
        Self {
            engine: BotEngine::with_long_polling(client, router),
        }
    }
}

impl<S> BotApp<S>
where
    S: UpdateSource,
{
    pub fn from_engine(engine: BotEngine<S>) -> Self {
        Self { engine }
    }

    pub fn engine(&self) -> &BotEngine<S> {
        &self.engine
    }

    pub fn engine_mut(&mut self) -> &mut BotEngine<S> {
        &mut self.engine
    }

    pub fn with_engine_config(mut self, config: EngineConfig) -> Self {
        self.engine = self.engine.with_config(config);
        self
    }

    pub fn on_source_error<F>(mut self, hook: F) -> Self
    where
        F: Fn(&Error) + Send + Sync + 'static,
    {
        self.engine = self.engine.on_source_error(hook);
        self
    }

    pub fn on_handler_error<F>(mut self, hook: F) -> Self
    where
        F: Fn(i64, &Error) + Send + Sync + 'static,
    {
        self.engine = self.engine.on_handler_error(hook);
        self
    }

    pub fn on_event<F>(mut self, hook: F) -> Self
    where
        F: Fn(&EngineEvent) + Send + Sync + 'static,
    {
        self.engine = self.engine.on_event(hook);
        self
    }

    pub async fn poll_once(&mut self) -> Result<Vec<DispatchOutcome>> {
        self.engine.poll_once().await
    }

    pub async fn run(&mut self) -> Result<()> {
        self.engine.run().await
    }

    pub async fn run_until<F>(&mut self, shutdown: F) -> Result<()>
    where
        F: Future<Output = ()> + Send,
    {
        self.engine.run_until(shutdown).await
    }

    pub fn into_engine(self) -> BotEngine<S> {
        self.engine
    }
}

/// Webhook-side runtime configuration.
#[derive(Clone, Debug, Default)]
pub struct WebhookConfig {
    pub expected_secret_token: Option<String>,
    pub continue_on_handler_error: bool,
}

/// Webhook dispatcher that can be embedded into any HTTP framework.
pub struct WebhookRunner {
    client: Client,
    router: Router,
    config: WebhookConfig,
    on_handler_error: Option<HandlerErrorHook>,
}

impl WebhookRunner {
    pub fn new(client: Client, router: Router) -> Self {
        Self {
            client,
            router,
            config: WebhookConfig::default(),
            on_handler_error: None,
        }
    }

    pub fn with_config(mut self, config: WebhookConfig) -> Self {
        self.config = config;
        self
    }

    pub fn expected_secret_token(mut self, secret_token: impl Into<String>) -> Self {
        self.config.expected_secret_token = Some(secret_token.into());
        self
    }

    pub fn continue_on_handler_error(mut self, enabled: bool) -> Self {
        self.config.continue_on_handler_error = enabled;
        self
    }

    pub fn on_handler_error<F>(mut self, hook: F) -> Self
    where
        F: Fn(i64, &Error) + Send + Sync + 'static,
    {
        self.on_handler_error = Some(Arc::new(hook));
        self
    }

    pub fn verify_secret_token(&self, incoming_secret: Option<&str>) -> bool {
        match self.config.expected_secret_token.as_deref() {
            None => true,
            Some(expected) => incoming_secret.is_some_and(|incoming| incoming == expected),
        }
    }

    /// Calls Telegram `setWebhook` with runner's secret token config.
    pub async fn configure_webhook(
        &self,
        url: impl Into<String>,
        drop_pending_updates: bool,
    ) -> Result<bool> {
        let mut request = SetWebhookRequest::new(url);
        request.secret_token = self.config.expected_secret_token.clone();
        request.drop_pending_updates = drop_pending_updates.then_some(true);
        self.client.updates().set_webhook(&request).await
    }

    /// Calls Telegram `deleteWebhook`.
    pub async fn delete_webhook(&self, drop_pending_updates: bool) -> Result<bool> {
        let request = DeleteWebhookRequest {
            drop_pending_updates: drop_pending_updates.then_some(true),
        };
        self.client.updates().delete_webhook(&request).await
    }

    /// Dispatches one already-deserialized update and returns structured outcome.
    pub async fn dispatch_update_outcome(&self, update: Update) -> Result<DispatchOutcome> {
        let update_id = update.update_id;
        let context = BotContext::new(self.client.clone());
        let result = self.router.dispatch(context, update).await;

        match result {
            Ok(true) => Ok(DispatchOutcome::Handled { update_id }),
            Ok(false) => Ok(DispatchOutcome::Ignored { update_id }),
            Err(error) => {
                self.notify_handler_error(update_id, &error);
                if self.config.continue_on_handler_error {
                    return Ok(DispatchOutcome::Ignored { update_id });
                }
                Err(error)
            }
        }
    }

    /// Verifies secret token and parses JSON into an `Update` without dispatching.
    pub fn parse_update_json(
        &self,
        payload: &[u8],
        incoming_secret: Option<&str>,
    ) -> Result<Update> {
        if !self.verify_secret_token(incoming_secret) {
            return Err(invalid_request("invalid webhook secret token"));
        }

        serde_json::from_slice(payload).map_err(|source| {
            invalid_request(format!(
                "failed to deserialize webhook update payload: {source}"
            ))
        })
    }

    /// Verifies secret token, parses JSON update and dispatches it with structured outcome.
    pub async fn dispatch_json_outcome(
        &self,
        payload: &[u8],
        incoming_secret: Option<&str>,
    ) -> Result<DispatchOutcome> {
        let update = self.parse_update_json(payload, incoming_secret)?;

        self.dispatch_update_outcome(update).await
    }

    fn notify_handler_error(&self, update_id: i64, error: &Error) {
        if let Some(hook) = self.on_handler_error.as_ref() {
            hook(update_id, error);
        }
    }
}

/// Bot testing helpers for update fixtures and in-memory dispatch.
pub mod testing {
    use serde_json::json;

    use super::*;

    /// Builds a synthetic message update.
    pub fn message_update(update_id: i64, chat_id: i64, text: &str) -> Result<Update> {
        serde_json::from_value(json!({
            "update_id": update_id,
            "message": {
                "message_id": update_id,
                "date": 1700000000 + update_id,
                "chat": {"id": chat_id, "type": "private"},
                "text": text
            }
        }))
        .map_err(|source| {
            invalid_request(format!("failed to build message update fixture: {source}"))
        })
    }

    /// Builds a synthetic callback update.
    pub fn callback_update(update_id: i64, chat_id: i64, data: &str) -> Result<Update> {
        serde_json::from_value(json!({
            "update_id": update_id,
            "callback_query": {
                "id": format!("cb-{update_id}"),
                "from": {
                    "id": 123,
                    "is_bot": false,
                    "first_name": "tester"
                },
                "message": {
                    "message_id": update_id,
                    "date": 1700000000 + update_id,
                    "chat": {"id": chat_id, "type": "private"},
                    "text": "button clicked"
                },
                "data": data
            }
        }))
        .map_err(|source| {
            invalid_request(format!("failed to build callback update fixture: {source}"))
        })
    }

    /// Lightweight router harness for fast bot handler tests.
    pub struct BotHarness {
        context: BotContext,
        router: Router,
    }

    impl BotHarness {
        pub fn new(router: Router) -> Result<Self> {
            let client = Client::builder("http://127.0.0.1:9")?
                .bot_token("123:abc")?
                .build()?;
            Ok(Self::with_client(client, router))
        }

        pub fn with_client(client: Client, router: Router) -> Self {
            Self {
                context: BotContext::new(client),
                router,
            }
        }

        pub async fn dispatch(&self, update: Update) -> Result<DispatchOutcome> {
            let update_id = update.update_id;
            match self.router.dispatch(self.context.clone(), update).await? {
                true => Ok(DispatchOutcome::Handled { update_id }),
                false => Ok(DispatchOutcome::Ignored { update_id }),
            }
        }

        pub async fn dispatch_json(&self, payload: &[u8]) -> Result<DispatchOutcome> {
            let update: Update = serde_json::from_slice(payload).map_err(|source| {
                invalid_request(format!(
                    "failed to deserialize test update payload: {source}"
                ))
            })?;
            self.dispatch(update).await
        }
    }
}

async fn wait_if_needed(duration: Duration) {
    if duration.is_zero() {
        return;
    }

    sleep(duration).await;
}

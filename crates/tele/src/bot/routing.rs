use super::*;

#[derive(Clone, Debug, Default)]
struct DispatchState {
    command_target: Option<String>,
}

type RouteFilterFn = Arc<dyn Fn(&Update, &DispatchState) -> bool + Send + Sync + 'static>;
type RouteHandlerFn =
    Arc<dyn Fn(BotContext, Update, DispatchState) -> HandlerFuture + Send + Sync + 'static>;
type ExtractedFilterFn<E> = Arc<dyn Fn(&E, &Update) -> bool + Send + Sync + 'static>;
type ExtractedGuardFn<E> = Arc<dyn Fn(&E, &Update) -> HandlerResult + Send + Sync + 'static>;
type ExtractedMapFn<E, T> = Arc<dyn Fn(E, &Update) -> Option<T> + Send + Sync + 'static>;

#[derive(Clone)]
struct Route {
    filter: RouteFilterFn,
    handler: RouteHandlerFn,
}

#[derive(Clone, Debug)]
struct CommandTargetConfig {
    username: Option<String>,
    auto_resolve: bool,
}

impl Default for CommandTargetConfig {
    fn default() -> Self {
        Self {
            username: None,
            auto_resolve: true,
        }
    }
}

fn command_mention_from_text(text: &str) -> Option<String> {
    let token = text.split_whitespace().next()?;
    let command = token.strip_prefix('/')?;
    let (name, mention) = command.split_once('@')?;
    if name.is_empty() {
        return None;
    }
    normalize_bot_username(mention)
}

fn update_mentions_command(update: &Update) -> bool {
    extract_text(update)
        .and_then(command_mention_from_text)
        .is_some()
}

fn updates_mention_command(updates: &[Update]) -> bool {
    updates.iter().any(update_mentions_command)
}

/// Parsed slash command with command name and trailing arguments.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandData {
    pub name: String,
    pub mention: Option<String>,
    pub args: String,
}

impl CommandData {
    pub fn args_trimmed(&self) -> &str {
        self.args.trim()
    }

    pub fn has_args(&self) -> bool {
        !self.args_trimmed().is_empty()
    }

    pub fn target_mention(&self) -> Option<&str> {
        self.mention.as_deref()
    }

    pub fn is_addressed_to(&self, bot_username: Option<&str>) -> bool {
        let Some(mention) = self.mention.as_deref() else {
            return true;
        };
        let Some(bot_username) = bot_username else {
            return false;
        };
        let Some(expected) = normalize_bot_username(bot_username) else {
            return false;
        };
        mention.eq_ignore_ascii_case(expected.as_str())
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

/// Route-level parser for a command's trailing argument string.
pub trait CommandArgs: Sized {
    fn parse(args: &str) -> std::result::Result<Self, String>;
}

impl CommandArgs for String {
    fn parse(args: &str) -> std::result::Result<Self, String> {
        Ok(args.trim().to_owned())
    }
}

impl CommandArgs for Vec<String> {
    fn parse(args: &str) -> std::result::Result<Self, String> {
        if args.trim().is_empty() {
            return Ok(Vec::new());
        }
        tokenize_command_args(args).ok_or_else(|| "invalid quoted command arguments".to_owned())
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

/// Codec-aware callback extractor payload with both decoded payload and raw data.
#[derive(Clone, Debug)]
pub struct CodedCallbackInput<T, C = CallbackPayloadCodec> {
    pub payload: T,
    pub raw: String,
    _codec: std::marker::PhantomData<C>,
}

impl<T, C> CodedCallbackInput<T, C>
where
    C: CallbackCodec<T>,
{
    pub fn from_raw(raw: impl Into<String>) -> Result<Self> {
        let raw = raw.into();
        let payload = C::decode_callback_data(raw.as_str())?;
        Ok(Self {
            payload,
            raw,
            _codec: std::marker::PhantomData,
        })
    }
}

/// Default typed callback extractor payload using [`CallbackPayload`].
pub type TypedCallbackInput<T> = CodedCallbackInput<T, CallbackPayloadCodec>;

/// Compact callback extractor payload using [`CompactCallbackCodec`].
pub type CompactCallbackInput<T> = CodedCallbackInput<T, CompactCallbackCodec>;

impl<T, C> UpdateExtractor for CodedCallbackInput<T, C>
where
    C: CallbackCodec<T>,
{
    fn extract(update: &Update) -> Option<Self> {
        let raw = extract_callback_data(update)?.to_owned();
        let payload = C::decode_callback_data(raw.as_str()).ok()?;
        Some(Self {
            payload,
            raw,
            _codec: std::marker::PhantomData,
        })
    }

    fn describe() -> &'static str {
        "callback payload"
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

    pub fn parse_json<T>(&self) -> Result<T>
    where
        T: DeserializeOwned,
    {
        serde_json::from_str(&self.0.data).map_err(|source| Error::InvalidRequest {
            reason: format!("invalid web_app_data JSON payload: {source}"),
        })
    }

    pub fn parse_query_payload<T>(&self) -> Result<WebAppQueryPayload<T>>
    where
        T: DeserializeOwned,
    {
        WebAppQueryPayload::parse(&self.0)
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

/// Request-scoped chat-member cache for the acting user.
#[derive(Clone, Debug)]
pub struct CurrentUserChatMember(pub ChatMember);

impl CurrentUserChatMember {
    pub fn into_inner(self) -> ChatMember {
        self.0
    }
}

impl AsRef<ChatMember> for CurrentUserChatMember {
    fn as_ref(&self) -> &ChatMember {
        &self.0
    }
}

/// Request-scoped chat-member cache for the bot account.
#[derive(Clone, Debug)]
pub struct CurrentBotChatMember(pub ChatMember);

impl CurrentBotChatMember {
    pub fn into_inner(self) -> ChatMember {
        self.0
    }
}

impl AsRef<ChatMember> for CurrentBotChatMember {
    fn as_ref(&self) -> &ChatMember {
        &self.0
    }
}

fn user_message_for_error(error: &Error, fallback: &str) -> String {
    match error.classification() {
        ErrorClass::Authentication => "bot authentication failed, please contact admin".to_owned(),
        ErrorClass::RateLimited => "too many requests, please retry shortly".to_owned(),
        _ => fallback.to_owned(),
    }
}

async fn resolve_error_with_policy(
    context: BotContext,
    update: Update,
    policy: ErrorPolicy,
    error: Error,
) -> Result<()> {
    match policy {
        ErrorPolicy::Propagate => Err(error),
        ErrorPolicy::Ignore => Ok(()),
        ErrorPolicy::ReplyUser { fallback_message } => {
            let message = user_message_for_error(&error, &fallback_message);
            let _ = context.reply_text(&update, message).await?;
            Ok(())
        }
    }
}

async fn resolve_handler_result(
    context: BotContext,
    update: Update,
    outcome: HandlerResult,
) -> Result<()> {
    match outcome {
        Ok(()) => Ok(()),
        Err(error) => context.resolve_handler_error(&update, error).await,
    }
}

async fn resolve_handler_result_with_policy(
    context: BotContext,
    update: Update,
    policy: ErrorPolicy,
    outcome: HandlerResult,
) -> Result<()> {
    match outcome {
        Ok(()) => Ok(()),
        Err(HandlerError::UserFacing { message }) => {
            context
                .resolve_handler_error(&update, HandlerError::user(message))
                .await
        }
        Err(HandlerError::Internal(error)) => {
            resolve_error_with_policy(context, update, policy, error).await
        }
    }
}

type ContextExtensionValue = Arc<dyn Any + Send + Sync>;
type ContextExtensions = Arc<StdRwLock<HashMap<TypeId, ContextExtensionValue>>>;

fn downcast_extension<T>(value: ContextExtensionValue) -> Option<Arc<T>>
where
    T: Send + Sync + 'static,
{
    Arc::downcast::<T>(value).ok()
}

/// Dispatch context passed to handlers and middlewares.
#[derive(Clone)]
pub struct BotContext {
    client: Client,
    extensions: ContextExtensions,
}

impl BotContext {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            extensions: Arc::new(StdRwLock::new(HashMap::new())),
        }
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    pub fn insert_extension<T>(&self, value: T) -> Option<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        self.insert_extension_shared(Arc::new(value))
    }

    pub fn insert_extension_shared<T>(&self, value: Arc<T>) -> Option<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        let previous = self
            .extensions
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(TypeId::of::<T>(), value);
        previous.and_then(downcast_extension::<T>)
    }

    pub fn get_or_insert_extension_with<T>(&self, init: impl FnOnce() -> T) -> Arc<T>
    where
        T: Send + Sync + 'static,
    {
        if let Some(value) = self.get_extension::<T>() {
            return value;
        }

        let mut extensions = self
            .extensions
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(value) = extensions
            .get(&TypeId::of::<T>())
            .cloned()
            .and_then(downcast_extension::<T>)
        {
            return value;
        }

        let value = Arc::new(init());
        let _ = extensions.insert(TypeId::of::<T>(), value.clone());
        value
    }

    pub fn get_extension<T>(&self) -> Option<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        self.extensions
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(&TypeId::of::<T>())
            .cloned()
            .and_then(downcast_extension::<T>)
    }

    pub fn contains_extension<T>(&self) -> bool
    where
        T: Send + Sync + 'static,
    {
        self.extensions
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .contains_key(&TypeId::of::<T>())
    }

    pub fn remove_extension<T>(&self) -> Option<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        self.extensions
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .remove(&TypeId::of::<T>())
            .and_then(downcast_extension::<T>)
    }

    pub fn clear_extensions(&self) {
        self.extensions
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clear();
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

    /// Registers explicit commands with retry/backoff policy.
    pub async fn set_commands_with_retry(
        &self,
        commands: Vec<BotCommand>,
        policy: BootstrapRetryPolicy,
    ) -> Result<bool> {
        self.client
            .ergo()
            .set_commands_with_retry(commands, policy)
            .await
    }

    /// Registers typed commands with optional scope and language code.
    pub async fn set_typed_commands_with_options<C>(
        &self,
        scope: Option<BotCommandScope>,
        language_code: Option<String>,
    ) -> Result<bool>
    where
        C: BotCommands,
    {
        self.client
            .ergo()
            .set_typed_commands_with_options::<C>(scope, language_code)
            .await
    }

    /// Registers typed commands with options and retry/backoff.
    pub async fn set_typed_commands_with_options_retry<C>(
        &self,
        scope: Option<BotCommandScope>,
        language_code: Option<String>,
        policy: BootstrapRetryPolicy,
    ) -> Result<bool>
    where
        C: BotCommands,
    {
        self.client
            .ergo()
            .set_typed_commands_with_options_retry::<C>(scope, language_code, policy)
            .await
    }

    /// Applies chat menu button with retry/backoff.
    pub async fn set_chat_menu_button_with_retry(
        &self,
        request: &crate::types::advanced::AdvancedSetChatMenuButtonRequest,
        policy: BootstrapRetryPolicy,
    ) -> Result<bool> {
        self.client
            .ergo()
            .set_chat_menu_button_with_retry(request, policy)
            .await
    }

    /// Runs one-shot startup bootstrap (`getMe`/commands/menu) without retries.
    pub async fn bootstrap(&self, plan: &BootstrapPlan) -> Result<BootstrapReport> {
        self.client.ergo().bootstrap(plan).await
    }

    /// Runs startup bootstrap with retry/backoff policy.
    pub async fn bootstrap_with_retry(
        &self,
        plan: &BootstrapPlan,
        policy: BootstrapRetryPolicy,
    ) -> Result<BootstrapReport> {
        self.client.ergo().bootstrap_with_retry(plan, policy).await
    }

    /// Runs startup bootstrap and prepares router command-target state.
    pub async fn bootstrap_router(
        &self,
        router: &Router,
        plan: &BootstrapPlan,
    ) -> Result<BootstrapReport> {
        self.bootstrap_router_with_retry(
            router,
            plan,
            BootstrapRetryPolicy {
                max_attempts: 1,
                continue_on_failure: false,
                ..BootstrapRetryPolicy::default()
            },
        )
        .await
    }

    /// Runs startup bootstrap with retry/backoff and prepares router state.
    pub async fn bootstrap_router_with_retry(
        &self,
        router: &Router,
        plan: &BootstrapPlan,
        policy: BootstrapRetryPolicy,
    ) -> Result<BootstrapReport> {
        let report = self.bootstrap_with_retry(plan, policy).await?;
        if let Some(me) = report.me.as_ref() {
            let _ = router.prepare_with_user(me)?;
        } else {
            let _ = router.prepare(self.client()).await?;
        }
        Ok(report)
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
        self.client
            .ergo()
            .answer_web_app_query(web_app_query_id, result)
            .await
    }

    /// Answers `answerWebAppQuery` with a pre-built inline result payload.
    pub async fn answer_web_app_query_result(
        &self,
        web_app_query_id: impl Into<String>,
        result: InlineQueryResult,
    ) -> Result<SentWebAppMessage> {
        let request =
            crate::types::advanced::AdvancedAnswerWebAppQueryRequest::new(web_app_query_id, result);
        self.advanced().answer_web_app_query_typed(&request).await
    }

    /// Parses WebApp payload and answers `answerWebAppQuery` in one step.
    pub async fn answer_web_app_query_from_payload<T, R>(
        &self,
        web_app_data: &WebAppData,
        result: R,
    ) -> Result<SentWebAppMessage>
    where
        T: DeserializeOwned,
        R: Serialize,
    {
        self.client
            .ergo()
            .answer_web_app_query_from_payload::<T, R>(web_app_data, result)
            .await
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

#[derive(Clone, Debug)]
struct CurrentBotUser(User);

async fn fetch_chat_member(
    context: &BotContext,
    chat_id: i64,
    user_id: UserId,
) -> Result<ChatMember> {
    let request = GetChatMemberRequest {
        chat_id: ChatId::from(chat_id),
        user_id,
    };
    context.chats().get_chat_member(&request).await
}

fn require_chat_context(update: &Update, message: &str) -> HandlerResult {
    let Some(chat) = extract_chat(update) else {
        return Err(HandlerError::user(message));
    };
    if chat.is_group_chat() {
        Ok(())
    } else {
        Err(HandlerError::user(message))
    }
}

async fn current_user_chat_member(context: &BotContext, update: &Update) -> Result<ChatMember> {
    if let Some(member) = context.get_extension::<CurrentUserChatMember>() {
        return Ok(member.as_ref().0.clone());
    }

    let Some(chat_id) = update_chat_id(update) else {
        return Err(invalid_request(
            "update does not contain a chat id for chat member lookup",
        ));
    };
    let Some(user) = extract_user(update) else {
        return Err(invalid_request(
            "update does not contain an acting user for chat member lookup",
        ));
    };

    let member = fetch_chat_member(context, chat_id, user.id).await?;
    let _ = context.insert_extension(CurrentUserChatMember(member.clone()));
    Ok(member)
}

async fn current_bot_chat_member(context: &BotContext, update: &Update) -> Result<ChatMember> {
    if let Some(member) = context.get_extension::<CurrentBotChatMember>() {
        return Ok(member.as_ref().0.clone());
    }

    let Some(chat_id) = update_chat_id(update) else {
        return Err(invalid_request(
            "update does not contain a chat id for bot member lookup",
        ));
    };

    let bot_user = if let Some(user) = context.get_extension::<CurrentBotUser>() {
        user.as_ref().0.clone()
    } else {
        let me = context.bot().get_me().await?;
        let _ = context.insert_extension(CurrentBotUser(me.clone()));
        me
    };

    let member = fetch_chat_member(context, chat_id, bot_user.id).await?;
    let _ = context.insert_extension(CurrentBotChatMember(member.clone()));
    Ok(member)
}

fn missing_permissions(
    member: &ChatMember,
    permissions: &[ChatMemberPermission],
) -> Vec<&'static str> {
    permissions
        .iter()
        .filter(|permission| !member.has_permission(**permission))
        .map(ChatMemberPermission::as_str)
        .collect()
}

/// Declarative update router with middleware support.
#[derive(Clone, Default)]
pub struct Router {
    routes: Vec<Route>,
    middlewares: Vec<MiddlewareFn>,
    fallback: Option<HandlerFn>,
    command_target: Arc<StdRwLock<CommandTargetConfig>>,
    has_command_routes: bool,
}

/// Route-level in-memory throttle key scope.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ThrottleScope {
    User,
    Chat,
    Command,
}

#[derive(Clone, Default)]
struct RouteThrottleStore {
    inner: Arc<StdRwLock<HashMap<String, VecDeque<Instant>>>>,
}

impl RouteThrottleStore {
    fn allow(&self, key: String, limit: usize, window: Duration) -> bool {
        if limit == 0 || window.is_zero() {
            return true;
        }

        let now = Instant::now();
        let mut inner = self
            .inner
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let (allowed, remove_key) = {
            let history = inner.entry(key.clone()).or_default();
            while history
                .front()
                .is_some_and(|instant| now.saturating_duration_since(*instant) >= window)
            {
                let _ = history.pop_front();
            }
            let allowed = history.len() < limit;
            if allowed {
                history.push_back(now);
            }
            (allowed, history.is_empty())
        };
        if remove_key {
            let _ = inner.remove(&key);
        }
        allowed
    }
}

#[derive(Clone)]
struct RouteDslConfig {
    guards: Vec<GuardFn>,
    route_label: String,
}

impl RouteDslConfig {
    fn new(route_label: impl Into<String>) -> Self {
        Self {
            guards: Vec::new(),
            route_label: route_label.into(),
        }
    }

    fn push_guard<G, Fut>(&mut self, guard: G)
    where
        G: Fn(BotContext, Update) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult> + Send + 'static,
    {
        self.guards.push(Arc::new(move |context, update| {
            Box::pin(guard(context, update))
        }));
    }

    fn group_only(&mut self) {
        self.push_guard(|_context, update| async move {
            let Some(chat) = extract_chat(&update) else {
                return Err(HandlerError::user(
                    "this route is only available in group chats",
                ));
            };
            if chat.is_group_chat() {
                Ok(())
            } else {
                Err(HandlerError::user(
                    "this route is only available in group chats",
                ))
            }
        });
    }

    fn supergroup_only(&mut self) {
        self.push_guard(|_context, update| async move {
            let Some(chat) = extract_chat(&update) else {
                return Err(HandlerError::user(
                    "this route is only available in supergroups",
                ));
            };
            if chat.is_supergroup() {
                Ok(())
            } else {
                Err(HandlerError::user(
                    "this route is only available in supergroups",
                ))
            }
        });
    }

    fn admin_only(&mut self) {
        self.push_guard(|context, update| async move {
            require_chat_context(&update, "this route is only available in group chats")?;
            let member = current_user_chat_member(&context, &update)
                .await
                .map_err(HandlerError::from)?;
            if member.is_admin() {
                Ok(())
            } else {
                Err(HandlerError::user("chat administrators only"))
            }
        });
    }

    fn owner_only(&mut self) {
        self.push_guard(|context, update| async move {
            require_chat_context(&update, "this route is only available in group chats")?;
            let member = current_user_chat_member(&context, &update)
                .await
                .map_err(HandlerError::from)?;
            if member.is_owner() {
                Ok(())
            } else {
                Err(HandlerError::user("chat owner only"))
            }
        });
    }

    fn require_permissions(&mut self, permissions: Vec<ChatMemberPermission>) {
        self.push_guard(move |context, update| {
            let permissions = permissions.clone();
            async move {
                require_chat_context(&update, "this route is only available in group chats")?;
                let member = current_user_chat_member(&context, &update)
                    .await
                    .map_err(HandlerError::from)?;
                let missing = missing_permissions(&member, permissions.as_slice());
                if missing.is_empty() {
                    Ok(())
                } else {
                    Err(HandlerError::user(format!(
                        "missing required permissions: {}",
                        missing.join(", ")
                    )))
                }
            }
        });
    }

    fn bot_can(&mut self, permissions: Vec<ChatMemberPermission>) {
        self.push_guard(move |context, update| {
            let permissions = permissions.clone();
            async move {
                require_chat_context(&update, "this route is only available in group chats")?;
                let member = current_bot_chat_member(&context, &update)
                    .await
                    .map_err(HandlerError::from)?;
                let missing = missing_permissions(&member, permissions.as_slice());
                if missing.is_empty() {
                    Ok(())
                } else {
                    Err(HandlerError::user(format!(
                        "bot is missing required permissions: {}",
                        missing.join(", ")
                    )))
                }
            }
        });
    }

    fn throttle(&mut self, scope: ThrottleScope, limit: u32, window: Duration) {
        if window.is_zero() {
            return;
        }

        let store = RouteThrottleStore::default();
        let route_label = self.route_label.clone();
        let limit = limit.max(1) as usize;
        self.push_guard(move |_context, update| {
            let store = store.clone();
            let route_label = route_label.clone();
            async move {
                let key = throttle_key(scope, &update, route_label.as_str())?;
                if store.allow(key, limit, window) {
                    Ok(())
                } else {
                    Err(HandlerError::user(
                        "too many matching requests, please retry shortly",
                    ))
                }
            }
        });
    }
}

fn throttle_key(
    scope: ThrottleScope,
    update: &Update,
    route_label: &str,
) -> std::result::Result<String, HandlerError> {
    match scope {
        ThrottleScope::User => {
            let Some(user_id) = extract_user_id(update) else {
                return Err(HandlerError::user(
                    "this route requires an acting user for throttling",
                ));
            };
            Ok(format!("{route_label}:user:{user_id}"))
        }
        ThrottleScope::Chat => {
            let Some(chat_id) = update_chat_id(update) else {
                return Err(HandlerError::user(
                    "this route requires a chat context for throttling",
                ));
            };
            Ok(format!("{route_label}:chat:{chat_id}"))
        }
        ThrottleScope::Command => Ok(format!("{route_label}:command")),
    }
}

async fn run_route_guards(
    guards: &[GuardFn],
    context: BotContext,
    update: Update,
) -> HandlerResult {
    for guard in guards {
        guard(context.clone(), update.clone()).await?;
    }
    Ok(())
}

fn extracted_route_matches<E>(update: &Update, filters: &[ExtractedFilterFn<E>]) -> bool
where
    E: UpdateExtractor,
{
    let Some(extracted) = E::extract(update) else {
        return false;
    };
    filters.iter().all(|filter| filter(&extracted, update))
}

fn run_extracted_guards<E>(
    guards: &[ExtractedGuardFn<E>],
    extracted: &E,
    update: &Update,
) -> HandlerResult {
    for guard in guards {
        guard(extracted, update)?;
    }
    Ok(())
}

macro_rules! impl_route_dsl_methods {
    () => {
        pub fn group_only(mut self) -> Self {
            self.config.group_only();
            self
        }

        pub fn supergroup_only(mut self) -> Self {
            self.config.supergroup_only();
            self
        }

        pub fn admin_only(mut self) -> Self {
            self.config.admin_only();
            self
        }

        pub fn owner_only(mut self) -> Self {
            self.config.owner_only();
            self
        }

        pub fn require_permissions(mut self, permissions: &[ChatMemberPermission]) -> Self {
            self.config.require_permissions(permissions.to_vec());
            self
        }

        pub fn bot_can(mut self, permissions: &[ChatMemberPermission]) -> Self {
            self.config.bot_can(permissions.to_vec());
            self
        }

        pub fn throttle(mut self, scope: ThrottleScope, limit: u32, window: Duration) -> Self {
            self.config.throttle(scope, limit, window);
            self
        }

        pub fn throttle_user(self, window: Duration) -> Self {
            self.throttle(ThrottleScope::User, 1, window)
        }

        pub fn throttle_chat(self, window: Duration) -> Self {
            self.throttle(ThrottleScope::Chat, 1, window)
        }

        pub fn throttle_command(self, window: Duration) -> Self {
            self.throttle(ThrottleScope::Command, 1, window)
        }
    };
}

impl Router {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn custom_route<P>(
        &mut self,
        route_label: impl Into<String>,
        predicate: P,
    ) -> UpdateRouteBuilder<'_>
    where
        P: Fn(&Update) -> bool + Send + Sync + 'static,
    {
        let filter = Arc::new(move |update: &Update, _state: &DispatchState| predicate(update));
        UpdateRouteBuilder::new(self, route_label.into(), filter)
    }

    pub fn message_route(&mut self) -> UpdateRouteBuilder<'_> {
        self.custom_route("message", |update| update.message.is_some())
    }

    pub fn message_like_route(&mut self) -> UpdateRouteBuilder<'_> {
        self.custom_route("message_like", |update| extract_message(update).is_some())
    }

    pub fn message_kind_route(&mut self, kind: MessageKind) -> UpdateRouteBuilder<'_> {
        self.custom_route(format!("message_kind:{kind:?}"), move |update| {
            update
                .message
                .as_ref()
                .is_some_and(|message| message.has_kind(kind))
        })
    }

    pub fn message_like_kind_route(&mut self, kind: MessageKind) -> UpdateRouteBuilder<'_> {
        self.custom_route(format!("message_like_kind:{kind:?}"), move |update| {
            extract_message(update).is_some_and(|message| message.has_kind(kind))
        })
    }

    pub fn update_kind_route(&mut self, kind: UpdateKind) -> UpdateRouteBuilder<'_> {
        self.custom_route(format!("update_kind:{kind:?}"), move |update| {
            update.has_kind(kind)
        })
    }

    pub fn callback_query_route(&mut self) -> UpdateRouteBuilder<'_> {
        self.custom_route("callback_query", |update| update.callback_query.is_some())
    }

    pub fn inline_query_route(&mut self) -> UpdateRouteBuilder<'_> {
        self.custom_route("inline_query", |update| update.inline_query.is_some())
    }

    pub fn extracted_route<E>(&mut self) -> ExtractedRouteBuilder<'_, E>
    where
        E: UpdateExtractor + Send + 'static,
    {
        ExtractedRouteBuilder::new(self, format!("extract:{}", std::any::type_name::<E>()))
    }

    pub fn text_route(&mut self) -> ExtractedRouteBuilder<'_, TextInput> {
        ExtractedRouteBuilder::new(self, "text")
    }

    pub fn callback_data_route(&mut self) -> ExtractedRouteBuilder<'_, CallbackInput> {
        ExtractedRouteBuilder::new(self, "callback_data")
    }

    pub fn callback_json_route<T>(&mut self) -> ExtractedRouteBuilder<'_, JsonCallback<T>>
    where
        T: DeserializeOwned + Send + 'static,
    {
        ExtractedRouteBuilder::new(
            self,
            format!("callback_json:{}", std::any::type_name::<T>()),
        )
    }

    pub fn web_app_route(&mut self) -> ExtractedRouteBuilder<'_, WebAppInput> {
        ExtractedRouteBuilder::new(self, "web_app")
    }

    pub fn write_access_allowed_route(
        &mut self,
    ) -> ExtractedRouteBuilder<'_, WriteAccessAllowedInput> {
        ExtractedRouteBuilder::new(self, "write_access_allowed")
    }

    pub fn command_input_route(&mut self) -> CommandInputRouteBuilder<'_> {
        CommandInputRouteBuilder::new(self)
    }

    pub fn command_route(&mut self, command: impl Into<String>) -> CommandRouteBuilder<'_> {
        CommandRouteBuilder::new(self, command.into())
    }

    pub fn typed_command_route<C>(&mut self) -> TypedCommandRouteBuilder<'_, C>
    where
        C: BotCommands + Send + Sync + 'static,
    {
        TypedCommandRouteBuilder::new(self)
    }

    pub fn callback_route_with_codec<T, C>(&mut self) -> CallbackRouteBuilder<'_, T, C>
    where
        T: Send + Sync + 'static,
        C: CallbackCodec<T>,
    {
        CallbackRouteBuilder::new(self)
    }

    pub fn typed_callback_route<T>(&mut self) -> TypedCallbackRouteBuilder<'_, T>
    where
        T: CallbackPayload + Send + Sync + 'static,
    {
        self.callback_route_with_codec::<T, CallbackPayloadCodec>()
    }

    pub fn compact_callback_route<T>(&mut self) -> CompactCallbackRouteBuilder<'_, T>
    where
        T: CompactCallbackPayload + Send + Sync + 'static,
    {
        self.callback_route_with_codec::<T, CompactCallbackCodec>()
    }

    fn command_target_snapshot(&self) -> CommandTargetConfig {
        self.command_target
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    fn command_target_username(&self) -> Option<String> {
        self.command_target_snapshot().username
    }

    fn set_command_target_config(&self, username: Option<String>, auto_resolve: bool) -> &Self {
        let mut state = self
            .command_target
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.username = username;
        state.auto_resolve = auto_resolve;
        self
    }

    async fn prepare_command_target(&self, client: &Client) -> Result<()> {
        if !self.has_command_routes {
            return Ok(());
        }

        let state = self.command_target_snapshot();
        if state.username.is_some() || !state.auto_resolve {
            return Ok(());
        }

        let me = client.bot().get_me().await?;
        let username = normalize_bot_username(me.username.as_deref().ok_or_else(|| {
            invalid_request(
                "getMe returned bot without username; command mention routing requires a username",
            )
        })?)
        .ok_or_else(|| invalid_request("bot username cannot be empty"))?;

        let mut state = self
            .command_target
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if state.username.is_none() && state.auto_resolve {
            state.username = Some(username);
        }

        Ok(())
    }

    fn prepare_command_target_with_user(&self, me: &User) -> Result<()> {
        if !self.has_command_routes {
            return Ok(());
        }

        let state = self.command_target_snapshot();
        if state.username.is_some() || !state.auto_resolve {
            return Ok(());
        }

        let username = normalize_bot_username(me.username.as_deref().ok_or_else(|| {
            invalid_request(
                "getMe returned bot without username; command mention routing requires a username",
            )
        })?)
        .ok_or_else(|| invalid_request("bot username cannot be empty"))?;

        let mut state = self
            .command_target
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if state.username.is_none() && state.auto_resolve {
            state.username = Some(username);
        }

        Ok(())
    }

    /// Pre-resolves runtime routing state that may require one-time SDK I/O.
    ///
    /// Today this eagerly caches the current bot username for command mention
    /// routing so dispatch itself can stay side-effect free.
    pub async fn prepare(&self, client: &Client) -> Result<&Self> {
        self.prepare_command_target(client).await?;
        Ok(self)
    }

    /// Prepares routing state from a previously fetched `getMe` result.
    pub fn prepare_with_user(&self, me: &User) -> Result<&Self> {
        self.prepare_command_target_with_user(me)?;
        Ok(self)
    }

    /// Prepares routing state only when the current update contains a bot mention command.
    pub async fn prepare_for_update(&self, client: &Client, update: &Update) -> Result<&Self> {
        if update_mentions_command(update) {
            self.prepare(client).await?;
        }
        Ok(self)
    }

    pub(super) async fn prepare_for_updates(
        &self,
        client: &Client,
        updates: &[Update],
    ) -> Result<()> {
        if updates_mention_command(updates) {
            self.prepare(client).await?;
        }
        Ok(())
    }

    /// Sets command target bot username used by command routes.
    ///
    /// When set, mentioned commands like `/start@MyBot` are accepted only if
    /// `MyBot` matches this target case-insensitively.
    pub fn command_target(mut self, bot_username: impl Into<String>) -> Result<Self> {
        let _ = self.set_command_target(bot_username)?;
        Ok(self)
    }

    /// Sets command target bot username used by command routes.
    pub fn set_command_target(&mut self, bot_username: impl Into<String>) -> Result<&mut Self> {
        let raw = bot_username.into();
        let bot_username = normalize_bot_username(raw.as_str())
            .ok_or_else(|| invalid_request("command target bot username cannot be empty"))?;
        let _ = self.set_command_target_config(Some(bot_username), false);
        Ok(self)
    }

    /// Clears manual command target and re-enables lazy auto-resolution.
    pub fn clear_command_target(&mut self) -> &mut Self {
        let _ = self.set_command_target_config(None, true);
        self
    }

    /// Disables lazy auto-resolution for mentioned commands.
    ///
    /// After disabling, commands like `/start@ThisBot` are ignored unless an
    /// explicit target was configured with `set_command_target`.
    pub fn disable_auto_command_target(&mut self) -> &mut Self {
        let _ = self.set_command_target_config(None, false);
        self
    }

    /// Enables lazy auto-resolution for mentioned commands.
    pub fn enable_auto_command_target(&mut self) -> &mut Self {
        let username = self.command_target_username();
        let _ = self.set_command_target_config(username, true);
        self
    }

    fn current_dispatch_state(&self) -> DispatchState {
        DispatchState {
            command_target: self.command_target_username(),
        }
    }

    fn route_with_state<P, H, Fut>(&mut self, predicate: P, handler: H) -> &mut Self
    where
        P: Fn(&Update, &DispatchState) -> bool + Send + Sync + 'static,
        H: Fn(BotContext, Update, DispatchState) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        self.routes.push(Route {
            filter: Arc::new(predicate),
            handler: to_route_handler_fn(handler),
        });
        self
    }

    fn route_with_policy_state<P, H, Fut>(
        &mut self,
        predicate: P,
        policy: ErrorPolicy,
        handler: H,
    ) -> &mut Self
    where
        P: Fn(&Update, &DispatchState) -> bool + Send + Sync + 'static,
        H: Fn(BotContext, Update, DispatchState) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult> + Send + 'static,
    {
        let handler = Arc::new(handler);
        self.route_with_state(predicate, move |context, update, state| {
            let handler = Arc::clone(&handler);
            let context_for_policy = context.clone();
            let update_for_policy = update.clone();
            let policy = policy.clone();
            async move {
                let outcome = handler(context, update, state).await;
                resolve_handler_result_with_policy(
                    context_for_policy,
                    update_for_policy,
                    policy,
                    outcome,
                )
                .await
            }
        })
    }

    fn route_fallible_with_state<P, H, Fut>(&mut self, predicate: P, handler: H) -> &mut Self
    where
        P: Fn(&Update, &DispatchState) -> bool + Send + Sync + 'static,
        H: Fn(BotContext, Update, DispatchState) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult> + Send + 'static,
    {
        let handler = Arc::new(handler);
        self.route_with_state(predicate, move |context, update, state| {
            let handler = Arc::clone(&handler);
            let context_for_error = context.clone();
            let update_for_error = update.clone();
            async move {
                let outcome = handler(context, update, state).await;
                resolve_handler_result(context_for_error, update_for_error, outcome).await
            }
        })
    }

    pub fn route<P, H, Fut>(&mut self, predicate: P, handler: H) -> &mut Self
    where
        P: Fn(&Update) -> bool + Send + Sync + 'static,
        H: Fn(BotContext, Update) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult> + Send + 'static,
    {
        self.route_fallible_with_state(
            move |update, _state| predicate(update),
            move |context, update, _state| handler(context, update),
        )
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
        Fut: Future<Output = HandlerResult> + Send + 'static,
    {
        self.route_with_policy_state(
            move |update, _state| predicate(update),
            policy,
            move |context, update, _state| handler(context, update),
        )
    }

    /// Adds route with fallible handler that can return user-facing errors.
    pub fn route_fallible<P, H, Fut>(&mut self, predicate: P, handler: H) -> &mut Self
    where
        P: Fn(&Update) -> bool + Send + Sync + 'static,
        H: Fn(BotContext, Update) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult> + Send + 'static,
    {
        self.route(predicate, handler)
    }

    pub fn fallback<H, Fut>(&mut self, handler: H) -> &mut Self
    where
        H: Fn(BotContext, Update) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult> + Send + 'static,
    {
        let handler = Arc::new(handler);
        self.fallback = Some(Arc::new(move |context: BotContext, update: Update| {
            let handler = Arc::clone(&handler);
            let context_for_error = context.clone();
            let update_for_error = update.clone();
            Box::pin(async move {
                let outcome = handler(context, update).await;
                resolve_handler_result(context_for_error, update_for_error, outcome).await
            })
        }));
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
    ///
    /// This method is intentionally side-effect free and does not perform
    /// network I/O. If you rely on auto command-target resolution for
    /// `/cmd@BotUsername`, call `prepare` or `prepare_for_update` first.
    pub async fn dispatch(&self, context: BotContext, update: Update) -> Result<bool> {
        let dispatch_state = self.current_dispatch_state();

        let handler = self
            .routes
            .iter()
            .find(|route| (route.filter)(&update, &dispatch_state))
            .map(|route| {
                let route_handler = Arc::clone(&route.handler);
                let state = dispatch_state.clone();
                Arc::new(move |context: BotContext, update: Update| {
                    let state = state.clone();
                    route_handler(context, update, state)
                }) as HandlerFn
            })
            .or_else(|| self.fallback.as_ref().map(Arc::clone));

        let Some(handler) = handler else {
            return Ok(false);
        };

        let wrapped = self.apply_middlewares(handler);
        wrapped(context, update).await?;
        Ok(true)
    }

    /// Prepares runtime routing state for the given update and immediately dispatches it.
    pub async fn dispatch_prepared(&self, context: BotContext, update: Update) -> Result<bool> {
        self.prepare_for_update(context.client(), &update).await?;
        self.dispatch(context, update).await
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

/// Chainable DSL for non-extracting update routes.
pub struct UpdateRouteBuilder<'a> {
    router: &'a mut Router,
    filter: RouteFilterFn,
    config: RouteDslConfig,
}

impl<'a> UpdateRouteBuilder<'a> {
    fn new(router: &'a mut Router, route_label: impl Into<String>, filter: RouteFilterFn) -> Self {
        Self {
            router,
            filter,
            config: RouteDslConfig::new(route_label),
        }
    }

    impl_route_dsl_methods!();

    pub fn handle<H, Fut>(self, handler: H) -> &'a mut Router
    where
        H: Fn(BotContext, Update) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult> + Send + 'static,
    {
        let filter = Arc::clone(&self.filter);
        let guards = Arc::new(self.config.guards);
        let handler = Arc::new(handler);
        self.router.route_fallible_with_state(
            move |update, state| filter(update, state),
            move |context, update, _state| {
                let guards = Arc::clone(&guards);
                let handler = Arc::clone(&handler);
                async move {
                    run_route_guards(guards.as_ref(), context.clone(), update.clone()).await?;
                    handler(context, update).await
                }
            },
        )
    }

    pub fn handle_fallible<H, Fut>(self, handler: H) -> &'a mut Router
    where
        H: Fn(BotContext, Update) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult> + Send + 'static,
    {
        self.handle(handler)
    }

    pub fn handle_with_policy<H, Fut>(self, policy: ErrorPolicy, handler: H) -> &'a mut Router
    where
        H: Fn(BotContext, Update) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult> + Send + 'static,
    {
        let filter = Arc::clone(&self.filter);
        let guards = Arc::new(self.config.guards);
        let handler = Arc::new(handler);
        self.router.route_with_state(
            move |update, state| filter(update, state),
            move |context, update, _state| {
                let guards = Arc::clone(&guards);
                let handler = Arc::clone(&handler);
                let policy = policy.clone();
                async move {
                    if let Err(error) =
                        run_route_guards(guards.as_ref(), context.clone(), update.clone()).await
                    {
                        return context.resolve_handler_error(&update, error).await;
                    }
                    let context_for_policy = context.clone();
                    let update_for_policy = update.clone();
                    let outcome = handler(context, update).await;
                    resolve_handler_result_with_policy(
                        context_for_policy,
                        update_for_policy,
                        policy,
                        outcome,
                    )
                    .await
                }
            },
        )
    }
}

/// Chainable DSL for extractor-backed routes.
pub struct ExtractedRouteBuilder<'a, E> {
    router: &'a mut Router,
    config: RouteDslConfig,
    filters: Vec<ExtractedFilterFn<E>>,
    extracted_guards: Vec<ExtractedGuardFn<E>>,
    _extracted: std::marker::PhantomData<E>,
}

impl<'a, E> ExtractedRouteBuilder<'a, E>
where
    E: UpdateExtractor + Send + 'static,
{
    fn new(router: &'a mut Router, route_label: impl Into<String>) -> Self {
        Self {
            router,
            config: RouteDslConfig::new(route_label),
            filters: Vec::new(),
            extracted_guards: Vec::new(),
            _extracted: std::marker::PhantomData,
        }
    }

    impl_route_dsl_methods!();

    pub fn filter<P>(mut self, predicate: P) -> Self
    where
        P: Fn(&E, &Update) -> bool + Send + Sync + 'static,
    {
        self.filters.push(Arc::new(predicate));
        self
    }

    pub fn guard<G>(mut self, guard: G) -> Self
    where
        G: Fn(&E, &Update) -> HandlerResult + Send + Sync + 'static,
    {
        self.extracted_guards.push(Arc::new(guard));
        self
    }

    pub fn map<T, M>(self, mapper: M) -> MappedExtractedRouteBuilder<'a, E, T>
    where
        T: Send + 'static,
        M: Fn(E, &Update) -> Option<T> + Send + Sync + 'static,
    {
        MappedExtractedRouteBuilder {
            router: self.router,
            config: self.config,
            filters: self.filters,
            extracted_guards: self.extracted_guards,
            mapper: Arc::new(mapper),
            _extracted: std::marker::PhantomData,
        }
    }

    pub fn handle<H, Fut>(self, handler: H) -> &'a mut Router
    where
        H: Fn(BotContext, Update, E) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult> + Send + 'static,
    {
        let filters = Arc::new(self.filters);
        let extracted_guards = Arc::new(self.extracted_guards);
        let guards = Arc::new(self.config.guards);
        let handler = Arc::new(handler);
        self.router.route_fallible_with_state(
            {
                let filters = Arc::clone(&filters);
                move |update, _state| extracted_route_matches::<E>(update, filters.as_ref())
            },
            move |context, update, _state| {
                let filters = Arc::clone(&filters);
                let extracted_guards = Arc::clone(&extracted_guards);
                let guards = Arc::clone(&guards);
                let handler = Arc::clone(&handler);
                async move {
                    run_route_guards(guards.as_ref(), context.clone(), update.clone()).await?;
                    let Some(extracted) = E::extract(&update) else {
                        return Err(HandlerError::internal(invalid_request(format!(
                            "update does not contain {}",
                            E::describe()
                        ))));
                    };
                    if !filters.iter().all(|filter| filter(&extracted, &update)) {
                        return Ok(());
                    }
                    run_extracted_guards(extracted_guards.as_ref(), &extracted, &update)?;
                    handler(context, update, extracted).await
                }
            },
        )
    }

    pub fn handle_fallible<H, Fut>(self, handler: H) -> &'a mut Router
    where
        H: Fn(BotContext, Update, E) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult> + Send + 'static,
    {
        self.handle(handler)
    }

    pub fn handle_with_policy<H, Fut>(self, policy: ErrorPolicy, handler: H) -> &'a mut Router
    where
        H: Fn(BotContext, Update, E) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult> + Send + 'static,
    {
        let filters = Arc::new(self.filters);
        let extracted_guards = Arc::new(self.extracted_guards);
        let guards = Arc::new(self.config.guards);
        let handler = Arc::new(handler);
        self.router.route_with_state(
            {
                let filters = Arc::clone(&filters);
                move |update, _state| extracted_route_matches::<E>(update, filters.as_ref())
            },
            move |context, update, _state| {
                let filters = Arc::clone(&filters);
                let extracted_guards = Arc::clone(&extracted_guards);
                let guards = Arc::clone(&guards);
                let handler = Arc::clone(&handler);
                let policy = policy.clone();
                async move {
                    if let Err(error) =
                        run_route_guards(guards.as_ref(), context.clone(), update.clone()).await
                    {
                        return context.resolve_handler_error(&update, error).await;
                    }
                    let Some(extracted) = E::extract(&update) else {
                        return Err(invalid_request(format!(
                            "update does not contain {}",
                            E::describe()
                        )));
                    };
                    if !filters.iter().all(|filter| filter(&extracted, &update)) {
                        return Ok(());
                    }
                    if let Err(error) =
                        run_extracted_guards(extracted_guards.as_ref(), &extracted, &update)
                    {
                        return context.resolve_handler_error(&update, error).await;
                    }
                    let context_for_policy = context.clone();
                    let update_for_policy = update.clone();
                    let outcome = handler(context, update, extracted).await;
                    resolve_handler_result_with_policy(
                        context_for_policy,
                        update_for_policy,
                        policy,
                        outcome,
                    )
                    .await
                }
            },
        )
    }
}

/// Chainable DSL for extractor routes with a mapping step before the handler.
pub struct MappedExtractedRouteBuilder<'a, E, T> {
    router: &'a mut Router,
    config: RouteDslConfig,
    filters: Vec<ExtractedFilterFn<E>>,
    extracted_guards: Vec<ExtractedGuardFn<E>>,
    mapper: ExtractedMapFn<E, T>,
    _extracted: std::marker::PhantomData<E>,
}

impl<'a, E, T> MappedExtractedRouteBuilder<'a, E, T>
where
    E: UpdateExtractor + Send + 'static,
    T: Send + 'static,
{
    impl_route_dsl_methods!();

    pub fn handle<H, Fut>(self, handler: H) -> &'a mut Router
    where
        H: Fn(BotContext, Update, T) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult> + Send + 'static,
    {
        let filters = Arc::new(self.filters);
        let extracted_guards = Arc::new(self.extracted_guards);
        let guards = Arc::new(self.config.guards);
        let mapper = Arc::clone(&self.mapper);
        let handler = Arc::new(handler);
        self.router.route_fallible_with_state(
            {
                let filters = Arc::clone(&filters);
                let mapper = Arc::clone(&mapper);
                move |update, _state| {
                    let Some(extracted) = E::extract(update) else {
                        return false;
                    };
                    filters.iter().all(|filter| filter(&extracted, update))
                        && mapper(extracted, update).is_some()
                }
            },
            move |context, update, _state| {
                let filters = Arc::clone(&filters);
                let extracted_guards = Arc::clone(&extracted_guards);
                let guards = Arc::clone(&guards);
                let mapper = Arc::clone(&mapper);
                let handler = Arc::clone(&handler);
                async move {
                    run_route_guards(guards.as_ref(), context.clone(), update.clone()).await?;
                    let Some(extracted) = E::extract(&update) else {
                        return Err(HandlerError::internal(invalid_request(format!(
                            "update does not contain {}",
                            E::describe()
                        ))));
                    };
                    if !filters.iter().all(|filter| filter(&extracted, &update)) {
                        return Ok(());
                    }
                    run_extracted_guards(extracted_guards.as_ref(), &extracted, &update)?;
                    let Some(mapped) = mapper(extracted, &update) else {
                        return Ok(());
                    };
                    handler(context, update, mapped).await
                }
            },
        )
    }

    pub fn handle_fallible<H, Fut>(self, handler: H) -> &'a mut Router
    where
        H: Fn(BotContext, Update, T) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult> + Send + 'static,
    {
        self.handle(handler)
    }

    pub fn handle_with_policy<H, Fut>(self, policy: ErrorPolicy, handler: H) -> &'a mut Router
    where
        H: Fn(BotContext, Update, T) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult> + Send + 'static,
    {
        let filters = Arc::new(self.filters);
        let extracted_guards = Arc::new(self.extracted_guards);
        let guards = Arc::new(self.config.guards);
        let mapper = Arc::clone(&self.mapper);
        let handler = Arc::new(handler);
        self.router.route_with_state(
            {
                let filters = Arc::clone(&filters);
                let mapper = Arc::clone(&mapper);
                move |update, _state| {
                    let Some(extracted) = E::extract(update) else {
                        return false;
                    };
                    filters.iter().all(|filter| filter(&extracted, update))
                        && mapper(extracted, update).is_some()
                }
            },
            move |context, update, _state| {
                let filters = Arc::clone(&filters);
                let extracted_guards = Arc::clone(&extracted_guards);
                let guards = Arc::clone(&guards);
                let mapper = Arc::clone(&mapper);
                let handler = Arc::clone(&handler);
                let policy = policy.clone();
                async move {
                    if let Err(error) =
                        run_route_guards(guards.as_ref(), context.clone(), update.clone()).await
                    {
                        return context.resolve_handler_error(&update, error).await;
                    }
                    let Some(extracted) = E::extract(&update) else {
                        return Err(invalid_request(format!(
                            "update does not contain {}",
                            E::describe()
                        )));
                    };
                    if !filters.iter().all(|filter| filter(&extracted, &update)) {
                        return Ok(());
                    }
                    if let Err(error) =
                        run_extracted_guards(extracted_guards.as_ref(), &extracted, &update)
                    {
                        return context.resolve_handler_error(&update, error).await;
                    }
                    let Some(mapped) = mapper(extracted, &update) else {
                        return Ok(());
                    };
                    let context_for_policy = context.clone();
                    let update_for_policy = update.clone();
                    let outcome = handler(context, update, mapped).await;
                    resolve_handler_result_with_policy(
                        context_for_policy,
                        update_for_policy,
                        policy,
                        outcome,
                    )
                    .await
                }
            },
        )
    }
}

/// Chainable DSL for raw slash-command routes.
pub struct CommandInputRouteBuilder<'a> {
    router: &'a mut Router,
    config: RouteDslConfig,
}

impl<'a> CommandInputRouteBuilder<'a> {
    fn new(router: &'a mut Router) -> Self {
        Self {
            router,
            config: RouteDslConfig::new("command_input"),
        }
    }

    impl_route_dsl_methods!();

    pub fn handle<H, Fut>(self, handler: H) -> &'a mut Router
    where
        H: Fn(BotContext, Update, CommandData) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult> + Send + 'static,
    {
        let guards = Arc::new(self.config.guards);
        let handler = Arc::new(handler);
        self.router.has_command_routes = true;
        self.router.route_fallible_with_state(
            move |update, state| {
                extract_command_data_for_bot(update, state.command_target.as_deref()).is_some()
            },
            move |context, update, state| {
                let guards = Arc::clone(&guards);
                let handler = Arc::clone(&handler);
                async move {
                    run_route_guards(guards.as_ref(), context.clone(), update.clone()).await?;
                    let Some(command) =
                        extract_command_data_for_bot(&update, state.command_target.as_deref())
                    else {
                        return Err(HandlerError::internal(invalid_request(
                            "update does not contain a valid command",
                        )));
                    };
                    handler(context, update, command).await
                }
            },
        )
    }

    pub fn handle_fallible<H, Fut>(self, handler: H) -> &'a mut Router
    where
        H: Fn(BotContext, Update, CommandData) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult> + Send + 'static,
    {
        self.handle(handler)
    }

    pub fn handle_with_policy<H, Fut>(self, policy: ErrorPolicy, handler: H) -> &'a mut Router
    where
        H: Fn(BotContext, Update, CommandData) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult> + Send + 'static,
    {
        let guards = Arc::new(self.config.guards);
        let handler = Arc::new(handler);
        self.router.has_command_routes = true;
        self.router.route_with_state(
            move |update, state| {
                extract_command_data_for_bot(update, state.command_target.as_deref()).is_some()
            },
            move |context, update, state| {
                let guards = Arc::clone(&guards);
                let handler = Arc::clone(&handler);
                let policy = policy.clone();
                async move {
                    if let Err(error) =
                        run_route_guards(guards.as_ref(), context.clone(), update.clone()).await
                    {
                        return context.resolve_handler_error(&update, error).await;
                    }
                    let Some(command) =
                        extract_command_data_for_bot(&update, state.command_target.as_deref())
                    else {
                        return Err(invalid_request("update does not contain a valid command"));
                    };
                    let context_for_policy = context.clone();
                    let update_for_policy = update.clone();
                    let outcome = handler(context, update, command).await;
                    resolve_handler_result_with_policy(
                        context_for_policy,
                        update_for_policy,
                        policy,
                        outcome,
                    )
                    .await
                }
            },
        )
    }
}

/// Chainable DSL for command-scoped route configuration.
pub struct CommandRouteBuilder<'a> {
    router: &'a mut Router,
    command: String,
    config: RouteDslConfig,
}

impl<'a> CommandRouteBuilder<'a> {
    fn new(router: &'a mut Router, command: String) -> Self {
        let route_label = format!("command:{command}");
        Self {
            router,
            command,
            config: RouteDslConfig::new(route_label),
        }
    }

    impl_route_dsl_methods!();

    pub fn parse<T>(self) -> ParsedCommandRouteBuilder<'a, T>
    where
        T: CommandArgs + Send + Sync + 'static,
    {
        ParsedCommandRouteBuilder {
            router: self.router,
            command: self.command,
            config: self.config,
            _parsed: std::marker::PhantomData,
        }
    }

    pub fn handle<H, Fut>(self, handler: H) -> &'a mut Router
    where
        H: Fn(BotContext, Update) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult> + Send + 'static,
    {
        let expected = self.command;
        let guards = Arc::new(self.config.guards);
        let handler = Arc::new(handler);
        self.router.has_command_routes = true;
        self.router.route_fallible_with_state(
            move |update, state| {
                extract_command_for_bot(update, state.command_target.as_deref())
                    .is_some_and(|command| command == expected)
            },
            move |context, update, _state| {
                let guards = Arc::clone(&guards);
                let handler = Arc::clone(&handler);
                async move {
                    run_route_guards(guards.as_ref(), context.clone(), update.clone()).await?;
                    handler(context, update).await
                }
            },
        )
    }

    pub fn handle_fallible<H, Fut>(self, handler: H) -> &'a mut Router
    where
        H: Fn(BotContext, Update) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult> + Send + 'static,
    {
        self.handle(handler)
    }

    pub fn handle_with_policy<H, Fut>(self, policy: ErrorPolicy, handler: H) -> &'a mut Router
    where
        H: Fn(BotContext, Update) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult> + Send + 'static,
    {
        let expected = self.command;
        let guards = Arc::new(self.config.guards);
        let handler = Arc::new(handler);
        self.router.has_command_routes = true;
        self.router.route_with_state(
            move |update, state| {
                extract_command_for_bot(update, state.command_target.as_deref())
                    .is_some_and(|command| command == expected)
            },
            move |context, update, _state| {
                let guards = Arc::clone(&guards);
                let handler = Arc::clone(&handler);
                let policy = policy.clone();
                async move {
                    if let Err(error) =
                        run_route_guards(guards.as_ref(), context.clone(), update.clone()).await
                    {
                        return context.resolve_handler_error(&update, error).await;
                    }
                    let context_for_policy = context.clone();
                    let update_for_policy = update.clone();
                    let outcome = handler(context, update).await;
                    resolve_handler_result_with_policy(
                        context_for_policy,
                        update_for_policy,
                        policy,
                        outcome,
                    )
                    .await
                }
            },
        )
    }
}

/// Chainable DSL for command routes that parse typed trailing arguments.
pub struct ParsedCommandRouteBuilder<'a, T> {
    router: &'a mut Router,
    command: String,
    config: RouteDslConfig,
    _parsed: std::marker::PhantomData<T>,
}

impl<'a, T> ParsedCommandRouteBuilder<'a, T>
where
    T: CommandArgs + Send + Sync + 'static,
{
    impl_route_dsl_methods!();

    pub fn handle<H, Fut>(self, handler: H) -> &'a mut Router
    where
        H: Fn(BotContext, Update, T) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult> + Send + 'static,
    {
        let expected = self.command;
        let guards = Arc::new(self.config.guards);
        let handler = Arc::new(handler);
        self.router.has_command_routes = true;
        self.router.route_fallible_with_state(
            move |update, state| {
                extract_command_for_bot(update, state.command_target.as_deref())
                    .is_some_and(|command| command == expected)
            },
            move |context, update, state| {
                let guards = Arc::clone(&guards);
                let handler = Arc::clone(&handler);
                async move {
                    run_route_guards(guards.as_ref(), context.clone(), update.clone()).await?;
                    let Some(command) =
                        extract_command_data_for_bot(&update, state.command_target.as_deref())
                    else {
                        return Err(HandlerError::internal(invalid_request(
                            "update does not contain a valid command",
                        )));
                    };
                    let parsed = T::parse(command.args_trimmed()).map_err(HandlerError::user)?;
                    handler(context, update, parsed).await
                }
            },
        )
    }

    pub fn handle_fallible<H, Fut>(self, handler: H) -> &'a mut Router
    where
        H: Fn(BotContext, Update, T) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult> + Send + 'static,
    {
        self.handle(handler)
    }

    pub fn handle_with_policy<H, Fut>(self, policy: ErrorPolicy, handler: H) -> &'a mut Router
    where
        H: Fn(BotContext, Update, T) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult> + Send + 'static,
    {
        let expected = self.command;
        let guards = Arc::new(self.config.guards);
        let handler = Arc::new(handler);
        self.router.has_command_routes = true;
        self.router.route_with_state(
            move |update, state| {
                extract_command_for_bot(update, state.command_target.as_deref())
                    .is_some_and(|command| command == expected)
            },
            move |context, update, state| {
                let guards = Arc::clone(&guards);
                let handler = Arc::clone(&handler);
                let policy = policy.clone();
                async move {
                    if let Err(error) =
                        run_route_guards(guards.as_ref(), context.clone(), update.clone()).await
                    {
                        return context.resolve_handler_error(&update, error).await;
                    }
                    let Some(command) =
                        extract_command_data_for_bot(&update, state.command_target.as_deref())
                    else {
                        return Err(invalid_request("update does not contain a valid command"));
                    };
                    let parsed = T::parse(command.args_trimmed()).map_err(HandlerError::user);
                    let parsed = match parsed {
                        Ok(parsed) => parsed,
                        Err(error) => return context.resolve_handler_error(&update, error).await,
                    };
                    let context_for_policy = context.clone();
                    let update_for_policy = update.clone();
                    let outcome = handler(context, update, parsed).await;
                    resolve_handler_result_with_policy(
                        context_for_policy,
                        update_for_policy,
                        policy,
                        outcome,
                    )
                    .await
                }
            },
        )
    }
}

/// Chainable DSL for typed slash-command routes.
pub struct TypedCommandRouteBuilder<'a, C> {
    router: &'a mut Router,
    config: RouteDslConfig,
    _command: std::marker::PhantomData<C>,
}

impl<'a, C> TypedCommandRouteBuilder<'a, C>
where
    C: BotCommands + Send + Sync + 'static,
{
    fn new(router: &'a mut Router) -> Self {
        Self {
            router,
            config: RouteDslConfig::new(format!("typed_command:{}", std::any::type_name::<C>())),
            _command: std::marker::PhantomData,
        }
    }

    impl_route_dsl_methods!();

    pub fn handle<H, Fut>(self, handler: H) -> &'a mut Router
    where
        H: Fn(BotContext, Update, C) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult> + Send + 'static,
    {
        let guards = Arc::new(self.config.guards);
        let handler = Arc::new(handler);
        self.router.has_command_routes = true;
        self.router.route_fallible_with_state(
            move |update, state| {
                parse_typed_command_for_bot::<C>(update, state.command_target.as_deref()).is_some()
            },
            move |context, update, state| {
                let guards = Arc::clone(&guards);
                let handler = Arc::clone(&handler);
                async move {
                    run_route_guards(guards.as_ref(), context.clone(), update.clone()).await?;
                    let Some(command) =
                        parse_typed_command_for_bot::<C>(&update, state.command_target.as_deref())
                    else {
                        return Err(HandlerError::internal(invalid_request(
                            "update does not contain a valid typed command",
                        )));
                    };
                    handler(context, update, command).await
                }
            },
        )
    }

    pub fn handle_input<H, Fut>(self, handler: H) -> &'a mut Router
    where
        H: Fn(BotContext, Update, TypedCommandInput<C>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult> + Send + 'static,
    {
        let guards = Arc::new(self.config.guards);
        let handler = Arc::new(handler);
        self.router.has_command_routes = true;
        self.router.route_fallible_with_state(
            move |update, state| {
                parse_typed_command_for_bot::<C>(update, state.command_target.as_deref()).is_some()
                    && extract_command_data_for_bot(update, state.command_target.as_deref())
                        .is_some()
            },
            move |context, update, state| {
                let guards = Arc::clone(&guards);
                let handler = Arc::clone(&handler);
                async move {
                    run_route_guards(guards.as_ref(), context.clone(), update.clone()).await?;
                    let Some(command) =
                        parse_typed_command_for_bot::<C>(&update, state.command_target.as_deref())
                    else {
                        return Err(HandlerError::internal(invalid_request(
                            "update does not contain a valid typed command",
                        )));
                    };
                    let Some(raw) =
                        extract_command_data_for_bot(&update, state.command_target.as_deref())
                    else {
                        return Err(HandlerError::internal(invalid_request(
                            "update does not contain a valid command",
                        )));
                    };
                    handler(context, update, TypedCommandInput { command, raw }).await
                }
            },
        )
    }

    pub fn handle_input_fallible<H, Fut>(self, handler: H) -> &'a mut Router
    where
        H: Fn(BotContext, Update, TypedCommandInput<C>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult> + Send + 'static,
    {
        self.handle_input(handler)
    }

    pub fn handle_fallible<H, Fut>(self, handler: H) -> &'a mut Router
    where
        H: Fn(BotContext, Update, C) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult> + Send + 'static,
    {
        self.handle(handler)
    }

    pub fn handle_with_policy<H, Fut>(self, policy: ErrorPolicy, handler: H) -> &'a mut Router
    where
        H: Fn(BotContext, Update, C) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult> + Send + 'static,
    {
        let guards = Arc::new(self.config.guards);
        let handler = Arc::new(handler);
        self.router.has_command_routes = true;
        self.router.route_with_state(
            move |update, state| {
                parse_typed_command_for_bot::<C>(update, state.command_target.as_deref()).is_some()
            },
            move |context, update, state| {
                let guards = Arc::clone(&guards);
                let handler = Arc::clone(&handler);
                let policy = policy.clone();
                async move {
                    if let Err(error) =
                        run_route_guards(guards.as_ref(), context.clone(), update.clone()).await
                    {
                        return context.resolve_handler_error(&update, error).await;
                    }
                    let Some(command) =
                        parse_typed_command_for_bot::<C>(&update, state.command_target.as_deref())
                    else {
                        return Err(invalid_request(
                            "update does not contain a valid typed command",
                        ));
                    };
                    let context_for_policy = context.clone();
                    let update_for_policy = update.clone();
                    let outcome = handler(context, update, command).await;
                    resolve_handler_result_with_policy(
                        context_for_policy,
                        update_for_policy,
                        policy,
                        outcome,
                    )
                    .await
                }
            },
        )
    }

    pub fn handle_input_with_policy<H, Fut>(self, policy: ErrorPolicy, handler: H) -> &'a mut Router
    where
        H: Fn(BotContext, Update, TypedCommandInput<C>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult> + Send + 'static,
    {
        let guards = Arc::new(self.config.guards);
        let handler = Arc::new(handler);
        self.router.has_command_routes = true;
        self.router.route_with_state(
            move |update, state| {
                parse_typed_command_for_bot::<C>(update, state.command_target.as_deref()).is_some()
                    && extract_command_data_for_bot(update, state.command_target.as_deref())
                        .is_some()
            },
            move |context, update, state| {
                let guards = Arc::clone(&guards);
                let handler = Arc::clone(&handler);
                let policy = policy.clone();
                async move {
                    if let Err(error) =
                        run_route_guards(guards.as_ref(), context.clone(), update.clone()).await
                    {
                        return context.resolve_handler_error(&update, error).await;
                    }
                    let Some(command) =
                        parse_typed_command_for_bot::<C>(&update, state.command_target.as_deref())
                    else {
                        return Err(invalid_request(
                            "update does not contain a valid typed command",
                        ));
                    };
                    let Some(raw) =
                        extract_command_data_for_bot(&update, state.command_target.as_deref())
                    else {
                        return Err(invalid_request("update does not contain a valid command"));
                    };
                    let context_for_policy = context.clone();
                    let update_for_policy = update.clone();
                    let outcome =
                        handler(context, update, TypedCommandInput { command, raw }).await;
                    resolve_handler_result_with_policy(
                        context_for_policy,
                        update_for_policy,
                        policy,
                        outcome,
                    )
                    .await
                }
            },
        )
    }
}

/// Chainable DSL for codec-aware callback routes.
pub struct CallbackRouteBuilder<'a, T, C = CallbackPayloadCodec> {
    router: &'a mut Router,
    config: RouteDslConfig,
    _payload: std::marker::PhantomData<T>,
    _codec: std::marker::PhantomData<C>,
}

pub type TypedCallbackRouteBuilder<'a, T> = CallbackRouteBuilder<'a, T, CallbackPayloadCodec>;
pub type CompactCallbackRouteBuilder<'a, T> = CallbackRouteBuilder<'a, T, CompactCallbackCodec>;

impl<'a, T, C> CallbackRouteBuilder<'a, T, C>
where
    T: Send + Sync + 'static,
    C: CallbackCodec<T>,
{
    fn new(router: &'a mut Router) -> Self {
        Self {
            router,
            config: RouteDslConfig::new(format!("callback:{}", std::any::type_name::<T>())),
            _payload: std::marker::PhantomData,
            _codec: std::marker::PhantomData,
        }
    }

    impl_route_dsl_methods!();

    pub fn handle<H, Fut>(self, handler: H) -> &'a mut Router
    where
        H: Fn(BotContext, Update, CodedCallbackInput<T, C>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult> + Send + 'static,
    {
        let guards = Arc::new(self.config.guards);
        let handler = Arc::new(handler);
        self.router.route_fallible(
            |update| CodedCallbackInput::<T, C>::extract(update).is_some(),
            move |context, update| {
                let guards = Arc::clone(&guards);
                let handler = Arc::clone(&handler);
                let payload = CodedCallbackInput::<T, C>::extract(&update);
                async move {
                    run_route_guards(guards.as_ref(), context.clone(), update.clone()).await?;
                    let Some(payload) = payload else {
                        return Err(HandlerError::internal(invalid_request(
                            "update does not contain a valid callback payload",
                        )));
                    };
                    handler(context, update, payload).await
                }
            },
        )
    }

    pub fn handle_fallible<H, Fut>(self, handler: H) -> &'a mut Router
    where
        H: Fn(BotContext, Update, CodedCallbackInput<T, C>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult> + Send + 'static,
    {
        self.handle(handler)
    }

    pub fn handle_with_policy<H, Fut>(self, policy: ErrorPolicy, handler: H) -> &'a mut Router
    where
        H: Fn(BotContext, Update, CodedCallbackInput<T, C>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult> + Send + 'static,
    {
        let guards = Arc::new(self.config.guards);
        let handler = Arc::new(handler);
        self.router.route_with_state(
            |update, _state| CodedCallbackInput::<T, C>::extract(update).is_some(),
            move |context, update, _state| {
                let guards = Arc::clone(&guards);
                let handler = Arc::clone(&handler);
                let policy = policy.clone();
                let payload = CodedCallbackInput::<T, C>::extract(&update);
                async move {
                    if let Err(error) =
                        run_route_guards(guards.as_ref(), context.clone(), update.clone()).await
                    {
                        return context.resolve_handler_error(&update, error).await;
                    }
                    let Some(payload) = payload else {
                        return Err(invalid_request(
                            "update does not contain a valid callback payload",
                        ));
                    };
                    let context_for_policy = context.clone();
                    let update_for_policy = update.clone();
                    let outcome = handler(context, update, payload).await;
                    resolve_handler_result_with_policy(
                        context_for_policy,
                        update_for_policy,
                        policy,
                        outcome,
                    )
                    .await
                }
            },
        )
    }
}

fn to_route_handler_fn<H, Fut>(handler: H) -> RouteHandlerFn
where
    H: Fn(BotContext, Update, DispatchState) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<()>> + Send + 'static,
{
    Arc::new(move |context, update, state| Box::pin(handler(context, update, state)))
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
    parse_command_text_for_bot(text, None)
}

/// Parses a slash command from raw message text with optional bot-username targeting.
///
/// When a command contains `@botname`, it is accepted only if `bot_username`
/// is provided and matches case-insensitively.
pub fn parse_command_text_for_bot(text: &str, bot_username: Option<&str>) -> Option<CommandData> {
    let token = text.split_whitespace().next()?;
    let command = token.strip_prefix('/')?;

    let (name, mention) = match command.split_once('@') {
        Some((name, mention)) => (name, Some(mention)),
        None => (command, None),
    };

    if name.is_empty() {
        return None;
    }

    let mention = match mention {
        Some(value) => Some(normalize_bot_username(value)?),
        None => None,
    };

    let args = text[token.len()..].trim().to_owned();
    let command = CommandData {
        name: name.to_owned(),
        mention,
        args,
    };
    if command.is_addressed_to(bot_username) {
        Some(command)
    } else {
        None
    }
}

fn normalize_bot_username(value: &str) -> Option<String> {
    let normalized = value.trim().trim_start_matches('@').trim();
    if normalized.is_empty() {
        None
    } else {
        Some(normalized.to_owned())
    }
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

/// Returns canonical chat extracted from the update.
pub fn extract_chat(update: &Update) -> Option<&Chat> {
    extract_message(update).map(Message::chat)
}

/// Returns the acting Telegram user for the update when available.
pub fn extract_user(update: &Update) -> Option<&User> {
    if let Some(query) = update.callback_query.as_ref() {
        return Some(&query.from);
    }
    if let Some(message) = update.message.as_ref() {
        return message.from_user();
    }
    if let Some(message) = update.edited_message.as_ref() {
        return message.from_user();
    }
    if let Some(message) = update.channel_post.as_ref() {
        return message.from_user();
    }
    if let Some(message) = update.edited_channel_post.as_ref() {
        return message.from_user();
    }
    None
}

/// Returns acting Telegram user id for the update when available.
pub fn extract_user_id(update: &Update) -> Option<i64> {
    Some(extract_user(update)?.id.0)
}

/// Returns primary kind of extracted message.
pub fn extract_message_kind(update: &Update) -> Option<MessageKind> {
    Some(extract_message(update)?.kind())
}

/// Returns plain text from extracted message when available.
pub fn extract_text(update: &Update) -> Option<&str> {
    extract_message(update)?.text.as_deref()
}

/// Returns Mini App payload from extracted message when available.
pub fn extract_web_app_data(update: &Update) -> Option<&WebAppData> {
    update.web_app_data()
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

/// Returns a decoded typed callback payload from update callback data.
pub fn extract_typed_callback<T>(update: &Update) -> Option<T>
where
    T: CallbackPayload,
{
    extract_callback_with_codec::<T, CallbackPayloadCodec>(update)
}

/// Returns a decoded callback payload from update callback data with an explicit codec.
pub fn extract_callback_with_codec<T, C>(update: &Update) -> Option<T>
where
    C: CallbackCodec<T>,
{
    let payload = extract_callback_data(update)?;
    C::decode_callback_data(payload).ok()
}

/// Returns a decoded compact callback payload from update callback data.
pub fn extract_compact_callback<T>(update: &Update) -> Option<T>
where
    T: CompactCallbackPayload,
{
    extract_callback_with_codec::<T, CompactCallbackCodec>(update)
}

/// Returns command name from canonical message text.
///
/// Mentioned commands (for example, `/start@OtherBot`) are ignored by default.
pub fn extract_command(update: &Update) -> Option<&str> {
    extract_command_for_bot(update, None)
}

/// Returns command name from canonical message text, filtered by target bot username.
pub fn extract_command_for_bot<'a>(
    update: &'a Update,
    bot_username: Option<&str>,
) -> Option<&'a str> {
    let text = extract_text(update)?;
    let token = text.split_whitespace().next()?;
    let command = token.strip_prefix('/')?;
    let (name, mention) = match command.split_once('@') {
        Some((name, mention)) => (name, Some(mention)),
        None => (command, None),
    };
    if name.is_empty() {
        return None;
    }

    let mention = mention.and_then(normalize_bot_username);
    let command = CommandData {
        name: name.to_owned(),
        mention,
        args: text[token.len()..].trim().to_owned(),
    };
    if command.is_addressed_to(bot_username) {
        Some(name)
    } else {
        None
    }
}

/// Returns command arguments as a trimmed string slice.
pub fn extract_command_args(update: &Update) -> Option<&str> {
    extract_command_args_for_bot(update, None)
}

/// Returns command arguments as a trimmed string slice, filtered by target bot username.
pub fn extract_command_args_for_bot<'a>(
    update: &'a Update,
    bot_username: Option<&str>,
) -> Option<&'a str> {
    let text = extract_text(update)?;
    let token = text.split_whitespace().next()?;
    let command = token.strip_prefix('/')?;
    let mention = command
        .split_once('@')
        .and_then(|(_, mention)| normalize_bot_username(mention));
    let name = command.split_once('@').map_or(command, |(name, _)| name);
    if name.is_empty() {
        return None;
    }

    let command_data = CommandData {
        name: name.to_owned(),
        mention,
        args: text[token.len()..].trim().to_owned(),
    };
    if command_data.is_addressed_to(bot_username) {
        Some(text[token.len()..].trim())
    } else {
        None
    }
}

/// Returns parsed command with owned command name and args.
pub fn extract_command_data(update: &Update) -> Option<CommandData> {
    extract_command_data_for_bot(update, None)
}

/// Returns parsed command with owned command name and args, filtered by target bot username.
pub fn extract_command_data_for_bot(
    update: &Update,
    bot_username: Option<&str>,
) -> Option<CommandData> {
    parse_command_text_for_bot(extract_text(update)?, bot_username)
}

/// Parses typed command from incoming update using a `BotCommands` implementation.
pub fn parse_typed_command<C: BotCommands>(update: &Update) -> Option<C> {
    let command = extract_command_data_for_bot(update, None)?;
    C::parse(&command.name, command.args_trimmed())
}

/// Parses typed command from incoming update for an explicit bot username target.
pub fn parse_typed_command_for_bot<C: BotCommands>(
    update: &Update,
    bot_username: Option<&str>,
) -> Option<C> {
    let command = extract_command_data_for_bot(update, bot_username)?;
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
    fn chat(&self) -> Option<&Chat> {
        self.message().map(Message::chat)
    }
    fn message_kind(&self) -> Option<MessageKind> {
        self.message().map(Message::kind)
    }
    fn update_kind(&self) -> UpdateKind {
        UpdateKind::Unknown
    }
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
    fn typed_callback<T>(&self) -> Option<T>
    where
        T: CallbackPayload;
    fn typed_callback_with_codec<T, C>(&self) -> Option<T>
    where
        C: CallbackCodec<T>;
    fn compact_callback<T>(&self) -> Option<T>
    where
        T: CompactCallbackPayload;
    fn command(&self) -> Option<&str>;
    fn command_args(&self) -> Option<&str>;
    fn command_data(&self) -> Option<CommandData>;
    fn typed_command<C>(&self) -> Option<C>
    where
        C: BotCommands;
    fn user(&self) -> Option<&User>;
    fn user_id(&self) -> Option<i64> {
        self.user().map(|user| user.id.0)
    }
    fn chat_id(&self) -> Option<i64>;
}

impl UpdateExt for Update {
    fn message(&self) -> Option<&Message> {
        extract_message(self)
    }

    fn chat(&self) -> Option<&Chat> {
        extract_chat(self)
    }

    fn message_kind(&self) -> Option<MessageKind> {
        extract_message_kind(self)
    }

    fn update_kind(&self) -> UpdateKind {
        Update::kind(self)
    }

    fn text(&self) -> Option<&str> {
        extract_text(self)
    }

    fn web_app_data(&self) -> Option<&WebAppData> {
        Update::web_app_data(self)
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

    fn typed_callback<T>(&self) -> Option<T>
    where
        T: CallbackPayload,
    {
        extract_typed_callback(self)
    }

    fn typed_callback_with_codec<T, C>(&self) -> Option<T>
    where
        C: CallbackCodec<T>,
    {
        extract_callback_with_codec::<T, C>(self)
    }

    fn compact_callback<T>(&self) -> Option<T>
    where
        T: CompactCallbackPayload,
    {
        extract_compact_callback(self)
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

    fn user(&self) -> Option<&User> {
        extract_user(self)
    }

    fn chat_id(&self) -> Option<i64> {
        update_chat_id(self)
    }
}

/// Tries to extract a canonical chat id from an incoming update.
pub fn update_chat_id(update: &Update) -> Option<i64> {
    extract_message(update).map(|message| message.chat.id)
}

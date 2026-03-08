use super::extractors::*;
use super::*;

mod builders;

pub use builders::*;

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

/// Request-state key for the acting member cache.
pub const CURRENT_ACTOR_CHAT_MEMBER: RequestStateKey<ChatMember> =
    RequestStateKey::new("current_actor_chat_member");

/// Request-state key for the bot member cache.
pub const CURRENT_BOT_CHAT_MEMBER: RequestStateKey<ChatMember> =
    RequestStateKey::new("current_bot_chat_member");

const CURRENT_BOT_USER: RequestStateKey<User> = RequestStateKey::new("current_bot_user");

#[derive(Clone)]
enum RouteResolution {
    Default,
    Policy(ErrorPolicy),
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
            let _ = context.app().reply_text(&update, message).await?;
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
        Err(HandlerError::Rejected(rejection)) => {
            context
                .resolve_handler_error(&update, HandlerError::rejected(rejection))
                .await
        }
        Err(HandlerError::Internal(error)) => {
            resolve_error_with_policy(context, update, policy, error).await
        }
    }
}

async fn resolve_route_result(
    context: BotContext,
    update: Update,
    resolution: RouteResolution,
    outcome: HandlerResult,
) -> Result<()> {
    match resolution {
        RouteResolution::Default => resolve_handler_result(context, update, outcome).await,
        RouteResolution::Policy(policy) => {
            resolve_handler_result_with_policy(context, update, policy, outcome).await
        }
    }
}

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

fn require_group_chat(update: &Update) -> std::result::Result<(), RouteRejection> {
    let Some(chat) = extract_chat(update) else {
        return Err(RouteRejection::GroupOnly);
    };
    if chat.is_group_chat() {
        Ok(())
    } else {
        Err(RouteRejection::GroupOnly)
    }
}

async fn current_actor_chat_member(
    context: &BotContext,
    update: &Update,
) -> std::result::Result<Arc<ChatMember>, HandlerError> {
    if let Some(member) = context
        .request_state()
        .slot(CURRENT_ACTOR_CHAT_MEMBER)
        .read()
    {
        return Ok(member);
    }

    let Some(chat_id) = update_chat_id(update) else {
        return Err(RouteRejection::ChatContextRequired.into());
    };
    let Some(user) = extract_actor(update) else {
        return Err(RouteRejection::ActorRequired.into());
    };

    let member = fetch_chat_member(context, chat_id, user.id).await?;
    let shared = Arc::new(member);
    let _ = context
        .request_state()
        .slot(CURRENT_ACTOR_CHAT_MEMBER)
        .set_shared(shared.clone());
    Ok(shared)
}

async fn current_bot_chat_member(
    context: &BotContext,
    update: &Update,
) -> std::result::Result<Arc<ChatMember>, HandlerError> {
    if let Some(member) = context.request_state().slot(CURRENT_BOT_CHAT_MEMBER).read() {
        return Ok(member);
    }

    let Some(chat_id) = update_chat_id(update) else {
        return Err(RouteRejection::ChatContextRequired.into());
    };

    let bot_user = if let Some(user) = context.request_state().slot(CURRENT_BOT_USER).read() {
        user
    } else {
        let me = context.bot().get_me().await?;
        let shared = Arc::new(me);
        let _ = context
            .request_state()
            .slot(CURRENT_BOT_USER)
            .set_shared(shared.clone());
        shared
    };

    let member = fetch_chat_member(context, chat_id, bot_user.id).await?;
    let shared = Arc::new(member);
    let _ = context
        .request_state()
        .slot(CURRENT_BOT_CHAT_MEMBER)
        .set_shared(shared.clone());
    Ok(shared)
}

fn missing_capabilities(
    member: &ChatMember,
    capabilities: &[ChatAdministratorCapability],
) -> Vec<ChatAdministratorCapability> {
    capabilities
        .iter()
        .copied()
        .filter(|capability| !member.has_capability(*capability))
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
    Actor,
    Subject,
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
    route_label: Arc<str>,
}

impl RouteDslConfig {
    fn new(route_label: impl Into<String>) -> Self {
        Self {
            guards: Vec::new(),
            route_label: Arc::<str>::from(route_label.into()),
        }
    }

    fn push_guard<G>(&mut self, guard: G)
    where
        G: for<'a> Fn(&'a BotContext, &'a Update) -> GuardFuture<'a> + Send + Sync + 'static,
    {
        self.guards.push(Arc::new(guard));
    }

    fn group_only(&mut self) {
        self.push_guard(|_context, update| {
            Box::pin(std::future::ready(
                require_group_chat(update).map_err(HandlerError::from),
            ))
        });
    }

    fn supergroup_only(&mut self) {
        self.push_guard(|_context, update| {
            Box::pin(std::future::ready(match extract_chat(update) {
                Some(chat) if chat.is_supergroup() => Ok(()),
                _ => Err(RouteRejection::SupergroupOnly.into()),
            }))
        });
    }

    fn admin_only(&mut self) {
        self.push_guard(|context, update| {
            Box::pin(async move {
                require_group_chat(update)?;
                let member = current_actor_chat_member(context, update).await?;
                if member.is_admin() {
                    Ok(())
                } else {
                    Err(RouteRejection::AdminOnly.into())
                }
            })
        });
    }

    fn owner_only(&mut self) {
        self.push_guard(|context, update| {
            Box::pin(async move {
                require_group_chat(update)?;
                let member = current_actor_chat_member(context, update).await?;
                if member.is_owner() {
                    Ok(())
                } else {
                    Err(RouteRejection::OwnerOnly.into())
                }
            })
        });
    }

    fn require_capabilities(&mut self, capabilities: Vec<ChatAdministratorCapability>) {
        self.push_guard(move |context, update| {
            let capabilities = capabilities.clone();
            Box::pin(async move {
                require_group_chat(update)?;
                let member = current_actor_chat_member(context, update).await?;
                let missing = missing_capabilities(member.as_ref(), capabilities.as_slice());
                if missing.is_empty() {
                    Ok(())
                } else {
                    Err(RouteRejection::MissingActorCapabilities(missing).into())
                }
            })
        });
    }

    fn bot_can(&mut self, capabilities: Vec<ChatAdministratorCapability>) {
        self.push_guard(move |context, update| {
            let capabilities = capabilities.clone();
            Box::pin(async move {
                require_group_chat(update)?;
                let member = current_bot_chat_member(context, update).await?;
                let missing = missing_capabilities(member.as_ref(), capabilities.as_slice());
                if missing.is_empty() {
                    Ok(())
                } else {
                    Err(RouteRejection::MissingBotCapabilities(missing).into())
                }
            })
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
            Box::pin(std::future::ready(
                match throttle_key(scope, update, route_label.as_ref()) {
                    Ok(key) => {
                        if store.allow(key, limit, window) {
                            Ok(())
                        } else {
                            Err(RouteRejection::Throttled.into())
                        }
                    }
                    Err(rejection) => Err(rejection.into()),
                },
            ))
        });
    }
}

fn throttle_key(
    scope: ThrottleScope,
    update: &Update,
    route_label: &str,
) -> std::result::Result<String, RouteRejection> {
    match scope {
        ThrottleScope::Actor => {
            let Some(actor_id) = extract_actor_id(update) else {
                return Err(RouteRejection::ActorRequired);
            };
            Ok(format!("{route_label}:actor:{actor_id}"))
        }
        ThrottleScope::Subject => {
            let Some(subject_id) = extract_subject_id(update) else {
                return Err(RouteRejection::SubjectRequired);
            };
            Ok(format!("{route_label}:subject:{subject_id}"))
        }
        ThrottleScope::Chat => {
            let Some(chat_id) = update_chat_id(update) else {
                return Err(RouteRejection::ChatContextRequired);
            };
            Ok(format!("{route_label}:chat:{chat_id}"))
        }
        ThrottleScope::Command => Ok(format!("{route_label}:command")),
    }
}

async fn run_route_guards(
    guards: &[GuardFn],
    context: &BotContext,
    update: &Update,
) -> HandlerResult {
    for guard in guards {
        guard(context, update).await?;
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

async fn evaluate_route_pipeline<T, I, H, Fut>(
    context: BotContext,
    update: Update,
    guards: &[GuardFn],
    input: I,
    handler: H,
) -> HandlerResult
where
    I: FnOnce(&Update) -> std::result::Result<Option<T>, HandlerError>,
    H: FnOnce(BotContext, Update, T) -> Fut,
    Fut: Future<Output = HandlerResult>,
{
    run_route_guards(guards, &context, &update).await?;
    let Some(input) = input(&update)? else {
        return Ok(());
    };
    handler(context, update, input).await
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

    pub fn chat_join_request_route(&mut self) -> ExtractedRouteBuilder<'_, ChatJoinRequestInput> {
        ExtractedRouteBuilder::new(self, "chat_join_request")
    }

    pub fn chat_member_route(&mut self) -> ExtractedRouteBuilder<'_, ChatMemberUpdatedInput> {
        ExtractedRouteBuilder::new(self, "chat_member")
    }

    pub fn my_chat_member_route(&mut self) -> ExtractedRouteBuilder<'_, MyChatMemberUpdatedInput> {
        ExtractedRouteBuilder::new(self, "my_chat_member")
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

    pub(crate) async fn prepare_for_updates(
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
    pub fn command_target(self, bot_username: impl Into<String>) -> Result<Self> {
        let _ = self.set_command_target(bot_username)?;
        Ok(self)
    }

    /// Sets command target bot username used by command routes.
    pub fn set_command_target(&self, bot_username: impl Into<String>) -> Result<&Self> {
        let raw = bot_username.into();
        let bot_username = normalize_bot_username(raw.as_str())
            .ok_or_else(|| invalid_request("command target bot username cannot be empty"))?;
        let _ = self.set_command_target_config(Some(bot_username), false);
        Ok(self)
    }

    /// Clears command target state and re-enables lazy auto-resolution.
    pub fn clear_command_target(&self) -> &Self {
        let _ = self.set_command_target_config(None, true);
        self
    }

    /// Disables lazy auto-resolution for mentioned commands.
    pub fn disable_auto_command_target(&self) -> &Self {
        let username = self.command_target_username();
        let _ = self.set_command_target_config(username, false);
        self
    }

    /// Enables lazy auto-resolution for mentioned commands.
    pub fn enable_auto_command_target(&self) -> &Self {
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

    fn route_handler_with_state<P, H, Fut>(
        &mut self,
        predicate: P,
        resolution: RouteResolution,
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
            let resolution = resolution.clone();
            async move {
                let context_for_resolution = context.clone();
                let update_for_resolution = update.clone();
                let outcome = handler(context, update, state).await;
                resolve_route_result(
                    context_for_resolution,
                    update_for_resolution,
                    resolution,
                    outcome,
                )
                .await
            }
        })
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
        self.route_handler_with_state(predicate, RouteResolution::Policy(policy), handler)
    }

    fn route_fallible_with_state<P, H, Fut>(&mut self, predicate: P, handler: H) -> &mut Self
    where
        P: Fn(&Update, &DispatchState) -> bool + Send + Sync + 'static,
        H: Fn(BotContext, Update, DispatchState) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult> + Send + 'static,
    {
        self.route_handler_with_state(predicate, RouteResolution::Default, handler)
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
            Box::pin(async move {
                let context_for_resolution = context.clone();
                let update_for_resolution = update.clone();
                let outcome = handler(context, update).await;
                resolve_route_result(
                    context_for_resolution,
                    update_for_resolution,
                    RouteResolution::Default,
                    outcome,
                )
                .await
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
    pub fn dispatch(
        &self,
        context: BotContext,
        update: Update,
    ) -> Pin<Box<dyn Future<Output = Result<bool>> + Send + '_>> {
        Box::pin(async move {
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
        })
    }

    /// Prepares runtime routing state for the given update and immediately dispatches it.
    pub fn dispatch_prepared(
        &self,
        context: BotContext,
        update: Update,
    ) -> Pin<Box<dyn Future<Output = Result<bool>> + Send + '_>> {
        Box::pin(async move {
            self.prepare_for_update(context.client(), &update).await?;
            self.dispatch(context, update).await
        })
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

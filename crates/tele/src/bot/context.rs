use super::*;

type RequestStateValue = Arc<dyn Any + Send + Sync>;
type ContextExtensions = Arc<StdRwLock<HashMap<RequestStateSlotId, RequestStateValue>>>;

const DEFAULT_REQUEST_STATE_SLOT: &str = "";

fn downcast_request_state<T>(value: RequestStateValue) -> Option<Arc<T>>
where
    T: Send + Sync + 'static,
{
    Arc::downcast::<T>(value).ok()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
struct RequestStateSlotId {
    type_id: TypeId,
    slot: &'static str,
}

fn request_state_slot_id<T>(key: RequestStateKey<T>) -> RequestStateSlotId
where
    T: Send + Sync + 'static,
{
    RequestStateSlotId {
        type_id: TypeId::of::<T>(),
        slot: key.slot,
    }
}

/// Typed request-state slot descriptor.
#[derive(Debug, Eq, PartialEq, Hash)]
pub struct RequestStateKey<T> {
    slot: &'static str,
    _marker: std::marker::PhantomData<fn() -> T>,
}

impl<T> Clone for RequestStateKey<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for RequestStateKey<T> {}

impl<T> RequestStateKey<T> {
    pub const fn new(slot: &'static str) -> Self {
        Self {
            slot,
            _marker: std::marker::PhantomData,
        }
    }

    pub const fn slot(self) -> &'static str {
        self.slot
    }
}

/// Borrowed access to one typed request-state slot.
pub struct RequestStateSlot<'a, T> {
    state: &'a RequestState,
    key: RequestStateKey<T>,
}

impl<'a, T> RequestStateSlot<'a, T>
where
    T: Send + Sync + 'static,
{
    pub fn set(&self, value: T) -> Option<Arc<T>> {
        self.set_shared(Arc::new(value))
    }

    pub fn set_shared(&self, value: Arc<T>) -> Option<Arc<T>> {
        let previous = self
            .state
            .inner
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(request_state_slot_id(self.key), value);
        previous.and_then(downcast_request_state::<T>)
    }

    pub fn read_or_init_with(&self, init: impl FnOnce() -> T) -> Arc<T> {
        if let Some(value) = self.read() {
            return value;
        }

        let mut state = self
            .state
            .inner
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(value) = state
            .get(&request_state_slot_id(self.key))
            .cloned()
            .and_then(downcast_request_state::<T>)
        {
            return value;
        }

        let value = Arc::new(init());
        let _ = state.insert(request_state_slot_id(self.key), value.clone());
        value
    }

    pub fn read(&self) -> Option<Arc<T>> {
        self.state
            .inner
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(&request_state_slot_id(self.key))
            .cloned()
            .and_then(downcast_request_state::<T>)
    }

    pub fn cloned(&self) -> Option<T>
    where
        T: Clone,
    {
        self.read().map(|value| value.as_ref().clone())
    }

    pub fn with<R>(&self, map: impl FnOnce(&T) -> R) -> Option<R> {
        self.read().map(|value| map(value.as_ref()))
    }

    pub fn contains(&self) -> bool {
        self.state
            .inner
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .contains_key(&request_state_slot_id(self.key))
    }

    pub fn remove(&self) -> Option<Arc<T>> {
        self.state
            .inner
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .remove(&request_state_slot_id(self.key))
            .and_then(downcast_request_state::<T>)
    }
}

/// Structured route-level rejection reason.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RouteRejection {
    Message(String),
    GroupOnly,
    SupergroupOnly,
    AdminOnly,
    OwnerOnly,
    ActorRequired,
    SubjectRequired,
    ChatContextRequired,
    MissingActorCapabilities(Vec<ChatAdministratorCapability>),
    MissingBotCapabilities(Vec<ChatAdministratorCapability>),
    Throttled,
}

impl RouteRejection {
    pub fn message(&self) -> String {
        match self {
            Self::Message(message) => message.clone(),
            Self::GroupOnly => "this route is only available in group chats".to_owned(),
            Self::SupergroupOnly => "this route is only available in supergroups".to_owned(),
            Self::AdminOnly => "chat administrators only".to_owned(),
            Self::OwnerOnly => "chat owner only".to_owned(),
            Self::ActorRequired => "this route requires an actor user".to_owned(),
            Self::SubjectRequired => "this route requires a subject user".to_owned(),
            Self::ChatContextRequired => "this route requires a chat context".to_owned(),
            Self::MissingActorCapabilities(missing) => format!(
                "missing required capabilities: {}",
                missing
                    .iter()
                    .map(ChatAdministratorCapability::as_str)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Self::MissingBotCapabilities(missing) => format!(
                "bot is missing required capabilities: {}",
                missing
                    .iter()
                    .map(ChatAdministratorCapability::as_str)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Self::Throttled => "too many matching requests, please retry shortly".to_owned(),
        }
    }

    pub fn custom(message: impl Into<String>) -> Self {
        Self::Message(message.into())
    }
}

/// Handler error type that separates route rejections from internal failures.
#[derive(Debug)]
pub enum HandlerError {
    Rejected(RouteRejection),
    Internal(Error),
}

impl HandlerError {
    pub fn user(message: impl Into<String>) -> Self {
        Self::Rejected(RouteRejection::custom(message))
    }

    pub fn rejected(rejection: RouteRejection) -> Self {
        Self::Rejected(rejection)
    }

    pub fn internal(error: Error) -> Self {
        Self::Internal(error)
    }
}

impl From<RouteRejection> for HandlerError {
    fn from(value: RouteRejection) -> Self {
        Self::Rejected(value)
    }
}

impl From<Error> for HandlerError {
    fn from(value: Error) -> Self {
        Self::Internal(value)
    }
}

/// Ergonomic result type for bot handlers.
pub type HandlerResult = std::result::Result<(), HandlerError>;

/// Typed request-scoped state store shared across middlewares and handlers for one dispatch.
#[derive(Clone, Default)]
pub struct RequestState {
    inner: ContextExtensions,
}

impl RequestState {
    pub fn slot<T>(&self, key: RequestStateKey<T>) -> RequestStateSlot<'_, T>
    where
        T: Send + Sync + 'static,
    {
        RequestStateSlot { state: self, key }
    }

    pub fn insert<T>(&self, value: T) -> Option<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        self.slot(RequestStateKey::<T>::new(DEFAULT_REQUEST_STATE_SLOT))
            .set(value)
    }

    pub fn insert_shared<T>(&self, value: Arc<T>) -> Option<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        self.slot(RequestStateKey::<T>::new(DEFAULT_REQUEST_STATE_SLOT))
            .set_shared(value)
    }

    pub fn get_or_insert_with<T>(&self, init: impl FnOnce() -> T) -> Arc<T>
    where
        T: Send + Sync + 'static,
    {
        self.slot(RequestStateKey::<T>::new(DEFAULT_REQUEST_STATE_SLOT))
            .read_or_init_with(init)
    }

    pub fn get<T>(&self) -> Option<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        self.slot(RequestStateKey::<T>::new(DEFAULT_REQUEST_STATE_SLOT))
            .read()
    }

    pub fn with<T, R>(&self, map: impl FnOnce(&T) -> R) -> Option<R>
    where
        T: Send + Sync + 'static,
    {
        self.slot(RequestStateKey::<T>::new(DEFAULT_REQUEST_STATE_SLOT))
            .with(map)
    }

    pub fn contains<T>(&self) -> bool
    where
        T: Send + Sync + 'static,
    {
        self.slot(RequestStateKey::<T>::new(DEFAULT_REQUEST_STATE_SLOT))
            .contains()
    }

    pub fn remove<T>(&self) -> Option<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        self.slot(RequestStateKey::<T>::new(DEFAULT_REQUEST_STATE_SLOT))
            .remove()
    }

    pub fn clear(&self) {
        self.inner
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clear();
    }
}

/// Request-scoped dispatch context passed to handlers and middlewares.
#[derive(Clone)]
pub struct BotContext {
    client: Client,
    request_state: RequestState,
}

impl BotContext {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            request_state: RequestState::default(),
        }
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Returns a control-plane facade for startup/bootstrap helpers.
    pub fn control(&self) -> BotControl {
        BotControl::new(self.client.clone())
    }

    pub fn request_state(&self) -> &RequestState {
        &self.request_state
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
        let chat_id = crate::client::reply_chat_id(update)?;
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

    /// Converts high-level handler error into transportable SDK result.
    pub async fn resolve_handler_error(&self, update: &Update, error: HandlerError) -> Result<()> {
        match error {
            HandlerError::Rejected(rejection) => {
                let _ = self.reply_text(update, rejection.message()).await?;
                Ok(())
            }
            HandlerError::Internal(error) => Err(error),
        }
    }
}

/// Control-plane helper facade built from a client.
#[derive(Clone)]
pub struct BotControl {
    client: Client,
}

impl BotControl {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Spawns a reliable outbox worker for send-side retry, throttling and idempotency.
    pub fn spawn_outbox(&self, config: OutboxConfig) -> BotOutbox {
        BotOutbox::spawn(self.client.clone(), config)
    }

    /// Registers typed command definitions from a `BotCommands` enum.
    pub async fn set_typed_commands<C>(&self) -> Result<bool>
    where
        C: crate::bot::BotCommands,
    {
        let request = SetMyCommandsRequest::new(crate::bot::command_definitions::<C>())?;
        self.client.bot().set_my_commands(&request).await
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
        C: crate::bot::BotCommands,
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
        C: crate::bot::BotCommands,
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
        router: &crate::bot::Router,
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
        router: &crate::bot::Router,
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
        self.client
            .advanced()
            .answer_web_app_query_typed(&request)
            .await
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
}

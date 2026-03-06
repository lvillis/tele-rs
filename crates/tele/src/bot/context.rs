use super::*;

type ContextExtensionValue = Arc<dyn Any + Send + Sync>;
type ContextExtensions = Arc<StdRwLock<HashMap<TypeId, ContextExtensionValue>>>;

fn downcast_extension<T>(value: ContextExtensionValue) -> Option<Arc<T>>
where
    T: Send + Sync + 'static,
{
    Arc::downcast::<T>(value).ok()
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

/// Request-scoped dispatch context passed to handlers and middlewares.
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

    /// Returns a control-plane facade for startup/bootstrap helpers.
    pub fn control(&self) -> BotControl {
        BotControl::new(self.client.clone())
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
        let Some(chat_id) = crate::bot::update_chat_id(update) else {
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

use super::*;

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

    /// Reads the default menu button configuration.
    pub async fn get_menu_button(&self) -> Result<MenuButton> {
        self.client.ergo().get_menu_button().await
    }

    /// Reads the menu button configuration for a specific chat.
    pub async fn get_chat_menu_button(&self, chat_id: i64) -> Result<MenuButton> {
        self.client.ergo().get_chat_menu_button(chat_id).await
    }

    /// Applies a menu button configuration.
    pub async fn set_menu_button(&self, config: impl Into<MenuButtonConfig>) -> Result<bool> {
        self.client.ergo().set_menu_button(config).await
    }

    /// Applies a menu button for a specific chat.
    pub async fn set_chat_menu_button(
        &self,
        chat_id: i64,
        menu_button: impl Into<MenuButton>,
    ) -> Result<bool> {
        self.client
            .ergo()
            .set_chat_menu_button(chat_id, menu_button)
            .await
    }

    /// Restores the default menu button.
    pub async fn set_default_menu_button(&self) -> Result<bool> {
        self.client.ergo().set_default_menu_button().await
    }

    /// Restores the default menu button for a specific chat.
    pub async fn set_chat_default_menu_button(&self, chat_id: i64) -> Result<bool> {
        self.client
            .ergo()
            .set_chat_default_menu_button(chat_id)
            .await
    }

    /// Sets the commands menu button.
    pub async fn set_commands_menu_button(&self) -> Result<bool> {
        self.client.ergo().set_commands_menu_button().await
    }

    /// Sets the commands menu button for a specific chat.
    pub async fn set_chat_commands_menu_button(&self, chat_id: i64) -> Result<bool> {
        self.client
            .ergo()
            .set_chat_commands_menu_button(chat_id)
            .await
    }

    /// Sets a Web App menu button.
    pub async fn set_web_app_menu_button(
        &self,
        text: impl Into<String>,
        web_app: impl Into<WebAppInfo>,
    ) -> Result<bool> {
        self.client
            .ergo()
            .set_web_app_menu_button(text, web_app)
            .await
    }

    /// Sets a Web App menu button for a specific chat.
    pub async fn set_chat_web_app_menu_button(
        &self,
        chat_id: i64,
        text: impl Into<String>,
        web_app: impl Into<WebAppInfo>,
    ) -> Result<bool> {
        self.client
            .ergo()
            .set_chat_web_app_menu_button(chat_id, text, web_app)
            .await
    }

    /// Applies a menu button configuration with retry/backoff.
    pub async fn set_menu_button_with_retry(
        &self,
        config: impl Into<MenuButtonConfig>,
        policy: BootstrapRetryPolicy,
    ) -> Result<bool> {
        self.client
            .ergo()
            .set_menu_button_with_retry(config, policy)
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

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

    /// Reads the default menu button configuration.
    pub fn get_menu_button(&self) -> Result<MenuButton> {
        self.client.advanced().get_chat_menu_button_typed(
            &crate::types::advanced::AdvancedGetChatMenuButtonRequest::new(),
        )
    }

    /// Reads the menu button configuration for a specific chat.
    pub fn get_chat_menu_button(&self, chat_id: i64) -> Result<MenuButton> {
        let request = crate::types::advanced::AdvancedGetChatMenuButtonRequest {
            chat_id: Some(chat_id),
        };
        self.client.advanced().get_chat_menu_button_typed(&request)
    }

    /// Applies a menu button configuration without dropping into `advanced()`.
    pub fn set_menu_button(&self, config: impl Into<MenuButtonConfig>) -> Result<bool> {
        let config = config.into();
        let request: crate::types::advanced::AdvancedSetChatMenuButtonRequest = (&config).into();
        self.client.advanced().set_chat_menu_button_typed(&request)
    }

    /// Applies a menu button for a specific chat.
    pub fn set_chat_menu_button(
        &self,
        chat_id: i64,
        menu_button: impl Into<MenuButton>,
    ) -> Result<bool> {
        self.set_menu_button(MenuButtonConfig::for_chat(chat_id, menu_button))
    }

    /// Restores the default menu button.
    pub fn set_default_menu_button(&self) -> Result<bool> {
        self.set_menu_button(MenuButtonConfig::default_button())
    }

    /// Restores the default menu button for a specific chat.
    pub fn set_chat_default_menu_button(&self, chat_id: i64) -> Result<bool> {
        self.set_menu_button(MenuButtonConfig::for_chat_default(chat_id))
    }

    /// Sets the commands menu button.
    pub fn set_commands_menu_button(&self) -> Result<bool> {
        self.set_menu_button(MenuButtonConfig::commands())
    }

    /// Sets the commands menu button for a specific chat.
    pub fn set_chat_commands_menu_button(&self, chat_id: i64) -> Result<bool> {
        self.set_menu_button(MenuButtonConfig::for_chat_commands(chat_id))
    }

    /// Sets a Web App menu button.
    pub fn set_web_app_menu_button(
        &self,
        text: impl Into<String>,
        web_app: impl Into<WebAppInfo>,
    ) -> Result<bool> {
        self.set_menu_button(MenuButtonConfig::web_app(text, web_app))
    }

    /// Sets a Web App menu button for a specific chat.
    pub fn set_chat_web_app_menu_button(
        &self,
        chat_id: i64,
        text: impl Into<String>,
        web_app: impl Into<WebAppInfo>,
    ) -> Result<bool> {
        self.set_menu_button(MenuButtonConfig::for_chat_web_app(chat_id, text, web_app))
    }

    /// Applies a menu button configuration with retry/backoff.
    pub fn set_menu_button_with_retry(
        &self,
        config: impl Into<MenuButtonConfig>,
        policy: BootstrapRetryPolicy,
    ) -> Result<bool> {
        let config = config.into();
        let request: crate::types::advanced::AdvancedSetChatMenuButtonRequest = (&config).into();
        retry_blocking(policy, || {
            self.client.advanced().set_chat_menu_button_typed(&request)
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
            let get_request = commands_get_request(commands);
            let current =
                retry_fetch_blocking(policy, || self.client.bot().get_my_commands(&get_request))?;
            if current
                .as_ref()
                .is_some_and(|value| value == &commands.commands)
            {
                report.commands_applied = Some(false);
                report.commands_synced = Some(true);
            } else {
                let applied =
                    retry_blocking(policy, || self.client.bot().set_my_commands(commands))?;
                report.commands_applied = Some(applied);
                report.commands_synced = Some(applied);
            }
        }
        if let Some(menu_button) = plan.menu_button.as_ref() {
            let get_request: crate::types::advanced::AdvancedGetChatMenuButtonRequest =
                menu_button.into();
            let set_request: crate::types::advanced::AdvancedSetChatMenuButtonRequest =
                menu_button.into();
            let desired_button = menu_button.menu_button.clone();
            let current = retry_fetch_blocking(policy, || {
                self.client
                    .advanced()
                    .get_chat_menu_button_typed(&get_request)
            })?;
            if current
                .as_ref()
                .is_some_and(|value| value == &desired_button)
            {
                report.menu_button_applied = Some(false);
                report.menu_button_synced = Some(true);
            } else {
                let applied = retry_blocking(policy, || {
                    self.client
                        .advanced()
                        .set_chat_menu_button_typed(&set_request)
                })?;
                report.menu_button_applied = Some(applied);
                report.menu_button_synced = Some(applied);
            }
        }

        Ok(report)
    }

    /// Runs common startup bootstrap without retries.
    pub fn bootstrap(&self, plan: &BootstrapPlan) -> Result<BootstrapReport> {
        self.bootstrap_with_retry(plan, single_attempt_bootstrap_policy())
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
        let request = web_app_query_request(web_app_query_id, result)?;
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

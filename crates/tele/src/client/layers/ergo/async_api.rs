#[cfg(feature = "_async")]
use super::*;

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
        let chat_id = reply_chat_id(update)?;
        self.send_text(chat_id, text).await
    }

    /// Answers callback query with optional message text.
    pub async fn answer_callback(
        &self,
        callback_query_id: impl Into<String>,
        text: Option<String>,
    ) -> Result<bool> {
        let request = callback_answer_request(callback_query_id, text);
        self.client.updates().answer_callback_query(&request).await
    }

    /// Answers callback query from update payload.
    pub async fn answer_callback_from_update(
        &self,
        update: &Update,
        text: Option<String>,
    ) -> Result<bool> {
        let Some(callback_query_id) = callback_query_id(update) else {
            return Err(super::super::support::invalid_request(
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

    /// Reads the default menu button configuration.
    pub async fn get_menu_button(&self) -> Result<MenuButton> {
        self.client
            .advanced()
            .get_chat_menu_button_typed(&get_default_menu_button_request())
            .await
    }

    /// Reads the menu button configuration for a specific chat.
    pub async fn get_chat_menu_button(&self, chat_id: i64) -> Result<MenuButton> {
        let request = get_chat_menu_button_request(chat_id);
        self.client
            .advanced()
            .get_chat_menu_button_typed(&request)
            .await
    }

    /// Applies a menu button configuration without dropping into `advanced()`.
    pub async fn set_menu_button(&self, config: impl Into<MenuButtonConfig>) -> Result<bool> {
        let config = config.into();
        let request = set_menu_button_request(&config);
        self.client
            .advanced()
            .set_chat_menu_button_typed(&request)
            .await
    }

    /// Applies a menu button for a specific chat.
    pub async fn set_chat_menu_button(
        &self,
        chat_id: i64,
        menu_button: impl Into<MenuButton>,
    ) -> Result<bool> {
        self.set_menu_button(MenuButtonConfig::for_chat(chat_id, menu_button))
            .await
    }

    /// Restores the default menu button.
    pub async fn set_default_menu_button(&self) -> Result<bool> {
        self.set_menu_button(MenuButtonConfig::default_button())
            .await
    }

    /// Restores the default menu button for a specific chat.
    pub async fn set_chat_default_menu_button(&self, chat_id: i64) -> Result<bool> {
        self.set_menu_button(MenuButtonConfig::for_chat_default(chat_id))
            .await
    }

    /// Sets the commands menu button.
    pub async fn set_commands_menu_button(&self) -> Result<bool> {
        self.set_menu_button(MenuButtonConfig::commands()).await
    }

    /// Sets the commands menu button for a specific chat.
    pub async fn set_chat_commands_menu_button(&self, chat_id: i64) -> Result<bool> {
        self.set_menu_button(MenuButtonConfig::for_chat_commands(chat_id))
            .await
    }

    /// Sets a Web App menu button.
    pub async fn set_web_app_menu_button(
        &self,
        text: impl Into<String>,
        web_app: impl Into<WebAppInfo>,
    ) -> Result<bool> {
        self.set_menu_button(MenuButtonConfig::web_app(text, web_app))
            .await
    }

    /// Sets a Web App menu button for a specific chat.
    pub async fn set_chat_web_app_menu_button(
        &self,
        chat_id: i64,
        text: impl Into<String>,
        web_app: impl Into<WebAppInfo>,
    ) -> Result<bool> {
        self.set_menu_button(MenuButtonConfig::for_chat_web_app(chat_id, text, web_app))
            .await
    }

    /// Applies a menu button configuration with retry/backoff.
    pub async fn set_menu_button_with_retry(
        &self,
        config: impl Into<MenuButtonConfig>,
        policy: BootstrapRetryPolicy,
    ) -> Result<bool> {
        let config = config.into();
        let request = set_menu_button_request(&config);
        retry_async(policy, || async {
            self.client
                .advanced()
                .set_chat_menu_button_typed(&request)
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
            let get_request = commands_get_request(commands);
            let current = retry_fetch_async(policy, || {
                let get_request = get_request.clone();
                async move { self.client.bot().get_my_commands(&get_request).await }
            })
            .await?;
            if commands_in_sync(current.as_ref(), commands) {
                mark_commands_unchanged(&mut report);
            } else {
                let applied = retry_async(policy, || async {
                    self.client.bot().set_my_commands(commands).await
                })
                .await?;
                mark_commands_applied(&mut report, applied);
            }
        }
        if let Some(menu_button) = plan.menu_button.as_ref() {
            let get_request: crate::types::advanced::AdvancedGetChatMenuButtonRequest =
                menu_button.into();
            let set_request = set_menu_button_request(menu_button);
            let current = retry_fetch_async(policy, || {
                let get_request = get_request.clone();
                async move {
                    self.client
                        .advanced()
                        .get_chat_menu_button_typed(&get_request)
                        .await
                }
            })
            .await?;
            if menu_button_in_sync(current.as_ref(), menu_button) {
                mark_menu_button_unchanged(&mut report);
            } else {
                let applied = retry_async(policy, || async {
                    self.client
                        .advanced()
                        .set_chat_menu_button_typed(&set_request)
                        .await
                })
                .await?;
                mark_menu_button_applied(&mut report, applied);
            }
        }

        Ok(report)
    }

    /// Runs common startup bootstrap without retries.
    pub async fn bootstrap(&self, plan: &BootstrapPlan) -> Result<BootstrapReport> {
        self.bootstrap_with_retry(plan, single_attempt_bootstrap_policy())
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
        let request = web_app_query_request(web_app_query_id, result)?;
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

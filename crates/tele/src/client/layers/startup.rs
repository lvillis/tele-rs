use super::bootstrap::{
    BootstrapOutcome, BootstrapRetryOutcome, bootstrap_failure_diagnostics,
    bootstrap_success_diagnostics,
};
#[cfg(feature = "_async")]
use super::bootstrap::{retry_async, retry_step_async};
#[cfg(feature = "_blocking")]
use super::bootstrap::{retry_blocking, retry_step_blocking};
use super::support::commands_get_request;
#[cfg(feature = "bot")]
use super::support::typed_commands_request;
use super::*;

fn get_default_menu_button_request() -> crate::types::advanced::AdvancedGetChatMenuButtonRequest {
    crate::types::advanced::AdvancedGetChatMenuButtonRequest::new()
}

fn get_chat_menu_button_request(
    chat_id: i64,
) -> crate::types::advanced::AdvancedGetChatMenuButtonRequest {
    crate::types::advanced::AdvancedGetChatMenuButtonRequest {
        chat_id: Some(chat_id),
    }
}

fn set_menu_button_request(
    config: &MenuButtonConfig,
) -> crate::types::advanced::AdvancedSetChatMenuButtonRequest {
    config.into()
}

fn commands_in_sync(current: &[BotCommand], commands: &SetMyCommandsRequest) -> bool {
    current == commands.commands.as_slice()
}

fn menu_button_in_sync(current: &MenuButton, menu_button: &MenuButtonConfig) -> bool {
    current == &menu_button.menu_button
}

fn single_attempt_bootstrap_policy() -> BootstrapRetryPolicy {
    BootstrapRetryPolicy {
        max_attempts: 1,
        continue_on_failure: false,
        ..BootstrapRetryPolicy::default()
    }
}

fn sync_step_report(
    status: BootstrapStepStatus,
    phase: BootstrapStepPhase,
    attempt_count: usize,
    applied: Option<bool>,
    synced: Option<bool>,
) -> BootstrapSyncStepReport {
    BootstrapSyncStepReport {
        applied,
        synced,
        diagnostics: bootstrap_success_diagnostics(status, phase, attempt_count),
    }
}

fn sync_step_failure_report(
    status: BootstrapStepStatus,
    phase: BootstrapStepPhase,
    error: &Error,
    attempt_count: usize,
    applied: Option<bool>,
    synced: Option<bool>,
) -> BootstrapSyncStepReport {
    BootstrapSyncStepReport {
        applied,
        synced,
        diagnostics: bootstrap_failure_diagnostics(status, phase, error, attempt_count),
    }
}

fn handle_get_me_failure(
    report: &mut BootstrapReport,
    policy: BootstrapGetMePolicy,
    error: Error,
    attempt_count: usize,
) -> Option<Error> {
    let warn_and_continue =
        matches!(policy, BootstrapGetMePolicy::WarnAndContinueOnRetryable) && error.is_retryable();
    report.me.diagnostics = bootstrap_failure_diagnostics(
        if warn_and_continue {
            BootstrapStepStatus::Warned
        } else {
            BootstrapStepStatus::Failed
        },
        BootstrapStepPhase::Fetch,
        &error,
        attempt_count,
    );
    if warn_and_continue { None } else { Some(error) }
}

#[cfg(feature = "_async")]
#[derive(Clone)]
pub struct StartupApi {
    client: Client,
}

#[cfg(feature = "_async")]
impl StartupApi {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn set_commands(&self, commands: Vec<BotCommand>) -> Result<bool> {
        let request = SetMyCommandsRequest::new(commands)?;
        self.client.bot().set_my_commands(&request).await
    }

    #[cfg(feature = "bot")]
    pub async fn set_typed_commands<C>(&self) -> Result<bool>
    where
        C: crate::bot::BotCommands,
    {
        self.set_commands(crate::bot::command_definitions::<C>())
            .await
    }

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

    pub async fn get_menu_button(&self) -> Result<MenuButton> {
        self.client
            .advanced()
            .get_chat_menu_button_typed(&get_default_menu_button_request())
            .await
    }

    pub async fn get_chat_menu_button(&self, chat_id: i64) -> Result<MenuButton> {
        let request = get_chat_menu_button_request(chat_id);
        self.client
            .advanced()
            .get_chat_menu_button_typed(&request)
            .await
    }

    pub async fn set_menu_button(&self, config: impl Into<MenuButtonConfig>) -> Result<bool> {
        let config = config.into();
        let request = set_menu_button_request(&config);
        self.client
            .advanced()
            .set_chat_menu_button_typed(&request)
            .await
    }

    pub async fn set_chat_menu_button(
        &self,
        chat_id: i64,
        menu_button: impl Into<MenuButton>,
    ) -> Result<bool> {
        self.set_menu_button(MenuButtonConfig::for_chat(chat_id, menu_button))
            .await
    }

    pub async fn set_default_menu_button(&self) -> Result<bool> {
        self.set_menu_button(MenuButtonConfig::default_button())
            .await
    }

    pub async fn set_chat_default_menu_button(&self, chat_id: i64) -> Result<bool> {
        self.set_menu_button(MenuButtonConfig::for_chat_default(chat_id))
            .await
    }

    pub async fn set_commands_menu_button(&self) -> Result<bool> {
        self.set_menu_button(MenuButtonConfig::commands()).await
    }

    pub async fn set_chat_commands_menu_button(&self, chat_id: i64) -> Result<bool> {
        self.set_menu_button(MenuButtonConfig::for_chat_commands(chat_id))
            .await
    }

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

    pub async fn bootstrap_with_retry(
        &self,
        plan: &BootstrapPlan,
        policy: BootstrapRetryPolicy,
    ) -> BootstrapOutcome {
        let mut report = BootstrapReport::default();

        if !matches!(plan.get_me, BootstrapGetMePolicy::Skip) {
            match retry_step_async(policy, || async { self.client.bot().get_me().await }).await {
                BootstrapRetryOutcome::Success {
                    value,
                    attempt_count,
                } => {
                    report.me.value = Some(value);
                    report.me.diagnostics = bootstrap_success_diagnostics(
                        BootstrapStepStatus::Succeeded,
                        BootstrapStepPhase::Fetch,
                        attempt_count,
                    );
                }
                BootstrapRetryOutcome::Failed {
                    error,
                    attempt_count,
                } => {
                    if let Some(error) =
                        handle_get_me_failure(&mut report, plan.get_me, error, attempt_count)
                    {
                        return BootstrapOutcome::failure(report, error);
                    }
                }
            }
        }

        if let Some(commands) = plan.commands.as_ref() {
            report.commands = Some(BootstrapSyncStepReport::default());
            let get_request = commands_get_request(commands);
            let fetch_attempts = match retry_step_async(policy, || {
                let get_request = get_request.clone();
                async move { self.client.bot().get_my_commands(&get_request).await }
            })
            .await
            {
                BootstrapRetryOutcome::Success {
                    value,
                    attempt_count,
                } => {
                    if commands_in_sync(&value, commands) {
                        report.commands = Some(sync_step_report(
                            BootstrapStepStatus::Unchanged,
                            BootstrapStepPhase::Check,
                            attempt_count,
                            Some(false),
                            Some(true),
                        ));
                        0
                    } else {
                        attempt_count
                    }
                }
                BootstrapRetryOutcome::Failed {
                    error,
                    attempt_count,
                } => {
                    report.commands = Some(sync_step_failure_report(
                        if policy.continue_on_failure {
                            BootstrapStepStatus::Warned
                        } else {
                            BootstrapStepStatus::Failed
                        },
                        BootstrapStepPhase::Check,
                        &error,
                        attempt_count,
                        Some(false),
                        Some(false),
                    ));
                    if policy.continue_on_failure {
                        0
                    } else {
                        return BootstrapOutcome::failure(report, error);
                    }
                }
            };
            if fetch_attempts > 0 {
                match retry_step_async(policy, || async {
                    self.client.bot().set_my_commands(commands).await
                })
                .await
                {
                    BootstrapRetryOutcome::Success {
                        value,
                        attempt_count,
                    } => {
                        report.commands = Some(sync_step_report(
                            if value {
                                BootstrapStepStatus::Applied
                            } else {
                                BootstrapStepStatus::Succeeded
                            },
                            BootstrapStepPhase::Apply,
                            fetch_attempts + attempt_count,
                            Some(value),
                            Some(value),
                        ));
                    }
                    BootstrapRetryOutcome::Failed {
                        error,
                        attempt_count,
                    } => {
                        report.commands = Some(sync_step_failure_report(
                            if policy.continue_on_failure {
                                BootstrapStepStatus::Warned
                            } else {
                                BootstrapStepStatus::Failed
                            },
                            BootstrapStepPhase::Apply,
                            &error,
                            fetch_attempts + attempt_count,
                            Some(false),
                            Some(false),
                        ));
                        if !policy.continue_on_failure {
                            return BootstrapOutcome::failure(report, error);
                        }
                    }
                }
            }
        }

        if let Some(menu_button) = plan.menu_button.as_ref() {
            report.menu_button = Some(BootstrapSyncStepReport::default());
            let get_request: crate::types::advanced::AdvancedGetChatMenuButtonRequest =
                menu_button.into();
            let fetch_attempts = match retry_step_async(policy, || {
                let get_request = get_request.clone();
                async move {
                    self.client
                        .advanced()
                        .get_chat_menu_button_typed(&get_request)
                        .await
                }
            })
            .await
            {
                BootstrapRetryOutcome::Success {
                    value,
                    attempt_count,
                } => {
                    if menu_button_in_sync(&value, menu_button) {
                        report.menu_button = Some(sync_step_report(
                            BootstrapStepStatus::Unchanged,
                            BootstrapStepPhase::Check,
                            attempt_count,
                            Some(false),
                            Some(true),
                        ));
                        0
                    } else {
                        attempt_count
                    }
                }
                BootstrapRetryOutcome::Failed {
                    error,
                    attempt_count,
                } => {
                    report.menu_button = Some(sync_step_failure_report(
                        if policy.continue_on_failure {
                            BootstrapStepStatus::Warned
                        } else {
                            BootstrapStepStatus::Failed
                        },
                        BootstrapStepPhase::Check,
                        &error,
                        attempt_count,
                        Some(false),
                        Some(false),
                    ));
                    if policy.continue_on_failure {
                        0
                    } else {
                        return BootstrapOutcome::failure(report, error);
                    }
                }
            };
            if fetch_attempts > 0 {
                let set_request = set_menu_button_request(menu_button);
                match retry_step_async(policy, || async {
                    self.client
                        .advanced()
                        .set_chat_menu_button_typed(&set_request)
                        .await
                })
                .await
                {
                    BootstrapRetryOutcome::Success {
                        value,
                        attempt_count,
                    } => {
                        report.menu_button = Some(sync_step_report(
                            if value {
                                BootstrapStepStatus::Applied
                            } else {
                                BootstrapStepStatus::Succeeded
                            },
                            BootstrapStepPhase::Apply,
                            fetch_attempts + attempt_count,
                            Some(value),
                            Some(value),
                        ));
                    }
                    BootstrapRetryOutcome::Failed {
                        error,
                        attempt_count,
                    } => {
                        report.menu_button = Some(sync_step_failure_report(
                            if policy.continue_on_failure {
                                BootstrapStepStatus::Warned
                            } else {
                                BootstrapStepStatus::Failed
                            },
                            BootstrapStepPhase::Apply,
                            &error,
                            fetch_attempts + attempt_count,
                            Some(false),
                            Some(false),
                        ));
                        if !policy.continue_on_failure {
                            return BootstrapOutcome::failure(report, error);
                        }
                    }
                }
            }
        }

        BootstrapOutcome::success(report)
    }

    pub async fn bootstrap(&self, plan: &BootstrapPlan) -> BootstrapOutcome {
        self.bootstrap_with_retry(plan, single_attempt_bootstrap_policy())
            .await
    }
}

#[cfg(feature = "_blocking")]
#[derive(Clone)]
pub struct BlockingStartupApi {
    client: BlockingClient,
}

#[cfg(feature = "_blocking")]
impl BlockingStartupApi {
    pub(crate) fn new(client: BlockingClient) -> Self {
        Self { client }
    }

    pub fn set_commands(&self, commands: Vec<BotCommand>) -> Result<bool> {
        let request = SetMyCommandsRequest::new(commands)?;
        self.client.bot().set_my_commands(&request)
    }

    #[cfg(feature = "bot")]
    pub fn set_typed_commands<C>(&self) -> Result<bool>
    where
        C: crate::bot::BotCommands,
    {
        self.set_commands(crate::bot::command_definitions::<C>())
    }

    pub fn set_commands_with_retry(
        &self,
        commands: Vec<BotCommand>,
        policy: BootstrapRetryPolicy,
    ) -> Result<bool> {
        let request = SetMyCommandsRequest::new(commands)?;
        retry_blocking(policy, || self.client.bot().set_my_commands(&request))
    }

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

    pub fn get_menu_button(&self) -> Result<MenuButton> {
        self.client
            .advanced()
            .get_chat_menu_button_typed(&get_default_menu_button_request())
    }

    pub fn get_chat_menu_button(&self, chat_id: i64) -> Result<MenuButton> {
        let request = get_chat_menu_button_request(chat_id);
        self.client.advanced().get_chat_menu_button_typed(&request)
    }

    pub fn set_menu_button(&self, config: impl Into<MenuButtonConfig>) -> Result<bool> {
        let config = config.into();
        let request = set_menu_button_request(&config);
        self.client.advanced().set_chat_menu_button_typed(&request)
    }

    pub fn set_chat_menu_button(
        &self,
        chat_id: i64,
        menu_button: impl Into<MenuButton>,
    ) -> Result<bool> {
        self.set_menu_button(MenuButtonConfig::for_chat(chat_id, menu_button))
    }

    pub fn set_default_menu_button(&self) -> Result<bool> {
        self.set_menu_button(MenuButtonConfig::default_button())
    }

    pub fn set_chat_default_menu_button(&self, chat_id: i64) -> Result<bool> {
        self.set_menu_button(MenuButtonConfig::for_chat_default(chat_id))
    }

    pub fn set_commands_menu_button(&self) -> Result<bool> {
        self.set_menu_button(MenuButtonConfig::commands())
    }

    pub fn set_chat_commands_menu_button(&self, chat_id: i64) -> Result<bool> {
        self.set_menu_button(MenuButtonConfig::for_chat_commands(chat_id))
    }

    pub fn set_menu_button_with_retry(
        &self,
        config: impl Into<MenuButtonConfig>,
        policy: BootstrapRetryPolicy,
    ) -> Result<bool> {
        let config = config.into();
        let request = set_menu_button_request(&config);
        retry_blocking(policy, || {
            self.client.advanced().set_chat_menu_button_typed(&request)
        })
    }

    pub fn bootstrap_with_retry(
        &self,
        plan: &BootstrapPlan,
        policy: BootstrapRetryPolicy,
    ) -> BootstrapOutcome {
        let mut report = BootstrapReport::default();

        if !matches!(plan.get_me, BootstrapGetMePolicy::Skip) {
            match retry_step_blocking(policy, || self.client.bot().get_me()) {
                BootstrapRetryOutcome::Success {
                    value,
                    attempt_count,
                } => {
                    report.me.value = Some(value);
                    report.me.diagnostics = bootstrap_success_diagnostics(
                        BootstrapStepStatus::Succeeded,
                        BootstrapStepPhase::Fetch,
                        attempt_count,
                    );
                }
                BootstrapRetryOutcome::Failed {
                    error,
                    attempt_count,
                } => {
                    if let Some(error) =
                        handle_get_me_failure(&mut report, plan.get_me, error, attempt_count)
                    {
                        return BootstrapOutcome::failure(report, error);
                    }
                }
            }
        }

        if let Some(commands) = plan.commands.as_ref() {
            report.commands = Some(BootstrapSyncStepReport::default());
            let get_request = commands_get_request(commands);
            let fetch_attempts = match retry_step_blocking(policy, || {
                self.client.bot().get_my_commands(&get_request)
            }) {
                BootstrapRetryOutcome::Success {
                    value,
                    attempt_count,
                } => {
                    if commands_in_sync(&value, commands) {
                        report.commands = Some(sync_step_report(
                            BootstrapStepStatus::Unchanged,
                            BootstrapStepPhase::Check,
                            attempt_count,
                            Some(false),
                            Some(true),
                        ));
                        0
                    } else {
                        attempt_count
                    }
                }
                BootstrapRetryOutcome::Failed {
                    error,
                    attempt_count,
                } => {
                    report.commands = Some(sync_step_failure_report(
                        if policy.continue_on_failure {
                            BootstrapStepStatus::Warned
                        } else {
                            BootstrapStepStatus::Failed
                        },
                        BootstrapStepPhase::Check,
                        &error,
                        attempt_count,
                        Some(false),
                        Some(false),
                    ));
                    if policy.continue_on_failure {
                        0
                    } else {
                        return BootstrapOutcome::failure(report, error);
                    }
                }
            };
            if fetch_attempts > 0 {
                match retry_step_blocking(policy, || self.client.bot().set_my_commands(commands)) {
                    BootstrapRetryOutcome::Success {
                        value,
                        attempt_count,
                    } => {
                        report.commands = Some(sync_step_report(
                            if value {
                                BootstrapStepStatus::Applied
                            } else {
                                BootstrapStepStatus::Succeeded
                            },
                            BootstrapStepPhase::Apply,
                            fetch_attempts + attempt_count,
                            Some(value),
                            Some(value),
                        ));
                    }
                    BootstrapRetryOutcome::Failed {
                        error,
                        attempt_count,
                    } => {
                        report.commands = Some(sync_step_failure_report(
                            if policy.continue_on_failure {
                                BootstrapStepStatus::Warned
                            } else {
                                BootstrapStepStatus::Failed
                            },
                            BootstrapStepPhase::Apply,
                            &error,
                            fetch_attempts + attempt_count,
                            Some(false),
                            Some(false),
                        ));
                        if !policy.continue_on_failure {
                            return BootstrapOutcome::failure(report, error);
                        }
                    }
                }
            }
        }

        if let Some(menu_button) = plan.menu_button.as_ref() {
            report.menu_button = Some(BootstrapSyncStepReport::default());
            let get_request: crate::types::advanced::AdvancedGetChatMenuButtonRequest =
                menu_button.into();
            let fetch_attempts = match retry_step_blocking(policy, || {
                self.client
                    .advanced()
                    .get_chat_menu_button_typed(&get_request)
            }) {
                BootstrapRetryOutcome::Success {
                    value,
                    attempt_count,
                } => {
                    if menu_button_in_sync(&value, menu_button) {
                        report.menu_button = Some(sync_step_report(
                            BootstrapStepStatus::Unchanged,
                            BootstrapStepPhase::Check,
                            attempt_count,
                            Some(false),
                            Some(true),
                        ));
                        0
                    } else {
                        attempt_count
                    }
                }
                BootstrapRetryOutcome::Failed {
                    error,
                    attempt_count,
                } => {
                    report.menu_button = Some(sync_step_failure_report(
                        if policy.continue_on_failure {
                            BootstrapStepStatus::Warned
                        } else {
                            BootstrapStepStatus::Failed
                        },
                        BootstrapStepPhase::Check,
                        &error,
                        attempt_count,
                        Some(false),
                        Some(false),
                    ));
                    if policy.continue_on_failure {
                        0
                    } else {
                        return BootstrapOutcome::failure(report, error);
                    }
                }
            };
            if fetch_attempts > 0 {
                let set_request = set_menu_button_request(menu_button);
                match retry_step_blocking(policy, || {
                    self.client
                        .advanced()
                        .set_chat_menu_button_typed(&set_request)
                }) {
                    BootstrapRetryOutcome::Success {
                        value,
                        attempt_count,
                    } => {
                        report.menu_button = Some(sync_step_report(
                            if value {
                                BootstrapStepStatus::Applied
                            } else {
                                BootstrapStepStatus::Succeeded
                            },
                            BootstrapStepPhase::Apply,
                            fetch_attempts + attempt_count,
                            Some(value),
                            Some(value),
                        ));
                    }
                    BootstrapRetryOutcome::Failed {
                        error,
                        attempt_count,
                    } => {
                        report.menu_button = Some(sync_step_failure_report(
                            if policy.continue_on_failure {
                                BootstrapStepStatus::Warned
                            } else {
                                BootstrapStepStatus::Failed
                            },
                            BootstrapStepPhase::Apply,
                            &error,
                            fetch_attempts + attempt_count,
                            Some(false),
                            Some(false),
                        ));
                        if !policy.continue_on_failure {
                            return BootstrapOutcome::failure(report, error);
                        }
                    }
                }
            }
        }

        BootstrapOutcome::success(report)
    }

    pub fn bootstrap(&self, plan: &BootstrapPlan) -> BootstrapOutcome {
        self.bootstrap_with_retry(plan, single_attempt_bootstrap_policy())
    }
}

use super::retry::backoff_delay;
use super::*;
use crate::ErrorClass;
use crate::util::retry_after_or_backoff;

/// Retry policy for setup/bootstrap helper methods.
#[derive(Clone, Copy, Debug)]
pub struct BootstrapRetryPolicy {
    pub max_attempts: usize,
    pub base_backoff: Duration,
    /// Maximum locally computed exponential backoff.
    ///
    /// Provider supplied `Retry-After` values are honored separately and are not clamped by this.
    pub max_backoff: Duration,
    pub jitter_ratio: f32,
    /// When true, exhausting retries downgrades sync/apply failures into warnings.
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

impl BootstrapRetryPolicy {
    pub fn validate(&self) -> Result<()> {
        if self.max_attempts == 0 {
            return Err(Error::Configuration {
                reason: "bootstrap retry max_attempts must be at least 1".to_owned(),
            });
        }
        if self.base_backoff.is_zero() {
            return Err(Error::Configuration {
                reason: "bootstrap retry base_backoff must be greater than zero".to_owned(),
            });
        }
        if self.max_backoff.is_zero() {
            return Err(Error::Configuration {
                reason: "bootstrap retry max_backoff must be greater than zero".to_owned(),
            });
        }
        if self.base_backoff > self.max_backoff {
            return Err(Error::Configuration {
                reason: "bootstrap retry base_backoff must not exceed max_backoff".to_owned(),
            });
        }
        if !self.jitter_ratio.is_finite() || !(0.0..=1.0).contains(&self.jitter_ratio) {
            return Err(Error::Configuration {
                reason: "bootstrap retry jitter_ratio must be finite and between 0.0 and 1.0"
                    .to_owned(),
            });
        }

        Ok(())
    }
}

/// Step policy for bootstrap `getMe` execution.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub enum BootstrapGetMePolicy {
    #[default]
    Skip,
    FailFast,
    WarnAndContinueOnRetryable,
}

/// Startup bootstrap plan for common bot initialization actions.
#[derive(Clone, Debug, Default)]
pub struct BootstrapPlan {
    pub get_me: BootstrapGetMePolicy,
    pub commands: Option<SetMyCommandsRequest>,
    pub menu_button: Option<MenuButtonConfig>,
}

impl BootstrapPlan {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_me(mut self, policy: BootstrapGetMePolicy) -> Self {
        self.get_me = policy;
        self
    }

    pub fn fail_fast_get_me(self) -> Self {
        self.get_me(BootstrapGetMePolicy::FailFast)
    }

    pub fn warn_and_continue_on_retryable_get_me(self) -> Self {
        self.get_me(BootstrapGetMePolicy::WarnAndContinueOnRetryable)
    }

    pub fn skip_get_me(self) -> Self {
        self.get_me(BootstrapGetMePolicy::Skip)
    }

    /// Syncs bot commands from a plain command list.
    pub fn commands(self, commands: Vec<BotCommand>) -> Result<Self> {
        self.commands_with_options(commands, None, None)
    }

    /// Syncs bot commands from a plain command list with optional scope and language.
    pub fn commands_with_options(
        mut self,
        commands: Vec<BotCommand>,
        scope: Option<BotCommandScope>,
        language_code: Option<String>,
    ) -> Result<Self> {
        self.commands = Some(super::support::build_set_my_commands_request(
            commands,
            scope,
            language_code,
        )?);
        Ok(self)
    }

    #[cfg(feature = "bot")]
    /// Syncs bot commands from a typed command enum.
    pub fn typed_commands<C>(self) -> Result<Self>
    where
        C: crate::bot::BotCommands,
    {
        self.typed_commands_with_options::<C>(None, None)
    }

    #[cfg(feature = "bot")]
    /// Syncs bot commands from a typed command enum with optional scope and language.
    pub fn typed_commands_with_options<C>(
        mut self,
        scope: Option<BotCommandScope>,
        language_code: Option<String>,
    ) -> Result<Self>
    where
        C: crate::bot::BotCommands,
    {
        self.commands = Some(super::support::typed_commands_request::<C>(
            scope,
            language_code,
        )?);
        Ok(self)
    }

    /// Escape hatch for supplying a fully custom `setMyCommands` request.
    pub fn commands_request(mut self, commands: SetMyCommandsRequest) -> Self {
        self.commands = Some(commands);
        self
    }

    pub fn menu_button(mut self, menu_button: impl Into<MenuButtonConfig>) -> Self {
        self.menu_button = Some(menu_button.into());
        self
    }

    pub fn menu_button_default(self) -> Self {
        self.menu_button(MenuButtonConfig::default_button())
    }

    pub fn menu_button_commands(self) -> Self {
        self.menu_button(MenuButtonConfig::commands())
    }

    pub fn menu_button_web_app(
        self,
        text: impl Into<String>,
        web_app: impl Into<crate::types::telegram::WebAppInfo>,
    ) -> Self {
        self.menu_button(MenuButtonConfig::web_app(text, web_app))
    }
}

/// Bootstrap step phase used by detailed reporting.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub enum BootstrapStepPhase {
    #[default]
    Fetch,
    Check,
    Apply,
}

/// Bootstrap step execution status used by detailed reporting.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub enum BootstrapStepStatus {
    #[default]
    Skipped,
    Succeeded,
    Unchanged,
    Applied,
    Warned,
    Failed,
}

/// Shared diagnostics attached to one bootstrap step.
#[derive(Clone, Debug, Default)]
#[non_exhaustive]
pub struct BootstrapStepDiagnostics {
    pub status: BootstrapStepStatus,
    pub phase: Option<BootstrapStepPhase>,
    pub classification: Option<ErrorClass>,
    pub retryable: bool,
    pub request_id: Option<String>,
    pub attempt_count: usize,
    pub message: Option<String>,
}

/// Detailed report for one fetch-like bootstrap step.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct BootstrapFetchStepReport<T> {
    pub value: Option<T>,
    pub diagnostics: BootstrapStepDiagnostics,
}

impl<T> Default for BootstrapFetchStepReport<T> {
    fn default() -> Self {
        Self {
            value: None,
            diagnostics: BootstrapStepDiagnostics::default(),
        }
    }
}

/// Detailed report for one sync/apply bootstrap step.
#[derive(Clone, Debug, Default)]
#[non_exhaustive]
pub struct BootstrapSyncStepReport {
    pub applied: Option<bool>,
    pub synced: Option<bool>,
    pub diagnostics: BootstrapStepDiagnostics,
}

/// Detailed step-by-step bootstrap report.
#[derive(Clone, Debug, Default)]
#[non_exhaustive]
pub struct BootstrapReport {
    pub me: BootstrapFetchStepReport<User>,
    pub commands: Option<BootstrapSyncStepReport>,
    pub menu_button: Option<BootstrapSyncStepReport>,
}

/// Bootstrap execution outcome that always preserves the step report.
#[derive(Debug)]
#[non_exhaustive]
pub struct BootstrapOutcome {
    pub report: BootstrapReport,
    pub error: Option<Error>,
}

impl BootstrapOutcome {
    pub fn success(report: BootstrapReport) -> Self {
        Self {
            report,
            error: None,
        }
    }

    pub fn failure(report: BootstrapReport, error: Error) -> Self {
        Self {
            report,
            error: Some(error),
        }
    }

    pub fn is_success(&self) -> bool {
        self.error.is_none()
    }

    pub fn error(&self) -> Option<&Error> {
        self.error.as_ref()
    }

    pub fn into_result(self) -> Result<BootstrapReport> {
        match self.error {
            Some(error) => Err(error),
            None => Ok(self.report),
        }
    }
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
        super::support::parse_web_app_query_payload(web_app_data)
    }
}

pub(crate) fn bootstrap_success_diagnostics(
    status: BootstrapStepStatus,
    phase: BootstrapStepPhase,
    attempt_count: usize,
) -> BootstrapStepDiagnostics {
    BootstrapStepDiagnostics {
        status,
        phase: Some(phase),
        classification: None,
        retryable: false,
        request_id: None,
        attempt_count,
        message: None,
    }
}

pub(crate) fn bootstrap_failure_diagnostics(
    status: BootstrapStepStatus,
    phase: BootstrapStepPhase,
    error: &Error,
    attempt_count: usize,
) -> BootstrapStepDiagnostics {
    BootstrapStepDiagnostics {
        status,
        phase: Some(phase),
        classification: Some(error.classification()),
        retryable: error.is_retryable(),
        request_id: error.request_id().map(ToOwned::to_owned),
        attempt_count,
        message: Some(error.to_string()),
    }
}

pub(crate) enum BootstrapRetryOutcome<T> {
    Success { value: T, attempt_count: usize },
    Failed { error: Error, attempt_count: usize },
}

#[cfg(feature = "_async")]
pub(crate) async fn retry_step_async<T, F, Fut>(
    policy: BootstrapRetryPolicy,
    mut op: F,
) -> BootstrapRetryOutcome<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    if let Err(error) = policy.validate() {
        return BootstrapRetryOutcome::Failed {
            error,
            attempt_count: 0,
        };
    }

    let max_attempts = policy.max_attempts;
    let mut attempt = 0;

    loop {
        attempt += 1;
        match op().await {
            Ok(value) => {
                return BootstrapRetryOutcome::Success {
                    value,
                    attempt_count: attempt,
                };
            }
            Err(error) => {
                let should_retry = error.is_retryable() && attempt < max_attempts;
                if !should_retry {
                    return BootstrapRetryOutcome::Failed {
                        error,
                        attempt_count: attempt,
                    };
                }
                let delay = retry_after_or_backoff(&error, || {
                    backoff_delay(
                        policy.base_backoff,
                        policy.max_backoff,
                        attempt,
                        f64::from(policy.jitter_ratio),
                    )
                });
                tokio::time::sleep(delay).await;
            }
        }
    }
}

#[cfg(feature = "_blocking")]
pub(crate) fn retry_step_blocking<T, F>(
    policy: BootstrapRetryPolicy,
    mut op: F,
) -> BootstrapRetryOutcome<T>
where
    F: FnMut() -> Result<T>,
{
    if let Err(error) = policy.validate() {
        return BootstrapRetryOutcome::Failed {
            error,
            attempt_count: 0,
        };
    }

    let max_attempts = policy.max_attempts;
    let mut attempt = 0;

    loop {
        attempt += 1;
        match op() {
            Ok(value) => {
                return BootstrapRetryOutcome::Success {
                    value,
                    attempt_count: attempt,
                };
            }
            Err(error) => {
                let should_retry = error.is_retryable() && attempt < max_attempts;
                if !should_retry {
                    return BootstrapRetryOutcome::Failed {
                        error,
                        attempt_count: attempt,
                    };
                }
                let delay = retry_after_or_backoff(&error, || {
                    backoff_delay(
                        policy.base_backoff,
                        policy.max_backoff,
                        attempt,
                        f64::from(policy.jitter_ratio),
                    )
                });
                std::thread::sleep(delay);
            }
        }
    }
}

#[cfg(feature = "_async")]
pub(crate) async fn retry_async<F, Fut>(policy: BootstrapRetryPolicy, op: F) -> Result<bool>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<bool>>,
{
    policy.validate()?;
    match retry_step_async(policy, op).await {
        BootstrapRetryOutcome::Success { value, .. } => Ok(value),
        BootstrapRetryOutcome::Failed { error, .. } => {
            if policy.continue_on_failure {
                Ok(false)
            } else {
                Err(error)
            }
        }
    }
}

#[cfg(feature = "_blocking")]
pub(crate) fn retry_blocking<F>(policy: BootstrapRetryPolicy, op: F) -> Result<bool>
where
    F: FnMut() -> Result<bool>,
{
    policy.validate()?;
    match retry_step_blocking(policy, op) {
        BootstrapRetryOutcome::Success { value, .. } => Ok(value),
        BootstrapRetryOutcome::Failed { error, .. } => {
            if policy.continue_on_failure {
                Ok(false)
            } else {
                Err(error)
            }
        }
    }
}

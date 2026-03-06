use super::*;

/// Retry policy for startup/bootstrap helper methods.
#[derive(Clone, Copy, Debug)]
pub struct BootstrapRetryPolicy {
    pub max_attempts: usize,
    pub base_backoff: Duration,
    pub max_backoff: Duration,
    pub jitter_ratio: f32,
    /// When true, exhausting retries returns `Ok(false)` instead of `Err(...)`.
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

/// Startup bootstrap plan for common bot initialization actions.
#[derive(Clone, Debug, Default)]
pub struct BootstrapPlan {
    pub verify_get_me: bool,
    pub commands: Option<SetMyCommandsRequest>,
    pub menu_button: Option<AdvancedSetChatMenuButtonRequest>,
}

/// Result summary of a bootstrap run.
#[derive(Clone, Debug, Default)]
pub struct BootstrapReport {
    pub me: Option<User>,
    pub commands_applied: Option<bool>,
    pub commands_synced: Option<bool>,
    pub menu_button_applied: Option<bool>,
    pub menu_button_synced: Option<bool>,
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

pub(crate) fn backoff_delay(
    base: Duration,
    max: Duration,
    attempt: usize,
    jitter_ratio: f32,
) -> Duration {
    let exponent = attempt.saturating_sub(1).min(16);
    let factor = 2u32.saturating_pow(exponent as u32);
    let delay = base.saturating_mul(factor).min(max);
    if delay.is_zero() || jitter_ratio <= 0.0 {
        return delay;
    }

    let ratio = f64::from(jitter_ratio.clamp(0.0, 1.0));
    let now_nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0_u128, |value| value.as_nanos());
    let unit = (now_nanos % 10_000) as f64 / 10_000.0;
    let multiplier = (1.0 - ratio) + (2.0 * ratio * unit);
    let jittered = Duration::from_secs_f64(delay.as_secs_f64() * multiplier);
    jittered.min(max)
}

#[cfg(feature = "_async")]
pub(crate) async fn retry_with_config_async<T, F, Fut>(retry: &RetryConfig, mut op: F) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let max_attempts = retry.max_attempts.max(1);
    let mut attempt = 0;

    loop {
        attempt += 1;
        match op().await {
            Ok(value) => return Ok(value),
            Err(error) => {
                let should_retry = error.is_retryable() && attempt < max_attempts;
                if !should_retry {
                    return Err(error);
                }
                let delay = error.retry_after().unwrap_or_else(|| {
                    backoff_delay(
                        retry.base_backoff,
                        retry.max_backoff,
                        attempt,
                        retry.jitter_ratio as f32,
                    )
                });
                tokio::time::sleep(delay.min(retry.max_backoff)).await;
            }
        }
    }
}

#[cfg(feature = "_blocking")]
pub(crate) fn retry_with_config_blocking<T, F>(retry: &RetryConfig, mut op: F) -> Result<T>
where
    F: FnMut() -> Result<T>,
{
    let max_attempts = retry.max_attempts.max(1);
    let mut attempt = 0;

    loop {
        attempt += 1;
        match op() {
            Ok(value) => return Ok(value),
            Err(error) => {
                let should_retry = error.is_retryable() && attempt < max_attempts;
                if !should_retry {
                    return Err(error);
                }
                let delay = error.retry_after().unwrap_or_else(|| {
                    backoff_delay(
                        retry.base_backoff,
                        retry.max_backoff,
                        attempt,
                        retry.jitter_ratio as f32,
                    )
                });
                std::thread::sleep(delay.min(retry.max_backoff));
            }
        }
    }
}

#[cfg(feature = "_async")]
pub(crate) async fn retry_async<F, Fut>(policy: BootstrapRetryPolicy, mut op: F) -> Result<bool>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<bool>>,
{
    let max_attempts = policy.max_attempts.max(1);
    let mut attempt = 0;

    loop {
        attempt += 1;
        match op().await {
            Ok(applied) => return Ok(applied),
            Err(error) => {
                let should_retry = error.is_retryable() && attempt < max_attempts;
                if should_retry {
                    let delay = backoff_delay(
                        policy.base_backoff,
                        policy.max_backoff,
                        attempt,
                        policy.jitter_ratio,
                    );
                    tokio::time::sleep(delay).await;
                    continue;
                }
                if policy.continue_on_failure {
                    return Ok(false);
                }
                return Err(error);
            }
        }
    }
}

#[cfg(feature = "_async")]
pub(crate) async fn retry_fetch_async<T, F, Fut>(
    policy: BootstrapRetryPolicy,
    mut op: F,
) -> Result<Option<T>>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let max_attempts = policy.max_attempts.max(1);
    let mut attempt = 0;

    loop {
        attempt += 1;
        match op().await {
            Ok(value) => return Ok(Some(value)),
            Err(error) => {
                let should_retry = error.is_retryable() && attempt < max_attempts;
                if should_retry {
                    let delay = backoff_delay(
                        policy.base_backoff,
                        policy.max_backoff,
                        attempt,
                        policy.jitter_ratio,
                    );
                    tokio::time::sleep(delay).await;
                    continue;
                }
                if policy.continue_on_failure {
                    return Ok(None);
                }
                return Err(error);
            }
        }
    }
}

#[cfg(feature = "_blocking")]
pub(crate) fn retry_blocking<F>(policy: BootstrapRetryPolicy, mut op: F) -> Result<bool>
where
    F: FnMut() -> Result<bool>,
{
    let max_attempts = policy.max_attempts.max(1);
    let mut attempt = 0;

    loop {
        attempt += 1;
        match op() {
            Ok(applied) => return Ok(applied),
            Err(error) => {
                let should_retry = error.is_retryable() && attempt < max_attempts;
                if should_retry {
                    let delay = backoff_delay(
                        policy.base_backoff,
                        policy.max_backoff,
                        attempt,
                        policy.jitter_ratio,
                    );
                    std::thread::sleep(delay);
                    continue;
                }
                if policy.continue_on_failure {
                    return Ok(false);
                }
                return Err(error);
            }
        }
    }
}

#[cfg(feature = "_blocking")]
pub(crate) fn retry_fetch_blocking<T, F>(
    policy: BootstrapRetryPolicy,
    mut op: F,
) -> Result<Option<T>>
where
    F: FnMut() -> Result<T>,
{
    let max_attempts = policy.max_attempts.max(1);
    let mut attempt = 0;

    loop {
        attempt += 1;
        match op() {
            Ok(value) => return Ok(Some(value)),
            Err(error) => {
                let should_retry = error.is_retryable() && attempt < max_attempts;
                if should_retry {
                    let delay = backoff_delay(
                        policy.base_backoff,
                        policy.max_backoff,
                        attempt,
                        policy.jitter_ratio,
                    );
                    std::thread::sleep(delay);
                    continue;
                }
                if policy.continue_on_failure {
                    return Ok(None);
                }
                return Err(error);
            }
        }
    }
}

use super::*;
use crate::ErrorClass;
use crate::util::{jittered_duration, retry_after_or_backoff};

pub(crate) fn backoff_delay(
    base: Duration,
    max: Duration,
    attempt: usize,
    jitter_ratio: f64,
) -> Duration {
    let exponent = attempt.saturating_sub(1).min(16);
    let factor = 2u32.saturating_pow(exponent as u32);
    let delay = base.saturating_mul(factor).min(max);
    jittered_duration(delay, jitter_ratio, max)
}

pub(crate) fn method_allows_retry(method: &str, retry: &RetryConfig) -> bool {
    retry.allow_non_idempotent_retries || is_read_only_telegram_method(method)
}

fn is_read_only_telegram_method(method: &str) -> bool {
    let method = method.trim_start_matches('/');
    let Some(next) = method.as_bytes().get(3) else {
        return false;
    };

    // Telegram Bot API read methods are lower-camel `get...`; treat every
    // mutating method as opt-in unless the server clearly rejected it.
    method.starts_with("get") && next.is_ascii_uppercase()
}

fn should_retry_method_error(
    method: &str,
    retry: &RetryConfig,
    error: &Error,
    attempt: usize,
    max_attempts: usize,
) -> bool {
    if !error.is_retryable() || attempt >= max_attempts {
        return false;
    }

    error.classification() == ErrorClass::RateLimited || method_allows_retry(method, retry)
}

#[cfg(feature = "_async")]
pub(crate) async fn retry_method_async<T, F, Fut>(
    method: &str,
    retry: &RetryConfig,
    mut op: F,
) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    retry.validate()?;
    let max_attempts = retry.max_attempts;
    let mut attempt = 0;

    loop {
        attempt += 1;
        match op().await {
            Ok(value) => return Ok(value),
            Err(error) => {
                if !should_retry_method_error(method, retry, &error, attempt, max_attempts) {
                    return Err(error);
                }
                let delay = retry_after_or_backoff(&error, || {
                    backoff_delay(
                        retry.base_backoff,
                        retry.max_backoff,
                        attempt,
                        retry.jitter_ratio,
                    )
                });
                tokio::time::sleep(delay).await;
            }
        }
    }
}

#[cfg(feature = "_blocking")]
pub(crate) fn retry_method_blocking<T, F>(method: &str, retry: &RetryConfig, mut op: F) -> Result<T>
where
    F: FnMut() -> Result<T>,
{
    retry.validate()?;
    let max_attempts = retry.max_attempts;
    let mut attempt = 0;

    loop {
        attempt += 1;
        match op() {
            Ok(value) => return Ok(value),
            Err(error) => {
                if !should_retry_method_error(method, retry, &error, attempt, max_attempts) {
                    return Err(error);
                }
                let delay = retry_after_or_backoff(&error, || {
                    backoff_delay(
                        retry.base_backoff,
                        retry.max_backoff,
                        attempt,
                        retry.jitter_ratio,
                    )
                });
                std::thread::sleep(delay);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn retryable_transport_error(status: u16) -> Error {
        Error::Transport {
            method: "sendMessage".to_owned(),
            status: Some(status),
            request_id: None,
            retry_after: None,
            request_path: None,
            message: "retryable transport failure".into(),
        }
    }

    #[test]
    fn method_retry_policy_is_conservative_by_default() {
        let retry = RetryConfig::default();

        assert!(method_allows_retry("getMe", &retry));
        assert!(method_allows_retry("getAvailableGifts", &retry));
        assert!(method_allows_retry("/getMe", &retry));
        assert!(!method_allows_retry("sendMessage", &retry));
        assert!(!method_allows_retry("getme", &retry));
    }

    #[test]
    fn method_retry_policy_can_opt_into_mutating_methods() {
        let retry = RetryConfig {
            allow_non_idempotent_retries: true,
            ..RetryConfig::default()
        };

        assert!(method_allows_retry("sendMessage", &retry));
    }

    #[test]
    fn non_idempotent_methods_skip_ambiguous_retries_by_default() {
        let retry = RetryConfig::default();
        let error = retryable_transport_error(502);

        assert!(!should_retry_method_error(
            "sendMessage",
            &retry,
            &error,
            1,
            3
        ));
    }

    #[test]
    fn rate_limited_methods_retry_even_when_mutating() {
        let retry = RetryConfig::default();
        let error = retryable_transport_error(429);

        assert!(should_retry_method_error(
            "sendMessage",
            &retry,
            &error,
            1,
            3
        ));
    }

    #[test]
    fn retry_after_header_does_not_bypass_non_idempotent_policy() {
        let retry = RetryConfig::default();
        let error = Error::Transport {
            method: "sendMessage".to_owned(),
            status: Some(503),
            request_id: None,
            retry_after: Some(Duration::from_secs(1)),
            request_path: None,
            message: "service unavailable".into(),
        };

        assert!(!should_retry_method_error(
            "sendMessage",
            &retry,
            &error,
            1,
            3
        ));
        assert!(should_retry_method_error("getMe", &retry, &error, 1, 3));
    }
}

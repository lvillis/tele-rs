use http::HeaderMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use url::Url;

use crate::Error;

const REDACTED_TOKEN: &str = "<redacted-token>";
const REDACTED_CREDENTIAL: &str = "redacted";

pub(crate) fn normalize_base_url(input: &str) -> Result<Url, Error> {
    if input.trim().len() != input.len() || input.is_empty() {
        return Err(invalid_base_url(
            input,
            "base url must not be empty or padded",
        ));
    }

    let parsed = Url::parse(input).map_err(|source| Error::InvalidBaseUrl {
        input: redact_url_credentials(input),
        source,
    })?;

    let scheme = parsed.scheme();
    if scheme != "https" && scheme != "http" {
        return Err(Error::InvalidBaseUrlScheme {
            scheme: scheme.to_owned(),
        });
    }
    if parsed.host_str().is_none() {
        return Err(invalid_base_url(input, "base url must include a host"));
    }
    if !parsed.username().is_empty() || parsed.password().is_some() {
        return Err(invalid_base_url(
            input,
            "base url must not include credentials",
        ));
    }
    if parsed.query().is_some() {
        return Err(invalid_base_url(
            input,
            "base url must not include query parameters",
        ));
    }
    if parsed.fragment().is_some() {
        return Err(invalid_base_url(
            input,
            "base url must not include a fragment",
        ));
    }

    let uri = parsed
        .as_str()
        .parse::<http::Uri>()
        .map_err(|_| invalid_base_url(input, "base url is not a valid HTTP URI"))?;
    if uri.scheme_str().is_none() || uri.host().is_none() {
        return Err(invalid_base_url(input, "base url is not absolute"));
    }

    Ok(parsed)
}

fn invalid_base_url(input: &str, reason: &str) -> Error {
    Error::Configuration {
        reason: format!(
            "invalid base url `{}`: {reason}",
            redact_url_credentials(input)
        ),
    }
}

pub(crate) fn validate_method_name(method: &str) -> Result<(), Error> {
    let mut chars = method.chars();
    let Some(first) = chars.next() else {
        return Err(Error::InvalidMethodName {
            method: method.to_owned(),
        });
    };

    if !first.is_ascii_lowercase() || !chars.all(|character| character.is_ascii_alphanumeric()) {
        return Err(Error::InvalidMethodName {
            method: method.to_owned(),
        });
    }
    Ok(())
}

pub(crate) fn build_api_path(token: &str, method: &str) -> String {
    format!("/bot{token}/{method}")
}

pub(crate) fn redact_token(input: &str, token: &str) -> String {
    if token.is_empty() {
        return input.to_owned();
    }

    input.replace(token, REDACTED_TOKEN)
}

pub(crate) fn redact_url_credentials(input: &str) -> String {
    if let Ok(mut url) = Url::parse(input) {
        if (!url.username().is_empty() || url.password().is_some())
            && url.set_username(REDACTED_CREDENTIAL).is_err()
        {
            return redact_url_credentials_fallback(input);
        }
        if url.password().is_some() && url.set_password(Some(REDACTED_CREDENTIAL)).is_err() {
            return redact_url_credentials_fallback(input);
        }
        return url.to_string();
    }

    redact_url_credentials_fallback(input)
}

fn redact_url_credentials_fallback(input: &str) -> String {
    let Some(scheme_end) = input.find("://") else {
        return input.to_owned();
    };
    let authority_start = scheme_end + 3;
    let rest = &input[authority_start..];
    let authority_end = rest
        .find(['/', '?', '#'])
        .map(|index| authority_start + index)
        .unwrap_or(input.len());
    let authority = &input[authority_start..authority_end];
    let Some(at_index) = authority.rfind('@') else {
        return input.to_owned();
    };

    let credentials_end = authority_start + at_index + 1;
    format!(
        "{}{}{}",
        &input[..authority_start],
        "redacted@",
        &input[credentials_end..]
    )
}

pub(crate) fn body_snippet(body: &[u8], max_chars: usize) -> Option<String> {
    if body.is_empty() || max_chars == 0 {
        return None;
    }

    let text = String::from_utf8_lossy(body);
    if text.is_empty() {
        return None;
    }

    let mut output = String::new();
    for (index, character) in text.chars().enumerate() {
        if index >= max_chars {
            output.push_str("...(truncated)");
            break;
        }
        output.push(character);
    }

    if output.is_empty() {
        None
    } else {
        Some(output)
    }
}

pub(crate) fn request_id_from_headers(headers: &HeaderMap) -> Option<String> {
    ["x-request-id", "x-telegram-request-id", "x-amzn-requestid"]
        .into_iter()
        .find_map(|name| {
            headers
                .get(name)
                .and_then(|value| value.to_str().ok())
                .map(ToOwned::to_owned)
        })
}

pub(crate) fn retry_after_or_backoff(
    error: &Error,
    backoff: impl FnOnce() -> Duration,
) -> Duration {
    error.retry_after().unwrap_or_else(backoff)
}

pub(crate) fn exponential_backoff(base: Duration, max: Duration, attempt: usize) -> Duration {
    let exponent = attempt.saturating_sub(1).min(16);
    let factor = 2u32.saturating_pow(exponent as u32);
    base.saturating_mul(factor).min(max)
}

pub(crate) fn jittered_duration(delay: Duration, jitter_ratio: f64, max: Duration) -> Duration {
    let delay = delay.min(max);
    if delay.is_zero() || jitter_ratio <= 0.0 {
        return delay;
    }

    jittered_duration_with_unit(delay, jitter_ratio, max, jitter_unit())
}

fn jitter_unit() -> f64 {
    let now_nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0_u128, |duration| duration.as_nanos());
    (now_nanos % 10_000) as f64 / 10_000.0
}

fn jittered_duration_with_unit(
    delay: Duration,
    jitter_ratio: f64,
    max: Duration,
    unit: f64,
) -> Duration {
    let ratio = jitter_ratio.clamp(0.0, 1.0);
    let unit = unit.clamp(0.0, 1.0);
    let multiplier = (1.0 - ratio) + (2.0 * ratio * unit);
    duration_from_secs_f64_saturating(delay.as_secs_f64() * multiplier).min(max)
}

fn duration_from_secs_f64_saturating(seconds: f64) -> Duration {
    match Duration::try_from_secs_f64(seconds) {
        Ok(duration) => duration,
        Err(_) => Duration::MAX,
    }
}

#[cfg(feature = "tracing")]
pub(crate) fn duration_millis_u64(duration: Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{
        body_snippet, exponential_backoff, normalize_base_url, redact_token,
        redact_url_credentials, retry_after_or_backoff, validate_method_name,
    };
    use crate::Error;

    #[test]
    fn redacts_token_in_text() {
        let redacted = redact_token("/bot123:abc/getMe", "123:abc");
        assert_eq!(redacted, "/bot<redacted-token>/getMe");
    }

    #[test]
    fn redacts_url_credentials() {
        let redacted = redact_url_credentials("postgres://user:secret@example.com/db");
        assert_eq!(redacted, "postgres://redacted:redacted@example.com/db");

        let redacted = redact_url_credentials("redis://:secret@example.com/0");
        assert_eq!(redacted, "redis://redacted:redacted@example.com/0");
    }

    #[test]
    fn validates_telegram_method_names() {
        for method in ["getMe", "sendMessage", "logOut", "close"] {
            assert!(validate_method_name(method).is_ok());
        }

        for method in ["", "_getMe", "123", "get_me", "GetMe", "get/me"] {
            assert!(matches!(
                validate_method_name(method),
                Err(Error::InvalidMethodName { .. })
            ));
        }
    }

    #[test]
    fn validates_base_url_shape_before_transport_build() -> Result<(), String> {
        assert!(normalize_base_url("https://api.telegram.org/bot-api").is_ok());
        assert!(normalize_base_url("http://127.0.0.1:8081").is_ok());

        for input in [
            "",
            " https://api.telegram.org",
            "https://api.telegram.org ",
            "https://user:secret@api.telegram.org",
            "https://api.telegram.org?token=secret",
            "https://api.telegram.org#fragment",
            "unix:///var/run/socket",
        ] {
            let error = match normalize_base_url(input) {
                Ok(_) => return Err(format!("expected invalid base url `{input}`")),
                Err(error) => error,
            };
            assert_eq!(error.classification(), crate::ErrorClass::Configuration);
            assert!(
                !error.to_string().contains("user:secret"),
                "base url errors must redact credentials"
            );
        }

        Ok(())
    }

    #[test]
    fn truncates_snippet() {
        let snippet = body_snippet(b"abcdefghijklmnopqrstuvwxyz", 5);
        assert_eq!(snippet.as_deref(), Some("abcde...(truncated)"));
    }

    #[test]
    fn retry_delay_respects_provider_retry_after() {
        let error = Error::Transport {
            method: "sendMessage".to_owned(),
            status: Some(429),
            request_id: None,
            retry_after: Some(Duration::from_secs(30)),
            request_path: None,
            message: "rate limited".into(),
        };

        let delay = retry_after_or_backoff(&error, || Duration::from_millis(200));

        assert_eq!(delay, Duration::from_secs(30));
    }

    #[test]
    fn retry_delay_uses_local_backoff_without_retry_after() {
        let error = Error::Transport {
            method: "getMe".to_owned(),
            status: Some(503),
            request_id: None,
            retry_after: None,
            request_path: None,
            message: "unavailable".into(),
        };

        let delay = retry_after_or_backoff(&error, || Duration::from_millis(200));

        assert_eq!(delay, Duration::from_millis(200));
    }

    #[test]
    fn exponential_backoff_saturates_and_clamps_to_max() {
        assert_eq!(
            exponential_backoff(Duration::from_millis(100), Duration::from_secs(10), 0),
            Duration::from_millis(100)
        );
        assert_eq!(
            exponential_backoff(Duration::from_millis(100), Duration::from_secs(10), 3),
            Duration::from_millis(400)
        );
        assert_eq!(
            exponential_backoff(Duration::from_secs(1), Duration::from_secs(5), 16),
            Duration::from_secs(5)
        );
        assert_eq!(
            exponential_backoff(Duration::MAX, Duration::MAX, usize::MAX),
            Duration::MAX
        );
    }

    #[test]
    fn jittered_duration_applies_bounds_without_panicking() {
        assert_eq!(
            super::jittered_duration_with_unit(
                Duration::from_secs(10),
                0.5,
                Duration::from_secs(100),
                1.0,
            ),
            Duration::from_secs(15)
        );
        assert_eq!(
            super::jittered_duration_with_unit(
                Duration::from_secs(10),
                1.0,
                Duration::from_secs(15),
                1.0,
            ),
            Duration::from_secs(15)
        );
        assert_eq!(
            super::jittered_duration_with_unit(Duration::MAX, 1.0, Duration::MAX, 1.0),
            Duration::MAX
        );
    }

    #[cfg(feature = "tracing")]
    #[test]
    fn duration_millis_u64_saturates_instead_of_truncating() {
        assert_eq!(super::duration_millis_u64(Duration::from_millis(42)), 42);
        assert_eq!(super::duration_millis_u64(Duration::MAX), u64::MAX);
    }
}

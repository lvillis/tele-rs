use std::time::Duration;

use http::Uri;
use http::header::{HeaderName, HeaderValue, USER_AGENT};

use crate::Error;
use crate::auth::{Auth, BotToken};
use crate::client::{ClientMetric, ClientObservability};
use crate::util::{normalize_base_url, redact_url_credentials};

const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(35);
const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const DEFAULT_TOTAL_TIMEOUT: Duration = Duration::from_secs(45);
const DEFAULT_MAX_RESPONSE_BODY_BYTES: usize = 8 * 1024 * 1024;
const DEFAULT_BODY_SNIPPET_LIMIT: usize = 2048;

/// Retry settings surfaced by `tele`.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct RetryConfig {
    pub max_attempts: usize,
    pub base_backoff: Duration,
    /// Maximum locally computed exponential backoff.
    ///
    /// Provider supplied `Retry-After` values are honored separately and are not clamped by this.
    pub max_backoff: Duration,
    pub jitter_ratio: f64,
    pub allow_non_idempotent_retries: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_backoff: Duration::from_millis(200),
            max_backoff: Duration::from_secs(2),
            jitter_ratio: 0.2,
            allow_non_idempotent_retries: false,
        }
    }
}

impl RetryConfig {
    pub fn validate(&self) -> Result<(), Error> {
        if self.max_attempts == 0 {
            return Err(Error::Configuration {
                reason: "retry max_attempts must be at least 1".to_owned(),
            });
        }
        if self.base_backoff.is_zero() {
            return Err(Error::Configuration {
                reason: "retry base_backoff must be greater than zero".to_owned(),
            });
        }
        if self.max_backoff.is_zero() {
            return Err(Error::Configuration {
                reason: "retry max_backoff must be greater than zero".to_owned(),
            });
        }
        if !self.jitter_ratio.is_finite() || !(0.0..=1.0).contains(&self.jitter_ratio) {
            return Err(Error::Configuration {
                reason: "retry jitter_ratio must be finite and between 0.0 and 1.0".to_owned(),
            });
        }
        if self.base_backoff > self.max_backoff {
            return Err(Error::Configuration {
                reason: "retry base_backoff must not exceed max_backoff".to_owned(),
            });
        }

        Ok(())
    }
}

/// Token bucket rate limit settings.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct RateLimitConfig {
    pub requests_per_second: f64,
    pub burst: usize,
    pub max_throttle_delay: Duration,
}

impl RateLimitConfig {
    pub fn standard() -> Self {
        Self {
            requests_per_second: 30.0,
            burst: 30,
            max_throttle_delay: Duration::from_secs(30),
        }
    }

    pub fn new(
        requests_per_second: f64,
        burst: usize,
        max_throttle_delay: Duration,
    ) -> Result<Self, Error> {
        let config = Self {
            requests_per_second,
            burst,
            max_throttle_delay,
        };
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), Error> {
        if !self.requests_per_second.is_finite() || self.requests_per_second <= 0.0 {
            return Err(Error::Configuration {
                reason: "rate limit requests_per_second must be finite and greater than 0"
                    .to_owned(),
            });
        }
        if self.burst == 0 {
            return Err(Error::Configuration {
                reason: "rate limit burst must be at least 1".to_owned(),
            });
        }
        if self.max_throttle_delay.is_zero() {
            return Err(Error::Configuration {
                reason: "rate limit max_throttle_delay must be greater than zero".to_owned(),
            });
        }

        Ok(())
    }
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self::standard()
    }
}

/// Request defaults used by transport.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct RequestDefaults {
    pub(crate) request_timeout: Duration,
    pub(crate) total_timeout: Option<Duration>,
    pub(crate) connect_timeout: Duration,
    pub(crate) max_response_body_bytes: usize,
    pub(crate) capture_body_snippet: bool,
    pub(crate) body_snippet_limit: usize,
    pub(crate) retry: RetryConfig,
    pub(crate) global_rate_limit: Option<RateLimitConfig>,
    pub(crate) per_host_rate_limit: Option<RateLimitConfig>,
    pub(crate) max_in_flight: Option<usize>,
    pub(crate) max_in_flight_per_host: Option<usize>,
    pub(crate) http_proxy: Option<Uri>,
    pub(crate) proxy_authorization: Option<HeaderValue>,
    pub(crate) no_proxy_rules: Vec<String>,
}

impl Default for RequestDefaults {
    fn default() -> Self {
        Self {
            request_timeout: DEFAULT_REQUEST_TIMEOUT,
            total_timeout: Some(DEFAULT_TOTAL_TIMEOUT),
            connect_timeout: DEFAULT_CONNECT_TIMEOUT,
            max_response_body_bytes: DEFAULT_MAX_RESPONSE_BODY_BYTES,
            capture_body_snippet: true,
            body_snippet_limit: DEFAULT_BODY_SNIPPET_LIMIT,
            retry: RetryConfig::default(),
            global_rate_limit: Some(RateLimitConfig::standard()),
            per_host_rate_limit: None,
            max_in_flight: Some(256),
            max_in_flight_per_host: Some(64),
            http_proxy: None,
            proxy_authorization: None,
            no_proxy_rules: Vec::new(),
        }
    }
}

impl RequestDefaults {
    pub(crate) fn validate(&self) -> Result<(), Error> {
        if self.request_timeout.is_zero() {
            return Err(Error::Configuration {
                reason: "request_timeout must be greater than zero".to_owned(),
            });
        }
        if self.connect_timeout.is_zero() {
            return Err(Error::Configuration {
                reason: "connect_timeout must be greater than zero".to_owned(),
            });
        }
        if self.total_timeout.is_some_and(|timeout| timeout.is_zero()) {
            return Err(Error::Configuration {
                reason: "total_timeout must be greater than zero when set".to_owned(),
            });
        }
        if self.max_response_body_bytes == 0 {
            return Err(Error::Configuration {
                reason: "max_response_body_bytes must be at least 1".to_owned(),
            });
        }
        if self.capture_body_snippet && self.body_snippet_limit == 0 {
            return Err(Error::Configuration {
                reason: "body_snippet_limit must be at least 1 when body snippets are enabled"
                    .to_owned(),
            });
        }
        self.retry.validate()?;
        if let Some(rate_limit) = self.global_rate_limit.as_ref() {
            rate_limit.validate()?;
        }
        if let Some(rate_limit) = self.per_host_rate_limit.as_ref() {
            rate_limit.validate()?;
        }
        if self.max_in_flight.is_some_and(|max| max == 0) {
            return Err(Error::Configuration {
                reason: "max_in_flight must be at least 1 when set".to_owned(),
            });
        }
        if self.max_in_flight_per_host.is_some_and(|max| max == 0) {
            return Err(Error::Configuration {
                reason: "max_in_flight_per_host must be at least 1 when set".to_owned(),
            });
        }

        Ok(())
    }
}

pub(crate) struct BuilderParts {
    pub(crate) base_url: String,
    pub(crate) auth: Auth,
    pub(crate) defaults: RequestDefaults,
    pub(crate) default_headers: Vec<(String, String)>,
    pub(crate) observability: ClientObservability,
}

/// SDK client builder.
pub struct ClientBuilder {
    base_url: String,
    auth: Auth,
    defaults: RequestDefaults,
    default_headers: Vec<(String, String)>,
    observability: ClientObservability,
}

impl ClientBuilder {
    pub(crate) fn new(base_url: impl AsRef<str>) -> Result<Self, Error> {
        let normalized = normalize_base_url(base_url.as_ref())?;

        Ok(Self {
            base_url: normalized.to_string(),
            auth: Auth::none(),
            defaults: RequestDefaults::default(),
            default_headers: Vec::new(),
            observability: ClientObservability::default(),
        })
    }

    /// Sets authentication mode.
    pub fn auth(mut self, auth: Auth) -> Self {
        self.auth = auth;
        self
    }

    /// Convenience setter for bot token.
    pub fn bot_token(mut self, token: impl Into<String>) -> Result<Self, Error> {
        self.auth = Auth::BotToken(BotToken::new(token)?);
        Ok(self)
    }

    /// Sets default per-phase timeout.
    pub fn request_timeout(mut self, timeout: Duration) -> Self {
        self.defaults.request_timeout = timeout;
        self
    }

    /// Sets total end-to-end timeout.
    pub fn total_timeout(mut self, timeout: Option<Duration>) -> Self {
        self.defaults.total_timeout = timeout;
        self
    }

    /// Sets connect timeout.
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.defaults.connect_timeout = timeout;
        self
    }

    /// Max response bytes accepted from Telegram.
    pub fn max_response_body_bytes(mut self, max_bytes: usize) -> Self {
        self.defaults.max_response_body_bytes = max_bytes;
        self
    }

    /// Enables or disables body snippet capture in errors.
    pub fn capture_body_snippet(mut self, enabled: bool) -> Self {
        self.defaults.capture_body_snippet = enabled;
        self
    }

    /// Max chars to keep for body snippets.
    pub fn body_snippet_limit(mut self, max_chars: usize) -> Self {
        self.defaults.body_snippet_limit = max_chars;
        self
    }

    /// Retry policy.
    pub fn retry_config(mut self, retry: RetryConfig) -> Result<Self, Error> {
        retry.validate()?;
        self.defaults.retry = retry;
        Ok(self)
    }

    /// Global local-side token bucket limiter.
    pub fn global_rate_limit(mut self, rate_limit: Option<RateLimitConfig>) -> Result<Self, Error> {
        if let Some(rate_limit) = rate_limit.as_ref() {
            rate_limit.validate()?;
        }
        self.defaults.global_rate_limit = rate_limit;
        Ok(self)
    }

    /// Per-host local-side token bucket limiter.
    pub fn per_host_rate_limit(
        mut self,
        rate_limit: Option<RateLimitConfig>,
    ) -> Result<Self, Error> {
        if let Some(rate_limit) = rate_limit.as_ref() {
            rate_limit.validate()?;
        }
        self.defaults.per_host_rate_limit = rate_limit;
        Ok(self)
    }

    /// Maximum total in-flight requests.
    pub fn max_in_flight(mut self, max: Option<usize>) -> Self {
        self.defaults.max_in_flight = max;
        self
    }

    /// Maximum in-flight requests per host.
    pub fn max_in_flight_per_host(mut self, max: Option<usize>) -> Self {
        self.defaults.max_in_flight_per_host = max;
        self
    }

    /// Adds one default header after validating it.
    pub fn default_header(
        mut self,
        name: impl AsRef<str>,
        value: impl AsRef<str>,
    ) -> Result<Self, Error> {
        let name = name.as_ref().to_owned();
        let value = value.as_ref().to_owned();

        HeaderName::from_bytes(name.as_bytes()).map_err(|source| Error::InvalidHeaderName {
            name: name.clone(),
            source,
        })?;
        HeaderValue::from_str(&value).map_err(|source| Error::InvalidHeaderValue {
            name: name.clone(),
            source,
        })?;

        self.default_headers.push((name, value));
        Ok(self)
    }

    /// Sets the `user-agent` header.
    pub fn user_agent(self, value: impl AsRef<str>) -> Result<Self, Error> {
        self.default_header(USER_AGENT.as_str(), value.as_ref())
    }

    /// Installs a hook that receives one metric per completed Telegram API request.
    pub fn on_metric<F>(mut self, hook: F) -> Self
    where
        F: Fn(&ClientMetric) + Send + Sync + 'static,
    {
        self.observability.on_metric = Some(std::sync::Arc::new(hook));
        self
    }

    /// Sets HTTP proxy URI.
    pub fn http_proxy(mut self, proxy_uri: impl AsRef<str>) -> Result<Self, Error> {
        let raw = proxy_uri.as_ref().trim();
        let parsed = raw.parse::<Uri>().map_err(|source| {
            invalid_http_proxy_uri(raw, format!("failed to parse proxy uri: {source}"))
        })?;
        validate_http_proxy_uri(raw, &parsed)?;
        self.defaults.http_proxy = Some(parsed);
        Ok(self)
    }

    /// Sets HTTP proxy URI using a pre-parsed value.
    pub fn http_proxy_uri(mut self, proxy_uri: Uri) -> Result<Self, Error> {
        validate_http_proxy_uri(&proxy_uri.to_string(), &proxy_uri)?;
        self.defaults.http_proxy = Some(proxy_uri);
        Ok(self)
    }

    /// Clears configured HTTP proxy URI.
    pub fn clear_http_proxy(mut self) -> Self {
        self.defaults.http_proxy = None;
        self
    }

    /// Sets proxy authorization header value.
    pub fn proxy_authorization(mut self, value: impl AsRef<str>) -> Result<Self, Error> {
        let mut parsed =
            HeaderValue::from_str(value.as_ref()).map_err(|source| Error::InvalidHeaderValue {
                name: "proxy-authorization".to_owned(),
                source,
            })?;
        parsed.set_sensitive(true);
        self.defaults.proxy_authorization = Some(parsed);
        Ok(self)
    }

    /// Clears configured proxy authorization header.
    pub fn clear_proxy_authorization(mut self) -> Self {
        self.defaults.proxy_authorization = None;
        self
    }

    /// Replaces NO_PROXY rules.
    pub fn no_proxy<I, S>(mut self, rules: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        self.defaults.no_proxy_rules = rules
            .into_iter()
            .map(|rule| rule.as_ref().trim().to_owned())
            .filter(|rule| !rule.is_empty())
            .collect();
        self
    }

    /// Appends a NO_PROXY rule.
    pub fn add_no_proxy(mut self, rule: impl AsRef<str>) -> Self {
        let rule = rule.as_ref().trim();
        if !rule.is_empty() {
            self.defaults.no_proxy_rules.push(rule.to_owned());
        }
        self
    }

    /// Clears all NO_PROXY rules.
    pub fn clear_no_proxy(mut self) -> Self {
        self.defaults.no_proxy_rules.clear();
        self
    }

    pub(crate) fn into_parts(self) -> BuilderParts {
        BuilderParts {
            base_url: self.base_url,
            auth: self.auth,
            defaults: self.defaults,
            default_headers: self.default_headers,
            observability: self.observability,
        }
    }

    /// Validates all transport defaults configured on this builder.
    pub fn validate(&self) -> Result<(), Error> {
        self.defaults.validate()
    }

    /// Builds async client.
    #[cfg(feature = "_async")]
    pub fn build(self) -> Result<super::async_client::Client, Error> {
        super::async_client::Client::from_builder(self)
    }

    /// Builds blocking client.
    #[cfg(feature = "_blocking")]
    pub fn build_blocking(self) -> Result<super::blocking_client::BlockingClient, Error> {
        super::blocking_client::BlockingClient::from_builder(self)
    }
}

fn validate_http_proxy_uri(raw: &str, parsed: &Uri) -> Result<(), Error> {
    let Some(scheme) = parsed.scheme_str() else {
        return Err(invalid_http_proxy_uri(
            raw,
            "proxy uri must include an explicit scheme",
        ));
    };
    if !scheme.eq_ignore_ascii_case("http") {
        return Err(invalid_http_proxy_uri(
            raw,
            "proxy uri must use http scheme",
        ));
    }
    if parsed.host().is_none() {
        return Err(invalid_http_proxy_uri(raw, "proxy uri must include host"));
    }
    if let Some(path_and_query) = parsed.path_and_query() {
        let path = path_and_query.path();
        if !path.is_empty() && path != "/" {
            return Err(invalid_http_proxy_uri(
                raw,
                "proxy uri must not include path segments",
            ));
        }
        if path_and_query.query().is_some() {
            return Err(invalid_http_proxy_uri(
                raw,
                "proxy uri must not include query parameters",
            ));
        }
    }

    Ok(())
}

fn invalid_http_proxy_uri(raw: &str, detail: impl std::fmt::Display) -> Error {
    Error::InvalidRequest {
        reason: format!(
            "invalid http proxy uri `{}`: {detail}",
            redact_url_credentials(raw)
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_invalid_http_proxy_uri() -> Result<(), Error> {
        let result = ClientBuilder::new("https://api.telegram.org")?.http_proxy("not-a-uri");
        assert!(result.is_err());
        let error = match result {
            Ok(_) => Error::InvalidRequest {
                reason: "expected proxy parsing error".to_owned(),
            },
            Err(error) => error,
        };

        assert!(matches!(error, Error::InvalidRequest { .. }));
        assert!(error.to_string().contains("invalid http proxy uri"));
        Ok(())
    }

    #[test]
    fn redacts_proxy_uri_credentials_in_validation_errors() -> Result<(), Error> {
        let result = ClientBuilder::new("https://api.telegram.org")?
            .http_proxy("http://user:secret@127.0.0.1:8080/proxy");
        let error = match result {
            Ok(_) => Error::InvalidRequest {
                reason: "expected proxy validation error".to_owned(),
            },
            Err(error) => error,
        };
        let rendered = error.to_string();

        assert!(rendered.contains("invalid http proxy uri"));
        assert!(!rendered.contains("user:secret"));
        assert!(!rendered.contains("secret"));
        assert!(rendered.contains("redacted"));
        Ok(())
    }

    #[test]
    fn stores_proxy_and_no_proxy_settings() -> Result<(), Error> {
        let builder = ClientBuilder::new("https://api.telegram.org")?
            .http_proxy("http://127.0.0.1:8080")?
            .proxy_authorization("Basic dXNlcjpwYXNz")?
            .no_proxy(["localhost", ".example.com"])
            .add_no_proxy("127.0.0.1");
        let parts = builder.into_parts();

        assert!(parts.defaults.http_proxy.is_some());
        assert!(parts.defaults.proxy_authorization.is_some());
        assert_eq!(
            parts.defaults.no_proxy_rules,
            vec!["localhost", ".example.com", "127.0.0.1"]
        );
        Ok(())
    }

    #[test]
    fn rejects_invalid_retry_config() -> Result<(), Error> {
        let retry = RetryConfig {
            max_attempts: 0,
            ..RetryConfig::default()
        };
        let result = ClientBuilder::new("https://api.telegram.org")?.retry_config(retry);
        assert!(matches!(result, Err(Error::Configuration { .. })));

        let retry = RetryConfig {
            jitter_ratio: f64::NAN,
            ..RetryConfig::default()
        };
        let result = ClientBuilder::new("https://api.telegram.org")?.retry_config(retry);
        assert!(matches!(result, Err(Error::Configuration { .. })));

        let retry = RetryConfig {
            base_backoff: Duration::ZERO,
            ..RetryConfig::default()
        };
        let result = ClientBuilder::new("https://api.telegram.org")?.retry_config(retry);
        assert!(matches!(result, Err(Error::Configuration { .. })));

        let retry = RetryConfig {
            base_backoff: Duration::from_secs(2),
            max_backoff: Duration::from_secs(1),
            ..RetryConfig::default()
        };
        let result = ClientBuilder::new("https://api.telegram.org")?.retry_config(retry);
        assert!(matches!(result, Err(Error::Configuration { .. })));
        Ok(())
    }

    #[test]
    fn rejects_invalid_rate_limit_config() -> Result<(), Error> {
        let invalid_rate = RateLimitConfig {
            requests_per_second: f64::INFINITY,
            ..RateLimitConfig::standard()
        };
        let result =
            ClientBuilder::new("https://api.telegram.org")?.global_rate_limit(Some(invalid_rate));
        assert!(matches!(result, Err(Error::Configuration { .. })));

        let invalid_burst = RateLimitConfig {
            burst: 0,
            ..RateLimitConfig::standard()
        };
        let result = ClientBuilder::new("https://api.telegram.org")?
            .per_host_rate_limit(Some(invalid_burst));
        assert!(matches!(result, Err(Error::Configuration { .. })));

        assert!(matches!(
            RateLimitConfig::new(1.0, 1, Duration::ZERO),
            Err(Error::Configuration { .. })
        ));
        Ok(())
    }

    #[test]
    fn validates_request_defaults_without_silent_clamping() -> Result<(), Error> {
        let builder = ClientBuilder::new("https://api.telegram.org")?;
        assert!(builder.validate().is_ok());

        let zero_request_timeout =
            ClientBuilder::new("https://api.telegram.org")?.request_timeout(Duration::ZERO);
        assert!(matches!(
            zero_request_timeout.validate(),
            Err(Error::Configuration { .. })
        ));

        let zero_total_timeout =
            ClientBuilder::new("https://api.telegram.org")?.total_timeout(Some(Duration::ZERO));
        assert!(matches!(
            zero_total_timeout.validate(),
            Err(Error::Configuration { .. })
        ));

        let zero_connect_timeout =
            ClientBuilder::new("https://api.telegram.org")?.connect_timeout(Duration::ZERO);
        assert!(matches!(
            zero_connect_timeout.validate(),
            Err(Error::Configuration { .. })
        ));

        let zero_body = ClientBuilder::new("https://api.telegram.org")?.max_response_body_bytes(0);
        assert!(matches!(
            zero_body.validate(),
            Err(Error::Configuration { .. })
        ));

        let zero_snippet = ClientBuilder::new("https://api.telegram.org")?.body_snippet_limit(0);
        assert!(matches!(
            zero_snippet.validate(),
            Err(Error::Configuration { .. })
        ));

        let zero_in_flight = ClientBuilder::new("https://api.telegram.org")?.max_in_flight(Some(0));
        assert!(matches!(
            zero_in_flight.validate(),
            Err(Error::Configuration { .. })
        ));

        let zero_in_flight_per_host =
            ClientBuilder::new("https://api.telegram.org")?.max_in_flight_per_host(Some(0));
        assert!(matches!(
            zero_in_flight_per_host.validate(),
            Err(Error::Configuration { .. })
        ));

        Ok(())
    }

    #[test]
    fn validates_preparsed_http_proxy_uri() -> Result<(), Error> {
        let parsed = match "https://127.0.0.1:8080".parse::<Uri>() {
            Ok(parsed) => parsed,
            Err(source) => {
                return Err(Error::InvalidRequest {
                    reason: format!("failed to parse test uri: {source}"),
                });
            }
        };
        let result = ClientBuilder::new("https://api.telegram.org")?.http_proxy_uri(parsed);
        assert!(result.is_err());
        Ok(())
    }

    #[cfg(feature = "bot")]
    #[test]
    fn default_timeouts_leave_headroom_for_long_polling() {
        let poll_timeout = Duration::from_secs(u64::from(
            crate::bot::PollingConfig::default().poll_timeout_seconds,
        ));
        assert!(
            DEFAULT_REQUEST_TIMEOUT > poll_timeout,
            "request timeout must exceed long-poll timeout"
        );
        assert!(
            DEFAULT_TOTAL_TIMEOUT > DEFAULT_REQUEST_TIMEOUT,
            "total timeout must exceed request timeout"
        );
    }
}

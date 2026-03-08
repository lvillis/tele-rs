use std::time::Duration;

use http::Uri;
use http::header::{HeaderName, HeaderValue, USER_AGENT};

use crate::Error;
use crate::auth::{Auth, BotToken};
use crate::client::{ClientMetric, ClientObservability};
use crate::util::normalize_base_url;

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
        self.defaults.request_timeout = timeout.max(Duration::from_millis(1));
        self
    }

    /// Sets total end-to-end timeout.
    pub fn total_timeout(mut self, timeout: Option<Duration>) -> Self {
        self.defaults.total_timeout = timeout.map(|value| value.max(Duration::from_millis(1)));
        self
    }

    /// Sets connect timeout.
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.defaults.connect_timeout = timeout.max(Duration::from_millis(1));
        self
    }

    /// Max response bytes accepted from Telegram.
    pub fn max_response_body_bytes(mut self, max_bytes: usize) -> Self {
        self.defaults.max_response_body_bytes = max_bytes.max(1);
        self
    }

    /// Enables or disables body snippet capture in errors.
    pub fn capture_body_snippet(mut self, enabled: bool) -> Self {
        self.defaults.capture_body_snippet = enabled;
        self
    }

    /// Max chars to keep for body snippets.
    pub fn body_snippet_limit(mut self, max_chars: usize) -> Self {
        self.defaults.body_snippet_limit = max_chars.max(1);
        self
    }

    /// Retry policy.
    pub fn retry_config(mut self, retry: RetryConfig) -> Self {
        self.defaults.retry = retry;
        self
    }

    /// Global local-side token bucket limiter.
    pub fn global_rate_limit(mut self, rate_limit: Option<RateLimitConfig>) -> Self {
        self.defaults.global_rate_limit = rate_limit;
        self
    }

    /// Per-host local-side token bucket limiter.
    pub fn per_host_rate_limit(mut self, rate_limit: Option<RateLimitConfig>) -> Self {
        self.defaults.per_host_rate_limit = rate_limit;
        self
    }

    /// Maximum total in-flight requests.
    pub fn max_in_flight(mut self, max: Option<usize>) -> Self {
        self.defaults.max_in_flight = max.map(|value| value.max(1));
        self
    }

    /// Maximum in-flight requests per host.
    pub fn max_in_flight_per_host(mut self, max: Option<usize>) -> Self {
        self.defaults.max_in_flight_per_host = max.map(|value| value.max(1));
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
        let parsed = raw.parse::<Uri>().map_err(|source| Error::InvalidRequest {
            reason: format!("invalid http proxy uri `{raw}`: {source}"),
        })?;
        let Some(scheme) = parsed.scheme_str() else {
            return Err(Error::InvalidRequest {
                reason: format!(
                    "invalid http proxy uri `{raw}`: proxy uri must include an explicit scheme"
                ),
            });
        };
        if !scheme.eq_ignore_ascii_case("http") {
            return Err(Error::InvalidRequest {
                reason: format!("invalid http proxy uri `{raw}`: proxy uri must use http scheme"),
            });
        }
        if parsed.host().is_none() {
            return Err(Error::InvalidRequest {
                reason: format!("invalid http proxy uri `{raw}`: proxy uri must include host"),
            });
        }
        if let Some(path_and_query) = parsed.path_and_query() {
            let path = path_and_query.path();
            if !path.is_empty() && path != "/" {
                return Err(Error::InvalidRequest {
                    reason: format!(
                        "invalid http proxy uri `{raw}`: proxy uri must not include path segments"
                    ),
                });
            }
            if path_and_query.query().is_some() {
                return Err(Error::InvalidRequest {
                    reason: format!(
                        "invalid http proxy uri `{raw}`: proxy uri must not include query parameters"
                    ),
                });
            }
        }
        self.defaults.http_proxy = Some(parsed);
        Ok(self)
    }

    /// Sets HTTP proxy URI using a pre-parsed value.
    pub fn http_proxy_uri(mut self, proxy_uri: Uri) -> Self {
        self.defaults.http_proxy = Some(proxy_uri);
        self
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

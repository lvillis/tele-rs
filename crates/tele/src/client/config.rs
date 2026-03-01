use std::time::Duration;

use http::header::{HeaderName, HeaderValue, USER_AGENT};

use crate::Error;
use crate::auth::{Auth, BotToken};
use crate::util::normalize_base_url;

const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const DEFAULT_TOTAL_TIMEOUT: Duration = Duration::from_secs(30);
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
        }
    }
}

pub(crate) struct BuilderParts {
    pub(crate) base_url: String,
    pub(crate) auth: Auth,
    pub(crate) defaults: RequestDefaults,
    pub(crate) default_headers: Vec<(String, String)>,
}

/// SDK client builder.
pub struct ClientBuilder {
    base_url: String,
    auth: Auth,
    defaults: RequestDefaults,
    default_headers: Vec<(String, String)>,
}

impl ClientBuilder {
    pub(crate) fn new(base_url: impl AsRef<str>) -> Result<Self, Error> {
        let normalized = normalize_base_url(base_url.as_ref())?;

        Ok(Self {
            base_url: normalized.to_string(),
            auth: Auth::none(),
            defaults: RequestDefaults::default(),
            default_headers: Vec::new(),
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

    pub(crate) fn into_parts(self) -> BuilderParts {
        BuilderParts {
            base_url: self.base_url,
            auth: self.auth,
            defaults: self.defaults,
            default_headers: self.default_headers,
        }
    }

    /// Builds async client.
    #[cfg(feature = "async")]
    pub fn build(self) -> Result<super::async_client::Client, Error> {
        super::async_client::Client::from_builder(self)
    }

    /// Builds blocking client.
    #[cfg(feature = "blocking")]
    pub fn build_blocking(self) -> Result<super::blocking_client::BlockingClient, Error> {
        super::blocking_client::BlockingClient::from_builder(self)
    }
}

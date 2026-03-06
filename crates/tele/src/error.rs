use std::time::Duration;

use http::StatusCode;
use thiserror::Error;

use crate::types::common::ResponseParameters;

/// Result type returned by `tele`.
pub type Result<T> = std::result::Result<T, Error>;

/// High-level error class for policy decisions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ErrorClass {
    Configuration,
    Validation,
    Authentication,
    RateLimited,
    Transport,
    Api,
    Decode,
    Protocol,
}

/// SDK error with transport, protocol, and API-level variants.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    #[error("invalid base url `{input}`: {source}")]
    InvalidBaseUrl {
        input: String,
        #[source]
        source: url::ParseError,
    },

    #[error("base url must use http or https, got `{scheme}`")]
    InvalidBaseUrlScheme { scheme: String },

    #[error("invalid Telegram method name `{method}`")]
    InvalidMethodName { method: String },

    #[error("invalid Telegram bot token")]
    InvalidBotToken,

    #[error("missing bot token authentication")]
    MissingBotToken,

    #[error("invalid request: {reason}")]
    InvalidRequest { reason: String },

    #[error("invalid client configuration: {reason}")]
    Configuration { reason: String },

    #[error("invalid default header name `{name}`: {source}")]
    InvalidHeaderName {
        name: String,
        #[source]
        source: http::header::InvalidHeaderName,
    },

    #[error("invalid default header value for `{name}`: {source}")]
    InvalidHeaderValue {
        name: String,
        #[source]
        source: http::header::InvalidHeaderValue,
    },

    #[error("failed to serialize request body: {source}")]
    SerializeRequest {
        #[source]
        source: serde_json::Error,
    },

    #[error("failed to read local file `{path}`: {source}")]
    ReadLocalFile {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to deserialize Telegram response for `{method}`")]
    DeserializeResponse {
        method: String,
        status: Option<u16>,
        request_id: Option<Box<str>>,
        body_snippet: Option<Box<str>>,
        #[source]
        source: serde_json::Error,
    },

    #[error("transport error while calling `{method}`: {message}")]
    Transport {
        method: String,
        status: Option<u16>,
        request_id: Option<Box<str>>,
        retry_after: Option<Duration>,
        request_path: Option<Box<str>>,
        message: Box<str>,
    },

    #[error("telegram api error while calling `{method}`: {description}")]
    Api {
        method: String,
        status: Option<u16>,
        request_id: Option<Box<str>>,
        error_code: Option<i64>,
        description: Box<str>,
        parameters: Option<Box<ResponseParameters>>,
        body_snippet: Option<Box<str>>,
    },

    #[error("telegram api returned `ok=true` without `result` for `{method}`")]
    MissingResult {
        method: String,
        status: Option<u16>,
        request_id: Option<Box<str>>,
        body_snippet: Option<Box<str>>,
    },
}

impl Error {
    /// High-level classification of this error.
    pub fn classification(&self) -> ErrorClass {
        match self {
            Self::InvalidBaseUrl { .. }
            | Self::InvalidBaseUrlScheme { .. }
            | Self::InvalidMethodName { .. }
            | Self::InvalidHeaderName { .. }
            | Self::InvalidHeaderValue { .. }
            | Self::Configuration { .. } => ErrorClass::Configuration,
            Self::InvalidBotToken
            | Self::MissingBotToken
            | Self::InvalidRequest { .. }
            | Self::SerializeRequest { .. }
            | Self::ReadLocalFile { .. } => ErrorClass::Validation,
            Self::DeserializeResponse { .. } => ErrorClass::Decode,
            Self::MissingResult { .. } => ErrorClass::Protocol,
            Self::Transport {
                status,
                retry_after,
                ..
            } => {
                if *status == Some(429) || retry_after.is_some() {
                    return ErrorClass::RateLimited;
                }
                if matches!(*status, Some(401 | 403)) {
                    return ErrorClass::Authentication;
                }
                ErrorClass::Transport
            }
            Self::Api {
                error_code,
                parameters,
                ..
            } => {
                if error_code.is_some_and(|code| code == 401 || code == 403) {
                    return ErrorClass::Authentication;
                }
                if error_code.is_some_and(|code| code == 429)
                    || parameters
                        .as_deref()
                        .and_then(|parameters| parameters.retry_after)
                        .is_some()
                {
                    return ErrorClass::RateLimited;
                }
                ErrorClass::Api
            }
        }
    }

    /// HTTP status code associated with this error, if available.
    pub fn status(&self) -> Option<StatusCode> {
        let code = match self {
            Self::DeserializeResponse { status, .. }
            | Self::Transport { status, .. }
            | Self::Api { status, .. }
            | Self::MissingResult { status, .. } => *status,
            _ => None,
        }?;

        StatusCode::from_u16(code).ok()
    }

    /// Provider request id when available.
    pub fn request_id(&self) -> Option<&str> {
        match self {
            Self::DeserializeResponse { request_id, .. }
            | Self::Transport { request_id, .. }
            | Self::Api { request_id, .. }
            | Self::MissingResult { request_id, .. } => request_id.as_deref(),
            _ => None,
        }
    }

    /// API error code returned by Telegram.
    pub fn error_code(&self) -> Option<i64> {
        match self {
            Self::Api { error_code, .. } => *error_code,
            _ => None,
        }
    }

    /// Retry-after duration inferred from either transport headers or Telegram parameters.
    pub fn retry_after(&self) -> Option<Duration> {
        match self {
            Self::Transport { retry_after, .. } => *retry_after,
            Self::Api { parameters, .. } => parameters
                .as_deref()
                .and_then(|parameters| parameters.retry_after)
                .map(Duration::from_secs),
            _ => None,
        }
    }

    /// Whether this error is generally retryable.
    pub fn is_retryable(&self) -> bool {
        if self.classification() == ErrorClass::RateLimited {
            return true;
        }

        if let Some(status) = self.status().map(|status| status.as_u16())
            && matches!(status, 408 | 409 | 425 | 429 | 500 | 502 | 503 | 504)
        {
            return true;
        }

        matches!(self, Self::Transport { .. })
    }

    /// Whether this error indicates API throttling.
    pub fn is_rate_limited(&self) -> bool {
        self.classification() == ErrorClass::RateLimited
    }

    /// Whether this error indicates invalid credentials or insufficient permissions.
    pub fn is_auth_error(&self) -> bool {
        if let Some(status) = self.status().map(|status| status.as_u16())
            && matches!(status, 401 | 403)
        {
            return true;
        }

        self.error_code()
            .is_some_and(|code| code == 401 || code == 403)
    }
}

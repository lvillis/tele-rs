use http::{HeaderMap, StatusCode};

use crate::bot::{DispatchOutcome, WebhookRunner, invalid_request};
use crate::types::update::Update;
use crate::{Error, Result};

/// Telegram webhook secret header name.
pub const TELEGRAM_SECRET_HEADER: &str = "x-telegram-bot-api-secret-token";

const INVALID_SECRET_REASON: &str = "invalid webhook secret token";
const DUPLICATE_SECRET_HEADER_REASON: &str = "duplicate webhook secret token header";
const INVALID_SECRET_HEADER_REASON: &str = "invalid webhook secret token header";

/// Extracts the Telegram webhook secret token from request headers.
///
/// Telegram sends at most one `x-telegram-bot-api-secret-token` header. Duplicate
/// values or non-UTF-8 header bytes are rejected before the payload reaches the
/// webhook dispatcher.
pub fn telegram_secret_token(headers: &HeaderMap) -> Result<Option<&str>> {
    let mut values = headers.get_all(TELEGRAM_SECRET_HEADER).iter();
    let Some(value) = values.next() else {
        return Ok(None);
    };

    if values.next().is_some() {
        return Err(Error::InvalidRequest {
            reason: DUPLICATE_SECRET_HEADER_REASON.to_owned(),
        });
    }

    let token = value.to_str().map_err(|_| Error::InvalidRequest {
        reason: INVALID_SECRET_HEADER_REASON.to_owned(),
    })?;

    Ok(Some(token))
}

/// Validates secret token, parses update payload and dispatches it through `WebhookRunner`.
pub async fn dispatch_webhook(
    runner: &WebhookRunner,
    headers: &HeaderMap,
    payload: &[u8],
) -> Result<DispatchOutcome> {
    let update =
        parse_webhook_update(runner, headers, payload).map_err(WebhookHttpError::into_error)?;
    runner.dispatch_update_outcome(update).await
}

/// Dispatches webhook payload and converts the result into an HTTP status for Telegram.
pub async fn dispatch_webhook_status(
    runner: &WebhookRunner,
    headers: &HeaderMap,
    payload: &[u8],
) -> StatusCode {
    let update = match parse_webhook_update(runner, headers, payload) {
        Ok(update) => update,
        Err(error) => return error.status(),
    };

    match runner.dispatch_update_outcome(update).await {
        Ok(_outcome) => StatusCode::OK,
        Err(_error) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

fn parse_webhook_update(
    runner: &WebhookRunner,
    headers: &HeaderMap,
    payload: &[u8],
) -> std::result::Result<Update, WebhookHttpError> {
    let incoming_secret = telegram_secret_token(headers).map_err(WebhookHttpError::BadRequest)?;

    if !runner.verify_secret_token(incoming_secret) {
        return Err(WebhookHttpError::Unauthorized(invalid_request(
            INVALID_SECRET_REASON,
        )));
    }

    serde_json::from_slice(payload).map_err(|source| {
        WebhookHttpError::BadRequest(invalid_request(format!(
            "failed to deserialize webhook update payload: {source}"
        )))
    })
}

enum WebhookHttpError {
    BadRequest(Error),
    Unauthorized(Error),
}

impl WebhookHttpError {
    fn status(&self) -> StatusCode {
        match self {
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::Unauthorized(_) => StatusCode::UNAUTHORIZED,
        }
    }

    fn into_error(self) -> Error {
        match self {
            Self::BadRequest(error) | Self::Unauthorized(error) => error,
        }
    }
}

#[cfg(test)]
mod tests {
    use http::HeaderValue;

    use super::*;

    #[test]
    fn telegram_secret_token_accepts_zero_or_one_valid_header() -> Result<()> {
        let headers = HeaderMap::new();
        assert_eq!(telegram_secret_token(&headers)?, None);

        let mut headers = HeaderMap::new();
        headers.insert(TELEGRAM_SECRET_HEADER, HeaderValue::from_static("secret"));
        assert_eq!(telegram_secret_token(&headers)?, Some("secret"));

        Ok(())
    }

    #[test]
    fn telegram_secret_token_rejects_ambiguous_or_malformed_headers()
    -> std::result::Result<(), Box<dyn std::error::Error>> {
        let mut duplicate_headers = HeaderMap::new();
        duplicate_headers.append(TELEGRAM_SECRET_HEADER, HeaderValue::from_static("secret"));
        duplicate_headers.append(TELEGRAM_SECRET_HEADER, HeaderValue::from_static("secret"));
        assert!(telegram_secret_token(&duplicate_headers).is_err());

        let mut malformed_headers = HeaderMap::new();
        let malformed = HeaderValue::from_bytes(&[0xff])?;
        malformed_headers.insert(TELEGRAM_SECRET_HEADER, malformed);
        assert!(telegram_secret_token(&malformed_headers).is_err());

        Ok(())
    }
}

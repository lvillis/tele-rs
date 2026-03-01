use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};

use crate::bot::{DispatchOutcome, WebhookRunner};
use crate::{Error, Result};

/// Telegram webhook secret header name.
pub const TELEGRAM_SECRET_HEADER: &str = "x-telegram-bot-api-secret-token";

const INVALID_SECRET_REASON: &str = "invalid webhook secret token";
const INVALID_JSON_REASON_PREFIX: &str = "failed to deserialize webhook update payload";

/// Extracts the Telegram webhook secret token from request headers.
pub fn telegram_secret_token(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(TELEGRAM_SECRET_HEADER)
        .and_then(|value| value.to_str().ok())
}

/// Validates secret token, parses update payload and dispatches it through `WebhookRunner`.
pub async fn dispatch_webhook(
    runner: &WebhookRunner,
    headers: &HeaderMap,
    payload: &[u8],
) -> Result<DispatchOutcome> {
    let incoming_secret = telegram_secret_token(headers);
    let update = runner
        .parse_update_json(payload, incoming_secret)
        .map_err(normalize_parse_error)?;

    runner.dispatch_update_outcome(update).await
}

/// Dispatches webhook payload and converts the result into an HTTP status for Telegram.
pub async fn dispatch_webhook_status(
    runner: &WebhookRunner,
    headers: &HeaderMap,
    payload: &[u8],
) -> StatusCode {
    match dispatch_webhook(runner, headers, payload).await {
        Ok(_outcome) => StatusCode::OK,
        Err(error) => status_from_error(&error),
    }
}

/// Ready-to-use axum handler helper.
///
/// Route state must be `Arc<WebhookRunner>`.
pub async fn webhook_handler(
    State(runner): State<Arc<WebhookRunner>>,
    headers: HeaderMap,
    body: Bytes,
) -> StatusCode {
    dispatch_webhook_status(runner.as_ref(), &headers, body.as_ref()).await
}

fn status_from_error(error: &Error) -> StatusCode {
    match error {
        Error::InvalidRequest { reason } if reason == INVALID_SECRET_REASON => {
            StatusCode::UNAUTHORIZED
        }
        Error::InvalidRequest { reason } if reason.starts_with(INVALID_JSON_REASON_PREFIX) => {
            StatusCode::BAD_REQUEST
        }
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

fn normalize_parse_error(error: Error) -> Error {
    match error {
        Error::InvalidRequest { reason } if reason == "invalid webhook secret token" => {
            Error::InvalidRequest {
                reason: INVALID_SECRET_REASON.to_owned(),
            }
        }
        Error::InvalidRequest { reason }
            if reason.starts_with("failed to deserialize webhook update payload:") =>
        {
            Error::InvalidRequest {
                reason: reason.replacen(
                    "failed to deserialize webhook update payload:",
                    INVALID_JSON_REASON_PREFIX,
                    1,
                ),
            }
        }
        other => other,
    }
}

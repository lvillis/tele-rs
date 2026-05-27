use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::State;
use http::{HeaderMap, StatusCode};

use crate::bot::WebhookRunner;
pub use crate::bot::{
    TELEGRAM_SECRET_HEADER, dispatch_webhook, dispatch_webhook_status, telegram_secret_token,
};

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

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// `setWebhook` request.
#[derive(Clone, Debug, Serialize)]
pub struct SetWebhookRequest {
    pub url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ip_address: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_connections: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowed_updates: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub drop_pending_updates: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub secret_token: Option<String>,
}

impl SetWebhookRequest {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            ip_address: None,
            max_connections: None,
            allowed_updates: None,
            drop_pending_updates: None,
            secret_token: None,
        }
    }
}

/// `deleteWebhook` request.
#[derive(Clone, Debug, Default, Serialize)]
pub struct DeleteWebhookRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub drop_pending_updates: Option<bool>,
}

/// Telegram webhook info response.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct WebhookInfo {
    pub url: String,
    pub has_custom_certificate: bool,
    pub pending_update_count: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ip_address: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_error_date: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_error_message: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_connections: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowed_updates: Option<Vec<String>>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::update::{AllowedUpdate, UpdateKind, validate_allowed_updates};
use crate::types::validation::https_url as validate_https_url;
use crate::{Error, Result};

const MAX_WEBHOOK_CONNECTIONS: u8 = 100;

/// Validated Telegram webhook secret token.
#[derive(Clone, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub struct WebhookSecretToken(String);

impl WebhookSecretToken {
    /// Creates a webhook secret token accepted by Telegram Bot API.
    ///
    /// Telegram limits this token to 1-256 ASCII characters from
    /// `A-Z`, `a-z`, `0-9`, `_` and `-`.
    pub fn new(token: impl Into<String>) -> Result<Self> {
        let token = token.into();
        validate_secret_token(&token)?;
        Ok(Self(token))
    }

    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for WebhookSecretToken {
    fn as_ref(&self) -> &str {
        self.expose()
    }
}

impl fmt::Debug for WebhookSecretToken {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WebhookSecretToken")
            .field("value", &"<redacted>")
            .finish()
    }
}

/// `setWebhook` request.
#[derive(Clone, Debug, Serialize)]
pub struct SetWebhookRequest {
    pub url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ip_address: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_connections: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowed_updates: Option<Vec<AllowedUpdate>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub drop_pending_updates: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub secret_token: Option<WebhookSecretToken>,
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

    pub fn secret_token(mut self, token: impl Into<String>) -> Result<Self> {
        self.secret_token = Some(WebhookSecretToken::new(token)?);
        Ok(self)
    }

    pub fn set_secret_token(&mut self, token: impl Into<String>) -> Result<&mut Self> {
        self.secret_token = Some(WebhookSecretToken::new(token)?);
        Ok(self)
    }

    pub fn allowed_updates(
        mut self,
        allowed_updates: impl IntoIterator<Item = AllowedUpdate>,
    ) -> Self {
        self.set_allowed_updates(allowed_updates);
        self
    }

    pub fn allowed_update_kinds(
        mut self,
        kinds: impl IntoIterator<Item = UpdateKind>,
    ) -> Result<Self> {
        self.set_allowed_update_kinds(kinds)?;
        Ok(self)
    }

    pub fn set_allowed_updates(
        &mut self,
        allowed_updates: impl IntoIterator<Item = AllowedUpdate>,
    ) -> &mut Self {
        self.allowed_updates = Some(allowed_updates.into_iter().collect());
        self
    }

    pub fn set_allowed_update_kinds(
        &mut self,
        kinds: impl IntoIterator<Item = UpdateKind>,
    ) -> Result<&mut Self> {
        self.allowed_updates = Some(AllowedUpdate::from_kinds(kinds)?);
        Ok(self)
    }

    pub fn clear_allowed_updates(&mut self) -> &mut Self {
        self.allowed_updates = None;
        self
    }

    pub fn validate(&self) -> Result<()> {
        validate_webhook_url(&self.url)?;
        if let Some(ip_address) = self.ip_address.as_deref() {
            validate_webhook_ip_address(ip_address)?;
        }
        if self
            .max_connections
            .is_some_and(|value| value == 0 || value > MAX_WEBHOOK_CONNECTIONS)
        {
            return Err(invalid_request(format!(
                "webhook max_connections must be 1-{MAX_WEBHOOK_CONNECTIONS}"
            )));
        }
        if let Some(allowed_updates) = self.allowed_updates.as_ref() {
            validate_allowed_updates(allowed_updates)?;
        }

        Ok(())
    }
}

/// `deleteWebhook` request.
#[derive(Clone, Debug, Default, Serialize)]
pub struct DeleteWebhookRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub drop_pending_updates: Option<bool>,
}

/// Telegram webhook info response.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct WebhookInfo {
    pub url: String,
    pub has_custom_certificate: bool,
    pub pending_update_count: u64,
    #[serde(default)]
    pub ip_address: Option<String>,
    #[serde(default)]
    pub last_error_date: Option<i64>,
    #[serde(default)]
    pub last_error_message: Option<String>,
    #[serde(default)]
    pub max_connections: Option<u8>,
    #[serde(default)]
    pub allowed_updates: Option<Vec<AllowedUpdate>>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

fn validate_secret_token(token: &str) -> Result<()> {
    if token.is_empty()
        || token.len() > 256
        || !token
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
    {
        return Err(Error::InvalidRequest {
            reason: "webhook secret token must be 1-256 ASCII letters, digits, `_` or `-`"
                .to_owned(),
        });
    }

    Ok(())
}

fn invalid_request(reason: impl Into<String>) -> Error {
    Error::InvalidRequest {
        reason: reason.into(),
    }
}

fn validate_webhook_url(url: &str) -> Result<()> {
    if url.trim().is_empty() {
        return Err(invalid_request("webhook url cannot be empty"));
    }

    validate_https_url("webhook url", url)
}

fn validate_webhook_ip_address(ip_address: &str) -> Result<()> {
    ip_address.parse::<std::net::IpAddr>().map_err(|source| {
        invalid_request(format!(
            "invalid webhook ip_address `{ip_address}`: {source}"
        ))
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_webhook_secret_token() {
        assert!(WebhookSecretToken::new("secret_TOKEN-123").is_ok());

        let too_long = "a".repeat(257);
        for token in ["", "secret token", "secret/token", "secret\n", &too_long] {
            assert!(matches!(
                WebhookSecretToken::new(token),
                Err(Error::InvalidRequest { .. })
            ));
        }
    }

    #[test]
    fn serializes_secret_token_as_plain_string() -> std::result::Result<(), serde_json::Error> {
        let request = SetWebhookRequest::new("https://example.com/hook")
            .secret_token("secret_TOKEN-123")
            .map_err(|error| serde_json::Error::io(std::io::Error::other(error)))?;

        let json = serde_json::to_value(request)?;
        assert_eq!(json["secret_token"], "secret_TOKEN-123");
        Ok(())
    }

    #[test]
    fn validates_set_webhook_request() -> Result<()> {
        let mut valid = SetWebhookRequest::new("https://example.com/hook");
        valid.max_connections = Some(100);
        valid.set_allowed_update_kinds([UpdateKind::Message, UpdateKind::CallbackQuery])?;
        assert!(valid.validate().is_ok());

        let invalid_url = SetWebhookRequest::new("http://example.com/hook");
        assert!(matches!(
            invalid_url.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let mut invalid_connections = SetWebhookRequest::new("https://example.com/hook");
        invalid_connections.max_connections = Some(0);
        assert!(matches!(
            invalid_connections.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let mut invalid_ip = SetWebhookRequest::new("https://example.com/hook");
        invalid_ip.ip_address = Some("not-an-ip".to_owned());
        assert!(matches!(
            invalid_ip.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        Ok(())
    }

    #[test]
    fn webhook_info_preserves_future_fields() -> std::result::Result<(), Box<dyn std::error::Error>>
    {
        let info: WebhookInfo = serde_json::from_value(serde_json::json!({
            "url": "https://example.com/hook",
            "has_custom_certificate": true,
            "pending_update_count": 7,
            "allowed_updates": ["message"],
            "future_field": "kept"
        }))?;

        assert_eq!(info.url, "https://example.com/hook");
        assert!(info.has_custom_certificate);
        assert_eq!(info.pending_update_count, 7);
        assert_eq!(info.allowed_updates.as_ref().map(Vec::len), Some(1));
        assert_eq!(
            info.extra.get("future_field"),
            Some(&serde_json::json!("kept"))
        );
        Ok(())
    }
}

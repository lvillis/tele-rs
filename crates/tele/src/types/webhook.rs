use std::collections::BTreeMap;
use std::fmt;

use serde::ser::SerializeMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::extra::{
    field_len as extra_field_len, serialize_fields as serialize_extra_fields,
};
use crate::types::update::{AllowedUpdate, UpdateKind, validate_allowed_updates};
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ip_address: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_error_date: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_error_message: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_connections: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowed_updates: Option<Vec<AllowedUpdate>>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl Serialize for WebhookInfo {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = [
            "url",
            "has_custom_certificate",
            "pending_update_count",
            "ip_address",
            "last_error_date",
            "last_error_message",
            "max_connections",
            "allowed_updates",
        ];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let optional_len = usize::from(self.ip_address.is_some())
            + usize::from(self.last_error_date.is_some())
            + usize::from(self.last_error_message.is_some())
            + usize::from(self.max_connections.is_some())
            + usize::from(self.allowed_updates.is_some());
        let mut object = serializer.serialize_map(Some(extra_len + optional_len + 3))?;
        object.serialize_entry("url", &self.url)?;
        object.serialize_entry("has_custom_certificate", &self.has_custom_certificate)?;
        object.serialize_entry("pending_update_count", &self.pending_update_count)?;
        if let Some(ip_address) = self.ip_address.as_ref() {
            object.serialize_entry("ip_address", ip_address)?;
        }
        if let Some(last_error_date) = self.last_error_date {
            object.serialize_entry("last_error_date", &last_error_date)?;
        }
        if let Some(last_error_message) = self.last_error_message.as_ref() {
            object.serialize_entry("last_error_message", last_error_message)?;
        }
        if let Some(max_connections) = self.max_connections {
            object.serialize_entry("max_connections", &max_connections)?;
        }
        if let Some(allowed_updates) = self.allowed_updates.as_ref() {
            object.serialize_entry("allowed_updates", allowed_updates)?;
        }
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
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

    let parsed = url::Url::parse(url)
        .map_err(|source| invalid_request(format!("invalid webhook url `{url}`: {source}")))?;
    if parsed.scheme() != "https" {
        return Err(invalid_request(format!(
            "webhook url must use https scheme, got `{}`",
            parsed.scheme()
        )));
    }
    if parsed.host_str().is_none() {
        return Err(invalid_request("webhook url must include host"));
    }

    Ok(())
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
    fn webhook_info_extra_cannot_override_reserved_fields()
    -> std::result::Result<(), Box<dyn std::error::Error>> {
        let mut info = WebhookInfo {
            url: "https://example.com/hook".to_owned(),
            has_custom_certificate: true,
            pending_update_count: 7,
            ip_address: Some("127.0.0.1".to_owned()),
            last_error_date: Some(1_700_000_000),
            last_error_message: Some("temporary failure".to_owned()),
            max_connections: Some(40),
            allowed_updates: Some(AllowedUpdate::from_kinds([UpdateKind::Message])?),
            extra: BTreeMap::new(),
        };
        info.extra.insert(
            "url".to_owned(),
            serde_json::json!("https://example.com/overridden"),
        );
        info.extra.insert(
            "has_custom_certificate".to_owned(),
            serde_json::json!(false),
        );
        info.extra
            .insert("pending_update_count".to_owned(), serde_json::json!(1));
        info.extra.insert(
            "allowed_updates".to_owned(),
            serde_json::json!(["callback_query"]),
        );
        info.extra
            .insert("future_field".to_owned(), serde_json::json!("kept"));

        let value = serde_json::to_value(&info)?;
        assert_eq!(value["url"], "https://example.com/hook");
        assert_eq!(value["has_custom_certificate"], true);
        assert_eq!(value["pending_update_count"], 7);
        assert_eq!(value["ip_address"], "127.0.0.1");
        assert_eq!(value["last_error_date"], 1_700_000_000);
        assert_eq!(value["last_error_message"], "temporary failure");
        assert_eq!(value["max_connections"], 40);
        assert_eq!(value["allowed_updates"], serde_json::json!(["message"]));
        assert_eq!(value["future_field"], "kept");

        Ok(())
    }
}

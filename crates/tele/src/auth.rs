use std::collections::BTreeMap;
use std::fmt;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use hmac::{Hmac, Mac};
use sha2::Sha256;
use url::form_urlencoded;

use crate::Error;

type HmacSha256 = Hmac<Sha256>;
const WEB_APP_DATA_KEY: &[u8] = b"WebAppData";

/// Verified Mini App `initData` payload.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct VerifiedWebAppInitData {
    auth_date: Option<u64>,
    fields: BTreeMap<String, String>,
}

impl VerifiedWebAppInitData {
    pub fn auth_date(&self) -> Option<u64> {
        self.auth_date
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.fields.get(key).map(String::as_str)
    }

    pub fn fields(&self) -> &BTreeMap<String, String> {
        &self.fields
    }

    pub fn into_fields(self) -> BTreeMap<String, String> {
        self.fields
    }
}

/// Parses Mini App `initData` query-string into decoded key-value pairs.
pub fn parse_web_app_init_data(init_data: &str) -> Result<BTreeMap<String, String>, Error> {
    if init_data.trim().is_empty() {
        return Err(Error::InvalidRequest {
            reason: "initData must not be empty".to_owned(),
        });
    }

    let mut fields = BTreeMap::new();
    for (key, value) in form_urlencoded::parse(init_data.as_bytes()) {
        let key = key.into_owned();
        let value = value.into_owned();
        if fields.insert(key.clone(), value).is_some() {
            return Err(Error::InvalidRequest {
                reason: format!("initData contains duplicate key `{key}`"),
            });
        }
    }

    if fields.is_empty() {
        return Err(Error::InvalidRequest {
            reason: "initData does not contain any fields".to_owned(),
        });
    }

    Ok(fields)
}

/// Verifies Mini App `initData` signature and optional max age.
///
/// This should run on the backend before trusting Mini App payloads.
pub fn verify_web_app_init_data(
    bot_token: &str,
    init_data: &str,
    max_age: Option<Duration>,
) -> Result<VerifiedWebAppInitData, Error> {
    if bot_token.trim().is_empty() {
        return Err(Error::InvalidBotToken);
    }

    let mut fields = parse_web_app_init_data(init_data)?;
    let hash_hex = fields.remove("hash").ok_or_else(|| Error::InvalidRequest {
        reason: "initData is missing `hash`".to_owned(),
    })?;

    let data_check_string = fields
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>()
        .join("\n");

    let secret_key = web_app_secret_key(bot_token)?;
    let mut mac = HmacSha256::new_from_slice(secret_key.as_slice()).map_err(|error| {
        Error::InvalidRequest {
            reason: format!("failed to initialize initData verifier: {error}"),
        }
    })?;
    mac.update(data_check_string.as_bytes());
    let expected_hash = mac.finalize().into_bytes();

    let actual_hash = decode_hex(hash_hex.as_str())?;
    if !constant_time_eq(expected_hash.as_ref(), actual_hash.as_slice()) {
        return Err(Error::InvalidRequest {
            reason: "invalid initData signature".to_owned(),
        });
    }

    let auth_date = match fields.get("auth_date") {
        Some(value) => Some(
            value
                .parse::<u64>()
                .map_err(|error| Error::InvalidRequest {
                    reason: format!("invalid initData `auth_date`: {error}"),
                })?,
        ),
        None => None,
    };

    if let Some(max_age) = max_age {
        let auth_date = auth_date.ok_or_else(|| Error::InvalidRequest {
            reason: "initData is missing `auth_date` required for max_age validation".to_owned(),
        })?;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| Error::InvalidRequest {
                reason: format!("system clock error while validating initData age: {error}"),
            })?
            .as_secs();
        let age_secs = now.saturating_sub(auth_date);
        if age_secs > max_age.as_secs() {
            return Err(Error::InvalidRequest {
                reason: format!(
                    "initData has expired: age={}s exceeds max_age={}s",
                    age_secs,
                    max_age.as_secs()
                ),
            });
        }
    }

    Ok(VerifiedWebAppInitData { auth_date, fields })
}

fn web_app_secret_key(bot_token: &str) -> Result<[u8; 32], Error> {
    let mut mac =
        HmacSha256::new_from_slice(WEB_APP_DATA_KEY).map_err(|error| Error::InvalidRequest {
            reason: format!("failed to derive Mini App secret key: {error}"),
        })?;
    mac.update(bot_token.as_bytes());
    let secret = mac.finalize().into_bytes();
    let mut output = [0_u8; 32];
    output.copy_from_slice(secret.as_ref());
    Ok(output)
}

fn decode_hex(input: &str) -> Result<Vec<u8>, Error> {
    if !input.len().is_multiple_of(2) {
        return Err(Error::InvalidRequest {
            reason: "initData hash has invalid hex length".to_owned(),
        });
    }

    let mut output = Vec::with_capacity(input.len() / 2);
    let bytes = input.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        let high = decode_hex_nibble(bytes[index]).ok_or_else(|| Error::InvalidRequest {
            reason: "initData hash contains non-hex characters".to_owned(),
        })?;
        let low = decode_hex_nibble(bytes[index + 1]).ok_or_else(|| Error::InvalidRequest {
            reason: "initData hash contains non-hex characters".to_owned(),
        })?;
        output.push((high << 4) | low);
        index += 2;
    }

    Ok(output)
}

fn decode_hex_nibble(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }

    let mut diff = 0_u8;
    for (lhs, rhs) in left.iter().zip(right.iter()) {
        diff |= lhs ^ rhs;
    }

    diff == 0
}

/// Authentication strategy for Telegram Bot API requests.
#[derive(Clone, Eq, PartialEq)]
#[non_exhaustive]
pub enum Auth {
    /// No authentication token.
    None,
    /// Bot token authentication.
    BotToken(BotToken),
}

impl Auth {
    /// Build an auth object with no credentials.
    pub const fn none() -> Self {
        Self::None
    }

    /// Build an auth object from a Telegram bot token.
    pub fn bot_token(token: impl Into<String>) -> Result<Self, Error> {
        Ok(Self::BotToken(BotToken::new(token)?))
    }

    pub(crate) fn token(&self) -> Option<&str> {
        match self {
            Self::None => None,
            Self::BotToken(token) => Some(token.expose()),
        }
    }
}

impl fmt::Debug for Auth {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => formatter.debug_tuple("Auth::None").finish(),
            Self::BotToken(_) => formatter
                .debug_struct("Auth::BotToken")
                .field("token", &"<redacted>")
                .finish(),
        }
    }
}

/// Bot token wrapper with redacted debug output.
#[derive(Clone, Eq, PartialEq)]
pub struct BotToken(String);

impl BotToken {
    /// Create a new bot token.
    pub fn new(token: impl Into<String>) -> Result<Self, Error> {
        let token = token.into();
        if token.trim().is_empty() {
            return Err(Error::InvalidBotToken);
        }

        Ok(Self(token))
    }

    pub(crate) fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for BotToken {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BotToken")
            .field("value", &"<redacted>")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error as StdError;

    use super::*;

    fn sign_init_data(
        bot_token: &str,
        fields: &[(&str, &str)],
    ) -> std::result::Result<String, Box<dyn StdError>> {
        let mut ordered = BTreeMap::new();
        for (key, value) in fields {
            ordered.insert((*key).to_owned(), (*value).to_owned());
        }

        let data_check_string = ordered
            .iter()
            .map(|(key, value)| format!("{key}={value}"))
            .collect::<Vec<_>>()
            .join("\n");

        let secret_key = web_app_secret_key(bot_token)?;
        let mut mac = HmacSha256::new_from_slice(secret_key.as_slice())?;
        mac.update(data_check_string.as_bytes());
        let hash = mac.finalize().into_bytes();
        let hash_hex = encode_hex(hash.as_ref());

        let mut serializer = form_urlencoded::Serializer::new(String::new());
        for (key, value) in ordered {
            serializer.append_pair(&key, &value);
        }
        serializer.append_pair("hash", &hash_hex);
        Ok(serializer.finish())
    }

    fn encode_hex(bytes: &[u8]) -> String {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut output = String::with_capacity(bytes.len() * 2);
        for byte in bytes {
            output.push(HEX[(byte >> 4) as usize] as char);
            output.push(HEX[(byte & 0x0f) as usize] as char);
        }
        output
    }

    #[test]
    fn verifies_valid_init_data() -> std::result::Result<(), Box<dyn StdError>> {
        let bot_token = "123456:bot-token";
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let auth_date = now.to_string();
        let init_data = sign_init_data(
            bot_token,
            &[
                ("auth_date", auth_date.as_str()),
                ("query_id", "q-1"),
                ("user", r#"{"id":42,"first_name":"Tele"}"#),
            ],
        )?;

        let verified =
            verify_web_app_init_data(bot_token, init_data.as_str(), Some(Duration::from_secs(60)))?;
        assert_eq!(verified.get("query_id"), Some("q-1"));
        assert_eq!(verified.auth_date(), Some(now));
        Ok(())
    }

    #[test]
    fn verifies_known_hash_vector() -> std::result::Result<(), Box<dyn StdError>> {
        let bot_token = "123456:bot-token";
        let init_data = "auth_date=1700000000&query_id=q-1&user=%7B%22id%22%3A42%2C%22first_name%22%3A%22Tele%22%7D&hash=e6e77ddca82b669a27e3d2bacd6535954ced7219f791f47ff7f2e257000f6b1c";
        let verified = verify_web_app_init_data(bot_token, init_data, None)?;
        assert_eq!(verified.auth_date(), Some(1_700_000_000));
        assert_eq!(verified.get("query_id"), Some("q-1"));
        Ok(())
    }

    #[test]
    fn rejects_invalid_signature() -> std::result::Result<(), Box<dyn StdError>> {
        let bot_token = "123456:bot-token";
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let auth_date = now.to_string();
        let mut init_data = sign_init_data(
            bot_token,
            &[("auth_date", auth_date.as_str()), ("query_id", "q-1")],
        )?;
        if let Some(hash_index) = init_data.find("hash=") {
            let value_index = hash_index + 5;
            if value_index < init_data.len() {
                let replacement = match init_data.as_bytes()[value_index] {
                    b'0' => "1",
                    _ => "0",
                };
                init_data.replace_range(value_index..=value_index, replacement);
            }
        }

        let error = match verify_web_app_init_data(bot_token, init_data.as_str(), None) {
            Ok(_) => return Err("verification should fail".into()),
            Err(error) => error,
        };
        assert!(matches!(error, Error::InvalidRequest { .. }));
        assert!(error.to_string().contains("invalid initData signature"));
        Ok(())
    }

    #[test]
    fn rejects_stale_init_data() -> std::result::Result<(), Box<dyn StdError>> {
        let bot_token = "123456:bot-token";
        let stale_auth_date = "1";
        let init_data = sign_init_data(
            bot_token,
            &[("auth_date", stale_auth_date), ("query_id", "q-1")],
        )?;

        let error = match verify_web_app_init_data(
            bot_token,
            init_data.as_str(),
            Some(Duration::from_secs(60)),
        ) {
            Ok(_) => return Err("stale payload should fail".into()),
            Err(error) => error,
        };
        assert!(matches!(error, Error::InvalidRequest { .. }));
        assert!(error.to_string().contains("initData has expired"));
        Ok(())
    }

    #[test]
    fn rejects_duplicate_keys_in_init_data() -> std::result::Result<(), Box<dyn StdError>> {
        let error = match parse_web_app_init_data("auth_date=1&auth_date=2&hash=deadbeef") {
            Ok(_) => return Err("duplicate keys must be rejected".into()),
            Err(error) => error,
        };
        assert!(matches!(error, Error::InvalidRequest { .. }));
        assert!(error.to_string().contains("duplicate key `auth_date`"));
        Ok(())
    }
}

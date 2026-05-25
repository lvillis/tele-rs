use std::collections::BTreeMap;
use std::fmt;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use graviola::hashing::{HashOutput, Sha256, hmac::Hmac};
use url::form_urlencoded;

use crate::Error;

type HmacSha256 = Hmac<Sha256>;
type Sha256Digest = [u8; 32];
const WEB_APP_DATA_KEY: &[u8] = b"WebAppData";
const WEB_APP_INIT_DATA_FUTURE_SKEW: Duration = Duration::from_secs(60);

/// Verified Mini App `initData` payload.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct VerifiedWebAppInitData {
    auth_date: u64,
    fields: BTreeMap<String, String>,
}

impl VerifiedWebAppInitData {
    /// Unix timestamp from the verified `auth_date` field.
    pub fn auth_date(&self) -> u64 {
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
        validate_init_data_key(&key)?;
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

fn validate_init_data_key(key: &str) -> Result<(), Error> {
    if key.is_empty() {
        return Err(Error::InvalidRequest {
            reason: "initData contains an empty key".to_owned(),
        });
    }
    if !key
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    {
        return Err(Error::InvalidRequest {
            reason: format!("initData contains invalid key `{key}`"),
        });
    }

    Ok(())
}

/// Verifies Mini App `initData` signature, required `auth_date`, and optional max age.
///
/// This should run on the backend before trusting Mini App payloads.
pub fn verify_web_app_init_data(
    bot_token: &str,
    init_data: &str,
    max_age: Option<Duration>,
) -> Result<VerifiedWebAppInitData, Error> {
    validate_bot_token(bot_token)?;

    let mut fields = parse_web_app_init_data(init_data)?;
    let hash_hex = fields.remove("hash").ok_or_else(|| Error::InvalidRequest {
        reason: "initData is missing `hash`".to_owned(),
    })?;

    let data_check_string = fields
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>()
        .join("\n");

    let secret_key = web_app_secret_key(bot_token);
    let expected_hash = hmac_sha256(secret_key, data_check_string.as_bytes());

    let actual_hash = decode_hex(hash_hex.as_str())?;
    if actual_hash.len() != std::mem::size_of::<Sha256Digest>() {
        return Err(Error::InvalidRequest {
            reason: "initData hash must decode to 32 bytes".to_owned(),
        });
    }
    if !expected_hash.ct_equal(actual_hash.as_slice()) {
        return Err(Error::InvalidRequest {
            reason: "invalid initData signature".to_owned(),
        });
    }

    let auth_date = fields
        .get("auth_date")
        .ok_or_else(|| Error::InvalidRequest {
            reason: "initData is missing `auth_date`".to_owned(),
        })?
        .parse::<u64>()
        .map_err(|error| Error::InvalidRequest {
            reason: format!("invalid initData `auth_date`: {error}"),
        })?;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| Error::InvalidRequest {
            reason: format!("system clock error while validating initData age: {error}"),
        })?
        .as_secs();
    if auth_date > now {
        let skew_secs = auth_date - now;
        if skew_secs > WEB_APP_INIT_DATA_FUTURE_SKEW.as_secs() {
            return Err(Error::InvalidRequest {
                reason: format!(
                    "initData `auth_date` is in the future: skew={}s exceeds allowed_skew={}s",
                    skew_secs,
                    WEB_APP_INIT_DATA_FUTURE_SKEW.as_secs()
                ),
            });
        }
    }

    if let Some(max_age) = max_age {
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

fn web_app_secret_key(bot_token: &str) -> Sha256Digest {
    hmac_sha256_bytes(WEB_APP_DATA_KEY, bot_token.as_bytes())
}

fn hmac_sha256(key: impl AsRef<[u8]>, data: impl AsRef<[u8]>) -> HashOutput {
    let mut mac = HmacSha256::new(key);
    mac.update(data);
    mac.finish()
}

fn hmac_sha256_bytes(key: impl AsRef<[u8]>, data: impl AsRef<[u8]>) -> Sha256Digest {
    let mut output = [0_u8; 32];
    output.copy_from_slice(hmac_sha256(key, data).as_ref());
    output
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
        validate_bot_token(&token)?;

        Ok(Self(token))
    }

    pub(crate) fn expose(&self) -> &str {
        &self.0
    }
}

fn validate_bot_token(token: &str) -> Result<(), Error> {
    let Some((bot_id, secret)) = token.split_once(':') else {
        return Err(Error::InvalidBotToken);
    };

    if bot_id.is_empty()
        || secret.is_empty()
        || secret.contains(':')
        || !bot_id.bytes().all(|byte| byte.is_ascii_digit())
        || !secret
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
    {
        return Err(Error::InvalidBotToken);
    }

    Ok(())
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

    type TestResult = std::result::Result<(), Box<dyn StdError>>;

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

        let secret_key = web_app_secret_key(bot_token);
        let hash = hmac_sha256(secret_key, data_check_string.as_bytes());
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
    fn verifies_valid_init_data() -> TestResult {
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
        assert_eq!(verified.auth_date(), now);
        Ok(())
    }

    #[test]
    fn verifies_known_hash_vector() -> TestResult {
        let bot_token = "123456:bot-token";
        let init_data = "auth_date=1700000000&query_id=q-1&user=%7B%22id%22%3A42%2C%22first_name%22%3A%22Tele%22%7D&hash=e6e77ddca82b669a27e3d2bacd6535954ced7219f791f47ff7f2e257000f6b1c";
        let verified = verify_web_app_init_data(bot_token, init_data, None)?;
        assert_eq!(verified.auth_date(), 1_700_000_000);
        assert_eq!(verified.get("query_id"), Some("q-1"));
        Ok(())
    }

    #[test]
    fn rejects_missing_auth_date_even_without_max_age() -> TestResult {
        let bot_token = "123456:bot-token";
        let init_data = sign_init_data(bot_token, &[("query_id", "q-1")])?;

        let error = match verify_web_app_init_data(bot_token, init_data.as_str(), None) {
            Ok(_) => return Err("missing auth_date should fail".into()),
            Err(error) => error,
        };
        assert!(matches!(error, Error::InvalidRequest { .. }));
        assert!(error.to_string().contains("auth_date"));
        Ok(())
    }

    #[test]
    fn rejects_invalid_signature() -> TestResult {
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
    fn rejects_wrong_hash_length() -> TestResult {
        let bot_token = "123456:bot-token";
        let init_data = "auth_date=1700000000&query_id=q-1&hash=deadbeef";

        let error = match verify_web_app_init_data(bot_token, init_data, None) {
            Ok(_) => return Err("short hash should fail".into()),
            Err(error) => error,
        };
        assert!(matches!(error, Error::InvalidRequest { .. }));
        assert!(error.to_string().contains("32 bytes"));
        Ok(())
    }

    #[test]
    fn rejects_stale_init_data() -> TestResult {
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
    fn rejects_future_init_data_beyond_clock_skew() -> TestResult {
        let bot_token = "123456:bot-token";
        let future = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() + 3600;
        let future_auth_date = future.to_string();
        let init_data = sign_init_data(
            bot_token,
            &[
                ("auth_date", future_auth_date.as_str()),
                ("query_id", "q-1"),
            ],
        )?;

        let error = match verify_web_app_init_data(
            bot_token,
            init_data.as_str(),
            Some(Duration::from_secs(60)),
        ) {
            Ok(_) => return Err("future payload should fail".into()),
            Err(error) => error,
        };
        assert!(matches!(error, Error::InvalidRequest { .. }));
        assert!(error.to_string().contains("future"));
        Ok(())
    }

    #[test]
    fn validates_bot_token_shape_before_path_use() {
        assert!(BotToken::new("123456:ABC_def-123").is_ok());

        for token in [
            "",
            " ",
            "abc:def",
            "123",
            "123:",
            ":abc",
            "123:abc:def",
            "123:abc/def",
            "123:abc def",
            "123:abc\n",
        ] {
            assert!(matches!(BotToken::new(token), Err(Error::InvalidBotToken)));
        }
    }

    #[test]
    fn rejects_duplicate_keys_in_init_data() -> TestResult {
        let error = match parse_web_app_init_data("auth_date=1&auth_date=2&hash=deadbeef") {
            Ok(_) => return Err("duplicate keys must be rejected".into()),
            Err(error) => error,
        };
        assert!(matches!(error, Error::InvalidRequest { .. }));
        assert!(error.to_string().contains("duplicate key `auth_date`"));
        Ok(())
    }

    #[test]
    fn rejects_malformed_init_data_keys() -> TestResult {
        for init_data in [
            "=value&hash=deadbeef",
            "bad-key=value&hash=deadbeef",
            "bad%0Akey=value&hash=deadbeef",
            "bad%3Dkey=value&hash=deadbeef",
        ] {
            let error = match parse_web_app_init_data(init_data) {
                Ok(_) => return Err(format!("malformed key should fail: {init_data}").into()),
                Err(error) => error,
            };
            assert!(matches!(error, Error::InvalidRequest { .. }));
        }

        assert!(parse_web_app_init_data("auth_date=1&query_id=q-1&hash=deadbeef").is_ok());
        Ok(())
    }
}

use http::HeaderMap;
use url::Url;

use crate::Error;

const REDACTED_TOKEN: &str = "<redacted-token>";
#[cfg(any(test, feature = "redis-session", feature = "postgres-session"))]
const REDACTED_CREDENTIAL: &str = "redacted";

pub(crate) fn normalize_base_url(input: &str) -> Result<Url, Error> {
    let parsed = Url::parse(input).map_err(|source| Error::InvalidBaseUrl {
        input: input.to_owned(),
        source,
    })?;

    let scheme = parsed.scheme();
    if scheme != "https" && scheme != "http" {
        return Err(Error::InvalidBaseUrlScheme {
            scheme: scheme.to_owned(),
        });
    }

    Ok(parsed)
}

pub(crate) fn validate_method_name(method: &str) -> Result<(), Error> {
    if method.is_empty()
        || !method
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '_')
    {
        return Err(Error::InvalidMethodName {
            method: method.to_owned(),
        });
    }
    Ok(())
}

pub(crate) fn build_api_path(token: &str, method: &str) -> String {
    format!("/bot{token}/{method}")
}

pub(crate) fn redact_token(input: &str, token: &str) -> String {
    if token.is_empty() {
        return input.to_owned();
    }

    input.replace(token, REDACTED_TOKEN)
}

#[cfg(any(test, feature = "redis-session", feature = "postgres-session"))]
pub(crate) fn redact_url_credentials(input: &str) -> String {
    if let Ok(mut url) = Url::parse(input) {
        if (!url.username().is_empty() || url.password().is_some())
            && url.set_username(REDACTED_CREDENTIAL).is_err()
        {
            return redact_url_credentials_fallback(input);
        }
        if url.password().is_some() && url.set_password(Some(REDACTED_CREDENTIAL)).is_err() {
            return redact_url_credentials_fallback(input);
        }
        return url.to_string();
    }

    redact_url_credentials_fallback(input)
}

#[cfg(any(test, feature = "redis-session", feature = "postgres-session"))]
fn redact_url_credentials_fallback(input: &str) -> String {
    let Some(scheme_end) = input.find("://") else {
        return input.to_owned();
    };
    let authority_start = scheme_end + 3;
    let rest = &input[authority_start..];
    let authority_end = rest
        .find(['/', '?', '#'])
        .map(|index| authority_start + index)
        .unwrap_or(input.len());
    let authority = &input[authority_start..authority_end];
    let Some(at_index) = authority.rfind('@') else {
        return input.to_owned();
    };

    let credentials_end = authority_start + at_index + 1;
    format!(
        "{}{}{}",
        &input[..authority_start],
        "redacted@",
        &input[credentials_end..]
    )
}

pub(crate) fn body_snippet(body: &[u8], max_chars: usize) -> Option<String> {
    if body.is_empty() || max_chars == 0 {
        return None;
    }

    let text = String::from_utf8_lossy(body);
    if text.is_empty() {
        return None;
    }

    let mut output = String::new();
    for (index, character) in text.chars().enumerate() {
        if index >= max_chars {
            output.push_str("...(truncated)");
            break;
        }
        output.push(character);
    }

    if output.is_empty() {
        None
    } else {
        Some(output)
    }
}

pub(crate) fn request_id_from_headers(headers: &HeaderMap) -> Option<String> {
    ["x-request-id", "x-telegram-request-id", "x-amzn-requestid"]
        .into_iter()
        .find_map(|name| {
            headers
                .get(name)
                .and_then(|value| value.to_str().ok())
                .map(ToOwned::to_owned)
        })
}

#[cfg(test)]
mod tests {
    use super::{body_snippet, redact_token, redact_url_credentials};

    #[test]
    fn redacts_token_in_text() {
        let redacted = redact_token("/bot123:abc/getMe", "123:abc");
        assert_eq!(redacted, "/bot<redacted-token>/getMe");
    }

    #[test]
    fn redacts_url_credentials() {
        let redacted = redact_url_credentials("postgres://user:secret@example.com/db");
        assert_eq!(redacted, "postgres://redacted:redacted@example.com/db");

        let redacted = redact_url_credentials("redis://:secret@example.com/0");
        assert_eq!(redacted, "redis://redacted:redacted@example.com/0");
    }

    #[test]
    fn truncates_snippet() {
        let snippet = body_snippet(b"abcdefghijklmnopqrstuvwxyz", 5);
        assert_eq!(snippet.as_deref(), Some("abcde...(truncated)"));
    }
}

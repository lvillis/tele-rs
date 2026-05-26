use std::time::{SystemTime, UNIX_EPOCH};

use crate::Error;

use super::common::ParseMode;
use super::message::{MessageEntity, MessageEntityKind};
use super::telegram::{ReplyMarkup, ReplyParameters, SuggestedPostParameters};

const SUGGESTED_POST_MAX_SEND_DELAY_SECONDS: i64 = 2_678_400;
const SUGGESTED_POST_MIN_SEND_DELAY_SECONDS: i64 = 300;

pub(crate) fn required_text(label: &str, value: &str) -> Result<(), Error> {
    if value.trim().is_empty() {
        return Err(Error::InvalidRequest {
            reason: format!("{label} cannot be empty"),
        });
    }
    if value.chars().any(char::is_control) {
        return Err(Error::InvalidRequest {
            reason: format!("{label} must not contain control characters"),
        });
    }

    Ok(())
}

pub(crate) fn required_string(field: &str, value: &str) -> Result<(), Error> {
    if value.trim().is_empty() {
        return Err(Error::InvalidRequest {
            reason: format!("{field} cannot be empty"),
        });
    }

    Ok(())
}

pub(crate) fn control_free_string(field: &str, value: &str) -> Result<(), Error> {
    if value.chars().any(char::is_control) {
        return Err(Error::InvalidRequest {
            reason: format!("{field} must not contain control characters"),
        });
    }

    Ok(())
}

pub(crate) fn suggested_post_parameters_send_date(field: &str, value: i64) -> Result<(), Error> {
    validate_send_date_delay(
        value,
        SUGGESTED_POST_MIN_SEND_DELAY_SECONDS,
        SUGGESTED_POST_MAX_SEND_DELAY_SECONDS,
        format!(
            "{field} must be between {SUGGESTED_POST_MIN_SEND_DELAY_SECONDS} and {SUGGESTED_POST_MAX_SEND_DELAY_SECONDS} seconds in the future"
        ),
    )
}

pub(crate) fn suggested_post_approval_send_date(field: &str, value: i64) -> Result<(), Error> {
    validate_send_date_delay(
        value,
        1,
        SUGGESTED_POST_MAX_SEND_DELAY_SECONDS,
        format!(
            "{field} must be in the future and not more than {SUGGESTED_POST_MAX_SEND_DELAY_SECONDS} seconds in the future"
        ),
    )
}

fn validate_send_date_delay(
    value: i64,
    min_delay_seconds: i64,
    max_delay_seconds: i64,
    reason: String,
) -> Result<(), Error> {
    let now = current_unix_timestamp_seconds()?;
    let delay = value
        .checked_sub(now)
        .ok_or_else(|| Error::InvalidRequest {
            reason: reason.clone(),
        })?;

    if !(min_delay_seconds..=max_delay_seconds).contains(&delay) {
        return Err(Error::InvalidRequest { reason });
    }

    Ok(())
}

fn current_unix_timestamp_seconds() -> Result<i64, Error> {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| Error::InvalidRequest {
            reason: "current system time is before Unix epoch".to_owned(),
        })?
        .as_secs();

    i64::try_from(seconds).map_err(|_| Error::InvalidRequest {
        reason: "current Unix timestamp exceeds i64 range".to_owned(),
    })
}

pub(crate) fn text_length_range(
    field: &str,
    value: &str,
    min_chars: usize,
    max_chars: usize,
) -> Result<(), Error> {
    if min_chars > 0 && value.trim().is_empty() {
        return Err(Error::InvalidRequest {
            reason: format!("{field} cannot be empty"),
        });
    }

    let length = value.chars().count();
    if length < min_chars {
        return Err(Error::InvalidRequest {
            reason: format!("{field} must be at least {min_chars} characters"),
        });
    }
    if length > max_chars {
        return Err(Error::InvalidRequest {
            reason: format!("{field} must be at most {max_chars} characters"),
        });
    }

    control_free_string(field, value)
}

pub(crate) fn username_or_empty(field: &str, value: &str) -> Result<(), Error> {
    if value.is_empty() {
        return Ok(());
    }

    if !(5..=32).contains(&value.len()) {
        return Err(Error::InvalidRequest {
            reason: format!("{field} must be empty or 5-32 ASCII characters"),
        });
    }
    if !value
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    {
        return Err(Error::InvalidRequest {
            reason: format!("{field} may only contain ASCII letters, digits or `_`"),
        });
    }

    Ok(())
}

pub(crate) fn emoji_free_text(field: &str, value: &str, max_chars: usize) -> Result<(), Error> {
    text_length_range(field, value, 0, max_chars)?;
    if value.chars().any(is_emoji_scalar) {
        return Err(Error::InvalidRequest {
            reason: format!("{field} must not contain emoji"),
        });
    }

    Ok(())
}

fn is_emoji_scalar(character: char) -> bool {
    matches!(
        character as u32,
        0x00A9
            | 0x00AE
            | 0x203C
            | 0x2049
            | 0x2122
            | 0x2139
            | 0x2194..=0x21AA
            | 0x231A..=0x231B
            | 0x2328
            | 0x23CF
            | 0x23E9..=0x23F3
            | 0x23F8..=0x23FA
            | 0x24C2
            | 0x25AA..=0x25AB
            | 0x25B6
            | 0x25C0
            | 0x25FB..=0x25FE
            | 0x2600..=0x27BF
            | 0x2934..=0x2935
            | 0x2B05..=0x2B55
            | 0x3030
            | 0x303D
            | 0x3297
            | 0x3299
            | 0xFE0F
            | 0x200D
            | 0x20E3
            | 0x1F000..=0x1FAFF
    )
}

pub(crate) fn string_id(field: &str, value: &str) -> Result<(), Error> {
    required_string(field, value)?;
    control_free_string(field, value)
}

pub(crate) fn url(field: &str, value: &str) -> Result<(), Error> {
    let parsed = parse_url(field, value)?;
    match parsed.scheme() {
        "http" | "https" | "tg" => {}
        scheme => {
            return Err(Error::InvalidRequest {
                reason: format!("{field} must use http, https, or tg scheme, got `{scheme}`"),
            });
        }
    }
    validate_http_host(field, &parsed)?;

    Ok(())
}

pub(crate) fn http_url(field: &str, value: &str) -> Result<(), Error> {
    let parsed = parse_url(field, value)?;
    match parsed.scheme() {
        "http" | "https" => {}
        scheme => {
            return Err(Error::InvalidRequest {
                reason: format!("{field} must use http or https scheme, got `{scheme}`"),
            });
        }
    }
    validate_http_host(field, &parsed)?;

    Ok(())
}

pub(crate) fn https_url(field: &str, value: &str) -> Result<(), Error> {
    let parsed = parse_url(field, value)?;
    if parsed.scheme() != "https" {
        return Err(Error::InvalidRequest {
            reason: format!("{field} must use HTTPS"),
        });
    }
    validate_http_host(field, &parsed)?;

    Ok(())
}

fn parse_url(field: &str, value: &str) -> Result<url::Url, Error> {
    required_string(field, value)?;
    control_free_string(field, value)?;
    if value.chars().any(char::is_whitespace) {
        return Err(Error::InvalidRequest {
            reason: format!("{field} must not contain whitespace"),
        });
    }

    url::Url::parse(value).map_err(|source| Error::InvalidRequest {
        reason: format!("{field} must be a valid URL: {source}"),
    })
}

fn validate_http_host(field: &str, parsed: &url::Url) -> Result<(), Error> {
    if matches!(parsed.scheme(), "http" | "https") && parsed.host_str().is_none() {
        return Err(Error::InvalidRequest {
            reason: format!("{field} must include a host"),
        });
    }

    Ok(())
}

pub(crate) fn required_len(field: &str, len: usize) -> Result<(), Error> {
    if len == 0 {
        return Err(Error::InvalidRequest {
            reason: format!("{field} cannot be empty"),
        });
    }

    Ok(())
}

pub(crate) fn positive_i64(field: &str, value: i64) -> Result<(), Error> {
    if value <= 0 {
        return Err(Error::InvalidRequest {
            reason: format!("{field} must be greater than 0"),
        });
    }

    Ok(())
}

pub(crate) fn non_negative_i64(field: &str, value: i64) -> Result<(), Error> {
    if value < 0 {
        return Err(Error::InvalidRequest {
            reason: format!("{field} cannot be negative"),
        });
    }

    Ok(())
}

pub(crate) fn i64_range(field: &str, value: i64, min: i64, max: i64) -> Result<(), Error> {
    if !(min..=max).contains(&value) {
        return Err(Error::InvalidRequest {
            reason: format!("{field} must be between {min} and {max}"),
        });
    }

    Ok(())
}

pub(crate) fn request_non_empty(method: &str, field: &str, value: &str) -> Result<(), Error> {
    if value.trim().is_empty() {
        return Err(Error::InvalidRequest {
            reason: format!("{method} requires non-empty `{field}`"),
        });
    }

    Ok(())
}

pub(crate) fn request_string_id(method: &str, field: &str, value: &str) -> Result<(), Error> {
    request_non_empty(method, field, value)?;
    if value.chars().any(char::is_control) {
        return Err(Error::InvalidRequest {
            reason: format!("{method} requires `{field}` without control characters"),
        });
    }

    Ok(())
}

pub(crate) fn optional_request_string_id(
    method: &str,
    field: &str,
    value: Option<&str>,
) -> Result<(), Error> {
    if let Some(value) = value {
        request_string_id(method, field, value)?;
    }

    Ok(())
}

pub(crate) fn optional_positive_i64(field: &str, value: Option<i64>) -> Result<(), Error> {
    if let Some(value) = value
        && value <= 0
    {
        return Err(Error::InvalidRequest {
            reason: format!("{field} must be greater than 0"),
        });
    }

    Ok(())
}

pub(crate) fn optional_request_positive_i64(
    method: &str,
    field: &str,
    value: Option<i64>,
) -> Result<(), Error> {
    if let Some(value) = value
        && value <= 0
    {
        return Err(Error::InvalidRequest {
            reason: format!("{method} requires `{field}` to be greater than zero"),
        });
    }

    Ok(())
}

pub(crate) fn optional_request_positive_u32(
    method: &str,
    field: &str,
    value: Option<u32>,
) -> Result<(), Error> {
    if matches!(value, Some(0)) {
        return Err(Error::InvalidRequest {
            reason: format!("{method} requires `{field}` to be greater than zero"),
        });
    }

    Ok(())
}

pub(crate) fn reply_parameters(reply_parameters: Option<&ReplyParameters>) -> Result<(), Error> {
    if let Some(reply_parameters) = reply_parameters {
        reply_parameters.validate()?;
    }

    Ok(())
}

pub(crate) fn reply_markup(reply_markup: Option<&ReplyMarkup>) -> Result<(), Error> {
    if let Some(reply_markup) = reply_markup {
        reply_markup.validate()?;
    }

    Ok(())
}

pub(crate) fn suggested_post_parameters(
    suggested_post_parameters: Option<&SuggestedPostParameters>,
) -> Result<(), Error> {
    if let Some(suggested_post_parameters) = suggested_post_parameters {
        suggested_post_parameters.validate()?;
    }

    Ok(())
}

pub(crate) fn parse_mode_entities_conflict(
    field: &str,
    parse_mode: Option<ParseMode>,
    entities: Option<&[MessageEntity]>,
) -> Result<(), Error> {
    if parse_mode.is_some() && entities.is_some() {
        return Err(Error::InvalidRequest {
            reason: format!("{field} cannot set both parse_mode and entities"),
        });
    }

    Ok(())
}

pub(crate) fn text_formatting(
    field: &str,
    text: &str,
    parse_mode: Option<ParseMode>,
    entities: Option<&[MessageEntity]>,
) -> Result<(), Error> {
    parse_mode_entities_conflict(field, parse_mode, entities)?;
    text_entities(field, text, entities)
}

pub(crate) fn optional_text_formatting(
    field: &str,
    text: Option<&str>,
    parse_mode: Option<ParseMode>,
    entities: Option<&[MessageEntity]>,
) -> Result<(), Error> {
    parse_mode_entities_conflict(field, parse_mode, entities)?;

    if parse_mode.is_none() && entities.is_none() {
        return Ok(());
    }

    let Some(text) = text else {
        return Err(Error::InvalidRequest {
            reason: format!("{field} formatting requires text"),
        });
    };

    text_entities(field, text, entities)
}

fn text_entities(field: &str, text: &str, entities: Option<&[MessageEntity]>) -> Result<(), Error> {
    let Some(entities) = entities else {
        return Ok(());
    };
    if entities.is_empty() {
        return Err(Error::InvalidRequest {
            reason: format!("{field} entities cannot be empty"),
        });
    }

    let text_len = text.encode_utf16().count();
    for entity in entities {
        validate_message_entity(field, entity, text_len)?;
    }

    Ok(())
}

fn validate_message_entity(
    field: &str,
    entity: &MessageEntity,
    text_len: usize,
) -> Result<(), Error> {
    if entity.length == 0 {
        return Err(Error::InvalidRequest {
            reason: format!("{field} entity length must be greater than 0"),
        });
    }

    let start = entity.offset as usize;
    let len = entity.length as usize;
    let Some(end) = start.checked_add(len) else {
        return Err(Error::InvalidRequest {
            reason: format!("{field} entity range exceeds text length"),
        });
    };
    if end > text_len {
        return Err(Error::InvalidRequest {
            reason: format!("{field} entity range exceeds text length"),
        });
    }

    match &entity.kind {
        MessageEntityKind::TextLink => {
            let Some(url) = entity.url.as_deref() else {
                return Err(Error::InvalidRequest {
                    reason: format!("{field} text_link entity requires url"),
                });
            };
            self::url("text_link url", url)?;
            reject_unsupported_message_entity_fields(field, entity, &["url"])?;
        }
        MessageEntityKind::TextMention => {
            let Some(user) = entity.user.as_ref() else {
                return Err(Error::InvalidRequest {
                    reason: format!("{field} text_mention entity requires user"),
                });
            };
            user.id.validate()?;
            required_text("text_mention user first_name", &user.first_name)?;
            reject_unsupported_message_entity_fields(field, entity, &["user"])?;
        }
        MessageEntityKind::CustomEmoji => {
            let Some(custom_emoji_id) = entity.custom_emoji_id.as_deref() else {
                return Err(Error::InvalidRequest {
                    reason: format!("{field} custom_emoji entity requires custom_emoji_id"),
                });
            };
            string_id("custom_emoji_id", custom_emoji_id)?;
            reject_unsupported_message_entity_fields(field, entity, &["custom_emoji_id"])?;
        }
        MessageEntityKind::Unknown(kind) => {
            return Err(Error::InvalidRequest {
                reason: format!("{field} contains unsupported entity type `{kind}`"),
            });
        }
        MessageEntityKind::DateTime => {
            return Err(Error::InvalidRequest {
                reason: format!("{field} contains unsupported entity type `date_time`"),
            });
        }
        MessageEntityKind::Mention
        | MessageEntityKind::Hashtag
        | MessageEntityKind::Cashtag
        | MessageEntityKind::BotCommand
        | MessageEntityKind::Url
        | MessageEntityKind::Email
        | MessageEntityKind::PhoneNumber
        | MessageEntityKind::Bold
        | MessageEntityKind::Italic
        | MessageEntityKind::Underline
        | MessageEntityKind::Strikethrough
        | MessageEntityKind::Spoiler
        | MessageEntityKind::Blockquote
        | MessageEntityKind::ExpandableBlockquote
        | MessageEntityKind::Code
        | MessageEntityKind::Pre => {
            let allowed = match &entity.kind {
                MessageEntityKind::Pre => &["language"][..],
                _ => &[][..],
            };
            reject_unsupported_message_entity_fields(field, entity, allowed)?;
        }
    }

    Ok(())
}

fn reject_unsupported_message_entity_fields(
    field: &str,
    entity: &MessageEntity,
    allowed: &[&str],
) -> Result<(), Error> {
    for (field_name, present) in [
        ("url", entity.url.is_some()),
        ("user", entity.user.is_some()),
        ("language", entity.language.is_some()),
        ("custom_emoji_id", entity.custom_emoji_id.is_some()),
        ("unix_time", entity.unix_time.is_some()),
        ("date_time_format", entity.date_time_format.is_some()),
    ] {
        if present && !allowed.contains(&field_name) {
            return Err(Error::InvalidRequest {
                reason: format!(
                    "{field} {} entity must not set {field_name}",
                    entity.kind.as_str()
                ),
            });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn current_unix_timestamp_for_test() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| duration.as_secs() as i64)
    }

    #[test]
    fn url_validation_rejects_ambiguous_raw_input() {
        assert!(url("url", "https://example.com/path").is_ok());
        assert!(url("url", "tg://resolve?domain=tele").is_ok());
        assert!(http_url("url", "https://example.com/path").is_ok());
        assert!(https_url("url", "https://example.com/path").is_ok());

        for value in [
            "",
            "   ",
            "https://exa\nmple.com",
            "https://exa\tmple.com",
            "https://example.com/a b",
            "https://",
            "ftp://example.com",
        ] {
            assert!(matches!(
                url("url", value),
                Err(Error::InvalidRequest { .. })
            ));
        }

        assert!(matches!(
            http_url("url", "tg://resolve?domain=tele"),
            Err(Error::InvalidRequest { .. })
        ));
        assert!(matches!(
            https_url("url", "http://example.com"),
            Err(Error::InvalidRequest { .. })
        ));
    }

    #[test]
    fn validates_suggested_post_send_date_windows() {
        let now = current_unix_timestamp_for_test();

        assert!(suggested_post_parameters_send_date("send_date", now + 600).is_ok());
        assert!(matches!(
            suggested_post_parameters_send_date("send_date", now + 299),
            Err(Error::InvalidRequest { .. })
        ));
        assert!(matches!(
            suggested_post_parameters_send_date("send_date", now + 2_679_000),
            Err(Error::InvalidRequest { .. })
        ));

        assert!(suggested_post_approval_send_date("send_date", now + 600).is_ok());
        assert!(matches!(
            suggested_post_approval_send_date("send_date", 0),
            Err(Error::InvalidRequest { .. })
        ));
        assert!(matches!(
            suggested_post_approval_send_date("send_date", now.saturating_sub(1)),
            Err(Error::InvalidRequest { .. })
        ));
        assert!(matches!(
            suggested_post_approval_send_date("send_date", now),
            Err(Error::InvalidRequest { .. })
        ));
        assert!(matches!(
            suggested_post_approval_send_date("send_date", now + 2_679_000),
            Err(Error::InvalidRequest { .. })
        ));
    }

    #[test]
    fn emoji_free_text_allows_empty_but_rejects_emoji_and_overflow() {
        assert!(emoji_free_text("tag", "", 16).is_ok());
        assert!(emoji_free_text("tag", "ops", 16).is_ok());
        assert!(matches!(
            emoji_free_text("tag", "ops🚀", 16),
            Err(Error::InvalidRequest { .. })
        ));
        assert!(matches!(
            emoji_free_text("tag", "sun☀", 16),
            Err(Error::InvalidRequest { .. })
        ));
        assert!(matches!(
            emoji_free_text("tag", "a\nb", 16),
            Err(Error::InvalidRequest { .. })
        ));
        assert!(matches!(
            emoji_free_text("tag", &"a".repeat(17), 16),
            Err(Error::InvalidRequest { .. })
        ));
    }

    #[test]
    fn text_length_range_supports_optional_and_required_short_text() {
        assert!(text_length_range("bio", "", 0, 140).is_ok());
        assert!(text_length_range("first_name", "tele", 1, 64).is_ok());
        assert!(matches!(
            text_length_range("first_name", "   ", 1, 64),
            Err(Error::InvalidRequest { .. })
        ));
        assert!(matches!(
            text_length_range("bio", "a\nb", 0, 140),
            Err(Error::InvalidRequest { .. })
        ));
        assert!(matches!(
            text_length_range("bio", &"a".repeat(141), 0, 140),
            Err(Error::InvalidRequest { .. })
        ));
    }

    #[test]
    fn username_or_empty_allows_reset_but_rejects_invalid_usernames() {
        assert!(username_or_empty("username", "").is_ok());
        assert!(username_or_empty("username", "tele_bot").is_ok());

        for value in ["abcd", "bad-name", "bad name", "абвгд"] {
            assert!(matches!(
                username_or_empty("username", value),
                Err(Error::InvalidRequest { .. })
            ));
        }
    }
}

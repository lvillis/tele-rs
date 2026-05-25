use crate::Error;

use super::common::ParseMode;
use super::message::{MessageEntity, MessageEntityKind};
use super::telegram::{ReplyMarkup, ReplyParameters, SuggestedPostParameters};

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

pub(crate) fn string_id(field: &str, value: &str) -> Result<(), Error> {
    required_string(field, value)?;
    if value.chars().any(char::is_control) {
        return Err(Error::InvalidRequest {
            reason: format!("{field} must not contain control characters"),
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
            required_string("url", url)?;
        }
        MessageEntityKind::TextMention => {
            if entity.user.is_none() {
                return Err(Error::InvalidRequest {
                    reason: format!("{field} text_mention entity requires user"),
                });
            }
        }
        MessageEntityKind::CustomEmoji => {
            let Some(custom_emoji_id) = entity.custom_emoji_id.as_deref() else {
                return Err(Error::InvalidRequest {
                    reason: format!("{field} custom_emoji entity requires custom_emoji_id"),
                });
            };
            string_id("custom_emoji_id", custom_emoji_id)?;
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
        | MessageEntityKind::Pre => {}
    }

    Ok(())
}

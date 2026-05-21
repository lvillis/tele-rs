use crate::Error;

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

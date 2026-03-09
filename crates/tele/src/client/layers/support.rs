use super::bootstrap::WebAppQueryPayload;
use super::*;

pub(crate) fn invalid_request(reason: impl Into<String>) -> Error {
    Error::InvalidRequest {
        reason: reason.into(),
    }
}

pub(crate) fn serialize_request_value<T>(value: T) -> Result<serde_json::Value>
where
    T: Serialize,
{
    serde_json::to_value(value).map_err(|source| Error::SerializeRequest { source })
}

pub(crate) fn normalize_language_code(language_code: Option<String>) -> Result<Option<String>> {
    let Some(language_code) = language_code else {
        return Ok(None);
    };
    if language_code.trim().is_empty() {
        return Err(invalid_request("language_code cannot be empty"));
    }
    Ok(Some(language_code))
}

pub(crate) fn build_set_my_commands_request(
    commands: Vec<BotCommand>,
    scope: Option<BotCommandScope>,
    language_code: Option<String>,
) -> Result<SetMyCommandsRequest> {
    let mut request = SetMyCommandsRequest::new(commands)?;
    request.scope = scope;
    request.language_code = normalize_language_code(language_code)?;
    Ok(request)
}

pub(crate) fn commands_get_request(request: &SetMyCommandsRequest) -> GetMyCommandsRequest {
    GetMyCommandsRequest {
        scope: request.scope.clone(),
        language_code: request.language_code.clone(),
    }
}

#[cfg(feature = "bot")]
pub(crate) fn typed_commands_request<C>(
    scope: Option<BotCommandScope>,
    language_code: Option<String>,
) -> Result<SetMyCommandsRequest>
where
    C: crate::bot::BotCommands,
{
    build_set_my_commands_request(crate::bot::command_definitions::<C>(), scope, language_code)
}

pub(crate) fn update_chat_id(update: &Update) -> Option<i64> {
    if let Some(message) = update.message.as_deref() {
        return Some(message.chat.id);
    }
    if let Some(message) = update.edited_message.as_deref() {
        return Some(message.chat.id);
    }
    if let Some(message) = update.channel_post.as_deref() {
        return Some(message.chat.id);
    }
    if let Some(message) = update.edited_channel_post.as_deref() {
        return Some(message.chat.id);
    }
    if let Some(request) = update.chat_join_request.as_ref() {
        return Some(request.chat.id);
    }
    if let Some(member_update) = update.chat_member.as_ref() {
        return Some(member_update.chat.id);
    }
    if let Some(member_update) = update.my_chat_member.as_ref() {
        return Some(member_update.chat.id);
    }

    update
        .callback_query
        .as_ref()
        .and_then(|query| query.message.as_deref())
        .map(|message| message.chat.id)
}

pub(crate) fn update_message(update: &Update) -> Option<&Message> {
    if let Some(message) = update.message.as_deref() {
        return Some(message);
    }
    if let Some(message) = update.edited_message.as_deref() {
        return Some(message);
    }
    if let Some(message) = update.channel_post.as_deref() {
        return Some(message);
    }
    if let Some(message) = update.edited_channel_post.as_deref() {
        return Some(message);
    }

    update
        .callback_query
        .as_ref()
        .and_then(|query| query.message.as_deref())
}

pub(crate) fn reply_chat_id(update: &Update) -> Result<i64> {
    if let Some(request) = update.chat_join_request.as_ref() {
        return Ok(request.user_chat_id);
    }

    update_chat_id(update)
        .ok_or_else(|| invalid_request("update does not contain a chat id for reply"))
}

pub(crate) fn callback_query_id(update: &Update) -> Option<String> {
    update.callback_query.as_ref().map(|query| query.id.clone())
}

pub(crate) fn parse_web_app_query_payload<T>(
    web_app_data: &WebAppData,
) -> Result<WebAppQueryPayload<T>>
where
    T: DeserializeOwned,
{
    let mut value: serde_json::Value =
        serde_json::from_str(&web_app_data.data).map_err(|source| Error::InvalidRequest {
            reason: format!("invalid web_app_data JSON payload: {source}"),
        })?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| invalid_request("web_app_data payload must be a JSON object"))?;

    let query_id = object
        .remove("query_id")
        .and_then(|value| value.as_str().map(str::to_owned))
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| invalid_request("web_app_data payload is missing non-empty `query_id`"))?;

    let payload = serde_json::from_value::<T>(serde_json::Value::Object(object.clone())).map_err(
        |source| Error::InvalidRequest {
            reason: format!("failed to parse typed web_app_data payload: {source}"),
        },
    )?;

    Ok(WebAppQueryPayload { query_id, payload })
}

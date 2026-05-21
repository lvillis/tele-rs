use super::bootstrap::WebAppQueryPayload;
use super::*;

pub(crate) fn invalid_request(reason: impl Into<String>) -> Error {
    Error::InvalidRequest {
        reason: reason.into(),
    }
}

pub(crate) fn normalize_language_code(language_code: Option<String>) -> Result<Option<String>> {
    let Some(language_code) = language_code else {
        return Ok(None);
    };
    crate::types::command::validate_language_code_value(&language_code)?;
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
    if let Some(message) = update.business_message.as_deref() {
        return Some(message);
    }
    if let Some(message) = update.edited_business_message.as_deref() {
        return Some(message);
    }
    if let Some(message) = update.guest_message.as_deref() {
        return Some(message);
    }

    update
        .callback_query
        .as_ref()
        .and_then(|query| query.message.as_deref())
}

pub(crate) struct ReplyContext {
    pub(crate) chat_id: i64,
    pub(crate) reply_parameters: Option<ReplyParameters>,
    pub(crate) business_connection_id: Option<String>,
}

pub(crate) fn reply_context(update: &Update) -> Result<ReplyContext> {
    if update.guest_message.is_some() {
        return Err(invalid_request(
            "guest message replies require answerGuestQuery; ordinary sendMessage cannot target a guest query",
        ));
    }

    if let Some(request) = update.chat_join_request.as_ref() {
        return Ok(ReplyContext {
            chat_id: request.user_chat_id,
            reply_parameters: None,
            business_connection_id: None,
        });
    }

    if let Some(message) = update_message(update) {
        return Ok(ReplyContext {
            chat_id: message.chat.id,
            reply_parameters: Some(ReplyParameters::new(message.message_id)),
            business_connection_id: message.business_connection_id.clone(),
        });
    }

    if let Some(deleted) = update.deleted_business_messages.as_ref() {
        return Ok(ReplyContext {
            chat_id: deleted.chat.id,
            reply_parameters: None,
            business_connection_id: Some(deleted.business_connection_id.clone()),
        });
    }

    let chat_id = update
        .chat_member
        .as_ref()
        .or(update.my_chat_member.as_ref())
        .map(|member_update| member_update.chat.id)
        .ok_or_else(|| invalid_request("update does not contain a chat id for reply"))?;

    Ok(ReplyContext {
        chat_id,
        reply_parameters: None,
        business_connection_id: None,
    })
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

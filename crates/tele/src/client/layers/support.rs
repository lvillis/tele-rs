use super::bootstrap::WebAppQueryPayload;
use super::*;
use crate::types::message::MaybeInaccessibleMessage;

const MISSING_REPLY_TARGET_REASON: &str = "update does not contain a chat id for reply";
const UNSUPPORTED_GUEST_REPLY_REASON: &str = "guest message replies require answerGuestQuery; ordinary sendMessage cannot target a guest query";

pub(crate) fn invalid_request(reason: impl Into<String>) -> Error {
    Error::InvalidRequest {
        reason: reason.into(),
    }
}

#[cfg(feature = "bot")]
pub(crate) fn is_unaddressable_reply_error(error: &Error) -> bool {
    matches!(
        error,
        Error::InvalidRequest { reason }
            if reason == MISSING_REPLY_TARGET_REASON || reason == UNSUPPORTED_GUEST_REPLY_REASON
    )
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
        .and_then(|message| message.accessible())
}

pub(crate) struct ReplyContext {
    pub(crate) chat_id: i64,
    pub(crate) message_thread_id: Option<i64>,
    pub(crate) direct_messages_topic_id: Option<i64>,
    pub(crate) reply_parameters: Option<ReplyParameters>,
    pub(crate) business_connection_id: Option<String>,
}

impl ReplyContext {
    fn chat(chat_id: i64) -> Self {
        Self {
            chat_id,
            message_thread_id: None,
            direct_messages_topic_id: None,
            reply_parameters: None,
            business_connection_id: None,
        }
    }

    fn from_message(message: &Message) -> Self {
        Self::chat(message.chat.id).with_message_context(message)
    }

    fn from_maybe_inaccessible_message(message: &MaybeInaccessibleMessage) -> Self {
        if let Some(message) = message.accessible() {
            return Self::from_message(message);
        }

        Self::chat(message.chat().id).replying_to(message.message_id())
    }

    fn with_message_context(mut self, message: &Message) -> Self {
        self.message_thread_id = message.message_thread_id;
        self.direct_messages_topic_id = message
            .direct_messages_topic
            .as_ref()
            .map(|topic| topic.topic_id);
        self.reply_parameters = Some(ReplyParameters::new(message.message_id));
        self.business_connection_id = message.business_connection_id.clone();
        self
    }

    fn replying_to(mut self, message_id: MessageId) -> Self {
        self.reply_parameters = Some(ReplyParameters::new(message_id));
        self
    }

    fn with_business_connection_id(mut self, business_connection_id: String) -> Self {
        self.business_connection_id = Some(business_connection_id);
        self
    }
}

pub(crate) fn reply_context(update: &Update) -> Result<ReplyContext> {
    if update.guest_message.is_some() {
        return Err(invalid_request(UNSUPPORTED_GUEST_REPLY_REASON));
    }

    if let Some(request) = update.chat_join_request.as_ref() {
        return Ok(ReplyContext::chat(request.user_chat_id));
    }

    if let Some(message) = update_message(update) {
        return Ok(ReplyContext::from_message(message));
    }

    if let Some(message) = update
        .callback_query
        .as_ref()
        .and_then(|query| query.message.as_deref())
    {
        return Ok(ReplyContext::from_maybe_inaccessible_message(message));
    }

    if let Some(deleted) = update.deleted_business_messages.as_ref() {
        return Ok(ReplyContext::chat(deleted.chat.id)
            .with_business_connection_id(deleted.business_connection_id.clone()));
    }

    if let Some(connection) = update.business_connection.as_ref() {
        return Ok(ReplyContext::chat(connection.user_chat_id)
            .with_business_connection_id(connection.id.clone()));
    }

    if let Some(reaction) = update.message_reaction.as_ref() {
        return Ok(ReplyContext::chat(reaction.chat.id).replying_to(reaction.message_id));
    }

    if let Some(reaction_count) = update.message_reaction_count.as_ref() {
        return Ok(
            ReplyContext::chat(reaction_count.chat.id).replying_to(reaction_count.message_id)
        );
    }

    if let Some(answer) = update.poll_answer.as_ref()
        && let Some(voter_chat) = answer.voter_chat.as_ref()
    {
        return Ok(ReplyContext::chat(voter_chat.id));
    }

    if let Some(boost) = update.chat_boost.as_ref() {
        return Ok(ReplyContext::chat(boost.chat.id));
    }

    if let Some(boost) = update.removed_chat_boost.as_ref() {
        return Ok(ReplyContext::chat(boost.chat.id));
    }

    let chat_id = update
        .chat_member
        .as_ref()
        .or(update.my_chat_member.as_ref())
        .map(|member_update| member_update.chat.id)
        .ok_or_else(|| invalid_request(MISSING_REPLY_TARGET_REASON))?;

    Ok(ReplyContext::chat(chat_id))
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

    let payload = serde_json::from_value::<T>(serde_json::Value::Object(std::mem::take(object)))
        .map_err(|source| Error::InvalidRequest {
            reason: format!("failed to parse typed web_app_data payload: {source}"),
        })?;

    Ok(WebAppQueryPayload { query_id, payload })
}

use super::*;
use crate::types::validation::MAX_MESSAGE_TEXT_CHARS;

const DEFAULT_REJECTION_MESSAGE: &str = "request rejected";

/// Structured route-level rejection reason.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RouteRejection {
    Message(String),
    GroupOnly,
    SupergroupOnly,
    AdminOnly,
    OwnerOnly,
    ActorRequired,
    SubjectRequired,
    ChatContextRequired,
    MissingActorCapabilities(Vec<ChatAdministratorCapability>),
    MissingBotCapabilities(Vec<ChatAdministratorCapability>),
    Throttled,
}

impl RouteRejection {
    pub fn message(&self) -> String {
        match self {
            Self::Message(message) => normalize_user_message(message, DEFAULT_REJECTION_MESSAGE),
            Self::GroupOnly => "this route is only available in group chats".to_owned(),
            Self::SupergroupOnly => "this route is only available in supergroups".to_owned(),
            Self::AdminOnly => "chat administrators only".to_owned(),
            Self::OwnerOnly => "chat owner only".to_owned(),
            Self::ActorRequired => "this route requires an actor user".to_owned(),
            Self::SubjectRequired => "this route requires a subject user".to_owned(),
            Self::ChatContextRequired => "this route requires a chat context".to_owned(),
            Self::MissingActorCapabilities(missing) => format!(
                "missing required capabilities: {}",
                missing
                    .iter()
                    .map(ChatAdministratorCapability::as_str)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Self::MissingBotCapabilities(missing) => format!(
                "bot is missing required capabilities: {}",
                missing
                    .iter()
                    .map(ChatAdministratorCapability::as_str)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Self::Throttled => "too many matching requests, please retry shortly".to_owned(),
        }
    }

    pub fn custom(message: impl Into<String>) -> Self {
        let message = message.into();
        Self::Message(normalize_user_message(&message, DEFAULT_REJECTION_MESSAGE))
    }
}

pub(crate) fn normalize_user_message(message: &str, default_message: &str) -> String {
    let mut normalized = message
        .trim()
        .chars()
        .map(|character| {
            if is_disallowed_rejection_character(character) {
                ' '
            } else {
                character
            }
        })
        .take(MAX_MESSAGE_TEXT_CHARS)
        .collect::<String>();
    normalized = normalized.trim().to_owned();

    if normalized.is_empty() {
        default_message.to_owned()
    } else {
        normalized
    }
}

fn is_disallowed_rejection_character(character: char) -> bool {
    character.is_control() && !matches!(character, '\n' | '\r' | '\t')
}

/// Handler error type that separates route rejections from internal failures.
#[derive(Debug)]
pub enum HandlerError {
    Rejected(RouteRejection),
    Internal(Error),
}

impl HandlerError {
    pub fn user(message: impl Into<String>) -> Self {
        Self::Rejected(RouteRejection::custom(message))
    }

    pub fn rejected(rejection: RouteRejection) -> Self {
        Self::Rejected(rejection)
    }

    pub fn internal(error: Error) -> Self {
        Self::Internal(error)
    }
}

impl From<RouteRejection> for HandlerError {
    fn from(value: RouteRejection) -> Self {
        Self::Rejected(value)
    }
}

impl From<Error> for HandlerError {
    fn from(value: Error) -> Self {
        Self::Internal(value)
    }
}

/// Ergonomic result type for bot handlers.
pub type HandlerResult = std::result::Result<(), HandlerError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn custom_rejection_message_is_sendable_text() {
        let rejection = RouteRejection::custom(" invalid\u{0007}input ");
        assert_eq!(rejection.message(), "invalid input");

        let rejection = RouteRejection::custom(" \n\t ");
        assert_eq!(rejection.message(), DEFAULT_REJECTION_MESSAGE);

        let rejection = RouteRejection::custom("a".repeat(MAX_MESSAGE_TEXT_CHARS + 1));
        assert_eq!(rejection.message().chars().count(), MAX_MESSAGE_TEXT_CHARS);
    }
}

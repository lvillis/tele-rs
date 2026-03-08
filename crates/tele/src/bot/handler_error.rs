use super::*;

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
            Self::Message(message) => message.clone(),
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
        Self::Message(message.into())
    }
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

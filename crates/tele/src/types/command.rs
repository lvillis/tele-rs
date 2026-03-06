use serde::{Deserialize, Serialize};

use crate::types::common::{ChatId, UserId};
use crate::{Error, Result};

/// Telegram bot command object.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct BotCommand {
    pub command: String,
    pub description: String,
}

impl BotCommand {
    pub fn new(command: impl Into<String>, description: impl Into<String>) -> Result<Self> {
        let command = command.into();
        let description = description.into();

        if command.trim().is_empty() {
            return Err(Error::InvalidRequest {
                reason: "command cannot be empty".to_owned(),
            });
        }

        if description.trim().is_empty() {
            return Err(Error::InvalidRequest {
                reason: "command description cannot be empty".to_owned(),
            });
        }

        Ok(Self {
            command,
            description,
        })
    }
}

/// Scope for bot command settings.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BotCommandScope {
    Default,
    AllPrivateChats,
    AllGroupChats,
    AllChatAdministrators,
    Chat { chat_id: ChatId },
    ChatAdministrators { chat_id: ChatId },
    ChatMember { chat_id: ChatId, user_id: UserId },
}

/// `setMyCommands` request.
#[derive(Clone, Debug, Serialize)]
pub struct SetMyCommandsRequest {
    pub commands: Vec<BotCommand>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<BotCommandScope>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language_code: Option<String>,
}

impl SetMyCommandsRequest {
    pub fn new(commands: Vec<BotCommand>) -> Result<Self> {
        if commands.is_empty() {
            return Err(Error::InvalidRequest {
                reason: "setMyCommands requires at least one command".to_owned(),
            });
        }

        Ok(Self {
            commands,
            scope: None,
            language_code: None,
        })
    }

    pub fn scope(mut self, scope: BotCommandScope) -> Self {
        self.scope = Some(scope);
        self
    }

    pub fn language_code(mut self, language_code: impl Into<String>) -> Result<Self> {
        let language_code = language_code.into();
        if language_code.trim().is_empty() {
            return Err(Error::InvalidRequest {
                reason: "language_code cannot be empty".to_owned(),
            });
        }
        self.language_code = Some(language_code);
        Ok(self)
    }
}

/// `getMyCommands` request.
#[derive(Clone, Debug, Default, Serialize)]
pub struct GetMyCommandsRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<BotCommandScope>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language_code: Option<String>,
}

/// `deleteMyCommands` request.
#[derive(Clone, Debug, Default, Serialize)]
pub struct DeleteMyCommandsRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<BotCommandScope>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language_code: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct SetMyNameRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language_code: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct GetMyNameRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language_code: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct SetMyDescriptionRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language_code: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct GetMyDescriptionRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language_code: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct SetMyShortDescriptionRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub short_description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language_code: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct GetMyShortDescriptionRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language_code: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BotName {
    pub name: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BotDescription {
    pub description: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BotShortDescription {
    pub short_description: String,
}

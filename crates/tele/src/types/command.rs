use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::common::{ChatId, UserId};
use crate::{Error, Result};

const MAX_BOT_NAME_CHARS: usize = 64;
const MAX_BOT_DESCRIPTION_CHARS: usize = 512;
const MAX_BOT_SHORT_DESCRIPTION_CHARS: usize = 120;
const MAX_LANGUAGE_CODE_BYTES: usize = 35;

/// Telegram bot command object.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub struct BotCommand {
    pub command: String,
    pub description: String,
    #[serde(flatten, skip_serializing)]
    pub extra: BTreeMap<String, Value>,
}

impl PartialEq for BotCommand {
    fn eq(&self, other: &Self) -> bool {
        self.command == other.command && self.description == other.description
    }
}

impl Eq for BotCommand {}

impl BotCommand {
    pub fn new(command: impl Into<String>, description: impl Into<String>) -> Result<Self> {
        let command = command.into();
        let description = description.into();

        validate_bot_command_name(&command)?;
        validate_bot_command_description(&description)?;

        Ok(Self {
            command,
            description,
            extra: BTreeMap::new(),
        })
    }
}

fn validate_bot_command_name(command: &str) -> Result<()> {
    let length = command.len();
    if length == 0 || length > 32 {
        return Err(Error::InvalidRequest {
            reason: "command must be 1-32 characters".to_owned(),
        });
    }

    if !command
        .bytes()
        .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
    {
        return Err(Error::InvalidRequest {
            reason: "command must contain only lowercase ASCII letters, digits, or `_`".to_owned(),
        });
    }

    Ok(())
}

fn validate_bot_command_description(description: &str) -> Result<()> {
    let length = description.chars().count();
    if !(1..=256).contains(&length) {
        return Err(Error::InvalidRequest {
            reason: "command description must be 1-256 characters".to_owned(),
        });
    }

    if description.chars().any(char::is_control) {
        return Err(Error::InvalidRequest {
            reason: "command description must not contain control characters".to_owned(),
        });
    }

    Ok(())
}

fn validate_bot_commands(commands: &[BotCommand]) -> Result<()> {
    if commands.is_empty() {
        return Err(Error::InvalidRequest {
            reason: "setMyCommands requires at least one command".to_owned(),
        });
    }
    if commands.len() > 100 {
        return Err(Error::InvalidRequest {
            reason: "setMyCommands accepts at most 100 commands".to_owned(),
        });
    }

    let mut seen = BTreeSet::new();
    for command in commands {
        validate_bot_command_name(&command.command)?;
        validate_bot_command_description(&command.description)?;
        if !seen.insert(command.command.as_str()) {
            return Err(Error::InvalidRequest {
                reason: format!("duplicate bot command `{}`", command.command),
            });
        }
    }

    Ok(())
}

pub(crate) fn validate_language_code_value(language_code: &str) -> Result<()> {
    if language_code.len() > MAX_LANGUAGE_CODE_BYTES {
        return Err(Error::InvalidRequest {
            reason: format!("language_code must be at most {MAX_LANGUAGE_CODE_BYTES} bytes")
                .to_owned(),
        });
    }
    if language_code.is_empty() {
        return Ok(());
    }

    if !language_code
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
    {
        return Err(Error::InvalidRequest {
            reason: "language_code must contain only ASCII letters, digits, or `-`".to_owned(),
        });
    }

    if language_code.starts_with('-')
        || language_code.ends_with('-')
        || language_code.contains("--")
    {
        return Err(Error::InvalidRequest {
            reason: "language_code must not contain empty subtags".to_owned(),
        });
    }

    Ok(())
}

fn validate_language_code_field(language_code: Option<&str>) -> Result<()> {
    if let Some(language_code) = language_code {
        validate_language_code_value(language_code)?;
    }

    Ok(())
}

#[derive(Clone, Copy)]
enum TextLayout {
    SingleLine,
    MultiLine,
}

fn validate_optional_text(
    field: &str,
    value: Option<&str>,
    layout: TextLayout,
    max_chars: usize,
) -> Result<()> {
    let Some(value) = value else {
        return Ok(());
    };

    let length = value.chars().count();
    if length > max_chars {
        return Err(Error::InvalidRequest {
            reason: format!("{field} must be at most {max_chars} characters"),
        });
    }
    if value
        .chars()
        .any(|character| is_disallowed_control(character, layout))
    {
        return Err(Error::InvalidRequest {
            reason: format!("{field} must not contain control characters"),
        });
    }

    Ok(())
}

fn is_disallowed_control(character: char, layout: TextLayout) -> bool {
    match layout {
        TextLayout::SingleLine => character.is_control(),
        TextLayout::MultiLine => character.is_control() && !matches!(character, '\n' | '\r' | '\t'),
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
        validate_bot_commands(&commands)?;

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
        validate_language_code_value(&language_code)?;
        self.language_code = Some(language_code);
        Ok(self)
    }

    pub fn validate(&self) -> Result<()> {
        validate_bot_commands(&self.commands)?;
        validate_language_code_field(self.language_code.as_deref())
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

impl GetMyCommandsRequest {
    pub fn validate(&self) -> Result<()> {
        validate_language_code_field(self.language_code.as_deref())
    }
}

/// `deleteMyCommands` request.
#[derive(Clone, Debug, Default, Serialize)]
pub struct DeleteMyCommandsRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<BotCommandScope>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language_code: Option<String>,
}

impl DeleteMyCommandsRequest {
    pub fn validate(&self) -> Result<()> {
        validate_language_code_field(self.language_code.as_deref())
    }
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct SetMyNameRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language_code: Option<String>,
}

impl SetMyNameRequest {
    pub fn validate(&self) -> Result<()> {
        validate_optional_text(
            "name",
            self.name.as_deref(),
            TextLayout::SingleLine,
            MAX_BOT_NAME_CHARS,
        )?;
        validate_language_code_field(self.language_code.as_deref())
    }
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct GetMyNameRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language_code: Option<String>,
}

impl GetMyNameRequest {
    pub fn validate(&self) -> Result<()> {
        validate_language_code_field(self.language_code.as_deref())
    }
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct SetMyDescriptionRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language_code: Option<String>,
}

impl SetMyDescriptionRequest {
    pub fn validate(&self) -> Result<()> {
        validate_optional_text(
            "description",
            self.description.as_deref(),
            TextLayout::MultiLine,
            MAX_BOT_DESCRIPTION_CHARS,
        )?;
        validate_language_code_field(self.language_code.as_deref())
    }
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct GetMyDescriptionRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language_code: Option<String>,
}

impl GetMyDescriptionRequest {
    pub fn validate(&self) -> Result<()> {
        validate_language_code_field(self.language_code.as_deref())
    }
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct SetMyShortDescriptionRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub short_description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language_code: Option<String>,
}

impl SetMyShortDescriptionRequest {
    pub fn validate(&self) -> Result<()> {
        validate_optional_text(
            "short_description",
            self.short_description.as_deref(),
            TextLayout::SingleLine,
            MAX_BOT_SHORT_DESCRIPTION_CHARS,
        )?;
        validate_language_code_field(self.language_code.as_deref())
    }
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct GetMyShortDescriptionRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language_code: Option<String>,
}

impl GetMyShortDescriptionRequest {
    pub fn validate(&self) -> Result<()> {
        validate_language_code_field(self.language_code.as_deref())
    }
}

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct BotName {
    pub name: String,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct BotDescription {
    pub description: String,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct BotShortDescription {
    pub short_description: String,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_bot_command_contract() -> Result<()> {
        assert!(BotCommand::new("start_1", "start the bot").is_ok());
        assert!(BotCommand::new("2fa", "manage two factor auth").is_ok());
        assert!(BotCommand::new("_debug", "debug current state").is_ok());
        assert!(BotCommand::new("x", "g").is_ok());
        assert!(BotCommand::new("ok", "go").is_ok());

        for command in [
            "",
            "Start",
            "/start",
            "start-bot",
            "a_very_long_command_name_over_32_chars",
        ] {
            assert!(matches!(
                BotCommand::new(command, "valid description"),
                Err(Error::InvalidRequest { .. })
            ));
        }

        for description in ["", "bad\ntext"] {
            assert!(matches!(
                BotCommand::new("start", description),
                Err(Error::InvalidRequest { .. })
            ));
        }

        Ok(())
    }

    #[test]
    fn bot_command_preserves_future_response_fields_without_resending_them()
    -> std::result::Result<(), Box<dyn std::error::Error>> {
        let mut command: BotCommand = serde_json::from_value(serde_json::json!({
            "command": "start",
            "description": "start the bot",
            "future_command_field": "response-only"
        }))?;

        assert_eq!(command.extra["future_command_field"], "response-only");
        assert_eq!(command, BotCommand::new("start", "start the bot")?);

        command
            .extra
            .insert("another_future_field".to_owned(), serde_json::json!(true));
        let value = serde_json::to_value(command)?;
        assert_eq!(value["command"], "start");
        assert_eq!(value["description"], "start the bot");
        assert!(value.get("future_command_field").is_none());
        assert!(value.get("another_future_field").is_none());

        Ok(())
    }

    #[test]
    fn validates_set_my_commands_collection() -> Result<()> {
        assert!(matches!(
            SetMyCommandsRequest::new(Vec::new()),
            Err(Error::InvalidRequest { .. })
        ));

        let duplicate = vec![
            BotCommand::new("start", "start the bot")?,
            BotCommand::new("start", "start again")?,
        ];
        assert!(matches!(
            SetMyCommandsRequest::new(duplicate),
            Err(Error::InvalidRequest { .. })
        ));

        let too_many = (0..101)
            .map(|index| BotCommand::new(format!("cmd{index}"), "valid description"))
            .collect::<Result<Vec<_>>>()?;
        assert!(matches!(
            SetMyCommandsRequest::new(too_many),
            Err(Error::InvalidRequest { .. })
        ));

        Ok(())
    }

    #[test]
    fn validates_language_codes_on_public_requests() -> Result<()> {
        SetMyCommandsRequest::new(vec![BotCommand::new("start", "start the bot")?])?
            .language_code("zh-hans")?
            .validate()?;
        SetMyCommandsRequest::new(vec![BotCommand::new("start", "start the bot")?])?
            .language_code("")?
            .validate()?;

        GetMyCommandsRequest {
            language_code: Some(String::new()),
            ..GetMyCommandsRequest::default()
        }
        .validate()?;

        for language_code in ["-", "en-", "en--us", "en us", "en_us"] {
            let request = GetMyCommandsRequest {
                language_code: Some(language_code.to_owned()),
                ..GetMyCommandsRequest::default()
            };
            assert!(matches!(
                request.validate(),
                Err(Error::InvalidRequest { .. })
            ));
        }

        Ok(())
    }

    #[test]
    fn validates_bot_profile_text_limits() {
        let valid = SetMyNameRequest {
            name: Some(String::new()),
            language_code: Some(String::new()),
        };
        assert!(valid.validate().is_ok());

        let invalid_name = SetMyNameRequest {
            name: Some("a".repeat(MAX_BOT_NAME_CHARS + 1)),
            ..SetMyNameRequest::default()
        };
        assert!(matches!(
            invalid_name.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let multiline_name = SetMyNameRequest {
            name: Some("bot\nname".to_owned()),
            ..SetMyNameRequest::default()
        };
        assert!(matches!(
            multiline_name.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let invalid_description = SetMyDescriptionRequest {
            description: Some("a".repeat(MAX_BOT_DESCRIPTION_CHARS + 1)),
            ..SetMyDescriptionRequest::default()
        };
        assert!(matches!(
            invalid_description.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let multiline_description = SetMyDescriptionRequest {
            description: Some("hello\nworld".to_owned()),
            ..SetMyDescriptionRequest::default()
        };
        assert!(multiline_description.validate().is_ok());

        let invalid_short_description = SetMyShortDescriptionRequest {
            short_description: Some("a".repeat(MAX_BOT_SHORT_DESCRIPTION_CHARS + 1)),
            ..SetMyShortDescriptionRequest::default()
        };
        assert!(matches!(
            invalid_short_description.validate(),
            Err(Error::InvalidRequest { .. })
        ));
    }

    #[test]
    fn bot_profile_responses_preserve_future_fields()
    -> std::result::Result<(), Box<dyn std::error::Error>> {
        let name: BotName = serde_json::from_value(serde_json::json!({
            "name": "Tele",
            "future_name_field": true
        }))?;
        let description: BotDescription = serde_json::from_value(serde_json::json!({
            "description": "Long description",
            "future_description_field": "kept"
        }))?;
        let short_description: BotShortDescription = serde_json::from_value(serde_json::json!({
            "short_description": "Short",
            "future_short_description_field": 7
        }))?;

        assert_eq!(name.name, "Tele");
        assert_eq!(name.extra["future_name_field"], true);
        assert_eq!(description.description, "Long description");
        assert_eq!(description.extra["future_description_field"], "kept");
        assert_eq!(short_description.short_description, "Short");
        assert_eq!(short_description.extra["future_short_description_field"], 7);

        Ok(())
    }
}

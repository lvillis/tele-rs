use super::*;

/// High-level chat menu button configuration used by app setup and Web App APIs.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MenuButtonConfig {
    pub chat_id: Option<i64>,
    pub menu_button: MenuButton,
}

impl MenuButtonConfig {
    pub fn new(menu_button: impl Into<MenuButton>) -> Self {
        Self {
            chat_id: None,
            menu_button: menu_button.into(),
        }
    }

    pub fn for_chat(chat_id: i64, menu_button: impl Into<MenuButton>) -> Self {
        Self::new(menu_button).chat_id(chat_id)
    }

    pub fn chat_id(mut self, chat_id: i64) -> Self {
        self.chat_id = Some(chat_id);
        self
    }

    pub fn menu_button(mut self, menu_button: impl Into<MenuButton>) -> Self {
        self.menu_button = menu_button.into();
        self
    }

    pub fn default_button() -> Self {
        Self::new(MenuButton::default_button())
    }

    pub fn commands() -> Self {
        Self::new(MenuButton::commands())
    }

    pub fn web_app(
        text: impl Into<String>,
        web_app: impl Into<crate::types::telegram::WebAppInfo>,
    ) -> Self {
        Self::new(MenuButton::web_app(text, web_app))
    }

    pub fn for_chat_default(chat_id: i64) -> Self {
        Self::default_button().chat_id(chat_id)
    }

    pub fn for_chat_commands(chat_id: i64) -> Self {
        Self::commands().chat_id(chat_id)
    }

    pub fn for_chat_web_app(
        chat_id: i64,
        text: impl Into<String>,
        web_app: impl Into<crate::types::telegram::WebAppInfo>,
    ) -> Self {
        Self::web_app(text, web_app).chat_id(chat_id)
    }
}

impl From<MenuButton> for MenuButtonConfig {
    fn from(value: MenuButton) -> Self {
        Self::new(value)
    }
}

impl From<crate::types::advanced::AdvancedSetChatMenuButtonRequest> for MenuButtonConfig {
    fn from(value: crate::types::advanced::AdvancedSetChatMenuButtonRequest) -> Self {
        Self {
            chat_id: value.chat_id,
            menu_button: value.menu_button.unwrap_or_default(),
        }
    }
}

impl From<&crate::types::advanced::AdvancedSetChatMenuButtonRequest> for MenuButtonConfig {
    fn from(value: &crate::types::advanced::AdvancedSetChatMenuButtonRequest) -> Self {
        Self {
            chat_id: value.chat_id,
            menu_button: value.menu_button.clone().unwrap_or_default(),
        }
    }
}

impl From<MenuButtonConfig> for crate::types::advanced::AdvancedGetChatMenuButtonRequest {
    fn from(value: MenuButtonConfig) -> Self {
        Self {
            chat_id: value.chat_id,
        }
    }
}

impl From<&MenuButtonConfig> for crate::types::advanced::AdvancedGetChatMenuButtonRequest {
    fn from(value: &MenuButtonConfig) -> Self {
        Self {
            chat_id: value.chat_id,
        }
    }
}

impl From<MenuButtonConfig> for crate::types::advanced::AdvancedSetChatMenuButtonRequest {
    fn from(value: MenuButtonConfig) -> Self {
        Self {
            chat_id: value.chat_id,
            menu_button: Some(value.menu_button),
        }
    }
}

impl From<&MenuButtonConfig> for crate::types::advanced::AdvancedSetChatMenuButtonRequest {
    fn from(value: &MenuButtonConfig) -> Self {
        Self {
            chat_id: value.chat_id,
            menu_button: Some(value.menu_button.clone()),
        }
    }
}

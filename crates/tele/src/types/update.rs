use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::bot::User;
use crate::types::message::{Message, Poll};
use crate::types::telegram::{InlineQueryResult, InlineQueryResultsButton, WebAppData};

/// Classified update payload kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[non_exhaustive]
pub enum UpdateKind {
    Message,
    EditedMessage,
    ChannelPost,
    EditedChannelPost,
    CallbackQuery,
    InlineQuery,
    ChosenInlineResult,
    Poll,
    PollAnswer,
    MyChatMember,
    ChatMember,
    ChatJoinRequest,
    Unknown,
}

const KNOWN_UPDATE_KINDS: [UpdateKind; 12] = [
    UpdateKind::Message,
    UpdateKind::EditedMessage,
    UpdateKind::ChannelPost,
    UpdateKind::EditedChannelPost,
    UpdateKind::CallbackQuery,
    UpdateKind::InlineQuery,
    UpdateKind::ChosenInlineResult,
    UpdateKind::Poll,
    UpdateKind::PollAnswer,
    UpdateKind::MyChatMember,
    UpdateKind::ChatMember,
    UpdateKind::ChatJoinRequest,
];

/// Telegram callback query object.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct CallbackQuery {
    pub id: String,
    pub from: User,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<Message>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inline_message_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chat_instance: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram inline query object.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct InlineQuery {
    pub id: String,
    pub from: User,
    pub query: String,
    pub offset: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chat_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<Value>,
}

/// Telegram chosen inline result object.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ChosenInlineResult {
    pub result_id: String,
    pub from: User,
    pub query: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inline_message_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<Value>,
}

/// Telegram poll answer object.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct PollAnswer {
    pub poll_id: String,
    pub user: User,
    pub option_ids: Vec<u8>,
}

/// Telegram update object.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Update {
    pub update_id: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<Message>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub edited_message: Option<Message>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel_post: Option<Message>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub edited_channel_post: Option<Message>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub callback_query: Option<CallbackQuery>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inline_query: Option<InlineQuery>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chosen_inline_result: Option<ChosenInlineResult>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub poll: Option<Poll>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub poll_answer: Option<PollAnswer>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub my_chat_member: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chat_member: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chat_join_request: Option<Value>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl Update {
    fn has_modeled_kind(&self) -> bool {
        self.message.is_some()
            || self.edited_message.is_some()
            || self.channel_post.is_some()
            || self.edited_channel_post.is_some()
            || self.callback_query.is_some()
            || self.inline_query.is_some()
            || self.chosen_inline_result.is_some()
            || self.poll.is_some()
            || self.poll_answer.is_some()
            || self.my_chat_member.is_some()
            || self.chat_member.is_some()
            || self.chat_join_request.is_some()
    }

    fn has_unmodeled_kind(&self) -> bool {
        !self.extra.is_empty()
    }

    /// Returns the primary update kind using stable precedence.
    pub fn kind(&self) -> UpdateKind {
        for kind in KNOWN_UPDATE_KINDS {
            if self.has_kind(kind) {
                return kind;
            }
        }

        UpdateKind::Unknown
    }

    /// Returns all detected update kinds.
    pub fn kinds(&self) -> Vec<UpdateKind> {
        let mut kinds = Vec::with_capacity(KNOWN_UPDATE_KINDS.len() + 1);
        for kind in KNOWN_UPDATE_KINDS {
            if self.has_kind(kind) {
                kinds.push(kind);
            }
        }

        if self.has_kind(UpdateKind::Unknown) {
            kinds.push(UpdateKind::Unknown);
        }

        kinds
    }

    /// Returns whether this update contains the given kind.
    pub fn has_kind(&self, kind: UpdateKind) -> bool {
        match kind {
            UpdateKind::Message => self.message.is_some(),
            UpdateKind::EditedMessage => self.edited_message.is_some(),
            UpdateKind::ChannelPost => self.channel_post.is_some(),
            UpdateKind::EditedChannelPost => self.edited_channel_post.is_some(),
            UpdateKind::CallbackQuery => self.callback_query.is_some(),
            UpdateKind::InlineQuery => self.inline_query.is_some(),
            UpdateKind::ChosenInlineResult => self.chosen_inline_result.is_some(),
            UpdateKind::Poll => self.poll.is_some(),
            UpdateKind::PollAnswer => self.poll_answer.is_some(),
            UpdateKind::MyChatMember => self.my_chat_member.is_some(),
            UpdateKind::ChatMember => self.chat_member.is_some(),
            UpdateKind::ChatJoinRequest => self.chat_join_request.is_some(),
            UpdateKind::Unknown => self.has_unmodeled_kind() || !self.has_modeled_kind(),
        }
    }

    /// Returns Mini App payload from the first available message-like field.
    pub fn web_app_data(&self) -> Option<&WebAppData> {
        if let Some(message) = self.message.as_ref() {
            return message.web_app_data();
        }
        if let Some(message) = self.edited_message.as_ref() {
            return message.web_app_data();
        }
        if let Some(message) = self.channel_post.as_ref() {
            return message.web_app_data();
        }
        if let Some(message) = self.edited_channel_post.as_ref() {
            return message.web_app_data();
        }

        self.callback_query
            .as_ref()
            .and_then(|query| query.message.as_ref())
            .and_then(|message| message.web_app_data())
    }
}

/// `getUpdates` request.
#[derive(Clone, Debug, Default, Serialize)]
pub struct GetUpdatesRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowed_updates: Option<Vec<String>>,
}

impl GetUpdatesRequest {
    pub fn with_timeout(timeout_seconds: u16) -> Self {
        Self {
            timeout: Some(timeout_seconds),
            ..Self::default()
        }
    }
}

/// `answerCallbackQuery` request.
#[derive(Clone, Debug, Serialize)]
pub struct AnswerCallbackQueryRequest {
    pub callback_query_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub show_alert: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_time: Option<u32>,
}

/// `answerInlineQuery` request.
#[derive(Clone, Debug, Serialize)]
pub struct AnswerInlineQueryRequest {
    pub inline_query_id: String,
    pub results: Vec<InlineQueryResult>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_time: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_personal: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_offset: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub button: Option<InlineQueryResultsButton>,
}

impl AnswerInlineQueryRequest {
    pub fn new(
        inline_query_id: impl Into<String>,
        results: impl IntoIterator<Item = InlineQueryResult>,
    ) -> Self {
        Self {
            inline_query_id: inline_query_id.into(),
            results: results.into_iter().collect(),
            cache_time: None,
            is_personal: None,
            next_offset: None,
            button: None,
        }
    }

    pub fn add_result(mut self, result: impl Into<InlineQueryResult>) -> Self {
        self.results.push(result.into());
        self
    }

    pub fn cache_time(mut self, cache_time: u32) -> Self {
        self.cache_time = Some(cache_time);
        self
    }

    pub fn is_personal(mut self, is_personal: bool) -> Self {
        self.is_personal = Some(is_personal);
        self
    }

    pub fn next_offset(mut self, next_offset: impl Into<String>) -> Self {
        self.next_offset = Some(next_offset.into());
        self
    }

    pub fn button(mut self, button: InlineQueryResultsButton) -> Self {
        self.button = Some(button);
        self
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error as StdError;

    use serde_json::json;

    use super::*;

    #[test]
    fn detects_primary_update_kind() -> std::result::Result<(), Box<dyn StdError>> {
        let update: Update = serde_json::from_value(json!({
            "update_id": 1,
            "message": {
                "message_id": 10,
                "date": 1700000000,
                "chat": {"id": 1, "type": "private"},
                "text": "hello"
            }
        }))?;

        assert_eq!(update.kind(), UpdateKind::Message);
        assert_eq!(update.kinds(), vec![UpdateKind::Message]);
        assert!(update.has_kind(UpdateKind::Message));
        Ok(())
    }

    #[test]
    fn supports_multi_kind_updates() -> std::result::Result<(), Box<dyn StdError>> {
        let update: Update = serde_json::from_value(json!({
            "update_id": 2,
            "message": {
                "message_id": 11,
                "date": 1700000001,
                "chat": {"id": 1, "type": "private"},
                "text": "hello"
            },
            "callback_query": {
                "id": "cb-1",
                "from": {
                    "id": 1,
                    "is_bot": false,
                    "first_name": "test"
                },
                "chat_instance": "ci-1",
                "data": "payload"
            }
        }))?;

        assert_eq!(
            update.kinds(),
            vec![UpdateKind::Message, UpdateKind::CallbackQuery]
        );
        assert!(update.has_kind(UpdateKind::CallbackQuery));
        Ok(())
    }

    #[test]
    fn returns_unknown_for_unmodeled_payload() -> std::result::Result<(), Box<dyn StdError>> {
        let update: Update = serde_json::from_value(json!({
            "update_id": 3,
            "new_kind_payload": {"foo": "bar"}
        }))?;

        assert_eq!(update.kind(), UpdateKind::Unknown);
        assert_eq!(update.kinds(), vec![UpdateKind::Unknown]);
        assert!(update.has_kind(UpdateKind::Unknown));
        Ok(())
    }

    #[test]
    fn keeps_unknown_alongside_modeled_kind() -> std::result::Result<(), Box<dyn StdError>> {
        let update: Update = serde_json::from_value(json!({
            "update_id": 4,
            "message": {
                "message_id": 12,
                "date": 1700000004,
                "chat": {"id": 1, "type": "private"},
                "text": "hello"
            },
            "new_kind_payload": {"foo": "bar"}
        }))?;

        assert_eq!(update.kind(), UpdateKind::Message);
        assert_eq!(
            update.kinds(),
            vec![UpdateKind::Message, UpdateKind::Unknown]
        );
        assert!(update.has_kind(UpdateKind::Unknown));
        Ok(())
    }

    fn update_for_kind(kind: UpdateKind) -> std::result::Result<Update, Box<dyn StdError>> {
        let payload = match kind {
            UpdateKind::Message => json!({
                "update_id": 100,
                "message": {
                    "message_id": 10,
                    "date": 1700000100,
                    "chat": {"id": 1, "type": "private"},
                    "text": "hello"
                }
            }),
            UpdateKind::EditedMessage => json!({
                "update_id": 101,
                "edited_message": {
                    "message_id": 10,
                    "date": 1700000101,
                    "chat": {"id": 1, "type": "private"},
                    "text": "hello"
                }
            }),
            UpdateKind::ChannelPost => json!({
                "update_id": 102,
                "channel_post": {
                    "message_id": 10,
                    "date": 1700000102,
                    "chat": {"id": -1001, "type": "channel"},
                    "text": "post"
                }
            }),
            UpdateKind::EditedChannelPost => json!({
                "update_id": 103,
                "edited_channel_post": {
                    "message_id": 10,
                    "date": 1700000103,
                    "chat": {"id": -1001, "type": "channel"},
                    "text": "post"
                }
            }),
            UpdateKind::CallbackQuery => json!({
                "update_id": 104,
                "callback_query": {
                    "id": "cb-104",
                    "from": {"id": 1, "is_bot": false, "first_name": "tester"},
                    "chat_instance": "ci",
                    "data": "payload"
                }
            }),
            UpdateKind::InlineQuery => json!({
                "update_id": 105,
                "inline_query": {
                    "id": "iq-105",
                    "from": {"id": 1, "is_bot": false, "first_name": "tester"},
                    "query": "search",
                    "offset": ""
                }
            }),
            UpdateKind::ChosenInlineResult => json!({
                "update_id": 106,
                "chosen_inline_result": {
                    "result_id": "res-106",
                    "from": {"id": 1, "is_bot": false, "first_name": "tester"},
                    "query": "search"
                }
            }),
            UpdateKind::Poll => json!({
                "update_id": 107,
                "poll": {
                    "id": "poll-107",
                    "question": "q?",
                    "options": [{"text": "a", "voter_count": 1}],
                    "total_voter_count": 1,
                    "is_closed": false,
                    "is_anonymous": false,
                    "type": "regular",
                    "allows_multiple_answers": false
                }
            }),
            UpdateKind::PollAnswer => json!({
                "update_id": 108,
                "poll_answer": {
                    "poll_id": "poll-107",
                    "user": {"id": 1, "is_bot": false, "first_name": "tester"},
                    "option_ids": [0]
                }
            }),
            UpdateKind::MyChatMember => json!({
                "update_id": 109,
                "my_chat_member": {"dummy": true}
            }),
            UpdateKind::ChatMember => json!({
                "update_id": 110,
                "chat_member": {"dummy": true}
            }),
            UpdateKind::ChatJoinRequest => json!({
                "update_id": 111,
                "chat_join_request": {"dummy": true}
            }),
            UpdateKind::Unknown => json!({
                "update_id": 112,
                "new_kind_payload": {"foo": "bar"}
            }),
        };

        Ok(serde_json::from_value(payload)?)
    }

    #[test]
    fn update_kind_matrix_stays_in_sync() -> std::result::Result<(), Box<dyn StdError>> {
        for kind in KNOWN_UPDATE_KINDS {
            let update = update_for_kind(kind)?;
            assert!(
                update.has_kind(kind),
                "missing has_kind mapping for {kind:?}"
            );
            assert!(
                update.kinds().contains(&kind),
                "missing kinds mapping for {kind:?}"
            );
        }
        Ok(())
    }

    #[test]
    fn unknown_update_kind_matrix_stays_in_sync() -> std::result::Result<(), Box<dyn StdError>> {
        let update = update_for_kind(UpdateKind::Unknown)?;
        assert_eq!(update.kind(), UpdateKind::Unknown);
        assert!(update.has_kind(UpdateKind::Unknown));
        assert_eq!(update.kinds(), vec![UpdateKind::Unknown]);
        Ok(())
    }
}

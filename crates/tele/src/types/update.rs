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
    /// Returns the primary update kind using stable precedence.
    pub fn kind(&self) -> UpdateKind {
        if self.message.is_some() {
            return UpdateKind::Message;
        }
        if self.edited_message.is_some() {
            return UpdateKind::EditedMessage;
        }
        if self.channel_post.is_some() {
            return UpdateKind::ChannelPost;
        }
        if self.edited_channel_post.is_some() {
            return UpdateKind::EditedChannelPost;
        }
        if self.callback_query.is_some() {
            return UpdateKind::CallbackQuery;
        }
        if self.inline_query.is_some() {
            return UpdateKind::InlineQuery;
        }
        if self.chosen_inline_result.is_some() {
            return UpdateKind::ChosenInlineResult;
        }
        if self.poll.is_some() {
            return UpdateKind::Poll;
        }
        if self.poll_answer.is_some() {
            return UpdateKind::PollAnswer;
        }
        if self.my_chat_member.is_some() {
            return UpdateKind::MyChatMember;
        }
        if self.chat_member.is_some() {
            return UpdateKind::ChatMember;
        }
        if self.chat_join_request.is_some() {
            return UpdateKind::ChatJoinRequest;
        }
        UpdateKind::Unknown
    }

    /// Returns all detected update kinds.
    pub fn kinds(&self) -> Vec<UpdateKind> {
        let mut kinds = Vec::new();
        if self.message.is_some() {
            kinds.push(UpdateKind::Message);
        }
        if self.edited_message.is_some() {
            kinds.push(UpdateKind::EditedMessage);
        }
        if self.channel_post.is_some() {
            kinds.push(UpdateKind::ChannelPost);
        }
        if self.edited_channel_post.is_some() {
            kinds.push(UpdateKind::EditedChannelPost);
        }
        if self.callback_query.is_some() {
            kinds.push(UpdateKind::CallbackQuery);
        }
        if self.inline_query.is_some() {
            kinds.push(UpdateKind::InlineQuery);
        }
        if self.chosen_inline_result.is_some() {
            kinds.push(UpdateKind::ChosenInlineResult);
        }
        if self.poll.is_some() {
            kinds.push(UpdateKind::Poll);
        }
        if self.poll_answer.is_some() {
            kinds.push(UpdateKind::PollAnswer);
        }
        if self.my_chat_member.is_some() {
            kinds.push(UpdateKind::MyChatMember);
        }
        if self.chat_member.is_some() {
            kinds.push(UpdateKind::ChatMember);
        }
        if self.chat_join_request.is_some() {
            kinds.push(UpdateKind::ChatJoinRequest);
        }

        if kinds.is_empty() {
            kinds.push(UpdateKind::Unknown);
        }

        kinds
    }

    /// Returns whether this update contains the given kind.
    pub fn has_kind(&self, kind: UpdateKind) -> bool {
        self.kinds().contains(&kind)
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
}

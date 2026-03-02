use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::bot::User;
use crate::types::message::{Message, Poll};
use crate::types::telegram::{InlineQueryResult, InlineQueryResultsButton};

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

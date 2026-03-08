use super::support::{callback_query_id, reply_chat_id};
use super::*;

mod async_api;
mod blocking_api;

#[cfg(feature = "_async")]
pub use async_api::ErgoApi;
#[cfg(feature = "_blocking")]
pub use blocking_api::BlockingErgoApi;

fn callback_answer_request(
    callback_query_id: impl Into<String>,
    text: Option<String>,
) -> AnswerCallbackQueryRequest {
    AnswerCallbackQueryRequest {
        callback_query_id: callback_query_id.into(),
        text,
        show_alert: None,
        url: None,
        cache_time: None,
    }
}

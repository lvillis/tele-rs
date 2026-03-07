use super::bootstrap::{BootstrapPlan, BootstrapReport, BootstrapRetryPolicy};
#[cfg(feature = "_async")]
use super::bootstrap::{retry_async, retry_fetch_async};
#[cfg(feature = "_blocking")]
use super::bootstrap::{retry_blocking, retry_fetch_blocking};
#[cfg(feature = "bot")]
use super::support::typed_commands_request;
use super::support::{
    callback_query_id, commands_get_request, desired_menu_button, menu_button_get_request,
    parse_web_app_query_payload, reply_chat_id,
};
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

fn web_app_query_request<T>(
    web_app_query_id: impl Into<String>,
    result: T,
) -> Result<AdvancedAnswerWebAppQueryRequest>
where
    T: Serialize,
{
    let result = InlineQueryResult::from_typed(result).map_err(|source| Error::InvalidRequest {
        reason: format!("failed to serialize WebApp inline result: {source}"),
    })?;
    Ok(AdvancedAnswerWebAppQueryRequest::new(
        web_app_query_id,
        result,
    ))
}

fn single_attempt_bootstrap_policy() -> BootstrapRetryPolicy {
    BootstrapRetryPolicy {
        max_attempts: 1,
        continue_on_failure: false,
        ..BootstrapRetryPolicy::default()
    }
}

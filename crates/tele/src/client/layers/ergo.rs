use super::bootstrap::{BootstrapPlan, BootstrapReport, BootstrapRetryPolicy};
#[cfg(feature = "_async")]
use super::bootstrap::{retry_async, retry_fetch_async};
#[cfg(feature = "_blocking")]
use super::bootstrap::{retry_blocking, retry_fetch_blocking};
#[cfg(feature = "bot")]
use super::support::typed_commands_request;
use super::support::{
    callback_query_id, commands_get_request, parse_web_app_query_payload, reply_chat_id,
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

fn get_default_menu_button_request() -> crate::types::advanced::AdvancedGetChatMenuButtonRequest {
    crate::types::advanced::AdvancedGetChatMenuButtonRequest::new()
}

fn get_chat_menu_button_request(
    chat_id: i64,
) -> crate::types::advanced::AdvancedGetChatMenuButtonRequest {
    crate::types::advanced::AdvancedGetChatMenuButtonRequest {
        chat_id: Some(chat_id),
    }
}

fn set_menu_button_request(
    config: &MenuButtonConfig,
) -> crate::types::advanced::AdvancedSetChatMenuButtonRequest {
    config.into()
}

fn commands_in_sync(current: Option<&Vec<BotCommand>>, commands: &SetMyCommandsRequest) -> bool {
    current.is_some_and(|value| value == &commands.commands)
}

fn menu_button_in_sync(current: Option<&MenuButton>, menu_button: &MenuButtonConfig) -> bool {
    current.is_some_and(|value| value == &menu_button.menu_button)
}

fn mark_commands_applied(report: &mut BootstrapReport, applied: bool) {
    report.commands_applied = Some(applied);
    report.commands_synced = Some(applied);
}

fn mark_commands_unchanged(report: &mut BootstrapReport) {
    report.commands_applied = Some(false);
    report.commands_synced = Some(true);
}

fn mark_menu_button_applied(report: &mut BootstrapReport, applied: bool) {
    report.menu_button_applied = Some(applied);
    report.menu_button_synced = Some(applied);
}

fn mark_menu_button_unchanged(report: &mut BootstrapReport) {
    report.menu_button_applied = Some(false);
    report.menu_button_synced = Some(true);
}

fn single_attempt_bootstrap_policy() -> BootstrapRetryPolicy {
    BootstrapRetryPolicy {
        max_attempts: 1,
        continue_on_failure: false,
        ..BootstrapRetryPolicy::default()
    }
}

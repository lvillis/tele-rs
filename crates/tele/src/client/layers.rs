use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::client::RetryConfig;
use crate::types::advanced::{AdvancedAnswerWebAppQueryRequest, AdvancedRequest};
use crate::types::bot::User;
use crate::types::command::{BotCommand, GetMyCommandsRequest, SetMyCommandsRequest};
use crate::types::common::ChatId;
use crate::types::message::{Message, SendMessageRequest, SentWebAppMessage};
use crate::types::telegram::{InlineQueryResult, MenuButton, WebAppData, WebAppInfo};
use crate::types::update::{AnswerCallbackQueryRequest, Update};
use crate::types::upload::UploadFile;
use crate::{Error, Result};

#[cfg(feature = "bot")]
use crate::types::command::BotCommandScope;

#[cfg(feature = "_blocking")]
use crate::BlockingClient;
#[cfg(feature = "_async")]
use crate::Client;

mod app;
mod bootstrap;
mod menu;
mod raw;
mod setup;
mod support;
mod typed;
mod web_app;

#[cfg(feature = "_async")]
pub use app::AppApi;
#[cfg(feature = "_blocking")]
pub use app::BlockingAppApi;
pub use bootstrap::{
    BootstrapFetchStepReport, BootstrapGetMePolicy, BootstrapOutcome, BootstrapPlan,
    BootstrapReport, BootstrapRetryPolicy, BootstrapStepDiagnostics, BootstrapStepPhase,
    BootstrapStepStatus, BootstrapSyncStepReport, WebAppQueryPayload,
};
pub use menu::MenuButtonConfig;
#[cfg(feature = "_blocking")]
pub use raw::BlockingRawApi;
#[cfg(feature = "_async")]
pub use raw::RawApi;
#[cfg(feature = "_blocking")]
pub use setup::BlockingSetupApi;
#[cfg(feature = "_async")]
pub use setup::SetupApi;
#[cfg(feature = "bot")]
pub(crate) use support::reply_chat_id;
#[cfg(feature = "_blocking")]
pub use typed::BlockingTypedApi;
#[cfg(feature = "_async")]
pub use typed::TypedApi;
#[cfg(feature = "_blocking")]
pub use web_app::BlockingWebAppApi;
#[cfg(feature = "_async")]
pub use web_app::WebAppApi;

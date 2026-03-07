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

mod bootstrap;
mod ergo;
mod raw;
mod support;
mod typed;

pub use bootstrap::{
    BootstrapPlan, BootstrapReport, BootstrapRetryPolicy, MenuButtonConfig, WebAppQueryPayload,
};
#[cfg(feature = "_blocking")]
pub use ergo::BlockingErgoApi;
#[cfg(feature = "_async")]
pub use ergo::ErgoApi;
#[cfg(feature = "_blocking")]
pub use raw::BlockingRawApi;
#[cfg(feature = "_async")]
pub use raw::RawApi;
#[cfg(feature = "bot")]
pub(crate) use support::reply_chat_id;
#[cfg(feature = "_blocking")]
pub use typed::BlockingTypedApi;
#[cfg(feature = "_async")]
pub use typed::TypedApi;

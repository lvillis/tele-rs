use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::client::RetryConfig;
use crate::types::advanced::{AdvancedAnswerWebAppQueryRequest, AdvancedRequest};
use crate::types::bot::User;
use crate::types::chat::{
    ChatAdministratorCapability, ChatMember, GetChatAdministratorsRequest, GetChatMemberRequest,
};
use crate::types::command::{
    BotCommand, BotCommandScope, GetMyCommandsRequest, SetMyCommandsRequest,
};
use crate::types::common::{ChatId, MessageId, ParseMode, UserId};
use crate::types::message::{
    Message, SendDocumentRequest, SendMessageRequest, SendPhotoRequest, SendVideoRequest,
    SentWebAppMessage,
};
use crate::types::telegram::{
    InlineQueryResult, LinkPreviewOptions, MenuButton, ReplyMarkup, ReplyParameters, WebAppData,
};
use crate::types::update::{AnswerCallbackQueryRequest, Update};
use crate::types::upload::UploadFile;
use crate::{Error, Result};

#[cfg(feature = "_blocking")]
use crate::BlockingClient;
#[cfg(feature = "_async")]
use crate::Client;

mod app;
mod bootstrap;
mod control;
mod membership;
mod menu;
mod moderation;
mod raw;
mod setup;
mod support;
mod typed;
mod web_app;

#[cfg(feature = "_async")]
pub use app::{AppApi, DocumentSendBuilder, PhotoSendBuilder, TextSendBuilder, VideoSendBuilder};
#[cfg(feature = "_blocking")]
pub use app::{
    BlockingAppApi, BlockingDocumentSendBuilder, BlockingPhotoSendBuilder, BlockingTextSendBuilder,
    BlockingVideoSendBuilder,
};
pub use bootstrap::{
    BootstrapFetchStepReport, BootstrapGetMePolicy, BootstrapOutcome, BootstrapPlan,
    BootstrapReport, BootstrapRetryPolicy, BootstrapStepDiagnostics, BootstrapStepPhase,
    BootstrapStepStatus, BootstrapSyncStepReport, WebAppQueryPayload,
};
#[cfg(feature = "_blocking")]
pub use control::BlockingControlApi;
#[cfg(feature = "_async")]
pub use control::ControlApi;
#[cfg(feature = "_blocking")]
pub use membership::BlockingMembershipApi;
#[cfg(feature = "_async")]
pub use membership::MembershipApi;
pub use menu::MenuButtonConfig;
#[cfg(feature = "_blocking")]
pub use moderation::BlockingModerationApi;
#[cfg(feature = "_blocking")]
pub use moderation::BlockingModerationNoticeApi;
#[cfg(feature = "_async")]
pub use moderation::ModerationApi;
#[cfg(feature = "_async")]
pub use moderation::ModerationNoticeApi;
pub use moderation::{BanMemberOptions, RestrictMemberOptions};
#[cfg(feature = "_blocking")]
pub use raw::BlockingRawApi;
#[cfg(feature = "_async")]
pub use raw::RawApi;
#[cfg(feature = "_blocking")]
pub use setup::BlockingSetupApi;
#[cfg(feature = "_async")]
pub use setup::SetupApi;
#[cfg(feature = "_blocking")]
pub use typed::BlockingTypedApi;
#[cfg(feature = "_async")]
pub use typed::TypedApi;
#[cfg(feature = "_blocking")]
pub use web_app::BlockingWebAppApi;
#[cfg(feature = "_async")]
pub use web_app::WebAppApi;

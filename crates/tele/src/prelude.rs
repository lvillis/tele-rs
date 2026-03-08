//! Common imports for building Telegram bots with `tele`.

#[cfg(feature = "_blocking")]
pub use crate::BlockingClient;
#[cfg(feature = "_async")]
pub use crate::Client;
pub use crate::{ClientBuilder, Error, Result};
pub use crate::{VerifiedWebAppInitData, parse_web_app_init_data, verify_web_app_init_data};

#[cfg(feature = "_async")]
pub use crate::client::{AppApi, RawApi, SetupApi, TypedApi, WebAppApi};
#[cfg(feature = "_blocking")]
pub use crate::client::{BlockingAppApi, BlockingRawApi, BlockingTypedApi};
#[cfg(feature = "_blocking")]
pub use crate::client::{BlockingSetupApi, BlockingWebAppApi};
pub use crate::client::{
    BootstrapFetchStepReport, BootstrapGetMePolicy, BootstrapOutcome, BootstrapPlan,
    BootstrapReport, BootstrapRetryPolicy, BootstrapStepDiagnostics, BootstrapStepPhase,
    BootstrapStepStatus, BootstrapSyncStepReport, ClientMetric, ClientMetricHook, MenuButtonConfig,
    WebAppQueryPayload,
};

pub use crate::types::{
    BotCommand, CallbackCodec, CallbackPayload, ChatAdministratorCapability, ChatId,
    ChatJoinRequest, ChatMemberStatus, ChatMemberUpdated, ChatPermissions, CompactCallbackPayload,
    Message, MessageEntityKind, MessageId, MessageKind, MessageOrigin, ParseMode, PollKind,
    ReplyMarkup, ReplyParameters, Update, UpdateKind,
};

#[cfg(all(feature = "bot", feature = "postgres-session"))]
pub use crate::bot::PostgresSessionStore;
#[cfg(all(feature = "bot", feature = "redis-session"))]
pub use crate::bot::RedisSessionStore;
#[cfg(feature = "bot")]
pub use crate::bot::{
    BotApp, BotContext, BotControl, BotEngine, BotOutbox, CURRENT_ACTOR_CHAT_MEMBER,
    CURRENT_BOT_CHAT_MEMBER, CallbackInput, CallbackRouteBuilder, ChannelUpdateSource,
    ChatJoinRequestInput, ChatMemberUpdatedInput, ChatSession, CommandArgs, CommandRouteBuilder,
    CompactCallbackInput, CompactCallbackRouteBuilder, DispatchMetricOutcome, DispatchOutcome,
    EngineConfig, EngineEvent, EngineMetric, ErrorPolicy, HandlerError, InMemorySessionStore,
    JsonCallback, JsonFileSessionStore, LongPollingSource, MyChatMemberUpdatedInput, OutboxConfig,
    ParsedCommandRouteBuilder, PollingConfig, RequestStateKey, RouteRejection, Router,
    SourceErrorBackoffConfig, TextInput, ThrottleScope, TypedCallbackInput,
    TypedCallbackRouteBuilder, TypedCommandInput, UpdateExt, UpdateExtractor, UpdateSink,
    WebAppInput, WebhookRunner, WriteAccessAllowedInput, channel_source,
};

#[cfg(feature = "macros")]
pub use crate::BotCommands;

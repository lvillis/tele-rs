//! Common imports for building Telegram bots with `tele`.

#[cfg(feature = "_blocking")]
pub use crate::BlockingClient;
#[cfg(feature = "_async")]
pub use crate::Client;
pub use crate::{ClientBuilder, Error, Result};
pub use crate::{VerifiedWebAppInitData, parse_web_app_init_data, verify_web_app_init_data};

#[cfg(feature = "_blocking")]
pub use crate::client::{BlockingErgoApi, BlockingRawApi, BlockingTypedApi};
pub use crate::client::{BootstrapPlan, BootstrapReport, BootstrapRetryPolicy, WebAppQueryPayload};
#[cfg(feature = "_async")]
pub use crate::client::{ErgoApi, RawApi, TypedApi};

pub use crate::types::{
    BotCommand, CallbackPayload, ChatId, ChatMemberPermission, Message, MessageId, MessageKind,
    ParseMode, ReplyMarkup, ReplyParameters, Update, UpdateKind,
};

#[cfg(all(feature = "bot", feature = "postgres-session"))]
pub use crate::bot::PostgresSessionStore;
#[cfg(all(feature = "bot", feature = "redis-session"))]
pub use crate::bot::RedisSessionStore;
#[cfg(feature = "bot")]
pub use crate::bot::{
    BotApp, BotContext, BotEngine, BotOutbox, CallbackInput, ChannelUpdateSource, ChatSession,
    CommandArgs, CommandRouteBuilder, CurrentBotChatMember, CurrentUserChatMember, DispatchOutcome,
    EngineConfig, EngineEvent, ErrorPolicy, HandlerError, InMemorySessionStore, JsonCallback,
    JsonFileSessionStore, LongPollingSource, OutboxConfig, ParsedCommandRouteBuilder,
    PollingConfig, Router, SourceErrorBackoffConfig, TextInput, ThrottleScope, TypedCallbackInput,
    TypedCallbackRouteBuilder, TypedCommandInput, UpdateExt, UpdateExtractor, UpdateSink,
    WebAppInput, WebhookRunner, WriteAccessAllowedInput, channel_source,
};

#[cfg(feature = "macros")]
pub use crate::BotCommands;

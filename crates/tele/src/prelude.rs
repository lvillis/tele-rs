//! Common imports for building Telegram bots with `tele`.

#[cfg(feature = "blocking")]
pub use crate::BlockingClient;
#[cfg(feature = "async")]
pub use crate::Client;
pub use crate::{ClientBuilder, Error, Result};

#[cfg(feature = "blocking")]
pub use crate::client::{BlockingErgoApi, BlockingRawApi, BlockingTypedApi};
#[cfg(feature = "async")]
pub use crate::client::{ErgoApi, RawApi, TypedApi};

pub use crate::types::{
    BotCommand, ChatId, Message, MessageId, ParseMode, ReplyMarkup, ReplyParameters, Update,
};

#[cfg(all(feature = "bot", feature = "postgres-session"))]
pub use crate::bot::PostgresSessionStore;
#[cfg(all(feature = "bot", feature = "redis-session"))]
pub use crate::bot::RedisSessionStore;
#[cfg(feature = "bot")]
pub use crate::bot::{
    BotApp, BotContext, BotEngine, BotOutbox, CallbackInput, ChannelUpdateSource, ChatSession,
    DispatchOutcome, EngineConfig, EngineEvent, ErrorPolicy, HandlerError, InMemorySessionStore,
    JsonCallback, JsonFileSessionStore, LongPollingSource, OutboxConfig, PollingConfig, Router,
    TextInput, TypedCommandInput, UpdateExt, UpdateExtractor, UpdateSink, WebAppInput,
    WebhookRunner, WriteAccessAllowedInput, channel_source,
};

#[cfg(feature = "macros")]
pub use crate::BotCommands;

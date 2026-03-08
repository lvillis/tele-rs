use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::future::Future;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::{Arc, RwLock as StdRwLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::Serialize;
use serde::de::DeserializeOwned;
use tokio::sync::{Mutex, RwLock, Semaphore, mpsc, oneshot};
use tokio::task::JoinSet;
use tokio::time::sleep;

#[cfg(feature = "axum")]
pub mod axum;

use crate::api::{
    AdvancedService, BotService, ChatsService, FilesService, MessagesService, PaymentsService,
    StickersService, UpdatesService,
};
use crate::client::{
    BootstrapPlan, BootstrapReport, BootstrapRetryPolicy, MenuButtonConfig, WebAppQueryPayload,
};
use crate::types::bot::User;
use crate::types::chat::{ChatAdministratorCapability, ChatMember, GetChatMemberRequest};
use crate::types::command::{BotCommand, BotCommandScope, SetMyCommandsRequest};
use crate::types::common::{ChatId, UserId};
use crate::types::message::{
    Chat, Message, MessageKind, SendMessageRequest, SentWebAppMessage, WriteAccessAllowed,
};
use crate::types::telegram::{
    CallbackCodec, CallbackPayload, CallbackPayloadCodec, CompactCallbackCodec,
    CompactCallbackPayload, InlineQueryResult, MenuButton, WebAppData, WebAppInfo,
};
use crate::types::update::{AnswerCallbackQueryRequest, GetUpdatesRequest, Update, UpdateKind};
use crate::types::webhook::{DeleteWebhookRequest, SetWebhookRequest};
use crate::{Client, Error, ErrorClass, Result};

type HandlerFuture = Pin<Box<dyn Future<Output = Result<()>> + Send + 'static>>;
type GuardFuture<'a> = Pin<Box<dyn Future<Output = HandlerResult> + Send + 'a>>;
type SessionFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T>> + Send + 'a>>;
type SourceFuture<'a> = Pin<Box<dyn Future<Output = Result<Vec<Update>>> + Send + 'a>>;

/// Shared async update handler function.
pub type HandlerFn = Arc<dyn Fn(BotContext, Update) -> HandlerFuture + Send + Sync + 'static>;

/// Shared async middleware function.
pub type MiddlewareFn =
    Arc<dyn Fn(BotContext, Update, HandlerFn) -> HandlerFuture + Send + Sync + 'static>;

type GuardFn =
    Arc<dyn for<'a> Fn(&'a BotContext, &'a Update) -> GuardFuture<'a> + Send + Sync + 'static>;

/// Hook called whenever update source polling fails.
pub type SourceErrorHook = Arc<dyn Fn(&Error) + Send + Sync + 'static>;

/// Async hook called whenever update source polling fails.
pub type AsyncSourceErrorHook = Arc<
    dyn Fn(&Error) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>> + Send + Sync + 'static,
>;

/// Hook called when a handler fails. The first parameter is `update_id`.
pub type HandlerErrorHook = Arc<dyn Fn(i64, &Error) + Send + Sync + 'static>;

/// Async hook called when a handler fails. The first parameter is `update_id`.
pub type AsyncHandlerErrorHook = Arc<
    dyn Fn(i64, &Error) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>>
        + Send
        + Sync
        + 'static,
>;

/// Hook called for high-level runtime events.
pub type EngineEventHook = Arc<dyn Fn(&EngineEvent) + Send + Sync + 'static>;

/// Async hook called for high-level runtime events.
pub type AsyncEngineEventHook = Arc<
    dyn Fn(&EngineEvent) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>>
        + Send
        + Sync
        + 'static,
>;

/// Hook called for runtime metrics.
pub type EngineMetricHook = Arc<dyn Fn(&EngineMetric) + Send + Sync + 'static>;

/// Async hook called for runtime metrics.
pub type AsyncEngineMetricHook = Arc<
    dyn Fn(&EngineMetric) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>>
        + Send
        + Sync
        + 'static,
>;

/// Runtime event payload for observability.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum EngineEvent {
    PollStarted,
    PollCompleted {
        update_count: usize,
    },
    PollFailed {
        classification: ErrorClass,
        retryable: bool,
        status: Option<u16>,
        error_code: Option<i64>,
        request_id: Option<String>,
        message: String,
    },
    DispatchStarted {
        update_id: i64,
    },
    UnknownKindsDetected {
        update_id: i64,
        update_kind: UpdateKind,
        message_kind: Option<MessageKind>,
    },
    DispatchCompleted {
        outcome: DispatchOutcome,
    },
    DispatchFailed {
        update_id: i64,
        classification: ErrorClass,
    },
}

/// Final dispatch outcome captured in metrics.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DispatchMetricOutcome {
    Handled,
    Ignored,
    Failed,
}

/// Structured runtime metrics emitted by `BotEngine`.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum EngineMetric {
    PollLatency {
        update_count: usize,
        latency: Duration,
    },
    DispatchLatency {
        update_id: i64,
        outcome: DispatchMetricOutcome,
        latency: Duration,
    },
    SourceError {
        classification: ErrorClass,
        retryable: bool,
        streak: usize,
    },
    SourceBackoff {
        streak: usize,
        delay: Duration,
    },
}

fn invalid_request(reason: impl Into<String>) -> Error {
    Error::InvalidRequest {
        reason: reason.into(),
    }
}

async fn run_blocking_io<T, F>(task: F) -> Result<T>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T> + Send + 'static,
{
    tokio::task::spawn_blocking(task)
        .await
        .map_err(|error| invalid_request(format!("blocking I/O task failed: {error}")))?
}

mod app;
mod context;
mod control;
mod handler_error;
mod outbox;
mod request_state;
mod routing;
mod runtime;
mod session;
pub mod testing;

pub use app::*;
pub use context::*;
pub use control::*;
pub use handler_error::*;
pub use outbox::*;
pub use request_state::*;
pub use routing::*;
pub use runtime::*;
pub use session::*;

fn write_file_atomic(path: &Path, contents: &[u8], subject: &str) -> Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|source| {
        invalid_request(format!(
            "failed to create directory for {subject} `{}`: {source}",
            parent.display()
        ))
    })?;

    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("snapshot");
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0_u128, |duration| duration.as_nanos());
    let process_id = std::process::id();

    for attempt in 0..16 {
        let temp_path = parent.join(format!(".{file_name}.tmp-{process_id}-{nonce}-{attempt}"));
        match fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&temp_path)
        {
            Ok(mut file) => {
                let write_result = (|| -> Result<()> {
                    file.write_all(contents).map_err(|source| {
                        invalid_request(format!(
                            "failed to write temp file for {subject} `{}`: {source}",
                            temp_path.display()
                        ))
                    })?;
                    file.sync_all().map_err(|source| {
                        invalid_request(format!(
                            "failed to sync temp file for {subject} `{}`: {source}",
                            temp_path.display()
                        ))
                    })?;
                    Ok(())
                })();
                if let Err(error) = write_result {
                    let _ = fs::remove_file(&temp_path);
                    return Err(error);
                }

                fs::rename(&temp_path, path).map_err(|source| {
                    let _ = fs::remove_file(&temp_path);
                    invalid_request(format!(
                        "failed to replace {subject} `{}` atomically: {source}",
                        path.display()
                    ))
                })?;
                return Ok(());
            }
            Err(source) if source.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(source) => {
                return Err(invalid_request(format!(
                    "failed to create temp file for {subject} `{}`: {source}",
                    temp_path.display()
                )));
            }
        }
    }

    Err(invalid_request(format!(
        "failed to allocate unique temp file for {subject} `{}`",
        path.display()
    )))
}

fn exponential_backoff(base: Duration, max: Duration, attempt: usize) -> Duration {
    let exponent = attempt.saturating_sub(1).min(16);
    let factor = 2u32.saturating_pow(exponent as u32);
    let delay = base.saturating_mul(factor);
    delay.min(max)
}

fn jitter_duration(delay: Duration, jitter_ratio: f32) -> Duration {
    if delay.is_zero() || jitter_ratio <= 0.0 {
        return delay;
    }

    let ratio = f64::from(jitter_ratio.clamp(0.0, 1.0));
    let now_nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0_u128, |duration| duration.as_nanos());
    let unit = (now_nanos % 10_000) as f64 / 10_000.0;
    let multiplier = (1.0 - ratio) + (2.0 * ratio * unit);
    Duration::from_secs_f64(delay.as_secs_f64() * multiplier)
}

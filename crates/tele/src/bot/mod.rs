use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::future::Future;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::{Arc, Mutex as StdMutex, RwLock as StdRwLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::Serialize;
use serde::de::DeserializeOwned;
use tokio::sync::{Mutex, RwLock, Semaphore, mpsc, oneshot};
use tokio::task::JoinSet;
use tokio::time::sleep;

#[cfg(feature = "axum")]
pub mod axum;
mod webhook_http;
pub use webhook_http::{
    TELEGRAM_SECRET_HEADER, dispatch_webhook, dispatch_webhook_status, telegram_secret_token,
};

use crate::api::{
    AdvancedService, BotService, ChatsService, FilesService, MessagesService, PaymentsService,
    StickersService, UpdatesService,
};
use crate::client::{BootstrapOutcome, BootstrapPlan, BootstrapRetryPolicy, WebAppQueryPayload};
use crate::types::bot::User;
use crate::types::chat::{ChatAdministratorCapability, ChatMember, GetChatMemberRequest};
use crate::types::common::{ChatId, UserId};
use crate::types::message::{Chat, Message, MessageKind, SendMessageRequest, WriteAccessAllowed};
use crate::types::telegram::{
    CallbackCodec, CallbackPayload, CallbackPayloadCodec, CompactCallbackCodec,
    CompactCallbackPayload, WebAppData,
};
use crate::types::update::{AllowedUpdate, GetUpdatesRequest, Update, UpdateKind};
use crate::types::webhook::{DeleteWebhookRequest, SetWebhookRequest, WebhookSecretToken};
use crate::{Client, Error, ErrorClass, Result};

type HandlerFuture = Pin<Box<dyn Future<Output = Result<()>> + Send + 'static>>;
type GuardFuture<'a> = Pin<Box<dyn Future<Output = HandlerResult> + Send + 'a>>;
type SessionFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T>> + Send + 'a>>;
type SourceFuture<'a> = Pin<Box<dyn Future<Output = Result<Vec<Update>>> + Send + 'a>>;
type SourceCommitFuture<'a> = Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>>;

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

fn runtime_error(reason: impl Into<String>) -> Error {
    Error::Runtime {
        reason: reason.into(),
    }
}

fn storage_error(
    operation: impl Into<Box<str>>,
    message: impl Into<Box<str>>,
    retryable: bool,
) -> Error {
    Error::Storage {
        operation: operation.into(),
        message: message.into(),
        retryable,
    }
}

fn storage_encode_error(
    operation: &'static str,
    subject: &str,
    source: serde_json::Error,
) -> Error {
    storage_error(
        operation,
        format!("failed to serialize {subject}: {source}"),
        false,
    )
}

fn storage_decode_error(
    operation: &'static str,
    subject: &str,
    path: &Path,
    source: serde_json::Error,
) -> Error {
    storage_error(
        operation,
        format!(
            "failed to deserialize {subject} `{}`: {source}",
            path.display()
        ),
        false,
    )
}

fn storage_snapshot_error(
    operation: &'static str,
    subject: &str,
    path: &Path,
    source: Error,
) -> Error {
    storage_error(
        operation,
        format!("invalid {subject} `{}`: {source}", path.display()),
        false,
    )
}

async fn run_blocking_io<T, F>(task: F) -> Result<T>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T> + Send + 'static,
{
    tokio::task::spawn_blocking(task)
        .await
        .map_err(|error| storage_error("blocking I/O task", error.to_string(), false))?
}

mod app;
mod context;
mod context_app;
mod handler_error;
mod outbox;
mod request_state;
mod routing;
mod runtime;
mod session;
pub mod testing;

pub use app::*;
pub use context::*;
pub use context_app::*;
pub(crate) use handler_error::normalize_user_message;
pub use handler_error::*;
pub use outbox::*;
pub use request_state::*;
pub use routing::*;
pub use runtime::*;
pub use session::*;

fn storage_parent(path: &Path) -> &Path {
    match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent,
        _ => Path::new("."),
    }
}

fn storage_file_name(path: &Path) -> &str {
    path.file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("snapshot")
}

fn storage_temp_path(path: &Path, attempt: usize) -> PathBuf {
    let parent = storage_parent(path);
    let file_name = storage_file_name(path);
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0_u128, |duration| duration.as_nanos());
    let process_id = std::process::id();
    parent.join(format!(".{file_name}.tmp-{process_id}-{nonce}-{attempt}"))
}

fn validate_file_storage_target(path: &Path, subject: &str) -> Result<()> {
    if path.exists() {
        let metadata = fs::metadata(path).map_err(|source| {
            storage_error(
                format!("{subject} metadata"),
                format!("failed to inspect {subject} `{}`: {source}", path.display()),
                true,
            )
        })?;
        if !metadata.is_file() {
            return Err(storage_error(
                format!("{subject} path validate"),
                format!("{subject} `{}` must be a regular file", path.display()),
                false,
            ));
        }
    }

    let parent = storage_parent(path);
    fs::create_dir_all(parent).map_err(|source| {
        storage_error(
            format!("{subject} directory create"),
            format!(
                "failed to create directory for {subject} `{}`: {source}",
                parent.display()
            ),
            true,
        )
    })?;

    for attempt in 0..16 {
        let temp_path = storage_temp_path(path, attempt);
        match fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&temp_path)
        {
            Ok(file) => {
                if let Err(source) = file.sync_all() {
                    let _ = fs::remove_file(&temp_path);
                    return Err(storage_error(
                        format!("{subject} temp sync"),
                        format!(
                            "failed to sync temp file for {subject} `{}`: {source}",
                            temp_path.display()
                        ),
                        true,
                    ));
                }
                drop(file);
                if let Err(source) = fs::remove_file(&temp_path) {
                    return Err(storage_error(
                        format!("{subject} temp cleanup"),
                        format!(
                            "failed to remove temp file for {subject} `{}`: {source}",
                            temp_path.display()
                        ),
                        true,
                    ));
                }
                sync_parent_directory(parent, subject)?;
                return Ok(());
            }
            Err(source) if source.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(source) => {
                return Err(storage_error(
                    format!("{subject} temp create"),
                    format!(
                        "failed to create temp file for {subject} `{}`: {source}",
                        temp_path.display()
                    ),
                    true,
                ));
            }
        }
    }

    Err(storage_error(
        format!("{subject} temp allocate"),
        format!(
            "failed to allocate unique temp file for {subject} `{}`",
            path.display()
        ),
        true,
    ))
}

async fn validate_file_storage_target_async(path: PathBuf, subject: &'static str) -> Result<()> {
    run_blocking_io(move || validate_file_storage_target(path.as_path(), subject)).await
}

fn write_file_atomic(path: &Path, contents: &[u8], subject: &str) -> Result<()> {
    let parent = storage_parent(path);
    fs::create_dir_all(parent).map_err(|source| {
        storage_error(
            format!("{subject} directory create"),
            format!(
                "failed to create directory for {subject} `{}`: {source}",
                parent.display()
            ),
            true,
        )
    })?;

    for attempt in 0..16 {
        let temp_path = storage_temp_path(path, attempt);
        match fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&temp_path)
        {
            Ok(mut file) => {
                let write_result = (|| -> Result<()> {
                    file.write_all(contents).map_err(|source| {
                        storage_error(
                            format!("{subject} temp write"),
                            format!(
                                "failed to write temp file for {subject} `{}`: {source}",
                                temp_path.display()
                            ),
                            true,
                        )
                    })?;
                    file.sync_all().map_err(|source| {
                        storage_error(
                            format!("{subject} temp sync"),
                            format!(
                                "failed to sync temp file for {subject} `{}`: {source}",
                                temp_path.display()
                            ),
                            true,
                        )
                    })?;
                    Ok(())
                })();
                if let Err(error) = write_result {
                    let _ = fs::remove_file(&temp_path);
                    return Err(error);
                }

                fs::rename(&temp_path, path).map_err(|source| {
                    let _ = fs::remove_file(&temp_path);
                    storage_error(
                        format!("{subject} replace"),
                        format!(
                            "failed to replace {subject} `{}` atomically: {source}",
                            path.display()
                        ),
                        true,
                    )
                })?;
                sync_parent_directory(parent, subject)?;
                return Ok(());
            }
            Err(source) if source.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(source) => {
                return Err(storage_error(
                    format!("{subject} temp create"),
                    format!(
                        "failed to create temp file for {subject} `{}`: {source}",
                        temp_path.display()
                    ),
                    true,
                ));
            }
        }
    }

    Err(storage_error(
        format!("{subject} temp allocate"),
        format!(
            "failed to allocate unique temp file for {subject} `{}`",
            path.display()
        ),
        true,
    ))
}

#[cfg(unix)]
fn sync_parent_directory(parent: &Path, subject: &str) -> Result<()> {
    fs::File::open(parent)
        .and_then(|directory| directory.sync_all())
        .map_err(|source| {
            storage_error(
                format!("{subject} directory sync"),
                format!(
                    "failed to sync directory for {subject} `{}`: {source}",
                    parent.display()
                ),
                true,
            )
        })
}

#[cfg(not(unix))]
fn sync_parent_directory(_parent: &Path, _subject: &str) -> Result<()> {
    Ok(())
}

fn exponential_backoff(base: Duration, max: Duration, attempt: usize) -> Duration {
    let exponent = attempt.saturating_sub(1).min(16);
    let factor = 2u32.saturating_pow(exponent as u32);
    let delay = base.saturating_mul(factor);
    delay.min(max)
}

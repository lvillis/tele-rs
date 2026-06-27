use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::future::Future;
use std::io::{Read, Write};
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

fn authentication_error(reason: impl Into<String>) -> Error {
    Error::Authentication {
        reason: reason.into(),
    }
}

#[cfg(any(feature = "redis-session", feature = "postgres-session"))]
fn configuration_error(reason: impl Into<String>) -> Error {
    Error::Configuration {
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
        .map_err(|error| runtime_error(format!("blocking I/O task failed: {error}")))?
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
    validate_file_storage_path(path, subject)?;
    validate_existing_file_storage_target(path, subject)?;
    validate_storage_parent_writable(path, subject)
}

fn normalize_file_storage_target(path: &Path, subject: &str) -> Result<PathBuf> {
    validate_file_storage_target(path, subject)?;
    canonical_file_storage_path(path, subject)
}

fn read_optional_storage_file(
    path: &Path,
    subject: &str,
    read_operation: &'static str,
) -> Result<Option<Vec<u8>>> {
    validate_file_storage_path(path, subject)?;

    for _ in 0..16 {
        let mut file = match open_current_regular_storage_file(path, subject, read_operation)? {
            StorageFileOpen::File(file) => file,
            StorageFileOpen::Missing => return Ok(None),
            StorageFileOpen::Changed => continue,
        };

        let mut raw = Vec::new();
        return file
            .read_to_end(&mut raw)
            .map(|_| Some(raw))
            .map_err(|source| {
                storage_error(
                    read_operation,
                    format!("failed to read {subject} `{}`: {source}", path.display()),
                    true,
                )
            });
    }

    Err(storage_error(
        read_operation,
        format!(
            "failed to read {subject} `{}`: storage file changed repeatedly",
            path.display()
        ),
        true,
    ))
}

fn canonical_file_storage_path(path: &Path, subject: &str) -> Result<PathBuf> {
    validate_file_storage_path(path, subject)?;
    let parent = storage_parent(path);
    let canonical_parent = parent.canonicalize().map_err(|source| {
        storage_error(
            format!("{subject} path canonicalize"),
            format!(
                "failed to canonicalize directory for {subject} `{}`: {source}",
                parent.display()
            ),
            true,
        )
    })?;
    let Some(file_name) = path.file_name() else {
        return Err(storage_error(
            format!("{subject} path validate"),
            format!("{subject} `{}` must name a regular file", path.display()),
            false,
        ));
    };

    Ok(canonical_parent.join(file_name))
}

fn validate_file_storage_path(path: &Path, subject: &str) -> Result<()> {
    if path.as_os_str().is_empty() {
        return Err(storage_error(
            format!("{subject} path validate"),
            format!("{subject} path must not be empty"),
            false,
        ));
    }

    if has_directory_syntax_suffix(path) {
        return Err(storage_error(
            format!("{subject} path validate"),
            format!("{subject} `{}` must name a regular file", path.display()),
            false,
        ));
    }

    if path.file_name().is_none() {
        return Err(storage_error(
            format!("{subject} path validate"),
            format!("{subject} `{}` must name a regular file", path.display()),
            false,
        ));
    }

    Ok(())
}

fn has_directory_syntax_suffix(path: &Path) -> bool {
    let raw = path.as_os_str().to_string_lossy();

    #[cfg(windows)]
    {
        raw.ends_with('/') || raw.ends_with('\\') || raw.ends_with("/.") || raw.ends_with("\\.")
    }

    #[cfg(not(windows))]
    {
        raw.ends_with('/') || raw.ends_with("/.")
    }
}

fn validate_existing_file_storage_target(path: &Path, subject: &str) -> Result<()> {
    let Some(metadata) = storage_path_metadata(path, subject)? else {
        return Ok(());
    };

    validate_storage_metadata(path, subject, &metadata)
}

fn storage_path_metadata(path: &Path, subject: &str) -> Result<Option<fs::Metadata>> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => Ok(Some(metadata)),
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(source) => Err(storage_error(
            format!("{subject} metadata"),
            format!("failed to inspect {subject} `{}`: {source}", path.display()),
            true,
        )),
    }
}

fn validate_storage_metadata(path: &Path, subject: &str, metadata: &fs::Metadata) -> Result<()> {
    let file_type = metadata.file_type();
    if file_type.is_symlink() {
        return Err(storage_error(
            format!("{subject} path validate"),
            format!("{subject} `{}` must not be a symbolic link", path.display()),
            false,
        ));
    }
    if !file_type.is_file() {
        return Err(storage_error(
            format!("{subject} path validate"),
            format!("{subject} `{}` must be a regular file", path.display()),
            false,
        ));
    }

    Ok(())
}

enum StorageFileOpen {
    File(fs::File),
    Missing,
    Changed,
}

fn open_current_regular_storage_file(
    path: &Path,
    subject: &str,
    read_operation: &'static str,
) -> Result<StorageFileOpen> {
    let Some(initial_path_metadata) = storage_path_metadata(path, subject)? else {
        return Ok(StorageFileOpen::Missing);
    };
    validate_storage_metadata(path, subject, &initial_path_metadata)?;

    let file = match fs::File::open(path) {
        Ok(file) => file,
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => {
            return Ok(StorageFileOpen::Changed);
        }
        Err(source) => {
            return Err(storage_error(
                read_operation,
                format!("failed to open {subject} `{}`: {source}", path.display()),
                true,
            ));
        }
    };

    let file_metadata = file.metadata().map_err(|source| {
        storage_error(
            format!("{subject} metadata"),
            format!(
                "failed to inspect opened {subject} `{}`: {source}",
                path.display()
            ),
            true,
        )
    })?;
    validate_storage_metadata(path, subject, &file_metadata)?;

    let Some(current_path_metadata) = storage_path_metadata(path, subject)? else {
        return Ok(StorageFileOpen::Changed);
    };
    validate_storage_metadata(path, subject, &current_path_metadata)?;

    if !same_file_identity(&current_path_metadata, &file_metadata) {
        return Ok(StorageFileOpen::Changed);
    }

    Ok(StorageFileOpen::File(file))
}

#[cfg(unix)]
fn same_file_identity(left: &fs::Metadata, right: &fs::Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;

    left.dev() == right.dev() && left.ino() == right.ino()
}

#[cfg(windows)]
fn same_file_identity(left: &fs::Metadata, right: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;

    match (
        left.volume_serial_number(),
        left.file_index(),
        right.volume_serial_number(),
        right.file_index(),
    ) {
        (Some(left_volume), Some(left_index), Some(right_volume), Some(right_index)) => {
            left_volume == right_volume && left_index == right_index
        }
        _ => true,
    }
}

#[cfg(not(any(unix, windows)))]
fn same_file_identity(_left: &fs::Metadata, _right: &fs::Metadata) -> bool {
    true
}

fn validate_storage_parent_writable(path: &Path, subject: &str) -> Result<()> {
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

async fn normalize_file_storage_target_async(
    path: PathBuf,
    subject: &'static str,
) -> Result<PathBuf> {
    run_blocking_io(move || normalize_file_storage_target(path.as_path(), subject)).await
}

fn write_file_atomic(path: &Path, contents: &[u8], subject: &str) -> Result<()> {
    validate_file_storage_path(path, subject)?;
    validate_existing_file_storage_target(path, subject)?;

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

#[cfg(test)]
mod storage_tests {
    use super::*;

    type BoxTestResult = std::result::Result<(), Box<dyn std::error::Error>>;

    fn temp_storage_root(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0_u128, |duration| duration.as_nanos());
        std::env::temp_dir().join(format!("tele-{name}-{}-{nanos}", std::process::id()))
    }

    #[test]
    fn file_storage_target_rejects_empty_path() {
        let result = validate_file_storage_target(Path::new(""), "test snapshot");

        assert!(matches!(
            result,
            Err(Error::Storage {
                retryable: false,
                ..
            })
        ));
    }

    #[test]
    fn file_storage_target_rejects_path_without_file_name() {
        let result = validate_file_storage_target(Path::new(".."), "test snapshot");

        assert!(matches!(
            result,
            Err(Error::Storage {
                retryable: false,
                ..
            })
        ));
    }

    #[test]
    fn file_storage_target_rejects_directory_syntax() {
        for path in [Path::new("missing-dir/"), Path::new("missing-dir/.")] {
            let result = validate_file_storage_target(path, "test snapshot");

            assert!(matches!(
                result,
                Err(Error::Storage {
                    retryable: false,
                    ..
                })
            ));
        }
    }

    #[test]
    fn storage_read_returns_none_for_missing_snapshot() {
        let path = temp_storage_root("storage-read-missing").join("snapshot.json");
        let result = read_optional_storage_file(&path, "test snapshot", "test snapshot read");

        assert!(matches!(result, Ok(None)));
    }

    #[test]
    fn storage_read_returns_regular_file_contents() -> BoxTestResult {
        let root = temp_storage_root("storage-read-regular");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root)?;

        let path = root.join("snapshot.json");
        fs::write(&path, b"snapshot")?;

        let result = read_optional_storage_file(&path, "test snapshot", "test snapshot read")?;

        assert_eq!(result.as_deref(), Some(b"snapshot".as_slice()));
        let _ = fs::remove_dir_all(&root);
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn atomic_write_rejects_symbolic_link_without_replacing_it() -> BoxTestResult {
        let root = temp_storage_root("storage-symlink");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root)?;

        let target = root.join("target.json");
        let link = root.join("link.json");
        fs::write(&target, b"old")?;
        std::os::unix::fs::symlink(&target, &link)?;

        let result = write_file_atomic(&link, b"new", "test snapshot");

        assert!(matches!(
            result,
            Err(Error::Storage {
                retryable: false,
                ..
            })
        ));
        assert_eq!(fs::read(&target)?, b"old");
        assert!(fs::symlink_metadata(&link)?.file_type().is_symlink());

        let _ = fs::remove_dir_all(&root);
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn storage_read_rejects_symbolic_link_without_following_it() -> BoxTestResult {
        let root = temp_storage_root("storage-read-symlink");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root)?;

        let target = root.join("target.json");
        let link = root.join("link.json");
        fs::write(&target, b"secret")?;
        std::os::unix::fs::symlink(&target, &link)?;

        let result = read_optional_storage_file(&link, "test snapshot", "test snapshot read");

        assert!(matches!(
            result,
            Err(Error::Storage {
                retryable: false,
                ..
            })
        ));
        let _ = fs::remove_dir_all(&root);
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn storage_read_rejects_dangling_symbolic_link_as_invalid_path() -> BoxTestResult {
        let root = temp_storage_root("storage-read-dangling-symlink");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root)?;

        let link = root.join("link.json");
        std::os::unix::fs::symlink(root.join("missing.json"), &link)?;

        let result = read_optional_storage_file(&link, "test snapshot", "test snapshot read");

        assert!(matches!(
            result,
            Err(Error::Storage {
                retryable: false,
                ..
            })
        ));
        let _ = fs::remove_dir_all(&root);
        Ok(())
    }
}

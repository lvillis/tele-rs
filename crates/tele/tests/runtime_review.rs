#![cfg(feature = "bot")]

use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tele::bot::{
    BotEngine, BotOutbox, EngineConfig, EngineMetric, OutboxConfig, Router,
    SourceErrorBackoffConfig, UpdateSource,
};
use tele::types::Update;
use tele::{Client, Error};

type TestResult = Result<(), Box<dyn std::error::Error>>;

struct RateLimitedSource {
    polls: Arc<AtomicUsize>,
}

impl UpdateSource for RateLimitedSource {
    fn poll<'a>(
        &'a mut self,
    ) -> Pin<Box<dyn Future<Output = tele::Result<Vec<Update>>> + Send + 'a>> {
        Box::pin(async move {
            if self.polls.fetch_add(1, Ordering::SeqCst) > 0 {
                return Err(Error::Runtime {
                    reason: "source polled before the rate limit expired".to_owned(),
                });
            }
            Err(Error::Api {
                method: "getUpdates".to_owned(),
                status: Some(429),
                request_id: None,
                error_code: Some(429),
                description: "Too Many Requests".into(),
                retry_after: Some(Duration::from_secs(120)),
                parameters: None,
                body_snippet: None,
            })
        })
    }
}

async fn assert_source_honors_retry_after(backoff: Option<SourceErrorBackoffConfig>) -> TestResult {
    let client = Client::builder("http://127.0.0.1:9")?
        .bot_token("123:abc")?
        .build()?;
    let polls = Arc::new(AtomicUsize::new(0));
    let delay = Arc::new(Mutex::new(None));
    let stop = Arc::new(tokio::sync::Notify::new());
    let source = RateLimitedSource {
        polls: Arc::clone(&polls),
    };
    let mut engine = BotEngine::new(client, source, Router::new())
        .with_config(EngineConfig {
            error_delay: Duration::from_millis(1),
            source_error_backoff: backoff,
            ..EngineConfig::default()
        })?
        .on_metric({
            let delay = Arc::clone(&delay);
            let stop = Arc::clone(&stop);
            move |metric| {
                if let EngineMetric::SourceBackoff {
                    delay: applied_delay,
                    ..
                } = metric
                {
                    *delay.lock().unwrap_or_else(|error| error.into_inner()) = Some(*applied_delay);
                    stop.notify_one();
                }
            }
        });

    // Shutdown must interrupt the provider's two-minute delay immediately.
    tokio::time::timeout(Duration::from_secs(1), engine.run_until(stop.notified())).await??;

    assert_eq!(
        *delay.lock().map_err(|_| "delay mutex poisoned")?,
        Some(Duration::from_secs(120))
    );
    assert_eq!(polls.load(Ordering::SeqCst), 1);
    Ok(())
}

#[tokio::test]
async fn source_retry_after_overrides_fixed_delay() -> TestResult {
    assert_source_honors_retry_after(None).await
}

#[tokio::test]
async fn source_retry_after_is_not_clamped_or_jittered_by_local_backoff() -> TestResult {
    assert_source_honors_retry_after(Some(SourceErrorBackoffConfig {
        base_delay: Duration::from_millis(1),
        max_delay: Duration::from_millis(1),
        jitter_ratio: 1.0,
    }))
    .await
}

struct TemporaryDirectory(PathBuf);

impl TemporaryDirectory {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let path = std::env::temp_dir().join(format!(
            "tele-outbox-conflicting-paths-{}-{nonce}",
            std::process::id()
        ));
        std::fs::create_dir_all(&path)?;
        Ok(Self(path))
    }
}

impl Drop for TemporaryDirectory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[tokio::test]
async fn outbox_rejects_shared_queue_and_dead_letter_file() -> TestResult {
    let root = TemporaryDirectory::new()?;
    let queue = root.0.join("queue.json");
    let client = Client::builder("http://127.0.0.1:9")?
        .bot_token("123:abc")?
        .build()?;
    let config = OutboxConfig::default()
        .with_persistence_path(&queue)
        .with_dead_letter_path(&queue);

    assert!(matches!(
        BotOutbox::spawn(client, config),
        Err(Error::Configuration { .. })
    ));
    assert!(!queue.exists());
    Ok(())
}

#[tokio::test]
async fn outbox_rejects_normalized_path_aliases_without_changing_snapshot() -> TestResult {
    let root = TemporaryDirectory::new()?;
    let queue = root.0.join("queue.json");
    let nested = root.0.join("nested");
    std::fs::create_dir(&nested)?;
    let original = br#"{"version":1,"queue":[],"entries":[]}"#;
    std::fs::write(&queue, original)?;
    let client = Client::builder("http://127.0.0.1:9")?
        .bot_token("123:abc")?
        .build()?;
    let config = OutboxConfig::default()
        .with_persistence_path(&queue)
        .with_dead_letter_path(nested.join("..").join("queue.json"));

    assert!(matches!(
        BotOutbox::spawn(client, config),
        Err(Error::Configuration { .. })
    ));
    assert_eq!(std::fs::read(&queue)?, original);
    Ok(())
}

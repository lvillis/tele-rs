use super::*;

/// Long-polling source configuration.
#[derive(Clone, Debug)]
pub struct PollingConfig {
    /// Polling timeout passed to `getUpdates` in seconds.
    ///
    /// When greater than zero, runtime requires at least one second of timeout
    /// budget headroom from `min(client.request_timeout, client.total_timeout)`.
    /// If budget is smaller, polling returns `Error::Configuration`.
    /// Set this value to `0` for explicit short polling.
    pub poll_timeout_seconds: u16,
    pub limit: Option<u8>,
    pub allowed_updates: Option<Vec<String>>,
    pub disable_webhook_on_start: bool,
    pub drop_pending_updates_on_start: bool,
    pub dedupe_window_size: usize,
    pub persist_offset_path: Option<PathBuf>,
}

impl Default for PollingConfig {
    fn default() -> Self {
        Self {
            poll_timeout_seconds: 30,
            limit: None,
            allowed_updates: None,
            disable_webhook_on_start: true,
            drop_pending_updates_on_start: false,
            dedupe_window_size: 2048,
            persist_offset_path: None,
        }
    }
}

impl PollingConfig {
    fn resolve_poll_timeout_seconds(
        &self,
        request_timeout: Duration,
        total_timeout: Option<Duration>,
    ) -> Result<u16> {
        let request_budget =
            total_timeout.map_or(request_timeout, |total| total.min(request_timeout));

        // Keep one second of headroom so transport timeout does not preempt long polling.
        let max_poll_timeout = request_budget
            .checked_sub(Duration::from_secs(1))
            .map_or(0, |timeout| {
                timeout.as_secs().min(u64::from(u16::MAX)) as u16
            });

        if self.poll_timeout_seconds > 0 && max_poll_timeout == 0 {
            return Err(Error::Configuration {
                reason: format!(
                    "poll_timeout_seconds={} requires at least 1s timeout budget headroom, got request_timeout={}ms and total_timeout={}ms; increase timeouts or set poll_timeout_seconds=0 for short polling",
                    self.poll_timeout_seconds,
                    request_timeout.as_millis(),
                    total_timeout.map_or(0_u128, |value| value.as_millis())
                ),
            });
        }

        Ok(self.poll_timeout_seconds.min(max_poll_timeout))
    }
}

/// Result of dispatching one update through router + middleware chain.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DispatchOutcome {
    Handled { update_id: i64 },
    Ignored { update_id: i64 },
}

impl DispatchOutcome {
    pub fn update_id(self) -> i64 {
        match self {
            Self::Handled { update_id } | Self::Ignored { update_id } => update_id,
        }
    }

    pub fn is_handled(self) -> bool {
        matches!(self, Self::Handled { .. })
    }
}

/// Pluggable update input source used by `BotEngine`.
pub trait UpdateSource: Send + 'static {
    fn poll<'a>(&'a mut self) -> SourceFuture<'a>;
}

/// Exponential backoff policy for source-side polling errors.
#[derive(Clone, Debug)]
pub struct SourceErrorBackoffConfig {
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub jitter_ratio: f32,
}

impl Default for SourceErrorBackoffConfig {
    fn default() -> Self {
        Self {
            base_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(30),
            jitter_ratio: 0.2,
        }
    }
}

/// Shared engine configuration independent from input source implementation.
#[derive(Clone, Debug)]
pub struct EngineConfig {
    pub idle_delay: Duration,
    pub error_delay: Duration,
    /// Optional exponential backoff for repeated source errors.
    ///
    /// When enabled, this takes precedence over `error_delay`.
    pub source_error_backoff: Option<SourceErrorBackoffConfig>,
    pub continue_on_source_error: bool,
    pub continue_on_handler_error: bool,
    pub max_handler_concurrency: usize,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            idle_delay: Duration::from_millis(100),
            error_delay: Duration::from_millis(500),
            source_error_backoff: None,
            continue_on_source_error: true,
            continue_on_handler_error: true,
            max_handler_concurrency: 1,
        }
    }
}

/// Long-polling update source that only fetches updates and tracks offsets.
#[derive(Clone)]
pub struct LongPollingSource {
    client: Client,
    config: PollingConfig,
    next_offset: Option<i64>,
    seen_update_ids: HashSet<i64>,
    seen_update_order: VecDeque<i64>,
    offset_loaded: bool,
    prepared: bool,
}

impl LongPollingSource {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            config: PollingConfig::default(),
            next_offset: None,
            seen_update_ids: HashSet::new(),
            seen_update_order: VecDeque::new(),
            offset_loaded: false,
            prepared: false,
        }
    }

    pub fn with_config(mut self, config: PollingConfig) -> Self {
        self.config = config;
        self
    }

    /// Sets polling config and validates timeout budget immediately.
    pub fn with_config_checked(mut self, config: PollingConfig) -> Result<Self> {
        self.config = config;
        let _ = self.validate_timeout_budget()?;
        Ok(self)
    }

    pub fn config_mut(&mut self) -> &mut PollingConfig {
        &mut self.config
    }

    /// Validates timeout budget and returns resolved poll timeout seconds.
    pub fn validate_timeout_budget(&self) -> Result<u16> {
        self.effective_poll_timeout_seconds()
    }

    pub fn next_offset(&self) -> Option<i64> {
        self.next_offset
    }

    pub fn set_next_offset(&mut self, offset: Option<i64>) -> &mut Self {
        self.next_offset = offset;
        self
    }

    pub fn with_offset_persistence_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.config.persist_offset_path = Some(path.into());
        self
    }

    pub fn clear_offset_persistence_path(mut self) -> Self {
        self.config.persist_offset_path = None;
        self
    }

    pub fn set_prepared(&mut self, prepared: bool) -> &mut Self {
        self.prepared = prepared;
        self
    }

    async fn ensure_prepared(&mut self) -> Result<()> {
        self.ensure_offset_loaded().await?;

        if self.prepared {
            return Ok(());
        }

        if self.config.disable_webhook_on_start {
            let request = DeleteWebhookRequest {
                drop_pending_updates: self.config.drop_pending_updates_on_start.then_some(true),
            };
            self.client.updates().delete_webhook(&request).await?;
        }

        self.prepared = true;
        Ok(())
    }

    fn advance_next_offset(&mut self, update_id: i64) -> bool {
        let candidate = update_id.saturating_add(1);
        let next = Some(
            self.next_offset
                .map_or(candidate, |current| current.max(candidate)),
        );
        let changed = next != self.next_offset;
        self.next_offset = next;
        changed
    }

    async fn ensure_offset_loaded(&mut self) -> Result<()> {
        if self.offset_loaded {
            return Ok(());
        }

        if self.next_offset.is_none()
            && let Some(path) = self.config.persist_offset_path.as_deref()
        {
            self.next_offset = load_persisted_polling_offset_async(path.to_path_buf()).await?;
        }

        self.offset_loaded = true;
        Ok(())
    }

    async fn persist_offset_if_configured(&self) -> Result<()> {
        let Some(path) = self.config.persist_offset_path.as_deref() else {
            return Ok(());
        };
        persist_polling_offset_async(path.to_path_buf(), self.next_offset).await
    }

    fn is_duplicate_update(&self, update_id: i64) -> bool {
        if self.config.dedupe_window_size == 0 {
            return false;
        }
        self.seen_update_ids.contains(&update_id)
    }

    fn remember_update(&mut self, update_id: i64) {
        if self.config.dedupe_window_size == 0 {
            return;
        }

        if !self.seen_update_ids.insert(update_id) {
            return;
        }

        self.seen_update_order.push_back(update_id);
        while self.seen_update_order.len() > self.config.dedupe_window_size {
            if let Some(oldest) = self.seen_update_order.pop_front() {
                self.seen_update_ids.remove(&oldest);
            }
        }
    }

    fn effective_poll_timeout_seconds(&self) -> Result<u16> {
        self.config.resolve_poll_timeout_seconds(
            self.client.request_timeout(),
            self.client.total_timeout(),
        )
    }
}

impl UpdateSource for LongPollingSource {
    fn poll<'a>(&'a mut self) -> SourceFuture<'a> {
        Box::pin(async move {
            self.ensure_prepared().await?;

            let mut request =
                GetUpdatesRequest::with_timeout(self.effective_poll_timeout_seconds()?);
            request.offset = self.next_offset;
            request.limit = self.config.limit;
            request.allowed_updates = self.config.allowed_updates.clone();

            let updates = self.client.updates().get_updates(&request).await?;
            let mut offset_changed = false;
            for update in &updates {
                offset_changed |= self.advance_next_offset(update.update_id);
            }
            if offset_changed {
                self.persist_offset_if_configured().await?;
            }

            let mut deduped = Vec::with_capacity(updates.len());
            for update in updates {
                if self.is_duplicate_update(update.update_id) {
                    continue;
                }
                self.remember_update(update.update_id);
                deduped.push(update);
            }

            Ok(deduped)
        })
    }
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
struct PollingOffsetSnapshot {
    #[serde(default = "default_polling_offset_snapshot_version")]
    version: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    next_offset: Option<i64>,
}

fn default_polling_offset_snapshot_version() -> u8 {
    1
}

fn load_persisted_polling_offset(path: &Path) -> Result<Option<i64>> {
    if !path.exists() {
        return Ok(None);
    }

    let raw = fs::read(path).map_err(|source| Error::ReadLocalFile {
        path: path.display().to_string(),
        source,
    })?;
    if raw.is_empty() {
        return Ok(None);
    }

    let snapshot: PollingOffsetSnapshot = serde_json::from_slice(&raw).map_err(|source| {
        invalid_request(format!(
            "failed to deserialize polling offset snapshot `{}`: {source}",
            path.display()
        ))
    })?;
    Ok(snapshot.next_offset)
}

fn persist_polling_offset(path: &Path, next_offset: Option<i64>) -> Result<()> {
    let snapshot = PollingOffsetSnapshot {
        version: default_polling_offset_snapshot_version(),
        next_offset,
    };
    let encoded =
        serde_json::to_vec(&snapshot).map_err(|source| Error::SerializeRequest { source })?;
    write_file_atomic(path, encoded.as_slice(), "polling offset snapshot")?;
    Ok(())
}

async fn load_persisted_polling_offset_async(path: PathBuf) -> Result<Option<i64>> {
    run_blocking_io(move || load_persisted_polling_offset(path.as_path())).await
}

async fn persist_polling_offset_async(path: PathBuf, next_offset: Option<i64>) -> Result<()> {
    run_blocking_io(move || persist_polling_offset(path.as_path(), next_offset)).await
}

/// Sink side of a channel-backed update source.
#[derive(Clone)]
pub struct UpdateSink {
    sender: mpsc::Sender<Update>,
}

impl UpdateSink {
    pub fn new(sender: mpsc::Sender<Update>) -> Self {
        Self { sender }
    }

    pub async fn send(&self, update: Update) -> Result<()> {
        self.sender
            .send(update)
            .await
            .map_err(|_| invalid_request("update sink channel is closed"))?;
        Ok(())
    }
}

/// Source side of a channel-backed update source.
pub struct ChannelUpdateSource {
    receiver: mpsc::Receiver<Update>,
    max_batch: usize,
}

impl ChannelUpdateSource {
    pub fn new(receiver: mpsc::Receiver<Update>) -> Self {
        Self {
            receiver,
            max_batch: 32,
        }
    }

    pub fn with_max_batch(mut self, max_batch: usize) -> Self {
        self.max_batch = max_batch.max(1);
        self
    }
}

impl UpdateSource for ChannelUpdateSource {
    fn poll<'a>(&'a mut self) -> SourceFuture<'a> {
        Box::pin(async move {
            let Some(first) = self.receiver.recv().await else {
                return Err(invalid_request("update source channel is closed"));
            };

            let mut updates = Vec::with_capacity(self.max_batch.max(1));
            updates.push(first);

            while updates.len() < self.max_batch {
                match self.receiver.try_recv() {
                    Ok(update) => updates.push(update),
                    Err(mpsc::error::TryRecvError::Empty) => break,
                    Err(mpsc::error::TryRecvError::Disconnected) => break,
                }
            }

            Ok(updates)
        })
    }
}

/// Creates a webhook-friendly channel source pair.
pub fn channel_source(buffer: usize) -> (UpdateSink, ChannelUpdateSource) {
    let (sender, receiver) = mpsc::channel(buffer.max(1));
    (UpdateSink::new(sender), ChannelUpdateSource::new(receiver))
}

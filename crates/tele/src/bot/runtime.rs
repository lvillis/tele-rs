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
    pub allowed_updates: Option<Vec<AllowedUpdate>>,
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
    pub fn allowed_updates(
        mut self,
        allowed_updates: impl IntoIterator<Item = AllowedUpdate>,
    ) -> Self {
        self.set_allowed_updates(allowed_updates);
        self
    }

    pub fn allowed_update_kinds(
        mut self,
        kinds: impl IntoIterator<Item = UpdateKind>,
    ) -> Result<Self> {
        self.set_allowed_update_kinds(kinds)?;
        Ok(self)
    }

    pub fn set_allowed_updates(
        &mut self,
        allowed_updates: impl IntoIterator<Item = AllowedUpdate>,
    ) -> &mut Self {
        self.allowed_updates = Some(allowed_updates.into_iter().collect());
        self
    }

    pub fn set_allowed_update_kinds(
        &mut self,
        kinds: impl IntoIterator<Item = UpdateKind>,
    ) -> Result<&mut Self> {
        self.allowed_updates = Some(AllowedUpdate::from_kinds(kinds)?);
        Ok(self)
    }

    pub fn clear_allowed_updates(&mut self) -> &mut Self {
        self.allowed_updates = None;
        self
    }

    pub fn validate(&self) -> Result<()> {
        let request = GetUpdatesRequest {
            limit: self.limit,
            allowed_updates: self.allowed_updates.clone(),
            ..GetUpdatesRequest::default()
        };

        request.validate().map_err(|error| match error {
            Error::InvalidRequest { reason } => Error::Configuration {
                reason: format!("invalid polling config: {reason}"),
            },
            error => error,
        })?;

        Ok(())
    }

    fn resolve_poll_timeout_seconds(
        &self,
        request_timeout: Duration,
        total_timeout: Option<Duration>,
    ) -> Result<u16> {
        self.validate()?;

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
    Failed { update_id: i64 },
}

impl DispatchOutcome {
    pub fn update_id(self) -> i64 {
        match self {
            Self::Handled { update_id }
            | Self::Ignored { update_id }
            | Self::Failed { update_id } => update_id,
        }
    }

    pub fn is_handled(self) -> bool {
        matches!(self, Self::Handled { .. })
    }

    pub fn is_failed(self) -> bool {
        matches!(self, Self::Failed { .. })
    }
}

/// Pluggable update input source used by `BotEngine`.
pub trait UpdateSource: Send + 'static {
    fn poll<'a>(&'a mut self) -> SourceFuture<'a>;

    fn commit<'a>(&'a mut self, _outcomes: &'a [DispatchOutcome]) -> SourceCommitFuture<'a> {
        Box::pin(async { Ok(()) })
    }
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

impl SourceErrorBackoffConfig {
    pub fn validate(&self) -> Result<()> {
        if self.base_delay.is_zero() {
            return Err(Error::Configuration {
                reason: "source_error_backoff base_delay must be greater than zero".to_owned(),
            });
        }
        if self.max_delay.is_zero() {
            return Err(Error::Configuration {
                reason: "source_error_backoff max_delay must be greater than zero".to_owned(),
            });
        }
        if self.base_delay > self.max_delay {
            return Err(Error::Configuration {
                reason: "source_error_backoff base_delay must not exceed max_delay".to_owned(),
            });
        }
        if !self.jitter_ratio.is_finite() || !(0.0..=1.0).contains(&self.jitter_ratio) {
            return Err(Error::Configuration {
                reason: "source_error_backoff jitter_ratio must be finite and between 0.0 and 1.0"
                    .to_owned(),
            });
        }

        Ok(())
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
    /// Maximum number of update handlers to run concurrently.
    ///
    /// This is applied only when `continue_on_handler_error` is true. Fail-fast dispatch stays
    /// source-ordered so later updates cannot perform side effects before an earlier failed update
    /// blocks offset commits and forces Telegram redelivery.
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

impl EngineConfig {
    pub fn validate(&self) -> Result<()> {
        if self.max_handler_concurrency == 0 {
            return Err(Error::Configuration {
                reason: "max_handler_concurrency must be at least 1".to_owned(),
            });
        }
        if self.continue_on_source_error
            && self.source_error_backoff.is_none()
            && self.error_delay.is_zero()
        {
            return Err(Error::Configuration {
                reason: "error_delay must be greater than zero when source errors are retried without backoff".to_owned(),
            });
        }
        if let Some(backoff) = self.source_error_backoff.as_ref() {
            backoff.validate()?;
        }

        Ok(())
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
        config.validate()?;
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

    /// Overrides the next polling offset and makes the override authoritative.
    ///
    /// This also clears the in-memory dedupe window so callers can intentionally rewind or clear
    /// offsets without stale local state suppressing redelivered updates.
    pub fn set_next_offset(&mut self, offset: Option<i64>) -> &mut Self {
        self.next_offset = offset;
        self.offset_loaded = true;
        self.seen_update_ids.clear();
        self.seen_update_order.clear();
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

    fn next_offset_after_update_ids<I>(&self, update_ids: I) -> Option<i64>
    where
        I: IntoIterator<Item = i64>,
    {
        update_ids
            .into_iter()
            .fold(self.next_offset, |next, update_id| {
                let candidate = update_id.saturating_add(1);
                Some(next.map_or(candidate, |current| current.max(candidate)))
            })
    }

    fn apply_committed_update(&mut self, update_id: i64) -> bool {
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

    async fn commit_update_ids(&mut self, update_ids: &[i64]) -> Result<()> {
        if update_ids.is_empty() {
            return Ok(());
        }

        let next_offset = self.next_offset_after_update_ids(update_ids.iter().copied());
        if next_offset != self.next_offset
            && let Some(path) = self.config.persist_offset_path.as_deref()
        {
            persist_polling_offset_async(path.to_path_buf(), next_offset).await?;
        }

        for update_id in update_ids {
            let _ = self.apply_committed_update(*update_id);
            self.remember_update(*update_id);
        }

        Ok(())
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
            self.config.validate()?;
            self.ensure_prepared().await?;

            let mut request =
                GetUpdatesRequest::with_timeout(self.effective_poll_timeout_seconds()?);
            request.offset = self.next_offset;
            request.limit = self.config.limit;
            request.allowed_updates = self.config.allowed_updates.clone();

            let updates = self.client.updates().get_updates(&request).await?;

            let mut deduped = Vec::with_capacity(updates.len());
            let mut batch_seen = HashSet::new();
            for update in updates {
                if self.is_duplicate_update(update.update_id)
                    || !batch_seen.insert(update.update_id)
                {
                    continue;
                }
                deduped.push(update);
            }

            Ok(deduped)
        })
    }

    fn commit<'a>(&'a mut self, outcomes: &'a [DispatchOutcome]) -> SourceCommitFuture<'a> {
        Box::pin(async move {
            let update_ids = outcomes
                .iter()
                .take_while(|outcome| !outcome.is_failed())
                .map(|outcome| outcome.update_id())
                .collect::<Vec<_>>();
            self.commit_update_ids(&update_ids).await
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

    let raw = fs::read(path).map_err(|source| {
        storage_error(
            "polling offset read",
            format!(
                "failed to read polling offset snapshot `{}`: {source}",
                path.display()
            ),
            true,
        )
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
    validate_polling_offset_snapshot(&snapshot)?;
    Ok(snapshot.next_offset)
}

fn validate_polling_offset_snapshot(snapshot: &PollingOffsetSnapshot) -> Result<()> {
    if snapshot.version != default_polling_offset_snapshot_version() {
        return Err(invalid_request(format!(
            "unsupported polling offset snapshot version `{}`",
            snapshot.version
        )));
    }
    if snapshot.next_offset.is_some_and(|offset| offset < 0) {
        return Err(invalid_request(
            "polling offset snapshot next_offset must not be negative",
        ));
    }

    Ok(())
}

fn persist_polling_offset(path: &Path, next_offset: Option<i64>) -> Result<()> {
    let snapshot = PollingOffsetSnapshot {
        version: default_polling_offset_snapshot_version(),
        next_offset,
    };
    validate_polling_offset_snapshot(&snapshot)?;
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

    pub fn with_max_batch(mut self, max_batch: usize) -> Result<Self> {
        if max_batch == 0 {
            return Err(Error::Configuration {
                reason: "channel update source max_batch must be at least 1".to_owned(),
            });
        }
        self.max_batch = max_batch;
        Ok(self)
    }
}

impl UpdateSource for ChannelUpdateSource {
    fn poll<'a>(&'a mut self) -> SourceFuture<'a> {
        Box::pin(async move {
            let Some(first) = self.receiver.recv().await else {
                return Err(invalid_request("update source channel is closed"));
            };

            let mut updates = Vec::with_capacity(self.max_batch);
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
pub fn channel_source(buffer: usize) -> Result<(UpdateSink, ChannelUpdateSource)> {
    if buffer == 0 {
        return Err(Error::Configuration {
            reason: "channel source buffer must be at least 1".to_owned(),
        });
    }
    let (sender, receiver) = mpsc::channel(buffer);
    Ok((UpdateSink::new(sender), ChannelUpdateSource::new(receiver)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_polling_offset_snapshot_metadata() {
        let mut snapshot = PollingOffsetSnapshot {
            version: default_polling_offset_snapshot_version(),
            next_offset: Some(1),
        };
        assert!(validate_polling_offset_snapshot(&snapshot).is_ok());

        snapshot.version = snapshot.version.saturating_add(1);
        assert!(matches!(
            validate_polling_offset_snapshot(&snapshot),
            Err(Error::InvalidRequest { .. })
        ));

        snapshot.version = default_polling_offset_snapshot_version();
        snapshot.next_offset = Some(-1);
        assert!(matches!(
            validate_polling_offset_snapshot(&snapshot),
            Err(Error::InvalidRequest { .. })
        ));
    }

    #[tokio::test]
    async fn explicit_offset_override_skips_persisted_offset_load() -> Result<()> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0_u128, |duration| duration.as_nanos());
        let offset_path = std::env::temp_dir().join(format!(
            "tele-offset-explicit-override-{}-{timestamp}.json",
            std::process::id()
        ));
        let _ = fs::remove_file(&offset_path);
        persist_polling_offset(&offset_path, Some(42))?;

        let client = Client::builder("http://127.0.0.1:9")?
            .bot_token("123:abc")?
            .build()?;
        let mut source = LongPollingSource::new(client).with_offset_persistence_path(&offset_path);

        source.set_next_offset(None);
        source.ensure_offset_loaded().await?;

        assert_eq!(source.next_offset(), None);

        let _ = fs::remove_file(&offset_path);
        Ok(())
    }

    #[tokio::test]
    async fn explicit_offset_override_clears_dedupe_window() -> Result<()> {
        let client = Client::builder("http://127.0.0.1:9")?
            .bot_token("123:abc")?
            .build()?;
        let mut source = LongPollingSource::new(client);

        source.commit_update_ids(&[10]).await?;
        assert_eq!(source.next_offset(), Some(11));
        assert!(source.is_duplicate_update(10));

        source.set_next_offset(Some(10));

        assert_eq!(source.next_offset(), Some(10));
        assert!(!source.is_duplicate_update(10));
        assert!(source.offset_loaded);

        Ok(())
    }
}

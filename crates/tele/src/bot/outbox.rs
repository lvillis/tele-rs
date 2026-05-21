use super::*;
use crate::util::retry_after_or_backoff;

const MAX_OUTBOX_IDEMPOTENCY_KEY_BYTES: usize = 256;

/// Reliable send-side outbox configuration.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct OutboxConfig {
    pub queue_capacity: usize,
    pub max_attempts: usize,
    pub base_backoff: Duration,
    /// Maximum locally computed exponential backoff.
    ///
    /// Provider supplied `Retry-After` values are honored separately and are not clamped by this.
    pub max_backoff: Duration,
    pub dedupe_ttl: Duration,
    pub persistence_path: Option<PathBuf>,
    pub dead_letter_path: Option<PathBuf>,
    pub max_dead_letters: usize,
    pub max_message_age: Option<Duration>,
}

impl Default for OutboxConfig {
    fn default() -> Self {
        Self {
            queue_capacity: 256,
            max_attempts: 4,
            base_backoff: Duration::from_millis(200),
            max_backoff: Duration::from_secs(5),
            dedupe_ttl: Duration::from_secs(120),
            persistence_path: None,
            dead_letter_path: None,
            max_dead_letters: 1024,
            max_message_age: None,
        }
    }
}

impl OutboxConfig {
    pub fn validate(&self) -> Result<()> {
        if self.queue_capacity == 0 {
            return Err(invalid_request(
                "outbox queue_capacity must be greater than 0",
            ));
        }
        if self.max_attempts == 0 {
            return Err(invalid_request(
                "outbox max_attempts must be greater than 0",
            ));
        }
        if self.base_backoff.is_zero() {
            return Err(invalid_request(
                "outbox base_backoff must be greater than 0",
            ));
        }
        if self.max_backoff.is_zero() {
            return Err(invalid_request("outbox max_backoff must be greater than 0"));
        }
        if self.base_backoff > self.max_backoff {
            return Err(invalid_request(
                "outbox base_backoff must not exceed max_backoff",
            ));
        }
        if self.dedupe_ttl.is_zero() {
            return Err(invalid_request("outbox dedupe_ttl must be greater than 0"));
        }
        if self.max_dead_letters == 0 {
            return Err(invalid_request(
                "outbox max_dead_letters must be greater than 0",
            ));
        }
        if self
            .max_message_age
            .is_some_and(|max_message_age| max_message_age.is_zero())
        {
            return Err(invalid_request(
                "outbox max_message_age must be greater than 0 when set",
            ));
        }

        Ok(())
    }

    pub fn with_persistence_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.persistence_path = Some(path.into());
        self
    }

    pub fn without_persistence(mut self) -> Self {
        self.persistence_path = None;
        self
    }

    pub fn with_dead_letter_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.dead_letter_path = Some(path.into());
        self
    }

    pub fn with_max_message_age(mut self, max_age: Option<Duration>) -> Self {
        self.max_message_age = max_age;
        self
    }
}

struct OutboxCommand {
    chat_id: ChatId,
    text: String,
    idempotency_key: Option<String>,
    responder: oneshot::Sender<Result<Message>>,
    _permit: tokio::sync::OwnedSemaphorePermit,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct PersistedOutboxCommand {
    chat_id: ChatId,
    text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    idempotency_key: Option<String>,
    #[serde(default = "unix_timestamp_millis_now")]
    enqueued_at_unix_ms: i64,
    #[serde(default)]
    attempt: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    last_error: Option<String>,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
struct OutboxSnapshot {
    #[serde(default = "default_outbox_snapshot_version")]
    version: u8,
    #[serde(default)]
    queue: Vec<PersistedOutboxCommand>,
}

fn default_outbox_snapshot_version() -> u8 {
    1
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct DeadLetterEntry {
    chat_id: ChatId,
    text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    idempotency_key: Option<String>,
    attempts: usize,
    reason: String,
    enqueued_at_unix_ms: i64,
    failed_at_unix_ms: i64,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
struct DeadLetterSnapshot {
    #[serde(default = "default_dead_letter_snapshot_version")]
    version: u8,
    #[serde(default)]
    entries: Vec<DeadLetterEntry>,
}

fn default_dead_letter_snapshot_version() -> u8 {
    1
}

struct QueuedOutboxCommand {
    payload: PersistedOutboxCommand,
    responder: Option<oneshot::Sender<Result<Message>>>,
    _permit: OutboxQueuePermit,
}

enum OutboxQueuePermit {
    Live(tokio::sync::OwnedSemaphorePermit),
    Persisted(Arc<Semaphore>),
}

impl Drop for OutboxQueuePermit {
    fn drop(&mut self) {
        match self {
            Self::Live(_permit) => {}
            Self::Persisted(semaphore) => semaphore.add_permits(1),
        }
    }
}

/// Asynchronous outbox handle for reliable message delivery.
#[derive(Clone)]
pub struct BotOutbox {
    sender: mpsc::Sender<OutboxCommand>,
    permits: Arc<Semaphore>,
}

impl BotOutbox {
    pub fn spawn(client: Client, config: OutboxConfig) -> Result<Self> {
        config.validate()?;
        validate_dead_letter_path(config.dead_letter_path.as_deref())?;
        let persisted_queue = load_outbox_queue(&config)?;
        let queue_capacity = config.queue_capacity;
        let available_permits = queue_capacity.saturating_sub(persisted_queue.len());
        let permits = Arc::new(Semaphore::new(available_permits));
        let (sender, receiver) = mpsc::channel(queue_capacity);
        tokio::spawn(run_outbox_worker(
            client,
            config,
            persisted_queue,
            receiver,
            Arc::clone(&permits),
        ));
        Ok(Self { sender, permits })
    }

    pub async fn send_text(
        &self,
        chat_id: impl Into<ChatId>,
        text: impl Into<String>,
    ) -> Result<Message> {
        self.send_text_with_key(chat_id, text, None).await
    }

    pub async fn send_text_with_key(
        &self,
        chat_id: impl Into<ChatId>,
        text: impl Into<String>,
        idempotency_key: Option<String>,
    ) -> Result<Message> {
        validate_idempotency_key(idempotency_key.as_deref())?;
        let request = SendMessageRequest::new(chat_id, text)?;
        let permit = Arc::clone(&self.permits)
            .acquire_owned()
            .await
            .map_err(|_| invalid_request("outbox worker is closed"))?;
        let (responder, receiver) = oneshot::channel();
        let command = OutboxCommand {
            chat_id: request.chat_id,
            text: request.text,
            idempotency_key,
            responder,
            _permit: permit,
        };

        self.sender
            .send(command)
            .await
            .map_err(|_| invalid_request("outbox worker is closed"))?;

        receiver
            .await
            .map_err(|_| invalid_request("outbox worker dropped response"))?
    }
}

fn validate_idempotency_key(key: Option<&str>) -> Result<()> {
    let Some(key) = key else {
        return Ok(());
    };

    if key.trim().is_empty() {
        return Err(invalid_request("outbox idempotency key cannot be empty"));
    }
    if key.len() > MAX_OUTBOX_IDEMPOTENCY_KEY_BYTES {
        return Err(invalid_request(format!(
            "outbox idempotency key exceeds {MAX_OUTBOX_IDEMPOTENCY_KEY_BYTES} bytes"
        )));
    }

    Ok(())
}

async fn run_outbox_worker(
    client: Client,
    config: OutboxConfig,
    persisted_queue: Vec<PersistedOutboxCommand>,
    mut receiver: mpsc::Receiver<OutboxCommand>,
    permits: Arc<Semaphore>,
) {
    let mut dedupe: HashMap<String, (Message, Instant)> = HashMap::new();
    let mut queue = persisted_queue
        .into_iter()
        .map(|payload| QueuedOutboxCommand {
            payload,
            responder: None,
            _permit: OutboxQueuePermit::Persisted(Arc::clone(&permits)),
        })
        .collect::<VecDeque<_>>();

    loop {
        while let Ok(command) = receiver.try_recv() {
            queue.push_back(QueuedOutboxCommand {
                payload: PersistedOutboxCommand {
                    chat_id: command.chat_id,
                    text: command.text,
                    idempotency_key: command.idempotency_key,
                    enqueued_at_unix_ms: unix_timestamp_millis_now(),
                    attempt: 0,
                    last_error: None,
                },
                responder: Some(command.responder),
                _permit: OutboxQueuePermit::Live(command._permit),
            });
        }

        if queue.is_empty() {
            let Some(command) = receiver.recv().await else {
                break;
            };

            queue.push_back(QueuedOutboxCommand {
                payload: PersistedOutboxCommand {
                    chat_id: command.chat_id,
                    text: command.text,
                    idempotency_key: command.idempotency_key,
                    enqueued_at_unix_ms: unix_timestamp_millis_now(),
                    attempt: 0,
                    last_error: None,
                },
                responder: Some(command.responder),
                _permit: OutboxQueuePermit::Live(command._permit),
            });
        }

        if let Err(_error) =
            persist_outbox_queue_async(config.persistence_path.clone(), &queue).await
        {
            sleep(outbox_persistence_retry_delay(&config)).await;
            continue;
        }

        let Some(front_payload) = queue.front().map(|entry| entry.payload.clone()) else {
            continue;
        };

        if is_outbox_message_expired(
            front_payload.enqueued_at_unix_ms,
            config.max_message_age,
            unix_timestamp_millis_now(),
        ) {
            let entry = match dead_letter_front_and_commit(
                &config,
                &mut queue,
                "message expired in outbox before delivery".to_owned(),
            )
            .await
            {
                Ok(Some(entry)) => entry,
                Ok(None) => continue,
                Err(_error) => {
                    sleep(outbox_persistence_retry_delay(&config)).await;
                    continue;
                }
            };
            if let Some(responder) = entry.responder {
                let _ = responder.send(Err(invalid_request("message expired in outbox queue")));
            }
            continue;
        }

        prune_dedupe_cache(&mut dedupe);

        if let Some(key) = front_payload.idempotency_key.as_deref()
            && let Some((cached, expires_at)) = dedupe.get(key)
            && *expires_at > Instant::now()
        {
            let entry = match commit_outbox_front(&config, &mut queue).await {
                Ok(Some(entry)) => entry,
                Ok(None) => continue,
                Err(_error) => {
                    sleep(outbox_persistence_retry_delay(&config)).await;
                    continue;
                }
            };
            if let Some(responder) = entry.responder {
                let _ = responder.send(Ok(cached.clone()));
            }
            continue;
        }

        let send_result = send_once(&client, &front_payload.chat_id, &front_payload.text).await;
        match send_result {
            Ok(message) => {
                if let Some(key) = front_payload.idempotency_key.clone() {
                    let expires_at = Instant::now() + config.dedupe_ttl;
                    dedupe.insert(key, (message.clone(), expires_at));
                }

                let entry = match commit_outbox_front(&config, &mut queue).await {
                    Ok(Some(entry)) => entry,
                    Ok(None) => continue,
                    Err(_error) => {
                        // Message may already be delivered upstream; stop worker to avoid local duplicate sends.
                        return;
                    }
                };
                if let Some(responder) = entry.responder {
                    let _ = responder.send(Ok(message));
                }
            }
            Err(error) => {
                let max_attempts = config.max_attempts;
                let error_message = error.to_string();
                let attempt = if let Some(front) = queue.front_mut() {
                    front.payload.attempt = front.payload.attempt.saturating_add(1);
                    front.payload.last_error = Some(error_message.clone());
                    front.payload.attempt
                } else {
                    1
                };
                let should_retry = error.is_retryable() && attempt < max_attempts;
                if should_retry {
                    let delay = retry_after_or_backoff(&error, || {
                        exponential_backoff(config.base_backoff, config.max_backoff, attempt)
                    });
                    if let Err(_error) =
                        persist_outbox_queue_async(config.persistence_path.clone(), &queue).await
                    {
                        sleep(outbox_persistence_retry_delay(&config)).await;
                        continue;
                    }
                    sleep(delay).await;
                    continue;
                }

                let entry =
                    match dead_letter_front_and_commit(&config, &mut queue, error_message).await {
                        Ok(Some(entry)) => entry,
                        Ok(None) => continue,
                        Err(_error) => {
                            sleep(outbox_persistence_retry_delay(&config)).await;
                            continue;
                        }
                    };
                if let Some(responder) = entry.responder {
                    let _ = responder.send(Err(error));
                }
            }
        }
    }
}

async fn dead_letter_front_and_commit(
    config: &OutboxConfig,
    queue: &mut VecDeque<QueuedOutboxCommand>,
    reason: String,
) -> Result<Option<QueuedOutboxCommand>> {
    let Some(entry) = queue.front() else {
        return Ok(None);
    };

    let dead_letter = to_dead_letter(&entry.payload, reason);
    append_dead_letter_async(
        config.dead_letter_path.clone(),
        config.max_dead_letters,
        dead_letter,
    )
    .await?;

    commit_outbox_front(config, queue).await
}

fn outbox_persistence_retry_delay(config: &OutboxConfig) -> Duration {
    let delay = config.base_backoff.min(config.max_backoff);
    if delay.is_zero() {
        Duration::from_millis(50)
    } else {
        delay
    }
}

async fn commit_outbox_front(
    config: &OutboxConfig,
    queue: &mut VecDeque<QueuedOutboxCommand>,
) -> Result<Option<QueuedOutboxCommand>> {
    let Some(entry) = queue.pop_front() else {
        return Ok(None);
    };

    if let Err(error) = persist_outbox_queue_async(config.persistence_path.clone(), queue).await {
        queue.push_front(entry);
        return Err(error);
    }

    Ok(Some(entry))
}

fn prune_dedupe_cache(dedupe: &mut HashMap<String, (Message, Instant)>) {
    let now = Instant::now();
    dedupe.retain(|_, (_message, expires_at)| *expires_at > now);
}

fn unix_timestamp_millis_now() -> i64 {
    let Ok(duration) = SystemTime::now().duration_since(UNIX_EPOCH) else {
        return 0;
    };
    i64::try_from(duration.as_millis()).unwrap_or(i64::MAX)
}

fn is_outbox_message_expired(
    enqueued_at_unix_ms: i64,
    max_message_age: Option<Duration>,
    now_unix_ms: i64,
) -> bool {
    let Some(max_message_age) = max_message_age else {
        return false;
    };

    let max_age_ms = i64::try_from(max_message_age.as_millis()).unwrap_or(i64::MAX);
    let elapsed = now_unix_ms.saturating_sub(enqueued_at_unix_ms);
    elapsed >= max_age_ms
}

fn to_dead_letter(payload: &PersistedOutboxCommand, reason: String) -> DeadLetterEntry {
    DeadLetterEntry {
        chat_id: payload.chat_id.clone(),
        text: payload.text.clone(),
        idempotency_key: payload.idempotency_key.clone(),
        attempts: payload.attempt,
        reason,
        enqueued_at_unix_ms: payload.enqueued_at_unix_ms,
        failed_at_unix_ms: unix_timestamp_millis_now(),
    }
}

fn validate_outbox_snapshot(snapshot: &OutboxSnapshot, config: &OutboxConfig) -> Result<()> {
    if snapshot.version != default_outbox_snapshot_version() {
        return Err(invalid_request(format!(
            "unsupported outbox snapshot version `{}`",
            snapshot.version
        )));
    }
    if snapshot.queue.len() > config.queue_capacity {
        return Err(invalid_request(format!(
            "outbox snapshot queue length {} exceeds queue_capacity {}",
            snapshot.queue.len(),
            config.queue_capacity
        )));
    }

    for (index, command) in snapshot.queue.iter().enumerate() {
        validate_persisted_outbox_command(command, config).map_err(|error| {
            invalid_request(format!(
                "invalid outbox snapshot queue entry {index}: {error}"
            ))
        })?;
    }

    Ok(())
}

fn validate_persisted_outbox_command(
    command: &PersistedOutboxCommand,
    config: &OutboxConfig,
) -> Result<()> {
    let _ = SendMessageRequest::new(command.chat_id.clone(), command.text.clone())?;
    validate_idempotency_key(command.idempotency_key.as_deref())?;
    if command.enqueued_at_unix_ms < 0 {
        return Err(invalid_request(
            "outbox enqueued_at_unix_ms must not be negative",
        ));
    }
    if command.attempt >= config.max_attempts {
        return Err(invalid_request(format!(
            "outbox attempt {} has no remaining retry budget for max_attempts {}",
            command.attempt, config.max_attempts
        )));
    }

    Ok(())
}

fn validate_dead_letter_snapshot(snapshot: &DeadLetterSnapshot) -> Result<()> {
    if snapshot.version != default_dead_letter_snapshot_version() {
        return Err(invalid_request(format!(
            "unsupported dead-letter snapshot version `{}`",
            snapshot.version
        )));
    }

    for (index, entry) in snapshot.entries.iter().enumerate() {
        validate_dead_letter_entry(entry).map_err(|error| {
            invalid_request(format!(
                "invalid dead-letter snapshot entry {index}: {error}"
            ))
        })?;
    }

    Ok(())
}

fn validate_dead_letter_entry(entry: &DeadLetterEntry) -> Result<()> {
    let _ = SendMessageRequest::new(entry.chat_id.clone(), entry.text.clone())?;
    validate_idempotency_key(entry.idempotency_key.as_deref())?;
    if entry.reason.trim().is_empty() {
        return Err(invalid_request("dead-letter reason cannot be empty"));
    }
    if entry.enqueued_at_unix_ms < 0 {
        return Err(invalid_request(
            "dead-letter enqueued_at_unix_ms must not be negative",
        ));
    }
    if entry.failed_at_unix_ms < 0 {
        return Err(invalid_request(
            "dead-letter failed_at_unix_ms must not be negative",
        ));
    }

    Ok(())
}

fn validate_dead_letter_path(path: Option<&Path>) -> Result<()> {
    if let Some(path) = path
        && path.exists()
    {
        let snapshot = load_dead_letter_snapshot(path)?;
        validate_dead_letter_snapshot(&snapshot)?;
    }

    Ok(())
}

fn append_dead_letter(
    path: Option<&Path>,
    max_dead_letters: usize,
    entry: DeadLetterEntry,
) -> Result<()> {
    let Some(path) = path else {
        return Ok(());
    };

    let mut snapshot = load_dead_letter_snapshot(path)?;
    validate_dead_letter_snapshot(&snapshot)?;
    snapshot.entries.push(entry);
    if snapshot.entries.len() > max_dead_letters {
        let overflow = snapshot.entries.len().saturating_sub(max_dead_letters);
        snapshot.entries.drain(0..overflow);
    }

    let encoded =
        serde_json::to_vec(&snapshot).map_err(|source| Error::SerializeRequest { source })?;
    write_file_atomic(path, encoded.as_slice(), "dead-letter snapshot")?;
    Ok(())
}

async fn append_dead_letter_async(
    path: Option<PathBuf>,
    max_dead_letters: usize,
    entry: DeadLetterEntry,
) -> Result<()> {
    run_blocking_io(move || append_dead_letter(path.as_deref(), max_dead_letters, entry)).await
}

fn load_dead_letter_snapshot(path: &Path) -> Result<DeadLetterSnapshot> {
    if !path.exists() {
        return Ok(DeadLetterSnapshot {
            version: default_dead_letter_snapshot_version(),
            entries: Vec::new(),
        });
    }

    let raw = fs::read(path).map_err(|source| {
        storage_error(
            "dead-letter read",
            format!(
                "failed to read dead-letter snapshot `{}`: {source}",
                path.display()
            ),
            true,
        )
    })?;
    if raw.is_empty() {
        return Ok(DeadLetterSnapshot {
            version: default_dead_letter_snapshot_version(),
            entries: Vec::new(),
        });
    }

    let snapshot: DeadLetterSnapshot = serde_json::from_slice(&raw).map_err(|source| {
        invalid_request(format!(
            "failed to deserialize dead-letter snapshot `{}`: {source}",
            path.display()
        ))
    })?;
    validate_dead_letter_snapshot(&snapshot)?;
    Ok(snapshot)
}

fn load_outbox_queue(config: &OutboxConfig) -> Result<Vec<PersistedOutboxCommand>> {
    let Some(path) = config.persistence_path.as_deref() else {
        return Ok(Vec::new());
    };

    if !path.exists() {
        return Ok(Vec::new());
    }

    let raw = fs::read(path).map_err(|source| {
        storage_error(
            "outbox read",
            format!(
                "failed to read outbox snapshot `{}`: {source}",
                path.display()
            ),
            true,
        )
    })?;
    if raw.is_empty() {
        return Ok(Vec::new());
    }

    let snapshot: OutboxSnapshot = serde_json::from_slice(&raw).map_err(|source| {
        invalid_request(format!(
            "failed to deserialize outbox snapshot `{}`: {source}",
            path.display()
        ))
    })?;
    validate_outbox_snapshot(&snapshot, config)?;
    Ok(snapshot.queue)
}

fn persist_outbox_queue(path: Option<&Path>, queue: &[PersistedOutboxCommand]) -> Result<()> {
    let Some(path) = path else {
        return Ok(());
    };

    let snapshot = OutboxSnapshot {
        version: default_outbox_snapshot_version(),
        queue: queue.to_vec(),
    };
    let encoded =
        serde_json::to_vec(&snapshot).map_err(|source| Error::SerializeRequest { source })?;
    write_file_atomic(path, encoded.as_slice(), "outbox snapshot")?;
    Ok(())
}

async fn persist_outbox_queue_async(
    path: Option<PathBuf>,
    queue: &VecDeque<QueuedOutboxCommand>,
) -> Result<()> {
    let persisted_queue = queue
        .iter()
        .map(|entry| entry.payload.clone())
        .collect::<Vec<_>>();
    run_blocking_io(move || persist_outbox_queue(path.as_deref(), &persisted_queue)).await
}

async fn send_once(client: &Client, chat_id: &ChatId, text: &str) -> Result<Message> {
    let request = SendMessageRequest::new(chat_id.clone(), text.to_owned())?;
    client.messages().send_message(&request).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn dead_letter_failure_keeps_queue_entry()
    -> std::result::Result<(), Box<dyn std::error::Error>> {
        let root = std::env::temp_dir().join(format!(
            "tele-outbox-dlq-failure-{}-{}",
            std::process::id(),
            unix_timestamp_millis_now()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root)?;

        let queue_path = root.join("queue.json");
        let blocked_parent = root.join("dead-letter-parent");
        let dead_letter_path = blocked_parent.join("dead-letter.json");
        fs::write(&blocked_parent, b"not a directory")?;

        let config = OutboxConfig::default()
            .with_persistence_path(queue_path.clone())
            .with_dead_letter_path(dead_letter_path);
        let permit_source = Arc::new(Semaphore::new(0));
        let mut queue = VecDeque::from([QueuedOutboxCommand {
            payload: PersistedOutboxCommand {
                chat_id: ChatId::from(12_i64),
                text: "hello".to_owned(),
                idempotency_key: Some("dead-letter-key".to_owned()),
                enqueued_at_unix_ms: unix_timestamp_millis_now(),
                attempt: 1,
                last_error: Some("failed".to_owned()),
            },
            responder: None,
            _permit: OutboxQueuePermit::Persisted(permit_source),
        }]);

        let result =
            dead_letter_front_and_commit(&config, &mut queue, "delivery failed".to_owned()).await;

        assert!(matches!(result, Err(Error::Storage { .. })));
        assert_eq!(queue.len(), 1);
        assert!(!queue_path.exists());

        let _ = fs::remove_dir_all(&root);
        Ok(())
    }
}

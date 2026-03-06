use super::*;

/// Reliable send-side outbox configuration.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct OutboxConfig {
    pub queue_capacity: usize,
    pub max_attempts: usize,
    pub base_backoff: Duration,
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
}

/// Asynchronous outbox handle for reliable message delivery.
#[derive(Clone)]
pub struct BotOutbox {
    sender: mpsc::Sender<OutboxCommand>,
}

impl BotOutbox {
    pub fn spawn(client: Client, config: OutboxConfig) -> Self {
        let (sender, receiver) = mpsc::channel(config.queue_capacity.max(1));
        tokio::spawn(run_outbox_worker(client, config, receiver));
        Self { sender }
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
        let (responder, receiver) = oneshot::channel();
        let command = OutboxCommand {
            chat_id: chat_id.into(),
            text: text.into(),
            idempotency_key,
            responder,
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

async fn run_outbox_worker(
    client: Client,
    config: OutboxConfig,
    mut receiver: mpsc::Receiver<OutboxCommand>,
) {
    let mut dedupe: HashMap<String, (Message, Instant)> = HashMap::new();
    let persisted_queue = match load_outbox_queue_async(config.persistence_path.clone()).await {
        Ok(queue) => queue,
        Err(_error) => return,
    };
    let mut queue = persisted_queue
        .into_iter()
        .map(|payload| QueuedOutboxCommand {
            payload,
            responder: None,
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
            let entry = match commit_outbox_front(&config, &mut queue).await {
                Ok(Some(entry)) => entry,
                Ok(None) => continue,
                Err(_error) => {
                    sleep(outbox_persistence_retry_delay(&config)).await;
                    continue;
                }
            };
            let dead_letter = to_dead_letter(
                &entry.payload,
                "message expired in outbox before delivery".to_owned(),
            );
            let _ = append_dead_letter_async(
                config.dead_letter_path.clone(),
                config.max_dead_letters,
                dead_letter,
            )
            .await;
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
                let max_attempts = config.max_attempts.max(1);
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
                    let delay = error.retry_after().unwrap_or_else(|| {
                        exponential_backoff(config.base_backoff, config.max_backoff, attempt)
                    });
                    if let Err(_error) =
                        persist_outbox_queue_async(config.persistence_path.clone(), &queue).await
                    {
                        sleep(outbox_persistence_retry_delay(&config)).await;
                        continue;
                    }
                    sleep(delay.min(config.max_backoff)).await;
                    continue;
                }

                let entry = match commit_outbox_front(&config, &mut queue).await {
                    Ok(Some(entry)) => entry,
                    Ok(None) => continue,
                    Err(_error) => {
                        sleep(outbox_persistence_retry_delay(&config)).await;
                        continue;
                    }
                };
                let dead_letter = to_dead_letter(&entry.payload, error_message);
                let _ = append_dead_letter_async(
                    config.dead_letter_path.clone(),
                    config.max_dead_letters,
                    dead_letter,
                )
                .await;
                if let Some(responder) = entry.responder {
                    let _ = responder.send(Err(error));
                }
            }
        }
    }
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

fn append_dead_letter(
    path: Option<&Path>,
    max_dead_letters: usize,
    entry: DeadLetterEntry,
) -> Result<()> {
    let Some(path) = path else {
        return Ok(());
    };

    let mut snapshot = load_dead_letter_snapshot(path)?;
    snapshot.entries.push(entry);
    let max_dead_letters = max_dead_letters.max(1);
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

    let raw = fs::read(path).map_err(|source| Error::ReadLocalFile {
        path: path.display().to_string(),
        source,
    })?;
    if raw.is_empty() {
        return Ok(DeadLetterSnapshot {
            version: default_dead_letter_snapshot_version(),
            entries: Vec::new(),
        });
    }

    serde_json::from_slice(&raw).map_err(|source| {
        invalid_request(format!(
            "failed to deserialize dead-letter snapshot `{}`: {source}",
            path.display()
        ))
    })
}

fn load_outbox_queue(path: Option<&Path>) -> Result<Vec<PersistedOutboxCommand>> {
    let Some(path) = path else {
        return Ok(Vec::new());
    };

    if !path.exists() {
        return Ok(Vec::new());
    }

    let raw = fs::read(path).map_err(|source| Error::ReadLocalFile {
        path: path.display().to_string(),
        source,
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

async fn load_outbox_queue_async(path: Option<PathBuf>) -> Result<Vec<PersistedOutboxCommand>> {
    run_blocking_io(move || load_outbox_queue(path.as_deref())).await
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

use super::*;

/// State transition result for a finite-state machine.
#[derive(Clone, Debug)]
pub enum StateTransition<S> {
    Keep,
    Set(S),
    Clear,
}

/// Abstract async session-state store.
pub trait SessionStore<S>: Send + Sync + 'static
where
    S: Clone + Send + Sync + 'static,
{
    fn load<'a>(&'a self, chat_id: i64) -> SessionFuture<'a, Option<S>>;
    fn save<'a>(&'a self, chat_id: i64, state: S) -> SessionFuture<'a, ()>;
    fn clear<'a>(&'a self, chat_id: i64) -> SessionFuture<'a, ()>;
}

/// In-memory session store for prototyping and small bots.
pub struct InMemorySessionStore<S>
where
    S: Clone + Send + Sync + 'static,
{
    inner: Arc<RwLock<HashMap<i64, S>>>,
}

impl<S> Clone for InMemorySessionStore<S>
where
    S: Clone + Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<S> Default for InMemorySessionStore<S>
where
    S: Clone + Send + Sync + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<S> InMemorySessionStore<S>
where
    S: Clone + Send + Sync + 'static,
{
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl<S> SessionStore<S> for InMemorySessionStore<S>
where
    S: Clone + Send + Sync + 'static,
{
    fn load<'a>(&'a self, chat_id: i64) -> SessionFuture<'a, Option<S>> {
        Box::pin(async move {
            let guard = self.inner.read().await;
            Ok(guard.get(&chat_id).cloned())
        })
    }

    fn save<'a>(&'a self, chat_id: i64, state: S) -> SessionFuture<'a, ()> {
        Box::pin(async move {
            let mut guard = self.inner.write().await;
            guard.insert(chat_id, state);
            Ok(())
        })
    }

    fn clear<'a>(&'a self, chat_id: i64) -> SessionFuture<'a, ()> {
        Box::pin(async move {
            let mut guard = self.inner.write().await;
            guard.remove(&chat_id);
            Ok(())
        })
    }
}

/// JSON-file backed session store for bots that need process restart recovery.
pub struct JsonFileSessionStore<S>
where
    S: Clone + Send + Sync + Serialize + DeserializeOwned + 'static,
{
    path: PathBuf,
    inner: Arc<RwLock<HashMap<i64, S>>>,
    persist_lock: Arc<Mutex<()>>,
}

impl<S> Clone for JsonFileSessionStore<S>
where
    S: Clone + Send + Sync + Serialize + DeserializeOwned + 'static,
{
    fn clone(&self) -> Self {
        Self {
            path: self.path.clone(),
            inner: Arc::clone(&self.inner),
            persist_lock: Arc::clone(&self.persist_lock),
        }
    }
}

impl<S> JsonFileSessionStore<S>
where
    S: Clone + Send + Sync + Serialize + DeserializeOwned + 'static,
{
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let initial = load_session_snapshot::<S>(&path)?;
        Ok(Self {
            path,
            inner: Arc::new(RwLock::new(initial)),
            persist_lock: Arc::new(Mutex::new(())),
        })
    }

    pub fn path(&self) -> &Path {
        self.path.as_path()
    }
}

impl<S> SessionStore<S> for JsonFileSessionStore<S>
where
    S: Clone + Send + Sync + Serialize + DeserializeOwned + 'static,
{
    fn load<'a>(&'a self, chat_id: i64) -> SessionFuture<'a, Option<S>> {
        Box::pin(async move {
            let guard = self.inner.read().await;
            Ok(guard.get(&chat_id).cloned())
        })
    }

    fn save<'a>(&'a self, chat_id: i64, state: S) -> SessionFuture<'a, ()> {
        Box::pin(async move {
            let _persist_guard = self.persist_lock.lock().await;
            let snapshot = {
                let mut guard = self.inner.write().await;
                guard.insert(chat_id, state);
                guard.clone()
            };
            persist_session_snapshot_async(self.path.clone(), snapshot).await?;
            Ok(())
        })
    }

    fn clear<'a>(&'a self, chat_id: i64) -> SessionFuture<'a, ()> {
        Box::pin(async move {
            let _persist_guard = self.persist_lock.lock().await;
            let snapshot = {
                let mut guard = self.inner.write().await;
                guard.remove(&chat_id);
                guard.clone()
            };
            persist_session_snapshot_async(self.path.clone(), snapshot).await?;
            Ok(())
        })
    }
}

fn load_session_snapshot<S>(path: &Path) -> Result<HashMap<i64, S>>
where
    S: Clone + Send + Sync + Serialize + DeserializeOwned + 'static,
{
    if !path.exists() {
        return Ok(HashMap::new());
    }

    let raw = fs::read(path).map_err(|source| Error::ReadLocalFile {
        path: path.display().to_string(),
        source,
    })?;

    if raw.is_empty() {
        return Ok(HashMap::new());
    }

    serde_json::from_slice(&raw).map_err(|source| Error::InvalidRequest {
        reason: format!(
            "failed to deserialize session store `{}`: {source}",
            path.display()
        ),
    })
}

fn persist_session_snapshot<S>(path: &Path, snapshot: &HashMap<i64, S>) -> Result<()>
where
    S: Clone + Send + Sync + Serialize + DeserializeOwned + 'static,
{
    let encoded =
        serde_json::to_vec(snapshot).map_err(|source| Error::SerializeRequest { source })?;
    write_file_atomic(path, encoded.as_slice(), "session store")?;
    Ok(())
}

async fn persist_session_snapshot_async<S>(path: PathBuf, snapshot: HashMap<i64, S>) -> Result<()>
where
    S: Clone + Send + Sync + Serialize + DeserializeOwned + 'static,
{
    run_blocking_io(move || persist_session_snapshot(path.as_path(), &snapshot)).await
}

#[cfg(feature = "redis-session")]
/// Redis-backed session store for distributed bot deployments.
pub struct RedisSessionStore<S>
where
    S: Clone + Send + Sync + Serialize + DeserializeOwned + 'static,
{
    client: redis::Client,
    namespace: String,
    _state: std::marker::PhantomData<S>,
}

#[cfg(feature = "redis-session")]
impl<S> Clone for RedisSessionStore<S>
where
    S: Clone + Send + Sync + Serialize + DeserializeOwned + 'static,
{
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            namespace: self.namespace.clone(),
            _state: std::marker::PhantomData,
        }
    }
}

#[cfg(feature = "redis-session")]
impl<S> RedisSessionStore<S>
where
    S: Clone + Send + Sync + Serialize + DeserializeOwned + 'static,
{
    pub fn new(redis_url: &str, namespace: impl Into<String>) -> Result<Self> {
        let namespace = namespace.into();
        if namespace.trim().is_empty() {
            return Err(invalid_request("redis session namespace cannot be empty"));
        }

        let client = redis::Client::open(redis_url).map_err(|source| {
            invalid_request(format!(
                "failed to create redis client `{redis_url}`: {source}"
            ))
        })?;

        Ok(Self {
            client,
            namespace,
            _state: std::marker::PhantomData,
        })
    }

    pub fn namespace(&self) -> &str {
        self.namespace.as_str()
    }

    fn session_key(&self, chat_id: i64) -> String {
        format!("{}:{chat_id}", self.namespace)
    }

    async fn connection(&self) -> Result<redis::aio::MultiplexedConnection> {
        self.client
            .get_multiplexed_async_connection()
            .await
            .map_err(|source| invalid_request(format!("failed to connect redis: {source}")))
    }
}

#[cfg(feature = "redis-session")]
impl<S> SessionStore<S> for RedisSessionStore<S>
where
    S: Clone + Send + Sync + Serialize + DeserializeOwned + 'static,
{
    fn load<'a>(&'a self, chat_id: i64) -> SessionFuture<'a, Option<S>> {
        Box::pin(async move {
            let key = self.session_key(chat_id);
            let mut connection = self.connection().await?;
            let payload: Option<String> = redis::cmd("GET")
                .arg(&key)
                .query_async(&mut connection)
                .await
                .map_err(|source| invalid_request(format!("redis GET `{key}` failed: {source}")))?;

            let Some(payload) = payload else {
                return Ok(None);
            };

            let state = serde_json::from_str::<S>(&payload).map_err(|source| {
                invalid_request(format!(
                    "redis state decode failed for key `{key}`: {source}"
                ))
            })?;
            Ok(Some(state))
        })
    }

    fn save<'a>(&'a self, chat_id: i64, state: S) -> SessionFuture<'a, ()> {
        Box::pin(async move {
            let key = self.session_key(chat_id);
            let payload = serde_json::to_string(&state)
                .map_err(|source| Error::SerializeRequest { source })?;
            let mut connection = self.connection().await?;
            let _: () = redis::cmd("SET")
                .arg(&key)
                .arg(&payload)
                .query_async(&mut connection)
                .await
                .map_err(|source| invalid_request(format!("redis SET `{key}` failed: {source}")))?;
            Ok(())
        })
    }

    fn clear<'a>(&'a self, chat_id: i64) -> SessionFuture<'a, ()> {
        Box::pin(async move {
            let key = self.session_key(chat_id);
            let mut connection = self.connection().await?;
            let _: i64 = redis::cmd("DEL")
                .arg(&key)
                .query_async(&mut connection)
                .await
                .map_err(|source| invalid_request(format!("redis DEL `{key}` failed: {source}")))?;
            Ok(())
        })
    }
}

#[cfg(feature = "postgres-session")]
/// Postgres-backed session store for durable multi-instance bots.
pub struct PostgresSessionStore<S>
where
    S: Clone + Send + Sync + Serialize + DeserializeOwned + 'static,
{
    client: Arc<tokio_postgres::Client>,
    table: String,
    _state: std::marker::PhantomData<S>,
}

#[cfg(feature = "postgres-session")]
impl<S> Clone for PostgresSessionStore<S>
where
    S: Clone + Send + Sync + Serialize + DeserializeOwned + 'static,
{
    fn clone(&self) -> Self {
        Self {
            client: Arc::clone(&self.client),
            table: self.table.clone(),
            _state: std::marker::PhantomData,
        }
    }
}

#[cfg(feature = "postgres-session")]
impl<S> PostgresSessionStore<S>
where
    S: Clone + Send + Sync + Serialize + DeserializeOwned + 'static,
{
    pub async fn connect(database_url: &str, table: impl Into<String>) -> Result<Self> {
        let table = table.into();
        validate_sql_identifier(&table)?;

        let (client, connection) = tokio_postgres::connect(database_url, tokio_postgres::NoTls)
            .await
            .map_err(|source| {
                invalid_request(format!(
                    "failed to connect postgres `{database_url}`: {source}"
                ))
            })?;

        tokio::spawn(async move {
            let _ = connection.await;
        });

        let create = format!(
            "CREATE TABLE IF NOT EXISTS {table} (chat_id BIGINT PRIMARY KEY, state TEXT NOT NULL)"
        );
        client.execute(&create, &[]).await.map_err(|source| {
            invalid_request(format!(
                "failed to create postgres session table `{table}`: {source}"
            ))
        })?;

        Ok(Self {
            client: Arc::new(client),
            table,
            _state: std::marker::PhantomData,
        })
    }

    pub fn table(&self) -> &str {
        self.table.as_str()
    }
}

#[cfg(feature = "postgres-session")]
impl<S> SessionStore<S> for PostgresSessionStore<S>
where
    S: Clone + Send + Sync + Serialize + DeserializeOwned + 'static,
{
    fn load<'a>(&'a self, chat_id: i64) -> SessionFuture<'a, Option<S>> {
        Box::pin(async move {
            let query = format!("SELECT state FROM {} WHERE chat_id = $1", self.table);
            let row = self
                .client
                .query_opt(&query, &[&chat_id])
                .await
                .map_err(|source| {
                    invalid_request(format!(
                        "postgres load failed for chat_id `{chat_id}`: {source}"
                    ))
                })?;

            let Some(row) = row else {
                return Ok(None);
            };

            let payload: String = row.try_get(0).map_err(|source| {
                invalid_request(format!(
                    "postgres session payload decode failed for chat_id `{chat_id}`: {source}"
                ))
            })?;
            let state = serde_json::from_str::<S>(&payload).map_err(|source| {
                invalid_request(format!(
                    "postgres session json decode failed for chat_id `{chat_id}`: {source}"
                ))
            })?;
            Ok(Some(state))
        })
    }

    fn save<'a>(&'a self, chat_id: i64, state: S) -> SessionFuture<'a, ()> {
        Box::pin(async move {
            let query = format!(
                "INSERT INTO {} (chat_id, state) VALUES ($1, $2) \
                 ON CONFLICT (chat_id) DO UPDATE SET state = EXCLUDED.state",
                self.table
            );
            let payload = serde_json::to_string(&state)
                .map_err(|source| Error::SerializeRequest { source })?;
            self.client
                .execute(&query, &[&chat_id, &payload])
                .await
                .map_err(|source| {
                    invalid_request(format!(
                        "postgres save failed for chat_id `{chat_id}`: {source}"
                    ))
                })?;
            Ok(())
        })
    }

    fn clear<'a>(&'a self, chat_id: i64) -> SessionFuture<'a, ()> {
        Box::pin(async move {
            let query = format!("DELETE FROM {} WHERE chat_id = $1", self.table);
            self.client
                .execute(&query, &[&chat_id])
                .await
                .map_err(|source| {
                    invalid_request(format!(
                        "postgres clear failed for chat_id `{chat_id}`: {source}"
                    ))
                })?;
            Ok(())
        })
    }
}

#[cfg(feature = "postgres-session")]
fn validate_sql_identifier(identifier: &str) -> Result<()> {
    let mut chars = identifier.chars();
    let Some(first) = chars.next() else {
        return Err(invalid_request("sql identifier cannot be empty"));
    };

    if !(first.is_ascii_alphabetic() || first == '_') {
        return Err(invalid_request(format!(
            "sql identifier `{identifier}` must start with [A-Za-z_]"
        )));
    }

    if !chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_') {
        return Err(invalid_request(format!(
            "sql identifier `{identifier}` contains invalid characters"
        )));
    }

    Ok(())
}

/// Loads chat-scoped state from a store.
pub async fn load_chat_state<S, Store>(store: &Store, update: &Update) -> Result<Option<S>>
where
    S: Clone + Send + Sync + 'static,
    Store: SessionStore<S> + ?Sized,
{
    let Some(chat_id) = update_chat_id(update) else {
        return Err(invalid_request(
            "update does not contain a chat id for state operations",
        ));
    };

    store.load(chat_id).await
}

/// Saves chat-scoped state into a store.
pub async fn save_chat_state<S, Store>(store: &Store, update: &Update, state: S) -> Result<()>
where
    S: Clone + Send + Sync + 'static,
    Store: SessionStore<S> + ?Sized,
{
    let Some(chat_id) = update_chat_id(update) else {
        return Err(invalid_request(
            "update does not contain a chat id for state operations",
        ));
    };

    store.save(chat_id, state).await
}

/// Clears chat-scoped state from a store.
pub async fn clear_chat_state<S, Store>(store: &Store, update: &Update) -> Result<()>
where
    S: Clone + Send + Sync + 'static,
    Store: SessionStore<S> + ?Sized,
{
    let Some(chat_id) = update_chat_id(update) else {
        return Err(invalid_request(
            "update does not contain a chat id for state operations",
        ));
    };

    store.clear(chat_id).await
}

/// Applies an FSM transition to chat-scoped state.
pub async fn apply_chat_state_transition<S, Store>(
    store: &Store,
    update: &Update,
    transition: StateTransition<S>,
) -> Result<()>
where
    S: Clone + Send + Sync + 'static,
    Store: SessionStore<S> + ?Sized,
{
    match transition {
        StateTransition::Keep => Ok(()),
        StateTransition::Set(state) => save_chat_state(store, update, state).await,
        StateTransition::Clear => clear_chat_state::<S, Store>(store, update).await,
    }
}

/// High-level chat-scoped session manager for FSM-style bots.
pub struct ChatSession<S, Store>
where
    S: Clone + Send + Sync + 'static,
    Store: SessionStore<S>,
{
    store: Arc<Store>,
    _state: std::marker::PhantomData<S>,
}

impl<S, Store> Clone for ChatSession<S, Store>
where
    S: Clone + Send + Sync + 'static,
    Store: SessionStore<S>,
{
    fn clone(&self) -> Self {
        Self {
            store: Arc::clone(&self.store),
            _state: std::marker::PhantomData,
        }
    }
}

impl<S, Store> ChatSession<S, Store>
where
    S: Clone + Send + Sync + 'static,
    Store: SessionStore<S>,
{
    pub fn new(store: Store) -> Self {
        Self {
            store: Arc::new(store),
            _state: std::marker::PhantomData,
        }
    }

    pub fn from_shared(store: Arc<Store>) -> Self {
        Self {
            store,
            _state: std::marker::PhantomData,
        }
    }

    pub fn store(&self) -> &Store {
        self.store.as_ref()
    }

    pub fn shared_store(&self) -> Arc<Store> {
        Arc::clone(&self.store)
    }

    pub async fn load(&self, update: &Update) -> Result<Option<S>> {
        load_chat_state(self.store(), update).await
    }

    pub async fn save(&self, update: &Update, state: S) -> Result<()> {
        save_chat_state(self.store(), update, state).await
    }

    pub async fn clear(&self, update: &Update) -> Result<()> {
        clear_chat_state::<S, Store>(self.store(), update).await
    }

    pub async fn apply(&self, update: &Update, transition: StateTransition<S>) -> Result<()> {
        apply_chat_state_transition(self.store(), update, transition).await
    }

    /// Loads state, runs transition function, then applies resulting state transition.
    pub async fn transition<R, F, Fut>(&self, update: &Update, f: F) -> Result<R>
    where
        F: FnOnce(Option<S>) -> Fut + Send,
        Fut: Future<Output = (R, StateTransition<S>)> + Send,
    {
        let current = self.load(update).await?;
        let (output, transition) = f(current).await;
        self.apply(update, transition).await?;
        Ok(output)
    }
}

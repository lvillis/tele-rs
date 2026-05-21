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
            let mut snapshot = {
                let guard = self.inner.read().await;
                guard.clone()
            };
            snapshot.insert(chat_id, state);
            persist_session_snapshot_async(self.path.clone(), snapshot.clone()).await?;
            let mut guard = self.inner.write().await;
            *guard = snapshot;
            Ok(())
        })
    }

    fn clear<'a>(&'a self, chat_id: i64) -> SessionFuture<'a, ()> {
        Box::pin(async move {
            let _persist_guard = self.persist_lock.lock().await;
            let mut snapshot = {
                let guard = self.inner.read().await;
                guard.clone()
            };
            snapshot.remove(&chat_id);
            persist_session_snapshot_async(self.path.clone(), snapshot.clone()).await?;
            let mut guard = self.inner.write().await;
            *guard = snapshot;
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

    let raw = fs::read(path).map_err(|source| {
        storage_error(
            "json session read",
            format!(
                "failed to read session store `{}`: {source}",
                path.display()
            ),
            true,
        )
    })?;

    if raw.is_empty() {
        return Ok(HashMap::new());
    }

    serde_json::from_slice(&raw).map_err(|source| {
        storage_error(
            "json session load",
            format!(
                "failed to deserialize session store `{}`: {source}",
                path.display()
            ),
            false,
        )
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
        validate_redis_namespace(&namespace)?;

        let client = redis::Client::open(redis_url).map_err(|source| {
            invalid_request(format!(
                "failed to create redis client `{}`: {source}",
                crate::util::redact_url_credentials(redis_url)
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
            .map_err(|source| storage_error("redis connect", source.to_string(), true))
    }
}

#[cfg(feature = "redis-session")]
fn validate_redis_namespace(namespace: &str) -> Result<()> {
    if namespace.is_empty() || namespace.chars().any(char::is_whitespace) {
        return Err(invalid_request(
            "redis session namespace must be non-empty and contain no whitespace",
        ));
    }
    if namespace.chars().any(char::is_control) {
        return Err(invalid_request(
            "redis session namespace must not contain control characters",
        ));
    }

    Ok(())
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
                .map_err(|source| {
                    storage_error("redis GET", format!("key `{key}` failed: {source}"), true)
                })?;

            let Some(payload) = payload else {
                return Ok(None);
            };

            let state = serde_json::from_str::<S>(&payload).map_err(|source| {
                storage_error(
                    "redis decode",
                    format!("redis state decode failed for key `{key}`: {source}"),
                    false,
                )
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
                .map_err(|source| {
                    storage_error("redis SET", format!("key `{key}` failed: {source}"), true)
                })?;
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
                .map_err(|source| {
                    storage_error("redis DEL", format!("key `{key}` failed: {source}"), true)
                })?;
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
    pool: sqlx::PgPool,
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
            pool: self.pool.clone(),
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

        let pool = sqlx::postgres::PgPoolOptions::new()
            .connect(database_url)
            .await
            .map_err(|source| {
                storage_error(
                    "postgres connect",
                    format!(
                        "failed to connect postgres `{}`: {source}",
                        crate::util::redact_url_credentials(database_url)
                    ),
                    true,
                )
            })?;

        Self::with_pool(pool, table).await
    }

    pub async fn with_pool(pool: sqlx::PgPool, table: impl Into<String>) -> Result<Self> {
        let table = table.into();
        validate_sql_identifier(&table)?;

        let table_sql = quote_sql_identifier(&table);
        let create = format!(
            "CREATE TABLE IF NOT EXISTS {table_sql} (chat_id BIGINT PRIMARY KEY, state JSONB NOT NULL)"
        );
        sqlx::query(&create)
            .execute(&pool)
            .await
            .map_err(|source| {
                storage_error(
                    "postgres migrate",
                    format!("failed to create postgres session table `{table}`: {source}"),
                    false,
                )
            })?;

        Ok(Self {
            pool,
            table,
            _state: std::marker::PhantomData,
        })
    }

    pub fn table(&self) -> &str {
        self.table.as_str()
    }

    pub fn pool(&self) -> &sqlx::PgPool {
        &self.pool
    }
}

#[cfg(feature = "postgres-session")]
impl<S> SessionStore<S> for PostgresSessionStore<S>
where
    S: Clone + Send + Sync + Serialize + DeserializeOwned + 'static,
{
    fn load<'a>(&'a self, chat_id: i64) -> SessionFuture<'a, Option<S>> {
        Box::pin(async move {
            use sqlx::Row as _;

            let table = quote_sql_identifier(&self.table);
            let query = format!("SELECT state FROM {table} WHERE chat_id = $1");
            let row = sqlx::query(&query)
                .bind(chat_id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|source| {
                    storage_error(
                        "postgres load",
                        format!("postgres load failed for chat_id `{chat_id}`: {source}"),
                        true,
                    )
                })?;

            let Some(row) = row else {
                return Ok(None);
            };

            let sqlx::types::Json(state) = row.try_get(0).map_err(|source| {
                storage_error(
                    "postgres decode",
                    format!(
                        "postgres session payload decode failed for chat_id `{chat_id}`: {source}"
                    ),
                    false,
                )
            })?;

            Ok(Some(state))
        })
    }

    fn save<'a>(&'a self, chat_id: i64, state: S) -> SessionFuture<'a, ()> {
        Box::pin(async move {
            let table = quote_sql_identifier(&self.table);
            let query = format!(
                "INSERT INTO {table} (chat_id, state) VALUES ($1, $2) \
                 ON CONFLICT (chat_id) DO UPDATE SET state = EXCLUDED.state",
            );
            sqlx::query(&query)
                .bind(chat_id)
                .bind(sqlx::types::Json(state))
                .execute(&self.pool)
                .await
                .map_err(|source| {
                    storage_error(
                        "postgres save",
                        format!("postgres save failed for chat_id `{chat_id}`: {source}"),
                        true,
                    )
                })?;
            Ok(())
        })
    }

    fn clear<'a>(&'a self, chat_id: i64) -> SessionFuture<'a, ()> {
        Box::pin(async move {
            let table = quote_sql_identifier(&self.table);
            let query = format!("DELETE FROM {table} WHERE chat_id = $1");
            sqlx::query(&query)
                .bind(chat_id)
                .execute(&self.pool)
                .await
                .map_err(|source| {
                    storage_error(
                        "postgres clear",
                        format!("postgres clear failed for chat_id `{chat_id}`: {source}"),
                        true,
                    )
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

#[cfg(feature = "postgres-session")]
fn quote_sql_identifier(identifier: &str) -> String {
    format!("\"{identifier}\"")
}

/// Loads chat-scoped state from a store.
pub async fn load_chat_state<S, Store>(store: &Store, update: &Update) -> Result<Option<S>>
where
    S: Clone + Send + Sync + 'static,
    Store: SessionStore<S> + ?Sized,
{
    store.load(chat_id_for_state(update)?).await
}

/// Saves chat-scoped state into a store.
pub async fn save_chat_state<S, Store>(store: &Store, update: &Update, state: S) -> Result<()>
where
    S: Clone + Send + Sync + 'static,
    Store: SessionStore<S> + ?Sized,
{
    store.save(chat_id_for_state(update)?, state).await
}

/// Clears chat-scoped state from a store.
pub async fn clear_chat_state<S, Store>(store: &Store, update: &Update) -> Result<()>
where
    S: Clone + Send + Sync + 'static,
    Store: SessionStore<S> + ?Sized,
{
    store.clear(chat_id_for_state(update)?).await
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
    let chat_id = chat_id_for_state(update)?;
    match transition {
        StateTransition::Keep => Ok(()),
        StateTransition::Set(state) => store.save(chat_id, state).await,
        StateTransition::Clear => store.clear(chat_id).await,
    }
}

fn chat_id_for_state(update: &Update) -> Result<i64> {
    update_chat_id(update)
        .ok_or_else(|| invalid_request("update does not contain a chat id for state operations"))
}

async fn apply_chat_state_transition_for_chat_id<S, Store>(
    store: &Store,
    chat_id: i64,
    transition: StateTransition<S>,
) -> Result<()>
where
    S: Clone + Send + Sync + 'static,
    Store: SessionStore<S> + ?Sized,
{
    match transition {
        StateTransition::Keep => Ok(()),
        StateTransition::Set(state) => store.save(chat_id, state).await,
        StateTransition::Clear => store.clear(chat_id).await,
    }
}

#[derive(Clone)]
struct ChatSessionLocks {
    inner: Arc<ChatSessionLockMap>,
}

type ChatSessionLockMap = Mutex<HashMap<i64, Arc<Mutex<()>>>>;
type ChatSessionLockRegistry =
    std::sync::Mutex<HashMap<usize, std::sync::Weak<ChatSessionLockMap>>>;

fn chat_session_lock_registry() -> &'static ChatSessionLockRegistry {
    static REGISTRY: std::sync::OnceLock<ChatSessionLockRegistry> = std::sync::OnceLock::new();
    REGISTRY.get_or_init(|| std::sync::Mutex::new(HashMap::new()))
}

fn chat_session_store_key<Store>(store: &Arc<Store>) -> usize {
    Arc::as_ptr(store) as *const () as usize
}

impl ChatSessionLocks {
    fn for_store<Store>(store: &Arc<Store>) -> Self {
        let key = chat_session_store_key(store);
        let mut registry = chat_session_lock_registry()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        if let Some(inner) = registry.get(&key).and_then(std::sync::Weak::upgrade) {
            return Self { inner };
        }

        registry.retain(|_, locks| locks.strong_count() > 0);
        let inner = Arc::new(Mutex::new(HashMap::new()));
        registry.insert(key, Arc::downgrade(&inner));
        Self { inner }
    }

    async fn acquire(&self, chat_id: i64) -> ChatSessionLockGuard {
        let lock = {
            let mut locks = self.inner.lock().await;
            locks
                .entry(chat_id)
                .or_insert_with(|| Arc::new(Mutex::new(())))
                .clone()
        };
        let guard = lock.lock_owned().await;
        ChatSessionLockGuard { _guard: guard }
    }

    async fn prune_idle(&self, chat_id: i64) {
        let mut locks = self.inner.lock().await;
        if locks
            .get(&chat_id)
            .is_some_and(|lock| Arc::strong_count(lock) == 1)
        {
            let _ = locks.remove(&chat_id);
        }
    }
}

struct ChatSessionLockGuard {
    _guard: tokio::sync::OwnedMutexGuard<()>,
}

/// High-level chat-scoped session manager for FSM-style bots.
pub struct ChatSession<S, Store>
where
    S: Clone + Send + Sync + 'static,
    Store: SessionStore<S>,
{
    store: Arc<Store>,
    locks: ChatSessionLocks,
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
            locks: self.locks.clone(),
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
        let store = Arc::new(store);
        let locks = ChatSessionLocks::for_store(&store);
        Self {
            store,
            locks,
            _state: std::marker::PhantomData,
        }
    }

    pub fn from_shared(store: Arc<Store>) -> Self {
        let locks = ChatSessionLocks::for_store(&store);
        Self {
            store,
            locks,
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
        let chat_id = chat_id_for_state(update)?;
        let guard = self.locks.acquire(chat_id).await;
        let result = self.store.save(chat_id, state).await;
        drop(guard);
        self.locks.prune_idle(chat_id).await;
        result
    }

    pub async fn clear(&self, update: &Update) -> Result<()> {
        let chat_id = chat_id_for_state(update)?;
        let guard = self.locks.acquire(chat_id).await;
        let result = self.store.clear(chat_id).await;
        drop(guard);
        self.locks.prune_idle(chat_id).await;
        result
    }

    pub async fn apply(&self, update: &Update, transition: StateTransition<S>) -> Result<()> {
        let chat_id = chat_id_for_state(update)?;
        let guard = self.locks.acquire(chat_id).await;
        let result =
            apply_chat_state_transition_for_chat_id(self.store(), chat_id, transition).await;
        drop(guard);
        self.locks.prune_idle(chat_id).await;
        result
    }

    /// Loads state, runs transition function, then applies resulting state transition.
    ///
    /// Transitions are serialized per chat id, so concurrent updates for the same chat cannot
    /// overwrite each other's load-modify-save cycle. Different chats still progress independently.
    pub async fn transition<R, F, Fut>(&self, update: &Update, f: F) -> Result<R>
    where
        F: FnOnce(Option<S>) -> Fut + Send,
        Fut: Future<Output = (R, StateTransition<S>)> + Send,
    {
        let chat_id = chat_id_for_state(update)?;
        let guard = self.locks.acquire(chat_id).await;
        let current = self.store.load(chat_id).await?;
        let (output, transition) = f(current).await;
        let result =
            apply_chat_state_transition_for_chat_id(self.store(), chat_id, transition).await;
        drop(guard);
        self.locks.prune_idle(chat_id).await;
        result.map(|()| output)
    }
}

#[cfg(all(test, any(feature = "redis-session", feature = "postgres-session")))]
mod tests {
    use super::*;

    #[cfg(feature = "redis-session")]
    #[test]
    fn validates_redis_namespace() {
        assert!(validate_redis_namespace("tele:sessions").is_ok());

        for namespace in ["", " ", "tele sessions", "tele\nsessions"] {
            assert!(matches!(
                validate_redis_namespace(namespace),
                Err(Error::InvalidRequest { .. })
            ));
        }
    }

    #[cfg(feature = "postgres-session")]
    #[test]
    fn quotes_valid_postgres_identifier() -> Result<()> {
        validate_sql_identifier("SessionState")?;
        assert_eq!(quote_sql_identifier("SessionState"), "\"SessionState\"");
        Ok(())
    }
}

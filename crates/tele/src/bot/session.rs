use super::*;

/// State transition result for a finite-state machine.
#[derive(Clone, Debug)]
pub enum StateTransition<S> {
    Keep,
    Set(S),
    Clear,
}

/// Identity used by [`ChatSession`] to share per-chat transition locks.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SessionLockScope(usize);

impl SessionLockScope {
    /// Builds a scope from the address of a stable in-process value.
    pub fn from_ref<T>(value: &T) -> Self
    where
        T: Sized,
    {
        Self((value as *const T).cast::<()>() as usize)
    }

    /// Builds a scope from the allocation identity of a shared [`Arc`].
    pub fn from_arc<T>(value: &Arc<T>) -> Self
    where
        T: Sized,
    {
        Self(Arc::as_ptr(value).cast::<()>() as usize)
    }
}

/// Abstract async session-state store.
pub trait SessionStore<S>: Send + Sync + 'static
where
    S: Clone + Send + Sync + 'static,
{
    /// Returns the scope used to serialize chat-scoped load-modify-save transitions.
    ///
    /// Stores that can identify shared backing state across clones or independently-created
    /// handles should override this and return a stable identity for that shared state.
    fn session_lock_scope(&self) -> SessionLockScope
    where
        Self: Sized,
    {
        SessionLockScope::from_ref(self)
    }

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
    fn session_lock_scope(&self) -> SessionLockScope {
        SessionLockScope::from_arc(&self.inner)
    }

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
    locks: Arc<JsonFileSessionLocks>,
    _state: std::marker::PhantomData<S>,
}

struct JsonFileSessionLocks {
    persist: Mutex<()>,
}

impl<S> Clone for JsonFileSessionStore<S>
where
    S: Clone + Send + Sync + Serialize + DeserializeOwned + 'static,
{
    fn clone(&self) -> Self {
        Self {
            path: self.path.clone(),
            locks: Arc::clone(&self.locks),
            _state: std::marker::PhantomData,
        }
    }
}

impl<S> JsonFileSessionStore<S>
where
    S: Clone + Send + Sync + Serialize + DeserializeOwned + 'static,
{
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = normalize_file_storage_target(path.as_ref(), "session store")?;
        let locks = json_file_session_locks(&path)?;
        load_session_snapshot::<S>(&path)?;
        Ok(Self {
            path,
            locks,
            _state: std::marker::PhantomData,
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
    fn session_lock_scope(&self) -> SessionLockScope {
        SessionLockScope::from_arc(&self.locks)
    }

    fn load<'a>(&'a self, chat_id: i64) -> SessionFuture<'a, Option<S>> {
        Box::pin(async move {
            let _persist_guard = self.locks.persist.lock().await;
            let snapshot = load_session_snapshot_async::<S>(self.path.clone()).await?;
            Ok(snapshot.get(&chat_id).cloned())
        })
    }

    fn save<'a>(&'a self, chat_id: i64, state: S) -> SessionFuture<'a, ()> {
        Box::pin(async move {
            let _persist_guard = self.locks.persist.lock().await;
            let mut snapshot = load_session_snapshot_async::<S>(self.path.clone()).await?;
            snapshot.insert(chat_id, state);
            persist_session_snapshot_async(self.path.clone(), snapshot).await?;
            Ok(())
        })
    }

    fn clear<'a>(&'a self, chat_id: i64) -> SessionFuture<'a, ()> {
        Box::pin(async move {
            let _persist_guard = self.locks.persist.lock().await;
            let mut snapshot = load_session_snapshot_async::<S>(self.path.clone()).await?;
            snapshot.remove(&chat_id);
            persist_session_snapshot_async(self.path.clone(), snapshot).await?;
            Ok(())
        })
    }
}

type JsonFileSessionLockRegistry =
    std::sync::Mutex<HashMap<PathBuf, std::sync::Weak<JsonFileSessionLocks>>>;

fn json_file_session_lock_registry() -> &'static JsonFileSessionLockRegistry {
    static REGISTRY: std::sync::OnceLock<JsonFileSessionLockRegistry> = std::sync::OnceLock::new();
    REGISTRY.get_or_init(|| std::sync::Mutex::new(HashMap::new()))
}

fn json_file_session_locks(path: &Path) -> Result<Arc<JsonFileSessionLocks>> {
    let key = canonical_file_storage_path(path, "session store")?;
    let mut registry = json_file_session_lock_registry()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    if let Some(locks) = registry.get(&key).and_then(std::sync::Weak::upgrade) {
        return Ok(locks);
    }

    registry.retain(|_, locks| locks.strong_count() > 0);
    let locks = Arc::new(JsonFileSessionLocks {
        persist: Mutex::new(()),
    });
    registry.insert(key, Arc::downgrade(&locks));
    Ok(locks)
}

#[cfg(any(feature = "redis-session", feature = "postgres-session"))]
type UnitSessionLockRegistry<K> = std::sync::Mutex<HashMap<K, std::sync::Weak<()>>>;

#[cfg(any(feature = "redis-session", feature = "postgres-session"))]
fn shared_unit_session_lock_scope<K>(
    registry: &'static UnitSessionLockRegistry<K>,
    key: K,
) -> Arc<()>
where
    K: Eq + std::hash::Hash,
{
    let mut registry = registry
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    if let Some(scope) = registry.get(&key).and_then(std::sync::Weak::upgrade) {
        return scope;
    }

    registry.retain(|_, scope| scope.strong_count() > 0);
    let scope = Arc::new(());
    registry.insert(key, Arc::downgrade(&scope));
    scope
}

fn load_session_snapshot<S>(path: &Path) -> Result<HashMap<i64, S>>
where
    S: Clone + Send + Sync + Serialize + DeserializeOwned + 'static,
{
    let Some(raw) = read_optional_storage_file(path, "session store", "json session read")? else {
        return Ok(HashMap::new());
    };

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

async fn load_session_snapshot_async<S>(path: PathBuf) -> Result<HashMap<i64, S>>
where
    S: Clone + Send + Sync + Serialize + DeserializeOwned + 'static,
{
    run_blocking_io(move || load_session_snapshot(path.as_path())).await
}

fn persist_session_snapshot<S>(path: &Path, snapshot: &HashMap<i64, S>) -> Result<()>
where
    S: Clone + Send + Sync + Serialize + DeserializeOwned + 'static,
{
    let encoded = serde_json::to_vec(snapshot)
        .map_err(|source| storage_encode_error("json session encode", "session store", source))?;
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
    lock_scope: Arc<()>,
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
            lock_scope: Arc::clone(&self.lock_scope),
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
            configuration_error(format!(
                "failed to create redis client `{}`: {source}",
                crate::util::redact_url_credentials(redis_url)
            ))
        })?;
        let lock_scope = redis_session_lock_scope(client.get_connection_info(), &namespace);

        Ok(Self {
            client,
            namespace,
            lock_scope,
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
#[derive(Eq, Hash, PartialEq)]
struct RedisSessionLockKey {
    addr: Box<str>,
    db: i64,
    namespace: Box<str>,
}

#[cfg(feature = "redis-session")]
fn redis_session_lock_registry() -> &'static UnitSessionLockRegistry<RedisSessionLockKey> {
    static REGISTRY: std::sync::OnceLock<UnitSessionLockRegistry<RedisSessionLockKey>> =
        std::sync::OnceLock::new();
    REGISTRY.get_or_init(|| std::sync::Mutex::new(HashMap::new()))
}

#[cfg(feature = "redis-session")]
fn redis_session_lock_scope(connection_info: &redis::ConnectionInfo, namespace: &str) -> Arc<()> {
    shared_unit_session_lock_scope(
        redis_session_lock_registry(),
        RedisSessionLockKey {
            addr: redis_connection_addr_key(connection_info.addr()).into(),
            db: connection_info.redis_settings().db(),
            namespace: namespace.into(),
        },
    )
}

#[cfg(feature = "redis-session")]
fn redis_connection_addr_key(addr: &redis::ConnectionAddr) -> String {
    match addr {
        redis::ConnectionAddr::Tcp(host, port) => format!("tcp://{host}:{port}"),
        redis::ConnectionAddr::TcpTls {
            host,
            port,
            insecure,
            ..
        } => format!("rediss://{host}:{port}?insecure={insecure}"),
        redis::ConnectionAddr::Unix(path) => format!("unix://{}", path.display()),
        _ => addr.to_string(),
    }
}

#[cfg(feature = "redis-session")]
fn validate_redis_namespace(namespace: &str) -> Result<()> {
    if namespace.is_empty() || namespace.chars().any(char::is_whitespace) {
        return Err(configuration_error(
            "redis session namespace must be non-empty and contain no whitespace",
        ));
    }
    if namespace.chars().any(char::is_control) {
        return Err(configuration_error(
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
    fn session_lock_scope(&self) -> SessionLockScope {
        SessionLockScope::from_arc(&self.lock_scope)
    }

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
                .map_err(|source| storage_encode_error("redis encode", "redis state", source))?;
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
    lock_scope: Arc<()>,
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
            lock_scope: Arc::clone(&self.lock_scope),
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
        let options = database_url
            .parse::<sqlx::postgres::PgConnectOptions>()
            .map_err(|source| {
                configuration_error(format!(
                    "invalid postgres database URL `{}`: {source}",
                    crate::util::redact_url_credentials(database_url)
                ))
            })?;
        let lock_scope = postgres_session_lock_scope(&options, &table);

        let pool = sqlx::postgres::PgPoolOptions::new()
            .connect_with(options)
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

        Self::with_validated_pool(pool, table, lock_scope).await
    }

    pub async fn with_pool(pool: sqlx::PgPool, table: impl Into<String>) -> Result<Self> {
        let table = table.into();
        validate_sql_identifier(&table)?;
        Self::with_validated_pool(pool, table, Arc::new(())).await
    }

    async fn with_validated_pool(
        pool: sqlx::PgPool,
        table: String,
        lock_scope: Arc<()>,
    ) -> Result<Self> {
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
            lock_scope,
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
    fn session_lock_scope(&self) -> SessionLockScope {
        SessionLockScope::from_arc(&self.lock_scope)
    }

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
            let state = encode_postgres_state(state)?;
            let table = quote_sql_identifier(&self.table);
            let query = format!(
                "INSERT INTO {table} (chat_id, state) VALUES ($1, $2) \
                 ON CONFLICT (chat_id) DO UPDATE SET state = EXCLUDED.state",
            );
            sqlx::query(&query)
                .bind(chat_id)
                .bind(state)
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
#[derive(Eq, Hash, PartialEq)]
struct PostgresSessionLockKey {
    endpoint: Box<str>,
    username: Box<str>,
    database: Box<str>,
    session_options: Box<str>,
    table: Box<str>,
}

#[cfg(feature = "postgres-session")]
fn postgres_session_lock_registry() -> &'static UnitSessionLockRegistry<PostgresSessionLockKey> {
    static REGISTRY: std::sync::OnceLock<UnitSessionLockRegistry<PostgresSessionLockKey>> =
        std::sync::OnceLock::new();
    REGISTRY.get_or_init(|| std::sync::Mutex::new(HashMap::new()))
}

#[cfg(feature = "postgres-session")]
fn postgres_session_lock_scope(options: &sqlx::postgres::PgConnectOptions, table: &str) -> Arc<()> {
    let database = options
        .get_database()
        .unwrap_or_else(|| options.get_username());
    shared_unit_session_lock_scope(
        postgres_session_lock_registry(),
        PostgresSessionLockKey {
            endpoint: postgres_connection_endpoint_key(options).into(),
            username: options.get_username().into(),
            database: database.into(),
            session_options: options.get_options().unwrap_or_default().into(),
            table: table.into(),
        },
    )
}

#[cfg(feature = "postgres-session")]
fn postgres_connection_endpoint_key(options: &sqlx::postgres::PgConnectOptions) -> String {
    if let Some(socket) = options.get_socket() {
        format!("unix://{}:{}", socket.display(), options.get_port())
    } else {
        format!("tcp://{}:{}", options.get_host(), options.get_port())
    }
}

#[cfg(feature = "postgres-session")]
const POSTGRES_IDENTIFIER_MAX_BYTES: usize = 63;

#[cfg(feature = "postgres-session")]
fn validate_sql_identifier(identifier: &str) -> Result<()> {
    let mut chars = identifier.chars();
    let Some(first) = chars.next() else {
        return Err(configuration_error("sql identifier cannot be empty"));
    };

    if identifier.len() > POSTGRES_IDENTIFIER_MAX_BYTES {
        return Err(configuration_error(format!(
            "sql identifier `{identifier}` exceeds PostgreSQL {POSTGRES_IDENTIFIER_MAX_BYTES}-byte limit"
        )));
    }

    if !(first.is_ascii_alphabetic() || first == '_') {
        return Err(configuration_error(format!(
            "sql identifier `{identifier}` must start with [A-Za-z_]"
        )));
    }

    if !chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_') {
        return Err(configuration_error(format!(
            "sql identifier `{identifier}` contains invalid characters"
        )));
    }

    Ok(())
}

#[cfg(feature = "postgres-session")]
fn quote_sql_identifier(identifier: &str) -> String {
    format!("\"{identifier}\"")
}

#[cfg(feature = "postgres-session")]
fn encode_postgres_state<S>(state: S) -> Result<sqlx::types::Json<serde_json::Value>>
where
    S: Serialize,
{
    let value = serde_json::to_value(state)
        .map_err(|source| storage_encode_error("postgres encode", "postgres state", source))?;
    Ok(sqlx::types::Json(value))
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

type ChatSessionLockMap = std::sync::Mutex<HashMap<i64, Arc<Mutex<()>>>>;
type ChatSessionLockRegistry =
    std::sync::Mutex<HashMap<SessionLockScope, std::sync::Weak<ChatSessionLockMap>>>;

fn chat_session_lock_registry() -> &'static ChatSessionLockRegistry {
    static REGISTRY: std::sync::OnceLock<ChatSessionLockRegistry> = std::sync::OnceLock::new();
    REGISTRY.get_or_init(|| std::sync::Mutex::new(HashMap::new()))
}

impl ChatSessionLocks {
    fn for_store<S, Store>(store: &Arc<Store>) -> Self
    where
        S: Clone + Send + Sync + 'static,
        Store: SessionStore<S>,
    {
        let key = store.session_lock_scope();
        let mut registry = chat_session_lock_registry()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        if let Some(inner) = registry.get(&key).and_then(std::sync::Weak::upgrade) {
            return Self { inner };
        }

        registry.retain(|_, locks| locks.strong_count() > 0);
        let inner = Arc::new(std::sync::Mutex::new(HashMap::new()));
        registry.insert(key, Arc::downgrade(&inner));
        Self { inner }
    }

    async fn acquire(&self, chat_id: i64) -> ChatSessionLockGuard {
        let lock = {
            let mut locks = self
                .inner
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            locks
                .entry(chat_id)
                .or_insert_with(|| Arc::new(Mutex::new(())))
                .clone()
        };
        let guard = lock.lock_owned().await;
        ChatSessionLockGuard {
            locks: self.clone(),
            chat_id,
            guard: Some(guard),
        }
    }

    fn prune_idle(&self, chat_id: i64) {
        let mut locks = self
            .inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if locks
            .get(&chat_id)
            .is_some_and(|lock| Arc::strong_count(lock) == 1)
        {
            let _ = locks.remove(&chat_id);
        }
    }
}

struct ChatSessionLockGuard {
    locks: ChatSessionLocks,
    chat_id: i64,
    guard: Option<tokio::sync::OwnedMutexGuard<()>>,
}

impl Drop for ChatSessionLockGuard {
    fn drop(&mut self) {
        drop(self.guard.take());
        self.locks.prune_idle(self.chat_id);
    }
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
        let locks = ChatSessionLocks::for_store::<S, Store>(&store);
        Self {
            store,
            locks,
            _state: std::marker::PhantomData,
        }
    }

    pub fn from_shared(store: Arc<Store>) -> Self {
        let locks = ChatSessionLocks::for_store::<S, Store>(&store);
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

    /// Loads the latest committed state for the update's chat.
    ///
    /// Reads join the same per-chat queue as writes and transitions, so a load
    /// for a chat waits for any in-flight mutation of that chat to finish.
    pub async fn load(&self, update: &Update) -> Result<Option<S>> {
        let chat_id = chat_id_for_state(update)?;
        let _guard = self.locks.acquire(chat_id).await;
        self.store.load(chat_id).await
    }

    pub async fn save(&self, update: &Update, state: S) -> Result<()> {
        let chat_id = chat_id_for_state(update)?;
        let _guard = self.locks.acquire(chat_id).await;
        self.store.save(chat_id, state).await
    }

    pub async fn clear(&self, update: &Update) -> Result<()> {
        let chat_id = chat_id_for_state(update)?;
        let _guard = self.locks.acquire(chat_id).await;
        self.store.clear(chat_id).await
    }

    pub async fn apply(&self, update: &Update, transition: StateTransition<S>) -> Result<()> {
        let chat_id = chat_id_for_state(update)?;
        let _guard = self.locks.acquire(chat_id).await;
        apply_chat_state_transition_for_chat_id(self.store(), chat_id, transition).await
    }

    /// Loads state, runs transition function, then applies resulting state transition.
    ///
    /// Transitions are serialized per chat id, so concurrent updates for the same chat cannot
    /// overwrite each other's load-modify-save cycle. Different chats still progress independently.
    /// Do not call `ChatSession` methods for the same chat from inside the transition function;
    /// use the supplied state and return the desired [`StateTransition`] instead.
    pub async fn transition<R, F, Fut>(&self, update: &Update, f: F) -> Result<R>
    where
        F: FnOnce(Option<S>) -> Fut + Send,
        Fut: Future<Output = (R, StateTransition<S>)> + Send,
    {
        let chat_id = chat_id_for_state(update)?;
        let _guard = self.locks.acquire(chat_id).await;
        let current = self.store.load(chat_id).await?;
        let (output, transition) = f(current).await;
        let result =
            apply_chat_state_transition_for_chat_id(self.store(), chat_id, transition).await;
        result.map(|()| output)
    }
}

#[cfg(test)]
mod chat_session_tests {
    use super::*;

    fn unique_session_path(prefix: &str) -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0_u128, |duration| duration.as_nanos());
        std::env::temp_dir().join(format!("{prefix}-{timestamp}.json"))
    }

    fn unique_session_root(prefix: &str) -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0_u128, |duration| duration.as_nanos());
        std::env::temp_dir().join(format!("{prefix}-{}-{timestamp}", std::process::id()))
    }

    #[derive(Clone)]
    struct FailingLoadStore;

    impl SessionStore<String> for FailingLoadStore {
        fn load<'a>(&'a self, _chat_id: i64) -> SessionFuture<'a, Option<String>> {
            Box::pin(async {
                Err(storage_error(
                    "test load",
                    "load failed before transition",
                    true,
                ))
            })
        }

        fn save<'a>(&'a self, _chat_id: i64, _state: String) -> SessionFuture<'a, ()> {
            Box::pin(async { Ok(()) })
        }

        fn clear<'a>(&'a self, _chat_id: i64) -> SessionFuture<'a, ()> {
            Box::pin(async { Ok(()) })
        }
    }

    #[tokio::test]
    async fn chat_session_prunes_idle_lock_after_transition_load_error() -> Result<()> {
        let session = ChatSession::<String, _>::new(FailingLoadStore);
        let update: Update = serde_json::from_value(serde_json::json!({
            "update_id": 1,
            "message": {
                "message_id": 1,
                "date": 1,
                "chat": {"id": 42, "type": "private"},
                "text": "state"
            }
        }))
        .map_err(|source| invalid_request(format!("failed to build test update: {source}")))?;
        let chat_id = chat_id_for_state(&update)?;

        let result = session
            .transition(&update, |_state| async {
                ((), StateTransition::Set("unreachable".to_owned()))
            })
            .await;

        assert!(matches!(result, Err(Error::Storage { .. })));
        let locks = session
            .locks
            .inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        assert!(!locks.contains_key(&chat_id));
        Ok(())
    }

    #[test]
    fn chat_session_new_reuses_lock_scope_for_cloned_memory_store() {
        let store = InMemorySessionStore::<String>::new();
        let session_a = ChatSession::new(store.clone());
        let session_b = ChatSession::new(store);

        assert!(Arc::ptr_eq(&session_a.locks.inner, &session_b.locks.inner));
    }

    #[test]
    fn chat_session_new_reuses_lock_scope_for_same_json_file_path() -> Result<()> {
        let path = unique_session_path("tele-session-lock-scope");
        let store_a = JsonFileSessionStore::<String>::open(&path)?;
        let store_b = JsonFileSessionStore::<String>::open(&path)?;
        let session_a = ChatSession::new(store_a);
        let session_b = ChatSession::new(store_b);

        assert!(Arc::ptr_eq(&session_a.locks.inner, &session_b.locks.inner));
        let _ = fs::remove_file(path);
        Ok(())
    }

    #[cfg(feature = "redis-session")]
    #[test]
    fn chat_session_new_reuses_lock_scope_for_same_redis_backend() -> Result<()> {
        let store_a =
            RedisSessionStore::<String>::new("redis://writer:secret@127.0.0.1:6379/0", "tele")?;
        let store_b =
            RedisSessionStore::<String>::new("redis://reader:other@127.0.0.1:6379/0", "tele")?;
        let session_a = ChatSession::new(store_a);
        let session_b = ChatSession::new(store_b);

        assert!(Arc::ptr_eq(&session_a.locks.inner, &session_b.locks.inner));

        let other = RedisSessionStore::<String>::new("redis://127.0.0.1:6379/0", "other")?;
        let other_session = ChatSession::new(other);
        assert!(!Arc::ptr_eq(
            &session_a.locks.inner,
            &other_session.locks.inner
        ));
        Ok(())
    }

    #[cfg(feature = "postgres-session")]
    #[test]
    fn postgres_session_lock_scope_reuses_same_connect_target() -> Result<()> {
        let options_a = "postgres://tele:secret@localhost/tele"
            .parse::<sqlx::postgres::PgConnectOptions>()
            .map_err(|source| configuration_error(format!("failed to parse test URL: {source}")))?;
        let options_b = "postgres://tele:other@localhost/tele"
            .parse::<sqlx::postgres::PgConnectOptions>()
            .map_err(|source| configuration_error(format!("failed to parse test URL: {source}")))?;
        let scope_a = postgres_session_lock_scope(&options_a, "tele_sessions");
        let scope_b = postgres_session_lock_scope(&options_b, "tele_sessions");
        let other = postgres_session_lock_scope(&options_b, "other_sessions");

        assert!(Arc::ptr_eq(&scope_a, &scope_b));
        assert!(!Arc::ptr_eq(&scope_a, &other));
        Ok(())
    }

    #[test]
    fn json_file_session_store_normalizes_storage_path_on_open() -> Result<()> {
        let root = unique_session_root("tele-session-normalized-path");
        let nested = root.join("nested");
        fs::create_dir_all(&nested).map_err(|source| {
            storage_error(
                "test session mkdir",
                format!("failed to create session test directory: {source}"),
                true,
            )
        })?;
        let path = nested.join("..").join("session.json");
        let expected = root.canonicalize().map_err(|source| {
            storage_error(
                "test session canonicalize",
                format!("failed to canonicalize session test root: {source}"),
                true,
            )
        })?;

        let store = JsonFileSessionStore::<String>::open(&path)?;

        assert_eq!(store.path(), expected.join("session.json").as_path());
        let _ = fs::remove_dir_all(root);
        Ok(())
    }

    #[tokio::test]
    async fn json_file_session_store_reads_latest_snapshot_across_instances() -> Result<()> {
        let path = unique_session_path("tele-session-shared-file");
        let store_a = JsonFileSessionStore::<String>::open(&path)?;
        let store_b = JsonFileSessionStore::<String>::open(&path)?;

        store_a.save(1, "alpha".to_owned()).await?;
        assert_eq!(store_b.load(1).await?, Some("alpha".to_owned()));

        store_b.save(2, "beta".to_owned()).await?;
        assert_eq!(store_a.load(2).await?, Some("beta".to_owned()));

        let snapshot = load_session_snapshot::<String>(&path)?;
        assert_eq!(snapshot.get(&1).map(String::as_str), Some("alpha"));
        assert_eq!(snapshot.get(&2).map(String::as_str), Some("beta"));
        let _ = fs::remove_file(path);
        Ok(())
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn json_file_session_store_rejects_runtime_symlink_replacement() -> Result<()> {
        let path = unique_session_path("tele-session-runtime-symlink");
        let target = path.with_extension("target.json");
        let store = JsonFileSessionStore::<String>::open(&path)?;
        store.save(1, "committed".to_owned()).await?;

        fs::remove_file(&path).map_err(|source| {
            storage_error(
                "test session cleanup",
                format!(
                    "failed to remove session file `{}`: {source}",
                    path.display()
                ),
                true,
            )
        })?;
        fs::write(&target, br#"{"1":"hijacked"}"#).map_err(|source| {
            storage_error(
                "test session target write",
                format!(
                    "failed to write symlink target `{}`: {source}",
                    target.display()
                ),
                true,
            )
        })?;
        std::os::unix::fs::symlink(&target, &path).map_err(|source| {
            storage_error(
                "test session symlink",
                format!(
                    "failed to create session symlink `{}`: {source}",
                    path.display()
                ),
                true,
            )
        })?;

        let loaded = store.load(1).await;

        assert!(matches!(loaded, Err(Error::Storage { .. })));
        let _ = fs::remove_file(path);
        let _ = fs::remove_file(target);
        Ok(())
    }

    #[derive(Clone, serde::Deserialize)]
    struct FailingSerializeState;

    impl serde::Serialize for FailingSerializeState {
        fn serialize<Serializer>(
            &self,
            _serializer: Serializer,
        ) -> std::result::Result<Serializer::Ok, Serializer::Error>
        where
            Serializer: serde::Serializer,
        {
            Err(serde::ser::Error::custom("state cannot be serialized"))
        }
    }

    #[tokio::test]
    async fn json_file_session_store_reports_state_encode_failure_as_storage() -> Result<()> {
        let path = unique_session_path("tele-session-encode");
        let store = JsonFileSessionStore::<FailingSerializeState>::open(&path)?;

        let result = store.save(1, FailingSerializeState).await;

        assert!(matches!(result, Err(Error::Storage { .. })));
        assert!(!path.exists());
        Ok(())
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
                Err(Error::Configuration { .. })
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

    #[cfg(feature = "postgres-session")]
    #[test]
    fn rejects_postgres_identifiers_that_postgres_would_truncate() {
        let too_long = "a".repeat(POSTGRES_IDENTIFIER_MAX_BYTES + 1);

        assert!(matches!(
            validate_sql_identifier(&too_long),
            Err(Error::Configuration { .. })
        ));
    }

    #[cfg(feature = "postgres-session")]
    #[test]
    fn postgres_state_encode_failure_is_non_retryable_storage() {
        #[derive(Clone)]
        struct FailingSerializeState;

        impl Serialize for FailingSerializeState {
            fn serialize<Serializer>(
                &self,
                _serializer: Serializer,
            ) -> std::result::Result<Serializer::Ok, Serializer::Error>
            where
                Serializer: serde::Serializer,
            {
                Err(serde::ser::Error::custom("state cannot be serialized"))
            }
        }

        let result = encode_postgres_state(FailingSerializeState);

        assert!(matches!(
            result,
            Err(Error::Storage {
                operation,
                retryable: false,
                ..
            }) if operation.as_ref() == "postgres encode"
        ));
    }
}

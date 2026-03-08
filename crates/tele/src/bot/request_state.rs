use super::*;

type RequestStateValue = Arc<dyn Any + Send + Sync>;
type ContextExtensions = Arc<StdRwLock<HashMap<RequestStateSlotId, RequestStateValue>>>;

const DEFAULT_REQUEST_STATE_SLOT: &str = "";

fn downcast_request_state<T>(value: RequestStateValue) -> Option<Arc<T>>
where
    T: Send + Sync + 'static,
{
    Arc::downcast::<T>(value).ok()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
struct RequestStateSlotId {
    type_id: TypeId,
    slot: &'static str,
}

fn request_state_slot_id<T>(key: RequestStateKey<T>) -> RequestStateSlotId
where
    T: Send + Sync + 'static,
{
    RequestStateSlotId {
        type_id: TypeId::of::<T>(),
        slot: key.slot,
    }
}

/// Typed request-state slot descriptor.
#[derive(Debug, Eq, PartialEq, Hash)]
pub struct RequestStateKey<T> {
    slot: &'static str,
    _marker: std::marker::PhantomData<fn() -> T>,
}

impl<T> Clone for RequestStateKey<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for RequestStateKey<T> {}

impl<T> RequestStateKey<T> {
    pub const fn new(slot: &'static str) -> Self {
        Self {
            slot,
            _marker: std::marker::PhantomData,
        }
    }

    pub const fn slot(self) -> &'static str {
        self.slot
    }
}

/// Borrowed access to one typed request-state slot.
pub struct RequestStateSlot<'a, T> {
    state: &'a RequestState,
    key: RequestStateKey<T>,
}

impl<'a, T> RequestStateSlot<'a, T>
where
    T: Send + Sync + 'static,
{
    pub fn set(&self, value: T) -> Option<Arc<T>> {
        self.set_shared(Arc::new(value))
    }

    pub fn set_shared(&self, value: Arc<T>) -> Option<Arc<T>> {
        let previous = self
            .state
            .inner
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(request_state_slot_id(self.key), value);
        previous.and_then(downcast_request_state::<T>)
    }

    pub fn read_or_init_with(&self, init: impl FnOnce() -> T) -> Arc<T> {
        if let Some(value) = self.read() {
            return value;
        }

        let mut state = self
            .state
            .inner
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(value) = state
            .get(&request_state_slot_id(self.key))
            .cloned()
            .and_then(downcast_request_state::<T>)
        {
            return value;
        }

        let value = Arc::new(init());
        let _ = state.insert(request_state_slot_id(self.key), value.clone());
        value
    }

    pub fn read(&self) -> Option<Arc<T>> {
        self.state
            .inner
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(&request_state_slot_id(self.key))
            .cloned()
            .and_then(downcast_request_state::<T>)
    }

    pub fn cloned(&self) -> Option<T>
    where
        T: Clone,
    {
        self.read().map(|value| value.as_ref().clone())
    }

    pub fn with<R>(&self, map: impl FnOnce(&T) -> R) -> Option<R> {
        self.read().map(|value| map(value.as_ref()))
    }

    pub fn contains(&self) -> bool {
        self.state
            .inner
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .contains_key(&request_state_slot_id(self.key))
    }

    pub fn remove(&self) -> Option<Arc<T>> {
        self.state
            .inner
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .remove(&request_state_slot_id(self.key))
            .and_then(downcast_request_state::<T>)
    }
}

/// Typed request-scoped state store shared across middlewares and handlers for one dispatch.
#[derive(Clone, Default)]
pub struct RequestState {
    inner: ContextExtensions,
}

impl RequestState {
    pub fn slot<T>(&self, key: RequestStateKey<T>) -> RequestStateSlot<'_, T>
    where
        T: Send + Sync + 'static,
    {
        RequestStateSlot { state: self, key }
    }

    pub fn insert<T>(&self, value: T) -> Option<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        self.slot(RequestStateKey::<T>::new(DEFAULT_REQUEST_STATE_SLOT))
            .set(value)
    }

    pub fn insert_shared<T>(&self, value: Arc<T>) -> Option<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        self.slot(RequestStateKey::<T>::new(DEFAULT_REQUEST_STATE_SLOT))
            .set_shared(value)
    }

    pub fn get_or_insert_with<T>(&self, init: impl FnOnce() -> T) -> Arc<T>
    where
        T: Send + Sync + 'static,
    {
        self.slot(RequestStateKey::<T>::new(DEFAULT_REQUEST_STATE_SLOT))
            .read_or_init_with(init)
    }

    pub fn get<T>(&self) -> Option<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        self.slot(RequestStateKey::<T>::new(DEFAULT_REQUEST_STATE_SLOT))
            .read()
    }

    pub fn with<T, R>(&self, map: impl FnOnce(&T) -> R) -> Option<R>
    where
        T: Send + Sync + 'static,
    {
        self.slot(RequestStateKey::<T>::new(DEFAULT_REQUEST_STATE_SLOT))
            .with(map)
    }

    pub fn contains<T>(&self) -> bool
    where
        T: Send + Sync + 'static,
    {
        self.slot(RequestStateKey::<T>::new(DEFAULT_REQUEST_STATE_SLOT))
            .contains()
    }

    pub fn remove<T>(&self) -> Option<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        self.slot(RequestStateKey::<T>::new(DEFAULT_REQUEST_STATE_SLOT))
            .remove()
    }

    pub fn clear(&self) {
        self.inner
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clear();
    }
}

use super::*;

#[cfg(feature = "_async")]
use super::bootstrap::retry_with_config_async;
#[cfg(feature = "_blocking")]
use super::bootstrap::retry_with_config_blocking;

/// Typed Telegram API layer for async clients.
#[cfg(feature = "_async")]
#[derive(Clone)]
pub struct TypedApi {
    client: Client,
}

#[cfg(feature = "_async")]
impl TypedApi {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Calls a typed request that carries method name and response type.
    pub async fn call<Q>(&self, request: &Q) -> Result<Q::Response>
    where
        Q: AdvancedRequest,
    {
        self.client.call_method(Q::METHOD, request).await
    }

    /// Calls typed request with method-scoped retry policy.
    pub async fn call_with_retry<Q>(&self, request: &Q, retry: RetryConfig) -> Result<Q::Response>
    where
        Q: AdvancedRequest,
    {
        retry_with_config_async(&retry, || async {
            self.client.call_method(Q::METHOD, request).await
        })
        .await
    }
}

/// Typed Telegram API layer for blocking clients.
#[cfg(feature = "_blocking")]
#[derive(Clone)]
pub struct BlockingTypedApi {
    client: BlockingClient,
}

#[cfg(feature = "_blocking")]
impl BlockingTypedApi {
    pub(crate) fn new(client: BlockingClient) -> Self {
        Self { client }
    }

    /// Calls a typed request that carries method name and response type.
    pub fn call<Q>(&self, request: &Q) -> Result<Q::Response>
    where
        Q: AdvancedRequest,
    {
        self.client.call_method(Q::METHOD, request)
    }

    /// Calls typed request with method-scoped retry policy.
    pub fn call_with_retry<Q>(&self, request: &Q, retry: RetryConfig) -> Result<Q::Response>
    where
        Q: AdvancedRequest,
    {
        retry_with_config_blocking(&retry, || self.client.call_method(Q::METHOD, request))
    }
}

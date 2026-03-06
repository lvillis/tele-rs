use super::*;

#[cfg(feature = "_async")]
use super::bootstrap::retry_with_config_async;
#[cfg(feature = "_blocking")]
use super::bootstrap::retry_with_config_blocking;

/// Raw Telegram API calling layer for async clients.
#[cfg(feature = "_async")]
#[derive(Clone)]
pub struct RawApi {
    client: Client,
}

#[cfg(feature = "_async")]
impl RawApi {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Calls any Telegram method with JSON payload.
    pub async fn call_json<R, P>(&self, method: &str, payload: &P) -> Result<R>
    where
        R: DeserializeOwned,
        P: Serialize + ?Sized,
    {
        self.client.call_method(method, payload).await
    }

    /// Calls JSON method with method-scoped retry policy.
    pub async fn call_json_with_retry<R, P>(
        &self,
        method: &str,
        payload: &P,
        retry: RetryConfig,
    ) -> Result<R>
    where
        R: DeserializeOwned,
        P: Serialize + ?Sized,
    {
        retry_with_config_async(&retry, || async {
            self.client.call_method(method, payload).await
        })
        .await
    }

    /// Calls any Telegram method without payload.
    pub async fn call_no_params<R>(&self, method: &str) -> Result<R>
    where
        R: DeserializeOwned,
    {
        self.client.call_method_no_params(method).await
    }

    /// Calls no-params method with method-scoped retry policy.
    pub async fn call_no_params_with_retry<R>(&self, method: &str, retry: RetryConfig) -> Result<R>
    where
        R: DeserializeOwned,
    {
        retry_with_config_async(&retry, || async {
            self.client.call_method_no_params(method).await
        })
        .await
    }

    /// Calls any Telegram method with a multipart file part.
    pub async fn call_multipart<R, P>(
        &self,
        method: &str,
        payload: &P,
        file_field_name: &str,
        file: &UploadFile,
    ) -> Result<R>
    where
        R: DeserializeOwned,
        P: Serialize + ?Sized,
    {
        self.client
            .call_method_multipart(method, payload, file_field_name, file)
            .await
    }

    /// Calls multipart method with method-scoped retry policy.
    pub async fn call_multipart_with_retry<R, P>(
        &self,
        method: &str,
        payload: &P,
        file_field_name: &str,
        file: &UploadFile,
        retry: RetryConfig,
    ) -> Result<R>
    where
        R: DeserializeOwned,
        P: Serialize + ?Sized,
    {
        retry_with_config_async(&retry, || async {
            self.client
                .call_method_multipart(method, payload, file_field_name, file)
                .await
        })
        .await
    }
}

/// Raw Telegram API calling layer for blocking clients.
#[cfg(feature = "_blocking")]
#[derive(Clone)]
pub struct BlockingRawApi {
    client: BlockingClient,
}

#[cfg(feature = "_blocking")]
impl BlockingRawApi {
    pub(crate) fn new(client: BlockingClient) -> Self {
        Self { client }
    }

    /// Calls any Telegram method with JSON payload.
    pub fn call_json<R, P>(&self, method: &str, payload: &P) -> Result<R>
    where
        R: DeserializeOwned,
        P: Serialize + ?Sized,
    {
        self.client.call_method(method, payload)
    }

    /// Calls JSON method with method-scoped retry policy.
    pub fn call_json_with_retry<R, P>(
        &self,
        method: &str,
        payload: &P,
        retry: RetryConfig,
    ) -> Result<R>
    where
        R: DeserializeOwned,
        P: Serialize + ?Sized,
    {
        retry_with_config_blocking(&retry, || self.client.call_method(method, payload))
    }

    /// Calls any Telegram method without payload.
    pub fn call_no_params<R>(&self, method: &str) -> Result<R>
    where
        R: DeserializeOwned,
    {
        self.client.call_method_no_params(method)
    }

    /// Calls no-params method with method-scoped retry policy.
    pub fn call_no_params_with_retry<R>(&self, method: &str, retry: RetryConfig) -> Result<R>
    where
        R: DeserializeOwned,
    {
        retry_with_config_blocking(&retry, || self.client.call_method_no_params(method))
    }

    /// Calls any Telegram method with a multipart file part.
    pub fn call_multipart<R, P>(
        &self,
        method: &str,
        payload: &P,
        file_field_name: &str,
        file: &UploadFile,
    ) -> Result<R>
    where
        R: DeserializeOwned,
        P: Serialize + ?Sized,
    {
        self.client
            .call_method_multipart(method, payload, file_field_name, file)
    }

    /// Calls multipart method with method-scoped retry policy.
    pub fn call_multipart_with_retry<R, P>(
        &self,
        method: &str,
        payload: &P,
        file_field_name: &str,
        file: &UploadFile,
        retry: RetryConfig,
    ) -> Result<R>
    where
        R: DeserializeOwned,
        P: Serialize + ?Sized,
    {
        retry_with_config_blocking(&retry, || {
            self.client
                .call_method_multipart(method, payload, file_field_name, file)
        })
    }
}

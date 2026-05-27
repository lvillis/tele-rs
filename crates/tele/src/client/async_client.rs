use std::sync::Arc;
use std::time::Instant;

use serde::Serialize;
use serde::de::DeserializeOwned;
#[cfg(feature = "tracing")]
use tracing::Instrument;

use crate::api::{
    AdvancedService, BotService, ChatsService, FilesService, MessagesService, PaymentsService,
    StickersService, UpdatesService,
};
use crate::auth::Auth;
use crate::transport::async_transport::AsyncTransport;
use crate::transport::serialize_multipart_fields;
use crate::transport::{TransportRequestConfig, TransportRetryMode};
use crate::types::upload::{UploadFile, UploadPart};
use crate::{Error, Result};

use super::config::{BuilderParts, RequestDefaults};
use super::{
    AppApi, ClientBuilder, ClientObservability, ControlApi, RawApi, TypedApi,
    emit_client_result_metric,
};

#[derive(Clone)]
pub struct Client {
    inner: Arc<Inner>,
}

struct Inner {
    auth: Auth,
    defaults: RequestDefaults,
    observability: ClientObservability,
    transport: AsyncTransport,
}

impl Client {
    pub fn builder(base_url: impl AsRef<str>) -> Result<ClientBuilder> {
        ClientBuilder::new(base_url)
    }

    pub(crate) fn from_builder(builder: ClientBuilder) -> Result<Self> {
        builder.validate()?;
        let BuilderParts {
            base_url,
            auth,
            defaults,
            default_headers,
            observability,
        } = builder.into_parts();

        let transport = AsyncTransport::new(base_url, &defaults, &default_headers)?;

        Ok(Self {
            inner: Arc::new(Inner {
                auth,
                defaults,
                observability,
                transport,
            }),
        })
    }

    pub fn bot(&self) -> BotService {
        BotService::new(self.clone())
    }

    pub fn messages(&self) -> MessagesService {
        MessagesService::new(self.clone())
    }

    pub fn chats(&self) -> ChatsService {
        ChatsService::new(self.clone())
    }

    pub fn files(&self) -> FilesService {
        FilesService::new(self.clone())
    }

    pub fn stickers(&self) -> StickersService {
        StickersService::new(self.clone())
    }

    pub fn payments(&self) -> PaymentsService {
        PaymentsService::new(self.clone())
    }

    pub fn advanced(&self) -> AdvancedService {
        AdvancedService::new(self.clone())
    }

    pub fn updates(&self) -> UpdatesService {
        UpdatesService::new(self.clone())
    }

    /// Low-level raw method caller.
    pub fn raw(&self) -> RawApi {
        RawApi::new(self.clone())
    }

    /// Typed method caller based on request-associated response types.
    pub fn typed(&self) -> TypedApi {
        TypedApi::new(self.clone())
    }

    /// Stable app-facing high-level facade.
    pub fn app(&self) -> AppApi {
        AppApi::new(self.clone())
    }

    /// Stable control-plane facade for setup and runtime orchestration.
    pub fn control(&self) -> ControlApi {
        ControlApi::new(self.clone())
    }

    pub async fn call_method<R, P>(&self, method: &str, payload: &P) -> Result<R>
    where
        R: DeserializeOwned,
        P: Serialize + ?Sized,
    {
        self.call_method_with_transport_retry(method, payload, TransportRetryMode::Inherit)
            .await
    }

    pub(crate) async fn call_method_without_transport_retry<R, P>(
        &self,
        method: &str,
        payload: &P,
    ) -> Result<R>
    where
        R: DeserializeOwned,
        P: Serialize + ?Sized,
    {
        self.call_method_with_transport_retry(method, payload, TransportRetryMode::Disabled)
            .await
    }

    async fn call_method_with_transport_retry<R, P>(
        &self,
        method: &str,
        payload: &P,
        retry_mode: TransportRetryMode,
    ) -> Result<R>
    where
        R: DeserializeOwned,
        P: Serialize + ?Sized,
    {
        let token = self.require_token()?;
        let config = TransportRequestConfig::new(&self.inner.defaults, retry_mode);
        let started_at = Instant::now();
        #[cfg(feature = "tracing")]
        let request_future = self
            .inner
            .transport
            .execute_json(method, token, payload, config)
            .instrument(tracing::debug_span!("tele.client.request", method));
        #[cfg(not(feature = "tracing"))]
        let request_future = self
            .inner
            .transport
            .execute_json(method, token, payload, config);
        let result = request_future.await;
        emit_client_result_metric(
            &self.inner.observability,
            method,
            started_at.elapsed(),
            &result,
        );
        result
    }

    pub async fn call_method_no_params<R>(&self, method: &str) -> Result<R>
    where
        R: DeserializeOwned,
    {
        self.call_method_no_params_with_transport_retry(method, TransportRetryMode::Inherit)
            .await
    }

    pub(crate) async fn call_method_no_params_without_transport_retry<R>(
        &self,
        method: &str,
    ) -> Result<R>
    where
        R: DeserializeOwned,
    {
        self.call_method_no_params_with_transport_retry(method, TransportRetryMode::Disabled)
            .await
    }

    async fn call_method_no_params_with_transport_retry<R>(
        &self,
        method: &str,
        retry_mode: TransportRetryMode,
    ) -> Result<R>
    where
        R: DeserializeOwned,
    {
        let token = self.require_token()?;
        let config = TransportRequestConfig::new(&self.inner.defaults, retry_mode);
        let started_at = Instant::now();
        #[cfg(feature = "tracing")]
        let request_future = self
            .inner
            .transport
            .execute_empty(method, token, config)
            .instrument(tracing::debug_span!("tele.client.request", method));
        #[cfg(not(feature = "tracing"))]
        let request_future = self.inner.transport.execute_empty(method, token, config);
        let result = request_future.await;
        emit_client_result_metric(
            &self.inner.observability,
            method,
            started_at.elapsed(),
            &result,
        );
        result
    }

    pub async fn call_method_multipart<R, P>(
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
        self.call_method_multipart_with_transport_retry(
            method,
            payload,
            file_field_name,
            file,
            TransportRetryMode::Inherit,
        )
        .await
    }

    pub(crate) async fn call_method_multipart_without_transport_retry<R, P>(
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
        self.call_method_multipart_with_transport_retry(
            method,
            payload,
            file_field_name,
            file,
            TransportRetryMode::Disabled,
        )
        .await
    }

    async fn call_method_multipart_with_transport_retry<R, P>(
        &self,
        method: &str,
        payload: &P,
        file_field_name: &str,
        file: &UploadFile,
        retry_mode: TransportRetryMode,
    ) -> Result<R>
    where
        R: DeserializeOwned,
        P: Serialize + ?Sized,
    {
        let token = self.require_token()?;
        let fields = serialize_multipart_fields(payload, &[file_field_name])?;
        let config = TransportRequestConfig::new(&self.inner.defaults, retry_mode);
        let started_at = Instant::now();
        #[cfg(feature = "tracing")]
        let request_future = self
            .inner
            .transport
            .execute_multipart(method, token, &fields, file_field_name, file, config)
            .instrument(tracing::debug_span!("tele.client.request", method));
        #[cfg(not(feature = "tracing"))]
        let request_future = self.inner.transport.execute_multipart(
            method,
            token,
            &fields,
            file_field_name,
            file,
            config,
        );
        let result = request_future.await;
        emit_client_result_metric(
            &self.inner.observability,
            method,
            started_at.elapsed(),
            &result,
        );
        result
    }

    pub async fn call_method_multipart_files<R, P>(
        &self,
        method: &str,
        payload: &P,
        skip_fields: &[&str],
        files: &[UploadPart],
    ) -> Result<R>
    where
        R: DeserializeOwned,
        P: Serialize + ?Sized,
    {
        self.call_method_multipart_files_with_transport_retry(
            method,
            payload,
            skip_fields,
            files,
            TransportRetryMode::Inherit,
        )
        .await
    }

    pub(crate) async fn call_method_multipart_files_without_transport_retry<R, P>(
        &self,
        method: &str,
        payload: &P,
        skip_fields: &[&str],
        files: &[UploadPart],
    ) -> Result<R>
    where
        R: DeserializeOwned,
        P: Serialize + ?Sized,
    {
        self.call_method_multipart_files_with_transport_retry(
            method,
            payload,
            skip_fields,
            files,
            TransportRetryMode::Disabled,
        )
        .await
    }

    async fn call_method_multipart_files_with_transport_retry<R, P>(
        &self,
        method: &str,
        payload: &P,
        skip_fields: &[&str],
        files: &[UploadPart],
        retry_mode: TransportRetryMode,
    ) -> Result<R>
    where
        R: DeserializeOwned,
        P: Serialize + ?Sized,
    {
        let token = self.require_token()?;
        let fields = serialize_multipart_fields(payload, skip_fields)?;
        let config = TransportRequestConfig::new(&self.inner.defaults, retry_mode);
        let started_at = Instant::now();
        #[cfg(feature = "tracing")]
        let request_future = self
            .inner
            .transport
            .execute_multipart_files(method, token, &fields, files, config)
            .instrument(tracing::debug_span!("tele.client.request", method));
        #[cfg(not(feature = "tracing"))]
        let request_future = self
            .inner
            .transport
            .execute_multipart_files(method, token, &fields, files, config);
        let result = request_future.await;
        emit_client_result_metric(
            &self.inner.observability,
            method,
            started_at.elapsed(),
            &result,
        );
        result
    }

    #[cfg(feature = "bot")]
    pub(crate) fn request_timeout(&self) -> std::time::Duration {
        self.inner.defaults.request_timeout
    }

    #[cfg(feature = "bot")]
    pub(crate) fn total_timeout(&self) -> Option<std::time::Duration> {
        self.inner.defaults.total_timeout
    }

    fn require_token(&self) -> Result<&str> {
        self.inner.auth.token().ok_or(Error::MissingBotToken)
    }
}

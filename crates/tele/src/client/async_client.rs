use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::Serialize;
use serde::de::DeserializeOwned;
#[cfg(feature = "tracing")]
use tracing::Instrument;

use crate::api::{
    AdvancedService, BotService, ChatsService, FilesService, MessagesService, PaymentsService,
    StickersService, UpdatesService,
};
use crate::auth::Auth;
use crate::transport::TransportRequestConfig;
use crate::transport::async_transport::AsyncTransport;
use crate::transport::serialize_multipart_fields;
use crate::types::upload::{UploadFile, UploadPart};
use crate::{Error, Result};

use super::config::{BuilderParts, RequestDefaults};
use super::retry::retry_method_async;
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
        let retry = self.inner.defaults.retry.clone();
        retry_method_async(
            method,
            &retry,
            self.inner.defaults.total_timeout,
            |total_timeout| async move {
                self.call_method_attempt(method, payload, total_timeout)
                    .await
            },
        )
        .await
    }

    pub(crate) async fn call_method_once<R, P>(&self, method: &str, payload: &P) -> Result<R>
    where
        R: DeserializeOwned,
        P: Serialize + ?Sized,
    {
        self.call_method_attempt(method, payload, self.inner.defaults.total_timeout)
            .await
    }

    pub(crate) async fn call_method_attempt<R, P>(
        &self,
        method: &str,
        payload: &P,
        total_timeout: Option<Duration>,
    ) -> Result<R>
    where
        R: DeserializeOwned,
        P: Serialize + ?Sized,
    {
        let token = self.require_token()?;
        let config =
            TransportRequestConfig::with_total_timeout(&self.inner.defaults, total_timeout);
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
        let retry = self.inner.defaults.retry.clone();
        retry_method_async(
            method,
            &retry,
            self.inner.defaults.total_timeout,
            |total_timeout| async move {
                self.call_method_no_params_attempt(method, total_timeout)
                    .await
            },
        )
        .await
    }

    pub(crate) async fn call_method_no_params_once<R>(&self, method: &str) -> Result<R>
    where
        R: DeserializeOwned,
    {
        self.call_method_no_params_attempt(method, self.inner.defaults.total_timeout)
            .await
    }

    pub(crate) async fn call_method_no_params_attempt<R>(
        &self,
        method: &str,
        total_timeout: Option<Duration>,
    ) -> Result<R>
    where
        R: DeserializeOwned,
    {
        let token = self.require_token()?;
        let config =
            TransportRequestConfig::with_total_timeout(&self.inner.defaults, total_timeout);
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
        let retry = self.inner.defaults.retry.clone();
        retry_method_async(
            method,
            &retry,
            self.inner.defaults.total_timeout,
            |total_timeout| async move {
                self.call_method_multipart_attempt(
                    method,
                    payload,
                    file_field_name,
                    file,
                    total_timeout,
                )
                .await
            },
        )
        .await
    }

    pub(crate) async fn call_method_multipart_attempt<R, P>(
        &self,
        method: &str,
        payload: &P,
        file_field_name: &str,
        file: &UploadFile,
        total_timeout: Option<Duration>,
    ) -> Result<R>
    where
        R: DeserializeOwned,
        P: Serialize + ?Sized,
    {
        let token = self.require_token()?;
        let fields = serialize_multipart_fields(payload, &[file_field_name])?;
        let config =
            TransportRequestConfig::with_total_timeout(&self.inner.defaults, total_timeout);
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
        let retry = self.inner.defaults.retry.clone();
        retry_method_async(
            method,
            &retry,
            self.inner.defaults.total_timeout,
            |total_timeout| async move {
                self.call_method_multipart_files_attempt(
                    method,
                    payload,
                    skip_fields,
                    files,
                    total_timeout,
                )
                .await
            },
        )
        .await
    }

    pub(crate) async fn call_method_multipart_files_attempt<R, P>(
        &self,
        method: &str,
        payload: &P,
        skip_fields: &[&str],
        files: &[UploadPart],
        total_timeout: Option<Duration>,
    ) -> Result<R>
    where
        R: DeserializeOwned,
        P: Serialize + ?Sized,
    {
        let token = self.require_token()?;
        let fields = serialize_multipart_fields(payload, skip_fields)?;
        let config =
            TransportRequestConfig::with_total_timeout(&self.inner.defaults, total_timeout);
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

    pub(crate) fn total_timeout(&self) -> Option<std::time::Duration> {
        self.inner.defaults.total_timeout
    }

    fn require_token(&self) -> Result<&str> {
        self.inner.auth.token().ok_or(Error::MissingBotToken)
    }
}

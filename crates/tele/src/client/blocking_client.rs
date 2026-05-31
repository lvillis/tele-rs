use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::api::{
    BlockingAdvancedService, BlockingBotService, BlockingChatsService, BlockingFilesService,
    BlockingMessagesService, BlockingPaymentsService, BlockingStickersService,
    BlockingUpdatesService,
};
use crate::auth::Auth;
use crate::transport::TransportRequestConfig;
use crate::transport::blocking_transport::BlockingTransport;
use crate::transport::serialize_multipart_fields;
use crate::types::upload::{UploadFile, UploadPart};
use crate::{Error, Result};

use super::config::{BuilderParts, RequestDefaults};
use super::retry::retry_method_blocking;
use super::{
    BlockingAppApi, BlockingControlApi, BlockingRawApi, BlockingTypedApi, ClientBuilder,
    ClientObservability, emit_client_result_metric,
};

#[derive(Clone)]
pub struct BlockingClient {
    inner: Arc<Inner>,
}

struct Inner {
    auth: Auth,
    defaults: RequestDefaults,
    observability: ClientObservability,
    transport: BlockingTransport,
}

impl BlockingClient {
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

        let transport = BlockingTransport::new(base_url, &defaults, &default_headers)?;

        Ok(Self {
            inner: Arc::new(Inner {
                auth,
                defaults,
                observability,
                transport,
            }),
        })
    }

    pub fn bot(&self) -> BlockingBotService {
        BlockingBotService::new(self.clone())
    }

    pub fn messages(&self) -> BlockingMessagesService {
        BlockingMessagesService::new(self.clone())
    }

    pub fn chats(&self) -> BlockingChatsService {
        BlockingChatsService::new(self.clone())
    }

    pub fn files(&self) -> BlockingFilesService {
        BlockingFilesService::new(self.clone())
    }

    pub fn stickers(&self) -> BlockingStickersService {
        BlockingStickersService::new(self.clone())
    }

    pub fn payments(&self) -> BlockingPaymentsService {
        BlockingPaymentsService::new(self.clone())
    }

    pub fn advanced(&self) -> BlockingAdvancedService {
        BlockingAdvancedService::new(self.clone())
    }

    pub fn updates(&self) -> BlockingUpdatesService {
        BlockingUpdatesService::new(self.clone())
    }

    /// Low-level raw method caller.
    pub fn raw(&self) -> BlockingRawApi {
        BlockingRawApi::new(self.clone())
    }

    /// Typed method caller based on request-associated response types.
    pub fn typed(&self) -> BlockingTypedApi {
        BlockingTypedApi::new(self.clone())
    }

    /// Stable app-facing high-level facade.
    pub fn app(&self) -> BlockingAppApi {
        BlockingAppApi::new(self.clone())
    }

    /// Stable control-plane facade for setup orchestration.
    pub fn control(&self) -> BlockingControlApi {
        BlockingControlApi::new(self.clone())
    }

    pub fn call_method<R, P>(&self, method: &str, payload: &P) -> Result<R>
    where
        R: DeserializeOwned,
        P: Serialize + ?Sized,
    {
        let retry = self.inner.defaults.retry.clone();
        retry_method_blocking(
            method,
            &retry,
            self.inner.defaults.total_timeout,
            |total_timeout| self.call_method_attempt(method, payload, total_timeout),
        )
    }

    pub(crate) fn call_method_once<R, P>(&self, method: &str, payload: &P) -> Result<R>
    where
        R: DeserializeOwned,
        P: Serialize + ?Sized,
    {
        self.call_method_attempt(method, payload, self.inner.defaults.total_timeout)
    }

    pub(crate) fn call_method_attempt<R, P>(
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
        #[cfg(feature = "tracing")]
        let _span = tracing::debug_span!("tele.client.request", method).entered();
        let started_at = Instant::now();
        let result = self
            .inner
            .transport
            .execute_json(method, token, payload, config);
        emit_client_result_metric(
            &self.inner.observability,
            method,
            started_at.elapsed(),
            &result,
        );
        result
    }

    pub fn call_method_no_params<R>(&self, method: &str) -> Result<R>
    where
        R: DeserializeOwned,
    {
        let retry = self.inner.defaults.retry.clone();
        retry_method_blocking(
            method,
            &retry,
            self.inner.defaults.total_timeout,
            |total_timeout| self.call_method_no_params_attempt(method, total_timeout),
        )
    }

    pub(crate) fn call_method_no_params_once<R>(&self, method: &str) -> Result<R>
    where
        R: DeserializeOwned,
    {
        self.call_method_no_params_attempt(method, self.inner.defaults.total_timeout)
    }

    pub(crate) fn call_method_no_params_attempt<R>(
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
        #[cfg(feature = "tracing")]
        let _span = tracing::debug_span!("tele.client.request", method).entered();
        let started_at = Instant::now();
        let result = self.inner.transport.execute_empty(method, token, config);
        emit_client_result_metric(
            &self.inner.observability,
            method,
            started_at.elapsed(),
            &result,
        );
        result
    }

    pub fn call_method_multipart<R, P>(
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
        retry_method_blocking(
            method,
            &retry,
            self.inner.defaults.total_timeout,
            |total_timeout| {
                self.call_method_multipart_attempt(
                    method,
                    payload,
                    file_field_name,
                    file,
                    total_timeout,
                )
            },
        )
    }

    pub(crate) fn call_method_multipart_attempt<R, P>(
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
        #[cfg(feature = "tracing")]
        let _span = tracing::debug_span!("tele.client.request", method).entered();
        let started_at = Instant::now();
        let result = self.inner.transport.execute_multipart(
            method,
            token,
            &fields,
            file_field_name,
            file,
            config,
        );
        emit_client_result_metric(
            &self.inner.observability,
            method,
            started_at.elapsed(),
            &result,
        );
        result
    }

    pub fn call_method_multipart_files<R, P>(
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
        retry_method_blocking(
            method,
            &retry,
            self.inner.defaults.total_timeout,
            |total_timeout| {
                self.call_method_multipart_files_attempt(
                    method,
                    payload,
                    skip_fields,
                    files,
                    total_timeout,
                )
            },
        )
    }

    pub(crate) fn call_method_multipart_files_attempt<R, P>(
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
        #[cfg(feature = "tracing")]
        let _span = tracing::debug_span!("tele.client.request", method).entered();
        let started_at = Instant::now();
        let result = self
            .inner
            .transport
            .execute_multipart_files(method, token, &fields, files, config);
        emit_client_result_metric(
            &self.inner.observability,
            method,
            started_at.elapsed(),
            &result,
        );
        result
    }

    fn require_token(&self) -> Result<&str> {
        self.inner.auth.token().ok_or(Error::MissingBotToken)
    }

    pub(crate) fn total_timeout(&self) -> Option<std::time::Duration> {
        self.inner.defaults.total_timeout
    }
}

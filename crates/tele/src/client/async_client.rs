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
use crate::transport::async_transport::AsyncTransport;
use crate::transport::serialize_multipart_fields;
use crate::types::upload::UploadFile;
use crate::{Error, Result};

use super::config::BuilderParts;
use super::{
    AppApi, ClientBuilder, ClientObservability, ControlApi, RawApi, RequestDefaults, TypedApi,
    emit_client_metric,
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
        let token = self.require_token()?;
        let started_at = Instant::now();
        #[cfg(feature = "tracing")]
        let request_future = self
            .inner
            .transport
            .execute_json(method, token, payload, &self.inner.defaults)
            .instrument(tracing::debug_span!("tele.client.request", method));
        #[cfg(not(feature = "tracing"))]
        let request_future =
            self.inner
                .transport
                .execute_json(method, token, payload, &self.inner.defaults);
        let result = request_future.await;
        self.emit_metric(method, started_at.elapsed(), &result);
        result
    }

    pub async fn call_method_no_params<R>(&self, method: &str) -> Result<R>
    where
        R: DeserializeOwned,
    {
        let token = self.require_token()?;
        let started_at = Instant::now();
        #[cfg(feature = "tracing")]
        let request_future = self
            .inner
            .transport
            .execute_empty(method, token, &self.inner.defaults)
            .instrument(tracing::debug_span!("tele.client.request", method));
        #[cfg(not(feature = "tracing"))]
        let request_future =
            self.inner
                .transport
                .execute_empty(method, token, &self.inner.defaults);
        let result = request_future.await;
        self.emit_metric(method, started_at.elapsed(), &result);
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
        let token = self.require_token()?;
        let fields = serialize_multipart_fields(payload, &[file_field_name])?;
        let started_at = Instant::now();
        #[cfg(feature = "tracing")]
        let request_future = self
            .inner
            .transport
            .execute_multipart(
                method,
                token,
                &fields,
                file_field_name,
                file,
                &self.inner.defaults,
            )
            .instrument(tracing::debug_span!("tele.client.request", method));
        #[cfg(not(feature = "tracing"))]
        let request_future = self.inner.transport.execute_multipart(
            method,
            token,
            &fields,
            file_field_name,
            file,
            &self.inner.defaults,
        );
        let result = request_future.await;
        self.emit_metric(method, started_at.elapsed(), &result);
        result
    }

    #[cfg(feature = "bot")]
    pub(crate) fn request_timeout(&self) -> Duration {
        self.inner.defaults.request_timeout
    }

    #[cfg(feature = "bot")]
    pub(crate) fn total_timeout(&self) -> Option<Duration> {
        self.inner.defaults.total_timeout
    }

    fn require_token(&self) -> Result<&str> {
        self.inner.auth.token().ok_or(Error::MissingBotToken)
    }

    fn emit_metric<R>(&self, method: &str, latency: Duration, result: &Result<R>) {
        let (success, status, classification, retryable, request_id) = match result {
            Ok(_) => (true, None, None, false, None),
            Err(error) => (
                false,
                error.status().map(|status| status.as_u16()),
                Some(error.classification()),
                error.is_retryable(),
                error.request_id().map(ToOwned::to_owned),
            ),
        };
        emit_client_metric(
            &self.inner.observability,
            super::ClientMetric {
                method: method.to_owned(),
                success,
                latency,
                status,
                classification,
                retryable,
                request_id,
            },
        );
    }
}

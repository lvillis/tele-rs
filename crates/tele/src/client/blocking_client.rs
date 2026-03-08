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
use crate::transport::blocking_transport::BlockingTransport;
use crate::transport::serialize_multipart_fields;
use crate::types::upload::UploadFile;
use crate::{Error, Result};

use super::config::BuilderParts;
use super::{
    BlockingAppApi, BlockingRawApi, BlockingTypedApi, ClientBuilder, ClientObservability,
    RequestDefaults, emit_client_metric,
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

    pub fn call_method<R, P>(&self, method: &str, payload: &P) -> Result<R>
    where
        R: DeserializeOwned,
        P: Serialize + ?Sized,
    {
        let token = self.require_token()?;
        #[cfg(feature = "tracing")]
        let _span = tracing::debug_span!("tele.client.request", method).entered();
        let started_at = Instant::now();
        let result =
            self.inner
                .transport
                .execute_json(method, token, payload, &self.inner.defaults);
        self.emit_metric(method, started_at.elapsed(), &result);
        result
    }

    pub fn call_method_no_params<R>(&self, method: &str) -> Result<R>
    where
        R: DeserializeOwned,
    {
        let token = self.require_token()?;
        #[cfg(feature = "tracing")]
        let _span = tracing::debug_span!("tele.client.request", method).entered();
        let started_at = Instant::now();
        let result = self
            .inner
            .transport
            .execute_empty(method, token, &self.inner.defaults);
        self.emit_metric(method, started_at.elapsed(), &result);
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
        let token = self.require_token()?;
        let fields = serialize_multipart_fields(payload, &[file_field_name])?;
        #[cfg(feature = "tracing")]
        let _span = tracing::debug_span!("tele.client.request", method).entered();
        let started_at = Instant::now();
        let result = self.inner.transport.execute_multipart(
            method,
            token,
            &fields,
            file_field_name,
            file,
            &self.inner.defaults,
        );
        self.emit_metric(method, started_at.elapsed(), &result);
        result
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

use std::sync::Arc;

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
use super::{BlockingErgoApi, BlockingRawApi, BlockingTypedApi, ClientBuilder, RequestDefaults};

#[derive(Clone)]
pub struct BlockingClient {
    inner: Arc<Inner>,
}

struct Inner {
    auth: Auth,
    defaults: RequestDefaults,
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
        } = builder.into_parts();

        let transport = BlockingTransport::new(base_url, &defaults, &default_headers)?;

        Ok(Self {
            inner: Arc::new(Inner {
                auth,
                defaults,
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

    /// High-level ergonomic helpers for common bot workflows.
    pub fn ergo(&self) -> BlockingErgoApi {
        BlockingErgoApi::new(self.clone())
    }

    pub fn call_method<R, P>(&self, method: &str, payload: &P) -> Result<R>
    where
        R: DeserializeOwned,
        P: Serialize + ?Sized,
    {
        let token = self.require_token()?;
        self.inner
            .transport
            .execute_json(method, token, payload, &self.inner.defaults)
    }

    pub fn call_method_no_params<R>(&self, method: &str) -> Result<R>
    where
        R: DeserializeOwned,
    {
        let token = self.require_token()?;
        self.inner
            .transport
            .execute_empty(method, token, &self.inner.defaults)
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
        self.inner.transport.execute_multipart(
            method,
            token,
            &fields,
            file_field_name,
            file,
            &self.inner.defaults,
        )
    }

    fn require_token(&self) -> Result<&str> {
        self.inner.auth.token().ok_or(Error::MissingBotToken)
    }
}

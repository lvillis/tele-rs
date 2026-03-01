use std::sync::Arc;

use serde::Serialize;
use serde::de::DeserializeOwned;

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
use super::{ClientBuilder, ErgoApi, RawApi, RequestDefaults, TypedApi};

#[derive(Clone)]
pub struct Client {
    inner: Arc<Inner>,
}

struct Inner {
    auth: Auth,
    defaults: RequestDefaults,
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
        } = builder.into_parts();

        let transport = AsyncTransport::new(base_url, &defaults, &default_headers)?;

        Ok(Self {
            inner: Arc::new(Inner {
                auth,
                defaults,
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

    /// High-level ergonomic helpers for common bot workflows.
    pub fn ergo(&self) -> ErgoApi {
        ErgoApi::new(self.clone())
    }

    pub async fn call_method<R, P>(&self, method: &str, payload: &P) -> Result<R>
    where
        R: DeserializeOwned,
        P: Serialize + ?Sized,
    {
        let token = self.require_token()?;
        self.inner
            .transport
            .execute_json(method, token, payload, &self.inner.defaults)
            .await
    }

    pub async fn call_method_no_params<R>(&self, method: &str) -> Result<R>
    where
        R: DeserializeOwned,
    {
        let token = self.require_token()?;
        self.inner
            .transport
            .execute_empty(method, token, &self.inner.defaults)
            .await
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
        self.inner
            .transport
            .execute_multipart(
                method,
                token,
                &fields,
                file_field_name,
                file,
                &self.inner.defaults,
            )
            .await
    }

    fn require_token(&self) -> Result<&str> {
        self.inner.auth.token().ok_or(Error::MissingBotToken)
    }
}

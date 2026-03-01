use crate::Result;
use crate::types::update::{
    AnswerCallbackQueryRequest, AnswerInlineQueryRequest, GetUpdatesRequest, Update,
};
use crate::types::upload::UploadFile;
use crate::types::webhook::{DeleteWebhookRequest, SetWebhookRequest, WebhookInfo};

#[cfg(feature = "blocking")]
use crate::BlockingClient;
#[cfg(feature = "async")]
use crate::Client;

/// Update polling and webhook methods.
#[cfg(feature = "async")]
#[derive(Clone)]
pub struct UpdatesService {
    client: Client,
}

#[cfg(feature = "async")]
impl UpdatesService {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Calls `getUpdates`.
    pub async fn get_updates(&self, request: &GetUpdatesRequest) -> Result<Vec<Update>> {
        self.client.call_method("getUpdates", request).await
    }

    /// Calls `setWebhook`.
    pub async fn set_webhook(&self, request: &SetWebhookRequest) -> Result<bool> {
        self.client.call_method("setWebhook", request).await
    }

    /// Calls `setWebhook` using multipart upload for certificate.
    pub async fn set_webhook_with_certificate(
        &self,
        request: &SetWebhookRequest,
        certificate: &UploadFile,
    ) -> Result<bool> {
        self.client
            .call_method_multipart("setWebhook", request, "certificate", certificate)
            .await
    }

    /// Calls `deleteWebhook`.
    pub async fn delete_webhook(&self, request: &DeleteWebhookRequest) -> Result<bool> {
        self.client.call_method("deleteWebhook", request).await
    }

    /// Calls `getWebhookInfo`.
    pub async fn get_webhook_info(&self) -> Result<WebhookInfo> {
        self.client.call_method_no_params("getWebhookInfo").await
    }

    /// Calls `answerCallbackQuery`.
    pub async fn answer_callback_query(
        &self,
        request: &AnswerCallbackQueryRequest,
    ) -> Result<bool> {
        self.client
            .call_method("answerCallbackQuery", request)
            .await
    }

    /// Calls `answerInlineQuery`.
    pub async fn answer_inline_query(&self, request: &AnswerInlineQueryRequest) -> Result<bool> {
        self.client.call_method("answerInlineQuery", request).await
    }
}

/// Blocking update and webhook methods.
#[cfg(feature = "blocking")]
#[derive(Clone)]
pub struct BlockingUpdatesService {
    client: BlockingClient,
}

#[cfg(feature = "blocking")]
impl BlockingUpdatesService {
    pub(crate) fn new(client: BlockingClient) -> Self {
        Self { client }
    }

    /// Calls `getUpdates`.
    pub fn get_updates(&self, request: &GetUpdatesRequest) -> Result<Vec<Update>> {
        self.client.call_method("getUpdates", request)
    }

    /// Calls `setWebhook`.
    pub fn set_webhook(&self, request: &SetWebhookRequest) -> Result<bool> {
        self.client.call_method("setWebhook", request)
    }

    /// Calls `setWebhook` using multipart upload for certificate.
    pub fn set_webhook_with_certificate(
        &self,
        request: &SetWebhookRequest,
        certificate: &UploadFile,
    ) -> Result<bool> {
        self.client
            .call_method_multipart("setWebhook", request, "certificate", certificate)
    }

    /// Calls `deleteWebhook`.
    pub fn delete_webhook(&self, request: &DeleteWebhookRequest) -> Result<bool> {
        self.client.call_method("deleteWebhook", request)
    }

    /// Calls `getWebhookInfo`.
    pub fn get_webhook_info(&self) -> Result<WebhookInfo> {
        self.client.call_method_no_params("getWebhookInfo")
    }

    /// Calls `answerCallbackQuery`.
    pub fn answer_callback_query(&self, request: &AnswerCallbackQueryRequest) -> Result<bool> {
        self.client.call_method("answerCallbackQuery", request)
    }

    /// Calls `answerInlineQuery`.
    pub fn answer_inline_query(&self, request: &AnswerInlineQueryRequest) -> Result<bool> {
        self.client.call_method("answerInlineQuery", request)
    }
}

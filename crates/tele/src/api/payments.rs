use crate::Result;
use crate::types::message::Message;
use crate::types::payment::{
    AnswerPreCheckoutQueryRequest, AnswerShippingQueryRequest, CreateInvoiceLinkRequest,
    SendInvoiceRequest,
};

#[cfg(feature = "blocking")]
use crate::BlockingClient;
#[cfg(feature = "async")]
use crate::Client;

/// Payments and invoices methods.
#[cfg(feature = "async")]
#[derive(Clone)]
pub struct PaymentsService {
    client: Client,
}

#[cfg(feature = "async")]
impl PaymentsService {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Calls `sendInvoice`.
    pub async fn send_invoice(&self, request: &SendInvoiceRequest) -> Result<Message> {
        request.validate()?;
        self.client.call_method("sendInvoice", request).await
    }

    /// Calls `createInvoiceLink`.
    pub async fn create_invoice_link(&self, request: &CreateInvoiceLinkRequest) -> Result<String> {
        request.validate()?;
        self.client.call_method("createInvoiceLink", request).await
    }

    /// Calls `answerShippingQuery`.
    pub async fn answer_shipping_query(
        &self,
        request: &AnswerShippingQueryRequest,
    ) -> Result<bool> {
        request.validate()?;
        self.client
            .call_method("answerShippingQuery", request)
            .await
    }

    /// Calls `answerPreCheckoutQuery`.
    pub async fn answer_pre_checkout_query(
        &self,
        request: &AnswerPreCheckoutQueryRequest,
    ) -> Result<bool> {
        request.validate()?;
        self.client
            .call_method("answerPreCheckoutQuery", request)
            .await
    }
}

/// Blocking payments and invoices methods.
#[cfg(feature = "blocking")]
#[derive(Clone)]
pub struct BlockingPaymentsService {
    client: BlockingClient,
}

#[cfg(feature = "blocking")]
impl BlockingPaymentsService {
    pub(crate) fn new(client: BlockingClient) -> Self {
        Self { client }
    }

    /// Calls `sendInvoice`.
    pub fn send_invoice(&self, request: &SendInvoiceRequest) -> Result<Message> {
        request.validate()?;
        self.client.call_method("sendInvoice", request)
    }

    /// Calls `createInvoiceLink`.
    pub fn create_invoice_link(&self, request: &CreateInvoiceLinkRequest) -> Result<String> {
        request.validate()?;
        self.client.call_method("createInvoiceLink", request)
    }

    /// Calls `answerShippingQuery`.
    pub fn answer_shipping_query(&self, request: &AnswerShippingQueryRequest) -> Result<bool> {
        request.validate()?;
        self.client.call_method("answerShippingQuery", request)
    }

    /// Calls `answerPreCheckoutQuery`.
    pub fn answer_pre_checkout_query(
        &self,
        request: &AnswerPreCheckoutQueryRequest,
    ) -> Result<bool> {
        request.validate()?;
        self.client.call_method("answerPreCheckoutQuery", request)
    }
}

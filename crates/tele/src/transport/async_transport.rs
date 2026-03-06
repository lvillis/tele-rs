use http::header::{CONTENT_LENGTH, CONTENT_TYPE, HeaderValue};
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::client::RequestDefaults;
use crate::transport::{
    PreparedTelegramCall, build_multipart_payload, build_transport_client, multipart_header_values,
};
use crate::types::upload::UploadFile;
use crate::{Error, Result};

pub(crate) struct AsyncTransport {
    client: reqx::Client,
}

impl AsyncTransport {
    pub(crate) fn new(
        base_url: String,
        defaults: &RequestDefaults,
        default_headers: &[(String, String)],
    ) -> Result<Self> {
        let client =
            build_transport_client(reqx::Client::builder(base_url), defaults, default_headers)?;
        Ok(Self { client })
    }

    pub(crate) async fn execute_json<P, R>(
        &self,
        method: &str,
        token: &str,
        payload: &P,
        defaults: &RequestDefaults,
    ) -> Result<R>
    where
        P: Serialize + ?Sized,
        R: DeserializeOwned,
    {
        let call = PreparedTelegramCall::new(method, token)?;
        let body =
            serde_json::to_vec(payload).map_err(|source| Error::SerializeRequest { source })?;

        let request = self.configure_request(self.client.post(call.path()), defaults);
        let response = request
            .header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
            .body(body)
            .send()
            .await
            .map_err(|source| call.map_transport_error(source))?;

        call.parse_response(response, defaults)
    }

    pub(crate) async fn execute_multipart<R>(
        &self,
        method: &str,
        token: &str,
        fields: &[(String, String)],
        file_field_name: &str,
        file: &UploadFile,
        defaults: &RequestDefaults,
    ) -> Result<R>
    where
        R: DeserializeOwned,
    {
        let call = PreparedTelegramCall::new(method, token)?;
        let payload = build_multipart_payload(fields, file_field_name, file);
        let (content_type, content_length) = multipart_header_values(&payload)?;

        let request = self.configure_request(self.client.post(call.path()), defaults);
        let response = request
            .header(CONTENT_TYPE, content_type)
            .header(CONTENT_LENGTH, content_length)
            .body_stream(payload.into_stream())
            .send()
            .await
            .map_err(|source| call.map_transport_error(source))?;

        call.parse_response(response, defaults)
    }

    pub(crate) async fn execute_empty<R>(
        &self,
        method: &str,
        token: &str,
        defaults: &RequestDefaults,
    ) -> Result<R>
    where
        R: DeserializeOwned,
    {
        let call = PreparedTelegramCall::new(method, token)?;

        let response = self
            .configure_request(self.client.post(call.path()), defaults)
            .send()
            .await
            .map_err(|source| call.map_transport_error(source))?;

        call.parse_response(response, defaults)
    }

    fn configure_request<'a>(
        &self,
        mut request: reqx::RequestBuilder<'a>,
        defaults: &RequestDefaults,
    ) -> reqx::RequestBuilder<'a> {
        request = request
            .timeout(defaults.request_timeout)
            .max_response_body_bytes(defaults.max_response_body_bytes);

        if let Some(total_timeout) = defaults.total_timeout {
            request = request.total_timeout(total_timeout);
        }

        request
    }
}

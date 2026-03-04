use http::Method;
use http::header::{CONTENT_LENGTH, CONTENT_TYPE, HeaderValue};
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::client::RequestDefaults;
use crate::transport::{
    build_multipart_payload, map_reqx_builder_error, map_reqx_error, parse_telegram_response,
    to_rate_limit_policy, to_retry_policy,
};
use crate::types::upload::UploadFile;
use crate::util::{build_api_path, validate_method_name};
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
        let mut builder = reqx::Client::builder(base_url)
            .request_timeout(defaults.request_timeout)
            .connect_timeout(defaults.connect_timeout)
            .max_response_body_bytes(defaults.max_response_body_bytes)
            .default_status_policy(reqx::StatusPolicy::Response)
            .redirect_policy(reqx::RedirectPolicy::none())
            .retry_policy(to_retry_policy(&defaults.retry))
            .client_name("tele");

        if let Some(total_timeout) = defaults.total_timeout {
            builder = builder.total_timeout(total_timeout);
        }

        if let Some(max_in_flight) = defaults.max_in_flight {
            builder = builder.max_in_flight(max_in_flight);
        }

        if let Some(max_in_flight_per_host) = defaults.max_in_flight_per_host {
            builder = builder.max_in_flight_per_host(max_in_flight_per_host);
        }

        if let Some(global_rate_limit) = defaults.global_rate_limit.as_ref() {
            builder = builder.global_rate_limit_policy(to_rate_limit_policy(global_rate_limit));
        }

        if let Some(per_host_rate_limit) = defaults.per_host_rate_limit.as_ref() {
            builder = builder.per_host_rate_limit_policy(to_rate_limit_policy(per_host_rate_limit));
        }

        if defaults.retry.allow_non_idempotent_retries {
            builder = builder.allow_non_idempotent_retries(true);
        }

        if let Some(http_proxy) = defaults.http_proxy.clone() {
            builder = builder.http_proxy(http_proxy);
        }

        if let Some(proxy_authorization) = defaults.proxy_authorization.clone() {
            builder = builder.proxy_authorization(proxy_authorization);
        }

        if !defaults.no_proxy_rules.is_empty() {
            builder = builder.no_proxy(defaults.no_proxy_rules.iter().map(String::as_str));
        }

        for (name, value) in default_headers {
            builder = builder
                .try_default_header(name, value)
                .map_err(map_reqx_builder_error)?;
        }

        let client = builder.build().map_err(map_reqx_builder_error)?;

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
        validate_method_name(method)?;
        let path = build_api_path(token, method);
        let body =
            serde_json::to_vec(payload).map_err(|source| Error::SerializeRequest { source })?;

        let request = self.configure_request(self.client.request(Method::POST, path), defaults);
        let response = request
            .header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
            .body(body)
            .send_with_status()
            .await
            .map_err(|source| map_reqx_error(method, token, source))?;

        parse_telegram_response(
            method,
            response.status(),
            response.headers(),
            response.body(),
            defaults.capture_body_snippet,
            defaults.body_snippet_limit,
        )
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
        validate_method_name(method)?;
        let path = build_api_path(token, method);
        let payload = build_multipart_payload(fields, file_field_name, file);

        let content_type = HeaderValue::from_str(payload.content_type()).map_err(|source| {
            Error::InvalidHeaderValue {
                name: CONTENT_TYPE.as_str().to_owned(),
                source,
            }
        })?;
        let content_length =
            HeaderValue::from_str(&payload.content_length().to_string()).map_err(|source| {
                Error::InvalidHeaderValue {
                    name: CONTENT_LENGTH.as_str().to_owned(),
                    source,
                }
            })?;

        let request = self.configure_request(self.client.request(Method::POST, path), defaults);
        let response = request
            .header(CONTENT_TYPE, content_type)
            .header(CONTENT_LENGTH, content_length)
            .body_stream(payload.into_stream())
            .send_with_status()
            .await
            .map_err(|source| map_reqx_error(method, token, source))?;

        parse_telegram_response(
            method,
            response.status(),
            response.headers(),
            response.body(),
            defaults.capture_body_snippet,
            defaults.body_snippet_limit,
        )
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
        validate_method_name(method)?;
        let path = build_api_path(token, method);

        let response = self
            .configure_request(self.client.request(Method::POST, path), defaults)
            .send_with_status()
            .await
            .map_err(|source| map_reqx_error(method, token, source))?;

        parse_telegram_response(
            method,
            response.status(),
            response.headers(),
            response.body(),
            defaults.capture_body_snippet,
            defaults.body_snippet_limit,
        )
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

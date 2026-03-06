#[cfg(feature = "_async")]
pub(crate) mod async_transport;
#[cfg(feature = "_blocking")]
pub(crate) mod blocking_transport;

use std::collections::BTreeSet;
use std::io;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[cfg(any(feature = "_blocking", test))]
use std::io::Read;

#[cfg(feature = "_async")]
use bytes::Bytes;
#[cfg(feature = "_async")]
use futures_core::Stream;
use http::header::{CONTENT_LENGTH, CONTENT_TYPE, HeaderValue};
use http::{HeaderMap, StatusCode};
use reqx::advanced::RateLimitPolicy;
use reqx::prelude::{RedirectPolicy, RetryPolicy, StatusPolicy};
use serde::Deserialize;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::Error;
use crate::client::{RateLimitConfig, RequestDefaults, RetryConfig};
use crate::types::common::ResponseParameters;
use crate::types::upload::UploadFile;
use crate::util::{
    body_snippet, build_api_path, redact_token, request_id_from_headers, validate_method_name,
};

static LOCAL_REQUEST_ID_SEQ: AtomicU64 = AtomicU64::new(1);

fn is_configuration_error_code(code: reqx::ErrorCode) -> bool {
    matches!(
        code,
        reqx::ErrorCode::InvalidUri
            | reqx::ErrorCode::InvalidNoProxyRule
            | reqx::ErrorCode::InvalidProxyConfig
            | reqx::ErrorCode::InvalidAdaptiveConcurrencyPolicy
            | reqx::ErrorCode::RequestBuild
            | reqx::ErrorCode::InvalidHeaderName
            | reqx::ErrorCode::InvalidHeaderValue
            | reqx::ErrorCode::TlsBackendUnavailable
            | reqx::ErrorCode::TlsBackendInit
            | reqx::ErrorCode::TlsConfig
    )
}

#[derive(Debug, Deserialize)]
struct TelegramEnvelope<T> {
    ok: bool,
    result: Option<T>,
    description: Option<String>,
    error_code: Option<i64>,
    parameters: Option<ResponseParameters>,
}

pub(crate) fn parse_telegram_response<T>(
    method: &str,
    status: StatusCode,
    headers: &HeaderMap,
    body: &[u8],
    capture_body_snippet: bool,
    snippet_limit: usize,
) -> Result<T, Error>
where
    T: DeserializeOwned,
{
    let request_id = request_id_from_headers(headers);
    let snippet = capture_body_snippet
        .then(|| body_snippet(body, snippet_limit))
        .flatten();

    let envelope: TelegramEnvelope<T> =
        serde_json::from_slice(body).map_err(|source| Error::DeserializeResponse {
            method: method.to_owned(),
            status: Some(status.as_u16()),
            request_id: request_id.clone().map(Into::into),
            body_snippet: snippet.clone().map(Into::into),
            source,
        })?;

    if envelope.ok {
        if let Some(result) = envelope.result {
            return Ok(result);
        }

        return Err(Error::MissingResult {
            method: method.to_owned(),
            status: Some(status.as_u16()),
            request_id: request_id.map(Into::into),
            body_snippet: snippet.map(Into::into),
        });
    }

    Err(Error::Api {
        method: method.to_owned(),
        status: Some(status.as_u16()),
        request_id: request_id.map(Into::into),
        error_code: envelope.error_code,
        description: envelope
            .description
            .unwrap_or_else(|| "telegram api returned an unknown error".to_owned())
            .into(),
        parameters: envelope.parameters.map(Box::new),
        body_snippet: snippet.map(Into::into),
    })
}

pub(crate) fn local_transport_request_id(method: &str) -> String {
    let sequence = LOCAL_REQUEST_ID_SEQ.fetch_add(1, Ordering::Relaxed);
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0_u128, |duration| duration.as_millis());
    format!("tele-{method}-{timestamp:x}-{sequence:x}")
}

pub(crate) fn map_reqx_error(
    method: &str,
    token: &str,
    source: reqx::Error,
    fallback_request_id: Option<String>,
) -> Error {
    let code = source.code();
    if is_configuration_error_code(code) {
        return Error::Configuration {
            reason: format!(
                "while calling `{method}`: {} [{}]",
                redact_token(&source.to_string(), token),
                code.as_str()
            ),
        };
    }

    let request_path = source.request_path().map(|path| redact_token(&path, token));

    Error::Transport {
        method: method.to_owned(),
        status: source.status_code(),
        request_id: source
            .request_id()
            .map(ToOwned::to_owned)
            .or(fallback_request_id)
            .map(Into::into),
        retry_after: source.retry_after(SystemTime::now()),
        request_path: request_path.map(Into::into),
        message: redact_token(&source.to_string(), token).into(),
    }
}

pub(crate) fn map_reqx_builder_error(source: reqx::Error) -> Error {
    Error::Configuration {
        reason: format!(
            "failed to build HTTP client: {} [{}]",
            source,
            source.code().as_str()
        ),
    }
}

pub(crate) fn to_retry_policy(retry: &RetryConfig) -> RetryPolicy {
    RetryPolicy::standard()
        .max_attempts(retry.max_attempts)
        .base_backoff(retry.base_backoff)
        .max_backoff(retry.max_backoff)
        .jitter_ratio(retry.jitter_ratio)
}

pub(crate) fn to_rate_limit_policy(config: &RateLimitConfig) -> RateLimitPolicy {
    RateLimitPolicy::standard()
        .requests_per_second(config.requests_per_second)
        .burst(config.burst)
        .max_throttle_delay(config.max_throttle_delay)
}

trait TransportClientBuilder: Sized {
    type Client;

    fn request_timeout(self, timeout: Duration) -> Self;
    fn connect_timeout(self, timeout: Duration) -> Self;
    fn max_response_body_bytes(self, max_response_body_bytes: usize) -> Self;
    fn default_status_policy(self, default_status_policy: StatusPolicy) -> Self;
    fn redirect_policy(self, redirect_policy: RedirectPolicy) -> Self;
    fn retry_policy(self, retry_policy: RetryPolicy) -> Self;
    fn client_name(self, client_name: &'static str) -> Self;
    fn total_timeout(self, total_timeout: Duration) -> Self;
    fn max_in_flight(self, max_in_flight: usize) -> Self;
    fn max_in_flight_per_host(self, max_in_flight_per_host: usize) -> Self;
    fn global_rate_limit_policy(self, global_rate_limit_policy: RateLimitPolicy) -> Self;
    fn per_host_rate_limit_policy(self, per_host_rate_limit_policy: RateLimitPolicy) -> Self;
    fn allow_non_idempotent_retries(self, enabled: bool) -> Self;
    fn http_proxy(self, proxy_uri: http::Uri) -> Self;
    fn proxy_authorization(self, proxy_authorization: HeaderValue) -> Self;
    fn try_add_no_proxy(self, rule: &str) -> reqx::Result<Self>;
    fn try_default_header(self, name: &str, value: &str) -> reqx::Result<Self>;
    fn build(self) -> reqx::Result<Self::Client>;
}

#[cfg(feature = "_async")]
impl TransportClientBuilder for reqx::ClientBuilder {
    type Client = reqx::Client;

    fn request_timeout(self, timeout: Duration) -> Self {
        reqx::ClientBuilder::request_timeout(self, timeout)
    }

    fn connect_timeout(self, timeout: Duration) -> Self {
        reqx::ClientBuilder::connect_timeout(self, timeout)
    }

    fn max_response_body_bytes(self, max_response_body_bytes: usize) -> Self {
        reqx::ClientBuilder::max_response_body_bytes(self, max_response_body_bytes)
    }

    fn default_status_policy(self, default_status_policy: StatusPolicy) -> Self {
        reqx::ClientBuilder::default_status_policy(self, default_status_policy)
    }

    fn redirect_policy(self, redirect_policy: RedirectPolicy) -> Self {
        reqx::ClientBuilder::redirect_policy(self, redirect_policy)
    }

    fn retry_policy(self, retry_policy: RetryPolicy) -> Self {
        reqx::ClientBuilder::retry_policy(self, retry_policy)
    }

    fn client_name(self, client_name: &'static str) -> Self {
        reqx::ClientBuilder::client_name(self, client_name)
    }

    fn total_timeout(self, total_timeout: Duration) -> Self {
        reqx::ClientBuilder::total_timeout(self, total_timeout)
    }

    fn max_in_flight(self, max_in_flight: usize) -> Self {
        reqx::ClientBuilder::max_in_flight(self, max_in_flight)
    }

    fn max_in_flight_per_host(self, max_in_flight_per_host: usize) -> Self {
        reqx::ClientBuilder::max_in_flight_per_host(self, max_in_flight_per_host)
    }

    fn global_rate_limit_policy(self, global_rate_limit_policy: RateLimitPolicy) -> Self {
        reqx::ClientBuilder::global_rate_limit_policy(self, global_rate_limit_policy)
    }

    fn per_host_rate_limit_policy(self, per_host_rate_limit_policy: RateLimitPolicy) -> Self {
        reqx::ClientBuilder::per_host_rate_limit_policy(self, per_host_rate_limit_policy)
    }

    fn allow_non_idempotent_retries(self, enabled: bool) -> Self {
        reqx::ClientBuilder::allow_non_idempotent_retries(self, enabled)
    }

    fn http_proxy(self, proxy_uri: http::Uri) -> Self {
        reqx::ClientBuilder::http_proxy(self, proxy_uri)
    }

    fn proxy_authorization(self, proxy_authorization: HeaderValue) -> Self {
        reqx::ClientBuilder::proxy_authorization(self, proxy_authorization)
    }

    fn try_add_no_proxy(self, rule: &str) -> reqx::Result<Self> {
        reqx::ClientBuilder::try_add_no_proxy(self, rule)
    }

    fn try_default_header(self, name: &str, value: &str) -> reqx::Result<Self> {
        reqx::ClientBuilder::try_default_header(self, name, value)
    }

    fn build(self) -> reqx::Result<Self::Client> {
        reqx::ClientBuilder::build(self)
    }
}

#[cfg(feature = "_blocking")]
impl TransportClientBuilder for reqx::blocking::ClientBuilder {
    type Client = reqx::blocking::Client;

    fn request_timeout(self, timeout: Duration) -> Self {
        reqx::blocking::ClientBuilder::request_timeout(self, timeout)
    }

    fn connect_timeout(self, timeout: Duration) -> Self {
        reqx::blocking::ClientBuilder::connect_timeout(self, timeout)
    }

    fn max_response_body_bytes(self, max_response_body_bytes: usize) -> Self {
        reqx::blocking::ClientBuilder::max_response_body_bytes(self, max_response_body_bytes)
    }

    fn default_status_policy(self, default_status_policy: StatusPolicy) -> Self {
        reqx::blocking::ClientBuilder::default_status_policy(self, default_status_policy)
    }

    fn redirect_policy(self, redirect_policy: RedirectPolicy) -> Self {
        reqx::blocking::ClientBuilder::redirect_policy(self, redirect_policy)
    }

    fn retry_policy(self, retry_policy: RetryPolicy) -> Self {
        reqx::blocking::ClientBuilder::retry_policy(self, retry_policy)
    }

    fn client_name(self, client_name: &'static str) -> Self {
        reqx::blocking::ClientBuilder::client_name(self, client_name)
    }

    fn total_timeout(self, total_timeout: Duration) -> Self {
        reqx::blocking::ClientBuilder::total_timeout(self, total_timeout)
    }

    fn max_in_flight(self, max_in_flight: usize) -> Self {
        reqx::blocking::ClientBuilder::max_in_flight(self, max_in_flight)
    }

    fn max_in_flight_per_host(self, max_in_flight_per_host: usize) -> Self {
        reqx::blocking::ClientBuilder::max_in_flight_per_host(self, max_in_flight_per_host)
    }

    fn global_rate_limit_policy(self, global_rate_limit_policy: RateLimitPolicy) -> Self {
        reqx::blocking::ClientBuilder::global_rate_limit_policy(self, global_rate_limit_policy)
    }

    fn per_host_rate_limit_policy(self, per_host_rate_limit_policy: RateLimitPolicy) -> Self {
        reqx::blocking::ClientBuilder::per_host_rate_limit_policy(self, per_host_rate_limit_policy)
    }

    fn allow_non_idempotent_retries(self, enabled: bool) -> Self {
        reqx::blocking::ClientBuilder::allow_non_idempotent_retries(self, enabled)
    }

    fn http_proxy(self, proxy_uri: http::Uri) -> Self {
        reqx::blocking::ClientBuilder::http_proxy(self, proxy_uri)
    }

    fn proxy_authorization(self, proxy_authorization: HeaderValue) -> Self {
        reqx::blocking::ClientBuilder::proxy_authorization(self, proxy_authorization)
    }

    fn try_add_no_proxy(self, rule: &str) -> reqx::Result<Self> {
        reqx::blocking::ClientBuilder::try_add_no_proxy(self, rule)
    }

    fn try_default_header(self, name: &str, value: &str) -> reqx::Result<Self> {
        reqx::blocking::ClientBuilder::try_default_header(self, name, value)
    }

    fn build(self) -> reqx::Result<Self::Client> {
        reqx::blocking::ClientBuilder::build(self)
    }
}

fn build_transport_client<B>(
    builder: B,
    defaults: &RequestDefaults,
    default_headers: &[(String, String)],
) -> Result<B::Client, Error>
where
    B: TransportClientBuilder,
{
    let mut builder = builder
        .request_timeout(defaults.request_timeout)
        .connect_timeout(defaults.connect_timeout)
        .max_response_body_bytes(defaults.max_response_body_bytes)
        .default_status_policy(StatusPolicy::Response)
        .redirect_policy(RedirectPolicy::none())
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

    for rule in &defaults.no_proxy_rules {
        builder = builder
            .try_add_no_proxy(rule)
            .map_err(map_reqx_builder_error)?;
    }

    for (name, value) in default_headers {
        builder = builder
            .try_default_header(name, value)
            .map_err(map_reqx_builder_error)?;
    }

    builder.build().map_err(map_reqx_builder_error)
}

pub(crate) struct PreparedTelegramCall<'a> {
    method: &'a str,
    token: &'a str,
    path: String,
    fallback_request_id: String,
}

impl<'a> PreparedTelegramCall<'a> {
    pub(crate) fn new(method: &'a str, token: &'a str) -> Result<Self, Error> {
        validate_method_name(method)?;

        Ok(Self {
            method,
            token,
            path: build_api_path(token, method),
            fallback_request_id: local_transport_request_id(method),
        })
    }

    pub(crate) fn path(&self) -> &str {
        &self.path
    }

    pub(crate) fn map_transport_error(&self, source: reqx::Error) -> Error {
        map_reqx_error(
            self.method,
            self.token,
            source,
            Some(self.fallback_request_id.clone()),
        )
    }

    pub(crate) fn parse_response<R>(
        &self,
        response: reqx::Response,
        defaults: &RequestDefaults,
    ) -> Result<R, Error>
    where
        R: DeserializeOwned,
    {
        parse_telegram_response(
            self.method,
            response.status(),
            response.headers(),
            response.body(),
            defaults.capture_body_snippet,
            defaults.body_snippet_limit,
        )
    }
}

pub(crate) fn multipart_header_values(
    payload: &MultipartPayload,
) -> Result<(HeaderValue, HeaderValue), Error> {
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

    Ok((content_type, content_length))
}

pub(crate) fn serialize_multipart_fields<P>(
    payload: &P,
    skip_fields: &[&str],
) -> Result<Vec<(String, String)>, Error>
where
    P: Serialize + ?Sized,
{
    let skip: BTreeSet<&str> = skip_fields.iter().copied().collect();

    let value =
        serde_json::to_value(payload).map_err(|source| Error::SerializeRequest { source })?;
    let object = value.as_object().ok_or_else(|| Error::InvalidRequest {
        reason: "multipart payload must serialize into an object".to_owned(),
    })?;

    let mut fields = Vec::new();
    for (key, value) in object {
        if skip.contains(key.as_str()) {
            continue;
        }

        if let Some(value) = value_to_form_string(value)? {
            fields.push((key.clone(), value));
        }
    }

    Ok(fields)
}

fn value_to_form_string(value: &Value) -> Result<Option<String>, Error> {
    match value {
        Value::Null => Ok(None),
        Value::String(value) => Ok(Some(value.clone())),
        Value::Bool(value) => Ok(Some(value.to_string())),
        Value::Number(value) => Ok(Some(value.to_string())),
        Value::Array(_) | Value::Object(_) => {
            let text = serde_json::to_string(value)
                .map_err(|source| Error::SerializeRequest { source })?;
            Ok(Some(text))
        }
    }
}

enum MultipartChunk {
    Owned(Vec<u8>),
    Shared(Arc<[u8]>),
}

impl MultipartChunk {
    fn len(&self) -> usize {
        match self {
            Self::Owned(bytes) => bytes.len(),
            Self::Shared(bytes) => bytes.len(),
        }
    }

    #[cfg(any(feature = "_blocking", test))]
    fn as_slice(&self) -> &[u8] {
        match self {
            Self::Owned(bytes) => bytes,
            Self::Shared(bytes) => bytes,
        }
    }

    #[cfg(feature = "_async")]
    fn into_bytes(self) -> Bytes {
        match self {
            Self::Owned(bytes) => Bytes::from(bytes),
            Self::Shared(bytes) => Bytes::from_owner(bytes),
        }
    }
}

pub(crate) struct MultipartPayload {
    chunks: Vec<MultipartChunk>,
    content_type: String,
    content_length: usize,
}

impl MultipartPayload {
    pub(crate) fn content_type(&self) -> &str {
        &self.content_type
    }

    pub(crate) fn content_length(&self) -> usize {
        self.content_length
    }

    #[cfg(any(feature = "_blocking", test))]
    pub(crate) fn into_reader(self) -> MultipartBodyReader {
        MultipartBodyReader::new(self.chunks)
    }

    #[cfg(feature = "_async")]
    pub(crate) fn into_stream(self) -> MultipartBodyStream {
        MultipartBodyStream::new(self.chunks)
    }
}

#[cfg(any(feature = "_blocking", test))]
pub(crate) struct MultipartBodyReader {
    chunks: Vec<MultipartChunk>,
    chunk_index: usize,
    chunk_offset: usize,
}

#[cfg(any(feature = "_blocking", test))]
impl MultipartBodyReader {
    fn new(chunks: Vec<MultipartChunk>) -> Self {
        Self {
            chunks,
            chunk_index: 0,
            chunk_offset: 0,
        }
    }
}

#[cfg(any(feature = "_blocking", test))]
impl Read for MultipartBodyReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }

        let mut written = 0;

        while written < buf.len() {
            if self.chunk_index >= self.chunks.len() {
                break;
            }

            let chunk = &self.chunks[self.chunk_index];
            let chunk_bytes = chunk.as_slice();
            let remaining = &chunk_bytes[self.chunk_offset..];

            if remaining.is_empty() {
                self.chunk_index += 1;
                self.chunk_offset = 0;
                continue;
            }

            let to_copy = remaining.len().min(buf.len() - written);
            buf[written..written + to_copy].copy_from_slice(&remaining[..to_copy]);
            written += to_copy;
            self.chunk_offset += to_copy;

            if self.chunk_offset >= chunk.len() {
                self.chunk_index += 1;
                self.chunk_offset = 0;
            }
        }

        Ok(written)
    }
}

#[cfg(feature = "_async")]
pub(crate) struct MultipartBodyStream {
    chunks: std::vec::IntoIter<MultipartChunk>,
}

#[cfg(feature = "_async")]
impl MultipartBodyStream {
    fn new(chunks: Vec<MultipartChunk>) -> Self {
        Self {
            chunks: chunks.into_iter(),
        }
    }
}

#[cfg(feature = "_async")]
impl Stream for MultipartBodyStream {
    type Item = Result<Bytes, io::Error>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        match self.chunks.next() {
            Some(chunk) => std::task::Poll::Ready(Some(Ok(chunk.into_bytes()))),
            None => std::task::Poll::Ready(None),
        }
    }
}

fn push_chunk(chunks: &mut Vec<MultipartChunk>, content_length: &mut usize, chunk: MultipartChunk) {
    *content_length += chunk.len();
    chunks.push(chunk);
}

pub(crate) fn build_multipart_payload(
    fields: &[(String, String)],
    file_field_name: &str,
    file: &UploadFile,
) -> MultipartPayload {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0_u128, |duration| duration.as_nanos());
    let boundary = format!("tele-sdk-boundary-{timestamp}");

    let mut chunks = Vec::new();
    let mut content_length = 0;

    for (name, value) in fields {
        push_chunk(
            &mut chunks,
            &mut content_length,
            MultipartChunk::Owned(format!("--{boundary}\r\n").into_bytes()),
        );
        push_chunk(
            &mut chunks,
            &mut content_length,
            MultipartChunk::Owned(
                format!(
                    "Content-Disposition: form-data; name=\"{}\"\r\n\r\n",
                    escape_quoted(name)
                )
                .into_bytes(),
            ),
        );
        push_chunk(
            &mut chunks,
            &mut content_length,
            MultipartChunk::Owned(value.as_bytes().to_vec()),
        );
        push_chunk(
            &mut chunks,
            &mut content_length,
            MultipartChunk::Owned(b"\r\n".to_vec()),
        );
    }

    push_chunk(
        &mut chunks,
        &mut content_length,
        MultipartChunk::Owned(format!("--{boundary}\r\n").into_bytes()),
    );
    push_chunk(
        &mut chunks,
        &mut content_length,
        MultipartChunk::Owned(
            format!(
                "Content-Disposition: form-data; name=\"{}\"; filename=\"{}\"\r\n",
                escape_quoted(file_field_name),
                escape_quoted(file.file_name())
            )
            .into_bytes(),
        ),
    );
    push_chunk(
        &mut chunks,
        &mut content_length,
        MultipartChunk::Owned(
            format!(
                "Content-Type: {}\r\n\r\n",
                file.content_type().unwrap_or("application/octet-stream")
            )
            .into_bytes(),
        ),
    );
    push_chunk(
        &mut chunks,
        &mut content_length,
        MultipartChunk::Shared(file.data_arc()),
    );
    push_chunk(
        &mut chunks,
        &mut content_length,
        MultipartChunk::Owned(b"\r\n".to_vec()),
    );
    push_chunk(
        &mut chunks,
        &mut content_length,
        MultipartChunk::Owned(format!("--{boundary}--\r\n").into_bytes()),
    );

    MultipartPayload {
        chunks,
        content_type: format!("multipart/form-data; boundary={boundary}"),
        content_length,
    }
}

fn escape_quoted(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use std::io::Read;

    use super::{build_multipart_payload, serialize_multipart_fields};
    use crate::types::upload::UploadFile;

    #[derive(serde::Serialize)]
    struct Payload {
        chat_id: i64,
        text: String,
        file: String,
    }

    #[test]
    fn serializes_multipart_fields_skipping_file_field() {
        let payload = Payload {
            chat_id: 1,
            text: "hello".to_owned(),
            file: "ignored".to_owned(),
        };

        let fields_result = serialize_multipart_fields(&payload, &["file"]);
        assert!(fields_result.is_ok());
        let fields = fields_result.unwrap_or_default();
        assert_eq!(fields.len(), 2);
        assert!(
            fields
                .iter()
                .any(|(key, value)| key == "chat_id" && value == "1")
        );
        assert!(
            fields
                .iter()
                .any(|(key, value)| key == "text" && value == "hello")
        );
    }

    #[test]
    fn builds_multipart_body_with_binary_file() {
        let file_result = UploadFile::from_bytes("hello.txt", b"abc123".to_vec());
        assert!(file_result.is_ok());
        let file = match file_result {
            Ok(value) => value,
            Err(_) => return,
        };
        let payload =
            build_multipart_payload(&[("chat_id".to_owned(), "1".to_owned())], "photo", &file);
        let content_length = payload.content_length();
        let content_type = payload.content_type().to_owned();

        let mut reader = payload.into_reader();
        let mut body = Vec::new();
        let read_result = reader.read_to_end(&mut body);
        assert!(read_result.is_ok());

        let text = String::from_utf8_lossy(&body);
        assert!(content_type.starts_with("multipart/form-data; boundary="));
        assert_eq!(content_length, body.len());
        assert!(text.contains("name=\"chat_id\""));
        assert!(text.contains("name=\"photo\"; filename=\"hello.txt\""));
        assert!(text.contains("abc123"));
    }
}

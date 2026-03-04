#[cfg(feature = "_async")]
pub(crate) mod async_transport;
#[cfg(feature = "_blocking")]
pub(crate) mod blocking_transport;

use std::collections::BTreeSet;
use std::io;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(any(feature = "_blocking", test))]
use std::io::Read;

#[cfg(feature = "_async")]
use bytes::Bytes;
#[cfg(feature = "_async")]
use futures_core::Stream;
use http::{HeaderMap, StatusCode};
use serde::Deserialize;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::Error;
use crate::client::{RateLimitConfig, RetryConfig};
use crate::types::common::ResponseParameters;
use crate::types::upload::UploadFile;
use crate::util::{body_snippet, redact_token, request_id_from_headers};

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
            request_id: request_id.clone(),
            body_snippet: snippet.clone(),
            source,
        })?;

    if envelope.ok {
        if let Some(result) = envelope.result {
            return Ok(result);
        }

        return Err(Error::MissingResult {
            method: method.to_owned(),
            status: Some(status.as_u16()),
            request_id,
            body_snippet: snippet,
        });
    }

    Err(Error::Api {
        method: method.to_owned(),
        status: Some(status.as_u16()),
        request_id,
        error_code: envelope.error_code,
        description: envelope
            .description
            .unwrap_or_else(|| "telegram api returned an unknown error".to_owned()),
        parameters: envelope.parameters,
        body_snippet: snippet,
    })
}

pub(crate) fn map_reqx_error(method: &str, token: &str, source: reqx::Error) -> Error {
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
        request_id: source.request_id().map(ToOwned::to_owned),
        retry_after: source.retry_after(SystemTime::now()),
        request_path,
        message: redact_token(&source.to_string(), token),
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

pub(crate) fn to_retry_policy(retry: &RetryConfig) -> reqx::RetryPolicy {
    reqx::RetryPolicy::standard()
        .max_attempts(retry.max_attempts)
        .base_backoff(retry.base_backoff)
        .max_backoff(retry.max_backoff)
        .jitter_ratio(retry.jitter_ratio)
}

pub(crate) fn to_rate_limit_policy(config: &RateLimitConfig) -> reqx::RateLimitPolicy {
    reqx::RateLimitPolicy::standard()
        .requests_per_second(config.requests_per_second)
        .burst(config.burst)
        .max_throttle_delay(config.max_throttle_delay)
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

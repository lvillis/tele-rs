//! Test utilities for SDK and downstream integration tests.

use std::error::Error as StdError;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// Result type used by test helpers.
pub type TestingResult<T> = std::result::Result<T, Box<dyn StdError + Send + Sync>>;

#[derive(Clone, Debug, Eq, PartialEq)]
enum TextMatchKind {
    Exact,
    CaseInsensitive,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct TextMatcher {
    needle: String,
    kind: TextMatchKind,
}

/// A recorded raw HTTP request captured by `FakeTelegramServer`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordedRequest {
    pub request_line: String,
    pub raw: String,
}

impl RecordedRequest {
    pub fn contains(&self, text: &str) -> bool {
        self.raw.contains(text)
    }

    pub fn contains_case_insensitive(&self, text: &str) -> bool {
        self.raw
            .to_ascii_lowercase()
            .contains(&text.to_ascii_lowercase())
    }
}

#[derive(Clone, Debug)]
struct ResponseSpec {
    status: u16,
    content_type: String,
    body: String,
    delay: Duration,
}

impl Default for ResponseSpec {
    fn default() -> Self {
        Self {
            status: 200,
            content_type: "application/json".to_owned(),
            body: r#"{"ok":true,"result":true}"#.to_owned(),
            delay: Duration::ZERO,
        }
    }
}

/// Declarative expectation for one incoming Telegram API request.
#[derive(Clone, Debug)]
pub struct RequestExpectation {
    path: String,
    matchers: Vec<TextMatcher>,
    response: ResponseSpec,
}

impl RequestExpectation {
    pub fn post(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            matchers: Vec::new(),
            response: ResponseSpec::default(),
        }
    }

    pub fn contains(mut self, text: impl Into<String>) -> Self {
        self.matchers.push(TextMatcher {
            needle: text.into(),
            kind: TextMatchKind::Exact,
        });
        self
    }

    pub fn contains_case_insensitive(mut self, text: impl Into<String>) -> Self {
        self.matchers.push(TextMatcher {
            needle: text.into(),
            kind: TextMatchKind::CaseInsensitive,
        });
        self
    }

    pub fn respond_json(mut self, status: u16, body: impl Into<String>) -> Self {
        self.response.status = status;
        self.response.content_type = "application/json".to_owned();
        self.response.body = body.into();
        self
    }

    pub fn delay(mut self, duration: Duration) -> Self {
        self.response.delay = duration;
        self
    }
}

/// Scriptable fake Telegram HTTP server for integration tests.
pub struct FakeTelegramServer {
    base_url: String,
    recorded_requests: Arc<Mutex<Vec<RecordedRequest>>>,
    handle: Option<thread::JoinHandle<std::result::Result<(), String>>>,
}

impl FakeTelegramServer {
    pub fn start(expectations: Vec<RequestExpectation>) -> TestingResult<Self> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let address = listener.local_addr()?;
        let recorded_requests = Arc::new(Mutex::new(Vec::new()));
        let recorded_requests_for_thread = Arc::clone(&recorded_requests);

        let handle = thread::spawn(move || {
            for expectation in expectations {
                let (mut stream, _) = accept_with_timeout(&listener, Duration::from_secs(3))?;
                stream
                    .set_read_timeout(Some(Duration::from_secs(2)))
                    .map_err(|error| error.to_string())?;

                let buffer = read_full_http_request(&mut stream)?;
                let request = String::from_utf8_lossy(&buffer).into_owned();
                let request_line = request
                    .lines()
                    .next()
                    .map(ToOwned::to_owned)
                    .unwrap_or_default();

                recorded_requests_for_thread
                    .lock()
                    .map_err(|_| "recorded request mutex poisoned".to_owned())?
                    .push(RecordedRequest {
                        request_line: request_line.clone(),
                        raw: request.clone(),
                    });

                let expected_request_line = format!("POST {} HTTP/1.1", expectation.path);
                if !request.contains(&expected_request_line) {
                    return Err(format!(
                        "unexpected request line, expected `{expected_request_line}`, got `{request_line}`"
                    ));
                }

                for matcher in &expectation.matchers {
                    let matched = match matcher.kind {
                        TextMatchKind::Exact => request.contains(&matcher.needle),
                        TextMatchKind::CaseInsensitive => request
                            .to_ascii_lowercase()
                            .contains(&matcher.needle.to_ascii_lowercase()),
                    };
                    if !matched {
                        return Err(format!(
                            "request `{request_line}` does not contain required text `{}`",
                            matcher.needle
                        ));
                    }
                }

                if !expectation.response.delay.is_zero() {
                    thread::sleep(expectation.response.delay);
                }

                let response = format!(
                    "HTTP/1.1 {} OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    expectation.response.status,
                    expectation.response.content_type,
                    expectation.response.body.len(),
                    expectation.response.body
                );
                stream
                    .write_all(response.as_bytes())
                    .map_err(|error| error.to_string())?;
                stream.flush().map_err(|error| error.to_string())?;
            }

            Ok(())
        });

        Ok(Self {
            base_url: format!("http://{address}"),
            recorded_requests,
            handle: Some(handle),
        })
    }

    pub fn single(expectation: RequestExpectation) -> TestingResult<Self> {
        Self::start(vec![expectation])
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn recorded_requests(&self) -> TestingResult<Vec<RecordedRequest>> {
        self.recorded_requests
            .lock()
            .map(|requests| requests.clone())
            .map_err(|_| "recorded request mutex poisoned".into())
    }

    pub fn finish(mut self) -> TestingResult<Vec<RecordedRequest>> {
        if let Some(handle) = self.handle.take() {
            match handle.join() {
                Ok(result) => {
                    result.map_err(|error| -> Box<dyn StdError + Send + Sync> { error.into() })?
                }
                Err(_) => return Err("fake telegram server thread panicked".into()),
            }
        }

        self.recorded_requests()
    }
}

fn find_header_end(buffer: &[u8]) -> Option<usize> {
    buffer
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|position| position + 4)
}

fn parse_content_length(header: &str) -> std::result::Result<usize, String> {
    for line in header.lines() {
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
        if name.eq_ignore_ascii_case("content-length") {
            let trimmed = value.trim();
            return trimmed
                .parse::<usize>()
                .map_err(|error| format!("invalid content-length `{trimmed}`: {error}"));
        }
    }

    Ok(0)
}

fn read_full_http_request(stream: &mut TcpStream) -> std::result::Result<Vec<u8>, String> {
    let mut request = Vec::with_capacity(16 * 1024);
    let mut chunk = [0_u8; 8 * 1024];
    let mut expected_total_bytes = None;

    loop {
        match stream.read(&mut chunk) {
            Ok(0) => break,
            Ok(read_bytes) => {
                request.extend_from_slice(&chunk[..read_bytes]);

                if expected_total_bytes.is_none()
                    && let Some(header_end) = find_header_end(&request)
                {
                    let header = String::from_utf8_lossy(&request[..header_end]);
                    let content_length = parse_content_length(&header)?;
                    expected_total_bytes = Some(header_end + content_length);
                }

                if let Some(expected) = expected_total_bytes
                    && request.len() >= expected
                {
                    break;
                }
            }
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
                ) =>
            {
                if let Some(expected) = expected_total_bytes
                    && request.len() >= expected
                {
                    break;
                }
                return Err(format!("timed out while reading request: {error}"));
            }
            Err(error) => return Err(error.to_string()),
        }
    }

    if let Some(expected) = expected_total_bytes
        && request.len() < expected
    {
        return Err(format!(
            "incomplete request body: expected {expected} bytes, got {}",
            request.len()
        ));
    }

    Ok(request)
}

fn accept_with_timeout(
    listener: &TcpListener,
    timeout: Duration,
) -> std::result::Result<(TcpStream, SocketAddr), String> {
    listener
        .set_nonblocking(true)
        .map_err(|error| error.to_string())?;
    let deadline = Instant::now() + timeout;
    loop {
        match listener.accept() {
            Ok((stream, address)) => {
                stream
                    .set_nonblocking(false)
                    .map_err(|error| error.to_string())?;
                return Ok((stream, address));
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                if Instant::now() >= deadline {
                    return Err(format!(
                        "timed out waiting for request after {}ms",
                        timeout.as_millis()
                    ));
                }
                thread::sleep(Duration::from_millis(10));
            }
            Err(error) => return Err(error.to_string()),
        }
    }
}

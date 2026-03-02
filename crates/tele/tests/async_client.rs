#![cfg(feature = "async")]

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

use tele::types::advanced::{AdvancedAnswerWebAppQueryRequest, AdvancedGetAvailableGiftsRequest};
use tele::types::{
    AnswerInlineQueryRequest, BotCommand, CreateInvoiceLinkRequest, GetFileRequest,
    GetMyCommandsRequest, InlineQueryResult, InlineQueryResultsButton, LabeledPrice,
    SendPhotoRequest, SendStickerRequest, SetMyCommandsRequest,
};
use tele::{Client, Error, UploadFile};

type DynError = Box<dyn std::error::Error>;
type ServerHandle = thread::JoinHandle<Result<(), String>>;

fn find_header_end(buffer: &[u8]) -> Option<usize> {
    buffer
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|pos| pos + 4)
}

fn parse_content_length(header: &str) -> Result<usize, String> {
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

fn read_full_http_request(stream: &mut TcpStream) -> Result<Vec<u8>, String> {
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

fn spawn_server(
    expected_path: &'static str,
    response_status: u16,
    response_body: &'static str,
) -> Result<(String, ServerHandle), DynError> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let address = listener.local_addr()?;

    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().map_err(|error| error.to_string())?;
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .map_err(|error| error.to_string())?;

        let buffer = read_full_http_request(&mut stream)?;
        let request = String::from_utf8_lossy(&buffer);

        let expected_request_line = format!("POST {expected_path} HTTP/1.1");
        if !request.contains(&expected_request_line) {
            return Err(format!("unexpected request line: {request}"));
        }

        let response = format!(
            "HTTP/1.1 {response_status} OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{response_body}",
            response_body.len()
        );

        stream
            .write_all(response.as_bytes())
            .map_err(|error| error.to_string())?;
        stream.flush().map_err(|error| error.to_string())?;

        Ok(())
    });

    Ok((format!("http://{address}"), handle))
}

fn spawn_server_with_checks(
    expected_path: &'static str,
    response_status: u16,
    response_body: &'static str,
    required_substrings: &'static [&'static str],
) -> Result<(String, ServerHandle), DynError> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let address = listener.local_addr()?;

    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().map_err(|error| error.to_string())?;
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .map_err(|error| error.to_string())?;

        let buffer = read_full_http_request(&mut stream)?;
        let request = String::from_utf8_lossy(&buffer);
        let request_lower = request.to_ascii_lowercase();

        let expected_request_line = format!("POST {expected_path} HTTP/1.1");
        let mut check_error = None;
        if !request.contains(&expected_request_line) {
            check_error = Some(format!("unexpected request line: {request}"));
        }

        if check_error.is_none() {
            for required in required_substrings {
                if !request.contains(required)
                    && !request_lower.contains(&required.to_ascii_lowercase())
                {
                    check_error = Some(format!(
                        "request does not contain required text `{required}`"
                    ));
                    break;
                }
            }
        }

        let response = format!(
            "HTTP/1.1 {response_status} OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{response_body}",
            response_body.len()
        );

        stream
            .write_all(response.as_bytes())
            .map_err(|error| error.to_string())?;
        stream.flush().map_err(|error| error.to_string())?;

        if let Some(error) = check_error {
            return Err(error);
        }

        Ok(())
    });

    Ok((format!("http://{address}"), handle))
}

fn join_server(handle: ServerHandle) -> Result<(), DynError> {
    match handle.join() {
        Ok(result) => result.map_err(Into::into),
        Err(_) => Err("server thread panicked".into()),
    }
}

#[tokio::test]
async fn get_me_success() -> Result<(), DynError> {
    let response =
        r#"{"ok":true,"result":{"id":42,"is_bot":true,"first_name":"tele","username":"tele_bot"}}"#;
    let (base_url, handle) = spawn_server("/bot123:abc/getMe", 200, response)?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;

    let me = client.bot().get_me().await?;
    assert_eq!(me.username.as_deref(), Some("tele_bot"));

    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn raw_layer_call_no_params_success() -> Result<(), DynError> {
    let response =
        r#"{"ok":true,"result":{"id":42,"is_bot":true,"first_name":"tele","username":"tele_bot"}}"#;
    let (base_url, handle) = spawn_server("/bot123:abc/getMe", 200, response)?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let me: tele::types::User = client.raw().call_no_params("getMe").await?;
    assert_eq!(me.username.as_deref(), Some("tele_bot"));

    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn typed_layer_advanced_request_success() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":{"gifts":[]}}"#;
    let (base_url, handle) = spawn_server("/bot123:abc/getAvailableGifts", 200, response)?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let request = AdvancedGetAvailableGiftsRequest::new();
    let value: serde_json::Value = client.typed().call(&request).await?;
    assert!(value.is_object());

    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn api_error_exposes_retry_after() -> Result<(), DynError> {
    let response = r#"{"ok":false,"error_code":429,"description":"Too Many Requests","parameters":{"retry_after":3}}"#;
    let (base_url, handle) = spawn_server("/bot123:abc/getMe", 200, response)?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;

    let err = match client.bot().get_me().await {
        Ok(_) => {
            return Err("expected Telegram API error".into());
        }
        Err(err) => err,
    };

    assert!(matches!(err, Error::Api { .. }));
    assert_eq!(err.retry_after(), Some(Duration::from_secs(3)));

    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn ergo_send_text_success() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":{"message_id":7,"date":1710000000,"chat":{"id":1,"type":"private"},"text":"hello"}}"#;
    let (base_url, handle) = spawn_server("/bot123:abc/sendMessage", 200, response)?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let sent = client.ergo().send_text(1_i64, "hello").await?;
    assert_eq!(sent.message_id.0, 7);

    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn transport_error_redacts_token() -> Result<(), DynError> {
    let client = Client::builder("http://127.0.0.1:9")?
        .bot_token("123:abc")?
        .request_timeout(Duration::from_millis(100))
        .total_timeout(Some(Duration::from_millis(300)))
        .build()?;

    let err = match client.bot().get_me().await {
        Ok(_) => {
            return Err("expected transport error".into());
        }
        Err(err) => err,
    };

    let text = err.to_string();
    assert!(!text.contains("123:abc"));
    Ok(())
}

#[tokio::test]
async fn set_and_get_my_commands_success() -> Result<(), DynError> {
    let set_response = r#"{"ok":true,"result":true}"#;
    let (set_base_url, set_handle) = spawn_server("/bot123:abc/setMyCommands", 200, set_response)?;

    let set_client = Client::builder(set_base_url)?
        .bot_token("123:abc")?
        .build()?;
    let set_request = SetMyCommandsRequest::new(vec![BotCommand::new("start", "start the bot")?])?;
    let set_result = set_client.bot().set_my_commands(&set_request).await?;
    assert!(set_result);
    join_server(set_handle)?;

    let get_response =
        r#"{"ok":true,"result":[{"command":"start","description":"start the bot"}]}"#;
    let (get_base_url, get_handle) = spawn_server("/bot123:abc/getMyCommands", 200, get_response)?;

    let get_client = Client::builder(get_base_url)?
        .bot_token("123:abc")?
        .build()?;
    let get_request = GetMyCommandsRequest::default();
    let commands = get_client.bot().get_my_commands(&get_request).await?;
    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].command, "start");
    join_server(get_handle)?;

    Ok(())
}

#[tokio::test]
async fn get_file_success() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":{"file_id":"file_1","file_unique_id":"uniq_1","file_size":128,"file_path":"photos/pic.jpg"}}"#;
    let (base_url, handle) = spawn_server("/bot123:abc/getFile", 200, response)?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let file = client
        .files()
        .get_file(&GetFileRequest::new("file_1"))
        .await?;
    assert_eq!(file.file_id, "file_1");
    assert_eq!(file.file_path.as_deref(), Some("photos/pic.jpg"));

    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn send_photo_upload_multipart_success() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":{"message_id":100,"date":1710000000,"chat":{"id":1,"type":"private"},"photo":[{"file_id":"file_1","file_unique_id":"uniq_1","width":10,"height":10}]}}"#;
    const CHECKS: [&str; 4] = [
        "Content-Type: multipart/form-data; boundary=",
        "name=\"chat_id\"",
        "name=\"photo\"; filename=\"image.jpg\"",
        "binary-photo-data",
    ];
    let (base_url, handle) =
        spawn_server_with_checks("/bot123:abc/sendPhoto", 200, response, &CHECKS)?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let file = UploadFile::from_bytes("image.jpg", b"binary-photo-data".to_vec())?;
    let request = SendPhotoRequest::new(1_i64, "ignored-in-multipart");
    let message = client.messages().send_photo_upload(&request, &file).await?;
    assert_eq!(message.message_id.0, 100);

    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn advanced_get_available_gifts_success() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":{"gifts":[]}}"#;
    let (base_url, handle) = spawn_server("/bot123:abc/getAvailableGifts", 200, response)?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let request = AdvancedGetAvailableGiftsRequest::new();
    let value = client
        .advanced()
        .get_available_gifts::<serde_json::Value>(&request)
        .await?;
    assert!(value.is_object());

    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn answer_web_app_query_typed_success() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":{"inline_message_id":"inline-msg-1"}}"#;
    const CHECKS: [&str; 3] = [
        "\"web_app_query_id\":\"query-1\"",
        "\"type\":\"article\"",
        "\"id\":\"result-1\"",
    ];
    let (base_url, handle) =
        spawn_server_with_checks("/bot123:abc/answerWebAppQuery", 200, response, &CHECKS)?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let result = InlineQueryResult::new(serde_json::json!({
        "type": "article",
        "id": "result-1",
        "title": "Mini App result",
        "input_message_content": {
            "message_text": "Mini App accepted"
        }
    }));
    let request = AdvancedAnswerWebAppQueryRequest::new("query-1", result);
    let sent = client
        .advanced()
        .answer_web_app_query_typed(&request)
        .await?;
    assert_eq!(sent.inline_message_id, "inline-msg-1");

    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn answer_inline_query_with_typed_button_success() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":true}"#;
    const CHECKS: [&str; 5] = [
        "\"inline_query_id\":\"inline-q-1\"",
        "\"type\":\"article\"",
        "\"id\":\"result-inline-1\"",
        "\"button\":{\"text\":\"Open Mini App\"",
        "\"web_app\":{\"url\":\"https://example.com/mini-app\"}",
    ];
    let (base_url, handle) =
        spawn_server_with_checks("/bot123:abc/answerInlineQuery", 200, response, &CHECKS)?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let request = AnswerInlineQueryRequest::new(
        "inline-q-1",
        vec![InlineQueryResult::article(
            "result-inline-1",
            "Inline title",
            "Inline message text",
        )],
    )
    .button(InlineQueryResultsButton::web_app(
        "Open Mini App",
        "https://example.com/mini-app",
    ));
    let ok = client.updates().answer_inline_query(&request).await?;
    assert!(ok);

    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn create_invoice_link_success() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":"https://t.me/$1234"}"#;
    let (base_url, handle) = spawn_server("/bot123:abc/createInvoiceLink", 200, response)?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let request = CreateInvoiceLinkRequest::new(
        "Pro Plan",
        "Monthly subscription",
        "order-1",
        "USD",
        vec![LabeledPrice::new("Pro Plan", 499)],
    )?;
    let link = client.payments().create_invoice_link(&request).await?;
    assert_eq!(link, "https://t.me/$1234");

    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn send_sticker_upload_multipart_success() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":{"message_id":101,"date":1710000001,"chat":{"id":1,"type":"private"}}}"#;
    const CHECKS: [&str; 4] = [
        "Content-Type: multipart/form-data; boundary=",
        "name=\"chat_id\"",
        "name=\"sticker\"; filename=\"sticker.webp\"",
        "binary-sticker-data",
    ];
    let (base_url, handle) =
        spawn_server_with_checks("/bot123:abc/sendSticker", 200, response, &CHECKS)?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let request = SendStickerRequest::new(1_i64, "ignored-in-multipart");
    let file = UploadFile::from_bytes("sticker.webp", b"binary-sticker-data".to_vec())?;
    let message = client
        .stickers()
        .send_sticker_upload(&request, &file)
        .await?;
    assert_eq!(message.message_id.0, 101);

    join_server(handle)?;
    Ok(())
}

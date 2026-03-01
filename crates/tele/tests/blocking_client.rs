#![cfg(feature = "blocking")]

use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;
use std::time::Duration;

use tele::BlockingClient;
use tele::types::advanced::AdvancedGetAvailableGiftsRequest;
use tele::types::{CreateInvoiceLinkRequest, GetChatMemberCountRequest, LabeledPrice};

type DynError = Box<dyn std::error::Error>;
type ServerHandle = thread::JoinHandle<Result<(), String>>;

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

        let mut buffer = vec![0_u8; 16 * 1024];
        let read_bytes = stream
            .read(&mut buffer)
            .map_err(|error| error.to_string())?;
        let request = String::from_utf8_lossy(&buffer[..read_bytes]);

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

fn join_server(handle: ServerHandle) -> Result<(), DynError> {
    match handle.join() {
        Ok(result) => result.map_err(Into::into),
        Err(_) => Err("server thread panicked".into()),
    }
}

#[tokio::test]
async fn blocking_get_me_success() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":{"id":7,"is_bot":true,"first_name":"tele","username":"blocking_bot"}}"#;
    let (base_url, handle) = spawn_server("/bot123:abc/getMe", 200, response)?;

    let client = BlockingClient::builder(base_url)?
        .bot_token("123:abc")?
        .build_blocking()?;

    let me = client.bot().get_me()?;
    assert_eq!(me.username.as_deref(), Some("blocking_bot"));

    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn blocking_raw_layer_call_no_params_success() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":{"id":7,"is_bot":true,"first_name":"tele","username":"blocking_bot"}}"#;
    let (base_url, handle) = spawn_server("/bot123:abc/getMe", 200, response)?;

    let client = BlockingClient::builder(base_url)?
        .bot_token("123:abc")?
        .build_blocking()?;

    let me: tele::types::User = client.raw().call_no_params("getMe")?;
    assert_eq!(me.username.as_deref(), Some("blocking_bot"));

    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn blocking_get_chat_member_count_success() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":42}"#;
    let (base_url, handle) = spawn_server("/bot123:abc/getChatMemberCount", 200, response)?;

    let client = BlockingClient::builder(base_url)?
        .bot_token("123:abc")?
        .build_blocking()?;

    let request = GetChatMemberCountRequest::new(-100_123_456_i64);
    let count = client.chats().get_chat_member_count(&request)?;
    assert_eq!(count, 42);

    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn blocking_advanced_get_available_gifts_success() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":{"gifts":[]}}"#;
    let (base_url, handle) = spawn_server("/bot123:abc/getAvailableGifts", 200, response)?;

    let client = BlockingClient::builder(base_url)?
        .bot_token("123:abc")?
        .build_blocking()?;

    let request = AdvancedGetAvailableGiftsRequest::new();
    let value: serde_json::Value = client.advanced().get_available_gifts(&request)?;
    assert!(value.is_object());

    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn blocking_typed_layer_advanced_request_success() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":{"gifts":[]}}"#;
    let (base_url, handle) = spawn_server("/bot123:abc/getAvailableGifts", 200, response)?;

    let client = BlockingClient::builder(base_url)?
        .bot_token("123:abc")?
        .build_blocking()?;

    let request = AdvancedGetAvailableGiftsRequest::new();
    let value: serde_json::Value = client.typed().call(&request)?;
    assert!(value.is_object());

    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn blocking_ergo_send_text_success() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":{"message_id":11,"date":1710000000,"chat":{"id":1,"type":"private"},"text":"hello"}}"#;
    let (base_url, handle) = spawn_server("/bot123:abc/sendMessage", 200, response)?;

    let client = BlockingClient::builder(base_url)?
        .bot_token("123:abc")?
        .build_blocking()?;

    let sent = client.ergo().send_text(1_i64, "hello")?;
    assert_eq!(sent.message_id.0, 11);

    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn blocking_create_invoice_link_success() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":"https://t.me/$5678"}"#;
    let (base_url, handle) = spawn_server("/bot123:abc/createInvoiceLink", 200, response)?;

    let client = BlockingClient::builder(base_url)?
        .bot_token("123:abc")?
        .build_blocking()?;

    let request = CreateInvoiceLinkRequest::new(
        "Basic Plan",
        "Weekly subscription",
        "order-2",
        "USD",
        vec![LabeledPrice::new("Basic Plan", 199)],
    )?;
    let link = client.payments().create_invoice_link(&request)?;
    assert_eq!(link, "https://t.me/$5678");

    join_server(handle)?;
    Ok(())
}

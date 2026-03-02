#![cfg(feature = "bot")]

use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde_json::json;
use tele::Client;
use tele::Error;
use tele::bot::{
    BotContext, BotEngine, BotOutbox, CallbackInput, ChatSession, CommandData, DispatchOutcome,
    EngineConfig, EngineEvent, ErrorPolicy, HandlerError, InMemorySessionStore,
    JsonFileSessionStore, LongPollingSource, OutboxConfig, PollingConfig, Router, StateTransition,
    TextInput, UpdateExt, UpdateExtractor, WebAppInput, WebhookRunner, WriteAccessAllowedInput,
    apply_chat_state_transition, channel_source, clear_chat_state, extract_callback_data,
    extract_callback_json, extract_command, extract_command_args, extract_command_data,
    extract_message, extract_text, extract_web_app_data, extract_write_access_allowed,
    load_chat_state, parse_command_text, save_chat_state, tokenize_command_args,
};
use tele::types::advanced::AdvancedSetChatMenuButtonRequest;
use tele::types::telegram::{
    InlineKeyboardButton, InlineQueryResult, InlineQueryResultsButton, KeyboardButton, MenuButton,
    MenuButtonWebApp, WebAppInfo,
};
use tele::types::update::Update;

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

        let mut buffer = vec![0_u8; 16 * 1024];
        let read_bytes = stream
            .read(&mut buffer)
            .map_err(|error| error.to_string())?;
        let request = String::from_utf8_lossy(&buffer[..read_bytes]);

        let expected_request_line = format!("POST {expected_path} HTTP/1.1");
        if !request.contains(&expected_request_line) {
            return Err(format!("unexpected request line: {request}"));
        }

        for required in required_substrings {
            if !request.contains(required) {
                return Err(format!(
                    "request missing required content `{required}`: {request}"
                ));
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

        Ok(())
    });

    Ok((format!("http://{address}"), handle))
}

fn spawn_server_sequence(
    expected_path: &'static str,
    responses: Vec<(u16, &'static str)>,
) -> Result<(String, ServerHandle), DynError> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let address = listener.local_addr()?;

    let handle = thread::spawn(move || {
        for (response_status, response_body) in responses {
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

fn parse_update(input: serde_json::Value) -> Option<Update> {
    serde_json::from_value(input).ok()
}

fn message_update(update_id: i64, chat_id: i64, text: &str) -> serde_json::Value {
    json!({
        "update_id": update_id,
        "message": {
            "message_id": update_id,
            "date": 1700000000 + update_id,
            "chat": {"id": chat_id, "type": "private"},
            "text": text
        }
    })
}

fn callback_update(update_id: i64, chat_id: i64, data: &str) -> serde_json::Value {
    json!({
        "update_id": update_id,
        "callback_query": {
            "id": format!("cb-{update_id}"),
            "from": {
                "id": 123,
                "is_bot": false,
                "first_name": "tester"
            },
            "message": {
                "message_id": update_id,
                "date": 1700000000 + update_id,
                "chat": {"id": chat_id, "type": "private"},
                "text": "button clicked"
            },
            "data": data
        }
    })
}

#[tokio::test]
async fn command_router_runs_with_middleware() -> Result<(), DynError> {
    let client = Client::builder("http://127.0.0.1:9")?
        .bot_token("123:abc")?
        .build()?;

    let middleware_hits = Arc::new(AtomicUsize::new(0));
    let handler_hits = Arc::new(AtomicUsize::new(0));

    let mut router = Router::new();
    {
        let middleware_hits = Arc::clone(&middleware_hits);
        router.middleware(move |context, update, next| {
            let middleware_hits = Arc::clone(&middleware_hits);
            async move {
                middleware_hits.fetch_add(1, Ordering::SeqCst);
                next(context, update).await
            }
        });
    }

    {
        let handler_hits = Arc::clone(&handler_hits);
        router.on_command("start", move |_context: BotContext, _update: Update| {
            let handler_hits = Arc::clone(&handler_hits);
            async move {
                handler_hits.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        });
    }

    let maybe_update = parse_update(serde_json::json!({
        "update_id": 100,
        "message": {
            "message_id": 1,
            "date": 1700000000,
            "chat": {"id": 1, "type": "private"},
            "text": "/start"
        }
    }));
    assert!(maybe_update.is_some());

    let Some(update) = maybe_update else {
        return Ok(());
    };

    assert_eq!(extract_command(&update), Some("start"));

    let handled = router.dispatch(BotContext::new(client), update).await?;
    assert!(handled);
    assert_eq!(middleware_hits.load(Ordering::SeqCst), 1);
    assert_eq!(handler_hits.load(Ordering::SeqCst), 1);

    Ok(())
}

#[tokio::test]
async fn command_and_update_extractors_work() -> Result<(), DynError> {
    let maybe_update = parse_update(message_update(200, 1, "/echo hello world"));
    assert!(maybe_update.is_some());
    let Some(update) = maybe_update else {
        return Ok(());
    };

    let parsed = parse_command_text("/echo hello world");
    assert_eq!(
        parsed,
        Some(CommandData {
            name: "echo".to_owned(),
            args: "hello world".to_owned()
        })
    );

    assert!(extract_message(&update).is_some());
    assert_eq!(extract_text(&update), Some("/echo hello world"));
    assert_eq!(extract_command(&update), Some("echo"));
    assert_eq!(extract_command_args(&update), Some("hello world"));
    assert_eq!(
        extract_command_data(&update),
        Some(CommandData {
            name: "echo".to_owned(),
            args: "hello world".to_owned()
        })
    );
    assert_eq!(update.command(), Some("echo"));
    assert_eq!(update.command_args(), Some("hello world"));
    assert_eq!(update.text(), Some("/echo hello world"));
    assert_eq!(update.chat_id(), Some(1));
    assert_eq!(
        tokenize_command_args(r#"hello "quoted world" again"#),
        Some(vec![
            "hello".to_owned(),
            "quoted world".to_owned(),
            "again".to_owned()
        ])
    );
    assert_eq!(
        tokenize_command_args(r#"a\ b"#),
        Some(vec!["a b".to_owned()])
    );
    assert_eq!(tokenize_command_args(r#""unterminated"#), None);

    let maybe_web_app_update = parse_update(serde_json::json!({
        "update_id": 202,
        "message": {
            "message_id": 2,
            "date": 1700000001,
            "chat": {"id": 1, "type": "private"},
            "web_app_data": {
                "data": "{\"query_id\":\"q-1\",\"action\":\"checkout\"}",
                "button_text": "Open Mini App"
            }
        }
    }));
    assert!(maybe_web_app_update.is_some());
    let Some(web_app_update) = maybe_web_app_update else {
        return Ok(());
    };

    assert_eq!(
        extract_web_app_data(&web_app_update).map(|data| data.button_text.as_str()),
        Some("Open Mini App")
    );
    assert_eq!(
        web_app_update.web_app_data().map(|data| data.data.as_str()),
        Some("{\"query_id\":\"q-1\",\"action\":\"checkout\"}")
    );
    assert!(
        web_app_update
            .message()
            .and_then(|message| message.web_app_data())
            .is_some()
    );
    let extracted = WebAppInput::extract(&web_app_update);
    assert_eq!(
        extracted.as_ref().map(|input| input.0.button_text.as_str()),
        Some("Open Mini App")
    );

    let maybe_write_access_update = parse_update(serde_json::json!({
        "update_id": 203,
        "message": {
            "message_id": 3,
            "date": 1700000002,
            "chat": {"id": 1, "type": "private"},
            "write_access_allowed": {
                "from_request": true,
                "web_app_name": "mini_app_sample"
            }
        }
    }));
    assert!(maybe_write_access_update.is_some());
    let Some(write_access_update) = maybe_write_access_update else {
        return Ok(());
    };
    assert_eq!(
        extract_write_access_allowed(&write_access_update).and_then(|value| value.from_request),
        Some(true)
    );
    assert_eq!(
        write_access_update
            .write_access_allowed()
            .and_then(|value| value.web_app_name.as_deref()),
        Some("mini_app_sample")
    );
    let write_access_extracted = WriteAccessAllowedInput::extract(&write_access_update);
    assert_eq!(
        write_access_extracted
            .as_ref()
            .and_then(|input| input.0.web_app_name.as_deref()),
        Some("mini_app_sample")
    );

    let maybe_callback = parse_update(callback_update(201, 1, "btn-1"));
    assert!(maybe_callback.is_some());
    let Some(callback) = maybe_callback else {
        return Ok(());
    };

    assert_eq!(extract_callback_data(&callback), Some("btn-1"));
    assert!(extract_callback_json::<serde_json::Value>(&callback).is_none());
    assert_eq!(callback.callback_data(), Some("btn-1"));
    assert!(callback.message().is_some());

    Ok(())
}

#[tokio::test]
async fn web_app_typed_builders_serialize() -> Result<(), DynError> {
    let article_result =
        InlineQueryResult::article("article-1", "Article Title", "Article Message Text");
    let article_result_json = serde_json::to_value(article_result)?;
    assert_eq!(article_result_json["type"], "article");
    assert_eq!(
        article_result_json["input_message_content"]["message_text"],
        "Article Message Text"
    );

    let inline_button = InlineKeyboardButton::new("Open Mini App")
        .web_app(WebAppInfo::new("https://example.com/mini-app"));
    let inline_json = serde_json::to_value(&inline_button)?;
    assert_eq!(
        inline_json["web_app"]["url"],
        "https://example.com/mini-app"
    );

    let keyboard_button =
        KeyboardButton::new("Open Mini App").web_app("https://example.com/mini-app-keyboard");
    let keyboard_json = serde_json::to_value(&keyboard_button)?;
    assert_eq!(
        keyboard_json["web_app"]["url"],
        "https://example.com/mini-app-keyboard"
    );

    let menu_button = MenuButton::web_app(
        "Open Mini App",
        WebAppInfo::new("https://example.com/menu-mini-app"),
    );
    let menu_web_app = menu_button.as_web_app();
    assert_eq!(
        menu_web_app.map(|value| value.text.as_str()),
        Some("Open Mini App")
    );
    assert_eq!(
        menu_web_app.map(|value| value.web_app.url.as_str()),
        Some("https://example.com/menu-mini-app")
    );

    let menu_button_from_struct = MenuButton::from(MenuButtonWebApp::new(
        "Open Mini App",
        "https://example.com/menu-mini-app-struct",
    ));
    let menu_struct_json = serde_json::to_value(menu_button_from_struct)?;
    assert_eq!(menu_struct_json["type"], "web_app");

    let unknown_menu_button =
        MenuButton::new(json!({ "type": "custom_menu_button", "raw_field": "raw_value" }));
    let unknown_menu_json = serde_json::to_value(unknown_menu_button)?;
    assert_eq!(unknown_menu_json["type"], "custom_menu_button");
    assert_eq!(unknown_menu_json["raw_field"], "raw_value");

    let request = AdvancedSetChatMenuButtonRequest::new()
        .chat_id(10001)
        .menu_button_web_app("Open Mini App", "https://example.com/menu");
    let request_json = serde_json::to_value(&request)?;
    assert_eq!(request_json["chat_id"], 10001);
    assert_eq!(request_json["menu_button"]["type"], "web_app");
    assert_eq!(request_json["menu_button"]["text"], "Open Mini App");
    assert_eq!(
        request_json["menu_button"]["web_app"]["url"],
        "https://example.com/menu"
    );

    let inline_query_button = InlineQueryResultsButton::web_app(
        "Open Mini App",
        "https://example.com/inline-button-mini-app",
    );
    let inline_query_button_json = serde_json::to_value(inline_query_button)?;
    assert_eq!(inline_query_button_json["text"], "Open Mini App");
    assert_eq!(
        inline_query_button_json["web_app"]["url"],
        "https://example.com/inline-button-mini-app"
    );

    Ok(())
}

#[tokio::test]
async fn long_polling_source_dispatches_updates() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":[{"update_id":777,"message":{"message_id":10,"date":1710000000,"chat":{"id":1,"type":"private"},"text":"/start"}}]}"#;
    let (base_url, handle) = spawn_server("/bot123:abc/getUpdates", 200, response)?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;

    let handler_hits = Arc::new(AtomicUsize::new(0));
    let mut router = Router::new();
    {
        let handler_hits = Arc::clone(&handler_hits);
        router.on_command("start", move |_context: BotContext, _update: Update| {
            let handler_hits = Arc::clone(&handler_hits);
            async move {
                handler_hits.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        });
    }

    let source = LongPollingSource::new(client.clone()).with_config(PollingConfig {
        disable_webhook_on_start: false,
        poll_timeout_seconds: 1,
        ..PollingConfig::default()
    });
    let mut engine = BotEngine::new(client, source, router).with_config(EngineConfig {
        continue_on_source_error: false,
        continue_on_handler_error: false,
        ..EngineConfig::default()
    });

    let outcomes = engine.poll_once().await?;
    assert_eq!(outcomes, vec![DispatchOutcome::Handled { update_id: 777 }]);
    assert_eq!(engine.source_mut().next_offset(), Some(778));
    assert_eq!(handler_hits.load(Ordering::SeqCst), 1);

    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn long_polling_source_loads_persisted_offset() -> Result<(), DynError> {
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let offset_path = std::env::temp_dir().join(format!("tele-offset-{timestamp}.json"));
    let snapshot = serde_json::json!({
        "version": 1,
        "next_offset": 501
    });
    fs::write(&offset_path, serde_json::to_vec(&snapshot)?)?;

    let response = r#"{"ok":true,"result":[]}"#;
    let (base_url, handle) =
        spawn_server_with_checks("/bot123:abc/getUpdates", 200, response, &["\"offset\":501"])?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let source = LongPollingSource::new(client.clone()).with_config(PollingConfig {
        disable_webhook_on_start: false,
        persist_offset_path: Some(offset_path.clone()),
        poll_timeout_seconds: 1,
        ..PollingConfig::default()
    });
    let mut engine = BotEngine::new(client, source, Router::new()).with_config(EngineConfig {
        continue_on_source_error: false,
        continue_on_handler_error: false,
        ..EngineConfig::default()
    });

    let outcomes = engine.poll_once().await?;
    assert!(outcomes.is_empty());
    assert_eq!(engine.source_mut().next_offset(), Some(501));

    join_server(handle)?;
    let _ = fs::remove_file(offset_path);
    Ok(())
}

#[tokio::test]
async fn long_polling_source_dedupes_duplicate_update_ids() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":[{"update_id":990,"message":{"message_id":1,"date":1710000101,"chat":{"id":1,"type":"private"},"text":"/start"}},{"update_id":990,"message":{"message_id":2,"date":1710000102,"chat":{"id":1,"type":"private"},"text":"/start"}}]}"#;
    let (base_url, handle) = spawn_server("/bot123:abc/getUpdates", 200, response)?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let handler_hits = Arc::new(AtomicUsize::new(0));
    let mut router = Router::new();
    {
        let handler_hits = Arc::clone(&handler_hits);
        router.on_command("start", move |_context: BotContext, _update: Update| {
            let handler_hits = Arc::clone(&handler_hits);
            async move {
                handler_hits.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        });
    }

    let source = LongPollingSource::new(client.clone()).with_config(PollingConfig {
        disable_webhook_on_start: false,
        poll_timeout_seconds: 1,
        dedupe_window_size: 128,
        ..PollingConfig::default()
    });
    let mut engine = BotEngine::new(client, source, router).with_config(EngineConfig {
        continue_on_source_error: false,
        continue_on_handler_error: false,
        ..EngineConfig::default()
    });

    let outcomes = engine.poll_once().await?;
    assert_eq!(outcomes, vec![DispatchOutcome::Handled { update_id: 990 }]);
    assert_eq!(handler_hits.load(Ordering::SeqCst), 1);
    assert_eq!(engine.source_mut().next_offset(), Some(991));

    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn long_polling_source_persists_offset_after_poll() -> Result<(), DynError> {
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let offset_path = std::env::temp_dir().join(format!("tele-offset-save-{timestamp}.json"));

    let response = r#"{"ok":true,"result":[{"update_id":701,"message":{"message_id":1,"date":1710000111,"chat":{"id":1,"type":"private"},"text":"hello"}}]}"#;
    let (base_url, handle) = spawn_server("/bot123:abc/getUpdates", 200, response)?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let source = LongPollingSource::new(client.clone()).with_config(PollingConfig {
        disable_webhook_on_start: false,
        persist_offset_path: Some(offset_path.clone()),
        poll_timeout_seconds: 1,
        ..PollingConfig::default()
    });
    let mut engine = BotEngine::new(client, source, Router::new()).with_config(EngineConfig {
        continue_on_source_error: false,
        continue_on_handler_error: false,
        ..EngineConfig::default()
    });

    let outcomes = engine.poll_once().await?;
    assert_eq!(outcomes, vec![DispatchOutcome::Ignored { update_id: 701 }]);
    assert_eq!(engine.source_mut().next_offset(), Some(702));

    let raw = fs::read(&offset_path)?;
    let snapshot: serde_json::Value = serde_json::from_slice(&raw)?;
    assert_eq!(
        snapshot
            .get("next_offset")
            .and_then(serde_json::Value::as_i64),
        Some(702)
    );

    join_server(handle)?;
    let _ = fs::remove_file(offset_path);
    Ok(())
}

#[tokio::test]
async fn bot_engine_with_long_polling_source_dispatches_updates() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":[{"update_id":888,"message":{"message_id":10,"date":1710000000,"chat":{"id":1,"type":"private"},"text":"/start"}}]}"#;
    let (base_url, handle) = spawn_server("/bot123:abc/getUpdates", 200, response)?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;

    let handler_hits = Arc::new(AtomicUsize::new(0));
    let mut router = Router::new();
    {
        let handler_hits = Arc::clone(&handler_hits);
        router.on_command("start", move |_context: BotContext, _update: Update| {
            let handler_hits = Arc::clone(&handler_hits);
            async move {
                handler_hits.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        });
    }

    let source = LongPollingSource::new(client.clone()).with_config(PollingConfig {
        disable_webhook_on_start: false,
        poll_timeout_seconds: 1,
        ..PollingConfig::default()
    });
    let mut engine = BotEngine::new(client, source, router).with_config(EngineConfig {
        continue_on_source_error: false,
        continue_on_handler_error: false,
        ..EngineConfig::default()
    });

    let outcomes = engine.poll_once().await?;
    assert_eq!(outcomes, vec![DispatchOutcome::Handled { update_id: 888 }]);
    assert_eq!(handler_hits.load(Ordering::SeqCst), 1);

    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn bot_engine_channel_source_dispatches_updates() -> Result<(), DynError> {
    let client = Client::builder("http://127.0.0.1:9")?
        .bot_token("123:abc")?
        .build()?;

    let hits = Arc::new(AtomicUsize::new(0));
    let mut router = Router::new();
    {
        let hits = Arc::clone(&hits);
        router.on_command("start", move |_context: BotContext, _update: Update| {
            let hits = Arc::clone(&hits);
            async move {
                hits.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        });
    }

    let (sink, source) = channel_source(8);
    let mut engine = BotEngine::new(client, source, router).with_config(EngineConfig {
        continue_on_source_error: false,
        continue_on_handler_error: false,
        ..EngineConfig::default()
    });

    let maybe_update = parse_update(message_update(999, 1, "/start"));
    assert!(maybe_update.is_some());
    let Some(update) = maybe_update else {
        return Ok(());
    };
    sink.send(update).await?;

    let outcomes = engine.poll_once().await?;
    assert_eq!(outcomes, vec![DispatchOutcome::Handled { update_id: 999 }]);
    assert_eq!(hits.load(Ordering::SeqCst), 1);

    Ok(())
}

#[tokio::test]
async fn run_until_stops_on_shutdown_even_when_poll_errors() -> Result<(), DynError> {
    let client = Client::builder("http://127.0.0.1:9")?
        .bot_token("123:abc")?
        .request_timeout(Duration::from_millis(80))
        .total_timeout(Some(Duration::from_millis(120)))
        .build()?;

    let source = LongPollingSource::new(client.clone()).with_config(PollingConfig {
        disable_webhook_on_start: false,
        ..PollingConfig::default()
    });
    let mut engine = BotEngine::new(client, source, Router::new()).with_config(EngineConfig {
        continue_on_source_error: true,
        error_delay: Duration::from_millis(10),
        ..EngineConfig::default()
    });

    let shutdown = async {
        tokio::time::sleep(Duration::from_millis(120)).await;
    };

    let result = engine.run_until(shutdown).await;
    assert!(result.is_ok());
    Ok(())
}

#[tokio::test]
async fn webhook_runner_validates_secret_and_dispatches_json() -> Result<(), DynError> {
    let client = Client::builder("http://127.0.0.1:9")?
        .bot_token("123:abc")?
        .build()?;

    let handler_hits = Arc::new(AtomicUsize::new(0));
    let mut router = Router::new();
    {
        let handler_hits = Arc::clone(&handler_hits);
        router.on_command("start", move |_context: BotContext, _update: Update| {
            let handler_hits = Arc::clone(&handler_hits);
            async move {
                handler_hits.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        });
    }

    let runner = WebhookRunner::new(client, router).expected_secret_token("secret-token");
    let payload = serde_json::to_vec(&message_update(901, 10, "/start ping"))?;

    let outcome = runner
        .dispatch_json_outcome(&payload, Some("secret-token"))
        .await?;
    assert_eq!(outcome, DispatchOutcome::Handled { update_id: 901 });
    assert_eq!(handler_hits.load(Ordering::SeqCst), 1);

    let wrong_secret_error = runner
        .dispatch_json_outcome(&payload, Some("wrong"))
        .await
        .err();
    assert!(matches!(
        wrong_secret_error,
        Some(Error::InvalidRequest { reason }) if reason.contains("secret")
    ));

    Ok(())
}

#[tokio::test]
async fn fallible_route_maps_user_error_to_reply() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":{"message_id":99,"date":1710000009,"chat":{"id":10,"type":"private"},"text":"invalid input"}}"#;
    let (base_url, handle) = spawn_server("/bot123:abc/sendMessage", 200, response)?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;

    let mut router = Router::new();
    router.on_message_fallible(|_context: BotContext, _update: Update| async move {
        Err(HandlerError::user("invalid input"))
    });

    let maybe_update = parse_update(message_update(902, 10, "bad request"));
    assert!(maybe_update.is_some());
    let Some(update) = maybe_update else {
        return Ok(());
    };

    let handled = router.dispatch(BotContext::new(client), update).await?;
    assert!(handled);

    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn bot_context_answers_callback_from_update() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":true}"#;
    let (base_url, handle) = spawn_server("/bot123:abc/answerCallbackQuery", 200, response)?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let context = BotContext::new(client);

    let maybe_update = parse_update(callback_update(903, 10, "confirm"));
    assert!(maybe_update.is_some());
    let Some(update) = maybe_update else {
        return Ok(());
    };

    let answered = context
        .answer_callback_from_update(&update, Some("received".to_owned()))
        .await?;
    assert!(answered);

    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn session_store_helpers_apply_state_transitions() -> Result<(), DynError> {
    let store = InMemorySessionStore::<String>::new();

    let maybe_update = parse_update(message_update(1001, 11, "hi"));
    assert!(maybe_update.is_some());
    let Some(update) = maybe_update else {
        return Ok(());
    };

    save_chat_state(&store, &update, "step-1".to_owned()).await?;
    let loaded = load_chat_state(&store, &update).await?;
    assert_eq!(loaded.as_deref(), Some("step-1"));

    apply_chat_state_transition(&store, &update, StateTransition::Keep).await?;
    let loaded_after_keep = load_chat_state(&store, &update).await?;
    assert_eq!(loaded_after_keep.as_deref(), Some("step-1"));

    apply_chat_state_transition(&store, &update, StateTransition::Set("step-2".to_owned())).await?;
    let loaded_after_set = load_chat_state(&store, &update).await?;
    assert_eq!(loaded_after_set.as_deref(), Some("step-2"));

    apply_chat_state_transition::<String, _>(&store, &update, StateTransition::Clear).await?;
    let loaded_after_clear = load_chat_state(&store, &update).await?;
    assert!(loaded_after_clear.is_none());

    clear_chat_state::<String, _>(&store, &update).await?;

    Ok(())
}

#[tokio::test]
async fn chat_session_transition_applies_state() -> Result<(), DynError> {
    let session = ChatSession::<String, _>::new(InMemorySessionStore::new());
    let maybe_update = parse_update(message_update(1003, 22, "state"));
    assert!(maybe_update.is_some());
    let Some(update) = maybe_update else {
        return Ok(());
    };

    let output = session
        .transition(&update, |state| async move {
            let next = match state {
                None => "step-1".to_owned(),
                Some(previous) => format!("{previous}-next"),
            };
            ("ok".to_owned(), StateTransition::Set(next))
        })
        .await?;
    assert_eq!(output, "ok");
    assert_eq!(session.load(&update).await?.as_deref(), Some("step-1"));

    session
        .transition(&update, |state| async move {
            let exists = state.is_some();
            let transition = if exists {
                StateTransition::Clear
            } else {
                StateTransition::Keep
            };
            (exists, transition)
        })
        .await?;
    assert!(session.load(&update).await?.is_none());

    Ok(())
}

#[tokio::test]
async fn session_store_helpers_error_without_chat_id() -> Result<(), DynError> {
    let store = InMemorySessionStore::<String>::new();
    let maybe_update = parse_update(json!({
        "update_id": 1002,
        "inline_query": {
            "id": "q1",
            "from": {"id": 42, "is_bot": false, "first_name": "A"},
            "query": "hello",
            "offset": ""
        }
    }));
    assert!(maybe_update.is_some());
    let Some(update) = maybe_update else {
        return Ok(());
    };

    let error = load_chat_state(&store, &update).await.err();
    assert!(matches!(
        error,
        Some(Error::InvalidRequest { reason }) if reason.contains("chat id")
    ));

    Ok(())
}

#[tokio::test]
async fn bot_engine_dispatches_concurrently_when_enabled() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":[{"update_id":301,"message":{"message_id":1,"date":1710000001,"chat":{"id":77,"type":"private"},"text":"/start"}},{"update_id":302,"message":{"message_id":2,"date":1710000002,"chat":{"id":77,"type":"private"},"text":"/start"}},{"update_id":303,"message":{"message_id":3,"date":1710000003,"chat":{"id":77,"type":"private"},"text":"/start"}}]}"#;
    let (base_url, handle) = spawn_server("/bot123:abc/getUpdates", 200, response)?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let in_flight = Arc::new(AtomicUsize::new(0));
    let max_in_flight = Arc::new(AtomicUsize::new(0));
    let handled = Arc::new(AtomicUsize::new(0));

    let mut router = Router::new();
    {
        let in_flight = Arc::clone(&in_flight);
        let max_in_flight = Arc::clone(&max_in_flight);
        let handled = Arc::clone(&handled);
        router.on_command("start", move |_context: BotContext, _update: Update| {
            let in_flight = Arc::clone(&in_flight);
            let max_in_flight = Arc::clone(&max_in_flight);
            let handled = Arc::clone(&handled);

            async move {
                let now = in_flight.fetch_add(1, Ordering::SeqCst) + 1;
                loop {
                    let observed = max_in_flight.load(Ordering::SeqCst);
                    if now <= observed {
                        break;
                    }
                    if max_in_flight
                        .compare_exchange(observed, now, Ordering::SeqCst, Ordering::SeqCst)
                        .is_ok()
                    {
                        break;
                    }
                }

                tokio::time::sleep(Duration::from_millis(40)).await;
                in_flight.fetch_sub(1, Ordering::SeqCst);
                handled.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        });
    }

    let source = LongPollingSource::new(client.clone()).with_config(PollingConfig {
        disable_webhook_on_start: false,
        poll_timeout_seconds: 1,
        ..PollingConfig::default()
    });
    let mut engine = BotEngine::new(client, source, router).with_config(EngineConfig {
        continue_on_source_error: false,
        continue_on_handler_error: false,
        max_handler_concurrency: 3,
        ..EngineConfig::default()
    });

    let outcomes = engine.poll_once().await?;
    assert_eq!(outcomes.len(), 3);
    assert_eq!(engine.source_mut().next_offset(), Some(304));
    assert_eq!(handled.load(Ordering::SeqCst), 3);
    assert!(max_in_flight.load(Ordering::SeqCst) >= 2);

    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn extractor_routes_dispatch_text_and_callback_inputs() -> Result<(), DynError> {
    let client = Client::builder("http://127.0.0.1:9")?
        .bot_token("123:abc")?
        .build()?;

    let text_hits = Arc::new(AtomicUsize::new(0));
    let callback_hits = Arc::new(AtomicUsize::new(0));

    let mut router = Router::new();
    {
        let callback_hits = Arc::clone(&callback_hits);
        router.on_extracted::<CallbackInput, _, _>(
            move |_context: BotContext, _update: Update, callback| {
                let callback_hits = Arc::clone(&callback_hits);
                async move {
                    if callback.into_inner() == r#"{"action":"ok"}"# {
                        callback_hits.fetch_add(1, Ordering::SeqCst);
                    }
                    Ok(())
                }
            },
        );
    }
    {
        let text_hits = Arc::clone(&text_hits);
        router.on_extracted::<TextInput, _, _>(
            move |_context: BotContext, _update: Update, text| {
                let text_hits = Arc::clone(&text_hits);
                async move {
                    if text.into_inner() == "hello" {
                        text_hits.fetch_add(1, Ordering::SeqCst);
                    }
                    Ok(())
                }
            },
        );
    }

    let Some(text_update) = parse_update(message_update(4001, 10, "hello")) else {
        return Ok(());
    };
    let Some(callback_evt) = parse_update(callback_update(4002, 10, r#"{"action":"ok"}"#)) else {
        return Ok(());
    };

    assert!(
        router
            .dispatch(BotContext::new(client.clone()), text_update)
            .await?
    );
    assert!(
        router
            .dispatch(BotContext::new(client), callback_evt)
            .await?
    );
    assert_eq!(text_hits.load(Ordering::SeqCst), 1);
    assert_eq!(callback_hits.load(Ordering::SeqCst), 1);

    Ok(())
}

#[tokio::test]
async fn extractor_combinators_filter_map_guard_work() -> Result<(), DynError> {
    let client = Client::builder("http://127.0.0.1:9")?
        .bot_token("123:abc")?
        .build()?;

    let filter_hits = Arc::new(AtomicUsize::new(0));
    let mut filter_router = Router::new();
    {
        let filter_hits = Arc::clone(&filter_hits);
        filter_router.on_extracted_filter::<TextInput, _, _, _>(
            |text, _update| text.0.starts_with("allow"),
            move |_context: BotContext, _update: Update, _text| {
                let filter_hits = Arc::clone(&filter_hits);
                async move {
                    filter_hits.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                }
            },
        );
    }
    let Some(filter_skip) = parse_update(message_update(4401, 1, "deny text")) else {
        return Ok(());
    };
    let Some(filter_hit) = parse_update(message_update(4402, 1, "allow text")) else {
        return Ok(());
    };
    assert!(
        !filter_router
            .dispatch(BotContext::new(client.clone()), filter_skip)
            .await?
    );
    assert!(
        filter_router
            .dispatch(BotContext::new(client.clone()), filter_hit)
            .await?
    );
    assert_eq!(filter_hits.load(Ordering::SeqCst), 1);

    let map_hits = Arc::new(AtomicUsize::new(0));
    let mut map_router = Router::new();
    {
        let map_hits = Arc::clone(&map_hits);
        map_router.on_extracted_map::<CallbackInput, String, _, _, _>(
            |callback, _update| {
                let value: serde_json::Value = serde_json::from_str(&callback.0).ok()?;
                Some(value.get("action")?.as_str()?.to_owned())
            },
            move |_context: BotContext, _update: Update, action: String| {
                let map_hits = Arc::clone(&map_hits);
                async move {
                    if action == "confirm" {
                        map_hits.fetch_add(1, Ordering::SeqCst);
                    }
                    Ok(())
                }
            },
        );
    }
    let Some(map_skip) = parse_update(callback_update(4403, 1, "not-json")) else {
        return Ok(());
    };
    let Some(map_hit) = parse_update(callback_update(4404, 1, r#"{"action":"confirm"}"#)) else {
        return Ok(());
    };
    assert!(
        !map_router
            .dispatch(BotContext::new(client.clone()), map_skip)
            .await?
    );
    assert!(
        map_router
            .dispatch(BotContext::new(client.clone()), map_hit)
            .await?
    );
    assert_eq!(map_hits.load(Ordering::SeqCst), 1);

    let guard_hits = Arc::new(AtomicUsize::new(0));
    let mut guard_router = Router::new();
    {
        let guard_hits = Arc::clone(&guard_hits);
        guard_router.on_extracted_guard::<TextInput, _, _, _>(
            |text, _update| {
                if text.0.contains("blocked") {
                    return Err(HandlerError::internal(Error::InvalidRequest {
                        reason: "blocked by guard".to_owned(),
                    }));
                }
                Ok(())
            },
            move |_context: BotContext, _update: Update, _text| {
                let guard_hits = Arc::clone(&guard_hits);
                async move {
                    guard_hits.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                }
            },
        );
    }
    let Some(guard_hit) = parse_update(message_update(4405, 1, "allowed")) else {
        return Ok(());
    };
    let Some(guard_blocked) = parse_update(message_update(4406, 1, "blocked")) else {
        return Ok(());
    };
    assert!(
        guard_router
            .dispatch(BotContext::new(client.clone()), guard_hit)
            .await?
    );
    let guard_error = guard_router
        .dispatch(BotContext::new(client), guard_blocked)
        .await
        .err();
    assert!(matches!(
        guard_error,
        Some(Error::InvalidRequest { reason }) if reason.contains("blocked")
    ));
    assert_eq!(guard_hits.load(Ordering::SeqCst), 1);

    Ok(())
}

#[tokio::test]
async fn route_with_policy_replies_user_on_error() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":{"message_id":120,"date":1710000009,"chat":{"id":10,"type":"private"},"text":"temporary failure"}}"#;
    let (base_url, handle) = spawn_server("/bot123:abc/sendMessage", 200, response)?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let mut router = Router::new();
    router.on_command_with_policy(
        "start",
        ErrorPolicy::ReplyUser {
            fallback_message: "temporary failure".to_owned(),
        },
        |_context: BotContext, _update: Update| async move {
            Err(Error::Transport {
                method: "sendMessage".to_owned(),
                status: Some(502),
                request_id: None,
                retry_after: None,
                request_path: None,
                message: "upstream unavailable".to_owned(),
            })
        },
    );

    let Some(update) = parse_update(message_update(4101, 10, "/start")) else {
        return Ok(());
    };

    assert!(router.dispatch(BotContext::new(client), update).await?);
    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn outbox_dedupes_and_retries() -> Result<(), DynError> {
    let retry_response = r#"{"ok":false,"error_code":429,"description":"too many requests","parameters":{"retry_after":1}}"#;
    let ok_response = r#"{"ok":true,"result":{"message_id":88,"date":1710000010,"chat":{"id":12,"type":"private"},"text":"hello"}}"#;
    let (base_url, handle) = spawn_server_sequence(
        "/bot123:abc/sendMessage",
        vec![(429, retry_response), (200, ok_response)],
    )?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let mut config = OutboxConfig::default();
    config.max_attempts = 3;
    config.dedupe_ttl = Duration::from_secs(60);
    let outbox = BotOutbox::spawn(client, config);

    let first = outbox
        .send_text_with_key(12_i64, "hello", Some("msg-1".to_owned()))
        .await?;
    let second = outbox
        .send_text_with_key(12_i64, "hello", Some("msg-1".to_owned()))
        .await?;

    assert_eq!(first.message_id, second.message_id);
    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn outbox_replays_persisted_messages_on_start() -> Result<(), DynError> {
    let ok_response = r#"{"ok":true,"result":{"message_id":188,"date":1710000011,"chat":{"id":12,"type":"private"},"text":"hello persisted"}}"#;
    let (base_url, handle) = spawn_server("/bot123:abc/sendMessage", 200, ok_response)?;

    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let path = std::env::temp_dir().join(format!("tele-outbox-{timestamp}.json"));
    let snapshot = serde_json::json!({
        "version": 1,
        "queue": [{
            "chat_id": 12,
            "text": "hello persisted",
            "idempotency_key": "persisted-1",
            "attempt": 0
        }]
    });
    fs::write(&path, serde_json::to_vec(&snapshot)?)?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let config = OutboxConfig::default().with_persistence_path(path.clone());
    let _outbox = BotOutbox::spawn(client, config);

    tokio::time::sleep(Duration::from_millis(120)).await;
    join_server(handle)?;

    let raw = fs::read(&path)?;
    let snapshot: serde_json::Value = serde_json::from_slice(&raw)?;
    assert_eq!(
        snapshot
            .get("queue")
            .and_then(serde_json::Value::as_array)
            .map(|queue| queue.len()),
        Some(0)
    );

    let _ = fs::remove_file(path);
    Ok(())
}

#[tokio::test]
async fn outbox_writes_dead_letter_on_exhausted_failures() -> Result<(), DynError> {
    let failure = r#"{"ok":false,"error_code":500,"description":"internal error"}"#;
    let (base_url, handle) = spawn_server("/bot123:abc/sendMessage", 500, failure)?;

    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let dead_letter_path = std::env::temp_dir().join(format!("tele-dead-letter-{timestamp}.json"));

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let mut config = OutboxConfig::default().with_dead_letter_path(dead_letter_path.clone());
    config.max_attempts = 1;
    let outbox = BotOutbox::spawn(client, config);

    let result = outbox
        .send_text_with_key(12_i64, "will fail", Some("dead-letter-1".to_owned()))
        .await;
    assert!(result.is_err());

    let raw = fs::read(&dead_letter_path)?;
    let snapshot: serde_json::Value = serde_json::from_slice(&raw)?;
    let entries = snapshot
        .get("entries")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    assert_eq!(entries.len(), 1);
    assert_eq!(
        entries[0]
            .get("idempotency_key")
            .and_then(serde_json::Value::as_str),
        Some("dead-letter-1")
    );

    join_server(handle)?;
    let _ = fs::remove_file(dead_letter_path);
    Ok(())
}

#[tokio::test]
async fn outbox_expires_persisted_message_and_moves_to_dead_letter() -> Result<(), DynError> {
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let queue_path = std::env::temp_dir().join(format!("tele-outbox-expire-{timestamp}.json"));
    let dead_letter_path =
        std::env::temp_dir().join(format!("tele-dead-letter-expire-{timestamp}.json"));

    let snapshot = serde_json::json!({
        "version": 1,
        "queue": [{
            "chat_id": 88,
            "text": "stale message",
            "idempotency_key": "expire-1",
            "enqueued_at_unix_ms": 1,
            "attempt": 0,
            "last_error": null
        }]
    });
    fs::write(&queue_path, serde_json::to_vec(&snapshot)?)?;

    let client = Client::builder("http://127.0.0.1:9")?
        .bot_token("123:abc")?
        .build()?;
    let config = OutboxConfig::default()
        .with_persistence_path(queue_path.clone())
        .with_dead_letter_path(dead_letter_path.clone())
        .with_max_message_age(Some(Duration::from_millis(1)));
    let _outbox = BotOutbox::spawn(client, config);

    tokio::time::sleep(Duration::from_millis(120)).await;

    let raw_queue = fs::read(&queue_path)?;
    let queue_snapshot: serde_json::Value = serde_json::from_slice(&raw_queue)?;
    assert_eq!(
        queue_snapshot
            .get("queue")
            .and_then(serde_json::Value::as_array)
            .map(|queue| queue.len()),
        Some(0)
    );

    let raw_dead_letter = fs::read(&dead_letter_path)?;
    let dead_letter_snapshot: serde_json::Value = serde_json::from_slice(&raw_dead_letter)?;
    let entries = dead_letter_snapshot
        .get("entries")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    assert_eq!(entries.len(), 1);
    assert!(
        entries[0]
            .get("reason")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|reason| reason.contains("expired"))
    );

    let _ = fs::remove_file(queue_path);
    let _ = fs::remove_file(dead_letter_path);
    Ok(())
}

#[tokio::test]
async fn json_file_session_store_persists_across_instances() -> Result<(), DynError> {
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let path = std::env::temp_dir().join(format!("tele-session-{timestamp}.json"));

    let Some(update) = parse_update(message_update(4201, 77, "state")) else {
        return Ok(());
    };

    let store = JsonFileSessionStore::<String>::open(&path)?;
    save_chat_state(&store, &update, "step-1".to_owned()).await?;

    let reopened = JsonFileSessionStore::<String>::open(&path)?;
    let loaded = load_chat_state(&reopened, &update).await?;
    assert_eq!(loaded.as_deref(), Some("step-1"));

    let _ = fs::remove_file(path);
    Ok(())
}

#[tokio::test]
async fn bot_engine_emits_events_and_testing_harness_dispatches() -> Result<(), DynError> {
    let client = Client::builder("http://127.0.0.1:9")?
        .bot_token("123:abc")?
        .build()?;

    let mut router = Router::new();
    router.on_message(|_context: BotContext, _update: Update| async move { Ok(()) });

    let harness = tele::bot::testing::BotHarness::new(router.clone())?;
    let fixture = tele::bot::testing::message_update(4301, 1, "hello")?;
    let outcome = harness.dispatch(fixture).await?;
    assert_eq!(outcome, DispatchOutcome::Handled { update_id: 4301 });

    let events = Arc::new(Mutex::new(Vec::<EngineEvent>::new()));
    let (sink, source) = channel_source(4);
    let mut engine = BotEngine::new(client, source, router)
        .with_config(EngineConfig {
            continue_on_source_error: false,
            continue_on_handler_error: false,
            ..EngineConfig::default()
        })
        .on_event({
            let events = Arc::clone(&events);
            move |event| {
                if let Ok(mut guard) = events.lock() {
                    guard.push(event.clone());
                }
            }
        });

    let fixture = tele::bot::testing::message_update(4302, 1, "hello")?;
    sink.send(fixture).await?;
    let outcomes = engine.poll_once().await?;
    assert_eq!(outcomes, vec![DispatchOutcome::Handled { update_id: 4302 }]);

    let captured = events.lock().map_err(|error| error.to_string())?;
    assert!(captured.contains(&EngineEvent::PollStarted));
    assert!(captured.contains(&EngineEvent::PollCompleted { update_count: 1 }));
    assert!(captured.contains(&EngineEvent::DispatchStarted { update_id: 4302 }));
    assert!(captured.contains(&EngineEvent::DispatchCompleted {
        outcome: DispatchOutcome::Handled { update_id: 4302 }
    }));

    Ok(())
}

#[cfg(feature = "redis-session")]
#[tokio::test]
async fn redis_session_store_rejects_empty_namespace() -> Result<(), DynError> {
    let error = tele::bot::RedisSessionStore::<String>::new("redis://127.0.0.1/", "   ").err();
    assert!(matches!(
        error,
        Some(Error::InvalidRequest { reason }) if reason.contains("namespace")
    ));
    Ok(())
}

#[cfg(feature = "postgres-session")]
#[tokio::test]
async fn postgres_session_store_rejects_invalid_table_name() -> Result<(), DynError> {
    let error = tele::bot::PostgresSessionStore::<String>::connect(
        "postgres://127.0.0.1:5432/postgres",
        "invalid-table",
    )
    .await
    .err();
    assert!(matches!(
        error,
        Some(Error::InvalidRequest { reason }) if reason.contains("identifier")
    ));
    Ok(())
}

#[tokio::test]
async fn long_polling_source_offset_never_moves_backward() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":[{"update_id":5001,"message":{"message_id":1,"date":1710000011,"chat":{"id":9,"type":"private"},"text":"a"}},{"update_id":4999,"message":{"message_id":2,"date":1710000012,"chat":{"id":9,"type":"private"},"text":"b"}}]}"#;
    let (base_url, handle) = spawn_server("/bot123:abc/getUpdates", 200, response)?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let source = LongPollingSource::new(client.clone()).with_config(PollingConfig {
        disable_webhook_on_start: false,
        poll_timeout_seconds: 1,
        ..PollingConfig::default()
    });
    let mut engine = BotEngine::new(client, source, Router::new()).with_config(EngineConfig {
        continue_on_source_error: false,
        continue_on_handler_error: false,
        max_handler_concurrency: 2,
        ..EngineConfig::default()
    });

    let outcomes = engine.poll_once().await?;
    assert_eq!(outcomes.len(), 2);
    assert_eq!(engine.source_mut().next_offset(), Some(5002));

    join_server(handle)?;
    Ok(())
}

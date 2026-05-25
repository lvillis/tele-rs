#![cfg(feature = "bot")]

use std::fs;
use std::io::ErrorKind;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::json;
use tele::Client;
use tele::bot::testing::BotHarness;
use tele::bot::{
    BotApp, BotContext, BotEngine, BotOutbox, BusinessConnectionInput,
    BusinessMessagesDeletedInput, CURRENT_ACTOR_CHAT_MEMBER, CURRENT_BOT_CHAT_MEMBER,
    CallbackInput, CallbackQueryInput, ChatBoostInput, ChatJoinRequestInput,
    ChatMemberUpdatedInput, ChatSession, CommandArgs, CommandData, DispatchMetricOutcome,
    DispatchOutcome, EngineConfig, EngineEvent, EngineMetric, ErrorPolicy, HandlerError,
    InMemorySessionStore, InlineQueryInput, JsonFileSessionStore, LongPollingSource,
    ManagedBotInput, MessageReactionInput, MyChatMemberUpdatedInput, OutboxConfig, PollAnswerInput,
    PollingConfig, RequestStateKey, Router, ShippingQueryInput, SourceErrorBackoffConfig,
    StateTransition, TextInput, UpdateExt, UpdateExtractor, UpdateSource, WebAppInput,
    WebhookRunner, WriteAccessAllowedInput, apply_chat_state_transition, channel_source,
    clear_chat_state, extract_callback_data, extract_callback_json, extract_chat_join_request,
    extract_chat_member_update, extract_command, extract_command_args, extract_command_data,
    extract_compact_callback, extract_message, extract_my_chat_member_update, extract_text,
    extract_typed_callback, extract_web_app_data, extract_write_access_allowed, load_chat_state,
    parse_command_text, parse_command_text_for_bot, save_chat_state, tokenize_command_args,
};
use tele::types::advanced::AdvancedSetChatMenuButtonRequest;
use tele::types::telegram::{
    CompactCallbackDecoder, CompactCallbackEncoder, CompactCallbackPayload, InlineKeyboardButton,
    InlineQueryResult, InlineQueryResultsButton, KeyboardButton, MenuButton, MenuButtonWebApp,
    WebAppInfo,
};
use tele::types::update::Update;
use tele::types::{
    AllowedUpdate, ChatAdministratorCapability, ChatMember, MessageKind, UpdateKind,
};
use tele::{BootstrapStepStatus, Error, ErrorClass, MenuButtonConfig};

type DynError = Box<dyn std::error::Error>;
type ServerHandle = thread::JoinHandle<Result<(), String>>;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct DemoCallbackPayload {
    action: String,
    target: i64,
}

impl CompactCallbackPayload for DemoCallbackPayload {
    fn encode_compact(&self, encoder: &mut CompactCallbackEncoder) -> tele::Result<()> {
        encoder
            .tag("demo")?
            .push(self.action.as_str())?
            .push_display(self.target)?;
        Ok(())
    }

    fn decode_compact(decoder: &mut CompactCallbackDecoder) -> tele::Result<Self> {
        decoder.expect_tag("demo")?;
        Ok(Self {
            action: decoder.next_string("action")?,
            target: decoder.next_parse("target")?,
        })
    }
}

#[test]
fn command_args_contract_distinguishes_required_and_optional() {
    assert_eq!(
        <String as CommandArgs>::parse("  hello world  "),
        Ok("hello world".to_owned())
    );
    assert!(<String as CommandArgs>::parse("   ").is_err());
    assert_eq!(<Vec<String> as CommandArgs>::parse("   "), Ok(Vec::new()));
    assert_eq!(<Option<i64> as CommandArgs>::parse("   "), Ok(None));
    assert_eq!(<Option<i64> as CommandArgs>::parse(" 42 "), Ok(Some(42)));
    assert!(<Option<i64> as CommandArgs>::parse("oops").is_err());
}

fn accept_with_timeout(
    listener: &TcpListener,
    timeout: Duration,
) -> Result<(std::net::TcpStream, std::net::SocketAddr), String> {
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
            Err(error) if error.kind() == ErrorKind::WouldBlock => {
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

async fn wait_for_condition<F>(
    timeout: Duration,
    poll_interval: Duration,
    mut condition: F,
) -> Result<(), DynError>
where
    F: FnMut() -> Result<bool, DynError>,
{
    let deadline = tokio::time::Instant::now() + timeout;
    loop {
        if condition()? {
            return Ok(());
        }
        if tokio::time::Instant::now() >= deadline {
            return Err(format!(
                "timed out waiting for condition after {}ms",
                timeout.as_millis()
            )
            .into());
        }
        tokio::time::sleep(poll_interval).await;
    }
}

fn blocked_child_path(prefix: &str, child_name: &str) -> Result<(PathBuf, PathBuf), DynError> {
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let root = std::env::temp_dir().join(format!("{prefix}-{timestamp}"));
    fs::create_dir_all(&root)?;
    let blocked_parent = root.join("not-a-directory");
    fs::write(&blocked_parent, b"not a directory")?;
    Ok((root, blocked_parent.join(child_name)))
}

fn spawn_server(
    expected_path: &'static str,
    response_status: u16,
    response_body: &'static str,
) -> Result<(String, ServerHandle), DynError> {
    spawn_server_with_request_hook(expected_path, response_status, response_body, |_| Ok(()))
}

fn spawn_server_with_request_hook<F>(
    expected_path: &'static str,
    response_status: u16,
    response_body: &'static str,
    hook: F,
) -> Result<(String, ServerHandle), DynError>
where
    F: FnOnce(&str) -> Result<(), String> + Send + 'static,
{
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let address = listener.local_addr()?;

    let handle = thread::spawn(move || {
        let (mut stream, _) = accept_with_timeout(&listener, Duration::from_secs(3))?;
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
        hook(&request)?;

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
    spawn_server_with_request_hook(
        expected_path,
        response_status,
        response_body,
        move |request| {
            for required in required_substrings {
                if !request.contains(required) {
                    return Err(format!(
                        "request missing required content `{required}`: {request}"
                    ));
                }
            }

            Ok(())
        },
    )
}

fn spawn_server_sequence(
    expected_path: &'static str,
    responses: Vec<(u16, &'static str)>,
) -> Result<(String, ServerHandle), DynError> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let address = listener.local_addr()?;

    let handle = thread::spawn(move || {
        for (response_status, response_body) in responses {
            let (mut stream, _) = accept_with_timeout(&listener, Duration::from_secs(3))?;
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

async fn join_server(handle: ServerHandle) -> Result<(), DynError> {
    tokio::task::spawn_blocking(move || match handle.join() {
        Ok(result) => result,
        Err(_) => Err("server thread panicked".to_owned()),
    })
    .await
    .map_err(|error| format!("failed to join server task: {error}"))?
    .map_err(Into::into)
}

fn parse_update(input: serde_json::Value) -> Option<Update> {
    serde_json::from_value(input).ok()
}

fn reject_blocked_text(text: &TextInput, _update: &Update) -> tele::bot::HandlerResult {
    if text.0.contains("blocked") {
        return Err(HandlerError::internal(Error::InvalidRequest {
            reason: "blocked by guard".to_owned(),
        }));
    }
    Ok(())
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

fn group_message_update(
    update_id: i64,
    chat_id: i64,
    user_id: i64,
    text: &str,
) -> serde_json::Value {
    json!({
        "update_id": update_id,
        "message": {
            "message_id": update_id,
            "date": 1700000000 + update_id,
            "chat": {"id": chat_id, "type": "supergroup", "title": "ops"},
            "from": {
                "id": user_id,
                "is_bot": false,
                "first_name": "moderator"
            },
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

fn inaccessible_callback_update(update_id: i64, chat_id: i64, data: &str) -> serde_json::Value {
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
                "date": 0,
                "chat": {"id": chat_id, "type": "supergroup", "title": "ops"}
            },
            "data": data
        }
    })
}

fn capability_field(capability: ChatAdministratorCapability) -> &'static str {
    match capability {
        ChatAdministratorCapability::ManageChat => "can_manage_chat",
        ChatAdministratorCapability::DeleteMessages => "can_delete_messages",
        ChatAdministratorCapability::ManageVideoChats => "can_manage_video_chats",
        ChatAdministratorCapability::RestrictMembers => "can_restrict_members",
        ChatAdministratorCapability::PromoteMembers => "can_promote_members",
        ChatAdministratorCapability::ChangeInfo => "can_change_info",
        ChatAdministratorCapability::InviteUsers => "can_invite_users",
        ChatAdministratorCapability::PostStories => "can_post_stories",
        ChatAdministratorCapability::EditStories => "can_edit_stories",
        ChatAdministratorCapability::DeleteStories => "can_delete_stories",
        ChatAdministratorCapability::PostMessages => "can_post_messages",
        ChatAdministratorCapability::EditMessages => "can_edit_messages",
        ChatAdministratorCapability::PinMessages => "can_pin_messages",
        ChatAdministratorCapability::ManageTopics => "can_manage_topics",
        _ => "unknown_permission",
    }
}

fn chat_member_with_capabilities(
    user_id: i64,
    is_bot: bool,
    status: &str,
    capabilities: &[ChatAdministratorCapability],
) -> Result<ChatMember, DynError> {
    let mut value = serde_json::Map::new();
    let mut user = serde_json::Map::new();
    let mut manage_chat = None;

    let _ = value.insert("status".to_owned(), json!(status));
    let _ = user.insert("id".to_owned(), json!(user_id));
    let _ = user.insert("is_bot".to_owned(), json!(is_bot));
    let _ = user.insert(
        "first_name".to_owned(),
        json!(if is_bot { "tele" } else { "moderator" }),
    );
    let _ = value.insert("user".to_owned(), serde_json::Value::Object(user));

    for capability in capabilities {
        let field = capability_field(*capability);
        if field == "can_manage_chat" {
            manage_chat = Some(true);
        } else if field != "unknown_permission" {
            let _ = value.insert(field.to_owned(), json!(true));
        }
    }
    if let Some(manage_chat) = manage_chat {
        let _ = value.insert("can_manage_chat".to_owned(), json!(manage_chat));
    }

    serde_json::from_value(serde_json::Value::Object(value)).map_err(Into::into)
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
        router
            .command_route("start")?
            .handle(move |_context: BotContext, _update: Update| {
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
            mention: None,
            args: "hello world".to_owned()
        })
    );
    assert_eq!(
        parse_command_text(" \t/echo hello world"),
        Some(CommandData {
            name: "echo".to_owned(),
            mention: None,
            args: "hello world".to_owned()
        })
    );
    assert_eq!(parse_command_text("/echo@ hello world"), None);
    assert_eq!(parse_command_text("/echo@Bad@Name hello world"), None);
    assert_eq!(
        parse_command_text_for_bot("/echo@@ThisBot hi", Some("ThisBot")),
        None
    );
    assert_eq!(
        parse_command_text_for_bot("/echo@nope hi", Some("Nope")),
        None
    );
    assert_eq!(
        parse_command_text_for_bot("/echo@RegularUser hi", Some("RegularUser")),
        None
    );
    assert_eq!(
        parse_command_text_for_bot("/echo@ThisBot hi", Some("@thisbot")),
        Some(CommandData {
            name: "echo".to_owned(),
            mention: Some("ThisBot".to_owned()),
            args: "hi".to_owned()
        })
    );
    assert_eq!(parse_command_text("/Echo hello world"), None);

    assert!(extract_message(&update).is_some());
    assert_eq!(update.update_kind(), UpdateKind::Message);
    assert_eq!(update.message_kind(), Some(MessageKind::Text));
    assert_eq!(extract_text(&update), Some("/echo hello world"));
    assert_eq!(extract_command(&update), Some("echo"));
    assert_eq!(extract_command_args(&update), Some("hello world"));
    assert_eq!(
        extract_command_data(&update),
        Some(CommandData {
            name: "echo".to_owned(),
            mention: None,
            args: "hello world".to_owned()
        })
    );
    assert_eq!(update.command(), Some("echo"));
    assert_eq!(update.command_args(), Some("hello world"));
    assert_eq!(update.text(), Some("/echo hello world"));
    assert_eq!(update.chat_id(), Some(1));
    let Some(spaced_update) = parse_update(message_update(202, 1, " \t/echo spaced args")) else {
        return Ok(());
    };
    assert_eq!(extract_command(&spaced_update), Some("echo"));
    assert_eq!(extract_command_args(&spaced_update), Some("spaced args"));
    assert_eq!(
        extract_command_data(&spaced_update),
        Some(CommandData {
            name: "echo".to_owned(),
            mention: None,
            args: "spaced args".to_owned()
        })
    );
    let Some(invalid_mention_update) = parse_update(message_update(201, 1, "/echo@ hello")) else {
        return Ok(());
    };
    assert_eq!(extract_command(&invalid_mention_update), None);
    assert_eq!(extract_command_args(&invalid_mention_update), None);

    let Some(callback_with_command_message) = parse_update(json!({
        "update_id": 2042,
        "callback_query": {
            "id": "cb-command-message",
            "from": {"id": 99, "is_bot": false, "first_name": "tester"},
            "message": {
                "message_id": 2042,
                "date": 1700002042,
                "chat": {"id": 1, "type": "private"},
                "text": "/echo hidden"
            },
            "data": "clicked"
        }
    })) else {
        return Ok(());
    };
    assert!(extract_message(&callback_with_command_message).is_some());
    assert_eq!(
        extract_text(&callback_with_command_message),
        Some("/echo hidden")
    );
    assert_eq!(extract_command(&callback_with_command_message), None);
    assert_eq!(extract_command_args(&callback_with_command_message), None);
    assert_eq!(extract_command_data(&callback_with_command_message), None);

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
    assert_eq!(web_app_update.update_kind(), UpdateKind::Message);
    assert_eq!(web_app_update.message_kind(), Some(MessageKind::WebAppData));
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
        write_access_update.message_kind(),
        Some(MessageKind::WriteAccessAllowed)
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

    let Some(callback_web_app_message) = parse_update(serde_json::json!({
        "update_id": 204,
        "callback_query": {
            "id": "cb-web-app-message",
            "from": {"id": 99, "is_bot": false, "first_name": "tester"},
            "message": {
                "message_id": 4,
                "date": 1700000004,
                "chat": {"id": 1, "type": "private"},
                "web_app_data": {
                    "data": "{\"query_id\":\"q-from-callback\"}",
                    "button_text": "Old Mini App"
                }
            },
            "data": "clicked"
        }
    })) else {
        return Ok(());
    };
    assert!(extract_web_app_data(&callback_web_app_message).is_some());
    assert!(WebAppInput::extract(&callback_web_app_message).is_none());

    let maybe_callback = parse_update(callback_update(201, 1, "btn-1"));
    assert!(maybe_callback.is_some());
    let Some(callback) = maybe_callback else {
        return Ok(());
    };

    assert_eq!(extract_callback_data(&callback), Some("btn-1"));
    assert_eq!(callback.update_kind(), UpdateKind::CallbackQuery);
    assert_eq!(callback.message_kind(), Some(MessageKind::Text));
    assert!(extract_callback_json::<serde_json::Value>(&callback).is_none());
    assert_eq!(callback.callback_data(), Some("btn-1"));
    assert!(callback.message().is_some());

    let Some(inaccessible_callback) =
        parse_update(inaccessible_callback_update(203, -10042, "btn-2"))
    else {
        return Ok(());
    };
    assert_eq!(extract_callback_data(&inaccessible_callback), Some("btn-2"));
    assert_eq!(
        inaccessible_callback.update_kind(),
        UpdateKind::CallbackQuery
    );
    assert_eq!(inaccessible_callback.chat_id(), Some(-10042));
    assert_eq!(
        inaccessible_callback.chat().map(|chat| chat.id),
        Some(-10042)
    );
    assert!(inaccessible_callback.message().is_none());
    assert_eq!(inaccessible_callback.message_kind(), None);

    let maybe_join_request = parse_update(serde_json::json!({
        "update_id": 204,
        "chat_join_request": {
            "chat": {"id": -1001, "type": "supergroup", "title": "mods"},
            "from": {"id": 77, "is_bot": false, "first_name": "candidate"},
            "user_chat_id": 7001,
            "date": 1700000003,
            "bio": "please let me in"
        }
    }));
    assert!(maybe_join_request.is_some());
    let Some(join_request) = maybe_join_request else {
        return Ok(());
    };

    assert_eq!(join_request.update_kind(), UpdateKind::ChatJoinRequest);
    assert_eq!(join_request.chat_id(), Some(-1001));
    assert_eq!(join_request.user_id(), Some(77));
    assert_eq!(
        extract_chat_join_request(&join_request).map(|request| request.user_chat_id),
        Some(7001)
    );
    assert_eq!(
        join_request
            .chat_join_request()
            .and_then(|request| request.bio.as_deref()),
        Some("please let me in")
    );
    let join_request_extracted = ChatJoinRequestInput::extract(&join_request);
    assert_eq!(
        join_request_extracted
            .as_ref()
            .map(|request| request.0.chat.id),
        Some(-1001)
    );

    let maybe_chat_member_update = parse_update(serde_json::json!({
        "update_id": 205,
        "chat_member": {
            "chat": {"id": -1002, "type": "supergroup", "title": "mods"},
            "from": {"id": 12, "is_bot": false, "first_name": "admin"},
            "date": 1700000004,
            "old_chat_member": {
                "status": "left",
                "user": {"id": 78, "is_bot": false, "first_name": "candidate"}
            },
            "new_chat_member": {
                "status": "member",
                "user": {"id": 78, "is_bot": false, "first_name": "candidate"}
            },
            "via_join_request": true
        }
    }));
    assert!(maybe_chat_member_update.is_some());
    let Some(chat_member_update) = maybe_chat_member_update else {
        return Ok(());
    };

    assert_eq!(chat_member_update.update_kind(), UpdateKind::ChatMember);
    assert_eq!(chat_member_update.chat_id(), Some(-1002));
    assert_eq!(chat_member_update.actor_id(), Some(12));
    assert_eq!(chat_member_update.subject_id(), Some(78));
    assert_eq!(
        extract_chat_member_update(&chat_member_update).map(|update| update.subject().id.0),
        Some(78)
    );
    assert!(
        chat_member_update
            .chat_member_update()
            .is_some_and(|update| update.via_join_request)
    );
    let chat_member_extracted = ChatMemberUpdatedInput::extract(&chat_member_update);
    assert_eq!(
        chat_member_extracted
            .as_ref()
            .map(|update| update.0.chat.id),
        Some(-1002)
    );

    let maybe_my_chat_member_update = parse_update(serde_json::json!({
        "update_id": 206,
        "my_chat_member": {
            "chat": {"id": -1003, "type": "supergroup", "title": "ops"},
            "from": {"id": 13, "is_bot": false, "first_name": "owner"},
            "date": 1700000005,
            "old_chat_member": {
                "status": "member",
                "user": {"id": 999, "is_bot": true, "first_name": "tele"}
            },
            "new_chat_member": {
                "status": "administrator",
                "user": {"id": 999, "is_bot": true, "first_name": "tele"},
                "can_manage_chat": true
            }
        }
    }));
    assert!(maybe_my_chat_member_update.is_some());
    let Some(my_chat_member_update) = maybe_my_chat_member_update else {
        return Ok(());
    };

    assert_eq!(
        my_chat_member_update.update_kind(),
        UpdateKind::MyChatMember
    );
    assert_eq!(my_chat_member_update.chat_id(), Some(-1003));
    assert_eq!(my_chat_member_update.actor_id(), Some(13));
    assert_eq!(my_chat_member_update.subject_id(), Some(999));
    assert_eq!(
        extract_my_chat_member_update(&my_chat_member_update).map(|update| update.subject().id.0),
        Some(999)
    );
    assert!(
        my_chat_member_update
            .my_chat_member_update()
            .is_some_and(|update| update.member().is_admin())
    );
    let my_chat_member_extracted = MyChatMemberUpdatedInput::extract(&my_chat_member_update);
    assert_eq!(
        my_chat_member_extracted
            .as_ref()
            .map(|update| update.0.chat.id),
        Some(-1003)
    );

    let maybe_inline_query = parse_update(serde_json::json!({
        "update_id": 207,
        "inline_query": {
            "id": "inline-1",
            "from": {"id": 88, "is_bot": false, "first_name": "inline"},
            "query": "lookup",
            "offset": ""
        }
    }));
    assert!(maybe_inline_query.is_some());
    let Some(inline_query) = maybe_inline_query else {
        return Ok(());
    };
    assert_eq!(inline_query.update_kind(), UpdateKind::InlineQuery);
    assert_eq!(inline_query.user_id(), Some(88));

    let maybe_chosen_inline_result = parse_update(serde_json::json!({
        "update_id": 208,
        "chosen_inline_result": {
            "result_id": "result-1",
            "from": {"id": 89, "is_bot": false, "first_name": "chooser"},
            "query": "lookup"
        }
    }));
    assert!(maybe_chosen_inline_result.is_some());
    let Some(chosen_inline_result) = maybe_chosen_inline_result else {
        return Ok(());
    };
    assert_eq!(
        chosen_inline_result.update_kind(),
        UpdateKind::ChosenInlineResult
    );
    assert_eq!(chosen_inline_result.user_id(), Some(89));

    let maybe_poll_answer = parse_update(serde_json::json!({
        "update_id": 209,
        "poll_answer": {
            "poll_id": "poll-1",
            "user": {"id": 90, "is_bot": false, "first_name": "voter"},
            "option_ids": [1]
        }
    }));
    assert!(maybe_poll_answer.is_some());
    let Some(poll_answer) = maybe_poll_answer else {
        return Ok(());
    };
    assert_eq!(poll_answer.update_kind(), UpdateKind::PollAnswer);
    assert_eq!(poll_answer.user_id(), Some(90));

    let maybe_anonymous_poll_answer = parse_update(serde_json::json!({
        "update_id": 210,
        "poll_answer": {
            "poll_id": "poll-2",
            "voter_chat": {"id": -10055, "type": "supergroup", "title": "ops"},
            "option_ids": [1]
        }
    }));
    assert!(maybe_anonymous_poll_answer.is_some());
    let Some(anonymous_poll_answer) = maybe_anonymous_poll_answer else {
        return Ok(());
    };
    assert_eq!(anonymous_poll_answer.update_kind(), UpdateKind::PollAnswer);
    assert_eq!(anonymous_poll_answer.chat_id(), Some(-10055));
    assert_eq!(
        anonymous_poll_answer.chat().map(|chat| chat.id),
        Some(-10055)
    );
    assert_eq!(anonymous_poll_answer.user_id(), None);

    let Some(reaction_update) = parse_update(serde_json::json!({
        "update_id": 211,
        "message_reaction": {
            "chat": {"id": -10056, "type": "supergroup", "title": "ops"},
            "message_id": 10,
            "user": {"id": 91, "is_bot": false, "first_name": "reactor"},
            "date": 1700000211,
            "old_reaction": [],
            "new_reaction": [{"type": "emoji", "emoji": "👍"}]
        }
    })) else {
        return Ok(());
    };
    assert_eq!(reaction_update.update_kind(), UpdateKind::MessageReaction);
    assert_eq!(reaction_update.chat_id(), Some(-10056));
    assert_eq!(reaction_update.user_id(), Some(91));
    assert_eq!(reaction_update.message_kind(), None);

    let Some(chat_boost_update) = parse_update(serde_json::json!({
        "update_id": 212,
        "chat_boost": {
            "chat": {"id": -10057, "type": "supergroup", "title": "ops"},
            "boost": {
                "boost_id": "boost-1",
                "add_date": 1700000212,
                "expiration_date": 1700086612,
                "source": {
                    "source": "premium",
                    "user": {"id": 92, "is_bot": false, "first_name": "booster"}
                }
            }
        }
    })) else {
        return Ok(());
    };
    assert_eq!(chat_boost_update.update_kind(), UpdateKind::ChatBoost);
    assert_eq!(chat_boost_update.chat_id(), Some(-10057));
    assert_eq!(chat_boost_update.user_id(), Some(92));

    let Some(shipping_update) = parse_update(serde_json::json!({
        "update_id": 213,
        "shipping_query": {
            "id": "shipping-1",
            "from": {"id": 93, "is_bot": false, "first_name": "buyer"},
            "invoice_payload": "payload",
            "shipping_address": {
                "country_code": "US",
                "state": "CA",
                "city": "San Francisco",
                "street_line1": "1 Market St",
                "street_line2": "Suite 1",
                "post_code": "94105"
            }
        }
    })) else {
        return Ok(());
    };
    assert_eq!(shipping_update.update_kind(), UpdateKind::ShippingQuery);
    assert_eq!(shipping_update.user_id(), Some(93));
    assert_eq!(shipping_update.chat_id(), None);

    Ok(())
}

#[tokio::test]
async fn business_update_extractors_work() -> Result<(), DynError> {
    let Some(message_update) = parse_update(json!({
        "update_id": 212,
        "business_message": {
            "message_id": 77,
            "business_connection_id": "business-1",
            "date": 1700000212,
            "chat": {"id": 7001, "type": "private", "first_name": "customer"},
            "from": {"id": 7001, "is_bot": false, "first_name": "customer"},
            "text": "/start"
        }
    })) else {
        return Ok(());
    };

    assert_eq!(message_update.update_kind(), UpdateKind::BusinessMessage);
    assert_eq!(message_update.text(), Some("/start"));
    assert_eq!(message_update.command(), Some("start"));
    assert_eq!(message_update.chat_id(), Some(7001));

    let Some(deleted_update) = parse_update(json!({
        "update_id": 213,
        "deleted_business_messages": {
            "business_connection_id": "business-1",
            "chat": {"id": 7001, "type": "private", "first_name": "customer"},
            "message_ids": [77, 78]
        }
    })) else {
        return Ok(());
    };

    assert_eq!(
        deleted_update.update_kind(),
        UpdateKind::DeletedBusinessMessages
    );
    assert_eq!(deleted_update.chat_id(), Some(7001));
    let Some(deleted) = deleted_update.deleted_business_messages() else {
        return Ok(());
    };
    assert_eq!(deleted.message_ids.len(), 2);

    Ok(())
}

#[tokio::test]
async fn bot_context_request_state_flows_through_middleware() -> Result<(), DynError> {
    #[derive(Clone, Debug, Eq, PartialEq)]
    struct TraceId(u64);

    const TRACE_ID: RequestStateKey<TraceId> = RequestStateKey::new("trace_id");
    const SECOND_TRACE_ID: RequestStateKey<TraceId> = RequestStateKey::new("second_trace_id");

    let client = Client::builder("http://127.0.0.1:9")?
        .bot_token("123:abc")?
        .build()?;
    let hits = Arc::new(AtomicUsize::new(0));

    let mut router = Router::new();
    router.middleware(|context, update, next| async move {
        let _ = context.request_state().slot(TRACE_ID).set(TraceId(42));
        let _ = context
            .request_state()
            .slot(SECOND_TRACE_ID)
            .set(TraceId(7));
        next(context, update).await
    });
    {
        let hits = Arc::clone(&hits);
        router
            .message_route()
            .handle(move |context: BotContext, _update: Update| {
                let hits = Arc::clone(&hits);
                async move {
                    assert_eq!(
                        context
                            .request_state()
                            .slot(TRACE_ID)
                            .cloned()
                            .map(|value| value.0),
                        Some(42)
                    );
                    assert_eq!(
                        context
                            .request_state()
                            .slot(SECOND_TRACE_ID)
                            .cloned()
                            .map(|value| value.0),
                        Some(7)
                    );
                    hits.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                }
            });
    }

    let Some(update) = parse_update(message_update(204, 1, "hello")) else {
        return Ok(());
    };

    assert!(router.dispatch(BotContext::new(client), update).await?);
    assert_eq!(hits.load(Ordering::SeqCst), 1);
    Ok(())
}

#[tokio::test]
async fn typed_callback_button_and_route_round_trip() -> Result<(), DynError> {
    let payload = DemoCallbackPayload {
        action: "confirm".to_owned(),
        target: 7,
    };
    let button = InlineKeyboardButton::typed_callback("Confirm", &payload)?;
    assert_eq!(
        button.decode_callback::<DemoCallbackPayload>()?,
        Some(payload.clone())
    );

    let Some(update) = parse_update(callback_update(
        205,
        1,
        button.callback_data().unwrap_or_default(),
    )) else {
        return Ok(());
    };
    assert_eq!(
        extract_typed_callback::<DemoCallbackPayload>(&update),
        Some(payload.clone())
    );
    assert_eq!(
        update.typed_callback::<DemoCallbackPayload>(),
        Some(payload.clone())
    );

    let client = Client::builder("http://127.0.0.1:9")?
        .bot_token("123:abc")?
        .build()?;
    let hits = Arc::new(AtomicUsize::new(0));
    let mut router = Router::new();
    {
        let hits = Arc::clone(&hits);
        let expected = payload.clone();
        router.typed_callback_route::<DemoCallbackPayload>().handle(
            move |_context: BotContext, _update: Update, callback| {
                let hits = Arc::clone(&hits);
                let expected = expected.clone();
                async move {
                    assert_eq!(callback.payload, expected);
                    hits.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                }
            },
        );
    }

    assert!(router.dispatch(BotContext::new(client), update).await?);
    assert_eq!(hits.load(Ordering::SeqCst), 1);
    Ok(())
}

#[tokio::test]
async fn compact_callback_button_and_route_round_trip() -> Result<(), DynError> {
    let payload = DemoCallbackPayload {
        action: "confirm".to_owned(),
        target: 7,
    };
    let json_button = InlineKeyboardButton::typed_callback("Confirm", &payload)?;
    let compact_button = InlineKeyboardButton::compact_callback("Confirm", &payload)?;
    assert!(
        compact_button.callback_data().unwrap_or_default().len()
            < json_button.callback_data().unwrap_or_default().len()
    );
    assert_eq!(
        compact_button.decode_compact_callback::<DemoCallbackPayload>()?,
        Some(payload.clone())
    );

    let Some(update) = parse_update(callback_update(
        206,
        1,
        compact_button.callback_data().unwrap_or_default(),
    )) else {
        return Ok(());
    };
    assert_eq!(
        extract_compact_callback::<DemoCallbackPayload>(&update),
        Some(payload.clone())
    );
    assert_eq!(
        update.compact_callback::<DemoCallbackPayload>(),
        Some(payload.clone())
    );

    let client = Client::builder("http://127.0.0.1:9")?
        .bot_token("123:abc")?
        .build()?;
    let hits = Arc::new(AtomicUsize::new(0));
    let mut router = Router::new();
    {
        let hits = Arc::clone(&hits);
        let expected = payload.clone();
        router
            .compact_callback_route::<DemoCallbackPayload>()
            .handle(move |_context: BotContext, _update: Update, callback| {
                let hits = Arc::clone(&hits);
                let expected = expected.clone();
                async move {
                    assert_eq!(callback.payload, expected);
                    hits.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                }
            });
    }

    assert!(router.dispatch(BotContext::new(client), update).await?);
    assert_eq!(hits.load(Ordering::SeqCst), 1);
    Ok(())
}

#[tokio::test]
async fn command_route_dsl_applies_guards_parse_and_throttle() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":{"message_id":501,"date":1710000000,"chat":{"id":-10042,"type":"supergroup"},"text":"rate limited"}}"#;
    let (base_url, handle) = spawn_server("/bot123:abc/sendMessage", 200, response)?;
    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;

    let Some(update) = parse_update(group_message_update(206, -10042, 123, "/ban @spam")) else {
        return Ok(());
    };

    let hits = Arc::new(AtomicUsize::new(0));
    let mut router = Router::new();
    {
        let hits = Arc::clone(&hits);
        router
            .command_route("ban")?
            .group_only()
            .admin_only()
            .require_capabilities(&[ChatAdministratorCapability::DeleteMessages])
            .bot_can(&[ChatAdministratorCapability::RestrictMembers])
            .throttle_actor(Duration::from_secs(30))
            .parse::<Vec<String>>()
            .handle(move |context: BotContext, _update: Update, args| {
                let hits = Arc::clone(&hits);
                async move {
                    assert_eq!(args, vec!["@spam".to_owned()]);
                    assert!(
                        context
                            .request_state()
                            .slot(CURRENT_ACTOR_CHAT_MEMBER)
                            .contains()
                    );
                    assert!(
                        context
                            .request_state()
                            .slot(CURRENT_BOT_CHAT_MEMBER)
                            .contains()
                    );
                    hits.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                }
            });
    }

    let user_member = chat_member_with_capabilities(
        123,
        false,
        "administrator",
        &[
            ChatAdministratorCapability::ManageChat,
            ChatAdministratorCapability::DeleteMessages,
        ],
    )?;
    let bot_member = chat_member_with_capabilities(
        1,
        true,
        "administrator",
        &[
            ChatAdministratorCapability::ManageChat,
            ChatAdministratorCapability::RestrictMembers,
        ],
    )?;

    let make_context = || {
        let context = BotContext::new(client.clone());
        let _ = context
            .request_state()
            .slot(CURRENT_ACTOR_CHAT_MEMBER)
            .set(user_member.clone());
        let _ = context
            .request_state()
            .slot(CURRENT_BOT_CHAT_MEMBER)
            .set(bot_member.clone());
        context
    };

    assert!(router.dispatch(make_context(), update.clone()).await?);
    assert_eq!(hits.load(Ordering::SeqCst), 1);

    assert!(router.dispatch(make_context(), update).await?);
    assert_eq!(hits.load(Ordering::SeqCst), 1);

    join_server(handle).await?;
    Ok(())
}

#[tokio::test]
async fn command_routes_respect_bot_target_and_canonical_message() -> Result<(), DynError> {
    let unconfigured_client = Client::builder("http://127.0.0.1:9")?
        .bot_token("123:abc")?
        .build()?;

    let mut disabled_auto_router = Router::new();
    let _ = disabled_auto_router.disable_auto_command_target();
    disabled_auto_router
        .command_route("start")?
        .handle(|_context: BotContext, _update: Update| async move { Ok(()) });
    let Some(targeted_this_bot_without_auto) =
        parse_update(message_update(205, 1, "/start@ThisBot hi"))
    else {
        return Ok(());
    };
    assert_eq!(parse_command_text("/start@OtherBot hi"), None);
    assert!(
        !disabled_auto_router
            .dispatch(
                BotContext::new(unconfigured_client.clone()),
                targeted_this_bot_without_auto,
            )
            .await?
    );

    let (base_url, handle) = spawn_server(
        "/bot123:abc/getMe",
        200,
        r#"{"ok":true,"result":{"id":1,"is_bot":true,"first_name":"tele","username":"ThisBot"}}"#,
    )?;
    let auto_client = Client::builder(&base_url)?.bot_token("123:abc")?.build()?;

    let mut auto_router = Router::new();
    auto_router
        .command_route("start")?
        .handle(|_context: BotContext, _update: Update| async move { Ok(()) });
    let Some(unprepared_targeted_this_bot) =
        parse_update(message_update(206, 1, "/start@ThisBot hi"))
    else {
        return Ok(());
    };
    assert!(
        !auto_router
            .dispatch(
                BotContext::new(unconfigured_client.clone()),
                unprepared_targeted_this_bot,
            )
            .await?
    );
    auto_router.prepare(&auto_client).await?;
    let Some(targeted_this_bot) = parse_update(message_update(207, 1, "/start@ThisBot hi")) else {
        return Ok(());
    };
    assert!(
        auto_router
            .dispatch(BotContext::new(auto_client.clone()), targeted_this_bot)
            .await?
    );
    join_server(handle).await?;

    let Some(targeted_other_bot) = parse_update(message_update(208, 1, "/start@OtherBot hi"))
    else {
        return Ok(());
    };
    assert!(
        !auto_router
            .dispatch(BotContext::new(auto_client), targeted_other_bot)
            .await?
    );

    let (harness_base_url, harness_handle) = spawn_server(
        "/bot123:abc/getMe",
        200,
        r#"{"ok":true,"result":{"id":1,"is_bot":true,"first_name":"tele","username":"ThisBot"}}"#,
    )?;
    let harness_client = Client::builder(&harness_base_url)?
        .bot_token("123:abc")?
        .build()?;
    let mut harness_router = Router::new();
    harness_router
        .command_route("start")?
        .handle(|_context: BotContext, _update: Update| async move { Ok(()) });
    let harness = BotHarness::with_client(harness_client, harness_router);
    let Some(harness_update) = parse_update(message_update(209, 1, "/start@ThisBot hi")) else {
        return Ok(());
    };
    assert!(matches!(
        harness.dispatch(harness_update).await?,
        DispatchOutcome::Handled { .. }
    ));
    join_server(harness_handle).await?;

    let mut targeted_router = Router::new();
    let _ = targeted_router.set_command_target("ThisBot")?;
    let _ = targeted_router.disable_auto_command_target();
    targeted_router
        .command_route("start")?
        .handle(|_context: BotContext, _update: Update| async move { Ok(()) });
    let Some(targeted_this_bot) = parse_update(message_update(210, 1, "/start@ThisBot hi")) else {
        return Ok(());
    };
    assert!(parse_command_text_for_bot("/start@ThisBot hi", Some("thisbot")).is_some());
    assert!(
        targeted_router
            .dispatch(BotContext::new(unconfigured_client), targeted_this_bot)
            .await?
    );

    let command_hits = Arc::new(AtomicUsize::new(0));
    let mut callback_command_router = Router::new();
    {
        let command_hits = Arc::clone(&command_hits);
        callback_command_router.command_route("start")?.handle(
            move |_context: BotContext, _update: Update| {
                let command_hits = Arc::clone(&command_hits);
                async move {
                    command_hits.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                }
            },
        );
    }
    let Some(callback_command_update) = parse_update(json!({
        "update_id": 2101,
        "callback_query": {
            "id": "cb-start-message",
            "from": {"id": 99, "is_bot": false, "first_name": "tester"},
            "message": {
                "message_id": 2101,
                "date": 1700002101,
                "chat": {"id": 1, "type": "private"},
                "text": "/start"
            },
            "data": "clicked"
        }
    })) else {
        return Ok(());
    };
    assert!(
        !callback_command_router
            .dispatch(
                BotContext::new(
                    Client::builder("http://127.0.0.1:9")?
                        .bot_token("123:abc")?
                        .build()?
                ),
                callback_command_update,
            )
            .await?
    );
    assert_eq!(command_hits.load(Ordering::SeqCst), 0);

    let Some(callback_mentioned_command_update) = parse_update(json!({
        "update_id": 2102,
        "callback_query": {
            "id": "cb-mentioned-start-message",
            "from": {"id": 99, "is_bot": false, "first_name": "tester"},
            "message": {
                "message_id": 2102,
                "date": 1700002102,
                "chat": {"id": 1, "type": "private"},
                "text": "/start@ThisBot"
            },
            "data": "clicked"
        }
    })) else {
        return Ok(());
    };
    assert!(
        !callback_command_router
            .dispatch_prepared(
                BotContext::new(
                    Client::builder("http://127.0.0.1:9")?
                        .bot_token("123:abc")?
                        .build()?
                ),
                callback_mentioned_command_update,
            )
            .await?
    );
    assert_eq!(command_hits.load(Ordering::SeqCst), 0);

    let Some(edited_update) = parse_update(json!({
        "update_id": 211,
        "edited_message": {
            "message_id": 211,
            "date": 1700000211,
            "chat": {"id": 1, "type": "private"},
            "text": "/echo changed"
        }
    })) else {
        return Ok(());
    };
    assert_eq!(extract_command(&edited_update), Some("echo"));
    assert_eq!(extract_command_args(&edited_update), Some("changed"));
    assert_eq!(
        extract_command_data(&edited_update),
        Some(CommandData {
            name: "echo".to_owned(),
            mention: None,
            args: "changed".to_owned(),
        })
    );

    Ok(())
}

#[tokio::test]
async fn router_dispatch_prepared_handles_command_mentions() -> Result<(), DynError> {
    let (base_url, handle) = spawn_server(
        "/bot123:abc/getMe",
        200,
        r#"{"ok":true,"result":{"id":1,"is_bot":true,"first_name":"tele","username":"ThisBot"}}"#,
    )?;
    let client = Client::builder(&base_url)?.bot_token("123:abc")?.build()?;

    let mut router = Router::new();
    router
        .command_route("start")?
        .handle(|_context: BotContext, _update: Update| async move { Ok(()) });
    let Some(update) = parse_update(message_update(212, 1, "/start@ThisBot hi")) else {
        return Ok(());
    };

    assert!(
        router
            .dispatch_prepared(BotContext::new(client), update)
            .await?
    );
    join_server(handle).await?;
    Ok(())
}

#[tokio::test]
async fn bootstrap_router_reuses_get_me_for_command_target_prepare() -> Result<(), DynError> {
    let (base_url, handle) = spawn_server(
        "/bot123:abc/getMe",
        200,
        r#"{"ok":true,"result":{"id":1,"is_bot":true,"first_name":"tele","username":"ThisBot"}}"#,
    )?;
    let client = Client::builder(&base_url)?.bot_token("123:abc")?.build()?;
    let control = client.control();

    let hits = Arc::new(AtomicUsize::new(0));
    let mut router = Router::new();
    {
        let hits = Arc::clone(&hits);
        router
            .command_route("start")?
            .handle(move |_context: BotContext, _update: Update| {
                let hits = Arc::clone(&hits);
                async move {
                    hits.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                }
            });
    }

    let outcome = control
        .bootstrap_router(&router, &tele::BootstrapPlan::new().fail_fast_get_me())
        .await;
    let report = outcome.into_result()?;
    assert_eq!(
        report
            .me
            .value
            .as_ref()
            .and_then(|me| me.username.as_deref()),
        Some("ThisBot")
    );

    join_server(handle).await?;

    let Some(update) = parse_update(message_update(213, 1, "/start@ThisBot hi")) else {
        return Ok(());
    };
    assert!(router.dispatch(BotContext::new(client), update).await?);
    assert_eq!(hits.load(Ordering::SeqCst), 1);
    Ok(())
}

#[tokio::test]
async fn bootstrap_router_skip_get_me_preserves_lazy_command_target_prepare() -> Result<(), DynError>
{
    let (base_url, handle) = spawn_server(
        "/bot123:abc/getMe",
        200,
        r#"{"ok":true,"result":{"id":1,"is_bot":true,"first_name":"tele","username":"ThisBot"}}"#,
    )?;
    let client = Client::builder(&base_url)?.bot_token("123:abc")?.build()?;
    let control = client.control();

    let hits = Arc::new(AtomicUsize::new(0));
    let mut router = Router::new();
    {
        let hits = Arc::clone(&hits);
        router
            .command_route("start")?
            .handle(move |_context: BotContext, _update: Update| {
                let hits = Arc::clone(&hits);
                async move {
                    hits.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                }
            });
    }

    let outcome = control
        .bootstrap_router(&router, &tele::BootstrapPlan::new().skip_get_me())
        .await;
    assert!(outcome.is_success());
    assert_eq!(
        outcome.report.me.diagnostics.status,
        BootstrapStepStatus::Skipped
    );

    let Some(update) = parse_update(message_update(214, 1, "/start@ThisBot hi")) else {
        return Ok(());
    };
    assert!(
        router
            .dispatch_prepared(BotContext::new(client), update)
            .await?
    );
    assert_eq!(hits.load(Ordering::SeqCst), 1);

    join_server(handle).await?;
    Ok(())
}

#[tokio::test]
async fn client_control_exposes_setup_facade() -> Result<(), DynError> {
    let (base_url, handle) = spawn_server(
        "/bot123:abc/getMe",
        200,
        r#"{"ok":true,"result":{"id":1,"is_bot":true,"first_name":"tele","username":"ThisBot"}}"#,
    )?;
    let client = Client::builder(&base_url)?.bot_token("123:abc")?.build()?;
    let control = client.control();

    let outcome = control
        .setup()
        .bootstrap(&tele::BootstrapPlan::new().fail_fast_get_me())
        .await;
    let report = outcome.into_result()?;
    assert_eq!(
        report
            .me
            .value
            .as_ref()
            .and_then(|me| me.username.as_deref()),
        Some("ThisBot")
    );

    join_server(handle).await?;
    Ok(())
}

#[tokio::test]
async fn bootstrap_router_respects_warn_get_me_policy_without_refetch() -> Result<(), DynError> {
    let client = Client::builder("http://127.0.0.1:9")?
        .bot_token("123:abc")?
        .request_timeout(Duration::from_millis(40))
        .total_timeout(Some(Duration::from_millis(120)))
        .build()?;
    let control = client.control();
    let router = Router::new();

    let outcome = control
        .bootstrap_router(
            &router,
            &tele::BootstrapPlan::new().warn_and_continue_on_retryable_get_me(),
        )
        .await;

    assert!(outcome.is_success());
    assert_eq!(
        outcome.report.me.diagnostics.status,
        BootstrapStepStatus::Warned
    );
    assert!(outcome.report.me.value.is_none());

    Ok(())
}

#[tokio::test]
async fn bootstrap_router_warn_policy_disables_later_auto_get_me() -> Result<(), DynError> {
    let client = Client::builder("http://127.0.0.1:9")?
        .bot_token("123:abc")?
        .request_timeout(Duration::from_millis(40))
        .total_timeout(Some(Duration::from_millis(120)))
        .build()?;
    let control = client.control();
    let mut router = Router::new();
    router
        .command_route("start")?
        .handle(|_context: BotContext, _update: Update| async move { Ok(()) });

    let outcome = control
        .bootstrap_router(
            &router,
            &tele::BootstrapPlan::new().warn_and_continue_on_retryable_get_me(),
        )
        .await;
    assert!(outcome.is_success());

    let Some(update) = parse_update(message_update(214, 1, "/start@ThisBot hi")) else {
        return Ok(());
    };
    assert!(!router.dispatch(BotContext::new(client), update).await?);

    Ok(())
}

#[tokio::test]
async fn long_polling_config_checked_fails_early() -> Result<(), DynError> {
    let client = Client::builder("http://127.0.0.1:9")?
        .bot_token("123:abc")?
        .request_timeout(Duration::from_millis(500))
        .total_timeout(Some(Duration::from_millis(500)))
        .build()?;

    let source = LongPollingSource::new(client).with_config_checked(PollingConfig {
        poll_timeout_seconds: 30,
        ..PollingConfig::default()
    });
    assert!(source.is_err());

    Ok(())
}

#[tokio::test]
async fn long_polling_config_checked_rejects_invalid_request_options() -> Result<(), DynError> {
    let client = Client::builder("http://127.0.0.1:9")?
        .bot_token("123:abc")?
        .build()?;

    let invalid_limit = LongPollingSource::new(client.clone()).with_config_checked(PollingConfig {
        limit: Some(0),
        ..PollingConfig::default()
    });
    assert!(matches!(invalid_limit, Err(Error::Configuration { .. })));

    let duplicate_allowed_update =
        LongPollingSource::new(client).with_config_checked(PollingConfig {
            allowed_updates: Some(vec![
                AllowedUpdate::new("message")?,
                AllowedUpdate::new("message")?,
            ]),
            ..PollingConfig::default()
        });
    assert!(matches!(
        duplicate_allowed_update,
        Err(Error::Configuration { .. })
    ));

    Ok(())
}

#[tokio::test]
async fn engine_config_rejects_invalid_runtime_values() -> Result<(), DynError> {
    let zero_concurrency = EngineConfig {
        max_handler_concurrency: 0,
        ..EngineConfig::default()
    };
    assert!(matches!(
        zero_concurrency.validate(),
        Err(Error::Configuration { .. })
    ));

    let zero_error_delay = EngineConfig {
        continue_on_source_error: true,
        error_delay: Duration::ZERO,
        source_error_backoff: None,
        ..EngineConfig::default()
    };
    assert!(matches!(
        zero_error_delay.validate(),
        Err(Error::Configuration { .. })
    ));

    let invalid_backoff = EngineConfig {
        source_error_backoff: Some(SourceErrorBackoffConfig {
            base_delay: Duration::from_secs(2),
            max_delay: Duration::from_secs(1),
            jitter_ratio: f32::NAN,
        }),
        ..EngineConfig::default()
    };
    assert!(matches!(
        invalid_backoff.validate(),
        Err(Error::Configuration { .. })
    ));

    Ok(())
}

#[tokio::test]
async fn bot_engine_rejects_invalid_config_before_polling_source() -> Result<(), DynError> {
    let client = Client::builder("http://127.0.0.1:9")?
        .bot_token("123:abc")?
        .build()?;
    let (_sink, source) = channel_source(1)?;
    let mut engine = BotEngine::new(client, source, Router::new()).with_config(EngineConfig {
        max_handler_concurrency: 0,
        ..EngineConfig::default()
    });

    let error = match engine.poll_once().await {
        Ok(outcomes) => {
            return Err(
                format!("invalid config unexpectedly produced outcomes: {outcomes:?}").into(),
            );
        }
        Err(error) => error,
    };
    assert!(matches!(error, Error::Configuration { .. }));

    Ok(())
}

#[test]
fn channel_source_rejects_zero_capacity_values() -> Result<(), DynError> {
    assert!(matches!(
        channel_source(0),
        Err(Error::Configuration { .. })
    ));

    let (_sink, source) = channel_source(1)?;
    assert!(matches!(
        source.with_max_batch(0),
        Err(Error::Configuration { .. })
    ));

    Ok(())
}

#[tokio::test]
async fn router_routes_by_message_and_update_kind() -> Result<(), DynError> {
    let client = Client::builder("http://127.0.0.1:9")?
        .bot_token("123:abc")?
        .build()?;

    let photo_hits = Arc::new(AtomicUsize::new(0));
    let callback_hits = Arc::new(AtomicUsize::new(0));
    let mut router = Router::new();
    {
        let photo_hits = Arc::clone(&photo_hits);
        router.message_kind_route(MessageKind::Photo).handle(
            move |_context: BotContext, _update: Update| {
                let photo_hits = Arc::clone(&photo_hits);
                async move {
                    photo_hits.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                }
            },
        );
    }
    {
        let callback_hits = Arc::clone(&callback_hits);
        router.update_kind_route(UpdateKind::CallbackQuery).handle(
            move |_context: BotContext, _update: Update| {
                let callback_hits = Arc::clone(&callback_hits);
                async move {
                    callback_hits.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                }
            },
        );
    }

    let maybe_photo = parse_update(serde_json::json!({
        "update_id": 320,
        "message": {
            "message_id": 320,
            "date": 1700000320,
            "chat": {"id": 1, "type": "private"},
            "photo": [{
                "file_id": "p-1",
                "file_unique_id": "u-1",
                "width": 16,
                "height": 16
            }],
            "caption": "preview"
        }
    }));
    assert!(maybe_photo.is_some());
    let Some(photo_update) = maybe_photo else {
        return Ok(());
    };

    let maybe_callback = parse_update(callback_update(321, 1, "btn-1"));
    assert!(maybe_callback.is_some());
    let Some(callback_update) = maybe_callback else {
        return Ok(());
    };

    assert!(
        router
            .dispatch(BotContext::new(client.clone()), photo_update)
            .await?
    );
    assert!(
        router
            .dispatch(BotContext::new(client), callback_update)
            .await?
    );
    assert_eq!(photo_hits.load(Ordering::SeqCst), 1);
    assert_eq!(callback_hits.load(Ordering::SeqCst), 1);

    Ok(())
}

#[tokio::test]
async fn router_distinguishes_incoming_and_any_message_kind() -> Result<(), DynError> {
    let client = Client::builder("http://127.0.0.1:9")?
        .bot_token("123:abc")?
        .build()?;

    let incoming_text_hits = Arc::new(AtomicUsize::new(0));
    let any_text_hits = Arc::new(AtomicUsize::new(0));
    let mut router = Router::new();

    {
        let incoming_text_hits = Arc::clone(&incoming_text_hits);
        router.message_kind_route(MessageKind::Text).handle(
            move |_context: BotContext, _update: Update| {
                let incoming_text_hits = Arc::clone(&incoming_text_hits);
                async move {
                    incoming_text_hits.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                }
            },
        );
    }
    {
        let any_text_hits = Arc::clone(&any_text_hits);
        router.message_like_kind_route(MessageKind::Text).handle(
            move |_context: BotContext, _update: Update| {
                let any_text_hits = Arc::clone(&any_text_hits);
                async move {
                    any_text_hits.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                }
            },
        );
    }

    let maybe_callback = parse_update(callback_update(322, 1, "btn-1"));
    assert!(maybe_callback.is_some());
    let Some(callback_update) = maybe_callback else {
        return Ok(());
    };

    assert!(
        router
            .dispatch(BotContext::new(client), callback_update)
            .await?
    );
    assert_eq!(incoming_text_hits.load(Ordering::SeqCst), 0);
    assert_eq!(any_text_hits.load(Ordering::SeqCst), 1);

    Ok(())
}

#[tokio::test]
async fn web_app_typed_builders_serialize() -> Result<(), DynError> {
    let article_result =
        InlineQueryResult::article("article-1", "Article Title", "Article Message Text")?;
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

    let menu_button_config =
        MenuButtonConfig::for_chat_web_app(10002, "Launch", "https://example.com/menu-config");
    let menu_button_request: AdvancedSetChatMenuButtonRequest = (&menu_button_config).into();
    let menu_button_request_json = serde_json::to_value(&menu_button_request)?;
    assert_eq!(menu_button_request_json["chat_id"], 10002);
    assert_eq!(menu_button_request_json["menu_button"]["type"], "web_app");
    assert_eq!(menu_button_request_json["menu_button"]["text"], "Launch");
    assert_eq!(
        menu_button_request_json["menu_button"]["web_app"]["url"],
        "https://example.com/menu-config"
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
        router
            .command_route("start")?
            .handle(move |_context: BotContext, _update: Update| {
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

    join_server(handle).await?;
    Ok(())
}

#[tokio::test]
async fn long_polling_source_uses_default_poll_timeout() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":[]}"#;
    const CHECKS: [&str; 1] = ["\"timeout\":30"];
    let (base_url, handle) =
        spawn_server_with_checks("/bot123:abc/getUpdates", 200, response, &CHECKS)?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let source = LongPollingSource::new(client.clone()).with_config(PollingConfig {
        disable_webhook_on_start: false,
        ..PollingConfig::default()
    });
    let mut engine = BotEngine::new(client, source, Router::new()).with_config(EngineConfig {
        continue_on_source_error: false,
        continue_on_handler_error: false,
        ..EngineConfig::default()
    });

    let outcomes = engine.poll_once().await?;
    assert!(outcomes.is_empty());

    join_server(handle).await?;
    Ok(())
}

#[tokio::test]
async fn long_polling_source_rejects_config_when_timeout_budget_is_too_small()
-> Result<(), DynError> {
    let client = Client::builder("http://127.0.0.1:9")?
        .bot_token("123:abc")?
        .request_timeout(Duration::from_millis(900))
        .total_timeout(Some(Duration::from_secs(3)))
        .build()?;
    let source = LongPollingSource::new(client.clone()).with_config(PollingConfig {
        disable_webhook_on_start: false,
        ..PollingConfig::default()
    });
    let mut engine = BotEngine::new(client, source, Router::new()).with_config(EngineConfig {
        continue_on_source_error: false,
        continue_on_handler_error: false,
        ..EngineConfig::default()
    });

    let error = match engine.poll_once().await {
        Ok(_) => return Err("expected polling timeout configuration error".into()),
        Err(error) => error,
    };
    assert!(matches!(error, Error::Configuration { .. }));
    assert!(error.to_string().contains("poll_timeout_seconds"));
    assert!(error.to_string().contains("set poll_timeout_seconds=0"));

    Ok(())
}

#[tokio::test]
async fn long_polling_source_allows_short_polling_with_zero_timeout() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":[]}"#;
    const CHECKS: [&str; 1] = ["\"timeout\":0"];
    let (base_url, handle) =
        spawn_server_with_checks("/bot123:abc/getUpdates", 200, response, &CHECKS)?;

    let client = Client::builder(base_url)?
        .bot_token("123:abc")?
        .request_timeout(Duration::from_millis(900))
        .total_timeout(Some(Duration::from_secs(3)))
        .build()?;
    let source = LongPollingSource::new(client.clone()).with_config(PollingConfig {
        disable_webhook_on_start: false,
        poll_timeout_seconds: 0,
        ..PollingConfig::default()
    });
    let mut engine = BotEngine::new(client, source, Router::new()).with_config(EngineConfig {
        continue_on_source_error: false,
        continue_on_handler_error: false,
        ..EngineConfig::default()
    });

    let outcomes = engine.poll_once().await?;
    assert!(outcomes.is_empty());

    join_server(handle).await?;
    Ok(())
}

#[test]
fn long_polling_source_rejects_poll_timeout_above_total_timeout_budget() -> Result<(), DynError> {
    let client = Client::builder("http://127.0.0.1:9")?
        .bot_token("123:abc")?
        .request_timeout(Duration::from_secs(40))
        .total_timeout(Some(Duration::from_secs(5)))
        .build()?;
    let source = LongPollingSource::new(client).with_config(PollingConfig {
        poll_timeout_seconds: 30,
        ..PollingConfig::default()
    });

    let error = match source.validate_timeout_budget() {
        Ok(timeout) => return Err(format!("expected timeout budget error, got {timeout}s").into()),
        Err(error) => error,
    };
    assert!(matches!(error, Error::Configuration { .. }));
    assert!(error.to_string().contains("poll_timeout_seconds=30"));
    assert!(error.to_string().contains("headroom of 4s"));
    assert!(error.to_string().contains("set poll_timeout_seconds=0"));
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

    join_server(handle).await?;
    let _ = fs::remove_file(offset_path);
    Ok(())
}

#[tokio::test]
async fn long_polling_source_rejects_invalid_offset_storage_before_network() -> Result<(), DynError>
{
    let (root, offset_path) = blocked_child_path("tele-offset-invalid-target", "offset.json")?;
    let client = Client::builder("http://127.0.0.1:9")?
        .bot_token("123:abc")?
        .build()?;
    let mut source = LongPollingSource::new(client).with_config(PollingConfig {
        persist_offset_path: Some(offset_path),
        ..PollingConfig::default()
    });

    let result = source.poll().await;

    assert!(matches!(result, Err(Error::Storage { .. })));
    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[tokio::test]
async fn long_polling_source_validates_offset_storage_after_explicit_offset_override()
-> Result<(), DynError> {
    let (root, offset_path) =
        blocked_child_path("tele-offset-override-invalid-target", "offset.json")?;
    let client = Client::builder("http://127.0.0.1:9")?
        .bot_token("123:abc")?
        .build()?;
    let mut source = LongPollingSource::new(client).with_config(PollingConfig {
        persist_offset_path: Some(offset_path),
        ..PollingConfig::default()
    });
    source.set_next_offset(Some(42));

    let result = source.poll().await;

    assert!(matches!(result, Err(Error::Storage { .. })));
    let _ = fs::remove_dir_all(root);
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
        router
            .command_route("start")?
            .handle(move |_context: BotContext, _update: Update| {
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

    join_server(handle).await?;
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

    join_server(handle).await?;
    let _ = fs::remove_file(offset_path);
    Ok(())
}

#[tokio::test]
async fn long_polling_source_does_not_ack_failed_dispatch() -> Result<(), DynError> {
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let offset_path = std::env::temp_dir().join(format!("tele-offset-failed-{timestamp}.json"));

    let response = r#"{"ok":true,"result":[{"update_id":801,"message":{"message_id":1,"date":1710000112,"chat":{"id":1,"type":"private"},"text":"/fail"}}]}"#;
    let (base_url, handle) = spawn_server("/bot123:abc/getUpdates", 200, response)?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let mut router = Router::new();
    router
        .command_route("fail")?
        .handle(|_context: BotContext, _update: Update| async move {
            Err(HandlerError::internal(Error::InvalidRequest {
                reason: "handler failed".to_owned(),
            }))
        });

    let source = LongPollingSource::new(client.clone()).with_config(PollingConfig {
        disable_webhook_on_start: false,
        persist_offset_path: Some(offset_path.clone()),
        poll_timeout_seconds: 1,
        ..PollingConfig::default()
    });
    let mut engine = BotEngine::new(client, source, router).with_config(EngineConfig {
        continue_on_source_error: false,
        continue_on_handler_error: false,
        ..EngineConfig::default()
    });

    let error = match engine.poll_once().await {
        Ok(_) => return Err("expected handler failure".into()),
        Err(error) => error,
    };
    assert!(matches!(error, Error::InvalidRequest { .. }));
    assert_eq!(engine.source_mut().next_offset(), None);
    assert!(!offset_path.exists());

    join_server(handle).await?;
    Ok(())
}

#[tokio::test]
async fn long_polling_source_acks_continued_handler_error() -> Result<(), DynError> {
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let offset_path =
        std::env::temp_dir().join(format!("tele-offset-continued-failed-{timestamp}.json"));

    let response = r#"{"ok":true,"result":[{"update_id":811,"message":{"message_id":1,"date":1710000113,"chat":{"id":1,"type":"private"},"text":"/fail"}}]}"#;
    let (base_url, handle) = spawn_server("/bot123:abc/getUpdates", 200, response)?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let mut router = Router::new();
    router
        .command_route("fail")?
        .handle(|_context: BotContext, _update: Update| async move {
            Err(HandlerError::internal(Error::InvalidRequest {
                reason: "handler failed".to_owned(),
            }))
        });

    let source = LongPollingSource::new(client.clone()).with_config(PollingConfig {
        disable_webhook_on_start: false,
        persist_offset_path: Some(offset_path.clone()),
        poll_timeout_seconds: 1,
        ..PollingConfig::default()
    });
    let mut engine = BotEngine::new(client, source, router).with_config(EngineConfig {
        continue_on_source_error: false,
        continue_on_handler_error: true,
        ..EngineConfig::default()
    });

    let outcomes = engine.poll_once().await?;
    assert_eq!(outcomes, vec![DispatchOutcome::Failed { update_id: 811 }]);
    assert_eq!(engine.source_mut().next_offset(), Some(812));

    let raw = fs::read(&offset_path)?;
    let snapshot: serde_json::Value = serde_json::from_slice(&raw)?;
    assert_eq!(
        snapshot
            .get("next_offset")
            .and_then(serde_json::Value::as_i64),
        Some(812)
    );

    join_server(handle).await?;
    let _ = fs::remove_file(offset_path);
    Ok(())
}

#[tokio::test]
async fn long_polling_commits_successful_prefix_before_fail_fast_handler_error()
-> Result<(), DynError> {
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let offset_path =
        std::env::temp_dir().join(format!("tele-offset-fail-fast-prefix-{timestamp}.json"));

    let response = r#"{"ok":true,"result":[{"update_id":812,"message":{"message_id":1,"date":1710000114,"chat":{"id":1,"type":"private"},"text":"ok"}},{"update_id":813,"message":{"message_id":2,"date":1710000115,"chat":{"id":1,"type":"private"},"text":"fail"}}]}"#;
    let (base_url, handle) = spawn_server("/bot123:abc/getUpdates", 200, response)?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let mut router = Router::new();
    router
        .message_route()
        .handle(|_context: BotContext, update: Update| async move {
            if update.text() == Some("fail") {
                return Err(HandlerError::internal(Error::InvalidRequest {
                    reason: "handler failed".to_owned(),
                }));
            }
            Ok(())
        });

    let source = LongPollingSource::new(client.clone()).with_config(PollingConfig {
        disable_webhook_on_start: false,
        persist_offset_path: Some(offset_path.clone()),
        poll_timeout_seconds: 1,
        ..PollingConfig::default()
    });
    let mut engine = BotEngine::new(client, source, router).with_config(EngineConfig {
        continue_on_source_error: false,
        continue_on_handler_error: false,
        ..EngineConfig::default()
    });

    let error = match engine.poll_once().await {
        Ok(outcomes) => return Err(format!("expected handler failure, got {outcomes:?}").into()),
        Err(error) => error,
    };
    assert!(matches!(error, Error::InvalidRequest { .. }));
    assert_eq!(engine.source_mut().next_offset(), Some(813));

    let raw = fs::read(&offset_path)?;
    let snapshot: serde_json::Value = serde_json::from_slice(&raw)?;
    assert_eq!(
        snapshot
            .get("next_offset")
            .and_then(serde_json::Value::as_i64),
        Some(813)
    );

    join_server(handle).await?;
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
        router
            .command_route("start")?
            .handle(move |_context: BotContext, _update: Update| {
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

    join_server(handle).await?;
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
        router
            .command_route("start")?
            .handle(move |_context: BotContext, _update: Update| {
                let hits = Arc::clone(&hits);
                async move {
                    hits.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                }
            });
    }

    let (sink, source) = channel_source(8)?;
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
    let metrics = Arc::new(Mutex::new(Vec::<EngineMetric>::new()));

    let source = LongPollingSource::new(client.clone()).with_config(PollingConfig {
        disable_webhook_on_start: false,
        poll_timeout_seconds: 0,
        ..PollingConfig::default()
    });
    let mut engine = BotEngine::new(client, source, Router::new())
        .with_config(EngineConfig {
            continue_on_source_error: true,
            error_delay: Duration::from_millis(10),
            source_error_backoff: Some(Default::default()),
            ..EngineConfig::default()
        })
        .on_metric({
            let metrics = Arc::clone(&metrics);
            move |metric| {
                if let Ok(mut captured) = metrics.lock() {
                    captured.push(metric.clone());
                }
            }
        });

    let shutdown = async {
        tokio::time::sleep(Duration::from_millis(120)).await;
    };

    let result = engine.run_until(shutdown).await;
    assert!(result.is_ok());
    let metrics = metrics.lock().map_err(|_| "engine metric mutex poisoned")?;
    assert!(
        metrics
            .iter()
            .any(|metric| matches!(metric, EngineMetric::SourceError { .. }))
    );
    assert!(
        metrics
            .iter()
            .any(|metric| matches!(metric, EngineMetric::SourceBackoff { .. }))
    );
    Ok(())
}

#[tokio::test]
async fn run_until_stops_on_non_retryable_source_error() -> Result<(), DynError> {
    let client = Client::builder("http://127.0.0.1:9")?
        .bot_token("123:abc")?
        .build()?;
    let (sink, source) = channel_source(1)?;
    drop(sink);
    let source_errors = Arc::new(AtomicUsize::new(0));
    let metrics = Arc::new(Mutex::new(Vec::<EngineMetric>::new()));

    let mut engine = BotEngine::new(client, source, Router::new())
        .with_config(EngineConfig {
            continue_on_source_error: true,
            error_delay: Duration::from_secs(60),
            ..EngineConfig::default()
        })
        .on_source_error({
            let source_errors = Arc::clone(&source_errors);
            move |_error| {
                source_errors.fetch_add(1, Ordering::SeqCst);
            }
        })
        .on_metric({
            let metrics = Arc::clone(&metrics);
            move |metric| {
                if let Ok(mut captured) = metrics.lock() {
                    captured.push(metric.clone());
                }
            }
        });

    let result = tokio::time::timeout(
        Duration::from_millis(100),
        engine.run_until(std::future::pending::<()>()),
    )
    .await;
    let error = match result {
        Ok(Err(error)) => error,
        Ok(Ok(())) => return Err("closed source unexpectedly completed cleanly".into()),
        Err(_) => return Err("closed source was incorrectly retried until timeout".into()),
    };

    assert!(matches!(error, Error::InvalidRequest { .. }));
    assert!(!error.is_retryable());
    assert_eq!(source_errors.load(Ordering::SeqCst), 1);
    let metrics = metrics.lock().map_err(|_| "engine metric mutex poisoned")?;
    assert!(
        metrics.iter().any(|metric| matches!(
            metric,
            EngineMetric::SourceError {
                retryable: false,
                ..
            }
        )),
        "non-retryable source errors should still emit source-error metrics"
    );
    Ok(())
}

#[tokio::test]
async fn run_until_stops_on_fail_fast_handler_error_even_when_source_errors_continue()
-> Result<(), DynError> {
    let client = Client::builder("http://127.0.0.1:9")?
        .bot_token("123:abc")?
        .build()?;
    let mut router = Router::new();
    router
        .message_route()
        .handle(|_context: BotContext, _update: Update| async move {
            Err(HandlerError::internal(Error::InvalidRequest {
                reason: "handler failed".to_owned(),
            }))
        });

    let (sink, source) = channel_source(1)?;
    let mut engine = BotEngine::new(client, source, router).with_config(EngineConfig {
        continue_on_source_error: true,
        continue_on_handler_error: false,
        error_delay: Duration::from_millis(10),
        ..EngineConfig::default()
    });

    sink.send(tele::bot::testing::message_update(820, 1, "fail")?)
        .await?;
    drop(sink);

    let result = engine
        .run_until(async {
            tokio::time::sleep(Duration::from_millis(150)).await;
        })
        .await;
    let error = match result {
        Ok(()) => return Err("handler failure was incorrectly treated as source retry".into()),
        Err(error) => error,
    };
    assert!(matches!(error, Error::InvalidRequest { .. }));
    assert!(error.to_string().contains("handler failed"));

    Ok(())
}

#[tokio::test]
async fn bot_engine_run_until_can_be_spawned_on_tokio() -> Result<(), DynError> {
    let client = Client::builder("http://127.0.0.1:9")?
        .bot_token("123:abc")?
        .build()?;

    let handle = tokio::spawn(async move {
        let (_sink, source) = channel_source(1)?;
        let mut engine = BotEngine::new(client, source, Router::new());
        engine.run_until(async {}).await
    });

    let result = handle.await?;
    assert!(result.is_ok());
    Ok(())
}

#[tokio::test]
async fn bot_app_run_until_can_be_spawned_on_tokio() -> Result<(), DynError> {
    let client = Client::builder("http://127.0.0.1:9")?
        .bot_token("123:abc")?
        .build()?;

    let handle = tokio::spawn(async move {
        let (_sink, source) = channel_source(1)?;
        let engine = BotEngine::new(client, source, Router::new());
        let mut app = BotApp::from_engine(engine);
        app.run_until(async {}).await
    });

    let result = handle.await?;
    assert!(result.is_ok());
    Ok(())
}

#[tokio::test]
async fn bot_engine_metric_hook_emits_poll_and_dispatch_latency() -> Result<(), DynError> {
    let client = Client::builder("http://127.0.0.1:9")?
        .bot_token("123:abc")?
        .build()?;
    let metrics = Arc::new(Mutex::new(Vec::<EngineMetric>::new()));

    let mut router = Router::new();
    router
        .command_route("start")?
        .handle(|_context: BotContext, _update: Update| async move { Ok(()) });

    let (sink, source) = channel_source(1)?;
    let Some(update) = parse_update(message_update(7001, 10, "/start")) else {
        return Err("failed to build /start update fixture".into());
    };
    sink.send(update).await?;

    let mut engine = BotEngine::new(client, source, router).on_metric({
        let metrics = Arc::clone(&metrics);
        move |metric| {
            if let Ok(mut captured) = metrics.lock() {
                captured.push(metric.clone());
            }
        }
    });

    let outcomes = engine.poll_once().await?;
    assert_eq!(outcomes, vec![DispatchOutcome::Handled { update_id: 7001 }]);

    let metrics = metrics.lock().map_err(|_| "engine metric mutex poisoned")?;
    assert!(metrics.iter().any(|metric| matches!(
        metric,
        EngineMetric::PollLatency {
            update_count: 1,
            ..
        }
    )));
    assert!(metrics.iter().any(|metric| matches!(
        metric,
        EngineMetric::DispatchLatency {
            update_id: 7001,
            outcome: DispatchMetricOutcome::Handled,
            ..
        }
    )));

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
        router
            .command_route("start")?
            .handle(move |_context: BotContext, _update: Update| {
                let handler_hits = Arc::clone(&handler_hits);
                async move {
                    handler_hits.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                }
            });
    }

    let runner = WebhookRunner::new(client, router).expected_secret_token("secret-token")?;
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
async fn webhook_runner_reports_continued_handler_error_as_failed() -> Result<(), DynError> {
    let client = Client::builder("http://127.0.0.1:9")?
        .bot_token("123:abc")?
        .build()?;

    let mut router = Router::new();
    router
        .message_route()
        .handle(|_context: BotContext, _update: Update| async move {
            Err(HandlerError::internal(Error::InvalidRequest {
                reason: "handler failed".to_owned(),
            }))
        });

    let runner = WebhookRunner::new(client, router).continue_on_handler_error(true);
    let payload = serde_json::to_vec(&message_update(903, 10, "boom"))?;

    let outcome = runner.dispatch_json_outcome(&payload, None).await?;
    assert_eq!(outcome, DispatchOutcome::Failed { update_id: 903 });

    Ok(())
}

#[tokio::test]
async fn fallible_route_maps_user_error_to_reply() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":{"message_id":99,"date":1710000009,"chat":{"id":10,"type":"private"},"text":"invalid input"}}"#;
    let (base_url, handle) = spawn_server_with_checks(
        "/bot123:abc/sendMessage",
        200,
        response,
        &["\"text\":\"invalid input\""],
    )?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;

    let mut router = Router::new();
    router
        .message_route()
        .handle(|_context: BotContext, _update: Update| async move {
            Err(HandlerError::user("invalid input"))
        });

    let maybe_update = parse_update(message_update(902, 10, "bad request"));
    assert!(maybe_update.is_some());
    let Some(update) = maybe_update else {
        return Ok(());
    };

    let handled = router.dispatch(BotContext::new(client), update).await?;
    assert!(handled);

    join_server(handle).await?;
    Ok(())
}

#[tokio::test]
async fn fallible_route_normalizes_empty_user_error_to_reply() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":{"message_id":100,"date":1710000010,"chat":{"id":10,"type":"private"},"text":"request rejected"}}"#;
    let (base_url, handle) = spawn_server_with_checks(
        "/bot123:abc/sendMessage",
        200,
        response,
        &["\"text\":\"request rejected\""],
    )?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;

    let mut router = Router::new();
    router
        .message_route()
        .handle(|_context: BotContext, _update: Update| async move {
            Err(HandlerError::user(" \n\t "))
        });

    let Some(update) = parse_update(message_update(904, 10, "bad request")) else {
        return Ok(());
    };

    assert!(router.dispatch(BotContext::new(client), update).await?);
    join_server(handle).await?;
    Ok(())
}

#[tokio::test]
async fn bot_context_app_answers_callback_from_update() -> Result<(), DynError> {
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
        .app()
        .answer_callback_from_update(&update, Some("received".to_owned()))
        .await?;
    assert!(answered);

    join_server(handle).await?;
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
async fn chat_session_transition_serializes_concurrent_updates_per_chat() -> Result<(), DynError> {
    let session = ChatSession::<usize, _>::new(InMemorySessionStore::new());
    let Some(update) = parse_update(message_update(1004, 23, "state")) else {
        return Ok(());
    };
    session.save(&update, 0).await?;

    let mut tasks = Vec::new();
    for _ in 0..16 {
        let session = session.clone();
        let update = update.clone();
        tasks.push(tokio::spawn(async move {
            session
                .transition(&update, |state| async move {
                    tokio::time::sleep(Duration::from_millis(5)).await;
                    ((), StateTransition::Set(state.unwrap_or_default() + 1))
                })
                .await
        }));
    }

    for task in tasks {
        task.await??;
    }

    assert_eq!(session.load(&update).await?, Some(16));
    Ok(())
}

#[tokio::test]
async fn chat_session_load_waits_for_in_flight_transition() -> Result<(), DynError> {
    let session = ChatSession::<usize, _>::new(InMemorySessionStore::new());
    let Some(update) = parse_update(message_update(1006, 25, "state")) else {
        return Ok(());
    };
    session.save(&update, 0).await?;

    let (started_tx, started_rx) = tokio::sync::oneshot::channel();
    let (release_tx, release_rx) = tokio::sync::oneshot::channel();
    let transition_session = session.clone();
    let transition_update = update.clone();
    let transition = tokio::spawn(async move {
        transition_session
            .transition(&transition_update, |state| async move {
                let _ = started_tx.send(());
                let _ = release_rx.await;
                ((), StateTransition::Set(state.unwrap_or_default() + 1))
            })
            .await
    });
    started_rx.await?;

    let load_session = session.clone();
    let load_update = update.clone();
    let load = tokio::spawn(async move { load_session.load(&load_update).await });

    tokio::time::sleep(Duration::from_millis(20)).await;
    assert!(
        !load.is_finished(),
        "load returned stale state while transition was still in flight"
    );

    if release_tx.send(()).is_err() {
        return Err("transition task dropped before release".into());
    }
    transition.await??;
    assert_eq!(load.await??, Some(1));
    Ok(())
}

#[tokio::test]
async fn chat_session_from_shared_reuses_per_chat_locks() -> Result<(), DynError> {
    let store = Arc::new(InMemorySessionStore::<usize>::new());
    let writer_a = ChatSession::from_shared(Arc::clone(&store));
    let writer_b = ChatSession::from_shared(store);
    let Some(update) = parse_update(message_update(1005, 24, "state")) else {
        return Ok(());
    };
    writer_a.save(&update, 0).await?;

    let mut tasks = Vec::new();
    for index in 0..16 {
        let session = if index % 2 == 0 {
            writer_a.clone()
        } else {
            writer_b.clone()
        };
        let update = update.clone();
        tasks.push(tokio::spawn(async move {
            session
                .transition(&update, |state| async move {
                    tokio::time::sleep(Duration::from_millis(5)).await;
                    ((), StateTransition::Set(state.unwrap_or_default() + 1))
                })
                .await
        }));
    }

    for task in tasks {
        task.await??;
    }

    assert_eq!(writer_a.load(&update).await?, Some(16));
    assert_eq!(writer_b.load(&update).await?, Some(16));
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
        router
            .command_route("start")?
            .handle(move |_context: BotContext, _update: Update| {
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
        continue_on_handler_error: true,
        max_handler_concurrency: 3,
        ..EngineConfig::default()
    });

    let outcomes = engine.poll_once().await?;
    assert_eq!(outcomes.len(), 3);
    assert_eq!(engine.source_mut().next_offset(), Some(304));
    assert_eq!(handled.load(Ordering::SeqCst), 3);
    assert!(max_in_flight.load(Ordering::SeqCst) >= 2);

    join_server(handle).await?;
    Ok(())
}

#[tokio::test]
async fn bot_engine_concurrent_outcomes_preserve_source_order() -> Result<(), DynError> {
    let client = Client::builder("http://127.0.0.1:9")?
        .bot_token("123:abc")?
        .build()?;

    let mut router = Router::new();
    router
        .message_route()
        .handle(|_context: BotContext, update: Update| async move {
            let delay = match update.update_id {
                501 => Duration::from_millis(60),
                502 => Duration::from_millis(20),
                _ => Duration::ZERO,
            };
            if !delay.is_zero() {
                tokio::time::sleep(delay).await;
            }
            Ok(())
        });

    let (sink, source) = channel_source(8)?;
    let mut engine = BotEngine::new(client, source, router).with_config(EngineConfig {
        continue_on_source_error: false,
        continue_on_handler_error: true,
        max_handler_concurrency: 3,
        ..EngineConfig::default()
    });

    sink.send(tele::bot::testing::message_update(501, 1, "slow")?)
        .await?;
    sink.send(tele::bot::testing::message_update(502, 1, "medium")?)
        .await?;
    sink.send(tele::bot::testing::message_update(503, 1, "fast")?)
        .await?;

    let outcomes = engine.poll_once().await?;
    assert_eq!(
        outcomes,
        vec![
            DispatchOutcome::Handled { update_id: 501 },
            DispatchOutcome::Handled { update_id: 502 },
            DispatchOutcome::Handled { update_id: 503 },
        ]
    );

    Ok(())
}

#[tokio::test]
async fn fail_fast_dispatch_stays_serial_even_when_concurrency_configured() -> Result<(), DynError>
{
    let response = r#"{"ok":true,"result":[{"update_id":601,"message":{"message_id":1,"date":1710000001,"chat":{"id":77,"type":"private"},"text":"ok"}},{"update_id":602,"message":{"message_id":2,"date":1710000002,"chat":{"id":77,"type":"private"},"text":"fail"}},{"update_id":603,"message":{"message_id":3,"date":1710000003,"chat":{"id":77,"type":"private"},"text":"must-not-run"}}]}"#;
    let (base_url, handle) = spawn_server("/bot123:abc/getUpdates", 200, response)?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let executed = Arc::new(Mutex::new(Vec::<i64>::new()));

    let mut router = Router::new();
    {
        let executed = Arc::clone(&executed);
        router
            .message_route()
            .handle(move |_context: BotContext, update: Update| {
                let executed = Arc::clone(&executed);
                async move {
                    {
                        let mut executed = executed.lock().map_err(|error| {
                            HandlerError::internal(Error::InvalidRequest {
                                reason: error.to_string(),
                            })
                        })?;
                        executed.push(update.update_id);
                    }
                    tokio::time::sleep(Duration::from_millis(10)).await;
                    if update.update_id == 602 {
                        return Err(HandlerError::internal(Error::InvalidRequest {
                            reason: "intentional handler failure".to_owned(),
                        }));
                    }
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

    let error = match engine.poll_once().await {
        Ok(_) => return Err("expected fail-fast handler failure".into()),
        Err(error) => error,
    };
    assert!(matches!(error, Error::InvalidRequest { .. }));
    assert_eq!(engine.source_mut().next_offset(), Some(602));
    assert_eq!(
        *executed.lock().map_err(|error| error.to_string())?,
        vec![601, 602]
    );

    join_server(handle).await?;
    Ok(())
}

#[tokio::test]
async fn long_polling_commits_all_outcomes_when_handler_errors_are_continued()
-> Result<(), DynError> {
    let response = r#"{"ok":true,"result":[{"update_id":401,"message":{"message_id":1,"date":1710000001,"chat":{"id":77,"type":"private"},"text":"ok"}},{"update_id":402,"message":{"message_id":2,"date":1710000002,"chat":{"id":77,"type":"private"},"text":"fail"}},{"update_id":403,"message":{"message_id":3,"date":1710000003,"chat":{"id":77,"type":"private"},"text":"ok"}}]}"#;
    let (base_url, handle) = spawn_server("/bot123:abc/getUpdates", 200, response)?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let mut router = Router::new();
    router
        .message_route()
        .handle(|_context: BotContext, update: Update| async move {
            if update.update_id == 402 {
                return Err(HandlerError::internal(Error::InvalidRequest {
                    reason: "intentional handler failure".to_owned(),
                }));
            }
            Ok(())
        });

    let source = LongPollingSource::new(client.clone()).with_config(PollingConfig {
        disable_webhook_on_start: false,
        poll_timeout_seconds: 1,
        ..PollingConfig::default()
    });
    let mut engine = BotEngine::new(client, source, router).with_config(EngineConfig {
        continue_on_source_error: false,
        continue_on_handler_error: true,
        ..EngineConfig::default()
    });

    let outcomes = engine.poll_once().await?;
    assert_eq!(
        outcomes,
        vec![
            DispatchOutcome::Handled { update_id: 401 },
            DispatchOutcome::Failed { update_id: 402 },
            DispatchOutcome::Handled { update_id: 403 },
        ]
    );
    assert_eq!(engine.source_mut().next_offset(), Some(404));

    join_server(handle).await?;
    Ok(())
}

#[tokio::test]
async fn concurrent_handler_panic_fails_cycle_without_ack() -> Result<(), DynError> {
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let offset_path =
        std::env::temp_dir().join(format!("tele-offset-concurrent-panic-{timestamp}.json"));
    let response = r#"{"ok":true,"result":[{"update_id":304,"message":{"message_id":1,"date":1710000004,"chat":{"id":77,"type":"private"},"text":"/panic"}}]}"#;
    let (base_url, handle) = spawn_server("/bot123:abc/getUpdates", 200, response)?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let mut router = Router::new();
    router
        .command_route("panic")?
        .handle(|_context: BotContext, _update: Update| async move {
            std::panic::resume_unwind(Box::new("intentional handler panic"));
        });

    let source = LongPollingSource::new(client.clone()).with_config(PollingConfig {
        disable_webhook_on_start: false,
        persist_offset_path: Some(offset_path.clone()),
        poll_timeout_seconds: 1,
        ..PollingConfig::default()
    });
    let mut engine = BotEngine::new(client, source, router).with_config(EngineConfig {
        continue_on_source_error: false,
        continue_on_handler_error: true,
        max_handler_concurrency: 2,
        ..EngineConfig::default()
    });

    let error = match engine.poll_once().await {
        Ok(outcomes) => {
            return Err(
                format!("handler panic unexpectedly produced outcomes: {outcomes:?}").into(),
            );
        }
        Err(error) => error,
    };
    assert!(matches!(error, Error::InvalidRequest { .. }));
    assert_eq!(engine.source_mut().next_offset(), None);
    assert!(!offset_path.exists());

    join_server(handle).await?;
    Ok(())
}

#[test]
fn command_route_rejects_invalid_registration_names() {
    for command in ["", "/start", "Start", "start@ThisBot", "bad-command"] {
        let mut router = Router::new();
        assert!(matches!(
            router.command_route(command),
            Err(Error::InvalidRequest { .. })
        ));
    }
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
        let text_hits = Arc::clone(&text_hits);
        router.extracted_route::<TextInput>().handle(
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
    {
        let callback_hits = Arc::clone(&callback_hits);
        router.extracted_route::<CallbackInput>().handle(
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
async fn chat_join_request_route_dispatches_typed_input() -> Result<(), DynError> {
    let client = Client::builder("http://127.0.0.1:9")?
        .bot_token("123:abc")?
        .build()?;

    let hits = Arc::new(AtomicUsize::new(0));
    let mut router = Router::new();
    {
        let hits = Arc::clone(&hits);
        router.chat_join_request_route().handle(
            move |_context: BotContext, _update: Update, request: ChatJoinRequestInput| {
                let hits = Arc::clone(&hits);
                async move {
                    if request.0.from.id.0 == 501 && request.0.chat.id == -2001 {
                        hits.fetch_add(1, Ordering::SeqCst);
                    }
                    Ok(())
                }
            },
        );
    }

    let Some(update) = parse_update(serde_json::json!({
        "update_id": 4003,
        "chat_join_request": {
            "chat": {"id": -2001, "type": "supergroup", "title": "screening"},
            "from": {"id": 501, "is_bot": false, "first_name": "candidate"},
            "user_chat_id": 99001,
            "date": 1700000403
        }
    })) else {
        return Ok(());
    };

    assert!(router.dispatch(BotContext::new(client), update).await?);
    assert_eq!(hits.load(Ordering::SeqCst), 1);

    Ok(())
}

#[tokio::test]
async fn member_update_routes_dispatch_typed_input() -> Result<(), DynError> {
    let client = Client::builder("http://127.0.0.1:9")?
        .bot_token("123:abc")?
        .build()?;

    let chat_member_hits = Arc::new(AtomicUsize::new(0));
    let my_chat_member_hits = Arc::new(AtomicUsize::new(0));
    let mut router = Router::new();
    {
        let chat_member_hits = Arc::clone(&chat_member_hits);
        router.chat_member_route().handle(
            move |_context: BotContext, _update: Update, member_update: ChatMemberUpdatedInput| {
                let chat_member_hits = Arc::clone(&chat_member_hits);
                async move {
                    if member_update.0.subject().id.0 == 601 && member_update.0.chat.id == -2101 {
                        chat_member_hits.fetch_add(1, Ordering::SeqCst);
                    }
                    Ok(())
                }
            },
        );
    }
    {
        let my_chat_member_hits = Arc::clone(&my_chat_member_hits);
        router.my_chat_member_route().handle(
            move |_context: BotContext,
                  _update: Update,
                  member_update: MyChatMemberUpdatedInput| {
                let my_chat_member_hits = Arc::clone(&my_chat_member_hits);
                async move {
                    if member_update.0.subject().id.0 == 999 && member_update.0.chat.id == -2102 {
                        my_chat_member_hits.fetch_add(1, Ordering::SeqCst);
                    }
                    Ok(())
                }
            },
        );
    }

    let Some(chat_member_update) = parse_update(serde_json::json!({
        "update_id": 4004,
        "chat_member": {
            "chat": {"id": -2101, "type": "supergroup", "title": "screening"},
            "from": {"id": 1, "is_bot": false, "first_name": "admin"},
            "date": 1700000404,
            "old_chat_member": {
                "status": "left",
                "user": {"id": 601, "is_bot": false, "first_name": "candidate"}
            },
            "new_chat_member": {
                "status": "member",
                "user": {"id": 601, "is_bot": false, "first_name": "candidate"}
            }
        }
    })) else {
        return Ok(());
    };
    let Some(my_chat_member_update) = parse_update(serde_json::json!({
        "update_id": 4005,
        "my_chat_member": {
            "chat": {"id": -2102, "type": "supergroup", "title": "screening"},
            "from": {"id": 1, "is_bot": false, "first_name": "admin"},
            "date": 1700000405,
            "old_chat_member": {
                "status": "member",
                "user": {"id": 999, "is_bot": true, "first_name": "tele"}
            },
            "new_chat_member": {
                "status": "administrator",
                "user": {"id": 999, "is_bot": true, "first_name": "tele"},
                "can_manage_chat": true
            }
        }
    })) else {
        return Ok(());
    };

    assert!(
        router
            .dispatch(BotContext::new(client.clone()), chat_member_update)
            .await?
    );
    assert!(
        router
            .dispatch(BotContext::new(client), my_chat_member_update)
            .await?
    );
    assert_eq!(chat_member_hits.load(Ordering::SeqCst), 1);
    assert_eq!(my_chat_member_hits.load(Ordering::SeqCst), 1);

    Ok(())
}

#[tokio::test]
async fn modern_update_routes_dispatch_typed_inputs() -> Result<(), DynError> {
    let client = Client::builder("http://127.0.0.1:9")?
        .bot_token("123:abc")?
        .build()?;

    let hits = Arc::new(AtomicUsize::new(0));
    let mut router = Router::new();
    {
        let hits = Arc::clone(&hits);
        router.business_connection_route().handle(
            move |_context: BotContext, _update: Update, connection: BusinessConnectionInput| {
                let hits = Arc::clone(&hits);
                async move {
                    if connection.0.id == "business-route" && connection.0.user_chat_id == 7007 {
                        hits.fetch_add(1, Ordering::SeqCst);
                    }
                    Ok(())
                }
            },
        );
    }
    {
        let hits = Arc::clone(&hits);
        router.deleted_business_messages_route().handle(
            move |_context: BotContext, _update: Update, deleted: BusinessMessagesDeletedInput| {
                let hits = Arc::clone(&hits);
                async move {
                    if deleted.0.business_connection_id == "business-route"
                        && deleted.0.message_ids.len() == 2
                    {
                        hits.fetch_add(1, Ordering::SeqCst);
                    }
                    Ok(())
                }
            },
        );
    }
    {
        let hits = Arc::clone(&hits);
        router.message_reaction_route().handle(
            move |_context: BotContext, _update: Update, reaction: MessageReactionInput| {
                let hits = Arc::clone(&hits);
                async move {
                    if reaction.0.chat.id == -3001 && reaction.0.message_id.0 == 10 {
                        hits.fetch_add(1, Ordering::SeqCst);
                    }
                    Ok(())
                }
            },
        );
    }
    {
        let hits = Arc::clone(&hits);
        router.shipping_query_route().handle(
            move |_context: BotContext, _update: Update, query: ShippingQueryInput| {
                let hits = Arc::clone(&hits);
                async move {
                    if query.0.id == "shipping-route"
                        && query.0.shipping_address.country_code == "US"
                    {
                        hits.fetch_add(1, Ordering::SeqCst);
                    }
                    Ok(())
                }
            },
        );
    }
    {
        let hits = Arc::clone(&hits);
        router.poll_answer_route().handle(
            move |_context: BotContext, _update: Update, answer: PollAnswerInput| {
                let hits = Arc::clone(&hits);
                async move {
                    if answer.0.poll_id == "poll-route" && answer.0.option_ids == [1] {
                        hits.fetch_add(1, Ordering::SeqCst);
                    }
                    Ok(())
                }
            },
        );
    }
    {
        let hits = Arc::clone(&hits);
        router.chat_boost_route().handle(
            move |_context: BotContext, _update: Update, boost: ChatBoostInput| {
                let hits = Arc::clone(&hits);
                async move {
                    if boost.0.chat.id == -3002 && boost.0.boost.boost_id == "boost-route" {
                        hits.fetch_add(1, Ordering::SeqCst);
                    }
                    Ok(())
                }
            },
        );
    }
    {
        let hits = Arc::clone(&hits);
        router.managed_bot_route().handle(
            move |_context: BotContext, _update: Update, managed: ManagedBotInput| {
                let hits = Arc::clone(&hits);
                async move {
                    if managed.0.user.id.0 == 71 && managed.0.bot.id.0 == 72 {
                        hits.fetch_add(1, Ordering::SeqCst);
                    }
                    Ok(())
                }
            },
        );
    }
    {
        let hits = Arc::clone(&hits);
        router.callback_query_input_route().handle(
            move |_context: BotContext, _update: Update, callback: CallbackQueryInput| {
                let hits = Arc::clone(&hits);
                async move {
                    if callback.0.id == "callback-route"
                        && callback.0.data.as_deref() == Some("payload")
                    {
                        hits.fetch_add(1, Ordering::SeqCst);
                    }
                    Ok(())
                }
            },
        );
    }
    {
        let hits = Arc::clone(&hits);
        router.inline_query_input_route().handle(
            move |_context: BotContext, _update: Update, inline: InlineQueryInput| {
                let hits = Arc::clone(&hits);
                async move {
                    if inline.0.id == "inline-route" && inline.0.query == "search" {
                        hits.fetch_add(1, Ordering::SeqCst);
                    }
                    Ok(())
                }
            },
        );
    }

    let payloads = [
        serde_json::json!({
            "update_id": 4010,
            "business_connection": {
                "id": "business-route",
                "user": {"id": 7007, "is_bot": false, "first_name": "customer"},
                "user_chat_id": 7007,
                "date": 1700000410,
                "is_enabled": true
            }
        }),
        serde_json::json!({
            "update_id": 4011,
            "deleted_business_messages": {
                "business_connection_id": "business-route",
                "chat": {"id": 7007, "type": "private", "first_name": "customer"},
                "message_ids": [10, 11]
            }
        }),
        serde_json::json!({
            "update_id": 4012,
            "message_reaction": {
                "chat": {"id": -3001, "type": "supergroup", "title": "mods"},
                "message_id": 10,
                "user": {"id": 70, "is_bot": false, "first_name": "reactor"},
                "date": 1700000412,
                "old_reaction": [],
                "new_reaction": [{"type": "emoji", "emoji": "👍"}]
            }
        }),
        serde_json::json!({
            "update_id": 4013,
            "shipping_query": {
                "id": "shipping-route",
                "from": {"id": 70, "is_bot": false, "first_name": "buyer"},
                "invoice_payload": "payload",
                "shipping_address": {
                    "country_code": "US",
                    "state": "CA",
                    "city": "San Francisco",
                    "street_line1": "1 Market St",
                    "street_line2": "Suite 1",
                    "post_code": "94105"
                }
            }
        }),
        serde_json::json!({
            "update_id": 4014,
            "poll_answer": {
                "poll_id": "poll-route",
                "user": {"id": 70, "is_bot": false, "first_name": "voter"},
                "option_ids": [1]
            }
        }),
        serde_json::json!({
            "update_id": 4015,
            "chat_boost": {
                "chat": {"id": -3002, "type": "supergroup", "title": "mods"},
                "boost": {
                    "boost_id": "boost-route",
                    "add_date": 1700000415,
                    "expiration_date": 1700086815,
                    "source": {
                        "source": "premium",
                        "user": {"id": 70, "is_bot": false, "first_name": "booster"}
                    }
                }
            }
        }),
        serde_json::json!({
            "update_id": 4016,
            "managed_bot": {
                "user": {"id": 71, "is_bot": false, "first_name": "owner"},
                "bot": {"id": 72, "is_bot": true, "first_name": "managed"}
            }
        }),
        serde_json::json!({
            "update_id": 4017,
            "callback_query": {
                "id": "callback-route",
                "from": {"id": 70, "is_bot": false, "first_name": "clicker"},
                "chat_instance": "ci",
                "data": "payload"
            }
        }),
        serde_json::json!({
            "update_id": 4018,
            "inline_query": {
                "id": "inline-route",
                "from": {"id": 70, "is_bot": false, "first_name": "searcher"},
                "query": "search",
                "offset": ""
            }
        }),
    ];

    let expected_hits = payloads.len();
    for payload in payloads {
        let Some(update) = parse_update(payload) else {
            return Ok(());
        };
        assert!(
            router
                .dispatch(BotContext::new(client.clone()), update)
                .await?
        );
    }
    assert_eq!(hits.load(Ordering::SeqCst), expected_hits);

    Ok(())
}

#[tokio::test]
async fn extractor_combinators_filter_map_guard_work() -> Result<(), DynError> {
    let client = Client::builder("http://127.0.0.1:9")?
        .bot_token("123:abc")?
        .build()?;

    let filter_hits = Arc::new(AtomicUsize::new(0));
    let filter_calls = Arc::new(AtomicUsize::new(0));
    let mut filter_router = Router::new();
    {
        let filter_hits = Arc::clone(&filter_hits);
        let filter_calls = Arc::clone(&filter_calls);
        filter_router
            .extracted_route::<TextInput>()
            .filter(move |text, _update| {
                filter_calls.fetch_add(1, Ordering::SeqCst);
                text.0.starts_with("allow")
            })
            .handle(move |_context: BotContext, _update: Update, _text| {
                let filter_hits = Arc::clone(&filter_hits);
                async move {
                    filter_hits.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                }
            });
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
    assert_eq!(filter_calls.load(Ordering::SeqCst), 2);

    let map_hits = Arc::new(AtomicUsize::new(0));
    let map_calls = Arc::new(AtomicUsize::new(0));
    let mut map_router = Router::new();
    {
        let map_hits = Arc::clone(&map_hits);
        let map_calls = Arc::clone(&map_calls);
        map_router
            .extracted_route::<CallbackInput>()
            .map(move |callback, _update| {
                map_calls.fetch_add(1, Ordering::SeqCst);
                let value: serde_json::Value = serde_json::from_str(&callback.0).ok()?;
                Some(value.get("action")?.as_str()?.to_owned())
            })
            .handle(
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
    assert_eq!(map_calls.load(Ordering::SeqCst), 2);

    let guard_hits = Arc::new(AtomicUsize::new(0));
    let mut guard_router = Router::new();
    {
        let guard_hits = Arc::clone(&guard_hits);
        guard_router
            .extracted_route::<TextInput>()
            .guard(reject_blocked_text)
            .handle(move |_context: BotContext, _update: Update, _text| {
                let guard_hits = Arc::clone(&guard_hits);
                async move {
                    guard_hits.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                }
            });
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
    let response = r#"{"ok":true,"result":{"message_id":120,"date":1710000009,"chat":{"id":10,"type":"private"},"text":"request failed, please try again later"}}"#;
    let (base_url, handle) = spawn_server_with_checks(
        "/bot123:abc/sendMessage",
        200,
        response,
        &["\"text\":\"request failed, please try again later\""],
    )?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let mut router = Router::new();
    router.command_route("start")?.handle_with_policy(
        ErrorPolicy::ReplyUser {
            fallback_message: " \n\t ".to_owned(),
        },
        |_context: BotContext, _update: Update| async move {
            Err(HandlerError::internal(Error::Transport {
                method: "sendMessage".to_owned(),
                status: Some(502),
                request_id: None,
                retry_after: None,
                request_path: None,
                message: "upstream unavailable".into(),
            }))
        },
    );

    let Some(update) = parse_update(message_update(4101, 10, "/start")) else {
        return Ok(());
    };

    assert!(router.dispatch(BotContext::new(client), update).await?);
    join_server(handle).await?;
    Ok(())
}

#[tokio::test]
async fn join_request_error_reply_targets_user_chat_id() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":{"message_id":121,"date":1710000010,"chat":{"id":7001,"type":"private"},"text":"temporary failure"}}"#;
    let (base_url, handle) = spawn_server_with_checks(
        "/bot123:abc/sendMessage",
        200,
        response,
        &["\"chat_id\":7001", "\"text\":\"temporary failure\""],
    )?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let mut router = Router::new();
    router.chat_join_request_route().handle_with_policy(
        ErrorPolicy::ReplyUser {
            fallback_message: "temporary failure".to_owned(),
        },
        |_context: BotContext, _update: Update, _request| async move {
            Err(HandlerError::internal(Error::Transport {
                method: "sendMessage".to_owned(),
                status: Some(502),
                request_id: None,
                retry_after: None,
                request_path: None,
                message: "upstream unavailable".into(),
            }))
        },
    );

    let Some(update) = parse_update(serde_json::json!({
        "update_id": 4102,
        "chat_join_request": {
            "chat": {"id": -10010, "type": "supergroup", "title": "mods"},
            "from": {"id": 701, "is_bot": false, "first_name": "candidate"},
            "user_chat_id": 7001,
            "date": 1700000412
        }
    })) else {
        return Ok(());
    };

    assert!(router.dispatch(BotContext::new(client), update).await?);
    join_server(handle).await?;
    Ok(())
}

#[tokio::test]
async fn outbox_dedupes_and_retries() -> Result<(), DynError> {
    let retry_response = r#"{"ok":false,"error_code":502,"description":"bad gateway"}"#;
    let ok_response = r#"{"ok":true,"result":{"message_id":88,"date":1710000010,"chat":{"id":12,"type":"private"},"text":"hello"}}"#;
    let (base_url, handle) = spawn_server_sequence(
        "/bot123:abc/sendMessage",
        vec![(502, retry_response), (200, ok_response)],
    )?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let mut config = OutboxConfig::default();
    config.max_attempts = 3;
    config.base_backoff = Duration::from_millis(1);
    config.max_backoff = Duration::from_millis(1);
    config.dedupe_ttl = Duration::from_secs(60);
    let outbox = BotOutbox::spawn(client, config)?;

    let first = outbox
        .send_text_with_key(12_i64, "hello", Some("msg-1".to_owned()))
        .await?;
    let second = outbox
        .send_text_with_key(12_i64, "hello", Some("msg-1".to_owned()))
        .await?;

    assert_eq!(first.message_id, second.message_id);
    join_server(handle).await?;
    Ok(())
}

#[tokio::test]
async fn outbox_rejects_idempotency_key_reuse_with_different_payload() -> Result<(), DynError> {
    let ok_response = r#"{"ok":true,"result":{"message_id":89,"date":1710000011,"chat":{"id":12,"type":"private"},"text":"hello"}}"#;
    let (base_url, handle) = spawn_server("/bot123:abc/sendMessage", 200, ok_response)?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let mut config = OutboxConfig::default();
    config.max_attempts = 1;
    config.dedupe_ttl = Duration::from_secs(60);
    let outbox = BotOutbox::spawn(client, config)?;

    let first = outbox
        .send_text_with_key(12_i64, "hello", Some("msg-reused".to_owned()))
        .await?;
    assert_eq!(first.message_id.0, 89);

    let second = outbox
        .send_text_with_key(12_i64, "different", Some("msg-reused".to_owned()))
        .await;
    assert!(matches!(second, Err(Error::InvalidRequest { .. })));

    join_server(handle).await?;
    Ok(())
}

#[tokio::test]
async fn outbox_reports_commit_failure_after_delivery() -> Result<(), DynError> {
    let ok_response = r#"{"ok":true,"result":{"message_id":90,"date":1710000012,"chat":{"id":12,"type":"private"},"text":"hello"}}"#;
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let root = std::env::temp_dir().join(format!("tele-outbox-commit-failure-{timestamp}"));
    fs::create_dir_all(&root)?;
    let queue_path = root.join("queue.json");
    let hook_queue_path = queue_path.clone();
    let (base_url, handle) =
        spawn_server_with_request_hook("/bot123:abc/sendMessage", 200, ok_response, move |_| {
            if !hook_queue_path.is_file() {
                return Err(format!(
                    "expected persisted outbox queue before response: {}",
                    hook_queue_path.display()
                ));
            }

            fs::remove_file(&hook_queue_path).map_err(|error| error.to_string())?;
            fs::create_dir(&hook_queue_path).map_err(|error| error.to_string())?;
            Ok(())
        })?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let config = OutboxConfig::default().with_persistence_path(queue_path.clone());
    let outbox = BotOutbox::spawn(client, config)?;

    let result = outbox
        .send_text_with_key(12_i64, "hello", Some("commit-failure".to_owned()))
        .await;
    join_server(handle).await?;

    match result {
        Err(Error::Storage {
            operation,
            message,
            retryable,
        }) => {
            assert_eq!(operation.as_ref(), "outbox commit");
            assert!(!retryable);
            assert!(message.contains("upstream delivery may have succeeded"));
        }
        other => {
            return Err(format!("expected delivered commit storage error, got {other:?}").into());
        }
    }

    assert!(queue_path.is_dir());

    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[tokio::test]
async fn outbox_rejects_invalid_message_before_enqueue() -> Result<(), DynError> {
    let client = Client::builder("http://127.0.0.1:9")?
        .bot_token("123:abc")?
        .build()?;
    let outbox = BotOutbox::spawn(client, OutboxConfig::default())?;

    assert!(matches!(
        outbox.send_text(12_i64, "").await,
        Err(Error::InvalidRequest { .. })
    ));
    assert!(matches!(
        outbox
            .send_text_with_key(12_i64, "hello", Some(String::new()))
            .await,
        Err(Error::InvalidRequest { .. })
    ));
    assert!(matches!(
        outbox
            .send_text_with_key(12_i64, "hello", Some("x".repeat(257)))
            .await,
        Err(Error::InvalidRequest { .. })
    ));

    Ok(())
}

#[tokio::test]
async fn outbox_rejects_invalid_config_on_spawn() -> Result<(), DynError> {
    let client = Client::builder("http://127.0.0.1:9")?
        .bot_token("123:abc")?
        .build()?;
    let mut config = OutboxConfig::default();
    config.queue_capacity = 0;

    let error = match BotOutbox::spawn(client, config) {
        Ok(_) => return Err("expected invalid outbox config to be rejected".into()),
        Err(error) => error,
    };
    assert!(matches!(error, Error::InvalidRequest { .. }));
    assert!(error.to_string().contains("queue_capacity"));
    Ok(())
}

#[tokio::test]
async fn outbox_rejects_invalid_persistence_path_on_spawn() -> Result<(), DynError> {
    let (root, queue_path) = blocked_child_path("tele-outbox-invalid-target", "queue.json")?;
    let client = Client::builder("http://127.0.0.1:9")?
        .bot_token("123:abc")?
        .build()?;
    let config = OutboxConfig::default().with_persistence_path(queue_path);

    let error = match BotOutbox::spawn(client, config) {
        Ok(_) => return Err("expected outbox spawn to reject invalid persistence target".into()),
        Err(error) => error,
    };

    assert!(matches!(error, Error::Storage { .. }));
    assert!(error.to_string().contains("outbox snapshot"));
    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[tokio::test]
async fn outbox_rejects_invalid_dead_letter_path_on_spawn() -> Result<(), DynError> {
    let (root, dead_letter_path) =
        blocked_child_path("tele-dead-letter-invalid-target", "dead-letter.json")?;
    let client = Client::builder("http://127.0.0.1:9")?
        .bot_token("123:abc")?
        .build()?;
    let config = OutboxConfig::default().with_dead_letter_path(dead_letter_path);

    let error = match BotOutbox::spawn(client, config) {
        Ok(_) => return Err("expected outbox spawn to reject invalid dead-letter target".into()),
        Err(error) => error,
    };

    assert!(matches!(error, Error::Storage { .. }));
    assert!(error.to_string().contains("dead-letter snapshot"));
    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[tokio::test]
async fn outbox_rejects_invalid_persisted_queue_on_spawn() -> Result<(), DynError> {
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let queue_path = std::env::temp_dir().join(format!("tele-outbox-invalid-{timestamp}.json"));
    fs::write(&queue_path, b"{invalid-json")?;

    let client = Client::builder("http://127.0.0.1:9")?
        .bot_token("123:abc")?
        .build()?;
    let config = OutboxConfig::default().with_persistence_path(queue_path.clone());
    let error = match BotOutbox::spawn(client, config) {
        Ok(_) => return Err("expected outbox spawn to reject invalid persistence".into()),
        Err(error) => error,
    };
    assert!(matches!(error, Error::Storage { .. }));
    assert!(error.to_string().contains("outbox snapshot"));

    let raw = fs::read_to_string(&queue_path)?;
    assert_eq!(raw, "{invalid-json");

    let _ = fs::remove_file(queue_path);
    Ok(())
}

#[tokio::test]
async fn outbox_rejects_invalid_persisted_queue_entry_on_spawn() -> Result<(), DynError> {
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let queue_path =
        std::env::temp_dir().join(format!("tele-outbox-invalid-entry-{timestamp}.json"));
    let snapshot = serde_json::json!({
        "version": 1,
        "queue": [{
            "chat_id": 0,
            "text": "hello",
            "idempotency_key": "persisted-1",
            "enqueued_at_unix_ms": 1,
            "attempt": 0
        }]
    });
    fs::write(&queue_path, serde_json::to_vec(&snapshot)?)?;

    let client = Client::builder("http://127.0.0.1:9")?
        .bot_token("123:abc")?
        .build()?;
    let config = OutboxConfig::default().with_persistence_path(queue_path.clone());
    let error = match BotOutbox::spawn(client, config) {
        Ok(_) => return Err("expected invalid outbox queue entry to be rejected".into()),
        Err(error) => error,
    };
    assert!(matches!(error, Error::Storage { .. }));
    assert!(error.to_string().contains("queue entry 0"));

    let _ = fs::remove_file(queue_path);
    Ok(())
}

#[tokio::test]
async fn outbox_rejects_persisted_queue_over_capacity_on_spawn() -> Result<(), DynError> {
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let queue_path =
        std::env::temp_dir().join(format!("tele-outbox-over-capacity-{timestamp}.json"));
    let snapshot = serde_json::json!({
        "version": 1,
        "queue": [
            {
                "chat_id": 12,
                "text": "first",
                "idempotency_key": "persisted-1",
                "enqueued_at_unix_ms": 1,
                "attempt": 0
            },
            {
                "chat_id": 12,
                "text": "second",
                "idempotency_key": "persisted-2",
                "enqueued_at_unix_ms": 2,
                "attempt": 0
            }
        ]
    });
    fs::write(&queue_path, serde_json::to_vec(&snapshot)?)?;

    let client = Client::builder("http://127.0.0.1:9")?
        .bot_token("123:abc")?
        .build()?;
    let mut config = OutboxConfig::default().with_persistence_path(queue_path.clone());
    config.queue_capacity = 1;
    let error = match BotOutbox::spawn(client, config) {
        Ok(_) => return Err("expected over-capacity outbox queue to be rejected".into()),
        Err(error) => error,
    };
    assert!(matches!(error, Error::Storage { .. }));
    assert!(error.to_string().contains("queue_capacity"));

    let _ = fs::remove_file(queue_path);
    Ok(())
}

#[tokio::test]
async fn outbox_rejects_exhausted_persisted_queue_entry_on_spawn() -> Result<(), DynError> {
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let queue_path =
        std::env::temp_dir().join(format!("tele-outbox-exhausted-entry-{timestamp}.json"));
    let snapshot = serde_json::json!({
        "version": 1,
        "queue": [{
            "chat_id": 12,
            "text": "hello",
            "idempotency_key": "persisted-1",
            "enqueued_at_unix_ms": 1,
            "attempt": 1
        }]
    });
    fs::write(&queue_path, serde_json::to_vec(&snapshot)?)?;

    let client = Client::builder("http://127.0.0.1:9")?
        .bot_token("123:abc")?
        .build()?;
    let mut config = OutboxConfig::default().with_persistence_path(queue_path.clone());
    config.max_attempts = 1;
    let error = match BotOutbox::spawn(client, config) {
        Ok(_) => return Err("expected exhausted outbox queue entry to be rejected".into()),
        Err(error) => error,
    };
    assert!(matches!(error, Error::Storage { .. }));
    assert!(error.to_string().contains("no remaining retry budget"));

    let _ = fs::remove_file(queue_path);
    Ok(())
}

#[tokio::test]
async fn outbox_rejects_invalid_dead_letter_snapshot_on_spawn() -> Result<(), DynError> {
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let dead_letter_path =
        std::env::temp_dir().join(format!("tele-dead-letter-invalid-{timestamp}.json"));
    let snapshot = serde_json::json!({
        "version": 1,
        "entries": [{
            "chat_id": 12,
            "text": "hello",
            "idempotency_key": "dead-letter-1",
            "attempts": 1,
            "reason": "",
            "enqueued_at_unix_ms": 1,
            "failed_at_unix_ms": 2
        }]
    });
    fs::write(&dead_letter_path, serde_json::to_vec(&snapshot)?)?;

    let client = Client::builder("http://127.0.0.1:9")?
        .bot_token("123:abc")?
        .build()?;
    let config = OutboxConfig::default().with_dead_letter_path(dead_letter_path.clone());
    let error = match BotOutbox::spawn(client, config) {
        Ok(_) => return Err("expected invalid dead-letter snapshot to be rejected".into()),
        Err(error) => error,
    };
    assert!(matches!(error, Error::Storage { .. }));
    assert!(error.to_string().contains("dead-letter"));

    let _ = fs::remove_file(dead_letter_path);
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
    let _outbox = BotOutbox::spawn(client, config)?;

    join_server(handle).await?;

    wait_for_condition(Duration::from_secs(2), Duration::from_millis(20), || {
        let raw = fs::read(&path)?;
        let snapshot: serde_json::Value = serde_json::from_slice(&raw)?;
        Ok(snapshot
            .get("queue")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|queue| queue.is_empty()))
    })
    .await?;

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
    let outbox = BotOutbox::spawn(client, config)?;

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

    join_server(handle).await?;
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
    let _outbox = BotOutbox::spawn(client, config)?;

    wait_for_condition(Duration::from_secs(2), Duration::from_millis(20), || {
        if !queue_path.exists() || !dead_letter_path.exists() {
            return Ok(false);
        }

        let raw_queue = fs::read(&queue_path)?;
        let queue_snapshot: serde_json::Value = serde_json::from_slice(&raw_queue)?;
        let queue_empty = queue_snapshot
            .get("queue")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|queue| queue.is_empty());
        if !queue_empty {
            return Ok(false);
        }

        let raw_dead_letter = fs::read(&dead_letter_path)?;
        let dead_letter_snapshot: serde_json::Value = serde_json::from_slice(&raw_dead_letter)?;
        let entries = dead_letter_snapshot
            .get("entries")
            .and_then(serde_json::Value::as_array)
            .cloned()
            .unwrap_or_default();
        Ok(entries.len() == 1
            && entries[0]
                .get("reason")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|reason| reason.contains("expired")))
    })
    .await?;

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
async fn json_file_session_store_rejects_invalid_path_on_open() -> Result<(), DynError> {
    let (root, path) = blocked_child_path("tele-session-invalid-target", "session.json")?;

    let error = match JsonFileSessionStore::<String>::open(&path) {
        Ok(_) => return Err("expected JSON session store to reject invalid storage target".into()),
        Err(error) => error,
    };

    assert!(matches!(error, Error::Storage { .. }));
    assert!(error.to_string().contains("session store"));
    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[tokio::test]
async fn json_file_session_store_keeps_memory_unchanged_when_persist_fails() -> Result<(), DynError>
{
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let path = std::env::temp_dir().join(format!("tele-session-fail-{timestamp}.json"));

    let Some(update) = parse_update(message_update(4202, 77, "state")) else {
        return Ok(());
    };

    let store = JsonFileSessionStore::<String>::open(&path)?;
    save_chat_state(&store, &update, "step-1".to_owned()).await?;

    fs::remove_file(&path)?;
    fs::create_dir(&path)?;

    let result = save_chat_state(&store, &update, "step-2".to_owned()).await;
    assert!(result.is_err());

    let loaded = load_chat_state(&store, &update).await?;
    assert_eq!(loaded.as_deref(), Some("step-1"));

    let _ = fs::remove_dir_all(path);
    Ok(())
}

#[tokio::test]
async fn bot_engine_emits_events_and_testing_harness_dispatches() -> Result<(), DynError> {
    let client = Client::builder("http://127.0.0.1:9")?
        .bot_token("123:abc")?
        .build()?;

    let mut router = Router::new();
    router
        .message_route()
        .handle(|_context: BotContext, _update: Update| async move { Ok(()) });

    let harness = tele::bot::testing::BotHarness::new(router.clone())?;
    let fixture = tele::bot::testing::message_update(4301, 1, "hello")?;
    let outcome = harness.dispatch(fixture).await?;
    assert_eq!(outcome, DispatchOutcome::Handled { update_id: 4301 });

    let events = Arc::new(Mutex::new(Vec::<EngineEvent>::new()));
    let (sink, source) = channel_source(4)?;
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

#[tokio::test]
async fn testing_harness_uses_fresh_request_context_per_dispatch() -> Result<(), DynError> {
    #[derive(Clone, Debug)]
    struct HarnessMarker;

    let leaked = Arc::new(AtomicUsize::new(0));
    let hits = Arc::new(AtomicUsize::new(0));
    let mut router = Router::new();
    {
        let leaked = Arc::clone(&leaked);
        let hits = Arc::clone(&hits);
        router
            .message_route()
            .handle(move |context: BotContext, _update: Update| {
                let leaked = Arc::clone(&leaked);
                let hits = Arc::clone(&hits);
                async move {
                    if context.request_state().contains::<HarnessMarker>() {
                        leaked.fetch_add(1, Ordering::SeqCst);
                    }
                    let _ = context.request_state().insert(HarnessMarker);
                    hits.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                }
            });
    }

    let harness = tele::bot::testing::BotHarness::new(router)?;
    let first = tele::bot::testing::message_update(4303, 1, "hello")?;
    let second = tele::bot::testing::message_update(4304, 1, "world")?;
    assert_eq!(
        harness.dispatch(first).await?,
        DispatchOutcome::Handled { update_id: 4303 }
    );
    assert_eq!(
        harness.dispatch(second).await?,
        DispatchOutcome::Handled { update_id: 4304 }
    );
    assert_eq!(hits.load(Ordering::SeqCst), 2);
    assert_eq!(leaked.load(Ordering::SeqCst), 0);

    Ok(())
}

#[tokio::test]
async fn bot_engine_emits_unknown_kind_event() -> Result<(), DynError> {
    let client = Client::builder("http://127.0.0.1:9")?
        .bot_token("123:abc")?
        .build()?;

    let mut router = Router::new();
    router
        .message_route()
        .handle(|_context: BotContext, _update: Update| async move { Ok(()) });

    let events = Arc::new(Mutex::new(Vec::<EngineEvent>::new()));
    let (sink, source) = channel_source(2)?;
    let mut engine = BotEngine::new(client, source, router).on_event({
        let events = Arc::clone(&events);
        move |event| {
            if let Ok(mut guard) = events.lock() {
                guard.push(event.clone());
            }
        }
    });

    let maybe_update = parse_update(json!({
        "update_id": 4303,
        "message": {
            "message_id": 4303,
            "date": 1700004303,
            "chat": {"id": 1, "type": "private"},
            "gift": {"kind": "mystery"}
        }
    }));
    assert!(maybe_update.is_some());
    let Some(update) = maybe_update else {
        return Ok(());
    };

    sink.send(update).await?;
    let outcomes = engine.poll_once().await?;
    assert_eq!(outcomes, vec![DispatchOutcome::Handled { update_id: 4303 }]);

    let captured = events.lock().map_err(|error| error.to_string())?;
    assert!(captured.contains(&EngineEvent::UnknownKindsDetected {
        update_id: 4303,
        update_kind: UpdateKind::Message,
        message_kind: Some(MessageKind::Unknown),
    }));

    Ok(())
}

#[tokio::test]
async fn bot_engine_poll_failed_emits_details_and_async_hook() -> Result<(), DynError> {
    let client = Client::builder("http://127.0.0.1:9")?
        .bot_token("123:abc")?
        .request_timeout(Duration::from_millis(200))
        .total_timeout(Some(Duration::from_millis(500)))
        .build()?;

    let events = Arc::new(Mutex::new(Vec::<EngineEvent>::new()));
    let async_hits = Arc::new(AtomicUsize::new(0));
    let mut engine = BotEngine::with_long_polling(client, Router::new())
        .with_config(EngineConfig {
            continue_on_source_error: false,
            ..EngineConfig::default()
        })
        .on_event({
            let events = Arc::clone(&events);
            move |event| {
                if let Ok(mut guard) = events.lock() {
                    guard.push(event.clone());
                }
            }
        })
        .on_event_async({
            let async_hits = Arc::clone(&async_hits);
            move |_event| {
                let async_hits = Arc::clone(&async_hits);
                async move {
                    async_hits.fetch_add(1, Ordering::SeqCst);
                }
            }
        });

    let poll = engine.poll_once().await;
    assert!(poll.is_err());
    assert!(async_hits.load(Ordering::SeqCst) > 0);

    let captured = events.lock().map_err(|error| error.to_string())?;
    let poll_failed = captured
        .iter()
        .find(|event| matches!(event, EngineEvent::PollFailed { .. }));
    assert!(poll_failed.is_some());
    if let Some(EngineEvent::PollFailed {
        classification,
        request_id,
        message,
        ..
    }) = poll_failed
    {
        assert_eq!(*classification, ErrorClass::Transport);
        assert!(request_id.as_deref().is_some());
        assert!(!message.is_empty());
    }

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

    join_server(handle).await?;
    Ok(())
}

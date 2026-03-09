#![cfg(feature = "_async")]

use std::sync::{Arc, Mutex};
use std::time::Duration;

use tele::testing::{FakeTelegramServer, RequestExpectation};
use tele::types::advanced::{AdvancedAnswerWebAppQueryRequest, AdvancedGetAvailableGiftsRequest};
use tele::types::{
    AnswerInlineQueryRequest, BotCommand, ChatAdministratorCapability, CreateInvoiceLinkRequest,
    GetFileRequest, GetMyCommandsRequest, InlineKeyboardButton, InlineKeyboardMarkup,
    InlineQueryResult, InlineQueryResultsButton, LabeledPrice, MessageId, ParseMode,
    SendPhotoRequest, SendStickerRequest, SetMyCommandsRequest, Update, WebAppData,
};
use tele::{
    BanMemberOptions, BootstrapPlan, BootstrapRetryPolicy, BootstrapStepPhase, BootstrapStepStatus,
    Client, ClientMetric, Error, ErrorClass, MenuButtonConfig, RestrictMemberOptions, UploadFile,
};

#[cfg(feature = "bot")]
use tele::types::BotCommandScope;

type DynError = Box<dyn std::error::Error + Send + Sync>;
type TestServer = FakeTelegramServer;

fn spawn_server(
    expected_path: &'static str,
    response_status: u16,
    response_body: &'static str,
) -> Result<(String, TestServer), DynError> {
    let server = FakeTelegramServer::single(
        RequestExpectation::post(expected_path).respond_json(response_status, response_body),
    )?;
    Ok((server.base_url().to_owned(), server))
}

fn spawn_server_with_checks(
    expected_path: &'static str,
    response_status: u16,
    response_body: &'static str,
    required_substrings: &'static [&'static str],
) -> Result<(String, TestServer), DynError> {
    let mut expectation =
        RequestExpectation::post(expected_path).respond_json(response_status, response_body);
    for required in required_substrings {
        expectation = expectation.contains_case_insensitive(*required);
    }
    let server = FakeTelegramServer::single(expectation)?;
    Ok((server.base_url().to_owned(), server))
}

fn spawn_server_script(
    script: Vec<(&'static str, u16, &'static str)>,
) -> Result<(String, TestServer), DynError> {
    let expectations = script
        .into_iter()
        .map(|(expected_path, response_status, response_body)| {
            RequestExpectation::post(expected_path).respond_json(response_status, response_body)
        })
        .collect();
    let server = FakeTelegramServer::start(expectations)?;
    Ok((server.base_url().to_owned(), server))
}

fn join_server(server: TestServer) -> Result<(), DynError> {
    let _ = server.finish()?;
    Ok(())
}

#[tokio::test]
async fn client_metric_hook_records_method_latency() -> Result<(), DynError> {
    let response =
        r#"{"ok":true,"result":{"id":42,"is_bot":true,"first_name":"tele","username":"tele_bot"}}"#;
    let (base_url, handle) = spawn_server("/bot123:abc/getMe", 200, response)?;
    let metrics = Arc::new(Mutex::new(Vec::<ClientMetric>::new()));

    let client = Client::builder(base_url)?
        .bot_token("123:abc")?
        .on_metric({
            let metrics = Arc::clone(&metrics);
            move |metric| {
                if let Ok(mut captured) = metrics.lock() {
                    captured.push(metric.clone());
                }
            }
        })
        .build()?;

    let _ = client.bot().get_me().await?;
    join_server(handle)?;

    let captured = metrics.lock().map_err(|_| "client metric mutex poisoned")?;
    assert_eq!(captured.len(), 1);
    assert_eq!(captured[0].method, "getMe");
    assert!(captured[0].success);
    assert!(captured[0].latency >= Duration::ZERO);
    assert_eq!(captured[0].classification, None);

    Ok(())
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
async fn app_send_text_success() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":{"message_id":7,"date":1710000000,"chat":{"id":1,"type":"private"},"text":"hello"}}"#;
    let (base_url, handle) = spawn_server("/bot123:abc/sendMessage", 200, response)?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let sent = client.app().send_text(1_i64, "hello").await?;
    assert_eq!(sent.message_id.0, 7);

    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn app_text_builder_supports_markup_and_common_options() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":{"message_id":9,"date":1710000001,"chat":{"id":1,"type":"private"},"text":"hello builder"}}"#;
    let expectations = vec![
        RequestExpectation::post("/bot123:abc/sendMessage")
            .contains_case_insensitive("\"chat_id\":1")
            .contains_case_insensitive("\"text\":\"hello builder\"")
            .contains_case_insensitive("\"parse_mode\":\"MarkdownV2\"")
            .contains_case_insensitive("\"disable_notification\":true")
            .contains_case_insensitive("\"protect_content\":true")
            .contains_case_insensitive("\"message_thread_id\":99")
            .contains_case_insensitive("\"reply_parameters\":{\"message_id\":55")
            .contains_case_insensitive("\"link_preview_options\":{\"is_disabled\":true")
            .contains_case_insensitive("\"reply_markup\":{\"inline_keyboard\":[[{\"text\":\"Open\"")
            .contains_case_insensitive("\"callback_data\":\"open:1\"")
            .respond_json(200, response),
    ];
    let server = FakeTelegramServer::start(expectations)?;

    let client = Client::builder(server.base_url())?
        .bot_token("123:abc")?
        .build()?;
    let markup =
        InlineKeyboardMarkup::single_row(vec![InlineKeyboardButton::callback("Open", "open:1")?]);

    let sent = client
        .app()
        .text(1_i64, "hello builder")?
        .parse_mode(ParseMode::MarkdownV2)
        .reply_to_message(MessageId(55))
        .message_thread_id(99)
        .disable_notification(true)
        .protect_content(true)
        .disable_link_preview()
        .reply_markup(markup)
        .send()
        .await?;
    assert_eq!(sent.message_id.0, 9);

    join_server(server)?;
    Ok(())
}

#[tokio::test]
async fn app_media_builders_support_common_send_options() -> Result<(), DynError> {
    let expectations = vec![
        RequestExpectation::post("/bot123:abc/sendPhoto")
            .contains_case_insensitive("\"chat_id\":1")
            .contains_case_insensitive("\"photo\":\"photo-file-id\"")
            .contains_case_insensitive("\"caption\":\"photo caption\"")
            .contains_case_insensitive("\"parse_mode\":\"MarkdownV2\"")
            .contains_case_insensitive("\"has_spoiler\":true")
            .contains_case_insensitive("\"disable_notification\":true")
            .contains_case_insensitive("\"protect_content\":true")
            .contains_case_insensitive("\"message_thread_id\":11")
            .contains_case_insensitive("\"reply_parameters\":{\"message_id\":55")
            .contains_case_insensitive("\"reply_markup\":{\"inline_keyboard\":[[{\"text\":\"View photo\"")
            .contains_case_insensitive("\"callback_data\":\"photo:1\"")
            .respond_json(
                200,
                r#"{"ok":true,"result":{"message_id":10,"date":1710000002,"chat":{"id":1,"type":"private"}}}"#,
            ),
        RequestExpectation::post("/bot123:abc/sendDocument")
            .contains_case_insensitive("\"chat_id\":1")
            .contains_case_insensitive("\"document\":\"document-file-id\"")
            .contains_case_insensitive("\"thumbnail\":\"document-thumb-id\"")
            .contains_case_insensitive("\"caption\":\"document caption\"")
            .contains_case_insensitive("\"parse_mode\":\"MarkdownV2\"")
            .contains_case_insensitive("\"disable_content_type_detection\":true")
            .contains_case_insensitive("\"disable_notification\":true")
            .contains_case_insensitive("\"protect_content\":true")
            .contains_case_insensitive("\"message_thread_id\":12")
            .contains_case_insensitive("\"reply_parameters\":{\"message_id\":56")
            .contains_case_insensitive("\"reply_markup\":{\"inline_keyboard\":[[{\"text\":\"View document\"")
            .contains_case_insensitive("\"callback_data\":\"document:1\"")
            .respond_json(
                200,
                r#"{"ok":true,"result":{"message_id":11,"date":1710000003,"chat":{"id":1,"type":"private"}}}"#,
            ),
        RequestExpectation::post("/bot123:abc/sendVideo")
            .contains_case_insensitive("\"chat_id\":1")
            .contains_case_insensitive("\"video\":\"video-file-id\"")
            .contains_case_insensitive("\"duration\":30")
            .contains_case_insensitive("\"width\":1920")
            .contains_case_insensitive("\"height\":1080")
            .contains_case_insensitive("\"thumbnail\":\"video-thumb-id\"")
            .contains_case_insensitive("\"caption\":\"video caption\"")
            .contains_case_insensitive("\"parse_mode\":\"MarkdownV2\"")
            .contains_case_insensitive("\"supports_streaming\":true")
            .contains_case_insensitive("\"has_spoiler\":true")
            .contains_case_insensitive("\"disable_notification\":true")
            .contains_case_insensitive("\"protect_content\":true")
            .contains_case_insensitive("\"message_thread_id\":13")
            .contains_case_insensitive("\"reply_parameters\":{\"message_id\":57")
            .contains_case_insensitive("\"reply_markup\":{\"inline_keyboard\":[[{\"text\":\"View video\"")
            .contains_case_insensitive("\"callback_data\":\"video:1\"")
            .respond_json(
                200,
                r#"{"ok":true,"result":{"message_id":12,"date":1710000004,"chat":{"id":1,"type":"private"}}}"#,
            ),
    ];
    let server = FakeTelegramServer::start(expectations)?;

    let client = Client::builder(server.base_url())?
        .bot_token("123:abc")?
        .build()?;

    let photo_markup = InlineKeyboardMarkup::single_row(vec![InlineKeyboardButton::callback(
        "View photo",
        "photo:1",
    )?]);
    let photo = client
        .app()
        .photo(1_i64, "photo-file-id")
        .caption("photo caption")
        .parse_mode(ParseMode::MarkdownV2)
        .has_spoiler(true)
        .reply_to_message(MessageId(55))
        .message_thread_id(11)
        .disable_notification(true)
        .protect_content(true)
        .reply_markup(photo_markup)
        .send()
        .await?;
    assert_eq!(photo.message_id.0, 10);

    let document_markup = InlineKeyboardMarkup::single_row(vec![InlineKeyboardButton::callback(
        "View document",
        "document:1",
    )?]);
    let document = client
        .app()
        .document(1_i64, "document-file-id")
        .thumbnail("document-thumb-id")
        .caption("document caption")
        .parse_mode(ParseMode::MarkdownV2)
        .disable_content_type_detection(true)
        .reply_to_message(MessageId(56))
        .message_thread_id(12)
        .disable_notification(true)
        .protect_content(true)
        .reply_markup(document_markup)
        .send()
        .await?;
    assert_eq!(document.message_id.0, 11);

    let video_markup = InlineKeyboardMarkup::single_row(vec![InlineKeyboardButton::callback(
        "View video",
        "video:1",
    )?]);
    let video = client
        .app()
        .video(1_i64, "video-file-id")
        .duration(30)
        .width(1920)
        .height(1080)
        .thumbnail("video-thumb-id")
        .caption("video caption")
        .parse_mode(ParseMode::MarkdownV2)
        .supports_streaming(true)
        .has_spoiler(true)
        .reply_to_message(MessageId(57))
        .message_thread_id(13)
        .disable_notification(true)
        .protect_content(true)
        .reply_markup(video_markup)
        .send()
        .await?;
    assert_eq!(video.message_id.0, 12);

    join_server(server)?;
    Ok(())
}

#[tokio::test]
async fn app_reply_text_uses_join_request_user_chat_id() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":{"message_id":8,"date":1710000001,"chat":{"id":7001,"type":"private"},"text":"hello"}}"#;
    let (base_url, handle) = spawn_server_with_checks(
        "/bot123:abc/sendMessage",
        200,
        response,
        &[
            "\"chat_id\":7001",
            "\"text\":\"hello\"",
            "\"disable_notification\":true",
        ],
    )?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let update: Update = serde_json::from_value(serde_json::json!({
        "update_id": 43,
        "chat_join_request": {
            "chat": {"id": -10010, "type": "supergroup", "title": "mods"},
            "from": {"id": 701, "is_bot": false, "first_name": "candidate"},
            "user_chat_id": 7001,
            "date": 1710000001
        }
    }))?;

    let sent = client
        .app()
        .reply(&update, "hello")?
        .disable_notification(true)
        .send()
        .await?;
    assert_eq!(sent.message_id.0, 8);

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
    assert!(err.request_id().is_some());
    Ok(())
}

#[tokio::test]
async fn build_configuration_error_is_not_mapped_as_transport() -> Result<(), DynError> {
    let error = match Client::builder("https://api.telegram.org")?
        .bot_token("123:abc")?
        .no_proxy(["example.com", "[::1]not-a-port"])
        .build()
    {
        Ok(_) => return Err("expected build failure".into()),
        Err(error) => error,
    };

    assert!(matches!(error, Error::Configuration { .. }));
    assert_eq!(error.classification(), ErrorClass::Configuration);
    assert!(!error.is_retryable());
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
async fn bootstrap_skips_unchanged_commands_and_menu_button() -> Result<(), DynError> {
    let script = vec![
        (
            "/bot123:abc/getMyCommands",
            200,
            r#"{"ok":true,"result":[{"command":"start","description":"start the bot"}]}"#,
        ),
        (
            "/bot123:abc/getChatMenuButton",
            200,
            r#"{"ok":true,"result":{"type":"commands"}}"#,
        ),
    ];
    let (base_url, handle) = spawn_server_script(script)?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let plan = BootstrapPlan::new()
        .commands(vec![BotCommand::new("start", "start the bot")?])?
        .menu_button(MenuButtonConfig::commands());

    let outcome = client.control().setup().bootstrap(&plan).await;
    assert!(outcome.is_success());
    let Some(commands) = outcome.report.commands.as_ref() else {
        return Err("expected commands step report".into());
    };
    assert_eq!(commands.applied, Some(false));
    assert_eq!(commands.synced, Some(true));
    let Some(menu_button) = outcome.report.menu_button.as_ref() else {
        return Err("expected menu button step report".into());
    };
    assert_eq!(menu_button.applied, Some(false));
    assert_eq!(menu_button.synced, Some(true));

    join_server(handle)?;
    Ok(())
}

#[cfg(feature = "bot")]
#[derive(Clone, Debug)]
enum DemoCommand {
    Start,
}

#[cfg(feature = "bot")]
impl tele::bot::BotCommands for DemoCommand {
    fn parse(command: &str, _args: &str) -> Option<Self> {
        if command == "start" {
            Some(Self::Start)
        } else {
            None
        }
    }

    fn descriptions() -> &'static [tele::bot::CommandDescription] {
        &[tele::bot::CommandDescription {
            command: "start",
            description: "start command",
        }]
    }
}

#[cfg(feature = "bot")]
#[tokio::test]
async fn setup_set_typed_commands_with_scope_and_language() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":true}"#;
    const CHECKS: [&str; 4] = [
        "\"commands\":[{\"command\":\"start\",\"description\":\"start command\"}]",
        "\"scope\":{\"type\":\"all_private_chats\"}",
        "\"language_code\":\"zh-hans\"",
        "POST /bot123:abc/setMyCommands HTTP/1.1",
    ];
    let (base_url, handle) =
        spawn_server_with_checks("/bot123:abc/setMyCommands", 200, response, &CHECKS)?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let applied = client
        .control()
        .setup()
        .set_typed_commands_with_options::<DemoCommand>(
            Some(BotCommandScope::AllPrivateChats),
            Some("zh-hans".to_owned()),
        )
        .await?;
    assert!(applied);

    join_server(handle)?;
    Ok(())
}

#[cfg(feature = "bot")]
#[tokio::test]
async fn bootstrap_plan_typed_commands_with_scope_and_language() -> Result<(), DynError> {
    let expectations = vec![
        RequestExpectation::post("/bot123:abc/getMyCommands")
            .contains_case_insensitive("\"scope\":{\"type\":\"all_private_chats\"}")
            .contains_case_insensitive("\"language_code\":\"zh-hans\"")
            .respond_json(200, r#"{"ok":true,"result":[]}"#),
        RequestExpectation::post("/bot123:abc/setMyCommands")
            .contains_case_insensitive(
                "\"commands\":[{\"command\":\"start\",\"description\":\"start command\"}]",
            )
            .contains_case_insensitive("\"scope\":{\"type\":\"all_private_chats\"}")
            .contains_case_insensitive("\"language_code\":\"zh-hans\"")
            .respond_json(200, r#"{"ok":true,"result":true}"#),
    ];
    let server = FakeTelegramServer::start(expectations)?;

    let client = Client::builder(server.base_url())?
        .bot_token("123:abc")?
        .build()?;
    let plan = BootstrapPlan::new().typed_commands_with_options::<DemoCommand>(
        Some(BotCommandScope::AllPrivateChats),
        Some("zh-hans".to_owned()),
    )?;

    let outcome = client.control().setup().bootstrap(&plan).await;
    assert!(outcome.is_success());
    let Some(commands) = outcome.report.commands.as_ref() else {
        return Err("expected commands step report".into());
    };
    assert_eq!(commands.applied, Some(true));
    assert_eq!(commands.synced, Some(true));

    join_server(server)?;
    Ok(())
}

#[tokio::test]
async fn bootstrap_retry_can_continue_on_failure() -> Result<(), DynError> {
    let client = Client::builder("http://127.0.0.1:9")?
        .bot_token("123:abc")?
        .request_timeout(Duration::from_millis(100))
        .total_timeout(Some(Duration::from_millis(300)))
        .build()?;

    let plan = BootstrapPlan::new().commands(vec![BotCommand::new("start", "start bot")?])?;
    let outcome = client
        .control()
        .setup()
        .bootstrap_with_retry(
            &plan,
            BootstrapRetryPolicy {
                max_attempts: 1,
                continue_on_failure: true,
                ..BootstrapRetryPolicy::default()
            },
        )
        .await;
    assert!(outcome.is_success());
    let Some(commands) = outcome.report.commands.as_ref() else {
        return Err("expected commands step report".into());
    };
    assert_eq!(commands.applied, Some(false));
    assert_eq!(commands.diagnostics.status, BootstrapStepStatus::Warned);

    Ok(())
}

#[tokio::test]
async fn setup_bootstrap_warns_on_retryable_get_me_after_retries() -> Result<(), DynError> {
    let client = Client::builder("http://127.0.0.1:9")?
        .bot_token("123:abc")?
        .request_timeout(Duration::from_millis(40))
        .total_timeout(Some(Duration::from_millis(120)))
        .build()?;
    let plan = BootstrapPlan::new().warn_and_continue_on_retryable_get_me();
    let outcome = client
        .control()
        .setup()
        .bootstrap_with_retry(
            &plan,
            BootstrapRetryPolicy {
                max_attempts: 2,
                base_backoff: Duration::from_millis(1),
                max_backoff: Duration::from_millis(5),
                continue_on_failure: false,
                ..BootstrapRetryPolicy::default()
            },
        )
        .await;

    assert!(outcome.is_success());
    assert!(outcome.error.is_none());
    assert!(outcome.report.me.value.is_none());
    assert_eq!(
        outcome.report.me.diagnostics.status,
        BootstrapStepStatus::Warned
    );
    assert_eq!(
        outcome.report.me.diagnostics.phase,
        Some(BootstrapStepPhase::Fetch)
    );
    assert_eq!(
        outcome.report.me.diagnostics.classification,
        Some(ErrorClass::Transport)
    );
    assert!(outcome.report.me.diagnostics.retryable);
    assert_eq!(outcome.report.me.diagnostics.attempt_count, 2);
    assert!(outcome.report.me.diagnostics.request_id.is_some());

    Ok(())
}

#[tokio::test]
async fn setup_bootstrap_reports_unchanged_steps() -> Result<(), DynError> {
    let script = vec![
        (
            "/bot123:abc/getMyCommands",
            200,
            r#"{"ok":true,"result":[{"command":"start","description":"start the bot"}]}"#,
        ),
        (
            "/bot123:abc/getChatMenuButton",
            200,
            r#"{"ok":true,"result":{"type":"commands"}}"#,
        ),
    ];
    let (base_url, handle) = spawn_server_script(script)?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let plan = BootstrapPlan::new()
        .commands(vec![BotCommand::new("start", "start the bot")?])?
        .menu_button(MenuButtonConfig::commands());

    let outcome = client
        .control()
        .setup()
        .bootstrap_with_retry(&plan, BootstrapRetryPolicy::default())
        .await;

    assert!(outcome.is_success());
    let Some(commands) = outcome.report.commands.as_ref() else {
        return Err("expected commands step report".into());
    };
    assert_eq!(commands.applied, Some(false));
    assert_eq!(commands.synced, Some(true));
    assert_eq!(commands.diagnostics.status, BootstrapStepStatus::Unchanged);
    assert_eq!(commands.diagnostics.phase, Some(BootstrapStepPhase::Check));
    assert_eq!(commands.diagnostics.attempt_count, 1);

    let Some(menu_button) = outcome.report.menu_button.as_ref() else {
        return Err("expected menu button step report".into());
    };
    assert_eq!(menu_button.applied, Some(false));
    assert_eq!(menu_button.synced, Some(true));
    assert_eq!(
        menu_button.diagnostics.status,
        BootstrapStepStatus::Unchanged
    );
    assert_eq!(
        menu_button.diagnostics.phase,
        Some(BootstrapStepPhase::Check)
    );
    assert_eq!(menu_button.diagnostics.attempt_count, 1);

    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn web_app_answer_query_from_payload() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":{"inline_message_id":"inline-42"}}"#;
    const CHECKS: [&str; 3] = [
        "\"web_app_query_id\":\"query-42\"",
        "\"type\":\"article\"",
        "\"title\":\"From Payload\"",
    ];
    let (base_url, handle) =
        spawn_server_with_checks("/bot123:abc/answerWebAppQuery", 200, response, &CHECKS)?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let web_app_data = WebAppData::new("{\"query_id\":\"query-42\",\"item\":\"coffee\"}", "Open");
    let result = InlineQueryResult::article("r-42", "From Payload", "ok")?;
    let sent = client
        .app()
        .web_app()
        .answer_query_from_payload::<serde_json::Value, _>(&web_app_data, result)
        .await?;
    assert_eq!(sent.inline_message_id, "inline-42");

    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn setup_and_web_app_facades_handle_menu_button_and_query_answer() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":true}"#;
    let answer_response = r#"{"ok":true,"result":{"inline_message_id":"inline-99"}}"#;
    let expectations = vec![
        RequestExpectation::post("/bot123:abc/setChatMenuButton")
            .contains_case_insensitive("\"chat_id\":42")
            .contains_case_insensitive("\"menu_button\":{\"type\":\"web_app\"")
            .contains_case_insensitive("\"url\":\"https://example.com/mini-app\"")
            .respond_json(200, response),
        RequestExpectation::post("/bot123:abc/answerWebAppQuery")
            .contains_case_insensitive("\"web_app_query_id\":\"query-99\"")
            .contains_case_insensitive("\"title\":\"Facade Answer\"")
            .respond_json(200, answer_response),
    ];
    let server = FakeTelegramServer::start(expectations)?;

    let client = Client::builder(server.base_url())?
        .bot_token("123:abc")?
        .build()?;
    let applied = client
        .control()
        .setup()
        .set_menu_button(MenuButtonConfig::for_chat_web_app(
            42,
            "Open Mini App",
            "https://example.com/mini-app",
        ))
        .await?;
    assert!(applied);

    let web_app_data = WebAppData::new("{\"query_id\":\"query-99\",\"item\":\"tea\"}", "Open");
    let result = InlineQueryResult::article("article-99", "Facade Answer", "done")?;
    let sent = client
        .app()
        .web_app()
        .answer_query_from_payload::<serde_json::Value, _>(&web_app_data, result)
        .await?;
    assert_eq!(sent.inline_message_id, "inline-99");

    join_server(server)?;
    Ok(())
}

#[tokio::test]
async fn app_membership_facade_handles_bot_member_and_capabilities() -> Result<(), DynError> {
    let expectations = vec![
        RequestExpectation::post("/bot123:abc/getMe").respond_json(
            200,
            r#"{"ok":true,"result":{"id":999,"is_bot":true,"first_name":"tele","username":"tele_bot"}}"#,
        ),
        RequestExpectation::post("/bot123:abc/getChatMember")
            .contains_case_insensitive("\"chat_id\":-10010")
            .contains_case_insensitive("\"user_id\":999")
            .respond_json(
                200,
                r#"{"ok":true,"result":{"status":"administrator","user":{"id":999,"is_bot":true,"first_name":"tele"},"can_manage_chat":true,"can_delete_messages":true}}"#,
            ),
        RequestExpectation::post("/bot123:abc/getMe").respond_json(
            200,
            r#"{"ok":true,"result":{"id":999,"is_bot":true,"first_name":"tele","username":"tele_bot"}}"#,
        ),
        RequestExpectation::post("/bot123:abc/getChatMember")
            .contains_case_insensitive("\"chat_id\":-10010")
            .contains_case_insensitive("\"user_id\":999")
            .respond_json(
                200,
                r#"{"ok":true,"result":{"status":"administrator","user":{"id":999,"is_bot":true,"first_name":"tele"},"can_manage_chat":true,"can_delete_messages":true}}"#,
            ),
        RequestExpectation::post("/bot123:abc/getChatAdministrators")
            .contains_case_insensitive("\"chat_id\":-10010")
            .respond_json(
                200,
                r#"{"ok":true,"result":[{"status":"administrator","user":{"id":999,"is_bot":true,"first_name":"tele"},"can_manage_chat":true,"can_delete_messages":true},{"status":"administrator","user":{"id":701,"is_bot":false,"first_name":"owner"},"can_manage_chat":true,"can_restrict_members":true}]}"#,
            ),
    ];
    let server = FakeTelegramServer::start(expectations)?;

    let client = Client::builder(server.base_url())?
        .bot_token("123:abc")?
        .build()?;
    let membership = client.app().membership();

    let bot_member = membership.bot_member(-10010_i64).await?;
    assert_eq!(bot_member.user().id.0, 999);
    assert!(bot_member.has_capability(ChatAdministratorCapability::ManageChat));

    let missing = membership
        .bot_missing_capabilities(
            -10010_i64,
            &[
                ChatAdministratorCapability::ManageChat,
                ChatAdministratorCapability::RestrictMembers,
            ],
        )
        .await?;
    assert_eq!(missing, vec![ChatAdministratorCapability::RestrictMembers]);

    let administrators = membership.administrators(-10010_i64).await?;
    assert_eq!(administrators.len(), 2);

    join_server(server)?;
    Ok(())
}

#[tokio::test]
async fn moderation_facade_handles_join_actions_and_member_controls() -> Result<(), DynError> {
    let expectations = vec![
        RequestExpectation::post("/bot123:abc/approveChatJoinRequest")
            .contains_case_insensitive("\"chat_id\":-10010")
            .contains_case_insensitive("\"user_id\":701")
            .respond_json(200, r#"{"ok":true,"result":true}"#),
        RequestExpectation::post("/bot123:abc/declineChatJoinRequest")
            .contains_case_insensitive("\"chat_id\":-10010")
            .contains_case_insensitive("\"user_id\":701")
            .respond_json(200, r#"{"ok":true,"result":true}"#),
        RequestExpectation::post("/bot123:abc/banChatMember")
            .contains_case_insensitive("\"chat_id\":-10010")
            .contains_case_insensitive("\"user_id\":701")
            .contains_case_insensitive("\"until_date\":1710009999")
            .contains_case_insensitive("\"revoke_messages\":true")
            .respond_json(200, r#"{"ok":true,"result":true}"#),
        RequestExpectation::post("/bot123:abc/restrictChatMember")
            .contains_case_insensitive("\"chat_id\":-10010")
            .contains_case_insensitive("\"user_id\":701")
            .contains_case_insensitive("\"can_send_messages\":false")
            .contains_case_insensitive("\"can_manage_topics\":false")
            .contains_case_insensitive("\"use_independent_chat_permissions\":true")
            .contains_case_insensitive("\"until_date\":1710011111")
            .respond_json(200, r#"{"ok":true,"result":true}"#),
        RequestExpectation::post("/bot123:abc/deleteMessage")
            .contains_case_insensitive("\"chat_id\":-10010")
            .contains_case_insensitive("\"message_id\":55")
            .respond_json(200, r#"{"ok":true,"result":true}"#),
    ];
    let server = FakeTelegramServer::start(expectations)?;

    let client = Client::builder(server.base_url())?
        .bot_token("123:abc")?
        .build()?;
    let join_update: Update = serde_json::from_value(serde_json::json!({
        "update_id": 43,
        "chat_join_request": {
            "chat": {"id": -10010, "type": "supergroup", "title": "mods"},
            "from": {"id": 701, "is_bot": false, "first_name": "candidate"},
            "user_chat_id": 7001,
            "date": 1710000001
        }
    }))?;
    let message_update: Update = serde_json::from_value(serde_json::json!({
        "update_id": 44,
        "message": {
            "message_id": 55,
            "date": 1710000002,
            "chat": {"id": -10010, "type": "supergroup", "title": "mods"},
            "from": {"id": 701, "is_bot": false, "first_name": "candidate"},
            "text": "spam"
        }
    }))?;
    let message = message_update
        .message
        .as_deref()
        .ok_or("missing test message")?;

    assert!(
        client
            .app()
            .moderation()
            .approve_join_request_from_update(&join_update)
            .await?
    );
    assert!(
        client
            .app()
            .moderation()
            .decline_join_request_from_update(&join_update)
            .await?
    );
    assert!(
        client
            .app()
            .moderation()
            .ban_author_with(
                message,
                BanMemberOptions::new()
                    .until_date(1710009999)
                    .revoke_messages(true),
            )
            .await?
    );
    assert!(
        client
            .app()
            .moderation()
            .mute_author_with(
                message,
                RestrictMemberOptions::new()
                    .use_independent_chat_permissions(true)
                    .until_date(1710011111),
            )
            .await?
    );
    assert!(
        client
            .app()
            .moderation()
            .delete_from_update(&message_update)
            .await?
    );

    join_server(server)?;
    Ok(())
}

#[tokio::test]
async fn moderation_notice_facade_reuses_text_builder() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":{"message_id":56,"date":1710000003,"chat":{"id":-10010,"type":"supergroup","title":"mods"},"message_thread_id":88,"text":"Message removed"}}"#;
    let expectations = vec![
        RequestExpectation::post("/bot123:abc/sendMessage")
            .contains_case_insensitive("\"chat_id\":-10010")
            .contains_case_insensitive("\"text\":\"Message removed\"")
            .contains_case_insensitive("\"reply_parameters\":{\"message_id\":55")
            .contains_case_insensitive("\"message_thread_id\":88")
            .contains_case_insensitive("\"disable_notification\":true")
            .contains_case_insensitive(
                "\"reply_markup\":{\"inline_keyboard\":[[{\"text\":\"Review\"",
            )
            .contains_case_insensitive("\"callback_data\":\"review:55\"")
            .respond_json(200, response),
    ];
    let server = FakeTelegramServer::start(expectations)?;

    let client = Client::builder(server.base_url())?
        .bot_token("123:abc")?
        .build()?;
    let update: Update = serde_json::from_value(serde_json::json!({
        "update_id": 45,
        "message": {
            "message_id": 55,
            "message_thread_id": 88,
            "date": 1710000002,
            "chat": {"id": -10010, "type": "supergroup", "title": "mods"},
            "from": {"id": 701, "is_bot": false, "first_name": "candidate"},
            "text": "spam"
        }
    }))?;
    let message = update.message.as_deref().ok_or("missing test message")?;
    let markup = InlineKeyboardMarkup::single_row(vec![InlineKeyboardButton::callback(
        "Review",
        "review:55",
    )?]);

    let sent = client
        .app()
        .moderation()
        .notice()
        .for_message(message, "Message removed")?
        .disable_notification(true)
        .reply_markup(markup)
        .send()
        .await?;
    assert_eq!(sent.message_id.0, 56);

    join_server(server)?;
    Ok(())
}

#[tokio::test]
async fn setup_set_chat_menu_button_uses_high_level_helper() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":true}"#;
    const CHECKS: [&str; 4] = [
        "\"chat_id\":42",
        "\"menu_button\":{\"type\":\"web_app\"",
        "\"text\":\"Open Mini App\"",
        "\"url\":\"https://example.com/mini-app\"",
    ];
    let (base_url, handle) =
        spawn_server_with_checks("/bot123:abc/setChatMenuButton", 200, response, &CHECKS)?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let applied = client
        .control()
        .setup()
        .set_menu_button(MenuButtonConfig::for_chat_web_app(
            42,
            "Open Mini App",
            "https://example.com/mini-app",
        ))
        .await?;
    assert!(applied);

    join_server(handle)?;
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
async fn app_photo_builder_send_upload_success() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":{"message_id":102,"date":1710000005,"chat":{"id":1,"type":"private"},"photo":[{"file_id":"file_2","file_unique_id":"uniq_2","width":10,"height":10}]}}"#;
    const CHECKS: [&str; 5] = [
        "Content-Type: multipart/form-data; boundary=",
        "name=\"chat_id\"",
        "name=\"caption\"",
        "name=\"photo\"; filename=\"builder-image.jpg\"",
        "binary-builder-photo-data",
    ];
    let (base_url, handle) =
        spawn_server_with_checks("/bot123:abc/sendPhoto", 200, response, &CHECKS)?;

    let client = Client::builder(base_url)?.bot_token("123:abc")?.build()?;
    let file = UploadFile::from_bytes("builder-image.jpg", b"binary-builder-photo-data".to_vec())?;
    let message = client
        .app()
        .photo(1_i64, "ignored-in-multipart")
        .caption("builder upload")
        .send_upload(&file)
        .await?;
    assert_eq!(message.message_id.0, 102);

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
    let inline_result =
        InlineQueryResult::article("result-inline-1", "Inline title", "Inline message text")?;
    let request = AnswerInlineQueryRequest::new("inline-q-1", vec![inline_result]).button(
        InlineQueryResultsButton::web_app("Open Mini App", "https://example.com/mini-app"),
    );
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

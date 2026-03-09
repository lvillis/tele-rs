#![cfg(feature = "_blocking")]

use std::time::Duration;

use tele::testing::{FakeTelegramServer, RequestExpectation};
use tele::types::advanced::AdvancedGetAvailableGiftsRequest;
use tele::types::{
    CreateInvoiceLinkRequest, GetChatMemberCountRequest, InlineKeyboardButton,
    InlineKeyboardMarkup, LabeledPrice, MessageId, ParseMode, WebAppData,
};
use tele::{
    BanMemberOptions, BlockingClient, Error, ErrorClass, MenuButtonConfig, RestrictMemberOptions,
};

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

fn join_server(server: TestServer) -> Result<(), DynError> {
    let _ = server.finish()?;
    Ok(())
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
async fn blocking_setup_and_web_app_facades_handle_menu_button_and_query_answer()
-> Result<(), DynError> {
    let expectations = vec![
        RequestExpectation::post("/bot123:abc/setChatMenuButton")
            .contains_case_insensitive("\"chat_id\":42")
            .contains_case_insensitive("\"menu_button\":{\"type\":\"web_app\"")
            .contains_case_insensitive("\"url\":\"https://example.com/mini-app\"")
            .respond_json(200, r#"{"ok":true,"result":true}"#),
        RequestExpectation::post("/bot123:abc/answerWebAppQuery")
            .contains_case_insensitive("\"web_app_query_id\":\"query-77\"")
            .contains_case_insensitive("\"title\":\"Blocking Facade\"")
            .respond_json(
                200,
                r#"{"ok":true,"result":{"inline_message_id":"inline-blocking-77"}}"#,
            ),
    ];
    let server = FakeTelegramServer::start(expectations)?;

    let client = BlockingClient::builder(server.base_url())?
        .bot_token("123:abc")?
        .build_blocking()?;
    let applied = client
        .control()
        .setup()
        .set_menu_button(MenuButtonConfig::for_chat_web_app(
            42,
            "Open Mini App",
            "https://example.com/mini-app",
        ))?;
    assert!(applied);

    let web_app_data = WebAppData::new("{\"query_id\":\"query-77\",\"item\":\"tea\"}", "Open");
    let result = tele::types::InlineQueryResult::article("blocking-77", "Blocking Facade", "ok")?;
    let sent = client
        .app()
        .web_app()
        .answer_query_from_payload::<serde_json::Value, _>(&web_app_data, result)?;
    assert_eq!(sent.inline_message_id, "inline-blocking-77");

    join_server(server)?;
    Ok(())
}

#[tokio::test]
async fn blocking_moderation_facade_handles_join_actions_and_member_controls()
-> Result<(), DynError> {
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

    let client = BlockingClient::builder(server.base_url())?
        .bot_token("123:abc")?
        .build_blocking()?;
    let join_update: tele::types::Update = serde_json::from_value(serde_json::json!({
        "update_id": 43,
        "chat_join_request": {
            "chat": {"id": -10010, "type": "supergroup", "title": "mods"},
            "from": {"id": 701, "is_bot": false, "first_name": "candidate"},
            "user_chat_id": 7001,
            "date": 1710000001
        }
    }))?;
    let message_update: tele::types::Update = serde_json::from_value(serde_json::json!({
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
            .approve_join_request_from_update(&join_update)?
    );
    assert!(
        client
            .app()
            .moderation()
            .decline_join_request_from_update(&join_update)?
    );
    assert!(
        client.app().moderation().ban_author_with(
            message,
            BanMemberOptions::new()
                .until_date(1710009999)
                .revoke_messages(true),
        )?
    );
    assert!(
        client.app().moderation().mute_author_with(
            message,
            RestrictMemberOptions::new()
                .use_independent_chat_permissions(true)
                .until_date(1710011111),
        )?
    );
    assert!(
        client
            .app()
            .moderation()
            .delete_from_update(&message_update)?
    );

    join_server(server)?;
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
async fn blocking_app_send_text_success() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":{"message_id":11,"date":1710000000,"chat":{"id":1,"type":"private"},"text":"hello"}}"#;
    let (base_url, handle) = spawn_server("/bot123:abc/sendMessage", 200, response)?;

    let client = BlockingClient::builder(base_url)?
        .bot_token("123:abc")?
        .build_blocking()?;

    let sent = client.app().send_text(1_i64, "hello")?;
    assert_eq!(sent.message_id.0, 11);

    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn blocking_text_builder_supports_markup_and_common_options() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":{"message_id":12,"date":1710000001,"chat":{"id":1,"type":"private"},"text":"hello builder"}}"#;
    let (base_url, handle) = spawn_server_with_checks(
        "/bot123:abc/sendMessage",
        200,
        response,
        &[
            "\"chat_id\":1",
            "\"text\":\"hello builder\"",
            "\"parse_mode\":\"MarkdownV2\"",
            "\"disable_notification\":true",
            "\"protect_content\":true",
            "\"message_thread_id\":99",
            "\"reply_parameters\":{\"message_id\":55",
            "\"link_preview_options\":{\"is_disabled\":true",
            "\"reply_markup\":{\"inline_keyboard\":[[{\"text\":\"Open\"",
            "\"callback_data\":\"open:1\"",
        ],
    )?;

    let client = BlockingClient::builder(base_url)?
        .bot_token("123:abc")?
        .build_blocking()?;
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
        .send()?;
    assert_eq!(sent.message_id.0, 12);

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

#[tokio::test]
async fn blocking_moderation_notice_facade_reuses_text_builder() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":{"message_id":56,"date":1710000003,"chat":{"id":-10010,"type":"supergroup","title":"mods"},"message_thread_id":88,"text":"Message removed"}}"#;
    let (base_url, handle) = spawn_server_with_checks(
        "/bot123:abc/sendMessage",
        200,
        response,
        &[
            "\"chat_id\":-10010",
            "\"text\":\"Message removed\"",
            "\"reply_parameters\":{\"message_id\":55",
            "\"message_thread_id\":88",
            "\"disable_notification\":true",
            "\"reply_markup\":{\"inline_keyboard\":[[{\"text\":\"Review\"",
            "\"callback_data\":\"review:55\"",
        ],
    )?;

    let client = BlockingClient::builder(base_url)?
        .bot_token("123:abc")?
        .build_blocking()?;
    let update: tele::types::Update = serde_json::from_value(serde_json::json!({
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
        .send()?;
    assert_eq!(sent.message_id.0, 56);

    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn blocking_transport_error_has_request_id() -> Result<(), DynError> {
    let client = BlockingClient::builder("http://127.0.0.1:9")?
        .bot_token("123:abc")?
        .request_timeout(Duration::from_millis(100))
        .total_timeout(Some(Duration::from_millis(300)))
        .build_blocking()?;

    let error = match client.bot().get_me() {
        Ok(_) => return Err("expected transport error".into()),
        Err(error) => error,
    };

    assert!(matches!(error, Error::Transport { .. }));
    assert!(error.request_id().is_some());
    assert!(!error.to_string().contains("123:abc"));
    Ok(())
}

#[tokio::test]
async fn blocking_build_configuration_error_is_not_mapped_as_transport() -> Result<(), DynError> {
    let error = match BlockingClient::builder("https://api.telegram.org")?
        .bot_token("123:abc")?
        .no_proxy(["example.com", "[::1]not-a-port"])
        .build_blocking()
    {
        Ok(_) => return Err("expected build failure".into()),
        Err(error) => error,
    };

    assert!(matches!(error, Error::Configuration { .. }));
    assert_eq!(error.classification(), ErrorClass::Configuration);
    assert!(!error.is_retryable());
    Ok(())
}

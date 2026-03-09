#![cfg(feature = "_blocking")]

use std::time::Duration;

use tele::testing::{FakeTelegramServer, RequestExpectation};
use tele::types::advanced::AdvancedGetAvailableGiftsRequest;
use tele::types::{
    ChatAdministratorCapability, CreateInvoiceLinkRequest, GetChatMemberCountRequest,
    InlineKeyboardButton, InlineKeyboardMarkup, InputMedia, LabeledPrice, MessageId, ParseMode,
    WebAppData,
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
async fn blocking_callback_answer_builder_supports_common_options() -> Result<(), DynError> {
    let response = r#"{"ok":true,"result":true}"#;
    const CHECKS: [&str; 5] = [
        "\"callback_query_id\":\"blocking-callback-42\"",
        "\"text\":\"Updated\"",
        "\"show_alert\":true",
        "\"url\":\"https://example.com/blocking-callback\"",
        "\"cache_time\":45",
    ];
    let (base_url, handle) =
        spawn_server_with_checks("/bot123:abc/answerCallbackQuery", 200, response, &CHECKS)?;

    let client = BlockingClient::builder(base_url)?
        .bot_token("123:abc")?
        .build_blocking()?;
    let ok = client
        .app()
        .callback_answer("blocking-callback-42")
        .text("Updated")
        .show_alert(true)
        .url("https://example.com/blocking-callback")
        .cache_time(45)
        .send()?;
    assert!(ok);

    join_server(handle)?;
    Ok(())
}

#[tokio::test]
async fn blocking_media_builders_support_common_send_options() -> Result<(), DynError> {
    let expectations = vec![
        RequestExpectation::post("/bot123:abc/sendPhoto")
            .contains_case_insensitive("\"chat_id\":1")
            .contains_case_insensitive("\"photo\":\"blocking-photo-file-id\"")
            .contains_case_insensitive("\"caption\":\"blocking photo caption\"")
            .contains_case_insensitive("\"parse_mode\":\"MarkdownV2\"")
            .contains_case_insensitive("\"has_spoiler\":true")
            .contains_case_insensitive("\"disable_notification\":true")
            .contains_case_insensitive("\"protect_content\":true")
            .contains_case_insensitive("\"message_thread_id\":21")
            .contains_case_insensitive("\"reply_parameters\":{\"message_id\":65")
            .contains_case_insensitive("\"reply_markup\":{\"inline_keyboard\":[[{\"text\":\"View blocking photo\"")
            .contains_case_insensitive("\"callback_data\":\"blocking-photo:1\"")
            .respond_json(
                200,
                r#"{"ok":true,"result":{"message_id":20,"date":1710000010,"chat":{"id":1,"type":"private"}}}"#,
            ),
        RequestExpectation::post("/bot123:abc/sendDocument")
            .contains_case_insensitive("\"chat_id\":1")
            .contains_case_insensitive("\"document\":\"blocking-document-file-id\"")
            .contains_case_insensitive("\"thumbnail\":\"blocking-document-thumb-id\"")
            .contains_case_insensitive("\"caption\":\"blocking document caption\"")
            .contains_case_insensitive("\"parse_mode\":\"MarkdownV2\"")
            .contains_case_insensitive("\"disable_content_type_detection\":true")
            .contains_case_insensitive("\"disable_notification\":true")
            .contains_case_insensitive("\"protect_content\":true")
            .contains_case_insensitive("\"message_thread_id\":22")
            .contains_case_insensitive("\"reply_parameters\":{\"message_id\":66")
            .contains_case_insensitive("\"reply_markup\":{\"inline_keyboard\":[[{\"text\":\"View blocking document\"")
            .contains_case_insensitive("\"callback_data\":\"blocking-document:1\"")
            .respond_json(
                200,
                r#"{"ok":true,"result":{"message_id":21,"date":1710000011,"chat":{"id":1,"type":"private"}}}"#,
            ),
        RequestExpectation::post("/bot123:abc/sendVideo")
            .contains_case_insensitive("\"chat_id\":1")
            .contains_case_insensitive("\"video\":\"blocking-video-file-id\"")
            .contains_case_insensitive("\"duration\":45")
            .contains_case_insensitive("\"width\":1280")
            .contains_case_insensitive("\"height\":720")
            .contains_case_insensitive("\"thumbnail\":\"blocking-video-thumb-id\"")
            .contains_case_insensitive("\"caption\":\"blocking video caption\"")
            .contains_case_insensitive("\"parse_mode\":\"MarkdownV2\"")
            .contains_case_insensitive("\"supports_streaming\":true")
            .contains_case_insensitive("\"has_spoiler\":true")
            .contains_case_insensitive("\"disable_notification\":true")
            .contains_case_insensitive("\"protect_content\":true")
            .contains_case_insensitive("\"message_thread_id\":23")
            .contains_case_insensitive("\"reply_parameters\":{\"message_id\":67")
            .contains_case_insensitive("\"reply_markup\":{\"inline_keyboard\":[[{\"text\":\"View blocking video\"")
            .contains_case_insensitive("\"callback_data\":\"blocking-video:1\"")
            .respond_json(
                200,
                r#"{"ok":true,"result":{"message_id":22,"date":1710000012,"chat":{"id":1,"type":"private"}}}"#,
            ),
    ];
    let server = FakeTelegramServer::start(expectations)?;

    let client = BlockingClient::builder(server.base_url())?
        .bot_token("123:abc")?
        .build_blocking()?;

    let photo_markup = InlineKeyboardMarkup::single_row(vec![InlineKeyboardButton::callback(
        "View blocking photo",
        "blocking-photo:1",
    )?]);
    let photo = client
        .app()
        .photo(1_i64, "blocking-photo-file-id")
        .caption("blocking photo caption")
        .parse_mode(ParseMode::MarkdownV2)
        .has_spoiler(true)
        .reply_to_message(MessageId(65))
        .message_thread_id(21)
        .disable_notification(true)
        .protect_content(true)
        .reply_markup(photo_markup)
        .send()?;
    assert_eq!(photo.message_id.0, 20);

    let document_markup = InlineKeyboardMarkup::single_row(vec![InlineKeyboardButton::callback(
        "View blocking document",
        "blocking-document:1",
    )?]);
    let document = client
        .app()
        .document(1_i64, "blocking-document-file-id")
        .thumbnail("blocking-document-thumb-id")
        .caption("blocking document caption")
        .parse_mode(ParseMode::MarkdownV2)
        .disable_content_type_detection(true)
        .reply_to_message(MessageId(66))
        .message_thread_id(22)
        .disable_notification(true)
        .protect_content(true)
        .reply_markup(document_markup)
        .send()?;
    assert_eq!(document.message_id.0, 21);

    let video_markup = InlineKeyboardMarkup::single_row(vec![InlineKeyboardButton::callback(
        "View blocking video",
        "blocking-video:1",
    )?]);
    let video = client
        .app()
        .video(1_i64, "blocking-video-file-id")
        .duration(45)
        .width(1280)
        .height(720)
        .thumbnail("blocking-video-thumb-id")
        .caption("blocking video caption")
        .parse_mode(ParseMode::MarkdownV2)
        .supports_streaming(true)
        .has_spoiler(true)
        .reply_to_message(MessageId(67))
        .message_thread_id(23)
        .disable_notification(true)
        .protect_content(true)
        .reply_markup(video_markup)
        .send()?;
    assert_eq!(video.message_id.0, 22);

    join_server(server)?;
    Ok(())
}

#[tokio::test]
async fn blocking_richer_media_builders_support_common_send_options() -> Result<(), DynError> {
    let expectations = vec![
        RequestExpectation::post("/bot123:abc/sendAudio")
            .contains_case_insensitive("\"chat_id\":1")
            .contains_case_insensitive("\"audio\":\"blocking-audio-file-id\"")
            .contains_case_insensitive("\"caption\":\"blocking audio caption\"")
            .contains_case_insensitive("\"parse_mode\":\"MarkdownV2\"")
            .contains_case_insensitive("\"duration\":150")
            .contains_case_insensitive("\"performer\":\"blocking band\"")
            .contains_case_insensitive("\"title\":\"blocking song\"")
            .contains_case_insensitive("\"thumbnail\":\"blocking-audio-thumb-id\"")
            .contains_case_insensitive("\"message_thread_id\":24")
            .contains_case_insensitive("\"reply_parameters\":{\"message_id\":68")
            .contains_case_insensitive("\"reply_markup\":{\"inline_keyboard\":[[{\"text\":\"Play blocking audio\"")
            .contains_case_insensitive("\"callback_data\":\"blocking-audio:1\"")
            .respond_json(
                200,
                r#"{"ok":true,"result":{"message_id":23,"date":1710000013,"chat":{"id":1,"type":"private"}}}"#,
            ),
        RequestExpectation::post("/bot123:abc/sendAnimation")
            .contains_case_insensitive("\"chat_id\":1")
            .contains_case_insensitive("\"animation\":\"blocking-animation-file-id\"")
            .contains_case_insensitive("\"caption\":\"blocking animation caption\"")
            .contains_case_insensitive("\"parse_mode\":\"MarkdownV2\"")
            .contains_case_insensitive("\"duration\":9")
            .contains_case_insensitive("\"width\":640")
            .contains_case_insensitive("\"height\":360")
            .contains_case_insensitive("\"thumbnail\":\"blocking-animation-thumb-id\"")
            .contains_case_insensitive("\"has_spoiler\":true")
            .contains_case_insensitive("\"message_thread_id\":25")
            .contains_case_insensitive("\"reply_parameters\":{\"message_id\":69")
            .contains_case_insensitive("\"reply_markup\":{\"inline_keyboard\":[[{\"text\":\"Play blocking animation\"")
            .contains_case_insensitive("\"callback_data\":\"blocking-animation:1\"")
            .respond_json(
                200,
                r#"{"ok":true,"result":{"message_id":24,"date":1710000014,"chat":{"id":1,"type":"private"}}}"#,
            ),
        RequestExpectation::post("/bot123:abc/sendVoice")
            .contains_case_insensitive("\"chat_id\":1")
            .contains_case_insensitive("\"voice\":\"blocking-voice-file-id\"")
            .contains_case_insensitive("\"caption\":\"blocking voice caption\"")
            .contains_case_insensitive("\"parse_mode\":\"MarkdownV2\"")
            .contains_case_insensitive("\"duration\":31")
            .contains_case_insensitive("\"message_thread_id\":26")
            .contains_case_insensitive("\"reply_parameters\":{\"message_id\":70")
            .contains_case_insensitive("\"reply_markup\":{\"inline_keyboard\":[[{\"text\":\"Play blocking voice\"")
            .contains_case_insensitive("\"callback_data\":\"blocking-voice:1\"")
            .respond_json(
                200,
                r#"{"ok":true,"result":{"message_id":25,"date":1710000015,"chat":{"id":1,"type":"private"}}}"#,
            ),
        RequestExpectation::post("/bot123:abc/sendSticker")
            .contains_case_insensitive("\"chat_id\":1")
            .contains_case_insensitive("\"sticker\":\"blocking-sticker-file-id\"")
            .contains_case_insensitive("\"emoji\":\":ok:\"")
            .contains_case_insensitive("\"message_thread_id\":27")
            .respond_json(
                200,
                r#"{"ok":true,"result":{"message_id":26,"date":1710000016,"chat":{"id":1,"type":"private"}}}"#,
            ),
        RequestExpectation::post("/bot123:abc/sendMediaGroup")
            .contains_case_insensitive("\"chat_id\":1")
            .contains_case_insensitive("\"media\":[{\"type\":\"photo\",\"media\":\"blocking-group-photo-file-id\",\"caption\":\"blocking group photo caption\"")
            .contains_case_insensitive("\"type\":\"video\",\"media\":\"blocking-group-video-file-id\",\"caption\":\"blocking group video caption\"")
            .contains_case_insensitive("\"supports_streaming\":true")
            .contains_case_insensitive("\"message_thread_id\":28")
            .contains_case_insensitive("\"reply_parameters\":{\"message_id\":72")
            .respond_json(
                200,
                r#"{"ok":true,"result":[{"message_id":27,"date":1710000017,"chat":{"id":1,"type":"private"}},{"message_id":28,"date":1710000018,"chat":{"id":1,"type":"private"}}]}"#,
            ),
    ];
    let server = FakeTelegramServer::start(expectations)?;

    let client = BlockingClient::builder(server.base_url())?
        .bot_token("123:abc")?
        .build_blocking()?;

    let audio_markup = InlineKeyboardMarkup::single_row(vec![InlineKeyboardButton::callback(
        "Play blocking audio",
        "blocking-audio:1",
    )?]);
    let audio = client
        .app()
        .audio(1_i64, "blocking-audio-file-id")
        .caption("blocking audio caption")
        .parse_mode(ParseMode::MarkdownV2)
        .duration(150)
        .performer("blocking band")
        .title("blocking song")
        .thumbnail("blocking-audio-thumb-id")
        .reply_to_message(MessageId(68))
        .message_thread_id(24)
        .reply_markup(audio_markup)
        .send()?;
    assert_eq!(audio.message_id.0, 23);

    let animation_markup = InlineKeyboardMarkup::single_row(vec![InlineKeyboardButton::callback(
        "Play blocking animation",
        "blocking-animation:1",
    )?]);
    let animation = client
        .app()
        .animation(1_i64, "blocking-animation-file-id")
        .caption("blocking animation caption")
        .parse_mode(ParseMode::MarkdownV2)
        .duration(9)
        .width(640)
        .height(360)
        .thumbnail("blocking-animation-thumb-id")
        .has_spoiler(true)
        .reply_to_message(MessageId(69))
        .message_thread_id(25)
        .reply_markup(animation_markup)
        .send()?;
    assert_eq!(animation.message_id.0, 24);

    let voice_markup = InlineKeyboardMarkup::single_row(vec![InlineKeyboardButton::callback(
        "Play blocking voice",
        "blocking-voice:1",
    )?]);
    let voice = client
        .app()
        .voice(1_i64, "blocking-voice-file-id")
        .caption("blocking voice caption")
        .parse_mode(ParseMode::MarkdownV2)
        .duration(31)
        .reply_to_message(MessageId(70))
        .message_thread_id(26)
        .reply_markup(voice_markup)
        .send()?;
    assert_eq!(voice.message_id.0, 25);

    let sticker_markup = InlineKeyboardMarkup::single_row(vec![InlineKeyboardButton::callback(
        "Review blocking sticker",
        "blocking-sticker:1",
    )?]);
    let sticker = client
        .app()
        .sticker(1_i64, "blocking-sticker-file-id")
        .emoji(":ok:")
        .reply_to_message(MessageId(71))?
        .message_thread_id(27)
        .reply_markup(sticker_markup)?
        .send()?;
    assert_eq!(sticker.message_id.0, 26);

    let group = client
        .app()
        .media_group(
            1_i64,
            vec![
                serde_json::from_value::<InputMedia>(serde_json::json!({
                    "type": "photo",
                    "media": "blocking-group-photo-file-id",
                    "caption": "blocking group photo caption",
                    "parse_mode": "MarkdownV2"
                }))?,
                serde_json::from_value::<InputMedia>(serde_json::json!({
                    "type": "video",
                    "media": "blocking-group-video-file-id",
                    "caption": "blocking group video caption",
                    "parse_mode": "MarkdownV2",
                    "width": 1280,
                    "height": 720,
                    "duration": 45,
                    "supports_streaming": true
                }))?,
            ],
        )?
        .reply_to_message(MessageId(72))
        .message_thread_id(28)
        .send()?;
    assert_eq!(group.len(), 2);
    assert_eq!(group[0].message_id.0, 27);

    join_server(server)?;
    Ok(())
}

#[tokio::test]
async fn blocking_sticker_builder_supports_common_send_options() -> Result<(), DynError> {
    let client = BlockingClient::builder("https://api.telegram.org")?
        .bot_token("123:abc")?
        .build_blocking()?;
    let markup = InlineKeyboardMarkup::single_row(vec![InlineKeyboardButton::callback(
        "Review blocking sticker",
        "blocking-sticker:1",
    )?]);

    let request = client
        .app()
        .sticker(1_i64, "blocking-sticker-file-id")
        .emoji(":ok:")
        .reply_to_message(MessageId(71))?
        .message_thread_id(27)
        .reply_markup(markup)?
        .into_request();

    assert_eq!(request.emoji.as_deref(), Some(":ok:"));
    assert_eq!(request.message_thread_id, Some(27));
    assert_eq!(
        request.reply_parameters,
        Some(serde_json::json!({"message_id":71}))
    );
    assert_eq!(
        request.reply_markup,
        Some(serde_json::json!({
            "inline_keyboard": [[{"text":"Review blocking sticker","callback_data":"blocking-sticker:1"}]]
        }))
    );

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
async fn blocking_membership_facade_handles_bot_member_and_capabilities() -> Result<(), DynError> {
    let expectations = vec![
        RequestExpectation::post("/bot123:abc/getMe").respond_json(
            200,
            r#"{"ok":true,"result":{"id":999,"is_bot":true,"first_name":"tele","username":"tele_blocking_bot"}}"#,
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
            r#"{"ok":true,"result":{"id":999,"is_bot":true,"first_name":"tele","username":"tele_blocking_bot"}}"#,
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

    let client = BlockingClient::builder(server.base_url())?
        .bot_token("123:abc")?
        .build_blocking()?;
    let membership = client.app().membership();

    let bot_member = membership.bot_member(-10010_i64)?;
    assert_eq!(bot_member.user().id.0, 999);
    assert!(bot_member.has_capability(ChatAdministratorCapability::ManageChat));

    let missing = membership.bot_missing_capabilities(
        -10010_i64,
        &[
            ChatAdministratorCapability::ManageChat,
            ChatAdministratorCapability::RestrictMembers,
        ],
    )?;
    assert_eq!(missing, vec![ChatAdministratorCapability::RestrictMembers]);

    let administrators = membership.administrators(-10010_i64)?;
    assert_eq!(administrators.len(), 2);

    join_server(server)?;
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

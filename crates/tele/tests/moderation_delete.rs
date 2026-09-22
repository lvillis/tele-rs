#![cfg(any(feature = "_async", feature = "_blocking"))]

use serde_json::{Value, json};
use tele::testing::{FakeTelegramServer, RequestExpectation};
use tele::types::{Message, Update};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

fn message() -> Value {
    json!({
        "message_id": 42, "date": 1,
        "chat": {"id": -100, "type": "supergroup"},
        "from": {"id": 123, "is_bot": true, "first_name": "bot"}
    })
}

fn deletion_cases() -> TestResult<Vec<(Update, &'static str, Value)>> {
    let regular = message();
    let mut business = message();
    business["business_connection_id"] = json!("connection-1");
    let mut ephemeral = message();
    ephemeral["message_id"] = json!(0);
    ephemeral["ephemeral_message_id"] = json!(91);
    ephemeral["receiver_user"] = json!({"id": 7, "is_bot": false, "first_name": "recipient"});

    Ok(vec![
        (
            serde_json::from_value(json!({"update_id": 1, "message": regular}))?,
            "deleteMessage",
            json!({"chat_id": -100, "message_id": 42}),
        ),
        (
            serde_json::from_value(json!({"update_id": 2, "business_message": business}))?,
            "deleteBusinessMessages",
            json!({"business_connection_id": "connection-1", "message_ids": [42]}),
        ),
        (
            serde_json::from_value(json!({"update_id": 3, "edited_business_message": business}))?,
            "deleteBusinessMessages",
            json!({"business_connection_id": "connection-1", "message_ids": [42]}),
        ),
        (
            serde_json::from_value(json!({"update_id": 4, "message": ephemeral}))?,
            "deleteEphemeralMessage",
            json!({"chat_id": -100, "receiver_user_id": 7, "ephemeral_message_id": 91}),
        ),
    ])
}

fn target_message(update: &Update) -> TestResult<&Message> {
    update
        .message
        .as_deref()
        .or(update.business_message.as_deref())
        .or(update.edited_business_message.as_deref())
        .ok_or_else(|| "missing fixture message".into())
}

fn assert_request_bodies(server: FakeTelegramServer, expected: &[Value]) -> TestResult {
    let requests = server.finish()?;
    assert_eq!(requests.len(), expected.len());
    for (request, expected_body) in requests.iter().zip(expected) {
        let (_, body) = request
            .raw
            .split_once("\r\n\r\n")
            .ok_or("missing request body")?;
        assert_eq!(serde_json::from_str::<Value>(body)?, *expected_body);
    }
    Ok(())
}

fn rejected_messages() -> TestResult<Vec<Message>> {
    let mut guest = message();
    guest["guest_query_id"] = json!("guest-1");
    let mut missing_receiver = message();
    missing_receiver["ephemeral_message_id"] = json!(91);
    let mut business_ephemeral = missing_receiver.clone();
    business_ephemeral["business_connection_id"] = json!("connection-1");
    business_ephemeral["receiver_user"] =
        json!({"id": 7, "is_bot": false, "first_name": "recipient"});
    [guest, missing_receiver, business_ephemeral]
        .into_iter()
        .map(|value| serde_json::from_value(value).map_err(Into::into))
        .collect()
}

fn rejected_updates() -> TestResult<Vec<Update>> {
    // The update kind still identifies a separate namespace if a malformed
    // payload omits the context field from its embedded message.
    [
        "guest_message",
        "business_message",
        "edited_business_message",
    ]
    .into_iter()
    .map(|kind| {
        serde_json::from_value(json!({"update_id": 1, kind: message()})).map_err(Into::into)
    })
    .collect()
}

fn ephemeral_callback() -> TestResult<Update> {
    let mut message = message();
    message["message_id"] = json!(0);
    message["ephemeral_message_id"] = json!(91);
    Ok(serde_json::from_value(json!({
        "update_id": 1,
        "callback_query": {
            "id": "callback-1", "chat_instance": "chat-1", "data": "dismiss",
            "from": {"id": 7, "is_bot": false, "first_name": "recipient"},
            "message": message
        }
    }))?)
}

#[cfg(feature = "_async")]
#[tokio::test]
async fn async_deletion_preserves_message_context() -> TestResult {
    for (update, method, body) in deletion_cases()? {
        let server = FakeTelegramServer::start(vec![
            RequestExpectation::post(format!("/bot123:abc/{method}")),
            RequestExpectation::post(format!("/bot123:abc/{method}")),
        ])?;
        let client = tele::Client::builder(server.base_url())?
            .bot_token("123:abc")?
            .build()?;
        let moderation = client.app().moderation();
        assert!(moderation.delete(target_message(&update)?).await?);
        assert!(moderation.delete_from_update(&update).await?);
        assert_request_bodies(server, &[body.clone(), body])?;
    }
    Ok(())
}

#[cfg(feature = "_blocking")]
#[test]
fn blocking_deletion_preserves_message_context() -> TestResult {
    for (update, method, body) in deletion_cases()? {
        let server = FakeTelegramServer::start(vec![
            RequestExpectation::post(format!("/bot123:abc/{method}")),
            RequestExpectation::post(format!("/bot123:abc/{method}")),
        ])?;
        let client = tele::BlockingClient::builder(server.base_url())?
            .bot_token("123:abc")?
            .build_blocking()?;
        let moderation = client.app().moderation();
        assert!(moderation.delete(target_message(&update)?)?);
        assert!(moderation.delete_from_update(&update)?);
        assert_request_bodies(server, &[body.clone(), body])?;
    }
    Ok(())
}

#[cfg(feature = "_async")]
#[tokio::test]
async fn async_deletion_rejects_ambiguous_targets_locally() -> TestResult {
    let client = tele::Client::builder("http://127.0.0.1:9")?
        .bot_token("123:abc")?
        .build()?;
    let moderation = client.app().moderation();
    for message in rejected_messages()? {
        assert!(matches!(
            moderation.delete(&message).await,
            Err(tele::Error::InvalidRequest { .. })
        ));
    }
    for update in rejected_updates()? {
        assert!(matches!(
            moderation.delete_from_update(&update).await,
            Err(tele::Error::InvalidRequest { .. })
        ));
    }
    Ok(())
}

#[cfg(feature = "_blocking")]
#[test]
fn blocking_deletion_rejects_ambiguous_targets_locally() -> TestResult {
    let client = tele::BlockingClient::builder("http://127.0.0.1:9")?
        .bot_token("123:abc")?
        .build_blocking()?;
    let moderation = client.app().moderation();
    for message in rejected_messages()? {
        assert!(matches!(
            moderation.delete(&message),
            Err(tele::Error::InvalidRequest { .. })
        ));
    }
    for update in rejected_updates()? {
        assert!(matches!(
            moderation.delete_from_update(&update),
            Err(tele::Error::InvalidRequest { .. })
        ));
    }
    Ok(())
}

#[cfg(feature = "_async")]
#[tokio::test]
async fn async_ephemeral_callback_supplies_deletion_recipient() -> TestResult {
    let server = FakeTelegramServer::single(RequestExpectation::post(
        "/bot123:abc/deleteEphemeralMessage",
    ))?;
    let client = tele::Client::builder(server.base_url())?
        .bot_token("123:abc")?
        .build()?;
    assert!(
        client
            .app()
            .moderation()
            .delete_from_update(&ephemeral_callback()?)
            .await?
    );
    assert_request_bodies(
        server,
        &[json!({"chat_id": -100, "receiver_user_id": 7, "ephemeral_message_id": 91})],
    )
}

#[cfg(feature = "_blocking")]
#[test]
fn blocking_ephemeral_callback_supplies_deletion_recipient() -> TestResult {
    let server = FakeTelegramServer::single(RequestExpectation::post(
        "/bot123:abc/deleteEphemeralMessage",
    ))?;
    let client = tele::BlockingClient::builder(server.base_url())?
        .bot_token("123:abc")?
        .build_blocking()?;
    assert!(
        client
            .app()
            .moderation()
            .delete_from_update(&ephemeral_callback()?)?
    );
    assert_request_bodies(
        server,
        &[json!({"chat_id": -100, "receiver_user_id": 7, "ephemeral_message_id": 91})],
    )
}

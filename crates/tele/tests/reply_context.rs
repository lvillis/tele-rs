#![cfg(any(feature = "_async", feature = "_blocking"))]

use serde_json::json;
use tele::types::{Message, Update};
use tele::{Error, Result};

type TestResult = std::result::Result<(), Box<dyn std::error::Error>>;

fn source_message() -> std::result::Result<Message, serde_json::Error> {
    serde_json::from_value(json!({
        "message_id": 12, "date": 1,
        "chat": {"id": -100, "type": "supergroup"},
        "from": {"id": 42, "is_bot": false, "first_name": "sender"},
        "message_thread_id": 7,
        "direct_messages_topic": {"topic_id": 8},
        "business_connection_id": "business"
    }))
}

macro_rules! reply_tests {
    () => {
        #[test]
        fn message_and_update_replies_preserve_the_same_delivery_context() -> TestResult {
            let app = client()?.app();
            let mut message = source_message()?;
            for ephemeral in [false, true] {
                if ephemeral {
                    message.message_id = 0.into();
                    message.ephemeral_message_id = Some(15);
                    message.receiver_user = Some(serde_json::from_value(json!({
                        "id": 43, "is_bot": false, "first_name": "recipient"
                    }))?);
                }
                let mut update: Update = serde_json::from_value(json!({"update_id": 1}))?;
                update.message = Some(Box::new(message.clone()));
                let expected = app.reply(&update, "reply")?.into_request();
                for request in [
                    app.reply_to(&message, "reply")?.into_request(),
                    app.moderation().notice().for_message(&message, "reply")?.into_request(),
                ] {
                    request.validate()?;
                    let value = serde_json::to_value(request)?;
                    assert_eq!(value, serde_json::to_value(&expected)?);
                    assert_eq!(value["chat_id"], -100);
                    assert_eq!(value["message_thread_id"], 7);
                    assert_eq!(value["direct_messages_topic_id"], 8);
                    assert_eq!(value["business_connection_id"], "business");
                    if ephemeral {
                        assert_eq!(value["ephemeral_message_parameters"]["receiver_user_id"], 43);
                        assert_eq!(value["reply_parameters"]["ephemeral_message_id"], 15);
                        assert!(value["reply_parameters"].get("message_id").is_none());
                    } else {
                        assert_eq!(value["reply_parameters"]["message_id"], 12);
                    }
                }
            }
            Ok(())
        }

        #[test]
        fn ephemeral_reply_requires_a_real_recipient() -> TestResult {
            let app = client()?.app();
            let mut message = source_message()?;
            message.message_id = 0.into();
            message.ephemeral_message_id = Some(15);
            let request = app.reply_to(&message, "private")?.into_request();
            assert_eq!(request.ephemeral_message_parameters.map(|p| p.receiver_user_id.0), Some(42));

            // A compatibility sender must not become the recipient of a private reply.
            message.sender_chat = Some(message.chat.clone());
            for result in [
                app.reply_to(&message, "private"),
                app.moderation().notice().for_message(&message, "private"),
            ] {
                assert!(matches!(result, Err(Error::InvalidRequest { reason })
                    if reason.contains("receiving user's identity")));
            }
            message.sender_chat = None;
            message.from = None;
            assert!(matches!(app.reply_to(&message, "private"), Err(Error::InvalidRequest { .. })));
            Ok(())
        }

        #[test]
        fn outgoing_ephemeral_messages_require_the_original_interaction() -> TestResult {
            let app = client()?.app();
            let mut message = source_message()?;
            message.message_id = 0.into();
            message.ephemeral_message_id = Some(15);
            message.receiver_user = message.from.clone();
            message.from = Some(serde_json::from_value(json!({
                "id": 99, "is_bot": true, "first_name": "bot"
            }))?);
            for result in [
                app.reply_to(&message, "reply"),
                app.moderation().notice().for_message(&message, "reply"),
            ] {
                assert!(matches!(result, Err(Error::InvalidRequest { reason })
                    if reason.contains("outgoing ephemeral message")));
            }
            Ok(())
        }

        #[test]
        fn ephemeral_callback_replies_use_the_clicker_and_callback_token() -> TestResult {
            let app = client()?.app();
            let update: Update = serde_json::from_value(json!({
                "update_id": 1,
                "callback_query": {
                    "id": "callback-token", "chat_instance": "chat", "data": "action",
                    "from": {"id": 42, "is_bot": false, "first_name": "clicker"},
                    "message": {
                        "message_id": 0, "ephemeral_message_id": 15, "date": 1,
                        "chat": {"id": -100, "type": "supergroup"},
                        "from": {"id": 99, "is_bot": true, "first_name": "bot"}
                    }
                }
            }))?;
            let request = app.reply(&update, "reply")?.into_request();
            request.validate()?;
            let value = serde_json::to_value(request)?;
            assert_eq!(value["ephemeral_message_parameters"]["receiver_user_id"], 42);
            assert_eq!(value["ephemeral_message_parameters"]["callback_query_id"], "callback-token");
            assert!(value.get("reply_parameters").is_none());
            Ok(())
        }

        #[test]
        fn poll_voter_identity_is_not_a_reply_destination() -> TestResult {
            let app = client()?.app();
            for voter in [
                json!({"voter_chat": {"id": -200, "type": "channel"}}),
                json!({"user": {"id": 42, "is_bot": false, "first_name": "voter"}}),
            ] {
                let mut answer = voter;
                answer["poll_id"] = json!("poll");
                answer["option_ids"] = json!([0]);
                let update: Update = serde_json::from_value(json!({
                    "update_id": 1, "poll_answer": answer
                }))?;
                assert!(matches!(app.reply(&update, "vote received"),
                    Err(Error::InvalidRequest { reason }) if reason.contains("chat id")));
            }
            Ok(())
        }

        #[test]
        fn guest_messages_cannot_bypass_the_reply_check() -> TestResult {
            let app = client()?.app();
            let mut message = source_message()?;
            message.guest_query_id = Some("guest-query".to_owned());
            for result in [
                app.reply_to(&message, "reply"),
                app.moderation().notice().for_message(&message, "reply"),
            ] {
                assert!(matches!(result, Err(Error::InvalidRequest { reason })
                    if reason.contains("answerGuestQuery")));
            }
            Ok(())
        }
    };
}

#[cfg(feature = "_async")]
mod asynchronous {
    use super::*;

    fn client() -> Result<tele::Client> {
        tele::Client::builder("http://127.0.0.1:9")?
            .bot_token("123:abc")?
            .build()
    }

    reply_tests!();
}

#[cfg(feature = "_blocking")]
mod blocking {
    use super::*;

    fn client() -> Result<tele::BlockingClient> {
        tele::BlockingClient::builder("http://127.0.0.1:9")?
            .bot_token("123:abc")?
            .build_blocking()
    }

    reply_tests!();
}

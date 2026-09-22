#![cfg(feature = "bot")]

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use serde_json::json;
use tele::bot::{BotContext, HandlerError, Router, UpdateExt};
use tele::types::Update;
use tele::{Client, Error};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn poll_voter_chat_does_not_supply_the_poll_conversation() -> TestResult {
    let update: Update = serde_json::from_value(json!({
        "update_id": 1,
        "poll_answer": {
            "poll_id": "forwarded-poll",
            "voter_chat": {"id": -10055, "type": "supergroup", "title": "voter"},
            "option_ids": [1]
        }
    }))?;

    assert!(update.chat().is_none());
    assert_eq!(update.chat_id(), None);
    assert_eq!(
        update
            .poll_answer
            .as_ref()
            .and_then(|answer| answer.voter_chat.as_ref())
            .map(|chat| chat.id),
        Some(-10055)
    );
    Ok(())
}

#[tokio::test]
async fn middleware_can_retry_an_extractor_route() -> TestResult {
    struct NonCloneText(String);

    let client = Client::builder("http://127.0.0.1:9")?
        .bot_token("123:abc")?
        .build()?;
    let attempts = Arc::new(AtomicUsize::new(0));
    let preparations = Arc::new(AtomicUsize::new(0));
    let mut router = Router::new();
    router.middleware(|context, update, next| async move {
        if next(context.clone(), update.clone()).await.is_err() {
            next(context, update).await
        } else {
            Ok(())
        }
    });
    let handler_attempts = Arc::clone(&attempts);
    let mapper_preparations = Arc::clone(&preparations);
    router
        .text_route()
        .map(move |text, _update| {
            mapper_preparations.fetch_add(1, Ordering::SeqCst);
            Some(NonCloneText(text.0.clone()))
        })
        .handle(move |_context, _update, text| {
            let attempts = Arc::clone(&handler_attempts);
            async move {
                assert_eq!(text.0, "hello");
                if attempts.fetch_add(1, Ordering::SeqCst) == 0 {
                    return Err(HandlerError::internal(Error::InvalidRequest {
                        reason: "retryable application failure".to_owned(),
                    }));
                }
                Ok(())
            }
        });
    let update: Update = serde_json::from_value(json!({
        "update_id": 2,
        "message": {
            "message_id": 1,
            "date": 1700000000,
            "chat": {"id": 10, "type": "private"},
            "text": "hello"
        }
    }))?;

    assert!(
        router
            .dispatch(BotContext::new(client.clone()), update.clone())
            .await?
    );
    assert_eq!(attempts.load(Ordering::SeqCst), 2);
    assert_eq!(preparations.load(Ordering::SeqCst), 2);

    // The ordinary, successful dispatch still prepares its owned input once.
    assert!(router.dispatch(BotContext::new(client), update).await?);
    assert_eq!(attempts.load(Ordering::SeqCst), 3);
    assert_eq!(preparations.load(Ordering::SeqCst), 3);
    Ok(())
}

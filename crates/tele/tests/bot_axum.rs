#![cfg(feature = "axum")]

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use serde_json::json;
use tele::bot::axum::{
    TELEGRAM_SECRET_HEADER, dispatch_webhook, dispatch_webhook_status, telegram_secret_token,
    webhook_handler,
};
use tele::bot::{BotContext, DispatchOutcome, Router, WebhookRunner};
use tele::types::update::Update;
use tele::{Client, Error};

type DynError = Box<dyn std::error::Error>;

fn build_client() -> Result<Client, DynError> {
    let client = Client::builder("http://127.0.0.1:9")?
        .bot_token("123:abc")?
        .build()?;
    Ok(client)
}

fn update_payload(text: &str) -> Result<Vec<u8>, DynError> {
    let payload = serde_json::to_vec(&json!({
        "update_id": 42,
        "message": {
            "message_id": 10,
            "date": 1710000000,
            "chat": {"id": 100, "type": "private"},
            "text": text
        }
    }))?;
    Ok(payload)
}

fn secret_headers(secret: &str) -> Result<HeaderMap, DynError> {
    let mut headers = HeaderMap::new();
    headers.insert(
        TELEGRAM_SECRET_HEADER,
        HeaderValue::from_str(secret).map_err(|error| format!("invalid secret header: {error}"))?,
    );
    Ok(headers)
}

#[tokio::test]
async fn dispatch_webhook_runs_router_handler() -> Result<(), DynError> {
    let client = build_client()?;
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

    let runner = WebhookRunner::new(client, router).expected_secret_token("secret");
    let payload = update_payload("/start hello")?;
    let headers = secret_headers("secret")?;

    let outcome = dispatch_webhook(&runner, &headers, &payload).await?;
    assert_eq!(outcome, DispatchOutcome::Handled { update_id: 42 });
    assert_eq!(handler_hits.load(Ordering::SeqCst), 1);

    Ok(())
}

#[tokio::test]
async fn dispatch_webhook_status_maps_secret_and_json_errors() -> Result<(), DynError> {
    let client = build_client()?;
    let runner = WebhookRunner::new(client, Router::new()).expected_secret_token("secret");

    let wrong_headers = secret_headers("wrong")?;
    let payload = update_payload("hello")?;
    let unauthorized = dispatch_webhook_status(&runner, &wrong_headers, &payload).await;
    assert_eq!(unauthorized, StatusCode::UNAUTHORIZED);

    let good_headers = secret_headers("secret")?;
    let bad_payload = br#"{"update_id":"invalid"}"#;
    let bad_request = dispatch_webhook_status(&runner, &good_headers, bad_payload).await;
    assert_eq!(bad_request, StatusCode::BAD_REQUEST);

    Ok(())
}

#[tokio::test]
async fn dispatch_webhook_status_maps_handler_error_to_500() -> Result<(), DynError> {
    let client = build_client()?;
    let mut router = Router::new();
    router.on_message(|_context: BotContext, _update: Update| async move {
        Err(Error::InvalidRequest {
            reason: "handler failed".to_owned(),
        })
    });

    let runner = WebhookRunner::new(client, router);
    let payload = update_payload("hello")?;
    let headers = HeaderMap::new();

    let status = dispatch_webhook_status(&runner, &headers, &payload).await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);

    Ok(())
}

#[tokio::test]
async fn webhook_handler_works_with_axum_state() -> Result<(), DynError> {
    let client = build_client()?;
    let mut router = Router::new();
    router.on_message(|_context: BotContext, _update: Update| async move { Ok(()) });

    let runner = Arc::new(WebhookRunner::new(client, router));
    let payload = update_payload("hello")?;
    let headers = HeaderMap::new();

    assert_eq!(telegram_secret_token(&headers), None);

    let status = webhook_handler(State(runner), headers, Bytes::from(payload)).await;
    assert_eq!(status, StatusCode::OK);

    Ok(())
}

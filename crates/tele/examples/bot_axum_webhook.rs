use std::env;
use std::sync::Arc;

use axum::{Router, routing::post};
use tele::Client;
use tele::bot::axum::webhook_handler;
use tele::bot::{BotContext, Router as BotRouter, UpdateExt, WebhookRunner};
use tele::types::update::Update;

fn read_env(name: &str) -> Result<String, Box<dyn std::error::Error>> {
    env::var(name).map_err(|error| format!("missing environment variable {name}: {error}").into())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = read_env("TELEGRAM_BOT_TOKEN")?;
    let webhook_secret = env::var("TELEGRAM_WEBHOOK_SECRET").ok();
    let webhook_path =
        env::var("TELEGRAM_WEBHOOK_PATH").unwrap_or_else(|_| "/telegram/webhook".to_owned());
    let bind = env::var("TELEGRAM_BIND").unwrap_or_else(|_| "0.0.0.0:8080".to_owned());

    let client = Client::builder("https://api.telegram.org")?
        .bot_token(token)?
        .build()?;

    let mut router = BotRouter::new();
    router
        .message_route()
        .handle(|context: BotContext, update: Update| async move {
            let text = update.text().unwrap_or("non-text message");
            let _sent = context
                .reply_text(&update, format!("webhook echo: {text}"))
                .await?;
            Ok(())
        });

    let mut runner = WebhookRunner::new(client.clone(), router);
    if let Some(secret) = webhook_secret {
        runner = runner.expected_secret_token(secret);
    }
    let runner = Arc::new(runner);

    let app = Router::new()
        .route(&webhook_path, post(webhook_handler))
        .with_state(Arc::clone(&runner));

    let listener = tokio::net::TcpListener::bind(&bind).await?;
    println!("listening on http://{bind}{webhook_path}");
    axum::serve(listener, app).await?;
    Ok(())
}

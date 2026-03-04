# tele-rs

Ergonomic Telegram Bot API SDK for Rust, powered by `reqx`.

## Features

- `async-tls-rustls-ring` (default): async client + rustls (ring provider)
- `async-tls-rustls-aws-lc-rs`: async client + rustls (aws-lc-rs provider)
- `async-tls-native`: async client + native-tls
- `blocking-tls-rustls-ring`: blocking client + rustls (ring provider)
- `blocking-tls-rustls-aws-lc-rs`: blocking client + rustls (aws-lc-rs provider)
- `blocking-tls-native`: blocking client + native-tls
- `async` / `blocking`: legacy aliases (map to rustls-ring variants)
- `bot`: complete bot runtime toolkit (extractor-based router, source-agnostic engine, long polling with dedupe/offset persistence, webhook dispatch, session stores, reliable outbox with dead-letter, observability hooks, testing harness)
- `axum`: axum webhook adapter built on top of `bot`
- `macros`: derive macros for typed bot commands (`#[derive(tele::BotCommands)]`)
- `otel`: enable reqx OpenTelemetry hooks
- `redis-session`: Redis-backed bot session store (`RedisSessionStore`)
- `postgres-session`: Postgres-backed bot session store (`PostgresSessionStore`)

## Covered APIs

- Bot/account: `getMe`, commands management, bot profile texts, user profile photos
- Messaging: text/media send, forward/copy, live location, poll, dice, edit/delete
- Chats: member/admin queries, permissions, moderation, invite links, pin/title/description
- Updates: polling, webhook config, callback/inline query answer
- API layers: `client.raw()` (raw method calls), `client.typed()` (request-associated response type), `client.ergo()` (high-level helpers)
- Bot runtime (`feature = "bot"`): router + middleware, `UpdateSource` + `BotEngine`/`BotApp`, long polling source (duplicate `update_id` filtering + optional offset persistence), webhook runner, runtime event hooks (`EngineEvent`)
- Bot ergonomics (`feature = "bot"`): typed extractors (`TextInput`/`CallbackInput`/`WebAppInput`/`WriteAccessAllowedInput`/`TypedCommandInput`), extractor combinators (`on_extracted_filter` / `on_extracted_map` / `on_extracted_guard`), declarative `ErrorPolicy`, `UpdateExt` helpers, `ChatSession` FSM wrapper, typed command routing
- Reliability (`feature = "bot"`): `BotOutbox` send queue with retry/backoff/429 handling, idempotency dedupe, optional on-disk queue persistence/replay, dead-letter recording, and message max-age expiration
- Sessions (`feature = "bot"`): `InMemorySessionStore` and `JsonFileSessionStore`
- Distributed sessions: `RedisSessionStore` (`feature = "redis-session"`), `PostgresSessionStore` (`feature = "postgres-session"`)
- Testing (`feature = "bot"`): `tele::bot::testing` fixtures and `BotHarness`
- Axum integration (`feature = "axum"`): ready-to-use webhook handler and status mapping helpers
- Files: `getFile`
- Full method coverage: all Bot API 9.4 methods are exposed, including newer domains
  (business, gifts/stars, stories, stickers, games, forums, invoices) via
  `client.advanced()` / `blocking_client.advanced()` with typed request models.
- First-class strong-typed domains: `client.stickers()` and `client.payments()`
  (also available for blocking client), with typed request/response models and upload helpers.

## Quick Start (async)

```rust,no_run
use tele::{Client, ParseMode, SendMessageRequest};

#[tokio::main]
async fn main() -> Result<(), tele::Error> {
    let client = Client::builder("https://api.telegram.org")?
        .bot_token("123456:telegram-bot-token")?
        .build()?;

    let me = client.bot().get_me().await?;
    println!("bot username: {:?}", me.username);

    let request = SendMessageRequest::new(123456789_i64, "hello from tele")?
        .parse_mode(ParseMode::MarkdownV2);
    let _sent = client.messages().send_message(&request).await?;

    Ok(())
}
```

## Upload Local File (async)

```rust,no_run
use tele::{Client, SendPhotoRequest, UploadFile};

#[tokio::main]
async fn main() -> Result<(), tele::Error> {
    let client = Client::builder("https://api.telegram.org")?
        .bot_token("123456:telegram-bot-token")?
        .build()?;

    let file = UploadFile::from_path("./image.jpg")?;
    let request = SendPhotoRequest::new(123456789_i64, "attach://image.jpg");
    let _message = client.messages().send_photo_upload(&request, &file).await?;

    Ok(())
}
```

## Quick Start (blocking)

```rust,no_run
use tele::{BlockingClient, SendMessageRequest};

fn main() -> Result<(), tele::Error> {
    let client = BlockingClient::builder("https://api.telegram.org")?
        .bot_token("123456:telegram-bot-token")?
        .build_blocking()?;

    let me = client.bot().get_me()?;
    println!("bot id: {}", me.id.0);

    let request = SendMessageRequest::new(123456789_i64, "hello")?;
    let _sent = client.messages().send_message(&request)?;

    Ok(())
}
```

## Bot Runtime (feature `bot`)

```rust,no_run
use tele::Client;
use tele::bot::{BotApp, BotContext, EngineConfig, Router, TextInput, UpdateExt};
use tele::types::update::Update;

#[tokio::main]
async fn main() -> Result<(), tele::Error> {
    let client = Client::builder("https://api.telegram.org")?
        .bot_token("123456:telegram-bot-token")?
        .build()?;

    let mut router = Router::new();
    router.on_command("start", |context: BotContext, update: Update| async move {
        let _ = context.reply_text(&update, "bot is running").await?;
        Ok(())
    });
    router.on_extracted::<TextInput, _, _>(|context: BotContext, update: Update, text| async move {
        let Some(chat_id) = update.chat_id() else {
            return Ok(());
        };
        let _ = context.send_text(chat_id, format!("echo: {}", text.into_inner())).await?;
        Ok(())
    });

    let mut app = BotApp::long_polling(client, router).with_engine_config(EngineConfig {
        max_handler_concurrency: 8,
        ..EngineConfig::default()
    });
    app.engine_mut().source_mut().config_mut().poll_timeout_seconds = 20;
    app.run().await
}
```

## API Layers (Raw / Typed / Ergo)

```rust,no_run
use tele::types::advanced::AdvancedGetAvailableGiftsRequest;
use tele::Client;

#[tokio::main]
async fn main() -> Result<(), tele::Error> {
    let client = Client::builder("https://api.telegram.org")?
        .bot_token("123456:telegram-bot-token")?
        .build()?;

    let _me: tele::types::User = client.raw().call_no_params("getMe").await?;
    let _gifts: serde_json::Value = client
        .typed()
        .call(&AdvancedGetAvailableGiftsRequest::new())
        .await?;
    let _sent = client.ergo().send_text(123456789_i64, "hello from ergo layer").await?;
    Ok(())
}
```

## Axum Webhook (feature `axum`)

```rust,no_run
use std::sync::Arc;
use axum::{Router, routing::post};
use tele::Client;
use tele::bot::WebhookRunner;
use tele::bot::axum::webhook_handler;

#[tokio::main]
async fn main() -> Result<(), tele::Error> {
    let client = Client::builder("https://api.telegram.org")?
        .bot_token("123456:telegram-bot-token")?
        .build()?;

    let runner = Arc::new(WebhookRunner::new(client, tele::bot::Router::new()));

    let app = Router::new()
        .route("/telegram/webhook", post(webhook_handler))
        .with_state(runner);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    axum::serve(listener, app)
        .await
        .map_err(|error| tele::Error::InvalidRequest {
            reason: format!("axum server error: {error}"),
        })?;

    Ok(())
}
```

## Typed Commands (feature `macros`)

```rust,no_run
#[derive(Debug, tele::BotCommands)]
enum Command {
    #[command(description = "start the bot", alias = "run")]
    Start,
    #[command(description = "echo text", aliases("repeat", "say"))]
    Echo(String),
}
```

- Supports `alias = "..."` and `aliases("...", "...")` for multiple command triggers.
- Typed argument parsing supports quotes and escapes, for example:
  `"/echo \"hello world\""` -> `Command::Echo("hello world".into())`.

## Quality Gates

- `just answer-check`
- `just ci`
- `just release-check`

## Release Workflow

- Workspace uses `cargo-release` with lockstep versioning (`Cargo.toml` -> `[workspace.metadata.release]`).
- First publish with current versions (no bump): `just release-run-unpublished`
- Preview a normal bump: `just release-plan level=patch`
- Execute a normal bump: `just release-run level=patch`
- Replace `patch` with `minor`, `major`, `rc`, `beta`, or `alpha` when needed.
- Release requires a clean git tree and an existing branch head commit.

## Design Targets

- Keep `client.raw()` / `client.typed()` / `client.ergo()` API stable and discoverable.
- Keep webhook core framework-agnostic (`WebhookRunner` + `DispatchOutcome`), adapters as optional features.
- Prefer strong request/response models over ad-hoc JSON values for public request fields.
- Keep runtime defaults robust (timeout/retry/concurrency/rate-limit controls).

## Examples

- Async send message (default backend): `cargo run -p tele --example async_send_message`
- Async send message (native-tls): `cargo run -p tele --example async_send_message --no-default-features --features async-tls-native`
- Blocking send message: `cargo run -p tele --example blocking_send_message --no-default-features --features blocking-tls-rustls-ring`
- Long polling bot: `cargo run -p tele --example bot_long_polling --features bot`
- Engine long polling bot: `cargo run -p tele --example bot_engine_long_polling --features bot`
- Quickstart bot (extractors + outbox + app wrapper): `cargo run -p tele --example bot_quickstart --features bot`
- Mini App bot (menu button + `web_app_data` + `answerWebAppQuery`): `cargo run -p tele --example bot_mini_app --features bot`
- Fallible FSM bot: `cargo run -p tele --example bot_fallible_session --features bot`
- Typed command bot: `cargo run -p tele --example bot_typed_commands --features macros`
- Axum webhook bot: `cargo run -p tele --example bot_axum_webhook --features axum`
- API layers demo: `cargo run -p tele --example api_layers`

Required environment variables:
- `TELEGRAM_BOT_TOKEN`
- `TELEGRAM_CHAT_ID` (send message examples)
- Optional: `TELEGRAM_TEXT`, `TELEGRAM_WEBHOOK_SECRET`, `TELEGRAM_WEBHOOK_PATH`, `TELEGRAM_BIND`, `TELEGRAM_OUTBOX_PATH`, `TELEGRAM_MINI_APP_URL`

Mini App backend verification helpers:
- `tele::verify_web_app_init_data(bot_token, init_data, max_age)`
- `tele::parse_web_app_init_data(init_data)`

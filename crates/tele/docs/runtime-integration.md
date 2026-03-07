# Runtime Integration Guide

This guide covers the recommended way to run `tele` bots inside a larger Tokio application.

## Spawnability

`BotEngine::run_until(...)` and `BotApp::run_until(...)` return `Send` futures. You can spawn them
with `tokio::spawn` like any other service task.

```rust,no_run
use tokio::sync::oneshot;
use tele::Client;
use tele::bot::{BotApp, Router};

#[tokio::main]
async fn main() -> Result<(), tele::Error> {
    let client = Client::builder("https://api.telegram.org")?
        .bot_token("123456:telegram-bot-token")?
        .build()?;

    let mut app = BotApp::long_polling(client, Router::new());
    let (_shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

    let bot_task = tokio::spawn(async move {
        let _ = app.run_until(async {
            let _ = shutdown_rx.await;
        }).await;
    });

    bot_task.await.expect("bot task join");
    Ok(())
}
```

## Recommended pattern

For multi-service apps:

- Build the bot once at startup.
- Run bootstrap before spawning if you want startup failures to fail fast.
- Pass a shared shutdown future into `run_until(...)`.
- Run your HTTP server, jobs, and bot as separate Tokio tasks.

Typical shape:

```text
main
|- bootstrap bot
|- spawn bot task
|- spawn http server task
|- spawn background workers
`- wait for shutdown signal, then cancel all tasks
```

## Graceful shutdown

`run_until(shutdown)` stops polling as soon as `shutdown` resolves.

Use whatever cancellation primitive already fits your app:

- `tokio::sync::oneshot`
- `tokio::sync::watch`
- `tokio_util::sync::CancellationToken`
- your own signal future

For long polling, shutdown latency is bounded by the current poll request finishing. Keep
`poll_timeout_seconds` aligned with your application's shutdown expectations.

## Bootstrap placement

The recommended startup order is:

1. Build `Client`, `Router`, and `BotApp`/`BotEngine`.
2. Run `bootstrap(...)` or `bootstrap_with_retry(...)` if you manage commands/menu button.
3. Spawn `run_until(...)`.

That keeps command/menu sync failures out of background task startup races.

## Webhook apps

Webhook mode is usually split differently:

- your HTTP server owns process lifetime
- `WebhookRunner` is called per request
- startup bootstrap still happens once during process init

You normally do not spawn a polling loop in webhook mode.

# tele

Ergonomic Telegram Bot API SDK and bot runtime toolkit for Rust, powered by `reqx`.

## Recommended Stable Surface

- `client.app()` / `context.app()`: primary runtime surface for business code such as text/media sends, replies, callback answers, Web App replies, moderation flows, and membership/capability checks.
- `client.control()`: startup/setup/orchestration surface for bootstrap, router preparation, and outbox management.
- `client.raw()` / `client.typed()` / `client.advanced()`: lower-level escape hatches when the high-level facades are intentionally not enough.

## Minimal Async Example

```rust,no_run
use tele::Client;
use tele::types::ParseMode;

#[tokio::main]
async fn main() -> Result<(), tele::Error> {
    let client = Client::builder("https://api.telegram.org")?
        .bot_token("123456:telegram-bot-token")?
        .build()?;

    let _sent = client
        .app()
        .text(123456789_i64, "hello from tele")?
        .parse_mode(ParseMode::MarkdownV2)
        .send()
        .await?;

    Ok(())
}
```

With `feature = "bot"`, prefer `context.app()` inside handlers and `client.control()` for startup/bootstrap/outbox orchestration.

For richer runtime flows, prefer `client.app().callback_answer(...)` for callback query replies,
`client.app().photo()/document()/video()/audio()/animation()/voice()/sticker()/media_group()` for media sends,
and `client.app().membership()` for install/bind capability checks before reaching for raw request structs.

Project guide, full API layer examples, and bot runtime examples are in the workspace root `README.md` and the `examples/` directory.

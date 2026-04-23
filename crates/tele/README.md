# tele

Telegram Bot API SDK for Rust.

```toml
[dependencies]
tele = "0.1"
```

Default is `async-tls-rustls-ring`. Choose another `async-tls-*` or `blocking-tls-*` feature if needed, and add `bot`, `axum`, `macros`, `redis-session`, or `postgres-session` when you need that surface.

```rust,no_run
use tele::Client;

#[tokio::main]
async fn main() -> Result<(), tele::Error> {
    let client = Client::builder("https://api.telegram.org")?
        .bot_token("123456:telegram-bot-token")?
        .build()?;

    client
        .app()
        .text(123456789_i64, "hello from tele")?
        .send()
        .await?;

    Ok(())
}
```

Docs: <https://docs.rs/tele>  
Examples: `examples/`

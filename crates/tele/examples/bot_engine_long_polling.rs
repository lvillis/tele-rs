use std::env;

use tele::Client;
use tele::bot::{
    BotContext, BotEngine, EngineConfig, LongPollingSource, PollingConfig, Router, UpdateExt,
};
use tele::types::update::Update;

fn read_env(name: &str) -> Result<String, Box<dyn std::error::Error>> {
    env::var(name).map_err(|error| format!("missing environment variable {name}: {error}").into())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = read_env("TELEGRAM_BOT_TOKEN")?;

    let client = Client::builder("https://api.telegram.org")?
        .bot_token(token)?
        .build()?;

    let mut router = Router::new();
    router
        .message_route()
        .handle(|context: BotContext, update: Update| async move {
            let Some(text) = update.text() else {
                return Ok(());
            };

            let reply = match update.command() {
                Some("start") => "bot engine is running".to_owned(),
                Some("ping") => "pong".to_owned(),
                _ => format!("echo: {text}"),
            };
            let _ = context.app().reply_text(&update, reply).await?;
            Ok(())
        });

    let source = LongPollingSource::new(client.clone()).with_config(PollingConfig {
        poll_timeout_seconds: 20,
        ..PollingConfig::default()
    });

    let mut engine = BotEngine::new(client, source, router).with_config(EngineConfig {
        max_handler_concurrency: 4,
        ..EngineConfig::default()
    });

    engine.run().await?;
    Ok(())
}

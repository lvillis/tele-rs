use std::env;
use std::time::Duration;

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

            let reply_text = match update.command() {
                Some("start") => "tele bot is running".to_owned(),
                Some("ping") => "pong".to_owned(),
                Some("echo") => format!("echo: {}", update.command_args().unwrap_or_default()),
                _ => format!("echo: {text}"),
            };

            let _sent = context.app().reply_text(&update, reply_text).await?;
            Ok(())
        });

    let source = LongPollingSource::new(client.clone()).with_config(PollingConfig {
        poll_timeout_seconds: 20,
        ..PollingConfig::default()
    });
    let mut engine = BotEngine::new(client, source, router).with_config(EngineConfig {
        idle_delay: Duration::from_millis(100),
        error_delay: Duration::from_millis(500),
        ..EngineConfig::default()
    });

    engine.run().await?;
    Ok(())
}

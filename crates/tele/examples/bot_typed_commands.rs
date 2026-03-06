use std::env;
use std::time::Duration;

use tele::Client;
use tele::bot::{
    BotContext, BotControl, BotEngine, EngineConfig, LongPollingSource, PollingConfig, Router,
    UpdateExt,
};
use tele::types::update::Update;

#[derive(Debug, tele::BotCommands)]
enum Command {
    #[command(description = "start the bot")]
    Start,
    #[command(description = "ping command")]
    Ping,
    #[command(description = "echo input text")]
    Echo(String),
}

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
    router.typed_command_route::<Command>().handle(
        |context: BotContext, update: Update, command: Command| async move {
            let reply = match command {
                Command::Start => "typed command bot is running".to_owned(),
                Command::Ping => "pong".to_owned(),
                Command::Echo(text) => format!("echo: {text}"),
            };

            let _ = context.reply_text(&update, reply).await?;
            Ok(())
        },
    );

    let control = BotControl::new(client.clone());
    let _ = control.set_typed_commands::<Command>().await?;

    router.fallback(|context: BotContext, update: Update| async move {
        let text = update.text().unwrap_or("unsupported update");
        let _ = context
            .reply_text(&update, format!("unknown command or input: {text}"))
            .await?;
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

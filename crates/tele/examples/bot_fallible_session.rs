use std::env;
use std::time::Duration;

use tele::Client;
use tele::bot::{
    BotContext, BotEngine, ChatSession, EngineConfig, HandlerError, InMemorySessionStore,
    LongPollingSource, PollingConfig, Router, UpdateExt,
};

fn read_env(name: &str) -> Result<String, Box<dyn std::error::Error>> {
    env::var(name).map_err(|error| format!("missing environment variable {name}: {error}").into())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = read_env("TELEGRAM_BOT_TOKEN")?;

    let client = Client::builder("https://api.telegram.org")?
        .bot_token(token)?
        .build()?;

    let session = ChatSession::<String, _>::new(InMemorySessionStore::new());
    let mut router = Router::new();

    {
        let session = session.clone();
        router
            .message_route()
            .handle(move |context: BotContext, update| {
                let session = session.clone();
                async move {
                    let Some(text) = update.text() else {
                        return Ok(());
                    };

                    if text == "/cancel" {
                        session.clear(&update).await?;
                        let _ = context
                            .app()
                            .reply_text(&update, "dialog cancelled")
                            .await?;
                        return Ok(());
                    }

                    match session.load(&update).await?.as_deref() {
                        None => {
                            session.save(&update, "awaiting_name".to_owned()).await?;
                            let _ = context
                                .app()
                                .reply_text(&update, "What is your name? Send /cancel to reset.")
                                .await?;
                        }
                        Some("awaiting_name") => {
                            let _ = context
                                .app()
                                .reply_text(&update, format!("Nice to meet you, {text}!"))
                                .await?;
                            session.clear(&update).await?;
                        }
                        Some(other) => {
                            return Err(HandlerError::user(format!(
                                "unexpected dialog state `{other}`, send /cancel"
                            )));
                        }
                    }

                    Ok(())
                }
            });
    }

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

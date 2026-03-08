use std::env;
use std::time::Duration;

use tele::Client;
use tele::bot::{
    BotApp, BotContext, EngineConfig, EngineEvent, ErrorPolicy, OutboxConfig, Router, UpdateExt,
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

    let outbox_config = match env::var("TELEGRAM_OUTBOX_PATH") {
        Ok(path) => OutboxConfig::default().with_persistence_path(path),
        Err(_error) => OutboxConfig::default(),
    };
    let outbox = client.control().spawn_outbox(outbox_config);

    let mut router = Router::new();
    router.command_route("start").handle_with_policy(
        ErrorPolicy::ReplyUser {
            fallback_message: "failed to process /start".to_owned(),
        },
        |context: BotContext, update: Update| async move {
            let _ = context
                .app()
                .reply_text(&update, "tele quickstart bot is running")
                .await?;
            Ok(())
        },
    );

    router
        .text_route()
        .filter(|text, _update| !text.0.starts_with('/'))
        .handle(move |_context: BotContext, update: Update, text| {
            let outbox = outbox.clone();
            async move {
                let Some(chat_id) = update.chat_id() else {
                    return Ok(());
                };
                let idempotency_key = Some(format!("echo-{}", update.update_id));
                let _ = outbox
                    .send_text_with_key(
                        chat_id,
                        format!("echo: {}", text.into_inner()),
                        idempotency_key,
                    )
                    .await?;
                Ok(())
            }
        });

    let mut app = BotApp::long_polling(client, router)
        .with_engine_config(EngineConfig {
            idle_delay: Duration::from_millis(100),
            error_delay: Duration::from_millis(500),
            max_handler_concurrency: 16,
            ..EngineConfig::default()
        })
        .on_event(|event| {
            if let EngineEvent::PollFailed { .. } = event {
                eprintln!("bot poll failed: {event:?}");
            }
        });

    app.engine_mut()
        .source_mut()
        .config_mut()
        .poll_timeout_seconds = 20;
    app.run().await?;
    Ok(())
}

use std::env;

use serde::Deserialize;
use tele::Client;
use tele::Error;
use tele::MenuButtonConfig;
use tele::bot::{BotApp, BotContext, Router, WebAppInput};
use tele::types::telegram::{InlineQueryResult, WebAppInfo};
use tele::types::update::Update;

#[derive(Debug, Deserialize)]
struct MiniAppPayload {
    #[serde(default)]
    query_id: Option<String>,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    message_text: Option<String>,
}

fn read_env(name: &str) -> Result<String, Box<dyn std::error::Error>> {
    env::var(name).map_err(|error| format!("missing environment variable {name}: {error}").into())
}

fn inline_article_result(
    id: String,
    title: String,
    message_text: String,
) -> Result<InlineQueryResult, serde_json::Error> {
    InlineQueryResult::article(id, title, message_text)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = read_env("TELEGRAM_BOT_TOKEN")?;
    let chat_id = read_env("TELEGRAM_CHAT_ID")?
        .parse::<i64>()
        .map_err(|error| format!("invalid TELEGRAM_CHAT_ID: {error}"))?;
    let mini_app_url = read_env("TELEGRAM_MINI_APP_URL")?;

    let client = Client::builder("https://api.telegram.org")?
        .bot_token(token)?
        .build()?;

    let _ = client
        .ergo()
        .set_menu_button(MenuButtonConfig::for_chat_web_app(
            chat_id,
            "Open Mini App",
            WebAppInfo::new(mini_app_url),
        ))
        .await?;

    let mut router = Router::new();
    router.extracted_route::<WebAppInput>().handle(
        |context: BotContext, update: Update, web_app| async move {
            let web_app_data = web_app.into_inner();

            // Verify `initData` signature on your backend before trusting Mini App data.
            let parsed_payload = serde_json::from_str::<MiniAppPayload>(&web_app_data.data).ok();

            let echoed_text = parsed_payload
                .as_ref()
                .and_then(|payload| payload.message_text.clone())
                .unwrap_or_else(|| {
                    format!(
                        "Mini App payload from '{}': {}",
                        web_app_data.button_text, web_app_data.data
                    )
                });
            let _ = context.reply_text(&update, echoed_text).await?;
            let result_title = parsed_payload
                .as_ref()
                .and_then(|payload| payload.title.clone())
                .unwrap_or_else(|| "Mini App result".to_owned());

            if let Some(query_id) = parsed_payload.and_then(|payload| payload.query_id) {
                let result = inline_article_result(
                    format!("mini-app-{}", update.update_id),
                    result_title,
                    format!("Mini App query accepted: {query_id}"),
                )
                .map_err(|source| Error::InvalidRequest {
                    reason: format!("failed to serialize Mini App inline result: {source}"),
                })?;
                let _ = context
                    .answer_web_app_query_result(query_id, result)
                    .await?;
            }

            Ok(())
        },
    );

    let mut app = BotApp::long_polling(client, router);
    app.engine_mut()
        .source_mut()
        .config_mut()
        .poll_timeout_seconds = 20;
    app.run().await?;
    Ok(())
}

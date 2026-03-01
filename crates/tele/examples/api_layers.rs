use std::env;

use tele::Client;
use tele::types::advanced::AdvancedGetAvailableGiftsRequest;
use tele::types::bot::User;

fn read_env(name: &str) -> Result<String, Box<dyn std::error::Error>> {
    env::var(name).map_err(|error| format!("missing environment variable {name}: {error}").into())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = read_env("TELEGRAM_BOT_TOKEN")?;
    let chat_id_text = read_env("TELEGRAM_CHAT_ID")?;
    let chat_id: i64 = chat_id_text
        .parse()
        .map_err(|error| format!("invalid TELEGRAM_CHAT_ID `{chat_id_text}`: {error}"))?;
    let text = env::var("TELEGRAM_TEXT").unwrap_or_else(|_| "hello from tele layers".to_owned());

    let client = Client::builder("https://api.telegram.org")?
        .bot_token(token)?
        .build()?;

    let me: User = client.raw().call_no_params("getMe").await?;
    println!("raw: bot username = {:?}", me.username);

    let request = AdvancedGetAvailableGiftsRequest::new();
    let gifts: serde_json::Value = client.typed().call(&request).await?;
    println!(
        "typed: getAvailableGifts keys = {}",
        gifts.as_object().map_or(0, |obj| obj.len())
    );

    let sent = client.ergo().send_text(chat_id, text).await?;
    println!(
        "ergo: sent message id={} chat_id={}",
        sent.message_id.0, sent.chat.id
    );

    Ok(())
}

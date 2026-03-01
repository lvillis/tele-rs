use std::env;

use tele::Client;
use tele::types::SendMessageRequest;

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

    let text =
        env::var("TELEGRAM_TEXT").unwrap_or_else(|_| "hello from tele async example".to_owned());

    let client = Client::builder("https://api.telegram.org")?
        .bot_token(token)?
        .build()?;

    let request = SendMessageRequest::new(chat_id, text)?;
    let message = client.messages().send_message(&request).await?;

    println!(
        "sent message id={} chat_id={}",
        message.message_id.0, message.chat.id
    );
    Ok(())
}

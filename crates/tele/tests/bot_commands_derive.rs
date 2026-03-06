#![cfg(feature = "macros")]

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use tele::Client;
use tele::bot::{BotCommands, BotContext, Router, command_definitions, parse_typed_command};
use tele::types::update::Update;

type DynError = Box<dyn std::error::Error>;

#[derive(Debug, PartialEq, tele::BotCommands)]
enum DemoCommand {
    #[command(description = "start bot", alias = "run")]
    Start,
    #[command(description = "echo text", aliases("repeat", "say"))]
    Echo(String),
    #[command(rename = "count", description = "set numeric count")]
    Count(i64),
    #[command(description = "sum two numbers")]
    Add(i64, i64),
    #[command(description = "set reminder")]
    Remind { minutes: i64, text: String },
    #[command(description = "optional argument")]
    Maybe(Option<i64>),
}

fn parse_update(input: serde_json::Value) -> Option<Update> {
    serde_json::from_value(input).ok()
}

#[test]
fn derive_parses_commands_and_builds_descriptions() -> Result<(), DynError> {
    assert_eq!(DemoCommand::parse("start", ""), Some(DemoCommand::Start));
    assert_eq!(DemoCommand::parse("run", ""), Some(DemoCommand::Start));
    assert_eq!(DemoCommand::parse("start", "extra"), None);
    assert_eq!(
        DemoCommand::parse("echo", "hello world"),
        Some(DemoCommand::Echo("hello world".to_owned()))
    );
    assert_eq!(
        DemoCommand::parse("repeat", "\"hello world\""),
        Some(DemoCommand::Echo("hello world".to_owned()))
    );
    assert_eq!(
        DemoCommand::parse("say", "\"hello\\\"world\""),
        Some(DemoCommand::Echo("hello\"world".to_owned()))
    );
    assert_eq!(DemoCommand::parse("echo", "\"unterminated"), None);
    assert_eq!(
        DemoCommand::parse("count", "42"),
        Some(DemoCommand::Count(42))
    );
    assert_eq!(DemoCommand::parse("count", "oops"), None);
    assert_eq!(
        DemoCommand::parse("add", "2 3"),
        Some(DemoCommand::Add(2, 3))
    );
    assert_eq!(DemoCommand::parse("add", "2"), None);
    assert_eq!(
        DemoCommand::parse("remind", "15 \"team standup in 10 minutes\""),
        Some(DemoCommand::Remind {
            minutes: 15,
            text: "team standup in 10 minutes".to_owned()
        })
    );
    assert_eq!(
        DemoCommand::parse("maybe", ""),
        Some(DemoCommand::Maybe(None))
    );
    assert_eq!(
        DemoCommand::parse("maybe", "9"),
        Some(DemoCommand::Maybe(Some(9)))
    );

    let descriptions = DemoCommand::descriptions();
    assert_eq!(descriptions.len(), 6);
    assert_eq!(descriptions[0].command, "start");
    assert_eq!(descriptions[0].description, "start bot");

    let bot_commands = command_definitions::<DemoCommand>();
    assert_eq!(bot_commands.len(), 6);
    assert_eq!(bot_commands[1].command, "echo");
    assert_eq!(bot_commands[1].description, "echo text");
    Ok(())
}

#[tokio::test]
async fn typed_command_router_dispatches() -> Result<(), DynError> {
    let client = Client::builder("http://127.0.0.1:9")?
        .bot_token("123:abc")?
        .build()?;

    let hits = Arc::new(AtomicUsize::new(0));
    let mut router = Router::new();
    {
        let hits = Arc::clone(&hits);
        router.typed_command_route::<DemoCommand>().handle(
            move |_context: BotContext, _update: Update, command: DemoCommand| {
                let hits = Arc::clone(&hits);
                async move {
                    if matches!(command, DemoCommand::Echo(_)) {
                        hits.fetch_add(1, Ordering::SeqCst);
                    }
                    Ok(())
                }
            },
        );
    }

    let maybe_update = parse_update(serde_json::json!({
        "update_id": 600,
        "message": {
            "message_id": 1,
            "date": 1710000600,
            "chat": {"id": 1, "type": "private"},
            "text": "/echo hey"
        }
    }));
    assert!(maybe_update.is_some());
    let Some(update) = maybe_update else {
        return Ok(());
    };

    assert_eq!(
        parse_typed_command::<DemoCommand>(&update),
        Some(DemoCommand::Echo("hey".to_owned()))
    );

    let handled = router.dispatch(BotContext::new(client), update).await?;
    assert!(handled);
    assert_eq!(hits.load(Ordering::SeqCst), 1);

    Ok(())
}

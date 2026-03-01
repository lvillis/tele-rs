use tele::bot::BotCommands;
use tele_macros::BotCommands as DeriveBotCommands;

#[derive(Debug, PartialEq, DeriveBotCommands)]
enum DemoCommand {
    #[command(description = "start bot", alias = "run")]
    Start,
    #[command(description = "echo text", aliases("repeat", "say"))]
    Echo(String),
    #[command(description = "count value")]
    Count(i64),
    #[command(description = "optional integer")]
    Maybe(Option<i64>),
}

fn main() {
    assert_eq!(DemoCommand::parse("start", ""), Some(DemoCommand::Start));
    assert_eq!(DemoCommand::parse("run", ""), Some(DemoCommand::Start));
    assert_eq!(
        DemoCommand::parse("repeat", "\"hello world\""),
        Some(DemoCommand::Echo("hello world".to_owned()))
    );
    assert_eq!(
        DemoCommand::parse("count", "5"),
        Some(DemoCommand::Count(5))
    );
    assert_eq!(
        DemoCommand::parse("maybe", ""),
        Some(DemoCommand::Maybe(None))
    );
    let _ = DemoCommand::descriptions();
}

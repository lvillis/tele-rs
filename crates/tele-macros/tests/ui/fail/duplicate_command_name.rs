use tele_macros::BotCommands as DeriveBotCommands;

#[derive(DeriveBotCommands)]
enum DuplicateCommandName {
    #[command(rename = "start", description = "start bot")]
    Start,
    #[command(rename = "start", description = "also start")]
    AnotherStart,
}

fn main() {}

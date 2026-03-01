use tele_macros::BotCommands as DeriveBotCommands;

#[derive(DeriveBotCommands)]
enum InvalidCommandName {
    #[command(rename = "bad-name", description = "invalid command")]
    BadName,
}

fn main() {}

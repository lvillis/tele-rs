use tele_macros::BotCommands as DeriveBotCommands;

#[derive(DeriveBotCommands)]
enum NonTrailingOptional {
    #[command(description = "ambiguous optional")]
    Search(Option<i64>, String),
}

fn main() {}

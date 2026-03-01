use tele_macros::BotCommands as DeriveBotCommands;

#[derive(DeriveBotCommands)]
enum BorrowedField {
    #[command(description = "borrowed field")]
    Echo(&'static str),
}

fn main() {}

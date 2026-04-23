use tele_macros::BotCommands as DeriveBotCommands;

extern crate self as tele;

pub mod bot {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct CommandDescription {
        pub command: &'static str,
        pub description: &'static str,
    }

    pub trait BotCommands: Sized {
        fn parse(command: &str, args: &str) -> Option<Self>;
        fn descriptions() -> &'static [CommandDescription];
    }

    #[derive(Clone, Copy)]
    enum QuoteKind {
        Single,
        Double,
    }

    pub fn tokenize_command_args(args: &str) -> Option<Vec<String>> {
        let mut tokens = Vec::new();
        let mut current = String::new();
        let mut chars = args.chars().peekable();
        let mut quote = None;
        let mut token_started = false;

        while let Some(ch) = chars.next() {
            match quote {
                Some(QuoteKind::Single) => match ch {
                    '\'' => quote = None,
                    '\\' => {
                        let escaped = chars.next()?;
                        current.push(escaped);
                        token_started = true;
                    }
                    _ => {
                        current.push(ch);
                        token_started = true;
                    }
                },
                Some(QuoteKind::Double) => match ch {
                    '"' => quote = None,
                    '\\' => {
                        let escaped = chars.next()?;
                        current.push(escaped);
                        token_started = true;
                    }
                    _ => {
                        current.push(ch);
                        token_started = true;
                    }
                },
                None => match ch {
                    '\'' => {
                        quote = Some(QuoteKind::Single);
                        token_started = true;
                    }
                    '"' => {
                        quote = Some(QuoteKind::Double);
                        token_started = true;
                    }
                    '\\' => {
                        let escaped = chars.next()?;
                        current.push(escaped);
                        token_started = true;
                    }
                    _ if ch.is_whitespace() => {
                        if token_started {
                            tokens.push(std::mem::take(&mut current));
                            token_started = false;
                        }

                        while chars.peek().is_some_and(|next| next.is_whitespace()) {
                            let _ = chars.next();
                        }
                    }
                    _ => {
                        current.push(ch);
                        token_started = true;
                    }
                },
            }
        }

        if quote.is_some() {
            return None;
        }

        if token_started {
            tokens.push(current);
        }

        Some(tokens)
    }
}

use crate::bot::BotCommands;

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

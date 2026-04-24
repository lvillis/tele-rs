mod advanced;
mod spec;
mod sync_spec;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args();
    let _program = args.next();

    match args.next().as_deref() {
        Some("check-advanced") => advanced::check()?,
        Some("gen-advanced") => advanced::generate()?,
        Some("sync-spec") => sync_spec::sync(args.next())?,
        Some(other) => {
            return Err(format!(
                "unknown command `{other}`; expected `check-advanced`, `gen-advanced`, or `sync-spec`"
            )
            .into());
        }
        None => {
            return Err(
                "missing command; expected `check-advanced`, `gen-advanced`, or `sync-spec`".into(),
            );
        }
    }

    if let Some(extra) = args.next() {
        return Err(format!("unexpected extra argument `{extra}`").into());
    }

    Ok(())
}

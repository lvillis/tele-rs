# `0.1.8` Migration Notes

`0.1.8` closes several gaps that previously forced downstream workarounds in governance bots.

## Typed Telegram coverage added

The following are now modeled directly:

- `Message.forward_origin`
- `Message.is_automatic_forward`
- `Update.chat_join_request`
- `chat_join_request` extractor/route support
- `ChatPermissions::deny_all()` and `ChatPermissions::read_only()`

If you previously parsed these values from `extra`, move that logic into the typed fields and
delete the fallback adapter code.

## Runtime integration

`BotEngine::run_until(...)` and `BotApp::run_until(...)` are spawn-safe again. Standard Tokio
service patterns now work:

- `tokio::spawn(async move { app.run_until(shutdown).await })`
- bot + HTTP server + background jobs in the same runtime

See [`runtime-integration.md`](runtime-integration.md) for the recommended structure.

## Menu button API

Menu button ergonomics are now centered on `MenuButtonConfig` instead of forcing callers through
`AdvancedSetChatMenuButtonRequest`.

Prefer:

- `client.control().setup().set_menu_button(...)`

Instead of building raw advanced requests in application code.

`BootstrapPlan.menu_button(...)` also takes `MenuButtonConfig`.

## Web App reply ergonomics

Inside handlers, prefer the dedicated `WebAppApi` facade:

- `context.app().web_app().answer_query(...)`
- `context.app().web_app().answer_query_result(...)`
- `context.app().web_app().answer_query_from_payload(...)`

This keeps Web App code on one stable high-level layer instead of spreading it across context
helpers. Menu button setup should stay on `client.control().setup()`.

## Bootstrap diff/sync

Startup bootstrap now diff-checks commands and menu button state before applying changes.

This reduces:

- unnecessary startup writes
- command/menu churn in logs
- avoidable Bot API calls

## Recommended cleanup for downstreams

After upgrading, remove local patches that only existed to cover these gaps:

- manual `message.extra["forward_origin"]` parsing
- manual `message.extra["is_automatic_forward"]` parsing
- untyped join-request JSON parsing
- manual "mute means deny every permission" helpers
- raw advanced menu-button request builders in normal app code

# Kind Routing Guide

This guide documents how `tele` classifies updates/messages and how router APIs map to those kinds.

## UpdateKind vs MessageKind

- `UpdateKind`: top-level Telegram update payload (`message`, `callback_query`, `poll`, etc.).
- `MessageKind`: content kind inside a message-like payload (`text`, `photo`, `web_app_data`, etc.).

Use `UpdateKind` for transport/event routing, and `MessageKind` for content routing.

## Primary kind precedence

Both `Update::kind()` and `Message::kind()` use stable precedence and return one primary kind.

- `Update::kinds()` / `Message::kinds()` return all detected kinds.
- `Update::has_kind()` / `Message::has_kind()` are zero-allocation checks.

`Unknown` is emitted when:
- no modeled kind is present, or
- unmodeled payload keys are detected.

## Router semantics

Incoming-message only:

- `router.on_message(...)`
- `router.on_message_kind(...)`
- `router.on_incoming_message(...)`
- `router.on_incoming_message_kind(...)`

Message-like variants (includes edited/channel/callback message):

- `router.on_any_message(...)`
- `router.on_any_message_kind(...)`

Top-level update routing:

- `router.on_update_kind(...)`

## Observability for Unknown kinds

`BotEngine` emits `EngineEvent::UnknownKindsDetected` when either:

- `update.kind() == UpdateKind::Unknown`, or
- `extract_message_kind(update) == Some(MessageKind::Unknown)`.

Example:

```rust,no_run
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use tele::bot::{BotEngine, EngineEvent};

let unknown_hits = Arc::new(AtomicUsize::new(0));
let engine = BotEngine::new(client, source, router).on_event({
    let unknown_hits = Arc::clone(&unknown_hits);
    move |event| {
        if matches!(event, EngineEvent::UnknownKindsDetected { .. }) {
            unknown_hits.fetch_add(1, Ordering::Relaxed);
        }
    }
});
```

For production, wire this event into your metrics backend (`counter`, `log`, or `trace`).

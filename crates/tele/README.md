# tele

Telegram Bot API SDK for Rust.

Targets [Telegram Bot API 10.3](https://core.telegram.org/bots/api), with all 185 methods in the checked-in API specification. New methods are exposed through `client.advanced()` and its blocking equivalent.

Rich messages use `InputRichMessage::html`, `InputRichMessage::markdown`, or `InputRichMessage::blocks`. Use `AdvancedSendRichMessageRequest` with `send_rich_message_typed`, or `call_typed_with_files` for named `attach://` uploads. The same upload entry point supports live photos and ephemeral media edits.

Ephemeral sends use `EphemeralMessageParameters`; replies use `ReplyParameters::ephemeral`. The app reply helpers preserve the recipient and ephemeral reply identifier for supported message types. Subscription and stopped-generation events are available through `UpdateKind::Subscription` and `UpdateKind::StoppedMessageGeneration`.

Migration notes:

- `ReplyParameters.message_id` and `EditMessageTextRequest.text` are now optional; their existing constructors still accept regular message IDs and text. Rich edits have `for_chat_rich_message` and `for_inline_rich_message` constructors.
- `InlineQueryResultArticle.input_message_content` uses `InputMessageContent`, supporting both text and rich content. Convert an existing text content value with `.into()`.
- `MaybeInaccessibleMessage::Inaccessible` now stores a boxed value, matching the accessible variant. Its accessor methods retain their return types.
- Request structs have new optional fields. Existing constructor calls continue to supply defaults; update direct struct literals as needed.

## Moderation

Use `client.app().moderation()` (or `context.moderation()` inside a handler) for deletion, bans, and mutes. `ban_author` and `mute_author`, including their `_with` variants, require an identifiable user sender. They reject messages with `sender_chat` locally, including anonymous administrator posts and messages sent on behalf of a channel.

Use `message.sender_user()` for user identity and `message.sender_chat()` for chat identity. `message.from_user()` exposes the raw Telegram field, which can contain a compatibility user. Bot routing's `actor()` and `subject()` likewise return no user for chat-authored messages; a callback still identifies the user who clicked the button. This changes actor-based filtering and throttling for chat-authored messages: use a chat-scoped rule where appropriate.

Channel bans remain explicit through `client.chats().ban_chat_sender_chat(&request)`. User moderation helpers never choose that action automatically. Administrator exemptions, spam detection, and whether to ban a channel belong to the caller's policy.

Deletion and banning are independent requests. If your policy requires attempting both even when one fails, keep both results instead of using `?` between them:

```rust,ignore
let moderation = context.moderation();
let deletion = moderation.delete(message).await;
let ban = moderation.ban_author(message).await;
// Record or handle each result separately; neither operation rolls back the other.
```

`membership().bot_missing_capabilities(...)` can check installation permissions. Moderation methods do not issue hidden permission lookups before each action; callers should handle the actual API result even after a successful permission check. The blocking client follows the same semantics.

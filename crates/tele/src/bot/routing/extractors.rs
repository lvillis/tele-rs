use super::*;
use crate::types::message::Poll;
use crate::types::update::{
    BusinessConnection, BusinessMessagesDeleted, CallbackQuery, ChatBoostRemoved, ChatBoostUpdated,
    ChatJoinRequest, ChatMemberUpdated, ChosenInlineResult, InlineQuery, ManagedBotUpdated,
    MessageReactionCountUpdated, MessageReactionUpdated, PaidMediaPurchased, PollAnswer,
    PreCheckoutQuery, ShippingQuery,
};

/// Parsed slash command with command name and trailing arguments.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandData {
    pub name: String,
    pub mention: Option<String>,
    pub args: String,
}

impl CommandData {
    pub fn args_trimmed(&self) -> &str {
        self.args.trim()
    }

    pub fn has_args(&self) -> bool {
        !self.args_trimmed().is_empty()
    }

    pub fn target_mention(&self) -> Option<&str> {
        self.mention.as_deref()
    }

    pub fn is_addressed_to(&self, bot_username: Option<&str>) -> bool {
        command_is_addressed_to(self.mention.as_deref(), bot_username)
    }
}

/// Command declaration metadata used for typed command registration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CommandDescription {
    pub command: &'static str,
    pub description: &'static str,
}

/// Typed command parser contract. Intended for use with `#[derive(tele::BotCommands)]`.
pub trait BotCommands: Sized {
    fn parse(command: &str, args: &str) -> Option<Self>;
    fn descriptions() -> &'static [CommandDescription];

    fn parse_text(text: &str) -> Option<Self> {
        let command = parse_command_text(text)?;
        Self::parse(&command.name, command.args_trimmed())
    }
}

/// Route-level parser for a command's trailing argument string.
///
/// Built-in implementations distinguish required and optional semantics:
/// `String` requires non-empty trailing text, `Vec<String>` accepts zero or more shell-like tokens,
/// and `Option<T>` treats empty input as `None`.
pub trait CommandArgs: Sized {
    fn parse(args: &str) -> std::result::Result<Self, String>;
}

impl CommandArgs for String {
    fn parse(args: &str) -> std::result::Result<Self, String> {
        let args = args.trim();
        if args.is_empty() {
            return Err("missing required command argument".to_owned());
        }
        Ok(args.to_owned())
    }
}

impl CommandArgs for Vec<String> {
    fn parse(args: &str) -> std::result::Result<Self, String> {
        if args.trim().is_empty() {
            return Ok(Vec::new());
        }
        tokenize_command_args(args).ok_or_else(|| "invalid quoted command arguments".to_owned())
    }
}

impl<T> CommandArgs for Option<T>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    fn parse(args: &str) -> std::result::Result<Self, String> {
        let args = args.trim();
        if args.is_empty() {
            return Ok(None);
        }
        args.parse::<T>()
            .map(Some)
            .map_err(|source| format!("invalid optional command argument: {source}"))
    }
}

/// Typed extractor contract for business handlers.
pub trait UpdateExtractor: Sized {
    fn extract(update: &Update) -> Option<Self>;

    fn describe() -> &'static str {
        "required update payload"
    }
}

macro_rules! typed_update_input {
    ($(#[$meta:meta])* $input:ident($payload:ty), $extractor:ident, $description:literal) => {
        $(#[$meta])*
        #[derive(Clone, Debug)]
        pub struct $input(pub $payload);

        impl $input {
            pub fn into_inner(self) -> $payload {
                self.0
            }
        }

        impl UpdateExtractor for $input {
            fn extract(update: &Update) -> Option<Self> {
                $extractor(update).cloned().map(Self)
            }

            fn describe() -> &'static str {
                $description
            }
        }
    };
}

/// Plain text message extractor payload.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextInput(pub String);

impl TextInput {
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl UpdateExtractor for TextInput {
    fn extract(update: &Update) -> Option<Self> {
        extract_message_update_text(update).map(|text| Self(text.to_owned()))
    }

    fn describe() -> &'static str {
        "text message"
    }
}

/// Raw callback data extractor payload.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallbackInput(pub String);

impl CallbackInput {
    pub fn into_inner(self) -> String {
        self.0
    }
}

/// Codec-aware callback extractor payload with both decoded payload and raw data.
#[derive(Clone, Debug)]
pub struct CodedCallbackInput<T, C = CallbackPayloadCodec> {
    pub payload: T,
    pub raw: String,
    _codec: std::marker::PhantomData<C>,
}

impl<T, C> CodedCallbackInput<T, C>
where
    C: CallbackCodec<T>,
{
    pub fn from_raw(raw: impl Into<String>) -> Result<Self> {
        let raw = raw.into();
        let payload = C::decode_callback_data(raw.as_str())?;
        Ok(Self {
            payload,
            raw,
            _codec: std::marker::PhantomData,
        })
    }
}

/// Default typed callback extractor payload using [`CallbackPayload`].
pub type TypedCallbackInput<T> = CodedCallbackInput<T, CallbackPayloadCodec>;

/// Compact callback extractor payload using [`CompactCallbackCodec`].
pub type CompactCallbackInput<T> = CodedCallbackInput<T, CompactCallbackCodec>;

impl<T, C> UpdateExtractor for CodedCallbackInput<T, C>
where
    C: CallbackCodec<T>,
{
    fn extract(update: &Update) -> Option<Self> {
        let raw = extract_callback_data(update)?.to_owned();
        let payload = C::decode_callback_data(raw.as_str()).ok()?;
        Some(Self {
            payload,
            raw,
            _codec: std::marker::PhantomData,
        })
    }

    fn describe() -> &'static str {
        "callback payload"
    }
}

impl UpdateExtractor for CallbackInput {
    fn extract(update: &Update) -> Option<Self> {
        extract_callback_data(update).map(|data| Self(data.to_owned()))
    }

    fn describe() -> &'static str {
        "callback data"
    }
}

/// Mini App payload extractor payload.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WebAppInput(pub WebAppData);

impl WebAppInput {
    pub fn into_inner(self) -> WebAppData {
        self.0
    }

    pub fn parse_json<T>(&self) -> Result<T>
    where
        T: DeserializeOwned,
    {
        serde_json::from_str(&self.0.data).map_err(|source| Error::InvalidRequest {
            reason: format!("invalid web_app_data JSON payload: {source}"),
        })
    }

    pub fn parse_query_payload<T>(&self) -> Result<WebAppQueryPayload<T>>
    where
        T: DeserializeOwned,
    {
        WebAppQueryPayload::parse(&self.0)
    }
}

impl UpdateExtractor for WebAppInput {
    fn extract(update: &Update) -> Option<Self> {
        extract_message_update(update)?
            .web_app_data()
            .cloned()
            .map(Self)
    }

    fn describe() -> &'static str {
        "web app data"
    }
}

/// Write-access service payload extractor.
#[derive(Clone, Debug)]
pub struct WriteAccessAllowedInput(pub WriteAccessAllowed);

impl WriteAccessAllowedInput {
    pub fn into_inner(self) -> WriteAccessAllowed {
        self.0
    }
}

impl UpdateExtractor for WriteAccessAllowedInput {
    fn extract(update: &Update) -> Option<Self> {
        extract_message_update(update)?
            .write_access_allowed()
            .cloned()
            .map(Self)
    }

    fn describe() -> &'static str {
        "write access allowed"
    }
}

typed_update_input!(
    /// Callback query extractor payload.
    CallbackQueryInput(CallbackQuery),
    extract_callback_query,
    "callback query"
);

typed_update_input!(
    /// Inline query extractor payload.
    InlineQueryInput(InlineQuery),
    extract_inline_query,
    "inline query"
);

typed_update_input!(
    /// Chosen inline result extractor payload.
    ChosenInlineResultInput(ChosenInlineResult),
    extract_chosen_inline_result,
    "chosen inline result"
);

typed_update_input!(
    /// Business connection extractor payload.
    BusinessConnectionInput(BusinessConnection),
    extract_business_connection,
    "business connection"
);

/// Chat join request extractor payload.
#[derive(Clone, Debug)]
pub struct ChatJoinRequestInput(pub ChatJoinRequest);

impl ChatJoinRequestInput {
    pub fn into_inner(self) -> ChatJoinRequest {
        self.0
    }
}

impl UpdateExtractor for ChatJoinRequestInput {
    fn extract(update: &Update) -> Option<Self> {
        extract_chat_join_request(update).cloned().map(Self)
    }

    fn describe() -> &'static str {
        "chat join request"
    }
}

/// Chat member update extractor payload.
#[derive(Clone, Debug)]
pub struct ChatMemberUpdatedInput(pub ChatMemberUpdated);

impl ChatMemberUpdatedInput {
    pub fn into_inner(self) -> ChatMemberUpdated {
        self.0
    }
}

impl UpdateExtractor for ChatMemberUpdatedInput {
    fn extract(update: &Update) -> Option<Self> {
        extract_chat_member_update(update).cloned().map(Self)
    }

    fn describe() -> &'static str {
        "chat member update"
    }
}

/// Bot membership update extractor payload.
#[derive(Clone, Debug)]
pub struct MyChatMemberUpdatedInput(pub ChatMemberUpdated);

impl MyChatMemberUpdatedInput {
    pub fn into_inner(self) -> ChatMemberUpdated {
        self.0
    }
}

impl UpdateExtractor for MyChatMemberUpdatedInput {
    fn extract(update: &Update) -> Option<Self> {
        extract_my_chat_member_update(update).cloned().map(Self)
    }

    fn describe() -> &'static str {
        "my chat member update"
    }
}

typed_update_input!(
    /// Business-message deletion extractor payload.
    BusinessMessagesDeletedInput(BusinessMessagesDeleted),
    extract_deleted_business_messages,
    "deleted business messages"
);

typed_update_input!(
    /// Message reaction extractor payload.
    MessageReactionInput(MessageReactionUpdated),
    extract_message_reaction,
    "message reaction"
);

typed_update_input!(
    /// Aggregate message reaction-count extractor payload.
    MessageReactionCountInput(MessageReactionCountUpdated),
    extract_message_reaction_count,
    "message reaction count"
);

typed_update_input!(
    /// Shipping query extractor payload.
    ShippingQueryInput(ShippingQuery),
    extract_shipping_query,
    "shipping query"
);

typed_update_input!(
    /// Pre-checkout query extractor payload.
    PreCheckoutQueryInput(PreCheckoutQuery),
    extract_pre_checkout_query,
    "pre-checkout query"
);

typed_update_input!(
    /// Paid media purchase extractor payload.
    PaidMediaPurchasedInput(PaidMediaPurchased),
    extract_purchased_paid_media,
    "paid media purchase"
);

typed_update_input!(
    /// Poll extractor payload.
    PollInput(Poll),
    extract_poll,
    "poll"
);

typed_update_input!(
    /// Poll answer extractor payload.
    PollAnswerInput(PollAnswer),
    extract_poll_answer,
    "poll answer"
);

typed_update_input!(
    /// Chat boost extractor payload.
    ChatBoostInput(ChatBoostUpdated),
    extract_chat_boost,
    "chat boost"
);

typed_update_input!(
    /// Removed chat boost extractor payload.
    ChatBoostRemovedInput(ChatBoostRemoved),
    extract_removed_chat_boost,
    "removed chat boost"
);

typed_update_input!(
    /// Managed bot extractor payload.
    ManagedBotInput(ManagedBotUpdated),
    extract_managed_bot,
    "managed bot"
);

/// JSON-decoded callback extractor payload.
#[derive(Clone, Debug)]
pub struct JsonCallback<T>(pub T);

impl<T> JsonCallback<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> UpdateExtractor for JsonCallback<T>
where
    T: DeserializeOwned,
{
    fn extract(update: &Update) -> Option<Self> {
        extract_callback_json(update).map(Self)
    }

    fn describe() -> &'static str {
        "json callback payload"
    }
}

/// Strongly-typed command extractor payload.
#[derive(Clone, Debug)]
pub struct TypedCommandInput<C> {
    pub command: C,
    pub raw: CommandData,
}

impl<C> UpdateExtractor for TypedCommandInput<C>
where
    C: BotCommands,
{
    fn extract(update: &Update) -> Option<Self> {
        let raw = extract_command_data(update)?;
        let command = C::parse(&raw.name, raw.args_trimmed())?;
        Some(Self { command, raw })
    }

    fn describe() -> &'static str {
        "typed command"
    }
}

/// Parses a slash command from raw message text.
pub fn parse_command_text(text: &str) -> Option<CommandData> {
    parse_command_text_for_bot(text, None)
}

struct ParsedCommandPrefix<'a> {
    name: &'a str,
    mention: Option<String>,
    args: &'a str,
}

fn parse_command_prefix(text: &str) -> Option<ParsedCommandPrefix<'_>> {
    let text = text.trim_start();
    let token = text.split_whitespace().next()?;
    let (name, mention) = parse_command_token(token)?;
    let args = text[token.len()..].trim();

    Some(ParsedCommandPrefix {
        name,
        mention,
        args,
    })
}

fn command_is_addressed_to(mention: Option<&str>, bot_username: Option<&str>) -> bool {
    let Some(mention) = mention else {
        return true;
    };
    let Some(bot_username) = bot_username else {
        return false;
    };
    let Some(expected) = normalize_bot_username(bot_username) else {
        return false;
    };
    mention.eq_ignore_ascii_case(expected.as_str())
}

/// Parses a slash command from raw message text with optional bot-username targeting.
///
/// When a command contains `@botname`, it is accepted only if `bot_username`
/// is provided and matches case-insensitively. Bot usernames must follow
/// Telegram bot mention rules: 5-32 ASCII letters, digits, or underscores, and
/// an ending `bot` suffix.
pub fn parse_command_text_for_bot(text: &str, bot_username: Option<&str>) -> Option<CommandData> {
    let parsed = parse_command_prefix(text)?;
    if !command_is_addressed_to(parsed.mention.as_deref(), bot_username) {
        return None;
    }

    Some(CommandData {
        name: parsed.name.to_owned(),
        mention: parsed.mention,
        args: parsed.args.to_owned(),
    })
}

pub(crate) fn normalize_bot_username(value: &str) -> Option<String> {
    let value = value.trim();
    let normalized = value.strip_prefix('@').unwrap_or(value);
    if normalized.is_empty()
        || normalized.trim() != normalized
        || normalized.starts_with('@')
        || !is_valid_bot_username(normalized)
    {
        None
    } else {
        Some(normalized.to_owned())
    }
}

pub(crate) fn parse_command_token(token: &str) -> Option<(&str, Option<String>)> {
    let command = token.strip_prefix('/')?;
    let (name, mention) = match command.split_once('@') {
        Some((_, mention)) if mention.starts_with('@') => return None,
        Some((name, mention)) => (name, Some(normalize_bot_username(mention)?)),
        None => (command, None),
    };

    if is_valid_command_name(name) {
        Some((name, mention))
    } else {
        None
    }
}

pub(crate) fn validate_route_command_name(command: &str) -> Result<String> {
    if command.starts_with('/') || command.contains('@') {
        return Err(invalid_request(
            "command route name must be the bare command without `/` or `@bot`",
        ));
    }
    if !is_valid_command_name(command) {
        return Err(invalid_request(format!(
            "invalid command route name `{command}`: expected 1-32 lowercase ASCII letters, digits, or `_`"
        )));
    }

    Ok(command.to_owned())
}

fn is_valid_command_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 32
        && name
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_')
}

fn is_valid_bot_username(username: &str) -> bool {
    (5..=32).contains(&username.len())
        && username
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
        && username.to_ascii_lowercase().ends_with("bot")
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum QuoteKind {
    Single,
    Double,
}

/// Tokenizes command arguments with quote and escape support.
///
/// Accepts single (`'...'`) and double (`"..."`) quotes and backslash escapes (`\`).
/// Returns `None` when input contains an unterminated quote or a dangling escape.
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

/// Returns canonical message object from update, prioritizing incoming message variants.
pub fn extract_message(update: &Update) -> Option<&Message> {
    if let Some(message) = extract_message_update(update) {
        return Some(message);
    }

    update
        .callback_query
        .as_ref()
        .and_then(|query| query.message.as_deref())
        .and_then(|message| message.accessible())
}

pub(super) fn extract_message_update(update: &Update) -> Option<&Message> {
    if let Some(message) = update.message.as_deref() {
        return Some(message);
    }
    if let Some(message) = update.edited_message.as_deref() {
        return Some(message);
    }
    if let Some(message) = update.channel_post.as_deref() {
        return Some(message);
    }
    if let Some(message) = update.edited_channel_post.as_deref() {
        return Some(message);
    }
    if let Some(message) = update.business_message.as_deref() {
        return Some(message);
    }
    if let Some(message) = update.edited_business_message.as_deref() {
        return Some(message);
    }
    if let Some(message) = update.guest_message.as_deref() {
        return Some(message);
    }

    None
}

pub(super) fn extract_message_update_text(update: &Update) -> Option<&str> {
    extract_message_update(update)?.text.as_deref()
}

/// Returns canonical chat extracted from the update.
pub fn extract_chat(update: &Update) -> Option<&Chat> {
    if let Some(message) = extract_message(update) {
        return Some(message.chat());
    }
    if let Some(message) = update
        .callback_query
        .as_ref()
        .and_then(|query| query.message.as_deref())
    {
        return Some(message.chat());
    }
    if let Some(reaction) = update.message_reaction.as_ref() {
        return Some(&reaction.chat);
    }
    if let Some(reaction_count) = update.message_reaction_count.as_ref() {
        return Some(&reaction_count.chat);
    }
    if let Some(member_update) = extract_chat_member_update(update) {
        return Some(&member_update.chat);
    }
    if let Some(member_update) = extract_my_chat_member_update(update) {
        return Some(&member_update.chat);
    }
    if let Some(deleted) = update.deleted_business_messages.as_ref() {
        return Some(&deleted.chat);
    }
    if let Some(voter_chat) = update
        .poll_answer
        .as_ref()
        .and_then(|answer| answer.voter_chat.as_ref())
    {
        return Some(voter_chat);
    }
    if let Some(boost) = update.chat_boost.as_ref() {
        return Some(&boost.chat);
    }
    if let Some(boost) = update.removed_chat_boost.as_ref() {
        return Some(&boost.chat);
    }

    extract_chat_join_request(update).map(|request| &request.chat)
}

/// Returns the actor that caused this update when available.
pub fn extract_actor(update: &Update) -> Option<&User> {
    if let Some(connection) = update.business_connection.as_ref() {
        return Some(&connection.user);
    }
    if let Some(query) = update.callback_query.as_ref() {
        return Some(&query.from);
    }
    if let Some(query) = update.inline_query.as_ref() {
        return Some(&query.from);
    }
    if let Some(result) = update.chosen_inline_result.as_ref() {
        return Some(&result.from);
    }
    if let Some(query) = update.shipping_query.as_ref() {
        return Some(&query.from);
    }
    if let Some(query) = update.pre_checkout_query.as_ref() {
        return Some(&query.from);
    }
    if let Some(purchase) = update.purchased_paid_media.as_ref() {
        return Some(&purchase.from);
    }
    if let Some(reaction) = update.message_reaction.as_ref()
        && let Some(user) = reaction.user.as_ref()
    {
        return Some(user);
    }
    if let Some(boost) = update
        .chat_boost
        .as_ref()
        .and_then(|boost| boost.boost.source.user.as_ref())
    {
        return Some(boost);
    }
    if let Some(boost) = update
        .removed_chat_boost
        .as_ref()
        .and_then(|boost| boost.source.user.as_ref())
    {
        return Some(boost);
    }
    if let Some(managed_bot) = update.managed_bot.as_ref() {
        return Some(&managed_bot.user);
    }
    if let Some(answer) = update.poll_answer.as_ref()
        && let Some(user) = answer.user.as_ref()
    {
        return Some(user);
    }
    if let Some(message) = update.message.as_deref() {
        return message.from_user();
    }
    if let Some(message) = update.edited_message.as_deref() {
        return message.from_user();
    }
    if let Some(message) = update.channel_post.as_deref() {
        return message.from_user();
    }
    if let Some(message) = update.edited_channel_post.as_deref() {
        return message.from_user();
    }
    if let Some(message) = update.business_message.as_deref() {
        return message.from_user();
    }
    if let Some(message) = update.edited_business_message.as_deref() {
        return message.from_user();
    }
    if let Some(message) = update.guest_message.as_deref() {
        return message.from_user();
    }
    if let Some(request) = update.chat_join_request.as_ref() {
        return Some(&request.from);
    }
    if let Some(member_update) = update.chat_member.as_ref() {
        return Some(&member_update.from);
    }
    if let Some(member_update) = update.my_chat_member.as_ref() {
        return Some(&member_update.from);
    }
    None
}

/// Returns the subject user this update is primarily about when available.
pub fn extract_subject(update: &Update) -> Option<&User> {
    if let Some(member_update) = update.chat_member.as_ref() {
        return member_update.subject();
    }
    if let Some(member_update) = update.my_chat_member.as_ref() {
        return member_update.subject();
    }
    if let Some(managed_bot) = update.managed_bot.as_ref() {
        return Some(&managed_bot.bot);
    }

    extract_actor(update)
}

/// Backward-compatible alias for `extract_actor`.
pub fn extract_user(update: &Update) -> Option<&User> {
    extract_actor(update)
}

/// Returns actor id for the update when available.
pub fn extract_actor_id(update: &Update) -> Option<i64> {
    Some(extract_actor(update)?.id.0)
}

/// Returns subject user id for the update when available.
pub fn extract_subject_id(update: &Update) -> Option<i64> {
    Some(extract_subject(update)?.id.0)
}

/// Backward-compatible alias for `extract_actor_id`.
pub fn extract_user_id(update: &Update) -> Option<i64> {
    extract_actor_id(update)
}

/// Returns primary kind of extracted message.
pub fn extract_message_kind(update: &Update) -> Option<MessageKind> {
    Some(extract_message(update)?.kind())
}

/// Returns plain text from extracted message when available.
pub fn extract_text(update: &Update) -> Option<&str> {
    extract_message(update)?.text.as_deref()
}

/// Returns Mini App payload from extracted message when available.
pub fn extract_web_app_data(update: &Update) -> Option<&WebAppData> {
    update.web_app_data()
}

/// Returns write-access service payload from extracted message when available.
pub fn extract_write_access_allowed(update: &Update) -> Option<&WriteAccessAllowed> {
    extract_message(update)?.write_access_allowed()
}

/// Returns typed callback query payload from update.
pub fn extract_callback_query(update: &Update) -> Option<&CallbackQuery> {
    update.callback_query.as_ref()
}

/// Returns typed inline query payload from update.
pub fn extract_inline_query(update: &Update) -> Option<&InlineQuery> {
    update.inline_query.as_ref()
}

/// Returns typed chosen inline result payload from update.
pub fn extract_chosen_inline_result(update: &Update) -> Option<&ChosenInlineResult> {
    update.chosen_inline_result.as_ref()
}

/// Returns typed business connection payload from update.
pub fn extract_business_connection(update: &Update) -> Option<&BusinessConnection> {
    update.business_connection()
}

/// Returns typed chat join request payload from update.
pub fn extract_chat_join_request(update: &Update) -> Option<&ChatJoinRequest> {
    update.chat_join_request()
}

/// Returns typed chat member update payload from update.
pub fn extract_chat_member_update(update: &Update) -> Option<&ChatMemberUpdated> {
    update.chat_member()
}

/// Returns typed current-bot member update payload from update.
pub fn extract_my_chat_member_update(update: &Update) -> Option<&ChatMemberUpdated> {
    update.my_chat_member()
}

/// Returns typed business-message deletion payload from update.
pub fn extract_deleted_business_messages(update: &Update) -> Option<&BusinessMessagesDeleted> {
    update.deleted_business_messages()
}

/// Returns typed message reaction payload from update.
pub fn extract_message_reaction(update: &Update) -> Option<&MessageReactionUpdated> {
    update.message_reaction()
}

/// Returns typed aggregate message reaction-count payload from update.
pub fn extract_message_reaction_count(update: &Update) -> Option<&MessageReactionCountUpdated> {
    update.message_reaction_count()
}

/// Returns typed shipping query payload from update.
pub fn extract_shipping_query(update: &Update) -> Option<&ShippingQuery> {
    update.shipping_query()
}

/// Returns typed pre-checkout query payload from update.
pub fn extract_pre_checkout_query(update: &Update) -> Option<&PreCheckoutQuery> {
    update.pre_checkout_query()
}

/// Returns typed paid media purchase payload from update.
pub fn extract_purchased_paid_media(update: &Update) -> Option<&PaidMediaPurchased> {
    update.purchased_paid_media()
}

/// Returns typed poll payload from update.
pub fn extract_poll(update: &Update) -> Option<&Poll> {
    update.poll.as_deref()
}

/// Returns typed poll answer payload from update.
pub fn extract_poll_answer(update: &Update) -> Option<&PollAnswer> {
    update.poll_answer.as_ref()
}

/// Returns typed chat boost payload from update.
pub fn extract_chat_boost(update: &Update) -> Option<&ChatBoostUpdated> {
    update.chat_boost()
}

/// Returns typed removed chat boost payload from update.
pub fn extract_removed_chat_boost(update: &Update) -> Option<&ChatBoostRemoved> {
    update.removed_chat_boost()
}

/// Returns typed managed bot payload from update.
pub fn extract_managed_bot(update: &Update) -> Option<&ManagedBotUpdated> {
    update.managed_bot()
}

/// Returns callback query data payload from update.
pub fn extract_callback_data(update: &Update) -> Option<&str> {
    update.callback_query.as_ref()?.data.as_deref()
}

/// Returns JSON-decoded callback payload from update.
pub fn extract_callback_json<T>(update: &Update) -> Option<T>
where
    T: DeserializeOwned,
{
    let payload = extract_callback_data(update)?;
    serde_json::from_str(payload).ok()
}

/// Returns a decoded typed callback payload from update callback data.
pub fn extract_typed_callback<T>(update: &Update) -> Option<T>
where
    T: CallbackPayload,
{
    extract_callback_with_codec::<T, CallbackPayloadCodec>(update)
}

/// Returns a decoded callback payload from update callback data with an explicit codec.
pub fn extract_callback_with_codec<T, C>(update: &Update) -> Option<T>
where
    C: CallbackCodec<T>,
{
    let payload = extract_callback_data(update)?;
    C::decode_callback_data(payload).ok()
}

/// Returns a decoded compact callback payload from update callback data.
pub fn extract_compact_callback<T>(update: &Update) -> Option<T>
where
    T: CompactCallbackPayload,
{
    extract_callback_with_codec::<T, CompactCallbackCodec>(update)
}

/// Returns command name from canonical message text.
///
/// Mentioned commands (for example, `/start@OtherBot`) are ignored by default.
pub fn extract_command(update: &Update) -> Option<&str> {
    extract_command_for_bot(update, None)
}

/// Returns command name from canonical message text, filtered by target bot username.
pub fn extract_command_for_bot<'a>(
    update: &'a Update,
    bot_username: Option<&str>,
) -> Option<&'a str> {
    let parsed = parse_command_prefix(extract_message_update_text(update)?)?;
    command_is_addressed_to(parsed.mention.as_deref(), bot_username).then_some(parsed.name)
}

/// Returns command arguments as a trimmed string slice.
pub fn extract_command_args(update: &Update) -> Option<&str> {
    extract_command_args_for_bot(update, None)
}

/// Returns command arguments as a trimmed string slice, filtered by target bot username.
pub fn extract_command_args_for_bot<'a>(
    update: &'a Update,
    bot_username: Option<&str>,
) -> Option<&'a str> {
    let parsed = parse_command_prefix(extract_message_update_text(update)?)?;
    command_is_addressed_to(parsed.mention.as_deref(), bot_username).then_some(parsed.args)
}

/// Returns parsed command with owned command name and args.
pub fn extract_command_data(update: &Update) -> Option<CommandData> {
    extract_command_data_for_bot(update, None)
}

/// Returns parsed command with owned command name and args, filtered by target bot username.
pub fn extract_command_data_for_bot(
    update: &Update,
    bot_username: Option<&str>,
) -> Option<CommandData> {
    parse_command_text_for_bot(extract_message_update_text(update)?, bot_username)
}

/// Parses typed command from incoming update using a `BotCommands` implementation.
pub fn parse_typed_command<C: BotCommands>(update: &Update) -> Option<C> {
    let command = extract_command_data_for_bot(update, None)?;
    C::parse(&command.name, command.args_trimmed())
}

/// Parses typed command from incoming update for an explicit bot username target.
pub fn parse_typed_command_for_bot<C: BotCommands>(
    update: &Update,
    bot_username: Option<&str>,
) -> Option<C> {
    let command = extract_command_data_for_bot(update, bot_username)?;
    C::parse(&command.name, command.args_trimmed())
}

/// Builds Telegram command descriptors from a typed command enum.
pub fn command_definitions<C: BotCommands>() -> Vec<crate::types::command::BotCommand> {
    C::descriptions()
        .iter()
        .map(|description| crate::types::command::BotCommand {
            command: description.command.to_owned(),
            description: description.description.to_owned(),
            extra: std::collections::BTreeMap::new(),
        })
        .collect()
}

/// Convenience extractor trait for update handlers.
pub trait UpdateExt {
    fn message(&self) -> Option<&Message>;
    fn chat(&self) -> Option<&Chat> {
        self.message().map(Message::chat)
    }
    fn message_kind(&self) -> Option<MessageKind> {
        self.message().map(Message::kind)
    }
    fn update_kind(&self) -> UpdateKind {
        UpdateKind::Unknown
    }
    fn text(&self) -> Option<&str>;
    fn web_app_data(&self) -> Option<&WebAppData> {
        None
    }
    fn write_access_allowed(&self) -> Option<&WriteAccessAllowed> {
        None
    }
    fn callback_query(&self) -> Option<&CallbackQuery> {
        None
    }
    fn inline_query(&self) -> Option<&InlineQuery> {
        None
    }
    fn chosen_inline_result(&self) -> Option<&ChosenInlineResult> {
        None
    }
    fn business_connection(&self) -> Option<&BusinessConnection> {
        None
    }
    fn chat_join_request(&self) -> Option<&ChatJoinRequest> {
        None
    }
    fn chat_member_update(&self) -> Option<&ChatMemberUpdated> {
        None
    }
    fn my_chat_member_update(&self) -> Option<&ChatMemberUpdated> {
        None
    }
    fn deleted_business_messages(&self) -> Option<&BusinessMessagesDeleted> {
        None
    }
    fn message_reaction(&self) -> Option<&MessageReactionUpdated> {
        None
    }
    fn message_reaction_count(&self) -> Option<&MessageReactionCountUpdated> {
        None
    }
    fn shipping_query(&self) -> Option<&ShippingQuery> {
        None
    }
    fn pre_checkout_query(&self) -> Option<&PreCheckoutQuery> {
        None
    }
    fn purchased_paid_media(&self) -> Option<&PaidMediaPurchased> {
        None
    }
    fn poll(&self) -> Option<&Poll> {
        None
    }
    fn poll_answer(&self) -> Option<&PollAnswer> {
        None
    }
    fn chat_boost(&self) -> Option<&ChatBoostUpdated> {
        None
    }
    fn removed_chat_boost(&self) -> Option<&ChatBoostRemoved> {
        None
    }
    fn managed_bot(&self) -> Option<&ManagedBotUpdated> {
        None
    }
    fn callback_data(&self) -> Option<&str>;
    fn callback_json<T>(&self) -> Option<T>
    where
        T: DeserializeOwned;
    fn typed_callback<T>(&self) -> Option<T>
    where
        T: CallbackPayload;
    fn typed_callback_with_codec<T, C>(&self) -> Option<T>
    where
        C: CallbackCodec<T>;
    fn compact_callback<T>(&self) -> Option<T>
    where
        T: CompactCallbackPayload;
    fn command(&self) -> Option<&str>;
    fn command_args(&self) -> Option<&str>;
    fn command_data(&self) -> Option<CommandData>;
    fn typed_command<C>(&self) -> Option<C>
    where
        C: BotCommands;
    fn actor(&self) -> Option<&User>;
    fn actor_id(&self) -> Option<i64> {
        self.actor().map(|user| user.id.0)
    }
    fn subject(&self) -> Option<&User>;
    fn subject_id(&self) -> Option<i64> {
        self.subject().map(|user| user.id.0)
    }
    fn user(&self) -> Option<&User>;
    fn user_id(&self) -> Option<i64> {
        self.actor_id()
    }
    fn chat_id(&self) -> Option<i64>;
}

impl UpdateExt for Update {
    fn message(&self) -> Option<&Message> {
        extract_message(self)
    }

    fn chat(&self) -> Option<&Chat> {
        extract_chat(self)
    }

    fn message_kind(&self) -> Option<MessageKind> {
        extract_message_kind(self)
    }

    fn update_kind(&self) -> UpdateKind {
        Update::kind(self)
    }

    fn text(&self) -> Option<&str> {
        extract_text(self)
    }

    fn web_app_data(&self) -> Option<&WebAppData> {
        Update::web_app_data(self)
    }

    fn write_access_allowed(&self) -> Option<&WriteAccessAllowed> {
        extract_write_access_allowed(self)
    }

    fn callback_query(&self) -> Option<&CallbackQuery> {
        extract_callback_query(self)
    }

    fn inline_query(&self) -> Option<&InlineQuery> {
        extract_inline_query(self)
    }

    fn chosen_inline_result(&self) -> Option<&ChosenInlineResult> {
        extract_chosen_inline_result(self)
    }

    fn business_connection(&self) -> Option<&BusinessConnection> {
        extract_business_connection(self)
    }

    fn chat_join_request(&self) -> Option<&ChatJoinRequest> {
        extract_chat_join_request(self)
    }

    fn chat_member_update(&self) -> Option<&ChatMemberUpdated> {
        extract_chat_member_update(self)
    }

    fn my_chat_member_update(&self) -> Option<&ChatMemberUpdated> {
        extract_my_chat_member_update(self)
    }

    fn deleted_business_messages(&self) -> Option<&BusinessMessagesDeleted> {
        extract_deleted_business_messages(self)
    }

    fn message_reaction(&self) -> Option<&MessageReactionUpdated> {
        extract_message_reaction(self)
    }

    fn message_reaction_count(&self) -> Option<&MessageReactionCountUpdated> {
        extract_message_reaction_count(self)
    }

    fn shipping_query(&self) -> Option<&ShippingQuery> {
        extract_shipping_query(self)
    }

    fn pre_checkout_query(&self) -> Option<&PreCheckoutQuery> {
        extract_pre_checkout_query(self)
    }

    fn purchased_paid_media(&self) -> Option<&PaidMediaPurchased> {
        extract_purchased_paid_media(self)
    }

    fn poll(&self) -> Option<&Poll> {
        extract_poll(self)
    }

    fn poll_answer(&self) -> Option<&PollAnswer> {
        extract_poll_answer(self)
    }

    fn chat_boost(&self) -> Option<&ChatBoostUpdated> {
        extract_chat_boost(self)
    }

    fn removed_chat_boost(&self) -> Option<&ChatBoostRemoved> {
        extract_removed_chat_boost(self)
    }

    fn managed_bot(&self) -> Option<&ManagedBotUpdated> {
        extract_managed_bot(self)
    }

    fn callback_data(&self) -> Option<&str> {
        extract_callback_data(self)
    }

    fn callback_json<T>(&self) -> Option<T>
    where
        T: DeserializeOwned,
    {
        extract_callback_json(self)
    }

    fn typed_callback<T>(&self) -> Option<T>
    where
        T: CallbackPayload,
    {
        extract_typed_callback(self)
    }

    fn typed_callback_with_codec<T, C>(&self) -> Option<T>
    where
        C: CallbackCodec<T>,
    {
        extract_callback_with_codec::<T, C>(self)
    }

    fn compact_callback<T>(&self) -> Option<T>
    where
        T: CompactCallbackPayload,
    {
        extract_compact_callback(self)
    }

    fn command(&self) -> Option<&str> {
        extract_command(self)
    }

    fn command_args(&self) -> Option<&str> {
        extract_command_args(self)
    }

    fn command_data(&self) -> Option<CommandData> {
        extract_command_data(self)
    }

    fn typed_command<C>(&self) -> Option<C>
    where
        C: BotCommands,
    {
        parse_typed_command(self)
    }

    fn actor(&self) -> Option<&User> {
        extract_actor(self)
    }

    fn subject(&self) -> Option<&User> {
        extract_subject(self)
    }

    fn user(&self) -> Option<&User> {
        self.actor()
    }

    fn chat_id(&self) -> Option<i64> {
        update_chat_id(self)
    }
}

/// Tries to extract a canonical chat id from an incoming update.
pub fn update_chat_id(update: &Update) -> Option<i64> {
    Some(extract_chat(update)?.id)
}

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use crate::types::bot::User;
use crate::types::chat::{ChatInviteLink, ChatMember};
use crate::types::common::MessageId;
use crate::types::message::{
    Chat, Location, MaybeInaccessibleMessage, Message, OrderInfo, Poll, ShippingAddress,
};
use crate::types::telegram::{
    InlineQueryResult, InlineQueryResultsButton, ReactionType, WebAppData,
};
use crate::types::validation::{
    control_free_string as validate_control_free_string, http_url as validate_http_url,
};
use crate::{Error, Result};

const MAX_GET_UPDATES_LIMIT: u8 = 100;
const MAX_CALLBACK_ANSWER_TEXT_CHARS: usize = 200;
const MAX_INLINE_QUERY_RESULTS: usize = 50;
const MAX_INLINE_NEXT_OFFSET_BYTES: usize = 64;

/// Classified update payload kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[non_exhaustive]
pub enum UpdateKind {
    Message,
    EditedMessage,
    ChannelPost,
    EditedChannelPost,
    BusinessConnection,
    BusinessMessage,
    EditedBusinessMessage,
    DeletedBusinessMessages,
    GuestMessage,
    MessageReaction,
    MessageReactionCount,
    CallbackQuery,
    InlineQuery,
    ChosenInlineResult,
    ShippingQuery,
    PreCheckoutQuery,
    PurchasedPaidMedia,
    Poll,
    PollAnswer,
    MyChatMember,
    ChatMember,
    ChatJoinRequest,
    ChatBoost,
    RemovedChatBoost,
    ManagedBot,
    Unknown,
}

const KNOWN_UPDATE_KINDS: [UpdateKind; 25] = [
    UpdateKind::Message,
    UpdateKind::EditedMessage,
    UpdateKind::ChannelPost,
    UpdateKind::EditedChannelPost,
    UpdateKind::BusinessConnection,
    UpdateKind::BusinessMessage,
    UpdateKind::EditedBusinessMessage,
    UpdateKind::DeletedBusinessMessages,
    UpdateKind::GuestMessage,
    UpdateKind::MessageReaction,
    UpdateKind::MessageReactionCount,
    UpdateKind::CallbackQuery,
    UpdateKind::InlineQuery,
    UpdateKind::ChosenInlineResult,
    UpdateKind::ShippingQuery,
    UpdateKind::PreCheckoutQuery,
    UpdateKind::PurchasedPaidMedia,
    UpdateKind::Poll,
    UpdateKind::PollAnswer,
    UpdateKind::MyChatMember,
    UpdateKind::ChatMember,
    UpdateKind::ChatJoinRequest,
    UpdateKind::ChatBoost,
    UpdateKind::RemovedChatBoost,
    UpdateKind::ManagedBot,
];

impl UpdateKind {
    /// Returns every modeled Telegram update kind except [`UpdateKind::Unknown`].
    pub const fn all_known() -> &'static [Self] {
        &KNOWN_UPDATE_KINDS
    }

    /// Returns the canonical Telegram field name for this update kind.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Message => "message",
            Self::EditedMessage => "edited_message",
            Self::ChannelPost => "channel_post",
            Self::EditedChannelPost => "edited_channel_post",
            Self::BusinessConnection => "business_connection",
            Self::BusinessMessage => "business_message",
            Self::EditedBusinessMessage => "edited_business_message",
            Self::DeletedBusinessMessages => "deleted_business_messages",
            Self::GuestMessage => "guest_message",
            Self::MessageReaction => "message_reaction",
            Self::MessageReactionCount => "message_reaction_count",
            Self::CallbackQuery => "callback_query",
            Self::InlineQuery => "inline_query",
            Self::ChosenInlineResult => "chosen_inline_result",
            Self::ShippingQuery => "shipping_query",
            Self::PreCheckoutQuery => "pre_checkout_query",
            Self::PurchasedPaidMedia => "purchased_paid_media",
            Self::Poll => "poll",
            Self::PollAnswer => "poll_answer",
            Self::MyChatMember => "my_chat_member",
            Self::ChatMember => "chat_member",
            Self::ChatJoinRequest => "chat_join_request",
            Self::ChatBoost => "chat_boost",
            Self::RemovedChatBoost => "removed_chat_boost",
            Self::ManagedBot => "managed_bot",
            Self::Unknown => "unknown",
        }
    }

    /// Returns the Telegram `allowed_updates` value for modeled kinds.
    pub const fn allowed_update_name(self) -> Option<&'static str> {
        match self {
            Self::Unknown => None,
            known => Some(known.as_str()),
        }
    }

    /// Parses a canonical Telegram update field name into a modeled kind.
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "message" => Some(Self::Message),
            "edited_message" => Some(Self::EditedMessage),
            "channel_post" => Some(Self::ChannelPost),
            "edited_channel_post" => Some(Self::EditedChannelPost),
            "business_connection" => Some(Self::BusinessConnection),
            "business_message" => Some(Self::BusinessMessage),
            "edited_business_message" => Some(Self::EditedBusinessMessage),
            "deleted_business_messages" => Some(Self::DeletedBusinessMessages),
            "guest_message" => Some(Self::GuestMessage),
            "message_reaction" => Some(Self::MessageReaction),
            "message_reaction_count" => Some(Self::MessageReactionCount),
            "callback_query" => Some(Self::CallbackQuery),
            "inline_query" => Some(Self::InlineQuery),
            "chosen_inline_result" => Some(Self::ChosenInlineResult),
            "shipping_query" => Some(Self::ShippingQuery),
            "pre_checkout_query" => Some(Self::PreCheckoutQuery),
            "purchased_paid_media" => Some(Self::PurchasedPaidMedia),
            "poll" => Some(Self::Poll),
            "poll_answer" => Some(Self::PollAnswer),
            "my_chat_member" => Some(Self::MyChatMember),
            "chat_member" => Some(Self::ChatMember),
            "chat_join_request" => Some(Self::ChatJoinRequest),
            "chat_boost" => Some(Self::ChatBoost),
            "removed_chat_boost" => Some(Self::RemovedChatBoost),
            "managed_bot" => Some(Self::ManagedBot),
            "unknown" => Some(Self::Unknown),
            _ => None,
        }
    }
}

impl fmt::Display for UpdateKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl std::str::FromStr for UpdateKind {
    type Err = Error;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        Self::from_name(value)
            .ok_or_else(|| invalid_request(format!("unknown update kind `{value}`")))
    }
}

/// Validated Telegram `allowed_updates` entry.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize)]
#[serde(transparent)]
pub struct AllowedUpdate(String);

impl AllowedUpdate {
    /// Creates a custom allowed update name.
    ///
    /// This keeps the SDK forward-compatible with Bot API update kinds that may
    /// not be modeled yet while still preventing empty/control-character names.
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        validate_allowed_update_name(&value)?;
        Ok(Self(value))
    }

    /// Creates an allowed update value from a modeled update kind.
    pub fn from_kind(kind: UpdateKind) -> Result<Self> {
        let name = kind.allowed_update_name().ok_or_else(|| {
            invalid_request("UpdateKind::Unknown cannot be used as allowed_updates")
        })?;
        Ok(Self(name.to_owned()))
    }

    /// Creates allowed update values from modeled update kinds.
    pub fn from_kinds(kinds: impl IntoIterator<Item = UpdateKind>) -> Result<Vec<Self>> {
        kinds.into_iter().map(Self::from_kind).collect()
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

impl AsRef<str> for AllowedUpdate {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for AllowedUpdate {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl std::str::FromStr for AllowedUpdate {
    type Err = Error;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        Self::new(value)
    }
}

impl<'de> Deserialize<'de> for AllowedUpdate {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

impl TryFrom<String> for AllowedUpdate {
    type Error = Error;

    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<&str> for AllowedUpdate {
    type Error = Error;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<UpdateKind> for AllowedUpdate {
    type Error = Error;

    fn try_from(kind: UpdateKind) -> std::result::Result<Self, Self::Error> {
        Self::from_kind(kind)
    }
}

impl From<AllowedUpdate> for String {
    fn from(value: AllowedUpdate) -> Self {
        value.into_inner()
    }
}

/// Telegram callback query object.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct CallbackQuery {
    pub id: String,
    pub from: User,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<Box<MaybeInaccessibleMessage>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inline_message_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chat_instance: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram inline query object.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct InlineQuery {
    pub id: String,
    pub from: User,
    pub query: String,
    pub offset: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chat_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<Location>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram chosen inline result object.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct ChosenInlineResult {
    pub result_id: String,
    pub from: User,
    pub query: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inline_message_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<Location>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram poll answer object.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct PollAnswer {
    pub poll_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub voter_chat: Option<Chat>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<User>,
    pub option_ids: Vec<u8>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram shipping query payload.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct ShippingQuery {
    pub id: String,
    pub from: User,
    pub invoice_payload: String,
    pub shipping_address: ShippingAddress,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram pre-checkout query payload.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct PreCheckoutQuery {
    pub id: String,
    pub from: User,
    pub currency: String,
    pub total_amount: i64,
    pub invoice_payload: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shipping_option_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order_info: Option<OrderInfo>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram paid media purchase payload.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct PaidMediaPurchased {
    pub from: User,
    pub paid_media_payload: String,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Rights granted to a bot by a business connection.
#[derive(Clone, Debug, Default, Deserialize)]
#[non_exhaustive]
pub struct BusinessBotRights {
    #[serde(default)]
    pub can_reply: bool,
    #[serde(default)]
    pub can_read_messages: bool,
    #[serde(default)]
    pub can_delete_sent_messages: bool,
    #[serde(default)]
    pub can_delete_all_messages: bool,
    #[serde(default)]
    pub can_edit_name: bool,
    #[serde(default)]
    pub can_edit_bio: bool,
    #[serde(default)]
    pub can_edit_profile_photo: bool,
    #[serde(default)]
    pub can_edit_username: bool,
    #[serde(default)]
    pub can_change_gift_settings: bool,
    #[serde(default)]
    pub can_view_gifts_and_stars: bool,
    #[serde(default)]
    pub can_convert_gifts_to_stars: bool,
    #[serde(default)]
    pub can_transfer_and_upgrade_gifts: bool,
    #[serde(default)]
    pub can_transfer_stars: bool,
    #[serde(default)]
    pub can_manage_stories: bool,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram business connection update payload.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct BusinessConnection {
    pub id: String,
    pub user: User,
    pub user_chat_id: i64,
    pub date: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rights: Option<BusinessBotRights>,
    pub is_enabled: bool,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl BusinessConnection {
    pub fn user_id(&self) -> i64 {
        self.user.id.0
    }
}

/// Reaction-count entry for aggregate reaction-count updates.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct ReactionCount {
    #[serde(rename = "type")]
    pub kind: ReactionType,
    pub total_count: i64,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram message reaction update payload.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct MessageReactionUpdated {
    pub chat: Chat,
    pub message_id: MessageId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<User>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor_chat: Option<Chat>,
    pub date: i64,
    pub old_reaction: Vec<ReactionType>,
    pub new_reaction: Vec<ReactionType>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram aggregate message reaction-count update payload.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct MessageReactionCountUpdated {
    pub chat: Chat,
    pub message_id: MessageId,
    pub date: i64,
    pub reactions: Vec<ReactionCount>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Forward-compatible chat boost source payload.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct ChatBoostSource {
    pub source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<User>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub giveaway_message_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prize_star_count: Option<i64>,
    #[serde(default)]
    pub is_unclaimed: bool,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram chat boost payload.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct ChatBoost {
    pub boost_id: String,
    pub add_date: i64,
    pub expiration_date: i64,
    pub source: ChatBoostSource,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram list of boosts added to a chat by a user.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct UserChatBoosts {
    pub boosts: Vec<ChatBoost>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram chat boost update payload.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct ChatBoostUpdated {
    pub chat: Chat,
    pub boost: ChatBoost,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram removed chat boost update payload.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct ChatBoostRemoved {
    pub chat: Chat,
    pub boost_id: String,
    pub remove_date: i64,
    pub source: ChatBoostSource,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram managed bot update payload.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct ManagedBotUpdated {
    pub user: User,
    pub bot: User,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram chat join request object.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct ChatJoinRequest {
    pub chat: Chat,
    pub from: User,
    pub user_chat_id: i64,
    pub date: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bio: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub invite_link: Option<ChatInviteLink>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl ChatJoinRequest {
    pub fn chat_id(&self) -> i64 {
        self.chat.id
    }

    pub fn user_id(&self) -> i64 {
        self.from.id.0
    }
}

/// Telegram business-message deletion update payload.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct BusinessMessagesDeleted {
    pub business_connection_id: String,
    pub chat: Chat,
    pub message_ids: Vec<MessageId>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl BusinessMessagesDeleted {
    pub fn chat_id(&self) -> i64 {
        self.chat.id
    }

    pub fn message_ids(&self) -> &[MessageId] {
        self.message_ids.as_slice()
    }
}

/// Telegram chat member update payload.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct ChatMemberUpdated {
    pub chat: Chat,
    pub from: User,
    pub date: i64,
    pub old_chat_member: ChatMember,
    pub new_chat_member: ChatMember,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub invite_link: Option<ChatInviteLink>,
    #[serde(default)]
    pub via_join_request: bool,
    #[serde(default)]
    pub via_chat_folder_invite_link: bool,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl ChatMemberUpdated {
    pub fn chat_id(&self) -> i64 {
        self.chat.id
    }

    pub fn actor(&self) -> &User {
        &self.from
    }

    pub fn actor_id(&self) -> i64 {
        self.from.id.0
    }

    pub fn member(&self) -> &ChatMember {
        &self.new_chat_member
    }

    pub fn subject(&self) -> Option<&User> {
        self.new_chat_member.user()
    }

    pub fn subject_id(&self) -> Option<i64> {
        self.subject().map(|user| user.id.0)
    }

    pub fn member_user(&self) -> Option<&User> {
        self.subject()
    }
}

/// Telegram update object.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct Update {
    pub update_id: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<Box<Message>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub edited_message: Option<Box<Message>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel_post: Option<Box<Message>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub edited_channel_post: Option<Box<Message>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub business_connection: Option<BusinessConnection>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub business_message: Option<Box<Message>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub edited_business_message: Option<Box<Message>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deleted_business_messages: Option<BusinessMessagesDeleted>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guest_message: Option<Box<Message>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_reaction: Option<MessageReactionUpdated>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_reaction_count: Option<MessageReactionCountUpdated>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub callback_query: Option<CallbackQuery>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inline_query: Option<InlineQuery>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chosen_inline_result: Option<ChosenInlineResult>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shipping_query: Option<ShippingQuery>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pre_checkout_query: Option<PreCheckoutQuery>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub purchased_paid_media: Option<PaidMediaPurchased>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub poll: Option<Box<Poll>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub poll_answer: Option<PollAnswer>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub my_chat_member: Option<ChatMemberUpdated>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chat_member: Option<ChatMemberUpdated>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chat_join_request: Option<ChatJoinRequest>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chat_boost: Option<ChatBoostUpdated>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub removed_chat_boost: Option<ChatBoostRemoved>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub managed_bot: Option<ManagedBotUpdated>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl Update {
    fn has_modeled_kind(&self) -> bool {
        self.message.is_some()
            || self.edited_message.is_some()
            || self.channel_post.is_some()
            || self.edited_channel_post.is_some()
            || self.business_connection.is_some()
            || self.business_message.is_some()
            || self.edited_business_message.is_some()
            || self.deleted_business_messages.is_some()
            || self.guest_message.is_some()
            || self.message_reaction.is_some()
            || self.message_reaction_count.is_some()
            || self.callback_query.is_some()
            || self.inline_query.is_some()
            || self.chosen_inline_result.is_some()
            || self.shipping_query.is_some()
            || self.pre_checkout_query.is_some()
            || self.purchased_paid_media.is_some()
            || self.poll.is_some()
            || self.poll_answer.is_some()
            || self.my_chat_member.is_some()
            || self.chat_member.is_some()
            || self.chat_join_request.is_some()
            || self.chat_boost.is_some()
            || self.removed_chat_boost.is_some()
            || self.managed_bot.is_some()
    }

    fn has_unmodeled_kind(&self) -> bool {
        !self.extra.is_empty()
    }

    /// Returns the primary update kind using stable precedence.
    pub fn kind(&self) -> UpdateKind {
        for kind in KNOWN_UPDATE_KINDS {
            if self.has_kind(kind) {
                return kind;
            }
        }

        UpdateKind::Unknown
    }

    /// Returns all detected update kinds.
    pub fn kinds(&self) -> Vec<UpdateKind> {
        let mut kinds = Vec::with_capacity(KNOWN_UPDATE_KINDS.len() + 1);
        for kind in KNOWN_UPDATE_KINDS {
            if self.has_kind(kind) {
                kinds.push(kind);
            }
        }

        if self.has_kind(UpdateKind::Unknown) {
            kinds.push(UpdateKind::Unknown);
        }

        kinds
    }

    /// Returns whether this update contains the given kind.
    pub fn has_kind(&self, kind: UpdateKind) -> bool {
        match kind {
            UpdateKind::Message => self.message.is_some(),
            UpdateKind::EditedMessage => self.edited_message.is_some(),
            UpdateKind::ChannelPost => self.channel_post.is_some(),
            UpdateKind::EditedChannelPost => self.edited_channel_post.is_some(),
            UpdateKind::BusinessConnection => self.business_connection.is_some(),
            UpdateKind::BusinessMessage => self.business_message.is_some(),
            UpdateKind::EditedBusinessMessage => self.edited_business_message.is_some(),
            UpdateKind::DeletedBusinessMessages => self.deleted_business_messages.is_some(),
            UpdateKind::GuestMessage => self.guest_message.is_some(),
            UpdateKind::MessageReaction => self.message_reaction.is_some(),
            UpdateKind::MessageReactionCount => self.message_reaction_count.is_some(),
            UpdateKind::CallbackQuery => self.callback_query.is_some(),
            UpdateKind::InlineQuery => self.inline_query.is_some(),
            UpdateKind::ChosenInlineResult => self.chosen_inline_result.is_some(),
            UpdateKind::ShippingQuery => self.shipping_query.is_some(),
            UpdateKind::PreCheckoutQuery => self.pre_checkout_query.is_some(),
            UpdateKind::PurchasedPaidMedia => self.purchased_paid_media.is_some(),
            UpdateKind::Poll => self.poll.is_some(),
            UpdateKind::PollAnswer => self.poll_answer.is_some(),
            UpdateKind::MyChatMember => self.my_chat_member.is_some(),
            UpdateKind::ChatMember => self.chat_member.is_some(),
            UpdateKind::ChatJoinRequest => self.chat_join_request.is_some(),
            UpdateKind::ChatBoost => self.chat_boost.is_some(),
            UpdateKind::RemovedChatBoost => self.removed_chat_boost.is_some(),
            UpdateKind::ManagedBot => self.managed_bot.is_some(),
            UpdateKind::Unknown => self.has_unmodeled_kind() || !self.has_modeled_kind(),
        }
    }

    /// Returns Mini App payload from the first available message-like field.
    pub fn web_app_data(&self) -> Option<&WebAppData> {
        if let Some(message) = self.message.as_deref() {
            return message.web_app_data();
        }
        if let Some(message) = self.edited_message.as_deref() {
            return message.web_app_data();
        }
        if let Some(message) = self.channel_post.as_deref() {
            return message.web_app_data();
        }
        if let Some(message) = self.edited_channel_post.as_deref() {
            return message.web_app_data();
        }
        if let Some(message) = self.business_message.as_deref() {
            return message.web_app_data();
        }
        if let Some(message) = self.edited_business_message.as_deref() {
            return message.web_app_data();
        }
        if let Some(message) = self.guest_message.as_deref() {
            return message.web_app_data();
        }

        self.callback_query
            .as_ref()
            .and_then(|query| query.message.as_deref())
            .and_then(|message| message.accessible())
            .and_then(|message| message.web_app_data())
    }

    pub fn chat_join_request(&self) -> Option<&ChatJoinRequest> {
        self.chat_join_request.as_ref()
    }

    pub fn business_connection(&self) -> Option<&BusinessConnection> {
        self.business_connection.as_ref()
    }

    pub fn deleted_business_messages(&self) -> Option<&BusinessMessagesDeleted> {
        self.deleted_business_messages.as_ref()
    }

    pub fn message_reaction(&self) -> Option<&MessageReactionUpdated> {
        self.message_reaction.as_ref()
    }

    pub fn message_reaction_count(&self) -> Option<&MessageReactionCountUpdated> {
        self.message_reaction_count.as_ref()
    }

    pub fn shipping_query(&self) -> Option<&ShippingQuery> {
        self.shipping_query.as_ref()
    }

    pub fn pre_checkout_query(&self) -> Option<&PreCheckoutQuery> {
        self.pre_checkout_query.as_ref()
    }

    pub fn purchased_paid_media(&self) -> Option<&PaidMediaPurchased> {
        self.purchased_paid_media.as_ref()
    }

    pub fn my_chat_member(&self) -> Option<&ChatMemberUpdated> {
        self.my_chat_member.as_ref()
    }

    pub fn chat_member(&self) -> Option<&ChatMemberUpdated> {
        self.chat_member.as_ref()
    }

    pub fn chat_boost(&self) -> Option<&ChatBoostUpdated> {
        self.chat_boost.as_ref()
    }

    pub fn removed_chat_boost(&self) -> Option<&ChatBoostRemoved> {
        self.removed_chat_boost.as_ref()
    }

    pub fn managed_bot(&self) -> Option<&ManagedBotUpdated> {
        self.managed_bot.as_ref()
    }
}

/// `getUpdates` request.
#[derive(Clone, Debug, Default, Serialize)]
pub struct GetUpdatesRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowed_updates: Option<Vec<AllowedUpdate>>,
}

impl GetUpdatesRequest {
    pub fn with_timeout(timeout_seconds: u16) -> Self {
        Self {
            timeout: Some(timeout_seconds),
            ..Self::default()
        }
    }

    pub fn allowed_updates(
        mut self,
        allowed_updates: impl IntoIterator<Item = AllowedUpdate>,
    ) -> Self {
        self.set_allowed_updates(allowed_updates);
        self
    }

    pub fn allowed_update_kinds(
        mut self,
        kinds: impl IntoIterator<Item = UpdateKind>,
    ) -> Result<Self> {
        self.set_allowed_update_kinds(kinds)?;
        Ok(self)
    }

    pub fn set_allowed_updates(
        &mut self,
        allowed_updates: impl IntoIterator<Item = AllowedUpdate>,
    ) -> &mut Self {
        self.allowed_updates = Some(allowed_updates.into_iter().collect());
        self
    }

    pub fn set_allowed_update_kinds(
        &mut self,
        kinds: impl IntoIterator<Item = UpdateKind>,
    ) -> Result<&mut Self> {
        self.allowed_updates = Some(AllowedUpdate::from_kinds(kinds)?);
        Ok(self)
    }

    pub fn clear_allowed_updates(&mut self) -> &mut Self {
        self.allowed_updates = None;
        self
    }

    pub fn validate(&self) -> Result<()> {
        if self
            .limit
            .is_some_and(|limit| limit == 0 || limit > MAX_GET_UPDATES_LIMIT)
        {
            return Err(invalid_request(format!(
                "getUpdates limit must be 1-{MAX_GET_UPDATES_LIMIT}"
            )));
        }

        if let Some(allowed_updates) = self.allowed_updates.as_ref() {
            validate_allowed_updates(allowed_updates)?;
        }

        Ok(())
    }
}

/// `answerCallbackQuery` request.
#[derive(Clone, Debug, Serialize)]
pub struct AnswerCallbackQueryRequest {
    pub callback_query_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub show_alert: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_time: Option<u32>,
}

impl AnswerCallbackQueryRequest {
    pub fn validate(&self) -> Result<()> {
        validate_non_empty_id("callback_query_id", &self.callback_query_id)?;
        if let Some(text) = self.text.as_deref() {
            validate_optional_text_limit(
                "callback answer text",
                text,
                MAX_CALLBACK_ANSWER_TEXT_CHARS,
            )?;
        }
        if let Some(url) = self.url.as_deref() {
            validate_http_url("callback answer url", url)?;
        }

        Ok(())
    }
}

/// `answerInlineQuery` request.
#[derive(Clone, Debug, Serialize)]
pub struct AnswerInlineQueryRequest {
    pub inline_query_id: String,
    pub results: Vec<InlineQueryResult>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_time: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_personal: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_offset: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub button: Option<InlineQueryResultsButton>,
}

impl AnswerInlineQueryRequest {
    pub fn new(
        inline_query_id: impl Into<String>,
        results: impl IntoIterator<Item = InlineQueryResult>,
    ) -> Self {
        Self {
            inline_query_id: inline_query_id.into(),
            results: results.into_iter().collect(),
            cache_time: None,
            is_personal: None,
            next_offset: None,
            button: None,
        }
    }

    pub fn add_result(mut self, result: impl Into<InlineQueryResult>) -> Self {
        self.results.push(result.into());
        self
    }

    pub fn cache_time(mut self, cache_time: u32) -> Self {
        self.cache_time = Some(cache_time);
        self
    }

    pub fn is_personal(mut self, is_personal: bool) -> Self {
        self.is_personal = Some(is_personal);
        self
    }

    pub fn next_offset(mut self, next_offset: impl Into<String>) -> Self {
        self.next_offset = Some(next_offset.into());
        self
    }

    pub fn button(mut self, button: InlineQueryResultsButton) -> Self {
        self.button = Some(button);
        self
    }

    pub fn validate(&self) -> Result<()> {
        validate_non_empty_id("inline_query_id", &self.inline_query_id)?;
        if self.results.len() > MAX_INLINE_QUERY_RESULTS {
            return Err(invalid_request(format!(
                "answerInlineQuery accepts at most {MAX_INLINE_QUERY_RESULTS} results"
            )));
        }
        for result in &self.results {
            result.validate()?;
        }
        if let Some(next_offset) = self.next_offset.as_deref() {
            if next_offset.len() > MAX_INLINE_NEXT_OFFSET_BYTES {
                return Err(invalid_request(format!(
                    "next_offset exceeds {MAX_INLINE_NEXT_OFFSET_BYTES} bytes"
                )));
            }
            validate_control_free_string("next_offset", next_offset)?;
        }
        if let Some(button) = self.button.as_ref() {
            button.validate()?;
        }

        Ok(())
    }
}

fn invalid_request(reason: impl Into<String>) -> Error {
    Error::InvalidRequest {
        reason: reason.into(),
    }
}

fn validate_non_empty_id(field: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(invalid_request(format!("{field} cannot be empty")));
    }
    if value.chars().any(char::is_control) {
        return Err(invalid_request(format!(
            "{field} must not contain control characters"
        )));
    }

    Ok(())
}

fn validate_optional_text_limit(field: &str, value: &str, max_chars: usize) -> Result<()> {
    if value.chars().count() > max_chars {
        return Err(invalid_request(format!(
            "{field} must be at most {max_chars} characters"
        )));
    }
    validate_control_free_string(field, value)
}

fn validate_allowed_update_name(update: &str) -> Result<()> {
    if update.trim().is_empty() {
        return Err(invalid_request(
            "allowed_updates must not contain empty values",
        ));
    }
    if update.chars().any(char::is_control) {
        return Err(invalid_request(
            "allowed_updates must not contain control characters",
        ));
    }
    if update.chars().any(char::is_whitespace) {
        return Err(invalid_request(
            "allowed_updates must not contain whitespace",
        ));
    }

    Ok(())
}

pub(crate) fn validate_allowed_updates(allowed_updates: &[AllowedUpdate]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for update in allowed_updates {
        validate_allowed_update_name(update.as_str())?;
        if !seen.insert(update.as_str()) {
            return Err(invalid_request(format!(
                "allowed_updates contains duplicate value `{update}`"
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::error::Error as StdError;

    use serde_json::json;

    use super::*;

    #[test]
    fn detects_primary_update_kind() -> std::result::Result<(), Box<dyn StdError>> {
        let update: Update = serde_json::from_value(json!({
            "update_id": 1,
            "message": {
                "message_id": 10,
                "date": 1700000000,
                "chat": {"id": 1, "type": "private"},
                "text": "hello"
            }
        }))?;

        assert_eq!(update.kind(), UpdateKind::Message);
        assert_eq!(update.kinds(), vec![UpdateKind::Message]);
        assert!(update.has_kind(UpdateKind::Message));
        Ok(())
    }

    #[test]
    fn supports_multi_kind_updates() -> std::result::Result<(), Box<dyn StdError>> {
        let update: Update = serde_json::from_value(json!({
            "update_id": 2,
            "message": {
                "message_id": 11,
                "date": 1700000001,
                "chat": {"id": 1, "type": "private"},
                "text": "hello"
            },
            "callback_query": {
                "id": "cb-1",
                "from": {
                    "id": 1,
                    "is_bot": false,
                    "first_name": "test"
                },
                "chat_instance": "ci-1",
                "data": "payload"
            }
        }))?;

        assert_eq!(
            update.kinds(),
            vec![UpdateKind::Message, UpdateKind::CallbackQuery]
        );
        assert!(update.has_kind(UpdateKind::CallbackQuery));
        Ok(())
    }

    #[test]
    fn callback_query_accepts_inaccessible_message() -> std::result::Result<(), Box<dyn StdError>> {
        let update: Update = serde_json::from_value(json!({
            "update_id": 21,
            "callback_query": {
                "id": "cb-inaccessible",
                "from": {"id": 1, "is_bot": false, "first_name": "tester"},
                "message": {
                    "message_id": 55,
                    "date": 0,
                    "chat": {"id": -10010, "type": "supergroup", "title": "mods"}
                },
                "chat_instance": "ci",
                "data": "payload"
            }
        }))?;

        let message = update
            .callback_query
            .as_ref()
            .and_then(|query| query.message.as_deref())
            .ok_or("missing callback message")?;
        assert!(!message.is_accessible());
        assert_eq!(message.chat().id, -10010);
        assert_eq!(message.message_id(), MessageId(55));
        assert_eq!(update.web_app_data(), None);
        Ok(())
    }

    #[test]
    fn returns_unknown_for_unmodeled_payload() -> std::result::Result<(), Box<dyn StdError>> {
        let update: Update = serde_json::from_value(json!({
            "update_id": 3,
            "new_kind_payload": {"foo": "bar"}
        }))?;

        assert_eq!(update.kind(), UpdateKind::Unknown);
        assert_eq!(update.kinds(), vec![UpdateKind::Unknown]);
        assert!(update.has_kind(UpdateKind::Unknown));
        Ok(())
    }

    #[test]
    fn keeps_unknown_alongside_modeled_kind() -> std::result::Result<(), Box<dyn StdError>> {
        let update: Update = serde_json::from_value(json!({
            "update_id": 4,
            "message": {
                "message_id": 12,
                "date": 1700000004,
                "chat": {"id": 1, "type": "private"},
                "text": "hello"
            },
            "new_kind_payload": {"foo": "bar"}
        }))?;

        assert_eq!(update.kind(), UpdateKind::Message);
        assert_eq!(
            update.kinds(),
            vec![UpdateKind::Message, UpdateKind::Unknown]
        );
        assert!(update.has_kind(UpdateKind::Unknown));
        Ok(())
    }

    fn update_for_kind(kind: UpdateKind) -> std::result::Result<Update, Box<dyn StdError>> {
        let payload = match kind {
            UpdateKind::Message => json!({
                "update_id": 100,
                "message": {
                    "message_id": 10,
                    "date": 1700000100,
                    "chat": {"id": 1, "type": "private"},
                    "text": "hello"
                }
            }),
            UpdateKind::EditedMessage => json!({
                "update_id": 101,
                "edited_message": {
                    "message_id": 10,
                    "date": 1700000101,
                    "chat": {"id": 1, "type": "private"},
                    "text": "hello"
                }
            }),
            UpdateKind::ChannelPost => json!({
                "update_id": 102,
                "channel_post": {
                    "message_id": 10,
                    "date": 1700000102,
                    "chat": {"id": -1001, "type": "channel"},
                    "text": "post"
                }
            }),
            UpdateKind::EditedChannelPost => json!({
                "update_id": 103,
                "edited_channel_post": {
                    "message_id": 10,
                    "date": 1700000103,
                    "chat": {"id": -1001, "type": "channel"},
                    "text": "post"
                }
            }),
            UpdateKind::BusinessConnection => json!({
                "update_id": 117,
                "business_connection": {
                    "id": "business-1",
                    "user": {"id": 7001, "is_bot": false, "first_name": "owner"},
                    "user_chat_id": 7001,
                    "date": 1700000117,
                    "rights": {
                        "can_reply": true,
                        "can_read_messages": true
                    },
                    "is_enabled": true
                }
            }),
            UpdateKind::BusinessMessage => json!({
                "update_id": 113,
                "business_message": {
                    "message_id": 20,
                    "business_connection_id": "business-1",
                    "date": 1700000113,
                    "chat": {"id": 7001, "type": "private", "first_name": "customer"},
                    "from": {"id": 7001, "is_bot": false, "first_name": "customer"},
                    "text": "business hello"
                }
            }),
            UpdateKind::EditedBusinessMessage => json!({
                "update_id": 114,
                "edited_business_message": {
                    "message_id": 21,
                    "business_connection_id": "business-1",
                    "date": 1700000114,
                    "chat": {"id": 7001, "type": "private", "first_name": "customer"},
                    "from": {"id": 7001, "is_bot": false, "first_name": "customer"},
                    "text": "business edited"
                }
            }),
            UpdateKind::DeletedBusinessMessages => json!({
                "update_id": 115,
                "deleted_business_messages": {
                    "business_connection_id": "business-1",
                    "chat": {"id": 7001, "type": "private", "first_name": "customer"},
                    "message_ids": [20, 21]
                }
            }),
            UpdateKind::GuestMessage => json!({
                "update_id": 116,
                "guest_message": {
                    "message_id": 22,
                    "guest_query_id": "guest-1",
                    "date": 1700000116,
                    "chat": {"id": 8001, "type": "private", "first_name": "guest"},
                    "from": {"id": 8001, "is_bot": false, "first_name": "guest"},
                    "text": "guest hello"
                }
            }),
            UpdateKind::MessageReaction => json!({
                "update_id": 118,
                "message_reaction": {
                    "chat": {"id": -1001, "type": "supergroup", "title": "mods"},
                    "message_id": 20,
                    "user": {"id": 1, "is_bot": false, "first_name": "reactor"},
                    "date": 1700000118,
                    "old_reaction": [],
                    "new_reaction": [{"type": "emoji", "emoji": "👍"}]
                }
            }),
            UpdateKind::MessageReactionCount => json!({
                "update_id": 119,
                "message_reaction_count": {
                    "chat": {"id": -1001, "type": "supergroup", "title": "mods"},
                    "message_id": 20,
                    "date": 1700000119,
                    "reactions": [{
                        "type": {"type": "emoji", "emoji": "👍"},
                        "total_count": 3
                    }]
                }
            }),
            UpdateKind::CallbackQuery => json!({
                "update_id": 104,
                "callback_query": {
                    "id": "cb-104",
                    "from": {"id": 1, "is_bot": false, "first_name": "tester"},
                    "chat_instance": "ci",
                    "data": "payload"
                }
            }),
            UpdateKind::InlineQuery => json!({
                "update_id": 105,
                "inline_query": {
                    "id": "iq-105",
                    "from": {"id": 1, "is_bot": false, "first_name": "tester"},
                    "query": "search",
                    "offset": ""
                }
            }),
            UpdateKind::ChosenInlineResult => json!({
                "update_id": 106,
                "chosen_inline_result": {
                    "result_id": "res-106",
                    "from": {"id": 1, "is_bot": false, "first_name": "tester"},
                    "query": "search"
                }
            }),
            UpdateKind::ShippingQuery => json!({
                "update_id": 120,
                "shipping_query": {
                    "id": "shipping-1",
                    "from": {"id": 1, "is_bot": false, "first_name": "buyer"},
                    "invoice_payload": "invoice-payload",
                    "shipping_address": {
                        "country_code": "US",
                        "state": "CA",
                        "city": "San Francisco",
                        "street_line1": "1 Market St",
                        "street_line2": "Suite 1",
                        "post_code": "94105"
                    }
                }
            }),
            UpdateKind::PreCheckoutQuery => json!({
                "update_id": 121,
                "pre_checkout_query": {
                    "id": "checkout-1",
                    "from": {"id": 1, "is_bot": false, "first_name": "buyer"},
                    "currency": "USD",
                    "total_amount": 145,
                    "invoice_payload": "invoice-payload",
                    "shipping_option_id": "ground",
                    "order_info": {
                        "name": "Buyer",
                        "email": "buyer@example.com"
                    }
                }
            }),
            UpdateKind::PurchasedPaidMedia => json!({
                "update_id": 122,
                "purchased_paid_media": {
                    "from": {"id": 1, "is_bot": false, "first_name": "buyer"},
                    "paid_media_payload": "paid-media-payload"
                }
            }),
            UpdateKind::Poll => json!({
                "update_id": 107,
                "poll": {
                    "id": "poll-107",
                    "question": "q?",
                    "options": [{"text": "a", "voter_count": 1}],
                    "total_voter_count": 1,
                    "is_closed": false,
                    "is_anonymous": false,
                    "type": "regular",
                    "allows_multiple_answers": false
                }
            }),
            UpdateKind::PollAnswer => json!({
                "update_id": 108,
                "poll_answer": {
                    "poll_id": "poll-107",
                    "user": {"id": 1, "is_bot": false, "first_name": "tester"},
                    "option_ids": [0]
                }
            }),
            UpdateKind::MyChatMember => json!({
                "update_id": 109,
                "my_chat_member": {
                    "chat": {"id": -1001, "type": "supergroup", "title": "mods"},
                    "from": {"id": 1, "is_bot": false, "first_name": "admin"},
                    "date": 1700000109,
                    "old_chat_member": {
                        "status": "member",
                        "user": {"id": 9, "is_bot": true, "first_name": "tele"}
                    },
                    "new_chat_member": {
                        "status": "administrator",
                        "user": {"id": 9, "is_bot": true, "first_name": "tele"},
                        "can_manage_chat": true
                    }
                }
            }),
            UpdateKind::ChatMember => json!({
                "update_id": 110,
                "chat_member": {
                    "chat": {"id": -1001, "type": "supergroup", "title": "mods"},
                    "from": {"id": 1, "is_bot": false, "first_name": "admin"},
                    "date": 1700000110,
                    "old_chat_member": {
                        "status": "left",
                        "user": {"id": 55, "is_bot": false, "first_name": "member"}
                    },
                    "new_chat_member": {
                        "status": "member",
                        "user": {"id": 55, "is_bot": false, "first_name": "member"}
                    },
                    "via_join_request": true
                }
            }),
            UpdateKind::ChatJoinRequest => json!({
                "update_id": 111,
                "chat_join_request": {
                    "chat": {"id": -1001, "type": "supergroup", "title": "mods"},
                    "from": {"id": 99, "is_bot": false, "first_name": "applicant"},
                    "user_chat_id": 9001,
                    "date": 1700000111,
                    "bio": "hello there"
                }
            }),
            UpdateKind::ChatBoost => json!({
                "update_id": 123,
                "chat_boost": {
                    "chat": {"id": -1001, "type": "supergroup", "title": "mods"},
                    "boost": {
                        "boost_id": "boost-1",
                        "add_date": 1700000123,
                        "expiration_date": 1700086523,
                        "source": {
                            "source": "premium",
                            "user": {"id": 1, "is_bot": false, "first_name": "booster"}
                        }
                    }
                }
            }),
            UpdateKind::RemovedChatBoost => json!({
                "update_id": 124,
                "removed_chat_boost": {
                    "chat": {"id": -1001, "type": "supergroup", "title": "mods"},
                    "boost_id": "boost-1",
                    "remove_date": 1700000124,
                    "source": {
                        "source": "premium",
                        "user": {"id": 1, "is_bot": false, "first_name": "booster"}
                    }
                }
            }),
            UpdateKind::ManagedBot => json!({
                "update_id": 125,
                "managed_bot": {
                    "user": {"id": 1, "is_bot": false, "first_name": "owner"},
                    "bot": {"id": 2, "is_bot": true, "first_name": "managed"}
                }
            }),
            UpdateKind::Unknown => json!({
                "update_id": 112,
                "new_kind_payload": {"foo": "bar"}
            }),
        };

        Ok(serde_json::from_value(payload)?)
    }

    #[test]
    fn update_kind_matrix_stays_in_sync() -> std::result::Result<(), Box<dyn StdError>> {
        for kind in KNOWN_UPDATE_KINDS {
            let update = update_for_kind(kind)?;
            assert!(
                update.has_kind(kind),
                "missing has_kind mapping for {kind:?}"
            );
            assert!(
                update.kinds().contains(&kind),
                "missing kinds mapping for {kind:?}"
            );
        }
        Ok(())
    }

    #[test]
    fn unknown_update_kind_matrix_stays_in_sync() -> std::result::Result<(), Box<dyn StdError>> {
        let update = update_for_kind(UpdateKind::Unknown)?;
        assert_eq!(update.kind(), UpdateKind::Unknown);
        assert!(update.has_kind(UpdateKind::Unknown));
        assert_eq!(update.kinds(), vec![UpdateKind::Unknown]);
        Ok(())
    }

    #[test]
    fn parses_chat_join_request_as_typed_model() -> std::result::Result<(), Box<dyn StdError>> {
        let update = update_for_kind(UpdateKind::ChatJoinRequest)?;
        let Some(join_request) = update.chat_join_request() else {
            return Err("missing typed join request".into());
        };

        assert_eq!(join_request.chat_id(), -1001);
        assert_eq!(join_request.user_id(), 99);
        assert_eq!(join_request.bio.as_deref(), Some("hello there"));

        Ok(())
    }

    #[test]
    fn parses_business_and_guest_updates_as_message_like_models()
    -> std::result::Result<(), Box<dyn StdError>> {
        let connection = update_for_kind(UpdateKind::BusinessConnection)?;
        let connection = connection
            .business_connection()
            .ok_or("missing business connection")?;
        assert_eq!(connection.id, "business-1");
        assert_eq!(connection.user_id(), 7001);
        assert!(
            connection
                .rights
                .as_ref()
                .is_some_and(|rights| rights.can_reply)
        );

        let business = update_for_kind(UpdateKind::BusinessMessage)?;
        assert_eq!(business.kind(), UpdateKind::BusinessMessage);
        let message = business
            .business_message
            .as_deref()
            .ok_or("missing business message")?;
        assert_eq!(message.message_id, MessageId(20));
        assert_eq!(message.chat.id, 7001);
        assert_eq!(
            message.business_connection_id.as_deref(),
            Some("business-1")
        );

        let edited = update_for_kind(UpdateKind::EditedBusinessMessage)?;
        assert_eq!(edited.kind(), UpdateKind::EditedBusinessMessage);
        assert_eq!(
            edited
                .edited_business_message
                .as_deref()
                .and_then(|message| message.text.as_deref()),
            Some("business edited")
        );

        let deleted = update_for_kind(UpdateKind::DeletedBusinessMessages)?;
        let deleted = deleted
            .deleted_business_messages()
            .ok_or("missing deleted business messages")?;
        assert_eq!(deleted.business_connection_id, "business-1");
        assert_eq!(deleted.chat_id(), 7001);
        assert_eq!(deleted.message_ids(), &[MessageId(20), MessageId(21)]);

        let guest = update_for_kind(UpdateKind::GuestMessage)?;
        assert_eq!(guest.kind(), UpdateKind::GuestMessage);
        let guest_message = guest
            .guest_message
            .as_deref()
            .ok_or("missing guest message")?;
        assert_eq!(guest_message.text.as_deref(), Some("guest hello"));
        assert_eq!(guest_message.guest_query_id.as_deref(), Some("guest-1"));

        Ok(())
    }

    #[test]
    fn parses_modern_update_payloads_as_typed_models() -> std::result::Result<(), Box<dyn StdError>>
    {
        let reaction = update_for_kind(UpdateKind::MessageReaction)?;
        let reaction = reaction
            .message_reaction()
            .ok_or("missing message reaction")?;
        assert_eq!(reaction.chat.id, -1001);
        assert_eq!(reaction.message_id, MessageId(20));
        assert_eq!(reaction.user.as_ref().map(|user| user.id.0), Some(1));
        assert_eq!(reaction.new_reaction.len(), 1);

        let reaction_count = update_for_kind(UpdateKind::MessageReactionCount)?;
        let reaction_count = reaction_count
            .message_reaction_count()
            .ok_or("missing message reaction count")?;
        assert_eq!(reaction_count.reactions.len(), 1);
        assert_eq!(reaction_count.reactions[0].total_count, 3);

        let shipping = update_for_kind(UpdateKind::ShippingQuery)?;
        let shipping = shipping.shipping_query().ok_or("missing shipping query")?;
        assert_eq!(shipping.from.id.0, 1);
        assert_eq!(shipping.shipping_address.country_code, "US");

        let checkout = update_for_kind(UpdateKind::PreCheckoutQuery)?;
        let checkout = checkout
            .pre_checkout_query()
            .ok_or("missing pre-checkout query")?;
        assert_eq!(checkout.total_amount, 145);
        assert_eq!(
            checkout
                .order_info
                .as_ref()
                .and_then(|info| info.email.as_deref()),
            Some("buyer@example.com")
        );

        let purchase = update_for_kind(UpdateKind::PurchasedPaidMedia)?;
        let purchase = purchase
            .purchased_paid_media()
            .ok_or("missing paid media purchase")?;
        assert_eq!(purchase.paid_media_payload, "paid-media-payload");

        let boost = update_for_kind(UpdateKind::ChatBoost)?;
        let boost = boost.chat_boost().ok_or("missing chat boost")?;
        assert_eq!(boost.chat.id, -1001);
        assert_eq!(
            boost.boost.source.user.as_ref().map(|user| user.id.0),
            Some(1)
        );

        let removed = update_for_kind(UpdateKind::RemovedChatBoost)?;
        let removed = removed
            .removed_chat_boost()
            .ok_or("missing removed chat boost")?;
        assert_eq!(removed.boost_id, "boost-1");

        let managed = update_for_kind(UpdateKind::ManagedBot)?;
        let managed = managed.managed_bot().ok_or("missing managed bot")?;
        assert_eq!(managed.user.id.0, 1);
        assert_eq!(managed.bot.id.0, 2);

        Ok(())
    }

    #[test]
    fn parses_chat_member_updates_as_typed_model() -> std::result::Result<(), Box<dyn StdError>> {
        let member_update = update_for_kind(UpdateKind::ChatMember)?;
        let my_member_update = update_for_kind(UpdateKind::MyChatMember)?;

        let Some(chat_member) = member_update.chat_member() else {
            return Err("missing chat_member update".into());
        };
        assert_eq!(chat_member.chat_id(), -1001);
        assert_eq!(chat_member.actor_id(), 1);
        assert_eq!(chat_member.subject_id(), Some(55));
        assert!(chat_member.via_join_request);

        let Some(my_chat_member) = my_member_update.my_chat_member() else {
            return Err("missing my_chat_member update".into());
        };
        assert_eq!(my_chat_member.chat_id(), -1001);
        assert_eq!(my_chat_member.actor_id(), 1);
        assert!(my_chat_member.member().is_admin());

        Ok(())
    }

    #[test]
    fn update_kind_names_round_trip() {
        for kind in UpdateKind::all_known() {
            assert_eq!(UpdateKind::from_name(kind.as_str()), Some(*kind));
            assert_eq!(kind.allowed_update_name(), Some(kind.as_str()));
        }

        assert_eq!(UpdateKind::Unknown.as_str(), "unknown");
        assert_eq!(UpdateKind::Unknown.allowed_update_name(), None);
        assert_eq!(UpdateKind::from_name("unknown"), Some(UpdateKind::Unknown));
        assert_eq!(UpdateKind::from_name("future_update"), None);
    }

    #[test]
    fn allowed_update_values_are_validated() -> std::result::Result<(), Box<dyn StdError>> {
        assert_eq!(
            AllowedUpdate::from_kind(UpdateKind::CallbackQuery)?.as_str(),
            "callback_query"
        );
        assert_eq!(
            serde_json::from_str::<AllowedUpdate>("\"message\"")?.as_str(),
            "message"
        );
        assert_eq!(
            AllowedUpdate::from_kinds([UpdateKind::Message, UpdateKind::ChatBoost])?,
            vec![
                AllowedUpdate::new("message")?,
                AllowedUpdate::new("chat_boost")?
            ]
        );
        assert!(matches!(
            AllowedUpdate::from_kind(UpdateKind::Unknown),
            Err(Error::InvalidRequest { .. })
        ));
        assert!(matches!(
            AllowedUpdate::new(""),
            Err(Error::InvalidRequest { .. })
        ));
        assert!(matches!(
            AllowedUpdate::new("message\n"),
            Err(Error::InvalidRequest { .. })
        ));
        assert!(matches!(
            AllowedUpdate::new(" message "),
            Err(Error::InvalidRequest { .. })
        ));
        assert!(serde_json::from_str::<AllowedUpdate>("\" message \"").is_err());

        Ok(())
    }

    #[test]
    fn validates_get_updates_request_bounds() -> Result<()> {
        let valid = GetUpdatesRequest {
            limit: Some(100),
            allowed_updates: Some(vec![
                AllowedUpdate::new("message")?,
                AllowedUpdate::new("callback_query")?,
            ]),
            ..GetUpdatesRequest::default()
        };
        assert!(valid.validate().is_ok());

        let invalid_limit = GetUpdatesRequest {
            limit: Some(0),
            ..GetUpdatesRequest::default()
        };
        assert!(matches!(
            invalid_limit.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let duplicate_update = GetUpdatesRequest {
            allowed_updates: Some(vec![
                AllowedUpdate::new("message")?,
                AllowedUpdate::new("message")?,
            ]),
            ..GetUpdatesRequest::default()
        };
        assert!(matches!(
            duplicate_update.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        Ok(())
    }

    #[test]
    fn validates_callback_and_inline_answers() -> Result<()> {
        let valid_callback = AnswerCallbackQueryRequest {
            callback_query_id: "callback-1".to_owned(),
            text: Some("ok".to_owned()),
            show_alert: None,
            url: Some("https://example.com".to_owned()),
            cache_time: None,
        };
        assert!(valid_callback.validate().is_ok());

        let invalid_callback = AnswerCallbackQueryRequest {
            callback_query_id: String::new(),
            text: None,
            show_alert: None,
            url: None,
            cache_time: None,
        };
        assert!(matches!(
            invalid_callback.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let too_many_results = (0..=MAX_INLINE_QUERY_RESULTS)
            .map(|index| {
                InlineQueryResult::new(json!({"type": "article", "id": index.to_string()}))
            })
            .collect::<Result<Vec<_>>>()?;
        let invalid_inline = AnswerInlineQueryRequest::new("inline-1", too_many_results);
        assert!(matches!(
            invalid_inline.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let invalid_offset = AnswerInlineQueryRequest::new("inline-1", Vec::new())
            .next_offset("x".repeat(MAX_INLINE_NEXT_OFFSET_BYTES + 1));
        assert!(matches!(
            invalid_offset.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let invalid_button = AnswerInlineQueryRequest::new("inline-1", Vec::new())
            .button(InlineQueryResultsButton::new("Open"));
        assert!(matches!(
            invalid_button.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        Ok(())
    }
}

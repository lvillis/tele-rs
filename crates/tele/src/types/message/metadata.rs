use std::collections::BTreeMap;

use serde::Deserialize;
use serde_json::Value;

use super::model::Message;
use super::payments::StarAmount;

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct SuggestedPostPrice {
    pub currency: String,
    pub amount: i64,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum SuggestedPostState {
    Pending,
    Approved,
    Declined,
    Unknown(String),
}

impl SuggestedPostState {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Pending => "pending",
            Self::Approved => "approved",
            Self::Declined => "declined",
            Self::Unknown(value) => value.as_str(),
        }
    }
}

impl<'de> Deserialize<'de> for SuggestedPostState {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let state = String::deserialize(deserializer)?;
        Ok(match state.as_str() {
            "pending" => Self::Pending,
            "approved" => Self::Approved,
            "declined" => Self::Declined,
            _ => Self::Unknown(state),
        })
    }
}

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct SuggestedPostInfo {
    pub state: SuggestedPostState,
    #[serde(default)]
    pub price: Option<SuggestedPostPrice>,
    #[serde(default)]
    pub send_date: Option<i64>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct SuggestedPostApproved {
    #[serde(default)]
    pub suggested_post_message: Option<Box<Message>>,
    #[serde(default)]
    pub price: Option<SuggestedPostPrice>,
    pub send_date: i64,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct SuggestedPostApprovalFailed {
    #[serde(default)]
    pub suggested_post_message: Option<Box<Message>>,
    pub price: SuggestedPostPrice,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct SuggestedPostDeclined {
    #[serde(default)]
    pub suggested_post_message: Option<Box<Message>>,
    #[serde(default)]
    pub comment: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct SuggestedPostPaid {
    #[serde(default)]
    pub suggested_post_message: Option<Box<Message>>,
    pub currency: String,
    #[serde(default)]
    pub amount: Option<i64>,
    #[serde(default)]
    pub star_amount: Option<StarAmount>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum SuggestedPostRefundReason {
    PostDeleted,
    PaymentRefunded,
    Unknown(String),
}

impl SuggestedPostRefundReason {
    pub fn as_str(&self) -> &str {
        match self {
            Self::PostDeleted => "post_deleted",
            Self::PaymentRefunded => "payment_refunded",
            Self::Unknown(value) => value.as_str(),
        }
    }
}

impl<'de> Deserialize<'de> for SuggestedPostRefundReason {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let reason = String::deserialize(deserializer)?;
        Ok(match reason.as_str() {
            "post_deleted" => Self::PostDeleted,
            "payment_refunded" => Self::PaymentRefunded,
            _ => Self::Unknown(reason),
        })
    }
}

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct SuggestedPostRefunded {
    #[serde(default)]
    pub suggested_post_message: Option<Box<Message>>,
    pub reason: SuggestedPostRefundReason,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[non_exhaustive]
pub enum MessageKind {
    WriteAccessAllowed,
    WebAppData,
    ConnectedWebsite,
    Poll,
    PaidMedia,
    Checklist,
    Game,
    Invoice,
    SuccessfulPayment,
    RefundedPayment,
    Gift,
    UniqueGift,
    GiftUpgradeSent,
    NewChatMembers,
    LeftChatMember,
    ChatOwnerLeft,
    ChatOwnerChanged,
    NewChatTitle,
    NewChatPhoto,
    DeleteChatPhoto,
    GroupChatCreated,
    SupergroupChatCreated,
    ChannelChatCreated,
    PinnedMessage,
    MessageAutoDeleteTimerChanged,
    MigrateToChat,
    MigrateFromChat,
    UsersShared,
    ChatShared,
    ProximityAlertTriggered,
    BoostAdded,
    ChecklistTasksDone,
    ChecklistTasksAdded,
    DirectMessagePriceChanged,
    ForumTopicCreated,
    ForumTopicEdited,
    ForumTopicClosed,
    ForumTopicReopened,
    GeneralForumTopicHidden,
    GeneralForumTopicUnhidden,
    GiveawayCreated,
    Giveaway,
    GiveawayWinners,
    GiveawayCompleted,
    PaidMessagePriceChanged,
    SuggestedPostApproved,
    SuggestedPostApprovalFailed,
    SuggestedPostDeclined,
    SuggestedPostPaid,
    SuggestedPostRefunded,
    VideoChatScheduled,
    VideoChatStarted,
    VideoChatEnded,
    VideoChatParticipantsInvited,
    Animation,
    Audio,
    Contact,
    Dice,
    Document,
    Location,
    Photo,
    Sticker,
    Story,
    Venue,
    Video,
    VideoNote,
    Voice,
    Text,
    Caption,
    Unknown,
}

pub(crate) const KNOWN_MESSAGE_KINDS: &[MessageKind] = &[
    MessageKind::WriteAccessAllowed,
    MessageKind::WebAppData,
    MessageKind::ConnectedWebsite,
    MessageKind::Poll,
    MessageKind::PaidMedia,
    MessageKind::Checklist,
    MessageKind::Game,
    MessageKind::Invoice,
    MessageKind::SuccessfulPayment,
    MessageKind::RefundedPayment,
    MessageKind::Gift,
    MessageKind::UniqueGift,
    MessageKind::GiftUpgradeSent,
    MessageKind::NewChatMembers,
    MessageKind::LeftChatMember,
    MessageKind::ChatOwnerLeft,
    MessageKind::ChatOwnerChanged,
    MessageKind::NewChatTitle,
    MessageKind::NewChatPhoto,
    MessageKind::DeleteChatPhoto,
    MessageKind::GroupChatCreated,
    MessageKind::SupergroupChatCreated,
    MessageKind::ChannelChatCreated,
    MessageKind::PinnedMessage,
    MessageKind::MessageAutoDeleteTimerChanged,
    MessageKind::MigrateToChat,
    MessageKind::MigrateFromChat,
    MessageKind::UsersShared,
    MessageKind::ChatShared,
    MessageKind::ProximityAlertTriggered,
    MessageKind::BoostAdded,
    MessageKind::ChecklistTasksDone,
    MessageKind::ChecklistTasksAdded,
    MessageKind::DirectMessagePriceChanged,
    MessageKind::ForumTopicCreated,
    MessageKind::ForumTopicEdited,
    MessageKind::ForumTopicClosed,
    MessageKind::ForumTopicReopened,
    MessageKind::GeneralForumTopicHidden,
    MessageKind::GeneralForumTopicUnhidden,
    MessageKind::GiveawayCreated,
    MessageKind::Giveaway,
    MessageKind::GiveawayWinners,
    MessageKind::GiveawayCompleted,
    MessageKind::PaidMessagePriceChanged,
    MessageKind::SuggestedPostApproved,
    MessageKind::SuggestedPostApprovalFailed,
    MessageKind::SuggestedPostDeclined,
    MessageKind::SuggestedPostPaid,
    MessageKind::SuggestedPostRefunded,
    MessageKind::VideoChatScheduled,
    MessageKind::VideoChatStarted,
    MessageKind::VideoChatEnded,
    MessageKind::VideoChatParticipantsInvited,
    MessageKind::Animation,
    MessageKind::Audio,
    MessageKind::Contact,
    MessageKind::Dice,
    MessageKind::Document,
    MessageKind::Location,
    MessageKind::Photo,
    MessageKind::Sticker,
    MessageKind::Story,
    MessageKind::Venue,
    MessageKind::Video,
    MessageKind::VideoNote,
    MessageKind::Voice,
    MessageKind::Text,
    MessageKind::Caption,
];

use std::collections::BTreeMap;

use serde::ser::SerializeMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::extra::{
    field_len as extra_field_len, serialize_fields as serialize_extra_fields,
    serialize_optional_field,
};

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

impl Serialize for SuggestedPostPrice {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = ["currency", "amount"];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let mut object = serializer.serialize_map(Some(extra_len + 2))?;
        object.serialize_entry("currency", &self.currency)?;
        object.serialize_entry("amount", &self.amount)?;
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
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

impl Serialize for SuggestedPostState {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct SuggestedPostInfo {
    pub state: SuggestedPostState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub price: Option<SuggestedPostPrice>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub send_date: Option<i64>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl Serialize for SuggestedPostInfo {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = ["state", "price", "send_date"];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let optional_len =
            usize::from(self.price.is_some()) + usize::from(self.send_date.is_some());
        let mut object = serializer.serialize_map(Some(extra_len + optional_len + 1))?;
        object.serialize_entry("state", &self.state)?;
        serialize_optional_field(&mut object, "price", &self.price)?;
        serialize_optional_field(&mut object, "send_date", &self.send_date)?;
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
}

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct SuggestedPostApproved {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suggested_post_message: Option<Box<Message>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub price: Option<SuggestedPostPrice>,
    pub send_date: i64,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl Serialize for SuggestedPostApproved {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = ["suggested_post_message", "price", "send_date"];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let optional_len =
            usize::from(self.suggested_post_message.is_some()) + usize::from(self.price.is_some());
        let mut object = serializer.serialize_map(Some(extra_len + optional_len + 1))?;
        serialize_optional_field(
            &mut object,
            "suggested_post_message",
            &self.suggested_post_message,
        )?;
        serialize_optional_field(&mut object, "price", &self.price)?;
        object.serialize_entry("send_date", &self.send_date)?;
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
}

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct SuggestedPostApprovalFailed {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suggested_post_message: Option<Box<Message>>,
    pub price: SuggestedPostPrice,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl Serialize for SuggestedPostApprovalFailed {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = ["suggested_post_message", "price"];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let optional_len = usize::from(self.suggested_post_message.is_some());
        let mut object = serializer.serialize_map(Some(extra_len + optional_len + 1))?;
        serialize_optional_field(
            &mut object,
            "suggested_post_message",
            &self.suggested_post_message,
        )?;
        object.serialize_entry("price", &self.price)?;
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
}

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct SuggestedPostDeclined {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suggested_post_message: Option<Box<Message>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl Serialize for SuggestedPostDeclined {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = ["suggested_post_message", "comment"];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let optional_len = usize::from(self.suggested_post_message.is_some())
            + usize::from(self.comment.is_some());
        let mut object = serializer.serialize_map(Some(extra_len + optional_len))?;
        serialize_optional_field(
            &mut object,
            "suggested_post_message",
            &self.suggested_post_message,
        )?;
        serialize_optional_field(&mut object, "comment", &self.comment)?;
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
}

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct SuggestedPostPaid {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suggested_post_message: Option<Box<Message>>,
    pub currency: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub amount: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub star_amount: Option<StarAmount>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl Serialize for SuggestedPostPaid {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = [
            "suggested_post_message",
            "currency",
            "amount",
            "star_amount",
        ];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let optional_len = usize::from(self.suggested_post_message.is_some())
            + usize::from(self.amount.is_some())
            + usize::from(self.star_amount.is_some());
        let mut object = serializer.serialize_map(Some(extra_len + optional_len + 1))?;
        serialize_optional_field(
            &mut object,
            "suggested_post_message",
            &self.suggested_post_message,
        )?;
        object.serialize_entry("currency", &self.currency)?;
        serialize_optional_field(&mut object, "amount", &self.amount)?;
        serialize_optional_field(&mut object, "star_amount", &self.star_amount)?;
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
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

impl Serialize for SuggestedPostRefundReason {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct SuggestedPostRefunded {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suggested_post_message: Option<Box<Message>>,
    pub reason: SuggestedPostRefundReason,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl Serialize for SuggestedPostRefunded {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = ["suggested_post_message", "reason"];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let optional_len = usize::from(self.suggested_post_message.is_some());
        let mut object = serializer.serialize_map(Some(extra_len + optional_len + 1))?;
        serialize_optional_field(
            &mut object,
            "suggested_post_message",
            &self.suggested_post_message,
        )?;
        object.serialize_entry("reason", &self.reason)?;
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn suggested_post_price_and_info_extra_cannot_override_reserved_fields()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut price: SuggestedPostPrice = serde_json::from_value(json!({
            "currency": "USD",
            "amount": 1000,
            "future": {"kept": true}
        }))?;
        price.extra.insert("currency".to_owned(), json!("EUR"));
        price.extra.insert("amount".to_owned(), json!(1));
        price
            .extra
            .insert("another_future".to_owned(), json!("kept"));

        let price_value = serde_json::to_value(price)?;
        assert_eq!(price_value["currency"], "USD");
        assert_eq!(price_value["amount"], 1000);
        assert_eq!(price_value["future"], json!({"kept": true}));
        assert_eq!(price_value["another_future"], "kept");

        let mut info: SuggestedPostInfo = serde_json::from_value(json!({
            "state": "approved",
            "future": {"kept": true}
        }))?;
        info.extra.insert("state".to_owned(), json!("declined"));
        info.extra
            .insert("price".to_owned(), json!({"currency": "EUR", "amount": 1}));
        info.extra.insert("send_date".to_owned(), json!(1));
        info.extra
            .insert("another_future".to_owned(), json!("kept"));

        let info_value = serde_json::to_value(info)?;
        assert_eq!(info_value["state"], "approved");
        assert!(info_value.get("price").is_none());
        assert!(info_value.get("send_date").is_none());
        assert_eq!(info_value["future"], json!({"kept": true}));
        assert_eq!(info_value["another_future"], "kept");
        Ok(())
    }

    #[test]
    fn suggested_post_lifecycle_extra_cannot_override_reserved_fields()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut approved: SuggestedPostApproved = serde_json::from_value(json!({
            "send_date": 20,
            "future": {"kept": true}
        }))?;
        approved.extra.insert("send_date".to_owned(), json!(1));
        approved
            .extra
            .insert("price".to_owned(), json!({"currency": "EUR", "amount": 1}));
        approved
            .extra
            .insert("another_future".to_owned(), json!("kept"));

        let approved_value = serde_json::to_value(approved)?;
        assert_eq!(approved_value["send_date"], 20);
        assert!(approved_value.get("price").is_none());
        assert_eq!(approved_value["future"], json!({"kept": true}));
        assert_eq!(approved_value["another_future"], "kept");

        let mut failed: SuggestedPostApprovalFailed = serde_json::from_value(json!({
            "price": {"currency": "USD", "amount": 1000},
            "future": {"kept": true}
        }))?;
        failed
            .extra
            .insert("price".to_owned(), json!({"currency": "EUR", "amount": 1}));
        failed.extra.insert(
            "suggested_post_message".to_owned(),
            json!({"message_id": 1}),
        );
        failed
            .extra
            .insert("another_future".to_owned(), json!("kept"));

        let failed_value = serde_json::to_value(failed)?;
        assert_eq!(failed_value["price"]["currency"], "USD");
        assert_eq!(failed_value["price"]["amount"], 1000);
        assert!(failed_value.get("suggested_post_message").is_none());
        assert_eq!(failed_value["future"], json!({"kept": true}));
        assert_eq!(failed_value["another_future"], "kept");

        let mut declined: SuggestedPostDeclined = serde_json::from_value(json!({
            "comment": "No",
            "future": {"kept": true}
        }))?;
        declined
            .extra
            .insert("comment".to_owned(), json!("spoofed"));
        declined.extra.insert(
            "suggested_post_message".to_owned(),
            json!({"message_id": 1}),
        );
        declined
            .extra
            .insert("another_future".to_owned(), json!("kept"));

        let declined_value = serde_json::to_value(declined)?;
        assert_eq!(declined_value["comment"], "No");
        assert!(declined_value.get("suggested_post_message").is_none());
        assert_eq!(declined_value["future"], json!({"kept": true}));
        assert_eq!(declined_value["another_future"], "kept");
        Ok(())
    }

    #[test]
    fn suggested_post_paid_and_refunded_extra_cannot_override_reserved_fields()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut paid: SuggestedPostPaid = serde_json::from_value(json!({
            "currency": "USD",
            "amount": 1000,
            "future": {"kept": true}
        }))?;
        paid.extra.insert("currency".to_owned(), json!("EUR"));
        paid.extra.insert("amount".to_owned(), json!(1));
        paid.extra
            .insert("star_amount".to_owned(), json!({"amount": 1}));
        paid.extra
            .insert("another_future".to_owned(), json!("kept"));

        let paid_value = serde_json::to_value(paid)?;
        assert_eq!(paid_value["currency"], "USD");
        assert_eq!(paid_value["amount"], 1000);
        assert!(paid_value.get("star_amount").is_none());
        assert_eq!(paid_value["future"], json!({"kept": true}));
        assert_eq!(paid_value["another_future"], "kept");

        let mut refunded: SuggestedPostRefunded = serde_json::from_value(json!({
            "reason": "payment_refunded",
            "future": {"kept": true}
        }))?;
        refunded
            .extra
            .insert("reason".to_owned(), json!("post_deleted"));
        refunded.extra.insert(
            "suggested_post_message".to_owned(),
            json!({"message_id": 1}),
        );
        refunded
            .extra
            .insert("another_future".to_owned(), json!("kept"));

        let refunded_value = serde_json::to_value(refunded)?;
        assert_eq!(refunded_value["reason"], "payment_refunded");
        assert!(refunded_value.get("suggested_post_message").is_none());
        assert_eq!(refunded_value["future"], json!({"kept": true}));
        assert_eq!(refunded_value["another_future"], "kept");
        Ok(())
    }
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

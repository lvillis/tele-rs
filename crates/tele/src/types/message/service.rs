use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::bot::User;
use crate::types::common::{MessageId, UserId};

use super::common::{Chat, PhotoSize};
use super::content::ChecklistTask;
use super::is_false;
use super::model::Message;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ChecklistTasksDone {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checklist_message: Option<Box<Message>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub marked_as_done_task_ids: Option<Vec<i64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub marked_as_not_done_task_ids: Option<Vec<i64>>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ChecklistTasksAdded {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checklist_message: Option<Box<Message>>,
    pub tasks: Vec<ChecklistTask>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ChatOwnerLeft {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub new_owner: Option<User>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ChatOwnerChanged {
    pub new_owner: User,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ProximityAlertTriggered {
    pub traveler: User,
    pub watcher: User,
    pub distance: u32,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ChatBoostAdded {
    pub boost_count: u32,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct VideoChatScheduled {
    pub start_date: i64,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct VideoChatStarted {
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct VideoChatEnded {
    pub duration: u32,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct VideoChatParticipantsInvited {
    pub users: Vec<User>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct PaidMessagePriceChanged {
    pub paid_message_star_count: u64,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct DirectMessagePriceChanged {
    pub are_direct_messages_enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub direct_message_star_count: Option<u64>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct GiveawayCreated {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prize_star_count: Option<u64>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Giveaway {
    pub chats: Vec<Chat>,
    pub winners_selection_date: i64,
    pub winner_count: u32,
    #[serde(default, skip_serializing_if = "is_false")]
    pub only_new_members: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub has_public_winners: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prize_description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub country_codes: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prize_star_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub premium_subscription_month_count: Option<u32>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct GiveawayWinners {
    pub chat: Chat,
    pub giveaway_message_id: MessageId,
    pub winners_selection_date: i64,
    pub winner_count: u32,
    pub winners: Vec<User>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub additional_chat_count: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prize_star_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub premium_subscription_month_count: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unclaimed_prize_count: Option<u32>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub only_new_members: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub was_refunded: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prize_description: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct GiveawayCompleted {
    pub winner_count: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unclaimed_prize_count: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub giveaway_message: Option<Box<Message>>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub is_star_giveaway: bool,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct WriteAccessAllowed {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_request: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub web_app_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_attachment_menu: Option<bool>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct MessageAutoDeleteTimerChanged {
    pub message_auto_delete_time: u32,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct SharedUser {
    pub user_id: UserId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub photo: Option<Vec<PhotoSize>>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct UsersShared {
    pub request_id: i64,
    pub users: Vec<SharedUser>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ChatShared {
    pub request_id: i64,
    pub chat_id: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub photo: Option<Vec<PhotoSize>>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

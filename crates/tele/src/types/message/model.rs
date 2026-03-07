use std::collections::BTreeMap;

use serde::de::{DeserializeOwned, Error as DeError};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::bot::User;
use crate::types::common::MessageId;
use crate::types::sticker::Sticker;
use crate::types::telegram::{LinkPreviewOptions, ReplyMarkup, WebAppData};

use super::common::{Chat, MessageEntity, MessageOrigin, PhotoSize};
use super::content::{Checklist, Contact, Dice, Game, Location, Poll, Venue};
use super::forum::{
    ForumTopicClosed, ForumTopicCreated, ForumTopicEdited, ForumTopicReopened,
    GeneralForumTopicHidden, GeneralForumTopicUnhidden,
};
use super::is_false;
use super::media::{Animation, Audio, Document, PaidMediaInfo, Story, Video, VideoNote, Voice};
use super::metadata::{
    KNOWN_MESSAGE_KINDS, MessageKind, SuggestedPostApprovalFailed, SuggestedPostApproved,
    SuggestedPostDeclined, SuggestedPostInfo, SuggestedPostPaid, SuggestedPostRefunded,
};
use super::payments::{Invoice, RefundedPayment, SuccessfulPayment};
use super::reply::{ExternalReplyInfo, MaybeInaccessibleMessage, TextQuote};
use super::service::{
    ChatBoostAdded, ChatOwnerChanged, ChatOwnerLeft, ChatShared, ChecklistTasksAdded,
    ChecklistTasksDone, DirectMessagePriceChanged, Giveaway, GiveawayCompleted, GiveawayCreated,
    GiveawayWinners, MessageAutoDeleteTimerChanged, PaidMessagePriceChanged,
    ProximityAlertTriggered, UsersShared, VideoChatEnded, VideoChatParticipantsInvited,
    VideoChatScheduled, VideoChatStarted, WriteAccessAllowed,
};

#[derive(Clone, Debug, Serialize)]
#[non_exhaustive]
pub struct Message {
    pub message_id: MessageId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from: Option<User>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sender_chat: Option<Chat>,
    pub chat: Chat,
    pub date: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author_signature: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sender_tag: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_thread_id: Option<i64>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub is_topic_message: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub forward_origin: Option<MessageOrigin>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub is_automatic_forward: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_to_message: Option<Box<MaybeInaccessibleMessage>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_reply: Option<Box<ExternalReplyInfo>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quote: Option<TextQuote>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_to_story: Option<Story>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_to_checklist_task_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub via_bot: Option<User>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub edit_date: Option<i64>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub has_protected_content: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub is_from_offline: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub is_paid_post: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub media_group_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub paid_star_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entities: Option<Vec<MessageEntity>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caption_entities: Option<Vec<MessageEntity>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub link_preview_options: Option<LinkPreviewOptions>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suggested_post_info: Option<Box<SuggestedPostInfo>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effect_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub animation: Option<Animation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio: Option<Audio>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contact: Option<Contact>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dice: Option<Dice>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub document: Option<Document>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub paid_media: Option<Box<PaidMediaInfo>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<Location>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub photo: Option<Vec<PhotoSize>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sticker: Option<Sticker>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub story: Option<Story>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub venue: Option<Venue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub video: Option<Video>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub video_note: Option<VideoNote>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub voice: Option<Voice>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub poll: Option<Box<Poll>>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub show_caption_above_media: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub has_media_spoiler: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checklist: Option<Box<Checklist>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub game: Option<Box<Game>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub web_app_data: Option<WebAppData>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub write_access_allowed: Option<WriteAccessAllowed>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub new_chat_members: Option<Vec<User>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub left_chat_member: Option<User>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chat_owner_left: Option<ChatOwnerLeft>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chat_owner_changed: Option<ChatOwnerChanged>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub new_chat_title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub new_chat_photo: Option<Vec<PhotoSize>>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub delete_chat_photo: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub group_chat_created: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub supergroup_chat_created: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub channel_chat_created: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pinned_message: Option<Box<MaybeInaccessibleMessage>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_auto_delete_timer_changed: Option<MessageAutoDeleteTimerChanged>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub migrate_to_chat_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub migrate_from_chat_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub invoice: Option<Box<Invoice>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub successful_payment: Option<Box<SuccessfulPayment>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refunded_payment: Option<Box<RefundedPayment>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub users_shared: Option<UsersShared>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chat_shared: Option<ChatShared>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub connected_website: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proximity_alert_triggered: Option<Box<ProximityAlertTriggered>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub boost_added: Option<Box<ChatBoostAdded>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checklist_tasks_done: Option<Box<ChecklistTasksDone>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checklist_tasks_added: Option<Box<ChecklistTasksAdded>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub direct_message_price_changed: Option<Box<DirectMessagePriceChanged>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub forum_topic_created: Option<ForumTopicCreated>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub forum_topic_edited: Option<ForumTopicEdited>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub forum_topic_closed: Option<ForumTopicClosed>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub forum_topic_reopened: Option<ForumTopicReopened>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub general_forum_topic_hidden: Option<GeneralForumTopicHidden>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub general_forum_topic_unhidden: Option<GeneralForumTopicUnhidden>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub giveaway_created: Option<Box<GiveawayCreated>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub giveaway: Option<Box<Giveaway>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub giveaway_winners: Option<Box<GiveawayWinners>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub giveaway_completed: Option<Box<GiveawayCompleted>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub paid_message_price_changed: Option<Box<PaidMessagePriceChanged>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suggested_post_approved: Option<Box<SuggestedPostApproved>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suggested_post_approval_failed: Option<Box<SuggestedPostApprovalFailed>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suggested_post_declined: Option<Box<SuggestedPostDeclined>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suggested_post_paid: Option<Box<SuggestedPostPaid>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suggested_post_refunded: Option<Box<SuggestedPostRefunded>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub video_chat_scheduled: Option<VideoChatScheduled>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub video_chat_started: Option<VideoChatStarted>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub video_chat_ended: Option<VideoChatEnded>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub video_chat_participants_invited: Option<VideoChatParticipantsInvited>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_markup: Option<Box<ReplyMarkup>>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

fn take_required_field<T, E>(
    object: &mut serde_json::Map<String, Value>,
    field: &'static str,
) -> std::result::Result<T, E>
where
    T: DeserializeOwned,
    E: DeError,
{
    let value = object
        .remove(field)
        .ok_or_else(|| E::missing_field(field))?;
    serde_json::from_value(value).map_err(E::custom)
}

fn take_optional_field<T, E>(
    object: &mut serde_json::Map<String, Value>,
    field: &'static str,
) -> std::result::Result<Option<T>, E>
where
    T: DeserializeOwned,
    E: DeError,
{
    object
        .remove(field)
        .map(|value| serde_json::from_value(value).map_err(E::custom))
        .transpose()
}

fn take_bool_field<E>(
    object: &mut serde_json::Map<String, Value>,
    field: &'static str,
) -> std::result::Result<bool, E>
where
    E: DeError,
{
    Ok(take_optional_field::<bool, E>(object, field)?.unwrap_or(false))
}

impl<'de> Deserialize<'de> for Message {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut object = serde_json::Map::<String, Value>::deserialize(deserializer)?;

        Ok(Self {
            message_id: take_required_field(&mut object, "message_id")?,
            from: take_optional_field(&mut object, "from")?,
            sender_chat: take_optional_field(&mut object, "sender_chat")?,
            chat: take_required_field(&mut object, "chat")?,
            date: take_required_field(&mut object, "date")?,
            author_signature: take_optional_field(&mut object, "author_signature")?,
            sender_tag: take_optional_field(&mut object, "sender_tag")?,
            message_thread_id: take_optional_field(&mut object, "message_thread_id")?,
            is_topic_message: take_bool_field(&mut object, "is_topic_message")?,
            forward_origin: take_optional_field(&mut object, "forward_origin")?,
            is_automatic_forward: take_bool_field(&mut object, "is_automatic_forward")?,
            reply_to_message: take_optional_field(&mut object, "reply_to_message")?,
            external_reply: take_optional_field(&mut object, "external_reply")?,
            quote: take_optional_field(&mut object, "quote")?,
            reply_to_story: take_optional_field(&mut object, "reply_to_story")?,
            reply_to_checklist_task_id: take_optional_field(
                &mut object,
                "reply_to_checklist_task_id",
            )?,
            via_bot: take_optional_field(&mut object, "via_bot")?,
            edit_date: take_optional_field(&mut object, "edit_date")?,
            has_protected_content: take_bool_field(&mut object, "has_protected_content")?,
            is_from_offline: take_bool_field(&mut object, "is_from_offline")?,
            is_paid_post: take_bool_field(&mut object, "is_paid_post")?,
            media_group_id: take_optional_field(&mut object, "media_group_id")?,
            paid_star_count: take_optional_field(&mut object, "paid_star_count")?,
            text: take_optional_field(&mut object, "text")?,
            caption: take_optional_field(&mut object, "caption")?,
            entities: take_optional_field(&mut object, "entities")?,
            caption_entities: take_optional_field(&mut object, "caption_entities")?,
            link_preview_options: take_optional_field(&mut object, "link_preview_options")?,
            suggested_post_info: take_optional_field(&mut object, "suggested_post_info")?,
            effect_id: take_optional_field(&mut object, "effect_id")?,
            animation: take_optional_field(&mut object, "animation")?,
            audio: take_optional_field(&mut object, "audio")?,
            contact: take_optional_field(&mut object, "contact")?,
            dice: take_optional_field(&mut object, "dice")?,
            document: take_optional_field(&mut object, "document")?,
            paid_media: take_optional_field(&mut object, "paid_media")?,
            location: take_optional_field(&mut object, "location")?,
            photo: take_optional_field(&mut object, "photo")?,
            sticker: take_optional_field(&mut object, "sticker")?,
            story: take_optional_field(&mut object, "story")?,
            venue: take_optional_field(&mut object, "venue")?,
            video: take_optional_field(&mut object, "video")?,
            video_note: take_optional_field(&mut object, "video_note")?,
            voice: take_optional_field(&mut object, "voice")?,
            poll: take_optional_field(&mut object, "poll")?,
            show_caption_above_media: take_bool_field(&mut object, "show_caption_above_media")?,
            has_media_spoiler: take_bool_field(&mut object, "has_media_spoiler")?,
            checklist: take_optional_field(&mut object, "checklist")?,
            game: take_optional_field(&mut object, "game")?,
            web_app_data: take_optional_field(&mut object, "web_app_data")?,
            write_access_allowed: take_optional_field(&mut object, "write_access_allowed")?,
            new_chat_members: take_optional_field(&mut object, "new_chat_members")?,
            left_chat_member: take_optional_field(&mut object, "left_chat_member")?,
            chat_owner_left: take_optional_field(&mut object, "chat_owner_left")?,
            chat_owner_changed: take_optional_field(&mut object, "chat_owner_changed")?,
            new_chat_title: take_optional_field(&mut object, "new_chat_title")?,
            new_chat_photo: take_optional_field(&mut object, "new_chat_photo")?,
            delete_chat_photo: take_bool_field(&mut object, "delete_chat_photo")?,
            group_chat_created: take_bool_field(&mut object, "group_chat_created")?,
            supergroup_chat_created: take_bool_field(&mut object, "supergroup_chat_created")?,
            channel_chat_created: take_bool_field(&mut object, "channel_chat_created")?,
            pinned_message: take_optional_field(&mut object, "pinned_message")?,
            message_auto_delete_timer_changed: take_optional_field(
                &mut object,
                "message_auto_delete_timer_changed",
            )?,
            migrate_to_chat_id: take_optional_field(&mut object, "migrate_to_chat_id")?,
            migrate_from_chat_id: take_optional_field(&mut object, "migrate_from_chat_id")?,
            invoice: take_optional_field(&mut object, "invoice")?,
            successful_payment: take_optional_field(&mut object, "successful_payment")?,
            refunded_payment: take_optional_field(&mut object, "refunded_payment")?,
            users_shared: take_optional_field(&mut object, "users_shared")?,
            chat_shared: take_optional_field(&mut object, "chat_shared")?,
            connected_website: take_optional_field(&mut object, "connected_website")?,
            proximity_alert_triggered: take_optional_field(
                &mut object,
                "proximity_alert_triggered",
            )?,
            boost_added: take_optional_field(&mut object, "boost_added")?,
            checklist_tasks_done: take_optional_field(&mut object, "checklist_tasks_done")?,
            checklist_tasks_added: take_optional_field(&mut object, "checklist_tasks_added")?,
            direct_message_price_changed: take_optional_field(
                &mut object,
                "direct_message_price_changed",
            )?,
            forum_topic_created: take_optional_field(&mut object, "forum_topic_created")?,
            forum_topic_edited: take_optional_field(&mut object, "forum_topic_edited")?,
            forum_topic_closed: take_optional_field(&mut object, "forum_topic_closed")?,
            forum_topic_reopened: take_optional_field(&mut object, "forum_topic_reopened")?,
            general_forum_topic_hidden: take_optional_field(
                &mut object,
                "general_forum_topic_hidden",
            )?,
            general_forum_topic_unhidden: take_optional_field(
                &mut object,
                "general_forum_topic_unhidden",
            )?,
            giveaway_created: take_optional_field(&mut object, "giveaway_created")?,
            giveaway: take_optional_field(&mut object, "giveaway")?,
            giveaway_winners: take_optional_field(&mut object, "giveaway_winners")?,
            giveaway_completed: take_optional_field(&mut object, "giveaway_completed")?,
            paid_message_price_changed: take_optional_field(
                &mut object,
                "paid_message_price_changed",
            )?,
            suggested_post_approved: take_optional_field(&mut object, "suggested_post_approved")?,
            suggested_post_approval_failed: take_optional_field(
                &mut object,
                "suggested_post_approval_failed",
            )?,
            suggested_post_declined: take_optional_field(&mut object, "suggested_post_declined")?,
            suggested_post_paid: take_optional_field(&mut object, "suggested_post_paid")?,
            suggested_post_refunded: take_optional_field(&mut object, "suggested_post_refunded")?,
            video_chat_scheduled: take_optional_field(&mut object, "video_chat_scheduled")?,
            video_chat_started: take_optional_field(&mut object, "video_chat_started")?,
            video_chat_ended: take_optional_field(&mut object, "video_chat_ended")?,
            video_chat_participants_invited: take_optional_field(
                &mut object,
                "video_chat_participants_invited",
            )?,
            reply_markup: take_optional_field(&mut object, "reply_markup")?,
            extra: object.into_iter().collect(),
        })
    }
}

fn is_unmodeled_message_content_key(key: &str) -> bool {
    matches!(
        key,
        "gift" | "unique_gift" | "gift_upgrade_sent" | "passport_data" | "chat_background_set"
    )
}

impl Message {
    pub fn chat(&self) -> &Chat {
        &self.chat
    }

    pub fn from_user(&self) -> Option<&User> {
        self.from.as_ref()
    }

    pub fn sender_chat(&self) -> Option<&Chat> {
        self.sender_chat.as_ref()
    }

    pub fn reply_to_message(&self) -> Option<&MaybeInaccessibleMessage> {
        self.reply_to_message.as_deref()
    }

    pub fn pinned_message(&self) -> Option<&MaybeInaccessibleMessage> {
        self.pinned_message.as_deref()
    }

    fn has_modeled_kind(&self) -> bool {
        self.write_access_allowed.is_some()
            || self.web_app_data.is_some()
            || self.connected_website.is_some()
            || self.poll.is_some()
            || self.paid_media.is_some()
            || self.checklist.is_some()
            || self.game.is_some()
            || self.invoice.is_some()
            || self.successful_payment.is_some()
            || self.refunded_payment.is_some()
            || self.new_chat_members.is_some()
            || self.left_chat_member.is_some()
            || self.chat_owner_left.is_some()
            || self.chat_owner_changed.is_some()
            || self.new_chat_title.is_some()
            || self.new_chat_photo.is_some()
            || self.delete_chat_photo
            || self.group_chat_created
            || self.supergroup_chat_created
            || self.channel_chat_created
            || self.pinned_message.is_some()
            || self.message_auto_delete_timer_changed.is_some()
            || self.migrate_to_chat_id.is_some()
            || self.migrate_from_chat_id.is_some()
            || self.users_shared.is_some()
            || self.chat_shared.is_some()
            || self.proximity_alert_triggered.is_some()
            || self.boost_added.is_some()
            || self.checklist_tasks_done.is_some()
            || self.checklist_tasks_added.is_some()
            || self.direct_message_price_changed.is_some()
            || self.forum_topic_created.is_some()
            || self.forum_topic_edited.is_some()
            || self.forum_topic_closed.is_some()
            || self.forum_topic_reopened.is_some()
            || self.general_forum_topic_hidden.is_some()
            || self.general_forum_topic_unhidden.is_some()
            || self.giveaway_created.is_some()
            || self.giveaway.is_some()
            || self.giveaway_winners.is_some()
            || self.giveaway_completed.is_some()
            || self.paid_message_price_changed.is_some()
            || self.suggested_post_approved.is_some()
            || self.suggested_post_approval_failed.is_some()
            || self.suggested_post_declined.is_some()
            || self.suggested_post_paid.is_some()
            || self.suggested_post_refunded.is_some()
            || self.video_chat_scheduled.is_some()
            || self.video_chat_started.is_some()
            || self.video_chat_ended.is_some()
            || self.video_chat_participants_invited.is_some()
            || self.animation.is_some()
            || self.audio.is_some()
            || self.contact.is_some()
            || self.dice.is_some()
            || self.document.is_some()
            || self.location.is_some()
            || self.photo.is_some()
            || self.sticker.is_some()
            || self.story.is_some()
            || self.venue.is_some()
            || self.video.is_some()
            || self.video_note.is_some()
            || self.voice.is_some()
            || self.text.is_some()
            || self.caption.is_some()
    }

    fn has_unmodeled_content(&self) -> bool {
        self.extra
            .keys()
            .any(|key| is_unmodeled_message_content_key(key))
    }

    pub fn web_app_data(&self) -> Option<&WebAppData> {
        self.web_app_data.as_ref()
    }

    pub fn write_access_allowed(&self) -> Option<&WriteAccessAllowed> {
        self.write_access_allowed.as_ref()
    }

    pub fn forward_origin(&self) -> Option<&MessageOrigin> {
        self.forward_origin.as_ref()
    }

    pub fn is_automatic_forward(&self) -> bool {
        self.is_automatic_forward
    }

    pub fn kind(&self) -> MessageKind {
        for &kind in KNOWN_MESSAGE_KINDS {
            if self.has_kind(kind) {
                return kind;
            }
        }

        MessageKind::Unknown
    }

    pub fn kinds(&self) -> Vec<MessageKind> {
        let mut kinds = Vec::with_capacity(KNOWN_MESSAGE_KINDS.len() + 1);
        for &kind in KNOWN_MESSAGE_KINDS {
            if self.has_kind(kind) {
                kinds.push(kind);
            }
        }

        if self.has_kind(MessageKind::Unknown) {
            kinds.push(MessageKind::Unknown);
        }

        kinds
    }

    pub fn has_kind(&self, kind: MessageKind) -> bool {
        match kind {
            MessageKind::WriteAccessAllowed => self.write_access_allowed.is_some(),
            MessageKind::WebAppData => self.web_app_data.is_some(),
            MessageKind::ConnectedWebsite => self.connected_website.is_some(),
            MessageKind::Poll => self.poll.is_some(),
            MessageKind::PaidMedia => self.paid_media.is_some(),
            MessageKind::Checklist => self.checklist.is_some(),
            MessageKind::Game => self.game.is_some(),
            MessageKind::Invoice => self.invoice.is_some(),
            MessageKind::SuccessfulPayment => self.successful_payment.is_some(),
            MessageKind::RefundedPayment => self.refunded_payment.is_some(),
            MessageKind::NewChatMembers => self.new_chat_members.is_some(),
            MessageKind::LeftChatMember => self.left_chat_member.is_some(),
            MessageKind::ChatOwnerLeft => self.chat_owner_left.is_some(),
            MessageKind::ChatOwnerChanged => self.chat_owner_changed.is_some(),
            MessageKind::NewChatTitle => self.new_chat_title.is_some(),
            MessageKind::NewChatPhoto => self.new_chat_photo.is_some(),
            MessageKind::DeleteChatPhoto => self.delete_chat_photo,
            MessageKind::GroupChatCreated => self.group_chat_created,
            MessageKind::SupergroupChatCreated => self.supergroup_chat_created,
            MessageKind::ChannelChatCreated => self.channel_chat_created,
            MessageKind::PinnedMessage => self.pinned_message.is_some(),
            MessageKind::MessageAutoDeleteTimerChanged => {
                self.message_auto_delete_timer_changed.is_some()
            }
            MessageKind::MigrateToChat => self.migrate_to_chat_id.is_some(),
            MessageKind::MigrateFromChat => self.migrate_from_chat_id.is_some(),
            MessageKind::UsersShared => self.users_shared.is_some(),
            MessageKind::ChatShared => self.chat_shared.is_some(),
            MessageKind::ProximityAlertTriggered => self.proximity_alert_triggered.is_some(),
            MessageKind::BoostAdded => self.boost_added.is_some(),
            MessageKind::ChecklistTasksDone => self.checklist_tasks_done.is_some(),
            MessageKind::ChecklistTasksAdded => self.checklist_tasks_added.is_some(),
            MessageKind::DirectMessagePriceChanged => self.direct_message_price_changed.is_some(),
            MessageKind::ForumTopicCreated => self.forum_topic_created.is_some(),
            MessageKind::ForumTopicEdited => self.forum_topic_edited.is_some(),
            MessageKind::ForumTopicClosed => self.forum_topic_closed.is_some(),
            MessageKind::ForumTopicReopened => self.forum_topic_reopened.is_some(),
            MessageKind::GeneralForumTopicHidden => self.general_forum_topic_hidden.is_some(),
            MessageKind::GeneralForumTopicUnhidden => self.general_forum_topic_unhidden.is_some(),
            MessageKind::GiveawayCreated => self.giveaway_created.is_some(),
            MessageKind::Giveaway => self.giveaway.is_some(),
            MessageKind::GiveawayWinners => self.giveaway_winners.is_some(),
            MessageKind::GiveawayCompleted => self.giveaway_completed.is_some(),
            MessageKind::PaidMessagePriceChanged => self.paid_message_price_changed.is_some(),
            MessageKind::SuggestedPostApproved => self.suggested_post_approved.is_some(),
            MessageKind::SuggestedPostApprovalFailed => {
                self.suggested_post_approval_failed.is_some()
            }
            MessageKind::SuggestedPostDeclined => self.suggested_post_declined.is_some(),
            MessageKind::SuggestedPostPaid => self.suggested_post_paid.is_some(),
            MessageKind::SuggestedPostRefunded => self.suggested_post_refunded.is_some(),
            MessageKind::VideoChatScheduled => self.video_chat_scheduled.is_some(),
            MessageKind::VideoChatStarted => self.video_chat_started.is_some(),
            MessageKind::VideoChatEnded => self.video_chat_ended.is_some(),
            MessageKind::VideoChatParticipantsInvited => {
                self.video_chat_participants_invited.is_some()
            }
            MessageKind::Animation => self.animation.is_some(),
            MessageKind::Audio => self.audio.is_some(),
            MessageKind::Contact => self.contact.is_some(),
            MessageKind::Dice => self.dice.is_some(),
            MessageKind::Document => self.document.is_some(),
            MessageKind::Location => self.location.is_some(),
            MessageKind::Photo => self.photo.is_some(),
            MessageKind::Sticker => self.sticker.is_some(),
            MessageKind::Story => self.story.is_some(),
            MessageKind::Venue => self.venue.is_some(),
            MessageKind::Video => self.video.is_some(),
            MessageKind::VideoNote => self.video_note.is_some(),
            MessageKind::Voice => self.voice.is_some(),
            MessageKind::Text => self.text.is_some(),
            MessageKind::Caption => self.caption.is_some(),
            MessageKind::Unknown => self.has_unmodeled_content() || !self.has_modeled_kind(),
        }
    }
}

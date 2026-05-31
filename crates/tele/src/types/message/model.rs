use std::collections::BTreeMap;

use serde::Deserialize;
use serde_json::Value;

use crate::types::bot::User;
use crate::types::common::MessageId;
use crate::types::gift::{GiftInfo, UniqueGiftInfo};
use crate::types::sticker::Sticker;
use crate::types::telegram::{LinkPreviewOptions, ReplyMarkup, WebAppData};

use super::common::{Chat, MessageEntity, MessageOrigin, PhotoSize};
use super::content::{Checklist, Contact, Dice, Game, Location, Poll, Venue};
use super::forum::{
    ForumTopicClosed, ForumTopicCreated, ForumTopicEdited, ForumTopicReopened,
    GeneralForumTopicHidden, GeneralForumTopicUnhidden,
};
use super::media::{
    Animation, Audio, Document, LivePhoto, PaidMediaInfo, Story, Video, VideoNote, Voice,
};
use super::metadata::{
    KNOWN_MESSAGE_KINDS, MessageKind, SuggestedPostApprovalFailed, SuggestedPostApproved,
    SuggestedPostDeclined, SuggestedPostInfo, SuggestedPostPaid, SuggestedPostRefunded,
};
use super::payments::{Invoice, RefundedPayment, SuccessfulPayment};
use super::reply::{ExternalReplyInfo, MaybeInaccessibleMessage, TextQuote};
use super::service::{
    ChatBackground, ChatBoostAdded, ChatOwnerChanged, ChatOwnerLeft, ChatShared,
    ChecklistTasksAdded, ChecklistTasksDone, DirectMessagePriceChanged, DirectMessagesTopic,
    Giveaway, GiveawayCompleted, GiveawayCreated, GiveawayWinners, ManagedBotCreated,
    MessageAutoDeleteTimerChanged, PaidMessagePriceChanged, PollOptionAdded, PollOptionDeleted,
    ProximityAlertTriggered, UsersShared, VideoChatEnded, VideoChatParticipantsInvited,
    VideoChatScheduled, VideoChatStarted, WriteAccessAllowed,
};

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct Message {
    pub message_id: MessageId,
    #[serde(default)]
    pub from: Option<User>,
    #[serde(default)]
    pub sender_chat: Option<Chat>,
    #[serde(default)]
    pub sender_boost_count: Option<u32>,
    pub chat: Chat,
    pub date: i64,
    #[serde(default)]
    pub guest_query_id: Option<String>,
    #[serde(default)]
    pub business_connection_id: Option<String>,
    #[serde(default)]
    pub author_signature: Option<String>,
    #[serde(default)]
    pub sender_business_bot: Option<User>,
    #[serde(default)]
    pub sender_tag: Option<String>,
    #[serde(default)]
    pub message_thread_id: Option<i64>,
    #[serde(default)]
    pub direct_messages_topic: Option<DirectMessagesTopic>,
    #[serde(default)]
    pub is_topic_message: bool,
    #[serde(default)]
    pub forward_origin: Option<MessageOrigin>,
    #[serde(default)]
    pub is_automatic_forward: bool,
    #[serde(default)]
    pub reply_to_message: Option<Box<MaybeInaccessibleMessage>>,
    #[serde(default)]
    pub external_reply: Option<Box<ExternalReplyInfo>>,
    #[serde(default)]
    pub quote: Option<TextQuote>,
    #[serde(default)]
    pub reply_to_story: Option<Story>,
    #[serde(default)]
    pub reply_to_checklist_task_id: Option<i64>,
    #[serde(default)]
    pub reply_to_poll_option_id: Option<String>,
    #[serde(default)]
    pub via_bot: Option<User>,
    #[serde(default)]
    pub guest_bot_caller_user: Option<User>,
    #[serde(default)]
    pub guest_bot_caller_chat: Option<Chat>,
    #[serde(default)]
    pub edit_date: Option<i64>,
    #[serde(default)]
    pub has_protected_content: bool,
    #[serde(default)]
    pub is_from_offline: bool,
    #[serde(default)]
    pub is_paid_post: bool,
    #[serde(default)]
    pub media_group_id: Option<String>,
    #[serde(default)]
    pub paid_star_count: Option<u64>,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub caption: Option<String>,
    #[serde(default)]
    pub entities: Option<Vec<MessageEntity>>,
    #[serde(default)]
    pub caption_entities: Option<Vec<MessageEntity>>,
    #[serde(default)]
    pub link_preview_options: Option<LinkPreviewOptions>,
    #[serde(default)]
    pub suggested_post_info: Option<Box<SuggestedPostInfo>>,
    #[serde(default)]
    pub effect_id: Option<String>,
    #[serde(default)]
    pub animation: Option<Animation>,
    #[serde(default)]
    pub audio: Option<Audio>,
    #[serde(default)]
    pub contact: Option<Contact>,
    #[serde(default)]
    pub dice: Option<Dice>,
    #[serde(default)]
    pub document: Option<Document>,
    #[serde(default)]
    pub live_photo: Option<LivePhoto>,
    #[serde(default)]
    pub paid_media: Option<Box<PaidMediaInfo>>,
    #[serde(default)]
    pub location: Option<Location>,
    #[serde(default)]
    pub photo: Option<Vec<PhotoSize>>,
    #[serde(default)]
    pub sticker: Option<Sticker>,
    #[serde(default)]
    pub story: Option<Story>,
    #[serde(default)]
    pub venue: Option<Venue>,
    #[serde(default)]
    pub video: Option<Video>,
    #[serde(default)]
    pub video_note: Option<VideoNote>,
    #[serde(default)]
    pub voice: Option<Voice>,
    #[serde(default)]
    pub poll: Option<Box<Poll>>,
    #[serde(default)]
    pub show_caption_above_media: bool,
    #[serde(default)]
    pub has_media_spoiler: bool,
    #[serde(default)]
    pub checklist: Option<Box<Checklist>>,
    #[serde(default)]
    pub game: Option<Box<Game>>,
    #[serde(default)]
    pub web_app_data: Option<WebAppData>,
    #[serde(default)]
    pub write_access_allowed: Option<WriteAccessAllowed>,
    #[serde(default)]
    pub new_chat_members: Option<Vec<User>>,
    #[serde(default)]
    pub left_chat_member: Option<User>,
    #[serde(default)]
    pub chat_owner_left: Option<ChatOwnerLeft>,
    #[serde(default)]
    pub chat_owner_changed: Option<ChatOwnerChanged>,
    #[serde(default)]
    pub new_chat_title: Option<String>,
    #[serde(default)]
    pub new_chat_photo: Option<Vec<PhotoSize>>,
    #[serde(default)]
    pub delete_chat_photo: bool,
    #[serde(default)]
    pub group_chat_created: bool,
    #[serde(default)]
    pub supergroup_chat_created: bool,
    #[serde(default)]
    pub channel_chat_created: bool,
    #[serde(default)]
    pub pinned_message: Option<Box<MaybeInaccessibleMessage>>,
    #[serde(default)]
    pub message_auto_delete_timer_changed: Option<MessageAutoDeleteTimerChanged>,
    #[serde(default)]
    pub migrate_to_chat_id: Option<i64>,
    #[serde(default)]
    pub migrate_from_chat_id: Option<i64>,
    #[serde(default)]
    pub invoice: Option<Box<Invoice>>,
    #[serde(default)]
    pub successful_payment: Option<Box<SuccessfulPayment>>,
    #[serde(default)]
    pub refunded_payment: Option<Box<RefundedPayment>>,
    #[serde(default)]
    pub gift: Option<Box<GiftInfo>>,
    #[serde(default)]
    pub unique_gift: Option<Box<UniqueGiftInfo>>,
    #[serde(default)]
    pub gift_upgrade_sent: Option<Box<GiftInfo>>,
    #[serde(default)]
    pub users_shared: Option<UsersShared>,
    #[serde(default)]
    pub chat_shared: Option<ChatShared>,
    #[serde(default)]
    pub connected_website: Option<String>,
    #[serde(default)]
    pub proximity_alert_triggered: Option<Box<ProximityAlertTriggered>>,
    #[serde(default)]
    pub boost_added: Option<Box<ChatBoostAdded>>,
    #[serde(default)]
    pub chat_background_set: Option<Box<ChatBackground>>,
    #[serde(default)]
    pub checklist_tasks_done: Option<Box<ChecklistTasksDone>>,
    #[serde(default)]
    pub checklist_tasks_added: Option<Box<ChecklistTasksAdded>>,
    #[serde(default)]
    pub direct_message_price_changed: Option<Box<DirectMessagePriceChanged>>,
    #[serde(default)]
    pub forum_topic_created: Option<ForumTopicCreated>,
    #[serde(default)]
    pub forum_topic_edited: Option<ForumTopicEdited>,
    #[serde(default)]
    pub forum_topic_closed: Option<ForumTopicClosed>,
    #[serde(default)]
    pub forum_topic_reopened: Option<ForumTopicReopened>,
    #[serde(default)]
    pub general_forum_topic_hidden: Option<GeneralForumTopicHidden>,
    #[serde(default)]
    pub general_forum_topic_unhidden: Option<GeneralForumTopicUnhidden>,
    #[serde(default)]
    pub giveaway_created: Option<Box<GiveawayCreated>>,
    #[serde(default)]
    pub giveaway: Option<Box<Giveaway>>,
    #[serde(default)]
    pub giveaway_winners: Option<Box<GiveawayWinners>>,
    #[serde(default)]
    pub giveaway_completed: Option<Box<GiveawayCompleted>>,
    #[serde(default)]
    pub managed_bot_created: Option<Box<ManagedBotCreated>>,
    #[serde(default)]
    pub paid_message_price_changed: Option<Box<PaidMessagePriceChanged>>,
    #[serde(default)]
    pub poll_option_added: Option<Box<PollOptionAdded>>,
    #[serde(default)]
    pub poll_option_deleted: Option<Box<PollOptionDeleted>>,
    #[serde(default)]
    pub suggested_post_approved: Option<Box<SuggestedPostApproved>>,
    #[serde(default)]
    pub suggested_post_approval_failed: Option<Box<SuggestedPostApprovalFailed>>,
    #[serde(default)]
    pub suggested_post_declined: Option<Box<SuggestedPostDeclined>>,
    #[serde(default)]
    pub suggested_post_paid: Option<Box<SuggestedPostPaid>>,
    #[serde(default)]
    pub suggested_post_refunded: Option<Box<SuggestedPostRefunded>>,
    #[serde(default)]
    pub video_chat_scheduled: Option<VideoChatScheduled>,
    #[serde(default)]
    pub video_chat_started: Option<VideoChatStarted>,
    #[serde(default)]
    pub video_chat_ended: Option<VideoChatEnded>,
    #[serde(default)]
    pub video_chat_participants_invited: Option<VideoChatParticipantsInvited>,
    #[serde(default)]
    pub reply_markup: Option<Box<ReplyMarkup>>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

fn is_unmodeled_message_content_key(key: &str) -> bool {
    matches!(key, "passport_data")
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
            || self.gift.is_some()
            || self.unique_gift.is_some()
            || self.gift_upgrade_sent.is_some()
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
            || self.chat_background_set.is_some()
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
            || self.managed_bot_created.is_some()
            || self.paid_message_price_changed.is_some()
            || self.poll_option_added.is_some()
            || self.poll_option_deleted.is_some()
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
            || self.live_photo.is_some()
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
            MessageKind::Gift => self.gift.is_some(),
            MessageKind::UniqueGift => self.unique_gift.is_some(),
            MessageKind::GiftUpgradeSent => self.gift_upgrade_sent.is_some(),
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
            MessageKind::ChatBackgroundSet => self.chat_background_set.is_some(),
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
            MessageKind::ManagedBotCreated => self.managed_bot_created.is_some(),
            MessageKind::PaidMessagePriceChanged => self.paid_message_price_changed.is_some(),
            MessageKind::PollOptionAdded => self.poll_option_added.is_some(),
            MessageKind::PollOptionDeleted => self.poll_option_deleted.is_some(),
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
            MessageKind::LivePhoto => self.live_photo.is_some(),
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

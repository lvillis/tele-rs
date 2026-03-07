//! Telegram message and message-related request models.

pub(crate) const fn is_false(value: &bool) -> bool {
    !*value
}

mod common;
mod content;
mod forum;
mod media;
mod metadata;
mod model;
mod payments;
mod reply;
mod requests;
mod service;

#[cfg(test)]
mod tests;

pub use common::{Chat, ChatType, MessageEntity, MessageEntityKind, MessageOrigin, PhotoSize};
pub use content::{
    Checklist, ChecklistTask, Contact, Dice, DiceEmoji, Game, Location, Poll, PollKind, PollOption,
    Venue,
};
pub use forum::{
    ForumTopicClosed, ForumTopicCreated, ForumTopicEdited, ForumTopicReopened,
    GeneralForumTopicHidden, GeneralForumTopicUnhidden,
};
pub use media::{
    Animation, Audio, Document, PaidMedia, PaidMediaInfo, Story, Video, VideoNote, VideoQuality,
    Voice,
};
pub use metadata::{
    MessageKind, SuggestedPostApprovalFailed, SuggestedPostApproved, SuggestedPostDeclined,
    SuggestedPostInfo, SuggestedPostPaid, SuggestedPostPrice, SuggestedPostRefundReason,
    SuggestedPostRefunded, SuggestedPostState,
};
pub use model::Message;
pub use payments::{
    Invoice, OrderInfo, RefundedPayment, ShippingAddress, StarAmount, SuccessfulPayment,
};
pub use reply::{ExternalReplyInfo, InaccessibleMessage, MaybeInaccessibleMessage, TextQuote};
pub use requests::{
    ChatAction, CopyMessageRequest, CopyMessagesRequest, DeleteMessageRequest,
    DeleteMessagesRequest, EditMessageCaptionRequest, EditMessageLiveLocationRequest,
    EditMessageReplyMarkupRequest, EditMessageResult, EditMessageTextRequest,
    ForwardMessageRequest, InputMedia, InputMediaAnimation, InputMediaAudio, InputMediaDocument,
    InputMediaPhoto, InputMediaVideo, MessageIdObject, SendAnimationRequest, SendAudioRequest,
    SendChatActionRequest, SendContactRequest, SendDiceRequest, SendDocumentRequest,
    SendLocationRequest, SendMediaGroupRequest, SendMessageRequest, SendPhotoRequest,
    SendPollRequest, SendVenueRequest, SendVideoNoteRequest, SendVideoRequest, SendVoiceRequest,
    SentWebAppMessage, StopMessageLiveLocationRequest, StopPollRequest,
};
pub use service::{
    ChatBoostAdded, ChatOwnerChanged, ChatOwnerLeft, ChatShared, ChecklistTasksAdded,
    ChecklistTasksDone, DirectMessagePriceChanged, Giveaway, GiveawayCompleted, GiveawayCreated,
    GiveawayWinners, MessageAutoDeleteTimerChanged, PaidMessagePriceChanged,
    ProximityAlertTriggered, SharedUser, UsersShared, VideoChatEnded, VideoChatParticipantsInvited,
    VideoChatScheduled, VideoChatStarted, WriteAccessAllowed,
};

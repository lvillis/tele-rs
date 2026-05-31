//! Telegram message and message-related request models.

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

pub use common::{
    Chat, ChatType, MessageEntity, MessageEntityKind, MessageOrigin, MessageOriginChannel,
    MessageOriginChat, MessageOriginHiddenUser, MessageOriginUser, PhotoSize,
};
pub use content::{
    Checklist, ChecklistTask, Contact, Dice, DiceEmoji, Game, GameHighScore, Location, Poll,
    PollKind, PollMedia, PollOption, Venue,
};
pub use forum::{
    ForumTopic, ForumTopicClosed, ForumTopicCreated, ForumTopicEdited, ForumTopicReopened,
    GeneralForumTopicHidden, GeneralForumTopicUnhidden,
};
pub use media::{
    Animation, Audio, Document, LivePhoto, PaidMedia, PaidMediaInfo, PaidMediaLivePhoto,
    PaidMediaPhoto, PaidMediaPreview, PaidMediaVideo, Story, Video, VideoNote, VideoQuality, Voice,
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
    InputMediaGroupItem, InputMediaLivePhoto, InputMediaLocation, InputMediaPhoto,
    InputMediaSticker, InputMediaVenue, InputMediaVideo, InputPollMedia, InputPollOption,
    InputPollOptionMedia, MessageIdObject, SendAnimationRequest, SendAudioRequest,
    SendChatActionRequest, SendContactRequest, SendDiceRequest, SendDocumentRequest,
    SendLocationRequest, SendMediaGroupRequest, SendMessageRequest, SendPhotoRequest,
    SendPollRequest, SendVenueRequest, SendVideoNoteRequest, SendVideoRequest, SendVoiceRequest,
    SentWebAppMessage, StopMessageLiveLocationRequest, StopPollRequest,
};
pub use service::{
    BackgroundFill, BackgroundFillFreeformGradient, BackgroundFillGradient, BackgroundFillSolid,
    BackgroundType, BackgroundTypeChatTheme, BackgroundTypeFill, BackgroundTypePattern,
    BackgroundTypeWallpaper, ChatBackground, ChatBoostAdded, ChatOwnerChanged, ChatOwnerLeft,
    ChatShared, ChecklistTasksAdded, ChecklistTasksDone, DirectMessagePriceChanged,
    DirectMessagesTopic, Giveaway, GiveawayCompleted, GiveawayCreated, GiveawayWinners,
    ManagedBotCreated, MessageAutoDeleteTimerChanged, PaidMessagePriceChanged, PollOptionAdded,
    PollOptionDeleted, ProximityAlertTriggered, SharedUser, UsersShared, VideoChatEnded,
    VideoChatParticipantsInvited, VideoChatScheduled, VideoChatStarted, WriteAccessAllowed,
};

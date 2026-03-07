//! Telegram Bot API request and response models.

pub mod advanced;
pub mod bot;
pub mod chat;
pub mod command;
pub mod common;
pub mod file;
pub mod message;
pub mod payment;
pub mod sticker;
pub mod telegram;
pub mod update;
pub mod upload;
pub mod webhook;

pub use bot::{GetUserProfilePhotosRequest, User, UserProfilePhotos};
pub use chat::{
    BanChatMemberRequest, BanChatSenderChatRequest, ChatAdministratorCapability,
    ChatAdministratorRights, ChatInviteLink, ChatMember, ChatMemberAdministrator, ChatMemberBanned,
    ChatMemberLeft, ChatMemberOwner, ChatMemberRegular, ChatMemberRestricted, ChatMemberStatus,
    ChatPermissions, CreateChatInviteLinkRequest, DeleteChatPhotoRequest,
    DeleteChatStickerSetRequest, EditChatInviteLinkRequest, ExportChatInviteLinkRequest,
    GetChatAdministratorsRequest, GetChatMemberCountRequest, GetChatMemberRequest, GetChatRequest,
    LeaveChatRequest, PinChatMessageRequest, PromoteChatMemberRequest, RestrictChatMemberRequest,
    RevokeChatInviteLinkRequest, SetChatAdministratorCustomTitleRequest, SetChatDescriptionRequest,
    SetChatPermissionsRequest, SetChatStickerSetRequest, SetChatTitleRequest,
    UnbanChatMemberRequest, UnbanChatSenderChatRequest, UnpinAllChatMessagesRequest,
    UnpinChatMessageRequest,
};
pub use command::{
    BotCommand, BotCommandScope, BotDescription, BotName, BotShortDescription,
    DeleteMyCommandsRequest, GetMyCommandsRequest, GetMyDescriptionRequest, GetMyNameRequest,
    GetMyShortDescriptionRequest, SetMyCommandsRequest, SetMyDescriptionRequest, SetMyNameRequest,
    SetMyShortDescriptionRequest,
};
pub use common::{ChatId, MessageId, ParseMode, ResponseParameters, UserId};
pub use file::{File, GetFileRequest};
pub use message::{
    Animation, Audio, Chat, ChatAction, ChatBoostAdded, ChatOwnerChanged, ChatOwnerLeft,
    ChatShared, ChatType, Checklist, ChecklistTask, ChecklistTasksAdded, ChecklistTasksDone,
    Contact, CopyMessageRequest, CopyMessagesRequest, DeleteMessageRequest, DeleteMessagesRequest,
    Dice, DiceEmoji, DirectMessagePriceChanged, Document, EditMessageCaptionRequest,
    EditMessageLiveLocationRequest, EditMessageReplyMarkupRequest, EditMessageResult,
    EditMessageTextRequest, ExternalReplyInfo, ForumTopicClosed, ForumTopicCreated,
    ForumTopicEdited, ForumTopicReopened, ForwardMessageRequest, Game, GeneralForumTopicHidden,
    GeneralForumTopicUnhidden, Giveaway, GiveawayCompleted, GiveawayCreated, GiveawayWinners,
    InaccessibleMessage, InputMedia, InputMediaAnimation, InputMediaAudio, InputMediaDocument,
    InputMediaPhoto, InputMediaVideo, Invoice, Location, MaybeInaccessibleMessage, Message,
    MessageAutoDeleteTimerChanged, MessageEntity, MessageEntityKind, MessageIdObject, MessageKind,
    MessageOrigin, OrderInfo, PaidMedia, PaidMediaInfo, PaidMessagePriceChanged, PhotoSize, Poll,
    PollKind, PollOption, ProximityAlertTriggered, RefundedPayment, SendAnimationRequest,
    SendAudioRequest, SendChatActionRequest, SendContactRequest, SendDiceRequest,
    SendDocumentRequest, SendLocationRequest, SendMediaGroupRequest, SendMessageRequest,
    SendPhotoRequest, SendPollRequest, SendVenueRequest, SendVideoNoteRequest, SendVideoRequest,
    SendVoiceRequest, SentWebAppMessage, SharedUser, ShippingAddress, StarAmount,
    StopMessageLiveLocationRequest, StopPollRequest, Story, SuccessfulPayment,
    SuggestedPostApprovalFailed, SuggestedPostApproved, SuggestedPostDeclined, SuggestedPostInfo,
    SuggestedPostPaid, SuggestedPostPrice, SuggestedPostRefundReason, SuggestedPostRefunded,
    SuggestedPostState, TextQuote, UsersShared, Venue, Video, VideoChatEnded,
    VideoChatParticipantsInvited, VideoChatScheduled, VideoChatStarted, VideoNote, VideoQuality,
    Voice, WriteAccessAllowed,
};
pub use payment::{
    AnswerPreCheckoutQueryRequest, AnswerShippingQueryRequest, CreateInvoiceLinkRequest,
    LabeledPrice, SendInvoiceRequest, ShippingOption,
};
pub use sticker::{
    AddStickerToSetRequest, CreateNewStickerSetRequest, DeleteStickerFromSetRequest,
    DeleteStickerSetRequest, GetCustomEmojiStickersRequest, GetStickerSetRequest, InputSticker,
    MaskPosition, ReplaceStickerInSetRequest, SendStickerRequest,
    SetCustomEmojiStickerSetThumbnailRequest, SetStickerEmojiListRequest,
    SetStickerKeywordsRequest, SetStickerMaskPositionRequest, SetStickerPositionInSetRequest,
    SetStickerSetThumbnailRequest, SetStickerSetTitleRequest, Sticker, StickerFormat, StickerSet,
    StickerType, UploadStickerFileRequest,
};
pub use telegram::{
    AcceptedGiftTypes, CallbackCodec, CallbackPayload, CallbackPayloadCodec, CompactCallbackCodec,
    CompactCallbackDecoder, CompactCallbackEncoder, CompactCallbackPayload, ForceReply,
    InlineKeyboardButton, InlineKeyboardMarkup, InlineQueryResult, InlineQueryResultArticle,
    InlineQueryResultsButton, InputChecklist, InputPaidMedia, InputStoryContent,
    InputTextMessageContent, JsonCallbackCodec, KeyboardButton, LinkPreviewOptions, MenuButton,
    MenuButtonKind, MenuButtonWebApp, PassportElementError, ReactionType, ReplyKeyboardMarkup,
    ReplyKeyboardRemove, ReplyMarkup, ReplyParameters, StoryArea, SuggestedPostParameters,
    WebAppData, WebAppInfo,
};
pub use update::{
    AnswerCallbackQueryRequest, AnswerInlineQueryRequest, CallbackQuery, ChatJoinRequest,
    ChatMemberUpdated, ChosenInlineResult, GetUpdatesRequest, InlineQuery, PollAnswer, Update,
    UpdateKind,
};
pub use upload::UploadFile;
pub use webhook::{DeleteWebhookRequest, SetWebhookRequest, WebhookInfo};

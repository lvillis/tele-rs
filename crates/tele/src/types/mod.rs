//! Telegram Bot API request and response models.

pub mod advanced;
pub mod bot;
pub mod chat;
pub mod command;
pub mod common;
pub mod file;
pub mod gift;
pub mod message;
pub mod payment;
pub mod sticker;
pub(crate) mod tagged;
pub mod telegram;
pub mod update;
pub mod upload;
pub(crate) mod validation;
pub mod webhook;

pub use bot::{GetUserProfilePhotosRequest, User, UserProfileAudios, UserProfilePhotos};
pub use chat::{
    BanChatMemberRequest, BanChatSenderChatRequest, ChatAdministratorCapability,
    ChatAdministratorRights, ChatInviteLink, ChatMember, ChatMemberAdministrator, ChatMemberBanned,
    ChatMemberLeft, ChatMemberOwner, ChatMemberRegular, ChatMemberRestricted, ChatMemberStatus,
    ChatPermissions, CreateChatInviteLinkRequest, DeleteChatPhotoRequest,
    DeleteChatStickerSetRequest, EditChatInviteLinkRequest, ExportChatInviteLinkRequest,
    GetChatAdministratorsRequest, GetChatMemberCountRequest, GetChatMemberRequest, GetChatRequest,
    LeaveChatRequest, PinChatMessageRequest, PromoteChatMemberRequest, RestrictChatMemberRequest,
    RevokeChatInviteLinkRequest, SetChatAdministratorCustomTitleRequest, SetChatDescriptionRequest,
    SetChatPermissionsRequest, SetChatPhotoRequest, SetChatStickerSetRequest, SetChatTitleRequest,
    UnbanChatMemberRequest, UnbanChatSenderChatRequest, UnpinAllChatMessagesRequest,
    UnpinChatMessageRequest,
};
pub use command::{
    BotCommand, BotCommandScope, BotDescription, BotName, BotShortDescription,
    DeleteMyCommandsRequest, GetMyCommandsRequest, GetMyDescriptionRequest, GetMyNameRequest,
    GetMyShortDescriptionRequest, SetMyCommandsRequest, SetMyDescriptionRequest, SetMyNameRequest,
    SetMyShortDescriptionRequest,
};
pub use common::{ChatId, MessageId, NumericChatId, ParseMode, ResponseParameters, UserId};
pub use file::{File, GetFileRequest};
pub use gift::{
    AffiliateInfo, Gift, GiftBackground, GiftInfo, Gifts, OwnedGift, OwnedGiftRegular,
    OwnedGiftUnique, OwnedGifts, RevenueWithdrawalState, RevenueWithdrawalStateFailed,
    RevenueWithdrawalStatePending, RevenueWithdrawalStateSucceeded, StarTransaction,
    StarTransactions, TransactionKind, TransactionPartner, TransactionPartnerAffiliateProgram,
    TransactionPartnerChat, TransactionPartnerFragment, TransactionPartnerOther,
    TransactionPartnerTelegramAds, TransactionPartnerTelegramApi, TransactionPartnerUser,
    UniqueGift, UniqueGiftBackdrop, UniqueGiftBackdropColors, UniqueGiftColors, UniqueGiftInfo,
    UniqueGiftModel, UniqueGiftOrigin, UniqueGiftSymbol,
};
pub use message::{
    Animation, Audio, Chat, ChatAction, ChatBoostAdded, ChatOwnerChanged, ChatOwnerLeft,
    ChatShared, ChatType, Checklist, ChecklistTask, ChecklistTasksAdded, ChecklistTasksDone,
    Contact, CopyMessageRequest, CopyMessagesRequest, DeleteMessageRequest, DeleteMessagesRequest,
    Dice, DiceEmoji, DirectMessagePriceChanged, Document, EditMessageCaptionRequest,
    EditMessageLiveLocationRequest, EditMessageReplyMarkupRequest, EditMessageResult,
    EditMessageTextRequest, ExternalReplyInfo, ForumTopic, ForumTopicClosed, ForumTopicCreated,
    ForumTopicEdited, ForumTopicReopened, ForwardMessageRequest, Game, GameHighScore,
    GeneralForumTopicHidden, GeneralForumTopicUnhidden, Giveaway, GiveawayCompleted,
    GiveawayCreated, GiveawayWinners, InaccessibleMessage, InputMedia, InputMediaAnimation,
    InputMediaAudio, InputMediaDocument, InputMediaGroupItem, InputMediaPhoto, InputMediaVideo,
    InputPollOption, Invoice, Location, MaybeInaccessibleMessage, Message,
    MessageAutoDeleteTimerChanged, MessageEntity, MessageEntityKind, MessageIdObject, MessageKind,
    MessageOrigin, MessageOriginChannel, MessageOriginChat, MessageOriginHiddenUser,
    MessageOriginUser, OrderInfo, PaidMedia, PaidMediaInfo, PaidMessagePriceChanged, PhotoSize,
    Poll, PollKind, PollOption, ProximityAlertTriggered, RefundedPayment, SendAnimationRequest,
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
    MaskPosition, MaskPositionPoint, ReplaceStickerInSetRequest, SendStickerRequest,
    SetCustomEmojiStickerSetThumbnailRequest, SetStickerEmojiListRequest,
    SetStickerKeywordsRequest, SetStickerMaskPositionRequest, SetStickerPositionInSetRequest,
    SetStickerSetThumbnailRequest, SetStickerSetTitleRequest, Sticker, StickerFormat, StickerKind,
    StickerSet, StickerType, UploadStickerFileRequest,
};
pub use telegram::{
    AcceptedGiftTypes, CallbackCodec, CallbackPayload, CallbackPayloadCodec, CompactCallbackCodec,
    CompactCallbackDecoder, CompactCallbackEncoder, CompactCallbackPayload, ForceReply,
    InlineKeyboardButton, InlineKeyboardMarkup, InlineQueryResult, InlineQueryResultArticle,
    InlineQueryResultArticleKind, InlineQueryResultsButton, InputChecklist, InputPaidMedia,
    InputProfilePhoto, InputStoryContent, InputTextMessageContent, JsonCallbackCodec,
    KeyboardButton, LinkPreviewOptions, MenuButton, MenuButtonKind, MenuButtonWebApp,
    PassportElementError, PreparedInlineMessage, PreparedKeyboardButton, ReactionType,
    ReplyKeyboardMarkup, ReplyKeyboardRemove, ReplyMarkup, ReplyParameters, StoryArea,
    SuggestedPostParameters, WebAppData, WebAppInfo,
};
pub use update::{
    AllowedUpdate, AnswerCallbackQueryRequest, AnswerInlineQueryRequest, BusinessBotRights,
    BusinessConnection, BusinessMessagesDeleted, CallbackQuery, ChatBoost, ChatBoostRemoved,
    ChatBoostSource, ChatBoostUpdated, ChatJoinRequest, ChatMemberUpdated, ChosenInlineResult,
    GetUpdatesRequest, InlineQuery, ManagedBotUpdated, MessageReactionCountUpdated,
    MessageReactionUpdated, PaidMediaPurchased, PollAnswer, PreCheckoutQuery, ReactionCount,
    ShippingQuery, Update, UpdateKind, UserChatBoosts,
};
pub use upload::{UploadFile, UploadPart};
pub use webhook::{DeleteWebhookRequest, SetWebhookRequest, WebhookInfo, WebhookSecretToken};

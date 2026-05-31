//! Telegram Bot API request and response models.

pub mod advanced;
pub mod bot;
pub mod chat;
pub mod command;
pub mod common;
pub(crate) mod extra;
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
    Animation, Audio, BackgroundFill, BackgroundFillFreeformGradient, BackgroundFillGradient,
    BackgroundFillSolid, BackgroundType, BackgroundTypeChatTheme, BackgroundTypeFill,
    BackgroundTypePattern, BackgroundTypeWallpaper, Chat, ChatAction, ChatBackground,
    ChatBoostAdded, ChatOwnerChanged, ChatOwnerLeft, ChatShared, ChatType, Checklist,
    ChecklistTask, ChecklistTasksAdded, ChecklistTasksDone, Contact, CopyMessageRequest,
    CopyMessagesRequest, DeleteMessageRequest, DeleteMessagesRequest, Dice, DiceEmoji,
    DirectMessagePriceChanged, DirectMessagesTopic, Document, EditMessageCaptionRequest,
    EditMessageLiveLocationRequest, EditMessageReplyMarkupRequest, EditMessageResult,
    EditMessageTextRequest, ExternalReplyInfo, ForumTopic, ForumTopicClosed, ForumTopicCreated,
    ForumTopicEdited, ForumTopicReopened, ForwardMessageRequest, Game, GameHighScore,
    GeneralForumTopicHidden, GeneralForumTopicUnhidden, Giveaway, GiveawayCompleted,
    GiveawayCreated, GiveawayWinners, InaccessibleMessage, InputMedia, InputMediaAnimation,
    InputMediaAudio, InputMediaDocument, InputMediaGroupItem, InputMediaLivePhoto,
    InputMediaLocation, InputMediaPhoto, InputMediaSticker, InputMediaVenue, InputMediaVideo,
    InputPollMedia, InputPollOption, InputPollOptionMedia, Invoice, LivePhoto, Location,
    ManagedBotCreated, MaybeInaccessibleMessage, Message, MessageAutoDeleteTimerChanged,
    MessageEntity, MessageEntityKind, MessageIdObject, MessageKind, MessageOrigin,
    MessageOriginChannel, MessageOriginChat, MessageOriginHiddenUser, MessageOriginUser, OrderInfo,
    PaidMedia, PaidMediaInfo, PaidMediaLivePhoto, PaidMediaPhoto, PaidMediaPreview, PaidMediaVideo,
    PaidMessagePriceChanged, PhotoSize, Poll, PollKind, PollMedia, PollOption, PollOptionAdded,
    PollOptionDeleted, ProximityAlertTriggered, RefundedPayment, SendAnimationRequest,
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
    AcceptedGiftTypes, ButtonStyle, CallbackCodec, CallbackPayload, CallbackPayloadCodec,
    CompactCallbackCodec, CompactCallbackDecoder, CompactCallbackEncoder, CompactCallbackPayload,
    ForceReply, InlineKeyboardButton, InlineKeyboardMarkup, InlineQueryResult,
    InlineQueryResultArticle, InlineQueryResultArticleKind, InlineQueryResultsButton,
    InputChecklist, InputChecklistTask, InputPaidMedia, InputPaidMediaLivePhoto,
    InputPaidMediaPhoto, InputPaidMediaVideo, InputProfilePhoto, InputProfilePhotoAnimated,
    InputProfilePhotoStatic, InputStoryContent, InputStoryContentPhoto, InputStoryContentVideo,
    InputTextMessageContent, JsonCallbackCodec, KeyboardButton, KeyboardButtonPollType,
    KeyboardButtonRequestChat, KeyboardButtonRequestManagedBot, KeyboardButtonRequestUsers,
    LinkPreviewOptions, LocationAddress, MenuButton, MenuButtonCommands, MenuButtonDefault,
    MenuButtonKind, MenuButtonWebApp, PassportElementError, PreparedInlineMessage,
    PreparedKeyboardButton, ReactionType, ReactionTypeCustomEmoji, ReactionTypeEmoji,
    ReactionTypePaid, ReplyKeyboardMarkup, ReplyKeyboardRemove, ReplyMarkup, ReplyParameters,
    StoryArea, StoryAreaPosition, StoryAreaType, StoryAreaTypeLink, StoryAreaTypeLocation,
    StoryAreaTypeSuggestedReaction, StoryAreaTypeUniqueGift, StoryAreaTypeWeather,
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

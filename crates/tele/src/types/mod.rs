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
    BanChatMemberRequest, BanChatSenderChatRequest, ChatAdministratorRights, ChatInviteLink,
    ChatMember, ChatPermissions, CreateChatInviteLinkRequest, DeleteChatPhotoRequest,
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
    Chat, ChatAction, ChatType, CopyMessageRequest, CopyMessagesRequest, DeleteMessageRequest,
    DeleteMessagesRequest, DiceEmoji, EditMessageCaptionRequest, EditMessageLiveLocationRequest,
    EditMessageReplyMarkupRequest, EditMessageResult, EditMessageTextRequest,
    ForwardMessageRequest, InputMedia, InputMediaAnimation, InputMediaAudio, InputMediaDocument,
    InputMediaPhoto, InputMediaVideo, Message, MessageEntity, MessageIdObject, PhotoSize, Poll,
    PollOption, SendAnimationRequest, SendAudioRequest, SendChatActionRequest, SendContactRequest,
    SendDiceRequest, SendDocumentRequest, SendLocationRequest, SendMediaGroupRequest,
    SendMessageRequest, SendPhotoRequest, SendPollRequest, SendVenueRequest, SendVideoNoteRequest,
    SendVideoRequest, SendVoiceRequest, SentWebAppMessage, StopMessageLiveLocationRequest,
    StopPollRequest, WriteAccessAllowed,
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
    AcceptedGiftTypes, ForceReply, InlineKeyboardButton, InlineKeyboardMarkup, InlineQueryResult,
    InlineQueryResultArticle, InlineQueryResultsButton, InputChecklist, InputPaidMedia,
    InputStoryContent, InputTextMessageContent, KeyboardButton, LinkPreviewOptions, MenuButton,
    MenuButtonKind, MenuButtonWebApp, PassportElementError, ReactionType, ReplyKeyboardMarkup,
    ReplyKeyboardRemove, ReplyMarkup, ReplyParameters, StoryArea, SuggestedPostParameters,
    WebAppData, WebAppInfo,
};
pub use update::{
    AnswerCallbackQueryRequest, AnswerInlineQueryRequest, CallbackQuery, ChosenInlineResult,
    GetUpdatesRequest, InlineQuery, PollAnswer, Update,
};
pub use upload::UploadFile;
pub use webhook::{DeleteWebhookRequest, SetWebhookRequest, WebhookInfo};

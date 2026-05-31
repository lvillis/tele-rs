//! Common imports for building Telegram bots with `tele`.

#[cfg(feature = "_blocking")]
pub use crate::BlockingClient;
#[cfg(feature = "_async")]
pub use crate::Client;
pub use crate::{ClientBuilder, Error, Result};
pub use crate::{VerifiedWebAppInitData, parse_web_app_init_data, verify_web_app_init_data};

#[cfg(feature = "_async")]
pub use crate::client::{
    AnimationSendBuilder, AnimationUploadBuilder, AppApi, AudioSendBuilder, AudioUploadBuilder,
    CallbackAnswerBuilder, ChatActionBuilder, ContactSendBuilder, ControlApi, DiceSendBuilder,
    DocumentSendBuilder, DocumentUploadBuilder, LocationSendBuilder, MediaGroupSendBuilder,
    MediaGroupUploadBuilder, MembershipApi, ModerationApi, ModerationNoticeApi, PhotoSendBuilder,
    PhotoUploadBuilder, PollSendBuilder, RawApi, SetupApi, StickerSendBuilder,
    StickerUploadBuilder, StopPollBuilder, TextSendBuilder, TypedApi, VenueSendBuilder,
    VideoNoteSendBuilder, VideoNoteUploadBuilder, VideoSendBuilder, VideoUploadBuilder,
    VoiceSendBuilder, VoiceUploadBuilder, WebAppApi,
};
pub use crate::client::{
    BanMemberOptions, BootstrapFetchStepReport, BootstrapGetMePolicy, BootstrapOutcome,
    BootstrapPlan, BootstrapReport, BootstrapRetryPolicy, BootstrapStepDiagnostics,
    BootstrapStepPhase, BootstrapStepStatus, BootstrapSyncStepReport, ClientMetric,
    ClientMetricHook, MenuButtonConfig, RestrictMemberOptions, WebAppQueryPayload,
};
#[cfg(feature = "_blocking")]
pub use crate::client::{
    BlockingAnimationSendBuilder, BlockingAnimationUploadBuilder, BlockingAppApi,
    BlockingAudioSendBuilder, BlockingAudioUploadBuilder, BlockingCallbackAnswerBuilder,
    BlockingChatActionBuilder, BlockingContactSendBuilder, BlockingControlApi,
    BlockingDiceSendBuilder, BlockingDocumentSendBuilder, BlockingDocumentUploadBuilder,
    BlockingLocationSendBuilder, BlockingMediaGroupSendBuilder, BlockingMediaGroupUploadBuilder,
    BlockingMembershipApi, BlockingModerationApi, BlockingModerationNoticeApi,
    BlockingPhotoSendBuilder, BlockingPhotoUploadBuilder, BlockingPollSendBuilder, BlockingRawApi,
    BlockingStickerSendBuilder, BlockingStickerUploadBuilder, BlockingStopPollBuilder,
    BlockingTextSendBuilder, BlockingTypedApi, BlockingVenueSendBuilder,
    BlockingVideoNoteSendBuilder, BlockingVideoNoteUploadBuilder, BlockingVideoSendBuilder,
    BlockingVideoUploadBuilder, BlockingVoiceSendBuilder, BlockingVoiceUploadBuilder,
};
#[cfg(feature = "_blocking")]
pub use crate::client::{BlockingSetupApi, BlockingWebAppApi};

pub use crate::types::{
    AllowedUpdate, BackgroundFill, BackgroundType, BotCommand, BusinessConnection,
    BusinessMessagesDeleted, CallbackCodec, CallbackPayload, ChatAction,
    ChatAdministratorCapability, ChatBackground, ChatBoostRemoved, ChatBoostUpdated, ChatId,
    ChatJoinRequest, ChatMemberStatus, ChatMemberUpdated, ChatPermissions, CompactCallbackPayload,
    DirectMessagesTopic, InputMedia, InputMediaAnimation, InputMediaAudio, InputMediaDocument,
    InputMediaGroupItem, InputMediaLivePhoto, InputMediaLocation, InputMediaPhoto,
    InputMediaSticker, InputMediaVenue, InputMediaVideo, InputPollMedia, InputPollOption,
    InputPollOptionMedia, LivePhoto, ManagedBotCreated, ManagedBotUpdated, Message,
    MessageEntityKind, MessageId, MessageKind, MessageOrigin, MessageReactionCountUpdated,
    MessageReactionUpdated, PaidMediaPurchased, ParseMode, Poll, PollKind, PollMedia,
    PollOptionAdded, PollOptionDeleted, PreCheckoutQuery, ReplyMarkup, ReplyParameters,
    ShippingQuery, Update, UpdateKind, UploadFile, UploadPart, WebhookSecretToken,
};

#[cfg(all(feature = "bot", feature = "postgres-session"))]
pub use crate::bot::PostgresSessionStore;
#[cfg(all(feature = "bot", feature = "redis-session"))]
pub use crate::bot::RedisSessionStore;
#[cfg(feature = "bot")]
pub use crate::bot::{
    BotApp, BotContext, BotEngine, BotOutbox, BusinessConnectionInput,
    BusinessMessagesDeletedInput, CURRENT_ACTOR_CHAT_MEMBER, CURRENT_BOT_CHAT_MEMBER,
    CallbackInput, CallbackQueryInput, CallbackRouteBuilder, ChannelUpdateSource, ChatBoostInput,
    ChatBoostRemovedInput, ChatJoinRequestInput, ChatMemberUpdatedInput, ChatSession,
    ChosenInlineResultInput, CommandArgs, CommandRouteBuilder, CompactCallbackInput,
    CompactCallbackRouteBuilder, ContextAppApi, DispatchMetricOutcome, DispatchOutcome,
    EngineConfig, EngineEvent, EngineMetric, ErrorPolicy, HandlerError, InMemorySessionStore,
    InlineQueryInput, JsonCallback, JsonFileSessionStore, LongPollingSource, ManagedBotInput,
    MessageReactionCountInput, MessageReactionInput, MyChatMemberUpdatedInput, OutboxConfig,
    PaidMediaPurchasedInput, ParsedCommandRouteBuilder, PollAnswerInput, PollInput, PollingConfig,
    PreCheckoutQueryInput, RequestStateKey, RouteRejection, Router, ShippingQueryInput,
    SourceErrorBackoffConfig, TELEGRAM_SECRET_HEADER, TextInput, ThrottleScope, TypedCallbackInput,
    TypedCallbackRouteBuilder, TypedCommandInput, UpdateExt, UpdateExtractor, UpdateSink,
    WebAppInput, WebhookRunner, WriteAccessAllowedInput, channel_source, dispatch_webhook,
    dispatch_webhook_status, telegram_secret_token,
};

#[cfg(feature = "macros")]
pub use crate::BotCommands;

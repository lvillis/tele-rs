use std::time::Duration;

use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::client::RetryConfig;
use crate::types::advanced::{AdvancedAnswerWebAppQueryRequest, AdvancedRequest};
use crate::types::bot::User;
use crate::types::chat::{
    ChatAdministratorCapability, ChatMember, GetChatAdministratorsRequest, GetChatMemberRequest,
};
use crate::types::command::{
    BotCommand, BotCommandScope, GetMyCommandsRequest, SetMyCommandsRequest,
};
use crate::types::common::{ChatId, MessageId, NumericChatId, ParseMode, UserId};
use crate::types::message::{
    ChatAction, DiceEmoji, InputMediaGroupItem, InputPollMedia, InputPollOption, Message,
    MessageEntity, Poll, PollKind, SendAnimationRequest, SendAudioRequest, SendChatActionRequest,
    SendContactRequest, SendDiceRequest, SendDocumentRequest, SendLocationRequest,
    SendMediaGroupRequest, SendMessageRequest, SendPhotoRequest, SendPollRequest, SendVenueRequest,
    SendVideoNoteRequest, SendVideoRequest, SendVoiceRequest, SentWebAppMessage, StopPollRequest,
};
use crate::types::sticker::SendStickerRequest;
use crate::types::telegram::{
    InlineKeyboardMarkup, InlineQueryResult, LinkPreviewOptions, MenuButton, ReplyMarkup,
    ReplyParameters, SuggestedPostParameters, WebAppData,
};
use crate::types::update::{AnswerCallbackQueryRequest, Update};
use crate::types::upload::{UploadFile, UploadPart};
use crate::{Error, Result};

#[cfg(feature = "_blocking")]
use crate::BlockingClient;
#[cfg(feature = "_async")]
use crate::Client;

mod app;
mod bootstrap;
mod control;
mod membership;
mod menu;
mod moderation;
mod raw;
mod setup;
mod support;
mod typed;
mod web_app;

#[cfg(feature = "_async")]
pub use app::{
    AnimationSendBuilder, AnimationUploadBuilder, AppApi, AudioSendBuilder, AudioUploadBuilder,
    CallbackAnswerBuilder, ChatActionBuilder, ContactSendBuilder, DiceSendBuilder,
    DocumentSendBuilder, DocumentUploadBuilder, LocationSendBuilder, MediaGroupSendBuilder,
    MediaGroupUploadBuilder, PhotoSendBuilder, PhotoUploadBuilder, PollSendBuilder,
    StickerSendBuilder, StickerUploadBuilder, StopPollBuilder, TextSendBuilder, VenueSendBuilder,
    VideoNoteSendBuilder, VideoNoteUploadBuilder, VideoSendBuilder, VideoUploadBuilder,
    VoiceSendBuilder, VoiceUploadBuilder,
};
#[cfg(feature = "_blocking")]
pub use app::{
    BlockingAnimationSendBuilder, BlockingAnimationUploadBuilder, BlockingAppApi,
    BlockingAudioSendBuilder, BlockingAudioUploadBuilder, BlockingCallbackAnswerBuilder,
    BlockingChatActionBuilder, BlockingContactSendBuilder, BlockingDiceSendBuilder,
    BlockingDocumentSendBuilder, BlockingDocumentUploadBuilder, BlockingLocationSendBuilder,
    BlockingMediaGroupSendBuilder, BlockingMediaGroupUploadBuilder, BlockingPhotoSendBuilder,
    BlockingPhotoUploadBuilder, BlockingPollSendBuilder, BlockingStickerSendBuilder,
    BlockingStickerUploadBuilder, BlockingStopPollBuilder, BlockingTextSendBuilder,
    BlockingVenueSendBuilder, BlockingVideoNoteSendBuilder, BlockingVideoNoteUploadBuilder,
    BlockingVideoSendBuilder, BlockingVideoUploadBuilder, BlockingVoiceSendBuilder,
    BlockingVoiceUploadBuilder,
};
pub use bootstrap::{
    BootstrapFetchStepReport, BootstrapGetMePolicy, BootstrapOutcome, BootstrapPlan,
    BootstrapReport, BootstrapRetryPolicy, BootstrapStepDiagnostics, BootstrapStepPhase,
    BootstrapStepStatus, BootstrapSyncStepReport, WebAppQueryPayload,
};
#[cfg(feature = "_blocking")]
pub use control::BlockingControlApi;
#[cfg(feature = "_async")]
pub use control::ControlApi;
#[cfg(feature = "_blocking")]
pub use membership::BlockingMembershipApi;
#[cfg(feature = "_async")]
pub use membership::MembershipApi;
pub use menu::MenuButtonConfig;
#[cfg(feature = "_blocking")]
pub use moderation::BlockingModerationApi;
#[cfg(feature = "_blocking")]
pub use moderation::BlockingModerationNoticeApi;
#[cfg(feature = "_async")]
pub use moderation::ModerationApi;
#[cfg(feature = "_async")]
pub use moderation::ModerationNoticeApi;
pub use moderation::{BanMemberOptions, RestrictMemberOptions};
#[cfg(feature = "_blocking")]
pub use raw::BlockingRawApi;
#[cfg(feature = "_async")]
pub use raw::RawApi;
#[cfg(feature = "_blocking")]
pub use setup::BlockingSetupApi;
#[cfg(feature = "_async")]
pub use setup::SetupApi;
#[cfg(feature = "bot")]
pub(crate) use support::is_unaddressable_reply_error;
#[cfg(feature = "_blocking")]
pub use typed::BlockingTypedApi;
#[cfg(feature = "_async")]
pub use typed::TypedApi;
#[cfg(feature = "_blocking")]
pub use web_app::BlockingWebAppApi;
#[cfg(feature = "_async")]
pub use web_app::WebAppApi;

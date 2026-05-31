mod config;
mod layers;
mod observability;
mod retry;

#[cfg(feature = "_async")]
mod async_client;
#[cfg(feature = "_blocking")]
mod blocking_client;

#[cfg(feature = "_async")]
pub use async_client::Client;
#[cfg(feature = "_blocking")]
pub use blocking_client::BlockingClient;
pub(crate) use config::RequestDefaults;
pub use config::{ClientBuilder, RateLimitConfig, RetryConfig};
#[cfg(feature = "bot")]
pub(crate) use layers::is_unaddressable_reply_error;
#[cfg(feature = "_async")]
pub use layers::{
    AnimationSendBuilder, AnimationUploadBuilder, AppApi, AudioSendBuilder, AudioUploadBuilder,
    CallbackAnswerBuilder, ChatActionBuilder, ContactSendBuilder, ControlApi, DiceSendBuilder,
    DocumentSendBuilder, DocumentUploadBuilder, LocationSendBuilder, MediaGroupSendBuilder,
    MediaGroupUploadBuilder, MembershipApi, ModerationApi, ModerationNoticeApi, PhotoSendBuilder,
    PhotoUploadBuilder, PollSendBuilder, RawApi, SetupApi, StickerSendBuilder,
    StickerUploadBuilder, StopPollBuilder, TextSendBuilder, TypedApi, VenueSendBuilder,
    VideoNoteSendBuilder, VideoNoteUploadBuilder, VideoSendBuilder, VideoUploadBuilder,
    VoiceSendBuilder, VoiceUploadBuilder, WebAppApi,
};
pub use layers::{
    BanMemberOptions, BootstrapFetchStepReport, BootstrapGetMePolicy, BootstrapOutcome,
    BootstrapPlan, BootstrapReport, BootstrapRetryPolicy, BootstrapStepDiagnostics,
    BootstrapStepPhase, BootstrapStepStatus, BootstrapSyncStepReport, MenuButtonConfig,
    RestrictMemberOptions, WebAppQueryPayload,
};
#[cfg(feature = "_blocking")]
pub use layers::{
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
pub use layers::{BlockingSetupApi, BlockingWebAppApi};
pub use observability::{ClientMetric, ClientMetricHook};
pub(crate) use observability::{ClientObservability, emit_client_result_metric};

#![forbid(unsafe_code)]

//! `tele` is an ergonomic Telegram Bot API SDK and bot runtime toolkit for Rust.
//! Built on `reqx`, with strict error modeling and async/blocking parity.
//!
//! Recommended stable surface:
//!
//! - `client.app()` / `context.app()` for runtime business code such as message sends, replies,
//!   callback answers, Web App flows, moderation, and membership/capability checks.
//! - `client.control()` for startup/setup/orchestration such as bootstrap, router preparation,
//!   and outbox management.
//! - `client.raw()` / `client.typed()` / `client.advanced()` as lower-level escape hatches when
//!   the high-level facades are intentionally not enough.
//!
//! Minimal async example:
//!
//! ```rust,no_run
//! use tele::Client;
//! use tele::types::ParseMode;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), tele::Error> {
//!     let client = Client::builder("https://api.telegram.org")?
//!         .bot_token("123456:telegram-bot-token")?
//!         .build()?;
//!
//!     let _sent = client
//!         .app()
//!         .text(123456789_i64, "hello from tele")?
//!         .parse_mode(ParseMode::MarkdownV2)
//!         .send()
//!         .await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! With `feature = "bot"`, prefer `context.app()` inside handlers and `client.control()` for
//! startup/bootstrap/outbox orchestration. For richer runtime flows, prefer
//! `client.app().callback_answer(...)`,
//! `client.app().photo()/document()/video()/audio()/animation()/voice()/sticker()/media_group()`,
//! `client.app().location()/venue()/contact()/poll()/dice()`,
//! `client.app().chat_action()/stop_poll()`, and `client.app().membership()` before dropping to
//! raw request structs. The package README and `examples/` directory contain the minimal bot and
//! API layer walkthroughs.

#[cfg(not(any(feature = "_async", feature = "_blocking")))]
compile_error!(
    "tele requires at least one transport feature: enable an `async-tls-*` or `blocking-tls-*` feature."
);

mod transport;
mod util;

pub mod api;
pub mod auth;
#[cfg(feature = "bot")]
pub mod bot;
pub mod client;
pub mod error;
pub mod prelude;
pub mod testing;
pub mod types;

pub use auth::{
    Auth, BotToken, VerifiedWebAppInitData, parse_web_app_init_data, verify_web_app_init_data,
};
#[cfg(feature = "_blocking")]
pub use client::BlockingClient;
#[cfg(feature = "_async")]
pub use client::Client;
#[cfg(feature = "_async")]
pub use client::{
    AnimationSendBuilder, AnimationUploadBuilder, AppApi, AudioSendBuilder, AudioUploadBuilder,
    CallbackAnswerBuilder, ChatActionBuilder, ContactSendBuilder, ControlApi, DiceSendBuilder,
    DocumentSendBuilder, DocumentUploadBuilder, LocationSendBuilder, MediaGroupSendBuilder,
    MediaGroupUploadBuilder, MembershipApi, ModerationApi, ModerationNoticeApi, PhotoSendBuilder,
    PhotoUploadBuilder, PollSendBuilder, RawApi, SetupApi, StickerSendBuilder,
    StickerUploadBuilder, StopPollBuilder, TextSendBuilder, TypedApi, VenueSendBuilder,
    VideoNoteSendBuilder, VideoNoteUploadBuilder, VideoSendBuilder, VideoUploadBuilder,
    VoiceSendBuilder, VoiceUploadBuilder, WebAppApi,
};
pub use client::{
    BanMemberOptions, BootstrapFetchStepReport, BootstrapGetMePolicy, BootstrapOutcome,
    BootstrapPlan, BootstrapReport, BootstrapRetryPolicy, BootstrapStepDiagnostics,
    BootstrapStepPhase, BootstrapStepStatus, BootstrapSyncStepReport, ClientMetric,
    ClientMetricHook, MenuButtonConfig, RestrictMemberOptions, WebAppQueryPayload,
};
#[cfg(feature = "_blocking")]
pub use client::{
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
pub use client::{BlockingSetupApi, BlockingWebAppApi};
pub use client::{ClientBuilder, RateLimitConfig, RetryConfig};
pub use error::{Error, ErrorClass, Result};
#[cfg(feature = "macros")]
pub use tele_macros::BotCommands;
pub use types::{UploadFile, UploadPart};

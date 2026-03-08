#![forbid(unsafe_code)]

//! `tele` is an ergonomic Telegram Bot API SDK for Rust.
//! Built on `reqx`, with strict error modeling and async/blocking parity.

#[cfg(not(any(feature = "_async", feature = "_blocking")))]
compile_error!(
    "tele requires at least one transport feature: enable an `async-tls-*` or `blocking-tls-*` feature."
);

#[cfg(all(
    feature = "_async",
    not(feature = "async-tls-rustls-ring"),
    not(feature = "async-tls-rustls-aws-lc-rs"),
    not(feature = "async-tls-native")
))]
compile_error!(
    "tele async transport requires one async TLS backend: enable `async-tls-rustls-ring`, `async-tls-rustls-aws-lc-rs`, or `async-tls-native`."
);

#[cfg(all(
    feature = "_blocking",
    not(feature = "blocking-tls-rustls-ring"),
    not(feature = "blocking-tls-rustls-aws-lc-rs"),
    not(feature = "blocking-tls-native")
))]
compile_error!(
    "tele blocking transport requires one blocking TLS backend: enable `blocking-tls-rustls-ring`, `blocking-tls-rustls-aws-lc-rs`, or `blocking-tls-native`."
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
pub use client::{AppApi, RawApi, SetupApi, TypedApi, WebAppApi};
#[cfg(feature = "_blocking")]
pub use client::{BlockingAppApi, BlockingRawApi, BlockingTypedApi};
#[cfg(feature = "_blocking")]
pub use client::{BlockingSetupApi, BlockingWebAppApi};
pub use client::{
    BootstrapFetchStepReport, BootstrapGetMePolicy, BootstrapOutcome, BootstrapPlan,
    BootstrapReport, BootstrapRetryPolicy, BootstrapStepDiagnostics, BootstrapStepPhase,
    BootstrapStepStatus, BootstrapSyncStepReport, ClientMetric, ClientMetricHook, MenuButtonConfig,
    WebAppQueryPayload,
};
pub use client::{ClientBuilder, RateLimitConfig, RetryConfig};
pub use error::{Error, ErrorClass, Result};
#[cfg(feature = "macros")]
pub use tele_macros::BotCommands;
pub use types::UploadFile;

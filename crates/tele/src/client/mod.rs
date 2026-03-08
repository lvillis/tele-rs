mod config;
mod layers;
mod observability;

#[cfg(feature = "_async")]
mod async_client;
#[cfg(feature = "_blocking")]
mod blocking_client;

#[cfg(feature = "_async")]
pub use async_client::Client;
#[cfg(feature = "_blocking")]
pub use blocking_client::BlockingClient;
pub use config::{ClientBuilder, RateLimitConfig, RequestDefaults, RetryConfig};
#[cfg(feature = "bot")]
pub(crate) use layers::reply_chat_id;
#[cfg(feature = "_blocking")]
pub use layers::{BlockingErgoApi, BlockingRawApi, BlockingTypedApi};
#[cfg(feature = "_blocking")]
pub use layers::{BlockingStartupApi, BlockingWebAppApi};
pub use layers::{
    BootstrapFetchStepReport, BootstrapGetMePolicy, BootstrapOutcome, BootstrapPlan,
    BootstrapReport, BootstrapRetryPolicy, BootstrapStepDiagnostics, BootstrapStepPhase,
    BootstrapStepStatus, BootstrapSyncStepReport, MenuButtonConfig, WebAppQueryPayload,
};
#[cfg(feature = "_async")]
pub use layers::{ErgoApi, RawApi, StartupApi, TypedApi, WebAppApi};
pub use observability::{ClientMetric, ClientMetricHook};
pub(crate) use observability::{ClientObservability, emit_client_metric};

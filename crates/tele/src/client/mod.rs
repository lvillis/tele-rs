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
#[cfg(feature = "_async")]
pub use layers::{AppApi, RawApi, SetupApi, TypedApi, WebAppApi};
#[cfg(feature = "_blocking")]
pub use layers::{BlockingAppApi, BlockingRawApi, BlockingTypedApi};
#[cfg(feature = "_blocking")]
pub use layers::{BlockingSetupApi, BlockingWebAppApi};
pub use layers::{
    BootstrapFetchStepReport, BootstrapGetMePolicy, BootstrapOutcome, BootstrapPlan,
    BootstrapReport, BootstrapRetryPolicy, BootstrapStepDiagnostics, BootstrapStepPhase,
    BootstrapStepStatus, BootstrapSyncStepReport, MenuButtonConfig, WebAppQueryPayload,
};
pub use observability::{ClientMetric, ClientMetricHook};
pub(crate) use observability::{ClientObservability, emit_client_metric};

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
#[cfg(feature = "_async")]
pub use layers::{AppApi, ControlApi, ModerationApi, RawApi, SetupApi, TypedApi, WebAppApi};
pub use layers::{
    BanMemberOptions, BootstrapFetchStepReport, BootstrapGetMePolicy, BootstrapOutcome,
    BootstrapPlan, BootstrapReport, BootstrapRetryPolicy, BootstrapStepDiagnostics,
    BootstrapStepPhase, BootstrapStepStatus, BootstrapSyncStepReport, MenuButtonConfig,
    RestrictMemberOptions, WebAppQueryPayload,
};
#[cfg(feature = "_blocking")]
pub use layers::{
    BlockingAppApi, BlockingControlApi, BlockingModerationApi, BlockingRawApi, BlockingTypedApi,
};
#[cfg(feature = "_blocking")]
pub use layers::{BlockingSetupApi, BlockingWebAppApi};
pub use observability::{ClientMetric, ClientMetricHook};
pub(crate) use observability::{ClientObservability, emit_client_metric};

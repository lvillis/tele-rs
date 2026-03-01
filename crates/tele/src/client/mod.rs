mod config;
mod layers;

#[cfg(feature = "async")]
mod async_client;
#[cfg(feature = "blocking")]
mod blocking_client;

#[cfg(feature = "async")]
pub use async_client::Client;
#[cfg(feature = "blocking")]
pub use blocking_client::BlockingClient;
pub use config::{ClientBuilder, RateLimitConfig, RequestDefaults, RetryConfig};
#[cfg(feature = "blocking")]
pub use layers::{BlockingErgoApi, BlockingRawApi, BlockingTypedApi};
#[cfg(feature = "async")]
pub use layers::{ErgoApi, RawApi, TypedApi};

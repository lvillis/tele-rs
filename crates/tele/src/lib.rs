#![forbid(unsafe_code)]
// The SDK exposes rich structured errors by design.
#![allow(clippy::result_large_err)]

//! `tele` is an ergonomic Telegram Bot API SDK for Rust.
//! Built on `reqx`, with strict error modeling and async/blocking parity.

#[cfg(not(any(feature = "async", feature = "blocking")))]
compile_error!("tele requires at least one feature: `async` or `blocking`.");

mod transport;
mod util;

pub mod api;
pub mod auth;
#[cfg(feature = "bot")]
pub mod bot;
pub mod client;
pub mod error;
pub mod prelude;
pub mod types;

pub use auth::{Auth, BotToken};
#[cfg(feature = "blocking")]
pub use client::BlockingClient;
#[cfg(feature = "async")]
pub use client::Client;
#[cfg(feature = "blocking")]
pub use client::{BlockingErgoApi, BlockingRawApi, BlockingTypedApi};
pub use client::{ClientBuilder, RateLimitConfig, RetryConfig};
#[cfg(feature = "async")]
pub use client::{ErgoApi, RawApi, TypedApi};
pub use error::{Error, ErrorClass, Result};
#[cfg(feature = "macros")]
pub use tele_macros::BotCommands;
pub use types::UploadFile;

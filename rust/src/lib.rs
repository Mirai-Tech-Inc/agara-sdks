//! Rust client for the agara prediction-markets trading API.

pub mod client;
pub mod error;
#[cfg(feature = "streaming")]
pub mod frames;
pub mod ids;
pub mod models;
pub mod problem;
pub mod retry;
#[cfg(feature = "signing")]
pub mod signing;
#[cfg(feature = "streaming")]
pub mod stream;
pub mod units;

pub use client::{AgaraClient, DEFAULT_BASE_URL};
pub use error::{AgaraError, Result};
pub use problem::{ProblemDetails, PublicFailure, Recovery};
pub use retry::RetryPolicy;
pub use units::{MICRO, Micro};

#[cfg(feature = "signing")]
pub use signing::{EngineDomain, SignedOrder, sign_limit_order};

#[cfg(feature = "streaming")]
pub use frames::{Channel, Frame};
#[cfg(feature = "streaming")]
pub use stream::{AgaraStreamClient, MAX_SUBSCRIPTIONS_PER_CONNECTION, Reconnect, StreamHandle};

#![doc = core::include_str!("../README.md")]

pub mod activity;
pub mod batch_signing;
pub mod batches;
pub mod bridge;
pub mod catalogue;
pub mod client;
pub mod error;
pub mod frames;
pub mod ids;
pub mod incentives;
pub mod models;
pub mod pnl;
pub mod prices;
pub mod problem;
pub mod retry;
pub mod signing;
pub mod stream;
pub mod units;
pub mod validation;
pub mod values;

mod endpoints;
mod input;

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

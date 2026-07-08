//! The SDK error type and the HTTP-status → variant mapping.

use core::time::Duration;

/// Result alias used throughout the crate.
pub type Result<T> = core::result::Result<T, AgaraError>;

/// Every error the SDK can surface. Server-originated failures are keyed
/// by HTTP status so callers can match a specific class; transport and
/// decode failures are distinct.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum AgaraError {
	/// 400 — malformed request body or invalid parameter values.
	#[error("[400] {message}")]
	BadRequest { message: String },
	/// 401 — missing / invalid / revoked / expired token.
	#[error("[401] {message}")]
	Auth { message: String },
	/// 403 — token is valid but lacks the required scope.
	#[error("[403] {message}")]
	Forbidden { message: String },
	/// 404 — order, market, or token id doesn't exist or isn't yours.
	#[error("[404] {message}")]
	NotFound { message: String },
	/// 409 — e.g. cancelling an already-terminal order, or a duplicate
	/// signed-order hash.
	#[error("[409] {message}")]
	Conflict { message: String },
	/// 422 — order rejected (insufficient balance / shares, FOK couldn't
	/// fill, post-only would cross, market halted).
	#[error("[422] {message}")]
	Rejected { message: String },
	/// 429 — per-tier token bucket exhausted. `retry_after` carries the
	/// server's hint when present.
	#[error("[429] {message}")]
	RateLimited {
		message: String,
		retry_after: Option<Duration>,
	},
	/// 5xx — temporary platform problem. Safe to retry with backoff.
	#[error("[{status}] {message}")]
	Server { status: u16, message: String },
	/// Any other non-success status the SDK doesn't categorize.
	#[error("[{status}] {message}")]
	Api { status: u16, message: String },
	/// A request never produced an HTTP response (DNS, connect, TLS,
	/// timeout).
	#[error("transport error: {0}")]
	Transport(#[from] reqwest::Error),
	/// Client-side validation caught a bad argument before sending.
	#[error("invalid request: {0}")]
	Validation(String),
	/// The response status was fine but the body didn't decode.
	#[error("decode error: {0}")]
	Decode(String),
}

impl AgaraError {
	/// Map an HTTP status + parsed message to the matching variant.
	pub(crate) fn from_status(status: u16, message: String, retry_after: Option<Duration>) -> Self {
		match status {
			400 => Self::BadRequest { message },
			401 => Self::Auth { message },
			403 => Self::Forbidden { message },
			404 => Self::NotFound { message },
			409 => Self::Conflict { message },
			422 => Self::Rejected { message },
			429 => Self::RateLimited { message, retry_after },
			500..=599 => Self::Server { status, message },
			_ => Self::Api { status, message },
		}
	}

	/// The HTTP status behind this error, if it originated from a
	/// response. `None` for transport / validation / decode / stream
	/// errors.
	pub fn status_code(&self) -> Option<u16> {
		match self {
			Self::BadRequest { .. } => Some(400),
			Self::Auth { .. } => Some(401),
			Self::Forbidden { .. } => Some(403),
			Self::NotFound { .. } => Some(404),
			Self::Conflict { .. } => Some(409),
			Self::Rejected { .. } => Some(422),
			Self::RateLimited { .. } => Some(429),
			Self::Server { status, .. } | Self::Api { status, .. } => Some(*status),
			_ => None,
		}
	}

	/// The server's retry hint, present only on a 429.
	pub fn retry_after(&self) -> Option<Duration> {
		match self {
			Self::RateLimited { retry_after, .. } => *retry_after,
			_ => None,
		}
	}

	/// Whether retrying the same request could plausibly succeed — 429,
	/// 5xx, or a transport failure.
	pub fn is_retryable(&self) -> bool {
		matches!(
			self,
			Self::RateLimited { .. } | Self::Server { .. } | Self::Transport(_)
		)
	}
}

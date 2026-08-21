//! The SDK error type and the HTTP-status → variant mapping.

use core::time::Duration;

use crate::problem::ProblemDetails;

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
	BadRequest {
		message: String,
		problem: Option<Box<ProblemDetails>>,
	},
	/// 401 — missing / invalid / revoked / expired token.
	#[error("[401] {message}")]
	Auth {
		message: String,
		problem: Option<Box<ProblemDetails>>,
	},
	/// 403 — token is valid but lacks the required scope.
	#[error("[403] {message}")]
	Forbidden {
		message: String,
		problem: Option<Box<ProblemDetails>>,
	},
	/// 404 — order, market, or token id doesn't exist or isn't yours.
	#[error("[404] {message}")]
	NotFound {
		message: String,
		problem: Option<Box<ProblemDetails>>,
	},
	/// 405 — the route exists but does not support this method.
	#[error("[405] {message}")]
	MethodNotAllowed {
		message: String,
		problem: Option<Box<ProblemDetails>>,
	},
	/// 409 — e.g. cancelling an already-terminal order, or a duplicate
	/// signed-order hash.
	#[error("[409] {message}")]
	Conflict {
		message: String,
		problem: Option<Box<ProblemDetails>>,
	},
	/// 410 — the endpoint was retired.
	#[error("[410] {message}")]
	Gone {
		message: String,
		problem: Option<Box<ProblemDetails>>,
	},
	/// 413 — the request body exceeded the server limit.
	#[error("[413] {message}")]
	PayloadTooLarge {
		message: String,
		problem: Option<Box<ProblemDetails>>,
	},
	/// 415 — the request was not sent as supported JSON.
	#[error("[415] {message}")]
	UnsupportedMediaType {
		message: String,
		problem: Option<Box<ProblemDetails>>,
	},
	/// 422 — order rejected (insufficient balance / shares, FOK couldn't
	/// fill, post-only would cross, market halted).
	#[error("[422] {message}")]
	Rejected {
		message: String,
		problem: Option<Box<ProblemDetails>>,
	},
	/// 424 — a prerequisite such as wallet setup is incomplete.
	#[error("[424] {message}")]
	FailedDependency {
		message: String,
		problem: Option<Box<ProblemDetails>>,
	},
	/// 425 — the requested derived result is not ready yet.
	#[error("[425] {message}")]
	TooEarly {
		message: String,
		problem: Option<Box<ProblemDetails>>,
	},
	/// 426 — the route requires a WebSocket upgrade.
	#[error("[426] {message}")]
	UpgradeRequired {
		message: String,
		problem: Option<Box<ProblemDetails>>,
	},
	/// 429 — per-tier token bucket exhausted. `retry_after` carries the
	/// server's hint when present.
	#[error("[429] {message}")]
	RateLimited {
		message: String,
		retry_after: Option<Duration>,
		problem: Option<Box<ProblemDetails>>,
	},
	/// 5xx — platform or dependency failure. Retry only when
	/// [`AgaraError::is_retryable`] is true.
	#[error("[{status}] {message}")]
	Server {
		status: u16,
		message: String,
		problem: Option<Box<ProblemDetails>>,
	},
	/// Any other non-success status the SDK doesn't categorize.
	#[error("[{status}] {message}")]
	Api {
		status: u16,
		message: String,
		problem: Option<Box<ProblemDetails>>,
	},
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
	pub(crate) fn from_status(
		status: u16,
		message: String,
		retry_after: Option<Duration>,
		problem: Option<ProblemDetails>,
	) -> Self {
		let problem = problem.map(Box::new);
		match status {
			400 => Self::BadRequest { message, problem },
			401 => Self::Auth { message, problem },
			403 => Self::Forbidden { message, problem },
			404 => Self::NotFound { message, problem },
			405 => Self::MethodNotAllowed { message, problem },
			409 => Self::Conflict { message, problem },
			410 => Self::Gone { message, problem },
			413 => Self::PayloadTooLarge { message, problem },
			415 => Self::UnsupportedMediaType { message, problem },
			422 => Self::Rejected { message, problem },
			424 => Self::FailedDependency { message, problem },
			425 => Self::TooEarly { message, problem },
			426 => Self::UpgradeRequired { message, problem },
			429 => Self::RateLimited { message, retry_after, problem },
			500..=599 => Self::Server { status, message, problem },
			_ => Self::Api { status, message, problem },
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
			Self::MethodNotAllowed { .. } => Some(405),
			Self::Conflict { .. } => Some(409),
			Self::Gone { .. } => Some(410),
			Self::PayloadTooLarge { .. } => Some(413),
			Self::UnsupportedMediaType { .. } => Some(415),
			Self::Rejected { .. } => Some(422),
			Self::FailedDependency { .. } => Some(424),
			Self::TooEarly { .. } => Some(425),
			Self::UpgradeRequired { .. } => Some(426),
			Self::RateLimited { .. } => Some(429),
			Self::Server { status, .. } | Self::Api { status, .. } => Some(*status),
			_ => None,
		}
	}

	/// The server's retry hint, present only on a 429.
	pub fn retry_after(&self) -> Option<Duration> {
		match self {
			Self::RateLimited { retry_after, .. } if retry_after.is_some() => *retry_after,
			_ => self.problem().and_then(|problem| problem.recovery.retry_after()),
		}
	}

	/// Parsed Problem Details, when the response used the typed contract.
	pub fn problem(&self) -> Option<&ProblemDetails> {
		match self {
			Self::BadRequest { problem, .. }
			| Self::Auth { problem, .. }
			| Self::Forbidden { problem, .. }
			| Self::NotFound { problem, .. }
			| Self::MethodNotAllowed { problem, .. }
			| Self::Conflict { problem, .. }
			| Self::Gone { problem, .. }
			| Self::PayloadTooLarge { problem, .. }
			| Self::UnsupportedMediaType { problem, .. }
			| Self::Rejected { problem, .. }
			| Self::FailedDependency { problem, .. }
			| Self::TooEarly { problem, .. }
			| Self::UpgradeRequired { problem, .. }
			| Self::RateLimited { problem, .. }
			| Self::Server { problem, .. }
			| Self::Api { problem, .. } => problem.as_deref(),
			_ => None,
		}
	}

	/// Whether retrying the same request is explicitly permitted. Transport
	/// failures and legacy 429 responses remain retryable; a 5xx status alone
	/// is deliberately not enough.
	pub fn is_retryable(&self) -> bool {
		if let Some(problem) = self.problem() {
			return problem.is_retryable();
		}

		matches!(self, Self::RateLimited { .. } | Self::Transport(_))
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn problem(status: u16, code: &str, recovery: &str) -> ProblemDetails {
		serde_json::from_value(serde_json::json!({
			"type": "urn:agara:problem:test",
			"title": "Test failure",
			"status": status,
			"code": code,
			"request_id": "00000000-0000-0000-0000-000000000000",
			"recovery": { "strategy": recovery }
		}))
		.unwrap()
	}

	#[test]
	fn typed_recovery_controls_retry_instead_of_status() {
		let terminal = AgaraError::from_status(
			503,
			"Test failure".to_owned(),
			None,
			Some(problem(503, "internal_error", "none")),
		);
		let transient = AgaraError::from_status(
			425,
			"Test failure".to_owned(),
			None,
			Some(problem(425, "pnl_not_ready", "retry")),
		);
		assert!(!terminal.is_retryable());
		assert!(transient.is_retryable());
	}

	#[test]
	fn future_code_cannot_activate_known_retry_strategy() {
		let error = AgaraError::from_status(
			503,
			"Future".to_owned(),
			None,
			Some(problem(503, "future_failure", "retry")),
		);
		assert!(!error.is_retryable());
	}

	#[test]
	fn legacy_server_error_is_not_assumed_retryable() {
		let error = AgaraError::from_status(503, "legacy".to_owned(), None, None);
		assert!(!error.is_retryable());
	}

	#[test]
	fn new_contract_statuses_have_dedicated_variants() {
		for status in [405, 410, 413, 415, 424, 425, 426] {
			let error = AgaraError::from_status(status, "test".to_owned(), None, None);
			assert_eq!(error.status_code(), Some(status));
			assert!(!matches!(error, AgaraError::Api { .. }));
		}
	}
}

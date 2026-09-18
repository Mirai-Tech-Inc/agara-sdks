//! Typed local failures and lossless HTTP status classifications.

mod tests;

use core::{fmt, time::Duration};

use std::borrow::Cow;

use crate::{
	ids::Exchange,
	problem::{ProblemDecodeError, ProblemDetails},
	validation::ValidationError,
};

/// Result returned by HTTP client operations.
pub type Result<T> = core::result::Result<T, AgaraError>;

/// HTTP retry timing evidence; unusable hints disable automatic replay.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum RetryAfter {
	/// A finite, nonnegative delay measured from response receipt.
	Delay(Duration),

	/// An unparseable or unrepresentable header, retained for caller-controlled recovery.
	Unusable {
		/// Original header bytes, including HTTP-date values this parser does not interpret.
		value: reqwest::header::HeaderValue,
	},
}

/// Server response evidence retained independently of its HTTP classification.
#[derive(Debug)]
pub struct HttpFailure {
	body: serde_json::Value,
	problem: Option<Box<ProblemDetails>>,
	retry_after: Option<RetryAfter>,
}

/// Response decoding, completeness, or framing failures.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ResponseError {
	/// JSON parsing failed; inspect the original serde error for category, line and column.
	#[error("invalid response JSON: {source}")]
	Json {
		/// Original JSON parser error.
		#[source]
		source: serde_json::Error,
	},

	/// A current typed failure violated the published problem contract.
	#[error("invalid problem contract: {source}")]
	Problem {
		/// Actual HTTP status received with the malformed typed body.
		status: u16,

		/// Original bounded JSON failure body, preserved for diagnostics.
		body: serde_json::Value,

		/// Exact field or registered-metadata invariant that failed.
		#[source]
		source: ProblemDecodeError,
	},

	/// An endpoint that promises a JSON body returned no bytes.
	#[error("response body was empty")]
	EmptyBody,

	/// The response exceeded the configured read bound.
	#[error("response body exceeds {limit} bytes")]
	BodyTooLarge {
		/// Configured maximum response size in bytes.
		limit: usize,
	},

	/// The response is not encoded using the expected protocol media type.
	#[error("unexpected response content type: {content_type:?}")]
	UnexpectedContentType {
		/// Received media type, absent when the server omitted the header.
		content_type: Option<Cow<'static, str>>,
	},

	/// A request with a streaming body cannot be replayed by the retry mechanism.
	#[error("request cannot be cloned for retry")]
	RequestNotCloneable,

	/// A convenience read could not provide a complete requested portfolio.
	#[error("requested portfolio data is unavailable for {exchanges:?}")]
	PartialData {
		/// Exchanges that must be read successfully before reconciling a complete portfolio.
		exchanges: Vec<Exchange>,
	},
}

/// Polling stopped before an authoritative completion state could be observed.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum PollError {
	/// The caller's absolute wait deadline expired.
	#[error("polling deadline elapsed")]
	DeadlineElapsed,

	/// A requested timeout cannot be represented as an absolute deadline.
	#[error("polling deadline cannot be represented")]
	InvalidDeadline,
}

/// A paginated read could not safely finish a complete result walk.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum PaginationError {
	/// The server repeated a previously returned continuation cursor.
	#[error("pagination cursor cycle")]
	CursorCycle,

	/// The walk exceeded its configured maximum number of pages.
	#[error("pagination exceeded {limit} pages")]
	PageLimitExceeded {
		/// Maximum number of pages permitted by the caller's bound.
		limit: usize,
	},
}

/// Errors callers can match without inspecting display text.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum AgaraError {
	/// 400: The request body or query could not be interpreted.
	#[error("[400] {failure}")]
	BadRequest {
		/// Preserved body, canonical problem details and response retry hint.
		failure: HttpFailure,
	},

	/// 401: The server rejected or requires credentials.
	#[error("[401] {failure}")]
	Auth {
		/// Preserved body, canonical problem details and response retry hint.
		failure: HttpFailure,
	},

	/// 403: The caller lacks permission or its account is restricted.
	#[error("[403] {failure}")]
	Forbidden {
		/// Preserved body, canonical problem details and response retry hint.
		failure: HttpFailure,
	},

	/// 404: The requested resource does not exist for this caller.
	#[error("[404] {failure}")]
	NotFound {
		/// Preserved body, canonical problem details and response retry hint.
		failure: HttpFailure,
	},

	/// 405: The endpoint does not accept this HTTP method.
	#[error("[405] {failure}")]
	MethodNotAllowed {
		/// Preserved body, canonical problem details and response retry hint.
		failure: HttpFailure,
	},

	/// 409: The requested operation conflicts with current server state.
	#[error("[409] {failure}")]
	Conflict {
		/// Preserved body, canonical problem details and response retry hint.
		failure: HttpFailure,
	},

	/// 410: The endpoint or resource has been retired.
	#[error("[410] {failure}")]
	Gone {
		/// Preserved body, canonical problem details and response retry hint.
		failure: HttpFailure,
	},

	/// 413: The request exceeded the server body-size limit.
	#[error("[413] {failure}")]
	PayloadTooLarge {
		/// Preserved body, canonical problem details and response retry hint.
		failure: HttpFailure,
	},

	/// 415: The request used an unsupported content type.
	#[error("[415] {failure}")]
	UnsupportedMediaType {
		/// Preserved body, canonical problem details and response retry hint.
		failure: HttpFailure,
	},

	/// 422: The request violated trading or input constraints.
	#[error("[422] {failure}")]
	Rejected {
		/// Preserved body, canonical problem details and response retry hint.
		failure: HttpFailure,
	},

	/// 424: A required wallet or signing prerequisite is incomplete.
	#[error("[424] {failure}")]
	FailedDependency {
		/// Preserved body, canonical problem details and response retry hint.
		failure: HttpFailure,
	},

	/// 425: The requested projection is not yet ready.
	#[error("[425] {failure}")]
	TooEarly {
		/// Preserved body, canonical problem details and response retry hint.
		failure: HttpFailure,
	},

	/// 426: The route requires a protocol upgrade.
	#[error("[426] {failure}")]
	UpgradeRequired {
		/// Preserved body, canonical problem details and response retry hint.
		failure: HttpFailure,
	},

	/// 429: The caller exhausted an applicable request rate limit.
	#[error("[429] {failure}")]
	RateLimited {
		/// Preserved body, canonical problem details and response retry hint.
		failure: HttpFailure,
	},

	/// A server or dependency returned an HTTP 5xx failure.
	#[error("[{status}] {failure}")]
	Server {
		/// Actual response status.
		status: u16,

		/// Preserved response evidence.
		failure: HttpFailure,
	},

	/// An HTTP status outside the SDK's named error classes.
	#[error("unexpected HTTP status {status}: {failure}")]
	UnexpectedStatus {
		/// Actual response status, never replaced by a guessed classification.
		status: u16,

		/// Unmodified response body and any validated typed problem.
		failure: HttpFailure,
	},

	/// HTTP transport failed; the original request/connection error remains inspectable.
	#[error("HTTP transport failed: {0}")]
	Transport(#[from] reqwest::Error),

	/// Local request or configuration validation rejected the operation before I/O.
	#[error(transparent)]
	Validation(#[from] ValidationError),

	/// A response could not be decoded or used as complete state.
	#[error(transparent)]
	Response(#[from] ResponseError),

	/// An explicit wait did not reach an authoritative completion state.
	#[error(transparent)]
	Poll(#[from] PollError),

	/// A complete paginated read could not be established.
	#[error(transparent)]
	Pagination(#[from] PaginationError),

	/// A signed-order envelope or unsigned order shape is invalid.
	#[error(transparent)]
	OrderValidation(#[from] crate::models::OrderValidationError),

	/// A presigned account-batch request violates its documented admission constraints.
	#[error(transparent)]
	BatchValidation(#[from] crate::batches::BatchValidationError),

	/// Local signing failed without erasing the underlying signing or input error.
	#[cfg(feature = "signing")]
	#[error(transparent)]
	Signing(#[from] crate::signing::SigningError),

	/// Typed account operations could not be composed into canonical calldata.
	#[cfg(feature = "signing")]
	#[error(transparent)]
	Compose(#[from] crate::batch_signing::compose::ComposeError),

	/// WebSocket operation, framing, or reconnect policy failed.
	#[cfg(feature = "streaming")]
	#[error(transparent)]
	Stream(#[from] crate::stream::StreamClientError),

	/// A price SSE transport or payload failed.
	#[cfg(feature = "streaming")]
	#[error(transparent)]
	PriceStream(#[from] crate::prices::PriceStreamError),
}

impl HttpFailure {
	/// The complete JSON body, or a JSON string containing a non-JSON response body.
	pub const fn body(&self) -> &serde_json::Value {
		&self.body
	}

	/// Validated typed problem, absent for legacy or non-problem responses.
	pub fn problem(&self) -> Option<&ProblemDetails> {
		self.problem.as_deref()
	}

	/// Server delay hint retained independently of whether a retry is permitted.
	pub fn retry_after(&self) -> Option<Duration> {
		match &self.retry_after {
			Some(RetryAfter::Delay(delay)) => Some(*delay),
			_ => None,
		}
	}

	/// Inspect the header even when its value cannot safely schedule an automatic retry.
	pub const fn retry_hint(&self) -> Option<&RetryAfter> {
		self.retry_after.as_ref()
	}
}

impl ResponseError {
	/// Preserve an unexpected media type without promoting it into a machine error code.
	pub fn unexpected_content_type(content_type: Option<Cow<'static, str>>) -> Self {
		Self::UnexpectedContentType { content_type }
	}
}

impl AgaraError {
	pub(crate) fn from_response(
		status: u16,
		body: serde_json::Value,
		retry_after: Option<RetryAfter>,
		problem: Option<ProblemDetails>,
	) -> Self {
		if let Some(problem) = &problem
			&& let Err(source) =
				problem.validate().and_then(|()| problem.validate_http_status(status))
		{
			return ResponseError::Problem { status, body, source }.into();
		}

		let failure = HttpFailure { body, retry_after, problem: problem.map(Box::new) };

		match status {
			400 => Self::BadRequest { failure },
			401 => Self::Auth { failure },
			403 => Self::Forbidden { failure },
			404 => Self::NotFound { failure },
			405 => Self::MethodNotAllowed { failure },
			409 => Self::Conflict { failure },
			410 => Self::Gone { failure },
			413 => Self::PayloadTooLarge { failure },
			415 => Self::UnsupportedMediaType { failure },
			422 => Self::Rejected { failure },
			424 => Self::FailedDependency { failure },
			425 => Self::TooEarly { failure },
			426 => Self::UpgradeRequired { failure },
			429 => Self::RateLimited { failure },
			500..=599 => Self::Server { status, failure },
			_ => Self::UnexpectedStatus { status, failure },
		}
	}

	/// Actual HTTP response status; local, parsing and transport errors have no status.
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
			Self::Server { status, .. }
			| Self::UnexpectedStatus { status, .. }
			| Self::Response(ResponseError::Problem { status, .. }) => Some(*status),
			_ => None,
		}
	}

	/// Preserved HTTP failure evidence, including an unknown or legacy body.
	pub fn http_failure(&self) -> Option<&HttpFailure> {
		match self {
			Self::BadRequest { failure }
			| Self::Auth { failure }
			| Self::Forbidden { failure }
			| Self::NotFound { failure }
			| Self::MethodNotAllowed { failure }
			| Self::Conflict { failure }
			| Self::Gone { failure }
			| Self::PayloadTooLarge { failure }
			| Self::UnsupportedMediaType { failure }
			| Self::Rejected { failure }
			| Self::FailedDependency { failure }
			| Self::TooEarly { failure }
			| Self::UpgradeRequired { failure }
			| Self::RateLimited { failure }
			| Self::Server { failure, .. }
			| Self::UnexpectedStatus { failure, .. } => Some(failure),
			_ => None,
		}
	}

	/// Validated server Problem Details when available.
	pub fn problem(&self) -> Option<&ProblemDetails> {
		self.http_failure().and_then(HttpFailure::problem)
	}

	/// A retained HTTP hint or canonical body-level delay; this does not authorize a retry.
	pub fn retry_after(&self) -> Option<Duration> {
		self.http_failure()
			.and_then(HttpFailure::retry_after)
			.into_iter()
			.chain(self.problem().and_then(|problem| problem.recovery.retry_after()))
			.max()
	}

	/// Whether a known problem explicitly permits retry. Transport ambiguity requires caller policy.
	pub fn is_retryable(&self) -> bool {
		!self.http_failure().is_some_and(|failure| {
			core::matches!(failure.retry_hint(), Some(RetryAfter::Unusable { .. }))
		}) && self.problem().is_some_and(ProblemDetails::is_retryable)
	}
}

impl fmt::Display for HttpFailure {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		if let Some(problem) = &self.problem {
			return f.write_str(problem.message());
		}

		if let Some(message) = self
			.body
			.as_str()
			.or_else(|| self.body.get("error").and_then(serde_json::Value::as_str))
		{
			return f.write_str(message);
		}

		fmt::Display::fmt(&self.body, f)
	}
}

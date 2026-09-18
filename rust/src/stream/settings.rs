use core::time::Duration;

use std::{borrow::Cow, sync::Arc};

use crate::{frames::FrameDecodeError, retry};

const DEFAULT_MAX_RECONNECTS: u32 = 8;

/// The independent WebSocket transport affected by a failure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StreamEndpoint {
	/// Public market-data subscriptions.
	Market,

	/// Private account subscriptions.
	Account,
}

/// The transport operation that failed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StreamPhase {
	/// Opening the WebSocket connection.
	Connect,

	/// Reading a server message.
	Read,

	/// Sending subscriptions, control messages or a protocol response.
	Write,
}

/// The reconnect setting that failed validation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReconnectField {
	/// First retry delay.
	InitialDelay,

	/// Maximum retry delay.
	MaxDelay,

	/// Fractional jitter, required to be finite and within 0..=1.
	Jitter,

	/// Finite reconnect count above the shared retry budget.
	MaxReconnects,
}

/// Exact reason a configured WebSocket base URL cannot be used.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum StreamUrlError {
	/// URL syntax parsing failed; the original parser classification is retained.
	#[error("URL parsing failed: {0}")]
	Parse(#[from] url::ParseError),

	/// Only HTTP, HTTPS, WS and WSS network URLs are accepted.
	#[error("expected an HTTP(S) or WS(S) URL")]
	UnsupportedScheme,

	/// A network host was absent.
	#[error("stream URL must have a host")]
	MissingHost,

	/// Embedded credentials would create a second authentication mechanism.
	#[error("stream URL must not embed credentials")]
	Credentials,

	/// Query parameters are not a stream base-URL component.
	#[error("stream URL must not contain a query")]
	Query,

	/// Fragments are not sent to the server.
	#[error("stream URL must not contain a fragment")]
	Fragment,

	/// The parsed URL could not be converted to its WebSocket scheme.
	#[error("URL cannot be converted to a WebSocket endpoint")]
	SchemeConversion,
}

/// Local stream failures, separate from the server's public problem contract.
#[derive(Clone, Debug, thiserror::Error)]
#[non_exhaustive]
pub enum StreamClientError {
	/// The base URL could not be used safely for a WebSocket endpoint.
	#[error("invalid stream base URL: {0}")]
	InvalidBaseUrl(#[from] StreamUrlError),

	/// A configured bearer was empty or contained whitespace/control characters.
	#[error("stream bearer must be nonempty and contain no whitespace or control characters")]
	InvalidToken,

	/// Account subscriptions require a configured bearer.
	#[error("account_events requires a bearer token")]
	MissingToken,

	/// At least one subscription is required before connecting.
	#[error("stream has no subscriptions")]
	NoSubscriptions,

	/// A reconnect setting could overflow, spin without delay or produce invalid jitter.
	#[error("invalid reconnect configuration: {field:?}")]
	InvalidReconnect {
		/// Setting that violated its documented invariant.
		field: ReconnectField,
	},

	/// The endpoint's distinct-subscription limit would be exceeded.
	#[error("{endpoint:?} subscriptions exceed {limit}")]
	SubscriptionLimit {
		/// Endpoint that would exceed its limit.
		endpoint: StreamEndpoint,

		/// Maximum distinct subscriptions.
		limit: usize,
	},

	/// The requested endpoint was not part of the initial connection set.
	#[error("{endpoint:?} endpoint is not open")]
	EndpointNotOpen {
		/// Missing endpoint.
		endpoint: StreamEndpoint,
	},

	/// A control operation targeted an endpoint whose task had already stopped.
	#[error("{endpoint:?} endpoint is closed")]
	EndpointClosed {
		/// Stopped endpoint.
		endpoint: StreamEndpoint,
	},

	/// A bounded control queue is full; no operation was silently discarded.
	#[error("{endpoint:?} control queue is full")]
	ControlBackpressure {
		/// Endpoint whose queue is full.
		endpoint: StreamEndpoint,
	},

	/// The consumer did not drain the bounded output queue; reconnect and reconcile.
	#[error("{endpoint:?} output queue is full; reconciliation is required")]
	Backpressure {
		/// Endpoint stopped to avoid silent frame loss.
		endpoint: StreamEndpoint,
	},

	/// The WebSocket connection deadline elapsed.
	#[error("{endpoint:?} connection timed out")]
	ConnectTimeout {
		/// Endpoint whose connection timed out.
		endpoint: StreamEndpoint,
	},

	/// The established connection stopped receiving server messages.
	#[error("{endpoint:?} stream idle deadline elapsed")]
	IdleTimeout {
		/// Silent endpoint.
		endpoint: StreamEndpoint,
	},

	/// A WebSocket operation failed with its original transport cause retained.
	#[error("{endpoint:?} {phase:?} failed: {source}")]
	Transport {
		/// Affected endpoint.
		endpoint: StreamEndpoint,

		/// Failed transport operation.
		phase: StreamPhase,

		/// Original tungstenite transport/protocol failure.
		#[source]
		source: Arc<tokio_tungstenite::tungstenite::Error>,
	},

	/// The peer closed the socket; the actual close code and reason are retained.
	#[error("{endpoint:?} closed with code {code:?}: {reason:?}")]
	Closed {
		/// Closed endpoint.
		endpoint: StreamEndpoint,

		/// Peer close code, absent when no close frame arrived.
		code: Option<u16>,

		/// Peer-supplied reason; absence means no close frame arrived.
		reason: Option<Cow<'static, str>>,
	},

	/// A text message did not satisfy the known JSON/frame contract.
	#[error("{endpoint:?} frame decoding failed: {source}")]
	Decode {
		/// Endpoint that sent the malformed frame.
		endpoint: StreamEndpoint,

		/// Precise local decode cause.
		#[source]
		source: FrameDecodeError,
	},

	/// All configured reconnects were consumed, or the retry counter reached its representable bound.
	#[error("{endpoint:?} reconnect limit reached after {reconnects} reconnects")]
	ReconnectExhausted {
		/// Endpoint that stopped retrying.
		endpoint: StreamEndpoint,

		/// Reconnect attempts after the initial attempt.
		reconnects: u32,
	},

	/// Repeated server-directed resubscriptions exceeded the bounded recovery budget.
	#[error("{endpoint:?} resubscribe limit reached after {attempts} attempts")]
	ResubscribeExhausted {
		/// Affected endpoint.
		endpoint: StreamEndpoint,

		/// Recovery attempts already sent.
		attempts: u32,
	},

	/// The monotonic clock cannot represent the requested future deadline.
	#[error("stream deadline is outside the monotonic clock range")]
	DeadlineOverflow,
}

/// Validated reconnect policy. Delays are positive, ordered and representable by the monotonic clock.
/// `max_reconnects` counts retries after the initial connection; `Some(0)` disables retries.
#[derive(Clone, Debug, dissolve_derive::Dissolve)]
#[dissolve(visibility = "pub(crate)")]
pub struct Reconnect {
	initial_delay: Duration,
	max_delay: Duration,
	jitter: retry::Jitter,
	max_reconnects: Option<retry::RetryLimit>,
}

impl StreamEndpoint {
	pub(super) const fn path(self) -> &'static str {
		match self {
			Self::Market => "/trade/v1/market-stream",
			Self::Account => "/trade/v1/account-stream",
		}
	}
}

#[bon::bon]
impl StreamClientError {
	/// Construct a base-URL validation failure.
	pub fn invalid_base_url(reason: StreamUrlError) -> Self {
		Self::InvalidBaseUrl(reason)
	}

	/// Preserve a peer close reason without interpreting it as a server Problem Details code.
	#[builder]
	pub fn closed(
		endpoint: StreamEndpoint,
		code: Option<u16>,
		reason: Option<Cow<'static, str>>,
	) -> Self {
		Self::Closed { endpoint, code, reason }
	}
}

#[bon::bon]
impl Reconnect {
	/// Construct a policy, rejecting non-finite/out-of-range jitter and zero/unordered/overflowing delays.
	#[builder]
	pub fn new(
		#[builder(default = retry::DEFAULT_INITIAL_BACKOFF)] initial_delay: Duration,
		#[builder(default = retry::DEFAULT_MAX_BACKOFF)] max_delay: Duration,
		#[builder(default = retry::DEFAULT_JITTER)] jitter: f64,
		#[builder(required, default = Some(DEFAULT_MAX_RECONNECTS))] max_reconnects: Option<u32>,
	) -> core::result::Result<Self, StreamClientError> {
		let jitter = retry::Jitter::new(jitter)
			.map_err(|_| StreamClientError::InvalidReconnect { field: ReconnectField::Jitter })?;
		let max_reconnects =
			max_reconnects.map(retry::RetryLimit::new).transpose().map_err(|_| {
				StreamClientError::InvalidReconnect { field: ReconnectField::MaxReconnects }
			})?;
		let policy = Self { initial_delay, max_delay, jitter, max_reconnects };
		policy.validate()?;

		Ok(policy)
	}

	/// First reconnect delay before exponential growth.
	pub const fn initial_delay(&self) -> Duration {
		self.initial_delay
	}

	/// Hard ceiling including jitter.
	pub const fn max_delay(&self) -> Duration {
		self.max_delay
	}

	/// Finite fractional jitter in 0..=1.
	pub const fn jitter(&self) -> retry::Jitter {
		self.jitter
	}

	/// Maximum total reconnects for this stream, or `None` for retries until counter exhaustion.
	pub fn max_reconnects(&self) -> Option<u32> {
		self.max_reconnects.map(retry::RetryLimit::raw)
	}

	pub(super) fn validate(&self) -> core::result::Result<(), StreamClientError> {
		if self.initial_delay.is_zero() || self.initial_delay > self.max_delay {
			return Err(StreamClientError::InvalidReconnect {
				field: ReconnectField::InitialDelay,
			});
		}
		if self.max_delay.is_zero()
			|| tokio::time::Instant::now().checked_add(self.max_delay).is_none()
		{
			return Err(StreamClientError::InvalidReconnect { field: ReconnectField::MaxDelay });
		}

		Ok(())
	}
}

impl Default for Reconnect {
	fn default() -> Self {
		Self::builder().build().expect("static reconnect defaults satisfy validation")
	}
}

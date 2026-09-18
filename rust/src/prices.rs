#![cfg(feature = "streaming")]

//! Exact provider-price SSE streams. Drop a stream to cancel its connection.
//! End of a complete response is `None`; truncation, idle expiry and malformed data are typed errors.

use core::{fmt, time::Duration};

use std::{borrow::Cow, collections::VecDeque};

use crate::{AgaraError, Result, client::AgaraClient};

const MAX_EVENT_BYTES: usize = 1_048_576;
const MAX_PROBLEM_BYTES: usize = 65_536;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const MAX_SYMBOLS: usize = 64;
const MAX_SYMBOL_LENGTH: usize = 64;
const IDLE_TIMEOUT: Duration = Duration::from_secs(45);

/// Which price-stream operation encountered a transport failure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PriceStreamPhase {
	/// Opening the HTTP response.
	Connect,

	/// Reading the server's rejection body.
	ErrorBody,

	/// Reading an established event stream.
	Read,
}

/// Local configuration, transport and protocol failures for price SSE.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum PriceStreamError {
	/// Symbols must match the provider's bounded ASCII grammar.
	#[error("invalid provider symbol: {0}")]
	InvalidSymbol(Cow<'static, str>),

	/// A subscription must contain one through 64 symbols.
	#[error("price subscription has {count} symbols; expected 1..=64")]
	SymbolCount {
		/// Number of supplied symbols.
		count: usize,
	},

	/// A price must be a signed decimal integer without whitespace or exponent notation.
	#[error("invalid price mantissa: {0}")]
	InvalidMantissa(Cow<'static, str>),

	/// The HTTP handshake did not complete before its deadline.
	#[error("price stream connection timed out")]
	ConnectTimeout,

	/// An established stream stopped delivering bytes before its idle deadline.
	#[error("price stream idle deadline elapsed")]
	IdleTimeout,

	/// A rejected connection's body did not complete before its deadline.
	#[error("price stream error-body deadline elapsed")]
	ErrorBodyTimeout,

	/// The HTTP transport failed at the indicated phase.
	#[error("price stream transport failed during {phase:?}: {source}")]
	Network {
		/// Operation that failed.
		phase: PriceStreamPhase,

		/// Original transport cause.
		#[source]
		source: reqwest::Error,
	},

	/// The server ended in the middle of an SSE event.
	#[error("price stream ended with an incomplete event ({pending_bytes} bytes)")]
	UnexpectedEof {
		/// Bytes since the last complete event boundary.
		pending_bytes: usize,
	},

	/// One event or error response exceeded its explicit byte bound.
	#[error("price stream item exceeds its {limit} byte limit")]
	TooLarge {
		/// Maximum allowed bytes for the item.
		limit: usize,
	},

	/// An SSE line contained malformed UTF-8.
	#[error("invalid SSE UTF-8: {0}")]
	Utf8(#[from] core::str::Utf8Error),

	/// A complete SSE data event was not JSON.
	#[error("invalid SSE JSON: {0}")]
	Json(#[source] crate::problem::ProblemDecodeError),

	/// JSON was valid but did not contain a valid provider-price payload.
	#[error("invalid price payload: {0}")]
	Payload(#[source] serde_json::Error),

	/// A failure body did not satisfy the canonical Problem Details contract.
	#[error("invalid price-stream rejection body for HTTP {status}: {source}")]
	InvalidProblem {
		/// Actual HTTP status.
		status: u16,

		/// JSON or contract validation cause.
		#[source]
		source: crate::problem::ProblemDecodeError,
	},

	/// The response was successful but did not advertise the SSE media type.
	#[error("expected text/event-stream, received {0:?}")]
	InvalidContentType(Option<Cow<'static, str>>),

	/// Content-Type bytes were not valid HTTP header text.
	#[error("invalid SSE Content-Type header encoding: {0}")]
	ContentTypeEncoding(#[source] reqwest::header::ToStrError),

	/// A retry field exceeded the representation's integer bound.
	#[error("SSE retry milliseconds exceed u64")]
	RetryOverflow,

	/// An earlier error poisoned this decoder; create a fresh decoder before resuming.
	#[error("SSE decoder cannot resume after a decoding failure")]
	DecoderFailed,
}

/// A signed decimal mantissa with no rounding or exponent conversion.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(transparent)]
pub struct PriceMantissa(String);

/// A nonempty provider symbol of at most 64 ASCII letters, digits, `.`, `_`, `/`, or `-`.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(transparent)]
pub struct ProviderSymbol(String);

/// Exact provider price and publish timestamp.
#[derive(Clone, Debug, serde::Deserialize)]
pub struct ProviderPrice {
	/// Validated exact signed integer mantissa.
	pub price: PriceMantissa,

	/// Base-ten exponent; it is not applied by the SDK.
	pub expo: i32,

	/// Nonnegative Unix milliseconds, decoded without floating-point conversion.
	pub publish_time_ms: u64,
}

/// One symbol and its latest provider value.
#[derive(Clone, Debug, serde::Deserialize)]
pub struct PriceEntry {
	/// Validated provider symbol.
	pub id: ProviderSymbol,

	/// Exact provider value.
	pub price: ProviderPrice,
}

/// Hermes-compatible JSON emitted by both price SSE endpoints.
#[derive(Clone, Debug, serde::Deserialize)]
pub struct PriceFrame {
	/// Entries supplied by the provider; an empty frame is preserved.
	pub parsed: Vec<PriceEntry>,
}

/// One decoded event with SSE metadata preserved.
#[derive(Clone, Debug)]
pub struct PriceEvent {
	/// Named event type; `None` has SSE's default message semantics.
	pub event: Option<String>,

	/// Last valid SSE event identifier, if supplied by the server.
	pub id: Option<String>,

	/// Latest valid server retry hint; this does not trigger automatic reconnection.
	pub retry: Option<Duration>,

	/// Validated price data.
	pub frame: PriceFrame,
}

/// Incremental byte decoder supporting LF, CRLF, CR and split UTF-8.
/// Bounds apply per event, so a chunk may contain many small events.
#[derive(Default)]
pub struct SseDecoder {
	line: Vec<u8>,
	data: Vec<String>,
	name: Option<String>,
	last_id: Option<String>,
	retry: Option<Duration>,
	event_bytes: usize,
	ignore_lf: bool,
	first_line: bool,
	failed: bool,
}

/// A live pull-based SSE response. Dropping it cancels the underlying response.
pub struct PriceStream {
	response: reqwest::Response,
	decoder: SseDecoder,
	pending: VecDeque<PriceEvent>,
	finished: bool,
}

fn validate_symbols(symbols: &[&str]) -> core::result::Result<(), PriceStreamError> {
	if symbols.is_empty() || symbols.len() > MAX_SYMBOLS {
		return Err(PriceStreamError::SymbolCount { count: symbols.len() });
	}

	for symbol in symbols {
		ProviderSymbol::new((*symbol).to_owned())?;
	}

	Ok(())
}

async fn read_problem(
	mut response: reqwest::Response,
) -> core::result::Result<Vec<u8>, PriceStreamError> {
	let mut bytes = Vec::new();
	while let Some(chunk) = response.chunk().await.map_err(|source| PriceStreamError::Network {
		phase: PriceStreamPhase::ErrorBody,
		source,
	})? {
		if bytes.len().checked_add(chunk.len()).is_none_or(|size| size > MAX_PROBLEM_BYTES) {
			return Err(PriceStreamError::TooLarge { limit: MAX_PROBLEM_BYTES });
		}
		bytes.extend_from_slice(&chunk);
	}

	Ok(bytes)
}

impl PriceStreamError {
	/// Construct a symbol validation failure.
	pub fn invalid_symbol<E>(message: E) -> Self
	where
		Cow<'static, str>: From<E>,
	{
		Self::InvalidSymbol(Cow::from(message))
	}

	/// Preserve the actual media type; `None` means the header was absent.
	pub fn invalid_content_type(value: Option<Cow<'static, str>>) -> Self {
		Self::InvalidContentType(value)
	}

	/// Construct a mantissa validation failure.
	pub fn invalid_mantissa<E>(message: E) -> Self
	where
		Cow<'static, str>: From<E>,
	{
		Self::InvalidMantissa(Cow::from(message))
	}
}

impl PriceMantissa {
	/// Validate a signed decimal integer. Empty, fractional and non-finite text is rejected.
	pub fn new(value: impl Into<String>) -> core::result::Result<Self, PriceStreamError> {
		let value = value.into();
		let digits = value.strip_prefix('-').unwrap_or(&value);
		if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
			return Err(PriceStreamError::invalid_mantissa(value));
		}

		Ok(Self(value))
	}

	/// Original exact decimal digits, including any sign.
	pub fn as_str(&self) -> &str {
		&self.0
	}
}

impl ProviderSymbol {
	/// Validate the current provider-symbol grammar without normalizing the identifier.
	pub fn new(value: impl Into<String>) -> core::result::Result<Self, PriceStreamError> {
		let value = value.into();
		if value.is_empty()
			|| value.len() > MAX_SYMBOL_LENGTH
			|| !value.bytes().all(|byte| byte.is_ascii_alphanumeric() || b"._/-".contains(&byte))
		{
			return Err(PriceStreamError::invalid_symbol(value));
		}

		Ok(Self(value))
	}

	/// Original provider identifier.
	pub fn as_str(&self) -> &str {
		&self.0
	}
}

impl SseDecoder {
	/// Decode complete events, retaining only the bounded unfinished event.
	/// After an error the decoder is poisoned and must be replaced.
	pub fn push(
		&mut self,
		bytes: &[u8],
	) -> core::result::Result<Vec<PriceEvent>, PriceStreamError> {
		if self.failed {
			return Err(PriceStreamError::DecoderFailed);
		}

		let result = self.push_inner(bytes);
		if result.is_err() {
			self.failed = true;
		}

		result
	}

	/// Validate that EOF occurred exactly between complete SSE events.
	pub fn finish(&mut self) -> core::result::Result<(), PriceStreamError> {
		if self.failed {
			return Err(PriceStreamError::DecoderFailed);
		}
		if self.event_bytes != 0 || !self.line.is_empty() || !self.data.is_empty() {
			self.failed = true;
			return Err(PriceStreamError::UnexpectedEof { pending_bytes: self.event_bytes });
		}

		Ok(())
	}

	fn push_inner(
		&mut self,
		bytes: &[u8],
	) -> core::result::Result<Vec<PriceEvent>, PriceStreamError> {
		let mut events = Vec::new();
		for byte in bytes {
			if self.ignore_lf {
				self.ignore_lf = false;
				if *byte == b'\n' {
					continue;
				}
			}

			self.event_bytes = self
				.event_bytes
				.checked_add(1)
				.filter(|size| *size <= MAX_EVENT_BYTES)
				.ok_or(PriceStreamError::TooLarge { limit: MAX_EVENT_BYTES })?;
			if *byte == b'\r' || *byte == b'\n' {
				self.ignore_lf = *byte == b'\r';
				if let Some(event) = self.finish_line()? {
					events.push(event);
				}
			} else {
				self.line.push(*byte);
			}
		}

		Ok(events)
	}

	fn finish_line(&mut self) -> core::result::Result<Option<PriceEvent>, PriceStreamError> {
		let bytes = core::mem::take(&mut self.line);
		let line = core::str::from_utf8(&bytes)?;
		let line = if !self.first_line {
			self.first_line = true;
			line.strip_prefix('\u{feff}').unwrap_or(line)
		} else {
			line
		};
		if line.is_empty() {
			self.event_bytes = 0;
			let data = core::mem::take(&mut self.data);
			let name = self.name.take();
			if data.is_empty() {
				return Ok(None);
			}

			let json = crate::problem::parse_json_value_with_limit(
				data.join("\n").as_bytes(),
				MAX_EVENT_BYTES,
			)
			.map_err(PriceStreamError::Json)?;
			let frame = serde_json::from_value(json).map_err(PriceStreamError::Payload)?;

			return Ok(Some(PriceEvent {
				event: name,
				id: self.last_id.clone(),
				retry: self.retry,
				frame,
			}));
		}

		let (key, value) = line.split_once(':').unwrap_or((line, ""));
		let value = value.strip_prefix(' ').unwrap_or(value);
		match key {
			"data" => self.data.push(value.to_owned()),
			"event" => self.name = (!value.is_empty()).then(|| value.to_owned()),
			"id" if !value.contains('\0') => self.last_id = Some(value.to_owned()),
			"retry" if !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit()) => {
				let milliseconds =
					value.parse::<u64>().map_err(|_| PriceStreamError::RetryOverflow)?;
				self.retry = Some(Duration::from_millis(milliseconds));
			},
			_ => {},
		}

		Ok(None)
	}
}

impl PriceStream {
	/// Pull one event. Clean EOF returns `None`; truncated EOF is `UnexpectedEof`.
	pub async fn next(&mut self) -> Result<Option<PriceEvent>> {
		loop {
			if let Some(event) = self.pending.pop_front() {
				return Ok(Some(event));
			}
			if self.finished {
				return Ok(None);
			}

			let chunk = tokio::time::timeout(IDLE_TIMEOUT, self.response.chunk())
				.await
				.map_err(|_| PriceStreamError::IdleTimeout)?
				.map_err(|source| PriceStreamError::Network {
					phase: PriceStreamPhase::Read,
					source,
				})?;
			match chunk {
				Some(chunk) => self.pending.extend(self.decoder.push(&chunk)?),
				None => {
					self.finished = true;
					self.decoder.finish()?;

					return Ok(None);
				},
			}
		}
	}

	pub(crate) async fn open(
		http: reqwest::Client,
		url: String,
		key: &str,
		value: &str,
	) -> Result<Self> {
		let response = tokio::time::timeout(
			CONNECT_TIMEOUT,
			http.get(url)
				.query(&[(key, value)])
				.header(reqwest::header::ACCEPT, "text/event-stream")
				.send(),
		)
		.await
		.map_err(|_| PriceStreamError::ConnectTimeout)?
		.map_err(|source| PriceStreamError::Network { phase: PriceStreamPhase::Connect, source })?;
		if !response.status().is_success() {
			let status = response.status().as_u16();
			let retry_after = crate::client::parse_retry_after(&response);
			let bytes = tokio::time::timeout(CONNECT_TIMEOUT, read_problem(response))
				.await
				.map_err(|_| PriceStreamError::ErrorBodyTimeout)??;
			let value = crate::problem::parse_json_value(&bytes)
				.map_err(|source| PriceStreamError::InvalidProblem { status, source })?;
			let problem = crate::ProblemDetails::parse(value.clone())
				.map_err(|source| PriceStreamError::InvalidProblem { status, source })?;
			problem
				.validate_http_status(status)
				.map_err(|source| PriceStreamError::InvalidProblem { status, source })?;

			return Err(AgaraError::from_response(
				status,
				value,
				retry_after,
				Some(problem),
			));
		}

		let content_type = response
			.headers()
			.get(reqwest::header::CONTENT_TYPE)
			.map(|value| value.to_str())
			.transpose()
			.map_err(PriceStreamError::ContentTypeEncoding)?;
		if !content_type.is_some_and(|value| {
			value
				.split(';')
				.next()
				.is_some_and(|media| media.trim().eq_ignore_ascii_case("text/event-stream"))
		}) {
			return Err(PriceStreamError::invalid_content_type(
				content_type.map(|value| Cow::Owned(value.to_owned())),
			)
			.into());
		}

		Ok(Self {
			response,
			decoder: SseDecoder::default(),
			pending: VecDeque::new(),
			finished: false,
		})
	}
}

impl<A> AgaraClient<A> {
	/// Subscribe to one validated provider symbol, such as `Crypto.BTC/USD`.
	pub async fn stream_price(&self, symbol: &str) -> Result<PriceStream> {
		validate_symbols(&[symbol])?;

		PriceStream::open(
			self.stream_transport(),
			self.url("/api/v1/prices/pyth-pro/stream"),
			"symbol",
			symbol,
		)
		.await
	}

	/// Subscribe to 1..=64 provider symbols. No mutation or automatic retry is performed.
	pub async fn stream_prices(&self, symbols: &[&str]) -> Result<PriceStream> {
		validate_symbols(symbols)?;

		PriceStream::open(
			self.stream_transport(),
			self.url("/api/v1/prices/stream"),
			"symbols",
			&symbols.join(","),
		)
		.await
	}
}

impl fmt::Display for PriceMantissa {
	fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.as_str().fmt(formatter)
	}
}

impl fmt::Display for ProviderSymbol {
	fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.as_str().fmt(formatter)
	}
}

impl<'de> serde::Deserialize<'de> for PriceMantissa {
	fn deserialize<D: serde::Deserializer<'de>>(
		deserializer: D,
	) -> core::result::Result<Self, D::Error> {
		let value = <String as serde::Deserialize>::deserialize(deserializer)?;

		Self::new(value).map_err(serde::de::Error::custom)
	}
}

impl<'de> serde::Deserialize<'de> for ProviderSymbol {
	fn deserialize<D: serde::Deserializer<'de>>(
		deserializer: D,
	) -> core::result::Result<Self, D::Error> {
		let value = <String as serde::Deserialize>::deserialize(deserializer)?;

		Self::new(value).map_err(serde::de::Error::custom)
	}
}

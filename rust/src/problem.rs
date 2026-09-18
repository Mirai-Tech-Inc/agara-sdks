//! Validated server failure contracts; unknown additions remain inspectable and inert.

mod codes;
mod decode;
mod error;
mod recovery;
mod registry;
mod tests;

pub use codes::{FieldErrorCode, FutureCode, KnownFieldErrorCode, KnownProblemCode, ProblemCode};
pub use error::{ContractMember, JsonKind, ProblemDecodeError, ProblemField};
pub use recovery::{Recovery, RecoveryBatchHash, RecoveryResource, RecoveryStrategy, ResourceKind};

use std::collections::BTreeMap;

use serde::{Deserialize, Deserializer};
use serde_json::Value;

/// Maximum encoded size accepted by the bounded problem parsers.
pub const MAX_PROBLEM_BYTES: usize = 65_536;

/// The HTTP producer that emitted a typed failure envelope.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProblemRepresentation {
	/// Origin responses require a canonical request ID and may include field errors.
	OriginHttp,

	/// Static edge responses contain neither origin request IDs nor field errors.
	EdgeHttp,
}

/// Registered socket recovery action, separate from a failure's HTTP recovery strategy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WebSocketAction {
	/// No automatic socket operation is allowed.
	None,

	/// Re-establish the affected subscription.
	Resubscribe,

	/// Reconnect only under the stream client's configured recovery policy.
	Reconnect,
}

/// Protocol-neutral public failure embedded in orders, batches and stream messages.
#[derive(Clone, Debug, PartialEq)]
pub struct PublicFailure {
	/// Known machine classification or a bounded future code.
	pub code: ProblemCode,

	/// Canonical safe summary for registered codes.
	pub title: String,

	/// Canonical safe detail; absence is meaningful for registered codes.
	pub detail: Option<String>,

	/// Validated recovery guidance; unknown codes always carry inert guidance.
	pub recovery: Recovery,

	/// Future top-level properties retained only when the code itself is unrecognized.
	pub extensions: BTreeMap<String, Value>,
}

/// Validated origin or static-edge HTTP Problem Details.
#[derive(Clone, Debug, PartialEq)]
pub struct ProblemDetails {
	/// URI deterministically derived from the wire code.
	pub type_uri: String,

	/// Canonical safe summary for registered codes.
	pub title: String,

	/// Declared HTTP status; compare with the actual status using `validate_http_status`.
	pub status: u16,

	/// Registered classification or a bounded future code.
	pub code: ProblemCode,

	/// Exact safe detail registered for a known code.
	pub detail: Option<String>,

	/// Canonical request UUID, required for origin responses and forbidden at the static edge.
	pub request_id: Option<uuid::Uuid>,

	/// Validated recovery guidance with unknown codes made inert.
	pub recovery: Recovery,

	/// Bounded request-field failures, preserving path and typed classification.
	pub field_errors: Vec<FieldError>,

	/// Producer representation validated when parsing this body.
	pub representation: ProblemRepresentation,

	/// Future top-level properties retained only for unknown codes.
	pub extensions: BTreeMap<String, Value>,
}

/// A request-field failure with independently bounded code, message and path.
#[derive(Clone, Debug, PartialEq)]
pub struct FieldError {
	/// Object-field names and zero-based array indices locating the invalid value.
	pub path: Vec<FieldPathSegment>,

	/// Known field-error classification or a bounded future code.
	pub code: FieldErrorCode,

	/// Bounded public explanation; never use this string as a machine discriminator.
	pub message: String,
}

/// One bounded component of a request-field validation path.
#[derive(Clone, Debug, PartialEq)]
pub enum FieldPathSegment {
	/// An object member name of at most sixty-four Unicode scalar values.
	Field(String),

	/// A zero-based index no greater than signed 32-bit maximum.
	Index(u32),
}

/// Parse at most 64 KiB of JSON, rejecting duplicate properties before a typed failure is decoded.
pub fn parse_json_value(bytes: &[u8]) -> Result<Value, ProblemDecodeError> {
	decode::json(bytes)
}

#[cfg(feature = "streaming")]
pub(crate) fn parse_json_value_with_limit(
	bytes: &[u8],
	limit: usize,
) -> Result<Value, ProblemDecodeError> {
	decode::json_with_limit(bytes, limit)
}

impl PublicFailure {
	/// Parse a current or future protocol-neutral failure, rejecting malformed current shapes.
	pub fn parse(value: Value) -> Result<Self, ProblemDecodeError> {
		decode::public_failure(value)
	}

	/// Parse bounded JSON while retaining the actual serde error on malformed bytes.
	pub fn parse_json(bytes: &[u8]) -> Result<Self, ProblemDecodeError> {
		Self::parse(decode::json(bytes)?)
	}

	/// Whether a registered code and every current metadata field remain canonical.
	pub fn is_known(&self) -> bool {
		self.code.known().is_some()
			&& decode::validate_failure(
				&self.code,
				&self.title,
				self.detail.as_deref(),
				&self.recovery,
			)
			.is_ok() && self.extensions.is_empty()
	}

	/// Registered socket action; future codes and codes not emitted over WebSockets return None.
	pub fn websocket_action(&self) -> Option<WebSocketAction> {
		if !self.is_known() {
			return None;
		}

		self.code.known().and_then(|code| registry::metadata(code).websocket)
	}
}

impl ProblemDetails {
	/// Parse an origin envelope when request_id exists, otherwise a static-edge envelope.
	pub fn parse(value: Value) -> Result<Self, ProblemDecodeError> {
		let representation = if value.get("request_id").is_some() {
			ProblemRepresentation::OriginHttp
		} else {
			ProblemRepresentation::EdgeHttp
		};

		Self::parse_as(value, representation)
	}

	/// Parse a known producer representation without allowing an edge body to claim origin fields.
	pub fn parse_as(
		value: Value,
		representation: ProblemRepresentation,
	) -> Result<Self, ProblemDecodeError> {
		decode::problem(value, representation)
	}

	/// Parse a bounded HTTP failure body, preserving its JSON parser error on failure.
	pub fn parse_json(bytes: &[u8]) -> Result<Self, ProblemDecodeError> {
		Self::parse(decode::json(bytes)?)
	}

	/// Reject an HTTP response whose real status differs from the body's claimed status.
	pub fn validate_http_status(&self, actual: u16) -> Result<(), ProblemDecodeError> {
		if actual != self.status {
			return Err(ProblemDecodeError::HttpStatusMismatch { actual, declared: self.status });
		}

		Ok(())
	}

	/// Safe display summary, preferring canonical detail when present.
	pub fn message(&self) -> &str {
		self.detail.as_deref().unwrap_or(&self.title)
	}

	/// Revalidate bounded fields and registered metadata after any caller mutation.
	pub fn validate(&self) -> Result<(), ProblemDecodeError> {
		decode::validate_problem(self)
	}

	/// Explicit retry permission from a current code with intact canonical metadata.
	pub fn is_retryable(&self) -> bool {
		decode::validate_problem(self).is_ok()
			&& self.code.known().is_some()
			&& self.recovery.is_retryable()
	}
}

impl<'de> Deserialize<'de> for PublicFailure {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		Self::parse(decode::deserialize_value(deserializer)?).map_err(serde::de::Error::custom)
	}
}

impl<'de> Deserialize<'de> for ProblemDetails {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		Self::parse(decode::deserialize_value(deserializer)?).map_err(serde::de::Error::custom)
	}
}

impl<'de> Deserialize<'de> for FieldError {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		decode::field_error(decode::deserialize_value(deserializer)?)
			.map_err(serde::de::Error::custom)
	}
}

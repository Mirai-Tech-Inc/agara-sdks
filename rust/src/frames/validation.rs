use core::{fmt, num::NonZeroU32};

use std::sync::Arc;

use serde_json::Value;

const MAX_CONTEXT_CHARS: usize = 256;

/// A specific member of a WebSocket envelope or payload.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameField {
	/// Envelope operation discriminator.
	Operation,

	/// Channel discriminator.
	Channel,

	/// Update payload object.
	Data,

	/// Update payload kind discriminator.
	Kind,

	/// Server-requested recovery action.
	Action,

	/// Nested canonical public failure.
	Failure,

	/// Subscription list entries.
	Channels,

	/// Outcome-token subject.
	TokenId,

	/// Condition subject.
	ConditionId,

	/// Event UUID subject.
	EventId,

	/// Engine sequence, absent on event lifecycle messages.
	Sequence,

	/// Reset diagnostic reason.
	ResetReason,

	/// Optional batch provenance hash.
	BatchHash,

	/// Winning native quantity redeemed.
	WinningSharesRedeemed,

	/// Losing native quantity removed.
	LosingSharesZeroed,

	/// Collateral paid out by redemption.
	PayoutMicro,

	/// Collateral moved by an account operation.
	AmountMicro,

	/// Remaining collateral balance.
	CashBalanceMicro,

	/// Fee charged for a fill.
	FeeMicro,
}

/// A field's structural or scalar violation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameFieldReason {
	/// Required field was absent.
	Missing,

	/// A JSON string was required.
	ExpectedString,

	/// A JSON object was required.
	ExpectedObject,

	/// A JSON array was required.
	ExpectedArray,

	/// A native nonnegative JSON integer was required.
	ExpectedUnsignedInteger,

	/// The string was empty.
	Empty,

	/// Unicode scalar length exceeded the wire limit.
	TooLong {
		/// Maximum accepted scalar count.
		limit: usize,

		/// Observed scalar count.
		actual: usize,
	},

	/// A current closed discriminator did not name an admitted variant.
	UnknownVariant,

	/// An unsigned value was negative.
	Negative,

	/// This member is forbidden for this frame variant.
	Unexpected,
}

/// Why the channel and subject scope cannot describe a valid current subscription.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SubjectViolation {
	/// More than one subject identifier was present.
	MultipleIdentifiers,

	/// The channel requires an identifier, but none was supplied.
	MissingIdentifier,

	/// The identifier kind does not belong to the channel.
	WrongIdentifier,

	/// The current channel discriminator is not recognized.
	UnknownChannel,

	/// Event lifecycle kinds and condition lifecycle kinds used the opposite scope.
	LifecycleScope,
}

/// A cross-field invariant violated by otherwise representable data.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameInvariant {
	/// Batch hash and batch index must either both be present or both absent.
	BatchProvenance,

	/// Minimum price cannot exceed maximum price.
	PriceRange,

	/// Remaining order quantity cannot exceed its original quantity.
	RemainingQuantity,

	/// A market must contain at least two outcomes.
	OutcomeCount,

	/// Outcome kind text cannot be empty.
	OutcomeKind,

	/// A reference price must use finite decimal notation.
	ReferencePrice,

	/// The current market ID must equal the event's main market ID.
	CurrentMarketIdentity,

	/// A current market must be active.
	CurrentMarketState,

	/// Resolution replacement status, market and expected-cycle fields disagree.
	ResolutionReplacement,

	/// Tick size must be strictly positive.
	PositiveTick,
}

/// A positive engine scale; zero cannot be constructed or deserialized.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(transparent)]
pub struct Scale(NonZeroU32);

/// An exact nonnegative 64-bit amount encoded as decimal text on the stream wire.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EncodedAmount(u64);

/// A recognized WebSocket frame violated its wire contract.
#[derive(Debug, Clone, thiserror::Error)]
#[non_exhaustive]
pub enum FrameDecodeError {
	/// The top-level value was not a frame object with an operation discriminator.
	#[error("invalid WebSocket envelope")]
	Envelope,

	/// A specific member failed structural or scalar validation.
	#[error("invalid stream field {field:?}: {reason:?}")]
	Field {
		/// Failed member.
		field: FrameField,

		/// Machine-matchable constraint violation.
		reason: FrameFieldReason,
	},

	/// The channel and subject identifiers cannot form a valid current scope.
	#[error("invalid stream subject: {0:?}")]
	Subject(SubjectViolation),

	/// Resource parsing failed with its original typed validation source retained.
	#[error("invalid stream identifier {field:?}: {source}")]
	Identifier {
		/// Identifier member being decoded.
		field: FrameField,

		/// Original resource-domain validation failure.
		#[source]
		source: Arc<crate::validation::ValidationError>,
	},

	/// Representable fields contradicted a published payload invariant.
	#[error("invalid stream payload invariant: {0:?}")]
	Invariant(FrameInvariant),

	/// A payload repeated an envelope identity with a different value.
	#[error("payload conflicts with envelope {field:?}")]
	IdentityConflict {
		/// Contradictory identity member.
		field: FrameField,
	},

	/// A text frame contained malformed, duplicate-key or oversized JSON.
	#[error("invalid WebSocket JSON: {0}")]
	Json(#[source] Arc<crate::problem::ProblemDecodeError>),

	/// Nested public failure metadata was not trustworthy.
	#[error("invalid stream public failure: {0}")]
	Problem(#[source] Arc<crate::problem::ProblemDecodeError>),

	/// Serde rejected a known payload's field types; the parser error remains available.
	#[error("invalid known stream payload: {0}")]
	Payload(#[source] Arc<serde_json::Error>),

	/// A zero scale cannot convert native quantities safely.
	#[error("engine scale must be positive")]
	ZeroScale,

	/// This endpoint supports JSON text plus WebSocket control frames, not binary payloads.
	#[error("unsupported non-text WebSocket message")]
	UnsupportedMessage,
}

pub(super) fn text(raw: &Value, field: FrameField) -> Result<&str, FrameDecodeError> {
	let value = raw
		.get(field.as_str())
		.ok_or(FrameDecodeError::Field { field, reason: FrameFieldReason::Missing })?;
	let text = value
		.as_str()
		.ok_or(FrameDecodeError::Field { field, reason: FrameFieldReason::ExpectedString })?;
	if text.is_empty() {
		return Err(FrameDecodeError::Field { field, reason: FrameFieldReason::Empty });
	}

	Ok(text)
}

pub(super) fn validate_subject(raw: &Value, required: bool) -> Result<(), FrameDecodeError> {
	let channel = match raw.get("channel") {
		None | Some(Value::Null) if !required => None,
		_ => Some(text(raw, FrameField::Channel)?),
	};
	let provided: Vec<_> = [
		FrameField::TokenId,
		FrameField::ConditionId,
		FrameField::EventId,
	]
	.into_iter()
	.filter(|field| raw.get(field.as_str()).is_some_and(|value| !value.is_null()))
	.collect();

	for field in &provided {
		let value = text(raw, *field)?;
		let actual = value.chars().count();
		if actual > MAX_CONTEXT_CHARS {
			return Err(FrameDecodeError::Field {
				field: *field,
				reason: FrameFieldReason::TooLong { limit: MAX_CONTEXT_CHARS, actual },
			});
		}
		if required && *field == FrameField::EventId {
			crate::ids::EventId::new(value).map_err(|source| FrameDecodeError::Identifier {
				field: *field,
				source: Arc::new(source),
			})?;
		}
	}
	if !required {
		if channel.is_some_and(|channel| {
			![
				"orderbook",
				"best_quote",
				"trades",
				"market_status",
				"account_events",
			]
			.contains(&channel)
		}) {
			return Err(FrameDecodeError::Subject(SubjectViolation::UnknownChannel));
		}

		return Ok(());
	}
	if provided.len() > 1 {
		return Err(FrameDecodeError::Subject(
			SubjectViolation::MultipleIdentifiers,
		));
	}
	if required && provided.is_empty() && channel != Some("account_events") {
		return Err(FrameDecodeError::Subject(
			SubjectViolation::MissingIdentifier,
		));
	}

	let valid = match channel {
		Some("orderbook" | "best_quote") => {
			provided.as_slice() == [FrameField::TokenId] || !required && provided.is_empty()
		},
		Some("trades") => {
			provided.as_slice() == [FrameField::ConditionId] || !required && provided.is_empty()
		},
		Some("market_status") => {
			provided.as_slice() == [FrameField::ConditionId]
				|| provided.as_slice() == [FrameField::EventId]
				|| !required && provided.is_empty()
		},
		Some("account_events") => provided.is_empty(),
		None => !required,
		Some(_) => return Err(FrameDecodeError::Subject(SubjectViolation::UnknownChannel)),
	};
	if !valid {
		return Err(FrameDecodeError::Subject(SubjectViolation::WrongIdentifier));
	}

	Ok(())
}

pub(super) fn validate_payload(payload: &Value) -> Result<(), FrameDecodeError> {
	let object = payload.as_object().ok_or(FrameDecodeError::Field {
		field: FrameField::Data,
		reason: FrameFieldReason::ExpectedObject,
	})?;
	if object.get("batch_hash").is_some_and(|value| !value.is_null())
		!= object.get("batch_index").is_some_and(|value| !value.is_null())
	{
		return Err(FrameDecodeError::Invariant(FrameInvariant::BatchProvenance));
	}
	if let Some(hash) = object.get("batch_hash").and_then(Value::as_str) {
		crate::ids::BatchHash::new(hash).map_err(|source| FrameDecodeError::Identifier {
			field: FrameField::BatchHash,
			source: Arc::new(source),
		})?;
	}
	if let (Some(min), Some(max)) = (
		object.get("min_price").and_then(Value::as_u64),
		object.get("max_price").and_then(Value::as_u64),
	) && min > max
	{
		return Err(FrameDecodeError::Invariant(FrameInvariant::PriceRange));
	}
	if let (Some(remaining), Some(original)) = (
		object.get("remaining_size").and_then(Value::as_u64),
		object.get("original_size").and_then(Value::as_u64),
	) && remaining > original
	{
		return Err(FrameDecodeError::Invariant(
			FrameInvariant::RemainingQuantity,
		));
	}
	for field in [
		FrameField::WinningSharesRedeemed,
		FrameField::LosingSharesZeroed,
		FrameField::PayoutMicro,
		FrameField::AmountMicro,
		FrameField::CashBalanceMicro,
		FrameField::FeeMicro,
	] {
		if let Some(value) = object.get(field.as_str())
			&& (value.as_str().is_some_and(|value| value.starts_with('-'))
				|| value.as_i64().is_some_and(|value| value < 0))
		{
			return Err(FrameDecodeError::Field { field, reason: FrameFieldReason::Negative });
		}
	}

	Ok(())
}

pub(super) fn has_future_enum(payload: &Value) -> bool {
	let allowed: [(&str, &[&str]); 4] = [
		("side", &["BUY", "SELL", "UNSPECIFIED"]),
		("role", &["MAKER", "TAKER"]),
		("tif", &["GTC", "FAK", "FOK", "UNSPECIFIED"]),
		(
			"settlement_mode",
			&["NORMAL", "MINT", "MERGE", "UNSPECIFIED"],
		),
	];

	allowed.iter().any(|(field, values)| {
		payload
			.get(field)
			.and_then(Value::as_str)
			.is_some_and(|value| !value.is_empty() && !values.contains(&value))
	}) || payload.get("replacement_status").and_then(Value::as_str).is_some_and(|value| {
		!value.is_empty() && !["active", "upcoming", "expected", "none"].contains(&value)
	}) || ["market", "replacement"].iter().any(|field| {
		payload
			.get(field)
			.and_then(|market| market.get("state"))
			.and_then(Value::as_str)
			.is_some_and(|state| {
				!state.is_empty()
					&& ![
						"DRAFT",
						"PROVISIONED",
						"ACTIVE",
						"INACTIVE",
						"TRADING_HALT",
						"PROPOSAL_PENDING",
						"PROPOSED",
						"DISPUTED",
						"RESOLVED",
						"VOID",
						"ARCHIVED",
					]
					.contains(&state)
			})
	})
}

pub(super) fn subject(raw: &Value) -> Result<super::Channel, FrameDecodeError> {
	validate_subject(raw, true)?;
	let identifier_error =
		|field, source| FrameDecodeError::Identifier { field, source: Arc::new(source) };

	match text(raw, FrameField::Channel)? {
		"orderbook" => crate::ids::TokenId::new(text(raw, FrameField::TokenId)?)
			.map(super::Channel::Orderbook)
			.map_err(|source| identifier_error(FrameField::TokenId, source)),
		"best_quote" => crate::ids::TokenId::new(text(raw, FrameField::TokenId)?)
			.map(super::Channel::BestQuote)
			.map_err(|source| identifier_error(FrameField::TokenId, source)),
		"trades" => crate::ids::ConditionId::new(text(raw, FrameField::ConditionId)?)
			.map(super::Channel::Trades)
			.map_err(|source| identifier_error(FrameField::ConditionId, source)),
		"market_status" if raw.get("event_id").is_some_and(|value| !value.is_null()) => {
			crate::ids::EventId::new(text(raw, FrameField::EventId)?)
				.map(super::Channel::EventMarketStatus)
				.map_err(|source| identifier_error(FrameField::EventId, source))
		},
		"market_status" => crate::ids::ConditionId::new(text(raw, FrameField::ConditionId)?)
			.map(super::Channel::MarketStatus)
			.map_err(|source| identifier_error(FrameField::ConditionId, source)),
		"account_events" => Ok(super::Channel::AccountEvents),
		_ => Err(FrameDecodeError::Subject(SubjectViolation::UnknownChannel)),
	}
}

pub(super) fn required_nullable<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
	D: serde::Deserializer<'de>,
	T: serde::Deserialize<'de>,
{
	<Option<T> as serde::Deserialize>::deserialize(deserializer)
}

pub(super) fn decimal_text(value: &str) -> bool {
	let value = value.strip_prefix('-').unwrap_or(value);
	let (whole, fraction) =
		value.split_once('.').map_or((value, None), |(whole, fraction)| (whole, Some(fraction)));

	!whole.is_empty()
		&& whole.bytes().all(|byte| byte.is_ascii_digit())
		&& fraction.is_none_or(|fraction| {
			!fraction.is_empty() && fraction.bytes().all(|byte| byte.is_ascii_digit())
		})
}

fn wire_enum<'de, D, T>(deserializer: D, allowed: &[&str]) -> Result<T, D::Error>
where
	D: serde::Deserializer<'de>,
	T: serde::de::DeserializeOwned,
{
	let value = <String as serde::Deserialize>::deserialize(deserializer)?;
	if !allowed.contains(&value.as_str()) {
		return Err(serde::de::Error::custom("unsupported wire enum value"));
	}

	serde_json::from_value(Value::String(value)).map_err(serde::de::Error::custom)
}

pub(super) fn side<'de, D: serde::Deserializer<'de>>(
	deserializer: D,
) -> Result<crate::ids::Side, D::Error> {
	wire_enum(deserializer, &["BUY", "SELL", "UNSPECIFIED"])
}
pub(super) fn role<'de, D: serde::Deserializer<'de>>(
	deserializer: D,
) -> Result<crate::ids::FillRole, D::Error> {
	wire_enum(deserializer, &["MAKER", "TAKER"])
}
pub(super) fn tif<'de, D: serde::Deserializer<'de>>(
	deserializer: D,
) -> Result<crate::ids::TimeInForce, D::Error> {
	wire_enum(deserializer, &["GTC", "FAK", "FOK", "UNSPECIFIED"])
}
pub(super) fn settlement_mode<'de, D: serde::Deserializer<'de>>(
	deserializer: D,
) -> Result<crate::ids::SettlementMode, D::Error> {
	wire_enum(deserializer, &["NORMAL", "MINT", "MERGE", "UNSPECIFIED"])
}

impl FrameField {
	/// Member name in its containing JSON object.
	pub const fn as_str(self) -> &'static str {
		match self {
			Self::Operation => "op",
			Self::Channel => "channel",
			Self::Data => "data",
			Self::Kind => "kind",
			Self::Action => "action",
			Self::Failure => "failure",
			Self::Channels => "channels",
			Self::TokenId => "token_id",
			Self::ConditionId => "condition_id",
			Self::EventId => "event_id",
			Self::Sequence => "sequence",
			Self::ResetReason => "reason",
			Self::BatchHash => "batch_hash",
			Self::WinningSharesRedeemed => "winning_shares_redeemed",
			Self::LosingSharesZeroed => "losing_shares_zeroed",
			Self::PayoutMicro => "payout_micro",
			Self::AmountMicro => "amount_micro",
			Self::CashBalanceMicro => "cash_balance_micro",
			Self::FeeMicro => "fee_micro",
		}
	}
}

impl Scale {
	/// Construct a positive scale, returning `ZeroScale` for zero.
	pub fn new(value: u32) -> Result<Self, FrameDecodeError> {
		NonZeroU32::new(value).map(Self).ok_or(FrameDecodeError::ZeroScale)
	}

	/// The positive scale used to convert native integers to units.
	pub const fn raw(self) -> u32 {
		self.0.get()
	}
}

impl EncodedAmount {
	/// Construct an amount from its exact unsigned magnitude.
	pub const fn new(value: u64) -> Self {
		Self(value)
	}

	/// Exact magnitude; consult the field's scale to convert to display units.
	pub const fn raw(self) -> u64 {
		self.0
	}
}

impl fmt::Display for Scale {
	fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.raw().fmt(formatter)
	}
}

impl<'de> serde::Deserialize<'de> for Scale {
	fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		let value = <u32 as serde::Deserialize>::deserialize(deserializer)?;

		Self::new(value).map_err(serde::de::Error::custom)
	}
}

impl<'de> serde::Deserialize<'de> for EncodedAmount {
	fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		let value = <String as serde::Deserialize>::deserialize(deserializer)?;
		if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
			return Err(serde::de::Error::custom(
				"expected unsigned decimal amount text",
			));
		}

		value.parse::<u64>().map(Self).map_err(serde::de::Error::custom)
	}
}

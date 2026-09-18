use std::borrow::Cow;

use super::Field;

/// The identifier invariant violated by a local input.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum IdentifierReason {
	/// Empty identifiers cannot select a resource.
	#[error("value must not be empty")]
	Empty,

	/// The identifier exceeds the endpoint's size bound.
	#[error("length {actual} exceeds maximum {max}")]
	TooLong {
		/// Maximum supported length.
		max: usize,

		/// Observed length in the validator's documented units.
		actual: usize,
	},

	/// Whitespace would change the intended identifier.
	#[error("whitespace is not permitted")]
	Whitespace,

	/// The identifier contains characters outside its defined alphabet.
	#[error("identifier contains an unsupported character")]
	InvalidCharacter,

	/// UUID syntax or canonical structure is invalid.
	#[error("expected a canonical UUID: {source}")]
	InvalidUuid {
		/// Original UUID parser error, retained for structured inspection.
		#[source]
		source: uuid::Error,
	},

	/// A nil UUID cannot identify an existing resource.
	#[error("nil UUID is not permitted")]
	NilUuid,

	/// An EVM address must contain exactly twenty bytes.
	#[error("expected a 20-byte EVM address")]
	InvalidAddress,

	/// The zero address cannot serve this role.
	#[error("zero address is not permitted")]
	ZeroAddress,

	/// A digest must contain exactly thirty-two bytes.
	#[error("expected a 32-byte digest")]
	InvalidDigest,

	/// An outcome token identifier must fit its unsigned chain representation.
	#[error("expected a supported unsigned token identifier")]
	InvalidTokenId,
}

/// Why an amount cannot be converted or submitted exactly.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum AmountReason {
	/// Conversion would discard a fraction of a micro unit.
	#[error("amount has fractional micro units")]
	Precision,

	/// Checked arithmetic overflowed before conversion completed.
	#[error("amount conversion overflowed")]
	Overflow,

	/// A submitted amount must be strictly positive.
	#[error("amount must be positive")]
	NonPositive,

	/// The converted amount is outside the wire representation's range.
	#[error("amount is outside the supported range")]
	OutOfRange,

	/// Integer text failed parsing; the original parser error is retained.
	#[error("invalid integer amount: {0}")]
	IntegerSyntax(#[source] core::num::ParseIntError),

	/// Decimal text failed parsing; the original decimal parser error is retained.
	#[error("invalid decimal amount: {0}")]
	DecimalSyntax(#[source] rust_decimal::Error),

	/// Integer text uses a noncanonical sign, leading zero, or fractional representation.
	#[error("amount must use canonical integer text")]
	NonCanonicalInteger,
}

/// A documented query constraint violated before network I/O.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum QueryReason {
	/// The value is not a finite number in the required representation.
	#[error("expected a valid numeric parameter")]
	InvalidNumber,

	/// The endpoint requires this parameter.
	#[error("parameter is required")]
	Required,

	/// A request value is outside the endpoint's closed vocabulary.
	#[error("unsupported query value {value}")]
	UnsupportedValue {
		/// Supplied non-sensitive query value, retained only for diagnostics.
		value: Cow<'static, str>,
	},

	/// A numeric parameter falls outside its inclusive bounds.
	#[error("value {actual} is outside [{min}, {max}]")]
	OutOfRange {
		/// Inclusive lower bound.
		min: i128,

		/// Inclusive upper bound.
		max: i128,

		/// Supplied numeric value.
		actual: i128,
	},

	/// A query string exceeds its scalar-count or byte-count limit.
	#[error("length {actual} exceeds maximum {max}")]
	TooLong {
		/// Maximum supported length.
		max: usize,

		/// Observed length in the validator's documented units.
		actual: usize,
	},

	/// A value is not a real calendar date in the required representation.
	#[error("expected a real calendar date")]
	InvalidDate {
		/// Original calendar parser error; absent when canonical text shape failed first.
		#[source]
		source: Option<chrono::ParseError>,
	},

	/// A value is not a valid timestamp in the required representation.
	#[error("expected a valid timestamp: {source}")]
	InvalidTimestamp {
		/// Original RFC3339 parser error, retained without converting it to text.
		#[source]
		source: chrono::ParseError,
	},

	/// A range starts after it ends.
	#[error("range start must not follow range end")]
	ReversedRange,

	/// The requested range exceeds the endpoint's documented maximum span.
	#[error("range exceeds maximum span {max}")]
	RangeTooWide {
		/// Maximum span in the endpoint's documented units.
		max: u64,
	},

	/// Two individually valid query fields form an unsupported combination.
	#[error("parameter is incompatible with {other:?}")]
	IncompatibleFields {
		/// The other parameter involved in the constraint.
		other: Field,
	},
}

/// An unsigned order-shape or time-in-force constraint.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum OrderInputReason {
	/// Unix expiration seconds cannot precede the epoch.
	#[error("order expiration cannot be negative")]
	NegativeExpiration,

	/// The selected side and order kind require this field.
	#[error("field is required for this order")]
	Required,

	/// The selected side and order kind prohibit this field.
	#[error("field is forbidden for this order")]
	Forbidden,

	/// An unknown enum value cannot be sent to a mutation endpoint.
	#[error("unsupported order field value")]
	Unsupported,

	/// Order size or budget must be strictly positive.
	#[error("order amount must be positive")]
	NonPositive,

	/// A limit price must be within the open unit interval.
	#[error("limit price must be strictly between zero and one")]
	InvalidPrice,

	/// A post-only order requires a resting time-in-force policy.
	#[error("post-only requires GTC or GTD")]
	PostOnlyTimeInForce,
}

impl QueryReason {
	/// Preserve an unsupported query value while keeping the reason machine-matchable.
	pub fn unsupported_value<E>(value: E) -> Self
	where
		Cow<'static, str>: From<E>,
	{
		Self::UnsupportedValue { value: Cow::from(value) }
	}
}

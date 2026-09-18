use std::borrow::Cow;

use super::KnownProblemCode;

/// Contract member whose type, syntax, or size failed validation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProblemField {
	/// Entire HTTP or protocol-neutral failure body.
	Body,

	/// Machine-readable problem code.
	Code,

	/// Canonical problem type URI.
	TypeUri,

	/// Public summary text.
	Title,

	/// Optional safe explanatory text.
	Detail,

	/// HTTP response status in the body.
	Status,

	/// Canonical origin request UUID.
	RequestId,

	/// Recovery strategy object.
	Recovery,

	/// Recovery discriminator.
	RecoveryStrategy,

	/// Existing resource that must be reconciled.
	RecoveryResource,

	/// Resource kind discriminator.
	ResourceKind,

	/// Referenced order UUID.
	OrderId,

	/// Referenced batch digest.
	BatchHash,

	/// Referenced batch-group UUID.
	GroupId,

	/// Delta-seconds before an allowed retry.
	AfterSeconds,

	/// Bounded list of validation failures.
	FieldErrors,

	/// One field-error object.
	FieldError,

	/// Validation-failure path.
	FieldPath,

	/// One object-field name or array index.
	FieldPathSegment,

	/// Field-error classification.
	FieldErrorCode,

	/// Safe field-error explanation.
	FieldErrorMessage,
}

/// Expected JSON kind when a contract member has the wrong representation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JsonKind {
	/// A JSON object with named members.
	Object,

	/// A JSON string, not a coerced scalar.
	String,

	/// A nonnegative JSON integer, not a floating-point number.
	UnsignedInteger,

	/// A JSON array.
	Array,
}

/// Registered metadata member that disagrees with a known code.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContractMember {
	/// Public title differs from the registry.
	Title,

	/// Public detail differs, including missing or unexpected detail.
	Detail,

	/// HTTP status differs from the registered status.
	Status,

	/// Registered code is not admitted in this protocol representation.
	Representation,

	/// Recovery strategy differs from the registered template.
	Recovery,

	/// Check-status resource has the wrong registered kind.
	ResourceKind,
}

/// Exact reason a typed server failure could not be trusted.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ProblemDecodeError {
	/// The original JSON parser rejected the response bytes.
	#[error("malformed problem JSON: {source}")]
	Json {
		/// Original syntax, data or I/O classification with line and column.
		#[source]
		source: serde_json::Error,
	},

	/// A required member is absent.
	#[error("problem is missing {field:?}")]
	MissingField {
		/// Required contract member.
		field: ProblemField,
	},

	/// A member used an incompatible JSON kind.
	#[error("{field:?} must be {expected:?}")]
	WrongType {
		/// Contract member with the invalid representation.
		field: ProblemField,

		/// Required JSON kind.
		expected: JsonKind,
	},

	/// A contract string must not be empty.
	#[error("{field:?} must not be empty")]
	EmptyText {
		/// Bounded string member that was empty.
		field: ProblemField,
	},

	/// A collection or scalar-count bound was exceeded.
	#[error("{field:?} length {actual} exceeds {limit}")]
	LimitExceeded {
		/// Contract member whose bound was exceeded.
		field: ProblemField,

		/// Maximum permitted size in the member's documented units.
		limit: usize,

		/// Observed size.
		actual: usize,
	},

	/// A bounded code did not use the required lower-snake-case alphabet.
	#[error("invalid machine code in {field:?}")]
	InvalidCode {
		/// Code or recovery discriminator that failed validation.
		field: ProblemField,
	},

	/// The type URI is not the URI deterministically derived from the wire code.
	#[error("problem type URI disagrees with its code")]
	InvalidUrn,

	/// UUID parsing failed, preserving the UUID parser's actual error.
	#[error("invalid UUID in {field:?}: {source}")]
	InvalidUuid {
		/// UUID-bearing contract member.
		field: ProblemField,

		/// Original UUID parser error.
		#[source]
		source: uuid::Error,
	},

	/// UUID text parsed but was not a canonical hyphenated RFC UUID or an admitted sentinel.
	#[error("{field:?} is not a canonical UUID")]
	NonCanonicalUuid {
		/// UUID-bearing contract member.
		field: ProblemField,
	},

	/// A recovery batch hash was not exactly thirty-two 0x-prefixed bytes.
	#[error("invalid recovery batch hash")]
	InvalidBatchHash,

	/// An integer was outside its published inclusive interval.
	#[error("{field:?} value {actual} is outside [{min}, {max}]")]
	NumberOutOfRange {
		/// Numeric member that failed validation.
		field: ProblemField,

		/// Inclusive minimum.
		min: u64,

		/// Inclusive maximum.
		max: u64,

		/// Actual unsigned wire integer.
		actual: u64,
	},

	/// A closed current shape contained an unrecognized property.
	#[error("unexpected problem property {name}")]
	UnexpectedField {
		/// Property name retained solely for diagnostics.
		name: Cow<'static, str>,
	},

	/// A current recovery kind does not name an admitted resource variant.
	#[error("unknown check-status resource kind")]
	UnknownResourceKind,

	/// A known code was combined with noncanonical metadata.
	#[error("{code} disagrees with its registered {member:?}")]
	RegistryMismatch {
		/// Registered classification that fixed the expected metadata.
		code: KnownProblemCode,

		/// Exact mismatching metadata member.
		member: ContractMember,
	},

	/// The actual HTTP status disagrees with the typed body.
	#[error("HTTP status {actual} differs from problem status {declared}")]
	HttpStatusMismatch {
		/// HTTP status actually received on the wire.
		actual: u16,

		/// Status claimed by the body.
		declared: u16,
	},
}

impl ProblemDecodeError {
	/// Retain an unexpected property name without classifying it as a known field.
	pub fn unexpected_field<E>(name: E) -> Self
	where
		Cow<'static, str>: From<E>,
	{
		Self::UnexpectedField { name: Cow::from(name) }
	}
}

use core::{fmt, num::NonZeroU64};

use serde::{Deserialize, Serialize};

use super::{BatchField, BatchValidationError, WireValueError};

const MAX_CLASSIFICATION_BYTES: usize = 64;

/// Batch lifecycle state. Unknown future values remain inspectable and never imply completion.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
#[non_exhaustive]
pub enum BatchStatus {
	/// Accepted; engine effects may still be applying.
	Pending,

	/// The chain submission phase has started.
	Submitted,

	/// Confirmed settlement and persistence completed.
	Settled,

	/// Failed; completion additionally requires an unwind timestamp.
	Failed,

	/// Failed with divergent engine/chain state requiring intervention.
	FailedDivergent,

	/// A future lifecycle value, retained without granting terminal semantics.
	Unknown(FutureBatchStatus),
}

/// Authority that signed the account batch, preserving future origin values.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
#[non_exhaustive]
pub enum BatchOrigin {
	/// The platform signed with delegated holder authorization.
	Privy,

	/// The holder submitted an externally signed batch.
	Presigned,

	/// A future origin classification.
	Unknown(FutureBatchOrigin),
}

/// A valid future lifecycle label that cannot alias any currently known batch status.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct FutureBatchStatus(String);

/// A valid future signature-origin label that cannot alias PRIVY or PRESIGNED.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct FutureBatchOrigin(String);

/// Account sequence constrained to the router's persistent nonnegative i64 range.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u64", into = "u64")]
pub struct BatchSequence(u64);

/// Positive Unix deadline in seconds, constrained to the router's persistent i64 range.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u64", into = "u64")]
pub struct BatchDeadline(NonZeroU64);

/// Exact 32-byte transaction hash, canonicalized to lowercase 0x-prefixed hexadecimal.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct TransactionHash(String);

pub(super) fn chunk_index<'de, D: serde::Deserializer<'de>>(
	deserializer: D,
) -> Result<u32, D::Error> {
	let value = i32::deserialize(deserializer)?;

	u32::try_from(value).map_err(|_| {
		serde::de::Error::invalid_value(
			serde::de::Unexpected::Signed(i64::from(value)),
			&"a nonnegative chunk index",
		)
	})
}

pub(super) fn chunk_count<'de, D: serde::Deserializer<'de>>(
	deserializer: D,
) -> Result<Option<u32>, D::Error> {
	let value = Option::<i32>::deserialize(deserializer)?;

	value
		.map(|value| {
			u32::try_from(value).map_err(|_| {
				serde::de::Error::invalid_value(
					serde::de::Unexpected::Signed(i64::from(value)),
					&"a nonnegative chunk count",
				)
			})
		})
		.transpose()
}

fn known_status(value: &str) -> Option<BatchStatus> {
	match value {
		"PENDING" => Some(BatchStatus::Pending),
		"SUBMITTED" => Some(BatchStatus::Submitted),
		"SETTLED" => Some(BatchStatus::Settled),
		"FAILED" => Some(BatchStatus::Failed),
		"FAILED_DIVERGENT" => Some(BatchStatus::FailedDivergent),
		_ => None,
	}
}

fn known_origin(value: &str) -> Option<BatchOrigin> {
	match value {
		"PRIVY" => Some(BatchOrigin::Privy),
		"PRESIGNED" => Some(BatchOrigin::Presigned),
		_ => None,
	}
}

fn validate_classification(value: &str, field: BatchField) -> Result<(), BatchValidationError> {
	if value.is_empty()
		|| value.len() > MAX_CLASSIFICATION_BYTES
		|| !value.as_bytes()[0].is_ascii_uppercase()
		|| !value
			.bytes()
			.all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_')
	{
		return Err(BatchValidationError::Field {
			field,
			source: WireValueError::ClassificationSyntax,
		});
	}

	Ok(())
}

impl BatchStatus {
	/// Parse a bounded SCREAMING_SNAKE_CASE status, preserving valid unknown values.
	pub fn new(value: impl Into<String>) -> Result<Self, BatchValidationError> {
		let value = value.into();
		validate_classification(&value, BatchField::Status)?;

		if let Some(known) = known_status(&value) {
			return Ok(known);
		}

		Ok(Self::Unknown(FutureBatchStatus(value)))
	}

	/// Borrow the exact known or future wire classification.
	pub fn as_str(&self) -> &str {
		match self {
			Self::Pending => "PENDING",
			Self::Submitted => "SUBMITTED",
			Self::Settled => "SETTLED",
			Self::Failed => "FAILED",
			Self::FailedDivergent => "FAILED_DIVERGENT",
			Self::Unknown(value) => value.as_str(),
		}
	}
}

impl BatchOrigin {
	/// Parse a bounded SCREAMING_SNAKE_CASE origin, preserving valid unknown values.
	pub fn new(value: impl Into<String>) -> Result<Self, BatchValidationError> {
		let value = value.into();
		validate_classification(&value, BatchField::Origin)?;

		if let Some(known) = known_origin(&value) {
			return Ok(known);
		}

		Ok(Self::Unknown(FutureBatchOrigin(value)))
	}

	/// Borrow the exact known or future wire origin.
	pub fn as_str(&self) -> &str {
		match self {
			Self::Privy => "PRIVY",
			Self::Presigned => "PRESIGNED",
			Self::Unknown(value) => value.as_str(),
		}
	}
}

impl FutureBatchStatus {
	/// Reject malformed labels and all currently known statuses, preventing round-trip aliases.
	pub fn new(value: impl Into<String>) -> Result<Self, BatchValidationError> {
		let value = value.into();
		validate_classification(&value, BatchField::Status)?;
		if known_status(&value).is_some() {
			return Err(BatchValidationError::Field {
				field: BatchField::Status,
				source: WireValueError::KnownClassification,
			});
		}

		Ok(Self(value))
	}

	/// Borrow the bounded, nonempty future lifecycle label.
	pub fn as_str(&self) -> &str {
		&self.0
	}
}

impl FutureBatchOrigin {
	/// Reject malformed labels and known origins, preventing unknown-to-known round-trip changes.
	pub fn new(value: impl Into<String>) -> Result<Self, BatchValidationError> {
		let value = value.into();
		validate_classification(&value, BatchField::Origin)?;
		if known_origin(&value).is_some() {
			return Err(BatchValidationError::Field {
				field: BatchField::Origin,
				source: WireValueError::KnownClassification,
			});
		}

		Ok(Self(value))
	}

	/// Borrow the bounded, nonempty future signature-origin label.
	pub fn as_str(&self) -> &str {
		&self.0
	}
}

impl BatchSequence {
	/// Reject values exceeding the router's persistent i64 representation.
	pub fn new(value: u64) -> Result<Self, BatchValidationError> {
		if value > i64::MAX as u64 {
			return Err(BatchValidationError::SequenceStorageRange);
		}

		Ok(Self(value))
	}

	/// Exact sequence, suitable for constructing the next signed submission.
	pub const fn raw(self) -> u64 {
		self.0
	}
}

impl BatchDeadline {
	/// Validate a positive persistent timestamp; this constructor permits historical deadlines.
	pub fn new(value: u64) -> Result<Self, BatchValidationError> {
		let value = NonZeroU64::new(value)
			.filter(|value| value.get() <= i64::MAX as u64)
			.ok_or(BatchValidationError::DeadlineStorageRange)?;

		Ok(Self(value))
	}

	/// Exact deadline in Unix seconds.
	pub const fn raw(self) -> u64 {
		self.0.get()
	}
}

impl TransactionHash {
	/// Validate an exact 32-byte hexadecimal transaction identity.
	pub fn new(value: impl Into<String>) -> Result<Self, BatchValidationError> {
		let value = value.into();
		super::decode_hex::<32>(&value).map_err(|source| BatchValidationError::Field {
			field: BatchField::TransactionHash,
			source,
		})?;

		Ok(Self(value.to_ascii_lowercase()))
	}

	/// Borrow the canonical transaction identity.
	pub fn as_str(&self) -> &str {
		&self.0
	}
}

impl fmt::Display for BatchStatus {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(self.as_str())
	}
}

impl fmt::Display for BatchOrigin {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(self.as_str())
	}
}

impl fmt::Display for TransactionHash {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(self.as_str())
	}
}

impl TryFrom<String> for BatchStatus {
	type Error = BatchValidationError;
	fn try_from(value: String) -> Result<Self, Self::Error> {
		Self::new(value)
	}
}

impl From<BatchStatus> for String {
	fn from(value: BatchStatus) -> Self {
		value.as_str().to_owned()
	}
}

impl TryFrom<String> for BatchOrigin {
	type Error = BatchValidationError;
	fn try_from(value: String) -> Result<Self, Self::Error> {
		Self::new(value)
	}
}

impl From<BatchOrigin> for String {
	fn from(value: BatchOrigin) -> Self {
		value.as_str().to_owned()
	}
}

impl TryFrom<u64> for BatchSequence {
	type Error = BatchValidationError;
	fn try_from(value: u64) -> Result<Self, Self::Error> {
		Self::new(value)
	}
}

impl From<BatchSequence> for u64 {
	fn from(value: BatchSequence) -> Self {
		value.raw()
	}
}

impl TryFrom<u64> for BatchDeadline {
	type Error = BatchValidationError;
	fn try_from(value: u64) -> Result<Self, Self::Error> {
		Self::new(value)
	}
}

impl From<BatchDeadline> for u64 {
	fn from(value: BatchDeadline) -> Self {
		value.raw()
	}
}

impl TryFrom<String> for TransactionHash {
	type Error = BatchValidationError;
	fn try_from(value: String) -> Result<Self, Self::Error> {
		Self::new(value)
	}
}

impl From<TransactionHash> for String {
	fn from(value: TransactionHash) -> Self {
		value.0
	}
}

impl TryFrom<String> for FutureBatchStatus {
	type Error = BatchValidationError;

	fn try_from(value: String) -> Result<Self, Self::Error> {
		Self::new(value)
	}
}

impl From<FutureBatchStatus> for String {
	fn from(value: FutureBatchStatus) -> Self {
		value.0
	}
}

impl TryFrom<String> for FutureBatchOrigin {
	type Error = BatchValidationError;

	fn try_from(value: String) -> Result<Self, Self::Error> {
		Self::new(value)
	}
}

impl From<FutureBatchOrigin> for String {
	fn from(value: FutureBatchOrigin) -> Self {
		value.0
	}
}

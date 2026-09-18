//! Account batch requests, reconciliation states, and group status.

mod state;

pub use state::{
	BatchDeadline, BatchOrigin, BatchSequence, BatchStatus, FutureBatchOrigin, FutureBatchStatus,
	TransactionHash,
};

use std::time::{SystemTime, SystemTimeError, UNIX_EPOCH};

use crate::{
	ids::{BatchGroupId, BatchHash},
	problem::PublicFailure,
	values::Timestamp,
};

/// Maximum operations accepted by the presigned batch gauntlet.
pub const MAX_BATCH_OPS: usize = 20;

/// Minimum withdrawal in micro-collateral units (one cent).
pub const MIN_WITHDRAW_MICRO: i64 = 10_000;

/// Maximum deadline horizon after the router's clock-skew allowance, in seconds.
pub const MAX_BATCH_DEADLINE_HORIZON_SECONDS: i64 = 86_340;

const UINT256_MAX_DECIMAL: &str =
	"115792089237316195423570985008687907853269984665640564039457584007913129639935";
const SIGNATURE_BYTES: usize = 65;
const SCALAR_BYTES: usize = 32;
const SIGNATURE_V_INDEX: usize = 64;
const SECP256K1_ORDER: [u8; SCALAR_BYTES] = [
	255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 254, 186, 174, 220,
	230, 175, 72, 160, 59, 191, 210, 94, 140, 208, 54, 65, 65,
];
const SECP256K1_HALF_ORDER: [u8; SCALAR_BYTES] = [
	127, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 93, 87, 110,
	115, 87, 164, 80, 29, 223, 233, 47, 70, 104, 27, 32, 160,
];

/// One presigned SPLIT, MERGE, or WITHDRAW intent; validate before composing or submitting.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BatchOpDto {
	/// Split collateral into a complete set: `shares_micro` of each leg.
	Split {
		/// Market row id used for lookup and gating.
		market_id: String,

		/// 0x-prefixed condition id the client signed over; must equal the
		/// stored condition of `market_id`.
		condition_id: String,

		/// Complete-set size in micro units; equals the collateral consumed.
		shares_micro: i64,
	},

	/// Merge a complete set back into collateral.
	Merge {
		/// Market row id used for lookup and gating.
		market_id: String,

		/// 0x-prefixed condition id the client signed over.
		condition_id: String,

		/// Complete-set size in micro units; equals the collateral produced.
		shares_micro: i64,
	},

	/// Transfer collateral out of the account.
	Withdraw {
		/// 0x-prefixed recipient; never the wallet's own account.
		destination: String,

		/// Amount in micro units.
		amount_micro: i64,
	},
}

/// Holder-signed batch submission. Validation enforces syntax, operation policy and deadline bounds.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]

pub struct AccountBatchSubmission {
	/// Typed ops in execution order.
	pub ops: Vec<BatchOpDto>,

	/// Account seq the signature binds. The live seq, or the next one above it when batches are
	/// already queued on this wallet; it may not skip a seq that no batch holds.
	pub seq: u64,

	/// Signature deadline, unix seconds.
	pub deadline_unix_seconds: u64,

	/// Holder signature over the batch digest, 0x hex.
	pub signature: String,

	/// The `FAILED_DIVERGENT` batch this batch heals, if any.
	pub heals_batch_hash: Option<String>,
}

/// Replacement operations signed at the original batch sequence; validates like a new submission.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]

pub struct AccountBatchSupersedeSubmission {
	/// Typed ops in execution order.
	pub ops: Vec<BatchOpDto>,

	/// Signature deadline, unix seconds.
	pub deadline_unix_seconds: u64,

	/// Holder signature over the successor digest (same seq), 0x hex.
	pub signature: String,
}

/// Submission identity used to reconcile acceptance and settlement.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]

pub struct AccountBatchAccepted {
	/// EIP-712 batch digest, 0x hex.
	pub batch_hash: BatchHash,
}

/// Persisted account-batch lifecycle, settlement proof and recovery state.
#[derive(Clone, Debug, serde::Deserialize)]

pub struct AccountBatchStatusDto {
	/// Validated EIP-712 batch identity.
	pub batch_hash: BatchHash,

	/// Known lifecycle state or a preserved, nonterminal future value.
	pub status: BatchStatus,

	/// Exact nonnegative sequence in the persistent i64 range.
	pub seq: BatchSequence,

	/// Positive signature deadline in Unix seconds, within persistent storage bounds.
	pub deadline_unix_seconds: BatchDeadline,

	/// Signature authority, preserving valid future classifications.
	pub origin: BatchOrigin,

	/// Validated transaction identity after confirmed settlement.
	pub tx_hash: Option<TransactionHash>,

	/// Parsed execution timestamp, when known.
	pub executed_at: Option<Timestamp>,

	/// Failure.
	pub failure: Option<PublicFailure>,

	/// Superseded by batch hash.
	pub superseded_by_batch_hash: Option<BatchHash>,

	/// Heals batch hash.
	pub heals_batch_hash: Option<BatchHash>,

	/// Parsed completion time for reversal of failed optimistic effects.
	pub unwound_at: Option<Timestamp>,

	/// Parsed row creation timestamp.
	pub created_at: Timestamp,
}

/// AccountBatchSupersedeOutcome wire contract.
#[derive(Clone, Debug, serde::Deserialize)]
#[serde(tag = "outcome", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AccountBatchSupersedeOutcome {
	/// A successor replaced the original pending batch.
	Superseded {
		/// Successor batch.
		batch: AccountBatchStatusDto,
	},

	/// The original batch had already progressed and was not replaced.
	Refused {
		/// Current original batch.
		current: AccountBatchStatusDto,
	},
}

/// One batch attempt occupying a group execution slot.
#[derive(Clone, Debug, serde::Deserialize)]

pub struct GroupChunkDto {
	/// Slot the row occupies.
	#[serde(deserialize_with = "state::chunk_index")]
	pub chunk_index: u32,

	/// EIP-712 batch digest; pollable at `GET /trade/v1/batches/{batch_hash}`.
	pub batch_hash: BatchHash,

	/// Lifecycle status, as its db value.
	pub status: BatchStatus,

	/// Confirmed transaction hash, when settled.
	pub tx_hash: Option<TransactionHash>,

	/// Safe failure classification, when failed.
	pub failure: Option<PublicFailure>,
}

/// Group completion and ordered batch attempts; incomplete groups have no completion time.
#[derive(Clone, Debug, serde::Deserialize)]

pub struct BatchGroupStatusDto {
	/// Validated UUID identifying the batch group.
	pub group_id: BatchGroupId,

	/// Ops in the stored plan (candidates before phase A confirms, the frozen
	/// confirmed subset after).
	pub op_count: usize,

	/// Chunks the confirmed plan mirrors; `null` while phase A runs.
	#[serde(default, deserialize_with = "state::chunk_count")]
	pub chunk_count: Option<u32>,

	/// Set when every chunk settled and the group closed.
	pub completed_at: Option<Timestamp>,

	/// Chunk rows, slot order; retaken slots list every attempt.
	pub chunks: Vec<GroupChunkDto>,

	/// Response timestamp.
	pub as_of: Timestamp,
}

/// A batch field whose wire representation is invalid.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BatchField {
	/// Persisted batch lifecycle classification.
	Status,

	/// Persisted signature origin classification.
	Origin,

	/// Confirmed on-chain transaction identity.
	TransactionHash,

	/// Raw call target address.
	CallTarget,

	/// Raw call native value in wei.
	CallValue,

	/// Canonical market UUID.
	MarketId,

	/// Condition identifier, encoded as 32 bytes of hexadecimal.
	ConditionId,

	/// Withdrawal recipient address.
	Destination,

	/// Holder signature over the batch digest.
	Signature,

	/// Prior divergent batch digest.
	HealsBatchHash,

	/// Account address used for self-withdrawal checks.
	Account,
}

/// Structural wire failures independent of cryptographic signing features.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum WireValueError {
	/// A future-value wrapper attempted to alias a currently known classification.
	#[error("future classification must not name a known variant")]
	KnownClassification,

	/// A classification is not bounded nonempty SCREAMING_SNAKE_CASE text.
	#[error("expected bounded SCREAMING_SNAKE_CASE classification")]
	ClassificationSyntax,

	/// A fixed-size hexadecimal value has the wrong prefix or length.
	#[error("expected 0x-prefixed hexadecimal encoding of {bytes} bytes")]
	HexLength {
		/// Required decoded byte length.
		bytes: usize,
	},

	/// A hexadecimal digit is invalid.
	#[error("invalid hexadecimal encoding")]
	HexEncoding,

	/// A decimal Uint256 is empty or contains non-decimal characters.
	#[error("expected an unsigned decimal integer")]
	DecimalSyntax,

	/// A decimal integer exceeds the Uint256 range.
	#[error("integer exceeds Uint256 range")]
	Uint256Overflow,

	/// A value whose domain requires positivity is zero.
	#[error("value must be nonzero")]
	Zero,

	/// The market identifier is not a UUID.
	#[error("market identifier is not a UUID")]
	UuidSyntax(#[source] uuid::Error),

	/// A nil UUID cannot identify a market.
	#[error("market identifier must not be nil")]
	NilUuid,

	/// ECDSA recovery parity is not 0, 1, 27, or 28.
	#[error("signature recovery byte must be 0, 1, 27, or 28")]
	SignatureParity,

	/// Orders preserve signature bytes on chain, which requires canonical Ethereum parity.
	#[error("order signature recovery byte must be 27 or 28")]
	SignatureNonCanonicalParity,

	/// ECDSA r is outside the curve's nonzero scalar range.
	#[error("signature r is outside the valid scalar range")]
	SignatureR,

	/// ECDSA s is outside the curve's nonzero scalar range.
	#[error("signature s is outside the valid scalar range")]
	SignatureS,

	/// ECDSA s violates the account contract's low-s requirement.
	#[error("signature must use canonical low-s encoding")]
	SignatureHighS,
}

/// A specific presigned-batch validation failure; no private input is included in errors.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum BatchValidationError {
	/// The batch contains no operations.
	#[error("a batch requires at least one operation")]
	EmptyOperations,

	/// The batch exceeds the public gauntlet operation limit.
	#[error("batch has {actual} operations; the maximum is {MAX_BATCH_OPS}")]
	TooManyOperations {
		/// Actual operation count.
		actual: usize,
	},

	/// One operation failed its own field or amount validation.
	#[error("operation {index}: {source}")]
	Operation {
		/// Zero-based operation index.
		index: usize,

		/// Original typed validation failure.
		#[source]
		source: Box<BatchValidationError>,
	},

	/// A field has invalid wire syntax.
	#[error("invalid {field:?}: {source}")]
	Field {
		/// Invalid field.
		field: BatchField,

		/// Exact structural failure.
		#[source]
		source: WireValueError,
	},

	/// Split and merge shares must be strictly positive micro-units.
	#[error("split and merge shares must be positive")]
	NonPositiveShares,

	/// A withdrawal falls below the router's one-cent floor.
	#[error("withdrawal must be at least {MIN_WITHDRAW_MICRO} micro-collateral units")]
	WithdrawalBelowMinimum,

	/// Sending collateral to the account itself is forbidden.
	#[error("withdrawal destination must differ from the account")]
	SelfWithdrawal,

	/// Sequence numbers must fit the persistent signed integer representation.
	#[error("batch sequence exceeds the persistent i64 range")]
	SequenceStorageRange,

	/// Deadlines must be positive and fit the persistent signed integer representation.
	#[error("batch deadline must be positive and fit the persistent i64 range")]
	DeadlineStorageRange,

	/// A submitted or freshly signed deadline must be later than the current instant.
	#[error("batch deadline has already passed")]
	DeadlineExpired,

	/// The registry TTL and router skew allowance prohibit a later deadline.
	#[error("batch deadline exceeds the {MAX_BATCH_DEADLINE_HORIZON_SECONDS}-second horizon")]
	DeadlineTooFar,

	/// The local clock precedes the Unix epoch.
	#[error("local clock precedes the Unix epoch")]
	ClockBeforeEpoch(#[source] SystemTimeError),

	/// A clock value cannot be represented by the router's signed timestamps.
	#[error("local clock is outside the persistent timestamp range")]
	ClockStorageRange,
}

pub(crate) fn decode_hex<const N: usize>(value: &str) -> Result<[u8; N], WireValueError> {
	let raw = value
		.strip_prefix("0x")
		.filter(|raw| raw.len() == N * 2)
		.ok_or(WireValueError::HexLength { bytes: N })?;
	let mut output = [0; N];
	for (index, pair) in raw.as_bytes().chunks_exact(2).enumerate() {
		let high = hex_nibble(pair[0]).ok_or(WireValueError::HexEncoding)?;
		let low = hex_nibble(pair[1]).ok_or(WireValueError::HexEncoding)?;
		output[index] = high * 16 + low;
	}

	Ok(output)
}

pub(crate) fn address_bytes(value: &str) -> Result<[u8; 20], WireValueError> {
	let bytes = decode_hex(value)?;
	if bytes == [0; 20] {
		return Err(WireValueError::Zero);
	}

	Ok(bytes)
}

pub(crate) fn signature_bytes(value: &str) -> Result<[u8; SIGNATURE_BYTES], WireValueError> {
	let bytes = decode_hex(value)?;
	let r = &bytes[..SCALAR_BYTES];
	let s = &bytes[SCALAR_BYTES..SIGNATURE_V_INDEX];
	if r.iter().all(|b| *b == 0) || r >= &SECP256K1_ORDER[..] {
		return Err(WireValueError::SignatureR);
	}
	if s.iter().all(|b| *b == 0) || s >= &SECP256K1_ORDER[..] {
		return Err(WireValueError::SignatureS);
	}
	if s > &SECP256K1_HALF_ORDER[..] {
		return Err(WireValueError::SignatureHighS);
	}
	if !core::matches!(bytes[SIGNATURE_V_INDEX], 0 | 1 | 27 | 28) {
		return Err(WireValueError::SignatureParity);
	}

	Ok(bytes)
}

pub(crate) fn uint256_decimal(value: &str) -> Result<&str, WireValueError> {
	if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
		return Err(WireValueError::DecimalSyntax);
	}
	let canonical = value.trim_start_matches('0');
	if canonical.len() > UINT256_MAX_DECIMAL.len()
		|| (canonical.len() == UINT256_MAX_DECIMAL.len() && canonical > UINT256_MAX_DECIMAL)
	{
		return Err(WireValueError::Uint256Overflow);
	}

	Ok(if canonical.is_empty() { "0" } else { canonical })
}

pub(crate) fn validate_operations(
	ops: &[BatchOpDto],
	account: Option<&str>,
) -> Result<(), BatchValidationError> {
	if ops.is_empty() {
		return Err(BatchValidationError::EmptyOperations);
	}
	if ops.len() > MAX_BATCH_OPS {
		return Err(BatchValidationError::TooManyOperations { actual: ops.len() });
	}
	for (index, op) in ops.iter().enumerate() {
		op.validate_for_account_inner(account).map_err(|source| {
			BatchValidationError::Operation { index, source: Box::new(source) }
		})?;
	}

	Ok(())
}

pub(crate) fn validate_binding(seq: u64, deadline: u64) -> Result<(), BatchValidationError> {
	if seq > i64::MAX as u64 {
		return Err(BatchValidationError::SequenceStorageRange);
	}
	if deadline == 0 || deadline > i64::MAX as u64 {
		return Err(BatchValidationError::DeadlineStorageRange);
	}

	Ok(())
}

pub(crate) fn now_unix_seconds() -> Result<i64, BatchValidationError> {
	let now = SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.map_err(BatchValidationError::ClockBeforeEpoch)?;

	i64::try_from(now.as_secs()).map_err(|_| BatchValidationError::ClockStorageRange)
}

fn hex_nibble(byte: u8) -> Option<u8> {
	match byte {
		b'0'..=b'9' => Some(byte - b'0'),
		b'a'..=b'f' => Some(byte - b'a' + 10),
		b'A'..=b'F' => Some(byte - b'A' + 10),
		_ => None,
	}
}

pub(crate) fn validate_deadline_at(deadline: u64, now: i64) -> Result<(), BatchValidationError> {
	validate_binding(0, deadline)?;
	if now < 0 {
		return Err(BatchValidationError::ClockStorageRange);
	}
	let horizon = i128::from(deadline) - i128::from(now);
	if horizon <= 0 {
		return Err(BatchValidationError::DeadlineExpired);
	}
	if horizon > i128::from(MAX_BATCH_DEADLINE_HORIZON_SECONDS) {
		return Err(BatchValidationError::DeadlineTooFar);
	}

	Ok(())
}

fn validate_market_id(value: &str) -> Result<(), WireValueError> {
	let id = uuid::Uuid::parse_str(value).map_err(WireValueError::UuidSyntax)?;
	if id.is_nil() {
		return Err(WireValueError::NilUuid);
	}

	Ok(())
}

impl BatchOpDto {
	/// Validate UUID, condition/address syntax, positivity and the withdrawal floor.
	pub fn validate(&self) -> Result<(), BatchValidationError> {
		self.validate_for_account_inner(None)
	}

	/// Additionally reject a withdrawal to `account`; the account must be a nonzero EVM address.
	pub fn validate_for_account(&self, account: &str) -> Result<(), BatchValidationError> {
		address_bytes(account)
			.map_err(|source| BatchValidationError::Field { field: BatchField::Account, source })?;

		self.validate_for_account_inner(Some(account))
	}

	fn validate_for_account_inner(
		&self,
		account: Option<&str>,
	) -> Result<(), BatchValidationError> {
		match self {
			Self::Split { market_id, condition_id, shares_micro }
			| Self::Merge { market_id, condition_id, shares_micro } => {
				validate_market_id(market_id).map_err(|source| BatchValidationError::Field {
					field: BatchField::MarketId,
					source,
				})?;
				decode_hex::<32>(condition_id).map_err(|source| BatchValidationError::Field {
					field: BatchField::ConditionId,
					source,
				})?;
				if *shares_micro <= 0 {
					return Err(BatchValidationError::NonPositiveShares);
				}
			},
			Self::Withdraw { destination, amount_micro } => {
				address_bytes(destination).map_err(|source| BatchValidationError::Field {
					field: BatchField::Destination,
					source,
				})?;
				if *amount_micro < MIN_WITHDRAW_MICRO {
					return Err(BatchValidationError::WithdrawalBelowMinimum);
				}
				if account.is_some_and(|account| account.eq_ignore_ascii_case(destination)) {
					return Err(BatchValidationError::SelfWithdrawal);
				}
			},
		}

		Ok(())
	}
}

impl AccountBatchSubmission {
	/// Validate this submission against the local clock without making network requests.
	pub fn validate(&self) -> Result<(), BatchValidationError> {
		self.validate_at(now_unix_seconds()?)
	}

	/// Validate deterministically at a nonnegative Unix timestamp, in seconds.
	pub fn validate_at(&self, now: i64) -> Result<(), BatchValidationError> {
		validate_binding(self.seq, self.deadline_unix_seconds)?;
		validate_deadline_at(self.deadline_unix_seconds, now)?;
		validate_operations(&self.ops, None)?;
		signature_bytes(&self.signature).map_err(|source| BatchValidationError::Field {
			field: BatchField::Signature,
			source,
		})?;
		if let Some(hash) = &self.heals_batch_hash {
			decode_hex::<32>(hash).map_err(|source| BatchValidationError::Field {
				field: BatchField::HealsBatchHash,
				source,
			})?;
		}

		Ok(())
	}

	/// Apply normal validation and reject withdrawals returning collateral to this account.
	pub fn validate_for_account(&self, account: &str) -> Result<(), BatchValidationError> {
		self.validate()?;
		address_bytes(account)
			.map_err(|source| BatchValidationError::Field { field: BatchField::Account, source })?;

		validate_operations(&self.ops, Some(account))
	}
}

impl AccountBatchSupersedeSubmission {
	/// Validate a successor against the local clock; its inherited sequence is checked by the router.
	pub fn validate(&self) -> Result<(), BatchValidationError> {
		self.validate_at(now_unix_seconds()?)
	}

	/// Validate a successor deterministically at a nonnegative Unix timestamp, in seconds.
	pub fn validate_at(&self, now: i64) -> Result<(), BatchValidationError> {
		validate_deadline_at(self.deadline_unix_seconds, now)?;
		validate_operations(&self.ops, None)?;
		signature_bytes(&self.signature).map_err(|source| BatchValidationError::Field {
			field: BatchField::Signature,
			source,
		})?;

		Ok(())
	}

	/// Apply successor validation and reject a withdrawal to the signing account itself.
	pub fn validate_for_account(&self, account: &str) -> Result<(), BatchValidationError> {
		self.validate()?;
		address_bytes(account)
			.map_err(|source| BatchValidationError::Field { field: BatchField::Account, source })?;

		validate_operations(&self.ops, Some(account))
	}
}

impl AccountBatchStatusDto {
	/// Recognize completion only when the producer's state-specific evidence is present.
	///
	/// Settlement requires a confirmed transaction and execution timestamp, with no failure.
	/// Failed rows require their public failure; clean failures additionally need completed unwind.
	/// Malformed caller-built DTOs and future states never become terminal merely from a label.
	pub fn is_terminal(&self) -> bool {
		match self.status {
			BatchStatus::Settled => {
				self.tx_hash.is_some() && self.executed_at.is_some() && self.failure.is_none()
			},
			BatchStatus::Failed => self.unwound_at.is_some() && self.failure.is_some(),
			BatchStatus::FailedDivergent => self.failure.is_some(),
			_ => false,
		}
	}
}

impl BatchGroupStatusDto {
	/// Check confirmed closure and one successful settlement per frozen chunk slot.
	///
	/// Earlier failed attempts are retained by the producer and do not prevent completion. A
	/// confirmed empty plan is complete with zero chunks. Unknown states, duplicate batch identities or settlements,
	/// missing transaction proofs and mutable DTO inconsistencies fail closed.
	pub fn is_complete(&self) -> bool {
		let Some(chunk_count) = self.chunk_count else {
			return false;
		};
		let Ok(expected) = usize::try_from(chunk_count) else {
			return false;
		};
		if self.completed_at.is_none() || expected != self.op_count.div_ceil(MAX_BATCH_OPS) {
			return false;
		}

		let mut settled = std::collections::BTreeSet::new();
		let mut identities = std::collections::BTreeSet::new();
		for chunk in &self.chunks {
			if chunk.chunk_index >= chunk_count || !identities.insert(chunk.batch_hash.as_str()) {
				return false;
			}

			match chunk.status {
				BatchStatus::Settled => {
					if chunk.tx_hash.is_none()
						|| chunk.failure.is_some()
						|| !settled.insert(chunk.chunk_index)
					{
						return false;
					}
				},
				BatchStatus::Failed if chunk.failure.is_some() => {},
				_ => return false,
			}
		}

		settled.len() == expected
	}
}

#![cfg(feature = "signing")]

//! Validated account-batch composition, hashing and holder signing.

pub mod compose;

mod bindings;
mod hashable;
mod tests;

use alloy::{
	primitives::{Address, B256, Bytes, U256},
	sol_types::{Eip712Domain, SolStruct},
};

use crate::{
	batches::{self, BatchField, BatchValidationError, MAX_BATCH_OPS},
	signing::{self, CallFailure, DomainField, SigningError},
};

/// Contract-defined domain name for account batches.
pub const ACCOUNT_DOMAIN_NAME: &str = "AgaraAccount";

/// Initial installed implementation version used by account deployments.
pub const INITIAL_ACCOUNT_IMPLEMENTATION_VERSION: u64 = 1;

/// One exact call in the EIP-712 batch. Raw calls are checked again by `batch_digest_local`.
#[derive(Clone, Debug)]
pub struct BatchCall {
	/// Nonzero call target; it must differ from the executing account.
	pub target: Address,

	/// Native value in wei, exactly represented as Uint256.
	pub value: U256,

	/// ABI calldata; this hashing DTO also permits empty data for native transfers.
	pub data: Bytes,
}

/// Canonical composed call encoded for inspection or transport.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct BatchCallDto {
	/// Nonzero 0x-prefixed EVM contract address.
	pub target: String,

	/// Unsigned decimal Uint256 native value in wei.
	pub value: String,

	/// Even-length 0x-prefixed ABI calldata.
	pub data: String,
}

/// Validate the domain, persistent integer bounds and call targets, then compute the exact digest.
///
/// Historical positive deadlines are permitted for offline conformance vectors. Submission and
/// `sign_batch` perform the live deadline check. The registry address is not available here, so
/// callers must obtain raw call targets from an authoritative allowlist; the account contract also
/// rejects calls to its registry. No input is silently truncated to a smaller integer.
pub fn batch_digest_local(
	account: Address,
	implementation_version: u64,
	seq: U256,
	deadline: U256,
	calls: &[BatchCall],
	chain_id: u64,
) -> Result<B256, SigningError> {
	validate_domain(account, implementation_version, chain_id)?;
	if seq > U256::from(i64::MAX as u64) {
		return Err(BatchValidationError::SequenceStorageRange.into());
	}
	if deadline.is_zero() || deadline > U256::from(i64::MAX as u64) {
		return Err(BatchValidationError::DeadlineStorageRange.into());
	}
	if calls.is_empty() {
		return Err(BatchValidationError::EmptyOperations.into());
	}
	if calls.len() > MAX_BATCH_OPS {
		return Err(BatchValidationError::TooManyOperations { actual: calls.len() }.into());
	}
	for (index, call) in calls.iter().enumerate() {
		if call.target.is_zero() {
			return Err(SigningError::Call { index, reason: CallFailure::ZeroTarget });
		}
		if call.target == account {
			return Err(SigningError::Call { index, reason: CallFailure::SelfTarget });
		}
	}

	let typed = hashable::Batch {
		wallet: account,
		seq,
		deadline,
		calls: calls
			.iter()
			.map(|call| hashable::Call {
				target: call.target,
				value: call.value,
				data: call.data.clone(),
			})
			.collect(),
	};
	let domain = Eip712Domain {
		name: Some(ACCOUNT_DOMAIN_NAME.into()),
		version: Some(implementation_version.to_string().into()),
		chain_id: Some(U256::from(chain_id)),
		verifying_contract: Some(account),
		salt: None,
	};

	Ok(typed.eip712_signing_hash(&domain))
}

/// Compose, validate and sign a presigned batch, returning both its digest and submission DTO.
///
/// Enforces the operation limit, withdrawal floor, self-withdrawal rule, persistent sequence bounds,
/// nonzero deployment domain and router TTL. `now_unix_seconds` optionally supplies an explicit clock
/// for reproducible offline signing; the default uses the local clock, and HTTP submission always
/// revalidates against its current clock. The private key is never copied into errors.
#[bon::builder]
pub fn sign_batch(
	private_key: &str,
	account: Address,
	implementation_version: u64,
	seq: u64,
	deadline_unix_seconds: u64,
	chain_id: u64,
	ops: &[compose::RoutedOp],
	context: &compose::ComposeContext,
	heals_batch_hash: Option<String>,
	now_unix_seconds: Option<i64>,
) -> Result<(B256, crate::batches::AccountBatchSubmission), SigningError> {
	let now = match now_unix_seconds {
		Some(now) => now,
		None => batches::now_unix_seconds()?,
	};
	batches::validate_deadline_at(deadline_unix_seconds, now)?;
	if let Some(hash) = &heals_batch_hash {
		batches::decode_hex::<32>(hash).map_err(|source| BatchValidationError::Field {
			field: BatchField::HealsBatchHash,
			source,
		})?;
	}
	validate_domain(account, implementation_version, chain_id)?;
	batches::validate_binding(seq, deadline_unix_seconds)?;
	let calls = compose::compose_for_account(ops, context, account)?;
	let calls = calls.iter().map(BatchCallDto::to_call).collect::<Result<Vec<_>, _>>()?;
	let digest = batch_digest_local(
		account,
		implementation_version,
		U256::from(seq),
		U256::from(deadline_unix_seconds),
		&calls,
		chain_id,
	)?;
	let signature = signing::sign_digest(private_key, &digest)?;
	let submission = crate::batches::AccountBatchSubmission {
		ops: ops.iter().map(|op| op.op.clone()).collect(),
		seq,
		deadline_unix_seconds,
		signature: alloy::hex::encode_prefixed(signature.as_bytes()),
		heals_batch_hash,
	};
	submission.validate_at(now)?;

	Ok((digest, submission))
}

fn validate_domain(
	account: Address,
	implementation_version: u64,
	chain_id: u64,
) -> Result<(), SigningError> {
	if account.is_zero() {
		return Err(SigningError::Domain(DomainField::Account));
	}
	if chain_id == 0 {
		return Err(SigningError::Domain(DomainField::ChainId));
	}
	if implementation_version == 0 {
		return Err(SigningError::Domain(DomainField::ImplementationVersion));
	}

	Ok(())
}

impl BatchCallDto {
	/// Decode exact call data while rejecting zero targets, malformed hex and Uint256 overflow.
	pub fn to_call(&self) -> Result<BatchCall, SigningError> {
		let target = batches::address_bytes(&self.target).map_err(|source| {
			BatchValidationError::Field { field: BatchField::CallTarget, source }
		})?;
		let value = batches::uint256_decimal(&self.value).map_err(|source| {
			BatchValidationError::Field { field: BatchField::CallValue, source }
		})?;
		let data = self.data.strip_prefix("0x").ok_or(SigningError::CallDataPrefix)?;
		let data = alloy::hex::decode(data).map_err(SigningError::CallData)?;

		Ok(BatchCall {
			target: Address::from(target),
			value: signing::decode_uint256(value)?,
			data: data.into(),
		})
	}
}

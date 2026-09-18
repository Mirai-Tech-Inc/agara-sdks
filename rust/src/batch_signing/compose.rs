//! Canonical presigned SPLIT, MERGE and WITHDRAW call composition.

use alloy::{
	primitives::{Address, B256, U256},
	sol_types::SolCall,
};

use super::{
	BatchCallDto,
	bindings::{IConditionalTokens, IERC20},
};
use crate::batches::{self, BatchField, BatchOpDto, BatchValidationError, MAX_BATCH_OPS};

const BINARY_PARTITION: [u64; 2] = [1, 2];

/// On-chain contract family used to execute a market operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum MarketRoute {
	/// Standard ConditionalTokens split and merge.
	Ctf,

	/// Neg-risk adapter merge; presigned neg-risk split is unsupported.
	NegRisk,
}

/// An operation paired with the route obtained from authoritative market metadata.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RoutedOp {
	/// Typed intent, validated again during every composition.
	pub op: BatchOpDto,

	/// Contract route; ignored for collateral withdrawals.
	pub route: MarketRoute,
}

/// Deployment contract whose configured address is invalid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContractTarget {
	/// ConditionalTokens contract.
	Ctf,

	/// Neg-risk merge adapter.
	NegRiskAdapter,

	/// Collateral ERC-20 token.
	Collateral,
}

/// Nonzero deployment contract addresses used for deterministic call composition.
#[derive(Debug, Clone, Copy, dissolve_derive::Dissolve)]
#[dissolve(visibility = "pub(crate)")]
pub struct ComposeContext {
	ctf: Address,
	neg_risk_adapter: Address,
	collateral: Address,
}

/// A typed canonical-composition failure; arbitrary text errors are not used.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ComposeError {
	/// A deployment contract was configured as the zero address.
	#[error("{0:?} contract address must be nonzero")]
	ZeroContract(ContractTarget),

	/// An operation or caller account violates the presigned batch contract.
	#[error(transparent)]
	Batch(#[from] BatchValidationError),

	/// The public presigned surface cannot split through the neg-risk adapter.
	#[error("operation {index}: neg-risk split is not supported")]
	NegRiskSplit {
		/// Zero-based operation index.
		index: usize,
	},
}

/// Validate one to twenty operations and compose their ABI calls in the same order.
///
/// This function cannot check self-withdrawal without an account address; use `compose_for_account`
/// when the account is known. Shares and withdrawals are integer micro-units; zero and negative
/// values are rejected, and withdrawals must meet the one-cent floor.
pub fn compose(
	ops: &[RoutedOp],
	context: &ComposeContext,
) -> Result<Vec<BatchCallDto>, ComposeError> {
	compose_checked(ops, context, None)
}

/// Compose after additionally rejecting zero accounts and withdrawals to the account itself.
pub fn compose_for_account(
	ops: &[RoutedOp],
	context: &ComposeContext,
	account: Address,
) -> Result<Vec<BatchCallDto>, ComposeError> {
	let account = alloy::hex::encode_prefixed(account);
	batches::address_bytes(&account)
		.map_err(|source| BatchValidationError::Field { field: BatchField::Account, source })?;

	compose_checked(ops, context, Some(&account))
}

fn compose_checked(
	ops: &[RoutedOp],
	context: &ComposeContext,
	account: Option<&str>,
) -> Result<Vec<BatchCallDto>, ComposeError> {
	if ops.is_empty() {
		return Err(BatchValidationError::EmptyOperations.into());
	}
	if ops.len() > MAX_BATCH_OPS {
		return Err(BatchValidationError::TooManyOperations { actual: ops.len() }.into());
	}
	for (index, routed) in ops.iter().enumerate() {
		let result = match account {
			Some(account) => routed.op.validate_for_account(account),
			None => routed.op.validate(),
		};
		result.map_err(|source| BatchValidationError::Operation {
			index,
			source: Box::new(source),
		})?;
	}

	ops.iter().enumerate().map(|(index, routed)| compose_one(index, routed, context)).collect()
}

fn compose_one(
	index: usize,
	routed: &RoutedOp,
	context: &ComposeContext,
) -> Result<BatchCallDto, ComposeError> {
	let context = (*context).dissolve();

	match (&routed.op, routed.route) {
		(BatchOpDto::Split { condition_id, shares_micro, .. }, MarketRoute::Ctf) => {
			let data = IConditionalTokens::splitPositionCall {
				collateralToken: context.collateral,
				parentCollectionId: B256::ZERO,
				conditionId: parse_condition(condition_id)?,
				partition: binary_partition(),
				amount: positive_amount(*shares_micro)?,
			};

			Ok(call(context.ctf, data.abi_encode()))
		},
		(BatchOpDto::Split { .. }, MarketRoute::NegRisk) => {
			Err(ComposeError::NegRiskSplit { index })
		},
		(BatchOpDto::Merge { condition_id, shares_micro, .. }, route) => {
			// Both merge routes use the same ABI signature and partition.
			let data = IConditionalTokens::mergePositionsCall {
				collateralToken: context.collateral,
				parentCollectionId: B256::ZERO,
				conditionId: parse_condition(condition_id)?,
				partition: binary_partition(),
				amount: positive_amount(*shares_micro)?,
			};
			let target = match route {
				MarketRoute::Ctf => context.ctf,
				MarketRoute::NegRisk => context.neg_risk_adapter,
			};

			Ok(call(target, data.abi_encode()))
		},
		(BatchOpDto::Withdraw { destination, amount_micro }, _) => {
			let destination = batches::address_bytes(destination).map_err(|source| {
				BatchValidationError::Field { field: BatchField::Destination, source }
			})?;
			let data = IERC20::transferCall {
				to: Address::from(destination),
				amount: positive_amount(*amount_micro)?,
			};

			Ok(call(context.collateral, data.abi_encode()))
		},
	}
}

fn parse_condition(value: &str) -> Result<B256, ComposeError> {
	let bytes = batches::decode_hex(value)
		.map_err(|source| BatchValidationError::Field { field: BatchField::ConditionId, source })?;

	Ok(B256::from(bytes))
}

fn positive_amount(value: i64) -> Result<U256, ComposeError> {
	if value <= 0 {
		return Err(BatchValidationError::NonPositiveShares.into());
	}

	Ok(U256::from(value.unsigned_abs()))
}

fn call(target: Address, data: Vec<u8>) -> BatchCallDto {
	BatchCallDto {
		target: alloy::hex::encode_prefixed(target),
		value: "0".to_owned(),
		data: alloy::hex::encode_prefixed(data),
	}
}

fn binary_partition() -> Vec<U256> {
	BINARY_PARTITION.iter().copied().map(U256::from).collect()
}

#[bon::bon]
impl ComposeContext {
	/// Validate nonzero deployment addresses. The builder returns a result and has no unchecked
	/// serde constructor; callers must obtain these addresses from their deployment configuration.
	#[builder]
	pub fn new(
		ctf: Address,
		neg_risk_adapter: Address,
		collateral: Address,
	) -> Result<Self, ComposeError> {
		for (target, address) in [
			(ContractTarget::Ctf, ctf),
			(ContractTarget::NegRiskAdapter, neg_risk_adapter),
			(ContractTarget::Collateral, collateral),
		] {
			if address.is_zero() {
				return Err(ComposeError::ZeroContract(target));
			}
		}

		Ok(Self { ctf, neg_risk_adapter, collateral })
	}

	/// Validated ConditionalTokens call target.
	pub fn ctf(&self) -> Address {
		self.ctf
	}

	/// Validated neg-risk adapter call target.
	pub fn neg_risk_adapter(&self) -> Address {
		self.neg_risk_adapter
	}

	/// Validated collateral ERC-20 call target.
	pub fn collateral(&self) -> Address {
		self.collateral
	}
}

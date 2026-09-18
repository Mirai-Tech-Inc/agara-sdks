//! Validated signed-order envelopes and request DTOs.

use std::time::{SystemTime, SystemTimeError, UNIX_EPOCH};

use crate::{
	batches::{self, WireValueError},
	ids::{ConditionId, Exchange, OrderHash, OrderType, Side, TimeInForce, TokenId},
	units::Micro,
};
use serde::Serialize;

const MIN_GTD_HORIZON_SECONDS: i64 = 30;
const SIGNATURE_RECOVERY_OFFSET: usize = 64;
const ETHEREUM_PARITY_EVEN: u8 = 27;
const ETHEREUM_PARITY_ODD: u8 = 28;
const MIN_LIMIT_NOTIONAL_MICRO: i64 = 100_000;
const MAX_LIMIT_NOTIONAL_MICRO: i64 = 100_000_000_000;

/// A signed-order field whose structural or economic validation failed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OrderField {
	/// Public outcome token identifier.
	TokenId,

	/// Decimal chain token identifier.
	ChainTokenId,

	/// Nonzero order salt.
	Salt,

	/// Smart-account maker address.
	Maker,

	/// Maker's signed amount.
	MakerAmount,

	/// Taker's signed amount.
	TakerAmount,

	/// Signed timestamp.
	Timestamp,

	/// Opaque metadata bytes32.
	Metadata,

	/// Opaque builder attribution bytes32.
	Builder,

	/// EIP-712 order digest.
	OrderHash,

	/// Holder signature.
	Signature,
}

/// Local signed-order validation failure, suitable for matching without parsing messages.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum OrderValidationError {
	/// A particular item in a signed-order batch failed validation before any submission.
	#[error("signed order {index}: {source}")]
	BatchEntry {
		/// Zero-based request index.
		index: usize,

		/// Original machine-matchable order validation failure.
		#[source]
		source: Box<OrderValidationError>,
	},

	/// Signed submission accepts LIMIT orders only.
	#[error("presigned orders must use LIMIT")]
	OrderType,

	/// Side must be explicitly BUY or SELL.
	#[error("side must be BUY or SELL")]
	Side,

	/// Price must be inside the open probability interval.
	#[error("price must be within 1..=999999 micro-probability")]
	Price,

	/// Shares must be positive integer micro-shares.
	#[error("shares must be positive")]
	Shares,

	/// An order violates the platform's minimum signed LIMIT notional.
	#[error("limit notional is below {MIN_LIMIT_NOTIONAL_MICRO} micro-collateral units")]
	NotionalBelowMinimum,

	/// An order violates the platform's maximum signed LIMIT notional.
	#[error("limit notional exceeds {MAX_LIMIT_NOTIONAL_MICRO} micro-collateral units")]
	NotionalAboveMaximum,

	/// Exact amount arithmetic could not be represented.
	#[error("order amount arithmetic overflow")]
	AmountOverflow,

	/// A supported time-in-force value is required.
	#[error("time in force must be GTC, GTD, FAK, or FOK")]
	TimeInForce,

	/// FAK and FOK cannot be post-only.
	#[error("post-only requires GTC or GTD")]
	PostOnlyTimeInForce,

	/// GTD requires an expiration timestamp.
	#[error("GTD requires expiration_unix_seconds")]
	ExpirationRequired,

	/// Other time-in-force values cannot carry expiration.
	#[error("expiration is permitted only for GTD")]
	ExpirationUnexpected,

	/// GTD requires at least thirty seconds of remaining life.
	#[error("GTD expiration must be at least {MIN_GTD_HORIZON_SECONDS} seconds ahead")]
	ExpirationTooSoon,

	/// A wire field does not have the expected fixed-width or integer representation.
	#[error("invalid {field:?}: {source}")]
	Field {
		/// Invalid wire field.
		field: OrderField,

		/// Precise structural failure.
		#[source]
		source: WireValueError,
	},

	/// Public and chain token identifiers denote different values.
	#[error("public token identifier differs from signed chain token")]
	TokenMismatch,

	/// Public and signed numeric side values disagree.
	#[error("public side differs from signed side")]
	SideMismatch,

	/// A signed amount differs from the amount implied by the public order.
	#[error("signed {field:?} differs from the order economics")]
	AmountMismatch {
		/// Signed amount that disagrees with price and shares.
		field: OrderField,
	},

	/// A request builder changed the economics originally signed by the caller.
	#[error("request economics differ from the signed order")]
	SignedEconomicsMismatch,

	/// Local clock precedes the Unix epoch.
	#[error("local clock precedes the Unix epoch")]
	ClockBeforeEpoch(#[source] SystemTimeError),

	/// The supplied clock cannot represent a valid router timestamp.
	#[error("local clock is outside the persistent timestamp range")]
	ClockStorageRange,
}

#[derive(Debug, Serialize)]
pub(crate) struct CreateOrderRequest {
	pub token_id: TokenId,
	pub side: Side,

	#[serde(rename = "type")]
	pub order_type: OrderType,
	pub time_in_force: TimeInForce,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub price_micro: Option<Micro>,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub collateral_amount_micro: Option<Micro>,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub shares_micro: Option<Micro>,
	pub post_only: bool,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub expiration_unix_seconds: Option<i64>,
}

/// A pre-signed limit order envelope, ready to POST. Produced by
/// `SignedOrder::to_request_body` (feature `signing`); public so callers
/// can build it by hand if they sign elsewhere.
#[derive(Clone, Debug, Serialize)]
pub struct SignedOrderRequest {
	/// Decimal outcome-token identifier; must equal `chain_token_id`.
	pub token_id: TokenId,

	/// BUY spends collateral for shares; SELL spends shares for collateral.
	pub side: Side,

	/// Must be LIMIT for a presigned submission.
	#[serde(rename = "type")]
	pub order_type: OrderType,

	/// GTC/GTD may rest; FAK/FOK execute immediately without leaving a resting remainder.
	pub time_in_force: TimeInForce,

	/// Limit price in micro-probability, between 1 and 999999.
	pub price_micro: Micro,

	/// Positive share quantity in integer micro-shares.
	pub shares_micro: Micro,

	/// Require maker-only execution; permitted only with GTC or GTD.
	pub post_only: bool,

	/// GTD-only Unix deadline, at least 30 seconds ahead; absent for every other time in force.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub expiration_unix_seconds: Option<i64>,

	/// EIP-712 digest binding all nine signed fields to the chain and exchange deployment.
	pub order_hash: OrderHash,

	/// Low-s 65-byte ECDSA holder signature in 0x hex; recovery parity must be 27 or 28.
	pub signature: String,

	/// Nonzero decimal Uint256 salt.
	pub salt: String,

	/// Nonzero account address that owns the order.
	pub maker: String,

	/// Decimal Uint256 token identifier; must denote the same outcome as `token_id`.
	pub chain_token_id: String,

	/// Decimal Uint256 maker amount, matching the side-specific order economics.
	pub maker_amount: String,

	/// Decimal Uint256 taker amount, matching the side-specific order economics.
	pub taker_amount: String,

	/// Signed numeric side: 0 for BUY, 1 for SELL.
	pub side_u8: u8,

	/// Decimal Uint256 signed verbatim, commonly zero; this is independent of the GTD deadline.
	pub timestamp: String,

	/// Exactly 32 opaque metadata bytes in 0x-prefixed hexadecimal.
	pub metadata: String,

	/// Exactly 32 builder-attribution bytes in 0x-prefixed hexadecimal.
	pub builder: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct SignedOrderBatchRequest {
	pub orders: Vec<SignedOrderRequest>,
}

#[derive(Debug, Serialize)]
pub(crate) struct OrdersListRequest {
	pub limit: u32,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub cursor: Option<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct PositionsListRequest {
	pub condition_ids: Vec<ConditionId>,
	pub exchanges: Vec<Exchange>,
}

/// One page of active orders, retaining filters while an opaque continuation cursor advances.
#[derive(Debug, Serialize)]
pub struct OpenOrdersListRequest {
	/// Restrict to these outcomes; an empty list selects every outcome.
	pub token_ids: Vec<TokenId>,

	/// Restrict to these venues; an empty list queries every onboarded exchange.
	pub exchanges: Vec<Exchange>,

	/// Requested page size, from 1 through 500.
	pub limit: u32,

	/// Opaque cursor from the preceding response; omit for the first page.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub cursor: Option<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct SplitRequest {
	pub condition_id: ConditionId,
	pub collateral_amount_micro: Micro,
}

#[derive(Debug, Serialize)]
pub(crate) struct MergeRequest {
	pub condition_id: ConditionId,
	pub shares_micro: Micro,
}

impl SignedOrderRequest {
	/// Check LIMIT shape, live expiry, exact amounts and canonical signature scalars before sending.
	///
	/// This does not prove the signature belongs to the holder or the hash matches a deployment;
	/// use `verify` with the signing feature when that domain and holder are available.
	pub fn validate(&self) -> Result<(), OrderValidationError> {
		let now = SystemTime::now()
			.duration_since(UNIX_EPOCH)
			.map_err(OrderValidationError::ClockBeforeEpoch)?;
		let now =
			i64::try_from(now.as_secs()).map_err(|_| OrderValidationError::ClockStorageRange)?;

		self.validate_at(now)
	}

	/// Perform structural and economic validation at an explicit nonnegative Unix timestamp.
	pub fn validate_at(&self, now: i64) -> Result<(), OrderValidationError> {
		if self.order_type != OrderType::Limit {
			return Err(OrderValidationError::OrderType);
		}
		validate_limit_options(
			self.side,
			self.time_in_force,
			self.post_only,
			self.expiration_unix_seconds,
			now,
		)?;
		let collateral = limit_collateral(self.price_micro, self.shares_micro)?;
		let field = |field, source| OrderValidationError::Field { field, source };
		let token = batches::uint256_decimal(self.token_id.as_str())
			.map_err(|source| field(OrderField::TokenId, source))?;
		let chain_token = batches::uint256_decimal(&self.chain_token_id)
			.map_err(|source| field(OrderField::ChainTokenId, source))?;
		if token != chain_token {
			return Err(OrderValidationError::TokenMismatch);
		}
		let salt = batches::uint256_decimal(&self.salt)
			.map_err(|source| field(OrderField::Salt, source))?;
		if salt == "0" {
			return Err(field(OrderField::Salt, WireValueError::Zero));
		}
		batches::address_bytes(&self.maker).map_err(|source| field(OrderField::Maker, source))?;
		batches::uint256_decimal(&self.timestamp)
			.map_err(|source| field(OrderField::Timestamp, source))?;
		batches::decode_hex::<32>(&self.metadata)
			.map_err(|source| field(OrderField::Metadata, source))?;
		batches::decode_hex::<32>(&self.builder)
			.map_err(|source| field(OrderField::Builder, source))?;
		batches::decode_hex::<32>(self.order_hash.as_str())
			.map_err(|source| field(OrderField::OrderHash, source))?;
		let signature = batches::signature_bytes(&self.signature)
			.map_err(|source| field(OrderField::Signature, source))?;
		if !core::matches!(
			signature[SIGNATURE_RECOVERY_OFFSET],
			ETHEREUM_PARITY_EVEN | ETHEREUM_PARITY_ODD
		) {
			return Err(field(
				OrderField::Signature,
				WireValueError::SignatureNonCanonicalParity,
			));
		}
		let (side, maker, taker) = match self.side {
			Side::Buy => (0, collateral, self.shares_micro.raw()),
			Side::Sell => (1, self.shares_micro.raw(), collateral),
			Side::Unspecified => return Err(OrderValidationError::Side),
		};
		if self.side_u8 != side {
			return Err(OrderValidationError::SideMismatch);
		}
		for (field_name, encoded, expected) in [
			(OrderField::MakerAmount, self.maker_amount.as_str(), maker),
			(OrderField::TakerAmount, self.taker_amount.as_str(), taker),
		] {
			let encoded =
				batches::uint256_decimal(encoded).map_err(|source| field(field_name, source))?;
			if encoded != expected.to_string() {
				return Err(OrderValidationError::AmountMismatch { field: field_name });
			}
		}

		Ok(())
	}
}

pub(crate) fn limit_collateral(price: Micro, shares: Micro) -> Result<i64, OrderValidationError> {
	if !(1..crate::units::MICRO).contains(&price.raw()) {
		return Err(OrderValidationError::Price);
	}
	if shares.raw() <= 0 {
		return Err(OrderValidationError::Shares);
	}
	let collateral = i128::from(price.raw())
		.checked_mul(i128::from(shares.raw()))
		.and_then(|amount| amount.checked_div(i128::from(crate::units::MICRO)))
		.ok_or(OrderValidationError::AmountOverflow)?;
	if collateral < i128::from(MIN_LIMIT_NOTIONAL_MICRO) {
		return Err(OrderValidationError::NotionalBelowMinimum);
	}
	if collateral > i128::from(MAX_LIMIT_NOTIONAL_MICRO) {
		return Err(OrderValidationError::NotionalAboveMaximum);
	}

	i64::try_from(collateral).map_err(|_| OrderValidationError::AmountOverflow)
}

fn validate_limit_options(
	side: Side,
	tif: TimeInForce,
	post_only: bool,
	expiration: Option<i64>,
	now: i64,
) -> Result<(), OrderValidationError> {
	if now < 0 {
		return Err(OrderValidationError::ClockStorageRange);
	}
	if side == Side::Unspecified {
		return Err(OrderValidationError::Side);
	}
	if tif == TimeInForce::Unspecified {
		return Err(OrderValidationError::TimeInForce);
	}
	if post_only && core::matches!(tif, TimeInForce::Fak | TimeInForce::Fok) {
		return Err(OrderValidationError::PostOnlyTimeInForce);
	}
	match (tif, expiration) {
		(TimeInForce::Gtd, None) => return Err(OrderValidationError::ExpirationRequired),
		(TimeInForce::Gtd, Some(expiration))
			if i128::from(expiration) < i128::from(now) + i128::from(MIN_GTD_HORIZON_SECONDS) =>
		{
			return Err(OrderValidationError::ExpirationTooSoon);
		},
		(TimeInForce::Gtd, Some(_)) | (_, None) => {},
		(_, Some(_)) => return Err(OrderValidationError::ExpirationUnexpected),
	}

	Ok(())
}

use crate::{
	ids::{ConditionId, FillRole, SettlementMode, Side, TimeInForce, TokenId},
	problem::PublicFailure,
};

use super::{EncodedAmount, Scale};

/// An engine-native price level.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Level {
	/// Unsigned engine price, scaled by `price_scale`.
	pub price: u32,

	/// Unsigned engine quantity, scaled by `size_scale`.
	pub size: u64,
}

/// A complete native-unit orderbook snapshot for one outcome.
#[derive(Clone, Debug, serde::Deserialize)]
#[serde(try_from = "WireOrderbookSnapshot")]
pub struct OrderbookSnapshot {
	/// Token id.
	pub token_id: TokenId,

	/// Engine sequence; filtered subscriptions do not guarantee contiguous values.
	pub sequence: u64,

	/// Bids.
	pub bids: Vec<Level>,

	/// Asks.
	pub asks: Vec<Level>,

	/// Tick size.
	pub tick_size: u32,

	/// Positive divisor for engine-native prices.
	pub price_scale: Scale,

	/// Positive divisor for engine-native quantities.
	pub size_scale: Scale,
}

/// Native-unit level replacements; zero size removes the corresponding level.
#[derive(Clone, Debug, serde::Deserialize)]
pub struct OrderbookDelta {
	/// Token id.
	pub token_id: TokenId,

	/// Engine sequence; filtered subscriptions do not guarantee contiguous values.
	pub sequence: u64,

	/// Each entry replaces the level at that price; `size == 0` removes it.
	pub bids: Vec<Level>,

	/// Asks.
	pub asks: Vec<Level>,

	/// Positive divisor for engine-native prices.
	pub price_scale: Scale,

	/// Positive divisor for engine-native quantities.
	pub size_scale: Scale,
}

/// Best available bid and ask; missing sides are explicit nulls on the wire.
#[derive(Clone, Debug, serde::Deserialize)]
pub struct BestQuote {
	/// Token id.
	pub token_id: TokenId,

	/// Engine sequence; filtered subscriptions do not guarantee contiguous values.
	pub sequence: u64,

	/// Bid.
	#[serde(deserialize_with = "super::validation::required_nullable")]
	pub bid: Option<Level>,

	/// Ask.
	#[serde(deserialize_with = "super::validation::required_nullable")]
	pub ask: Option<Level>,

	/// Positive divisor for engine-native prices.
	pub price_scale: Scale,

	/// Positive divisor for engine-native quantities.
	pub size_scale: Scale,
}

/// Initial engine trading bounds and strictly positive unit scales.
#[derive(Clone, Debug, serde::Deserialize)]
#[serde(try_from = "WireMarketCreated")]
pub struct MarketCreated {
	/// Condition id.
	pub condition_id: ConditionId,

	/// Engine sequence; filtered subscriptions do not guarantee contiguous values.
	pub sequence: u64,

	/// Num outcomes.
	pub num_outcomes: u32,

	/// Tick size.
	pub tick_size: u32,

	/// Positive divisor for engine-native prices.
	pub price_scale: Scale,

	/// Positive divisor for engine-native quantities.
	pub size_scale: Scale,

	/// Min price.
	pub min_price: u32,

	/// Max price.
	pub max_price: u32,

	/// Cross match enabled.
	pub cross_match_enabled: bool,
}

/// Trading halted for a condition.
#[derive(Clone, Debug, serde::Deserialize)]
pub struct MarketHalted {
	/// Condition id.
	pub condition_id: ConditionId,

	/// Engine sequence; filtered subscriptions do not guarantee contiguous values.
	pub sequence: u64,
}

/// Trading resumed for a condition.
#[derive(Clone, Debug, serde::Deserialize)]
pub struct MarketResumed {
	/// Condition id.
	pub condition_id: ConditionId,

	/// Engine sequence; filtered subscriptions do not guarantee contiguous values.
	pub sequence: u64,
}

/// The proposed winning token for a condition.
#[derive(Clone, Debug, serde::Deserialize)]
pub struct OutcomeProposed {
	/// Condition id.
	pub condition_id: ConditionId,

	/// Engine sequence; filtered subscriptions do not guarantee contiguous values.
	pub sequence: u64,

	/// Proposed token id.
	pub proposed_token_id: TokenId,
}

/// The final winning token for a condition.
#[derive(Clone, Debug, serde::Deserialize)]
pub struct MarketResolved {
	/// Condition id.
	pub condition_id: ConditionId,

	/// Engine sequence; filtered subscriptions do not guarantee contiguous values.
	pub sequence: u64,

	/// Winning token id.
	pub winning_token_id: TokenId,
}

/// The market fee policy changed; refresh current policy before quoting.
#[derive(Clone, Debug, serde::Deserialize)]
pub struct FeePolicyUpdated {
	/// Condition id.
	pub condition_id: ConditionId,

	/// Engine sequence; filtered subscriptions do not guarantee contiguous values.
	pub sequence: u64,
}

/// The market cross-matching setting changed.
#[derive(Clone, Debug, serde::Deserialize)]
pub struct CrossMatchToggled {
	/// Condition id.
	pub condition_id: ConditionId,

	/// Engine sequence; filtered subscriptions do not guarantee contiguous values.
	pub sequence: u64,

	/// Enabled.
	pub enabled: bool,
}

/// A public tape fill with exact native price and quantity.
#[derive(Clone, Debug, serde::Deserialize)]
pub struct Trade {
	/// Condition id.
	pub condition_id: ConditionId,

	/// Engine sequence; filtered subscriptions do not guarantee contiguous values.
	pub sequence: u64,

	/// Fill id.
	pub fill_id: String,

	/// Taker token id.
	pub taker_token_id: TokenId,

	/// Maker token id.
	pub maker_token_id: TokenId,

	/// Side.
	#[serde(deserialize_with = "super::validation::side")]
	pub side: Side,

	/// Unsigned engine price, scaled by `price_scale`.
	pub price: u32,

	/// Unsigned engine quantity, scaled by `size_scale`.
	pub size: u64,

	/// Positive divisor for engine-native prices.
	pub price_scale: Scale,

	/// Positive divisor for engine-native quantities.
	pub size_scale: Scale,

	/// Settlement mode.
	#[serde(deserialize_with = "super::validation::settlement_mode")]
	pub settlement_mode: SettlementMode,
}

/// This account's leg of an engine fill; settlement is tracked separately.
#[derive(Clone, Debug, serde::Deserialize)]
pub struct Fill {
	/// Engine sequence; filtered subscriptions do not guarantee contiguous values.
	pub sequence: u64,

	/// Role.
	#[serde(deserialize_with = "super::validation::role")]
	pub role: FillRole,

	/// Fill id.
	pub fill_id: String,

	/// Order id.
	pub order_id: crate::ids::OrderId,

	/// Order hash.
	pub order_hash: crate::ids::OrderHash,

	/// Token id.
	pub token_id: TokenId,

	/// Side.
	#[serde(deserialize_with = "super::validation::side")]
	pub side: Side,

	/// Unsigned engine price, scaled by `price_scale`.
	pub price: u32,

	/// Unsigned engine quantity, scaled by `size_scale`.
	pub size: u64,

	/// Positive divisor for engine-native prices.
	pub price_scale: Scale,

	/// Positive divisor for engine-native quantities.
	pub size_scale: Scale,

	/// Settlement mode.
	#[serde(deserialize_with = "super::validation::settlement_mode")]
	pub settlement_mode: SettlementMode,

	/// Fee micro.
	pub fee_micro: EncodedAmount,
}

/// The engine accepted an order; acceptance is not a settlement receipt.
#[derive(Clone, Debug, serde::Deserialize)]
#[serde(try_from = "WireOrderAccepted")]
pub struct OrderAccepted {
	/// Engine sequence; filtered subscriptions do not guarantee contiguous values.
	pub sequence: u64,

	/// Order id.
	pub order_id: crate::ids::OrderId,

	/// Order hash.
	pub order_hash: crate::ids::OrderHash,

	/// Token id.
	pub token_id: TokenId,

	/// Side.
	#[serde(deserialize_with = "super::validation::side")]
	pub side: Side,

	/// Unsigned engine price, scaled by `price_scale`.
	pub price: u32,

	/// Remaining size.
	pub remaining_size: u64,

	/// Original size.
	pub original_size: u64,

	/// Positive divisor for engine-native prices.
	pub price_scale: Scale,

	/// Positive divisor for engine-native quantities.
	pub size_scale: Scale,

	/// Tif.
	#[serde(deserialize_with = "super::validation::tif")]
	pub tif: TimeInForce,
}

/// The live remainder was cancelled; previously executed fills remain separate.
#[derive(Clone, Debug, serde::Deserialize)]
pub struct OrderCancelled {
	/// Engine sequence; filtered subscriptions do not guarantee contiguous values.
	pub sequence: u64,

	/// Order id.
	pub order_id: crate::ids::OrderId,

	/// Order hash.
	pub order_hash: crate::ids::OrderHash,

	/// Token id.
	pub token_id: TokenId,

	/// Side.
	#[serde(deserialize_with = "super::validation::side")]
	pub side: Side,

	/// Unsigned engine price, scaled by `price_scale`.
	pub price: u32,

	/// Remaining size.
	pub remaining_size: u64,

	/// Positive divisor for engine-native prices.
	pub price_scale: Scale,

	/// Positive divisor for engine-native quantities.
	pub size_scale: Scale,

	/// `USER`, `FAK_REMAINDER`, `SELF_TRADE_PREVENTION`, `MARKET_RESOLVED`.
	pub reason: String,
}

/// A router-originated asynchronous order failure with canonical public metadata.
#[derive(Clone, Debug, serde::Deserialize)]
pub struct OrderRejected {
	/// Router-originated, so `sequence` is 0 — dedupe by `order_id`.
	pub sequence: u64,

	/// Order id.
	pub order_id: crate::ids::OrderId,

	/// Order hash.
	#[serde(deserialize_with = "super::validation::required_nullable")]
	pub order_hash: Option<crate::ids::OrderHash>,

	/// Token id.
	#[serde(deserialize_with = "super::validation::required_nullable")]
	pub token_id: Option<TokenId>,

	/// Failure.
	pub failure: PublicFailure,
}

/// A complete set of outcome shares was minted to the account.
#[derive(Clone, Debug, serde::Deserialize)]
#[serde(try_from = "WireTokensMinted")]
pub struct TokensMinted {
	/// Batch hash.
	pub batch_hash: Option<crate::ids::BatchHash>,

	/// Batch index.
	pub batch_index: Option<u32>,

	/// Engine sequence; filtered subscriptions do not guarantee contiguous values.
	pub sequence: u64,

	/// Condition id.
	pub condition_id: ConditionId,

	/// Unsigned engine quantity, scaled by `size_scale`.
	pub size: u64,

	/// Positive divisor for engine-native quantities.
	pub size_scale: Scale,
}

/// A complete set of outcome shares was merged back to collateral.
#[derive(Clone, Debug, serde::Deserialize)]
#[serde(try_from = "WireTokensMerged")]
pub struct TokensMerged {
	/// Batch hash.
	pub batch_hash: Option<crate::ids::BatchHash>,

	/// Batch index.
	pub batch_index: Option<u32>,

	/// Engine sequence; filtered subscriptions do not guarantee contiguous values.
	pub sequence: u64,

	/// Condition id.
	pub condition_id: ConditionId,

	/// Unsigned engine quantity, scaled by `size_scale`.
	pub size: u64,

	/// Positive divisor for engine-native quantities.
	pub size_scale: Scale,
}

#[derive(serde::Deserialize)]
struct WireTokensMinted {
	batch_hash: Option<crate::ids::BatchHash>,
	batch_index: Option<u32>,
	sequence: u64,
	condition_id: ConditionId,
	size: u64,
	size_scale: Scale,
}

#[derive(serde::Deserialize)]
struct WireTokensMerged {
	batch_hash: Option<crate::ids::BatchHash>,
	batch_index: Option<u32>,
	sequence: u64,
	condition_id: ConditionId,
	size: u64,
	size_scale: Scale,
}

#[derive(serde::Deserialize)]
struct WireMarketCreated {
	condition_id: ConditionId,
	sequence: u64,
	num_outcomes: u32,
	tick_size: u32,
	price_scale: Scale,
	size_scale: Scale,
	min_price: u32,
	max_price: u32,
	cross_match_enabled: bool,
}

#[derive(serde::Deserialize)]
struct WireOrderbookSnapshot {
	token_id: TokenId,
	sequence: u64,
	bids: Vec<Level>,
	asks: Vec<Level>,
	tick_size: u32,
	price_scale: Scale,
	size_scale: Scale,
}

#[derive(serde::Deserialize)]
struct WireOrderAccepted {
	sequence: u64,
	order_id: crate::ids::OrderId,
	order_hash: crate::ids::OrderHash,
	token_id: TokenId,

	#[serde(deserialize_with = "super::validation::side")]
	side: Side,
	price: u32,
	remaining_size: u64,
	original_size: u64,
	price_scale: Scale,
	size_scale: Scale,

	#[serde(deserialize_with = "super::validation::tif")]
	tif: TimeInForce,
}

fn provenance(
	hash: &Option<crate::ids::BatchHash>,
	index: Option<u32>,
) -> Result<(), super::FrameDecodeError> {
	if hash.is_some() != index.is_some() {
		return Err(super::FrameDecodeError::Invariant(
			super::FrameInvariant::BatchProvenance,
		));
	}

	Ok(())
}

impl TryFrom<WireTokensMinted> for TokensMinted {
	type Error = super::FrameDecodeError;

	fn try_from(raw: WireTokensMinted) -> Result<Self, Self::Error> {
		let value = Self {
			batch_hash: raw.batch_hash,
			batch_index: raw.batch_index,
			sequence: raw.sequence,
			condition_id: raw.condition_id,
			size: raw.size,
			size_scale: raw.size_scale,
		};
		provenance(&value.batch_hash, value.batch_index)?;

		Ok(value)
	}
}

impl TryFrom<WireTokensMerged> for TokensMerged {
	type Error = super::FrameDecodeError;

	fn try_from(raw: WireTokensMerged) -> Result<Self, Self::Error> {
		let value = Self {
			batch_hash: raw.batch_hash,
			batch_index: raw.batch_index,
			sequence: raw.sequence,
			condition_id: raw.condition_id,
			size: raw.size,
			size_scale: raw.size_scale,
		};
		provenance(&value.batch_hash, value.batch_index)?;

		Ok(value)
	}
}

impl TryFrom<WireMarketCreated> for MarketCreated {
	type Error = super::FrameDecodeError;

	fn try_from(raw: WireMarketCreated) -> Result<Self, Self::Error> {
		let value = Self {
			condition_id: raw.condition_id,
			sequence: raw.sequence,
			num_outcomes: raw.num_outcomes,
			tick_size: raw.tick_size,
			price_scale: raw.price_scale,
			size_scale: raw.size_scale,
			min_price: raw.min_price,
			max_price: raw.max_price,
			cross_match_enabled: raw.cross_match_enabled,
		};
		if value.tick_size == 0 {
			return Err(super::FrameDecodeError::Invariant(
				super::FrameInvariant::PositiveTick,
			));
		}
		if value.min_price > value.max_price {
			return Err(super::FrameDecodeError::Invariant(
				super::FrameInvariant::PriceRange,
			));
		}
		if value.num_outcomes < 2 {
			return Err(super::FrameDecodeError::Invariant(
				super::FrameInvariant::OutcomeCount,
			));
		}

		Ok(value)
	}
}

impl TryFrom<WireOrderbookSnapshot> for OrderbookSnapshot {
	type Error = super::FrameDecodeError;

	fn try_from(raw: WireOrderbookSnapshot) -> Result<Self, Self::Error> {
		let value = Self {
			token_id: raw.token_id,
			sequence: raw.sequence,
			bids: raw.bids,
			asks: raw.asks,
			tick_size: raw.tick_size,
			price_scale: raw.price_scale,
			size_scale: raw.size_scale,
		};
		if value.tick_size == 0 {
			return Err(super::FrameDecodeError::Invariant(
				super::FrameInvariant::PositiveTick,
			));
		}

		Ok(value)
	}
}

impl TryFrom<WireOrderAccepted> for OrderAccepted {
	type Error = super::FrameDecodeError;

	fn try_from(raw: WireOrderAccepted) -> Result<Self, Self::Error> {
		let value = Self {
			sequence: raw.sequence,
			order_id: raw.order_id,
			order_hash: raw.order_hash,
			token_id: raw.token_id,
			side: raw.side,
			price: raw.price,
			remaining_size: raw.remaining_size,
			original_size: raw.original_size,
			price_scale: raw.price_scale,
			size_scale: raw.size_scale,
			tif: raw.tif,
		};
		if value.remaining_size > value.original_size {
			return Err(super::FrameDecodeError::Invariant(
				super::FrameInvariant::RemainingQuantity,
			));
		}

		Ok(value)
	}
}

impl<'de> serde::Deserialize<'de> for Level {
	fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		#[derive(serde::Deserialize)]
		#[serde(untagged)]
		enum Wire {
			Pair((u32, u64)),
			Object { price: u32, size: u64 },
		}

		match Wire::deserialize(deserializer)? {
			Wire::Pair((price, size)) | Wire::Object { price, size } => Ok(Self { price, size }),
		}
	}
}

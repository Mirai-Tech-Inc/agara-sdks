use super::{EncodedAmount, Scale};

use crate::{
	ids::{ConditionId, EventId, MarketId, TokenId},
	values::Timestamp,
};

/// Current market lifecycle states accepted by the platform producer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LifecycleState {
	/// Market draft.
	Draft,

	/// Prepared future market.
	Provisioned,

	/// Current tradable market.
	Active,

	/// Inactive market.
	Inactive,

	/// Trading is halted.
	TradingHalt,

	/// An outcome proposal is pending.
	ProposalPending,

	/// An outcome has been proposed.
	Proposed,

	/// The proposal is disputed.
	Disputed,

	/// Final resolved outcome.
	Resolved,

	/// Voided market.
	Void,

	/// Archived market.
	Archived,
}

/// Replacement relation supplied after market resolution.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReplacementStatus {
	/// Replacement is the active main market.
	Active,

	/// Replacement is provisioned for a future cycle.
	Upcoming,

	/// A replacement has not yet been created; a cycle is supplied.
	Expected,

	/// No replacement is expected.
	None,
}

/// Resolved tokens were redeemed, with exact unsigned quantities and payout.
#[derive(Clone, Debug, serde::Deserialize)]
#[serde(try_from = "WireTokensRedeemed")]
pub struct TokensRedeemed {
	/// Engine sequence; filtered subscriptions do not guarantee contiguous values.
	pub sequence: u64,

	/// Condition id.
	pub condition_id: ConditionId,

	/// Winning token id.
	pub winning_token_id: TokenId,

	/// Losing token id.
	pub losing_token_id: TokenId,

	/// Winning shares redeemed.
	pub winning_shares_redeemed: EncodedAmount,

	/// Losing shares zeroed.
	pub losing_shares_zeroed: EncodedAmount,

	/// Payout micro.
	pub payout_micro: EncodedAmount,

	/// Positive divisor for engine-native quantities.
	pub size_scale: Scale,

	/// Batch hash.
	pub batch_hash: Option<crate::ids::BatchHash>,

	/// Batch index.
	pub batch_index: Option<u32>,
}

/// Exact collateral movement and resulting account balance.
#[derive(Clone, Debug, serde::Deserialize)]
#[serde(try_from = "WireCollateralChanged")]
pub struct CollateralChanged {
	/// Engine sequence; filtered subscriptions do not guarantee contiguous values.
	pub sequence: u64,

	/// Amount micro.
	pub amount_micro: EncodedAmount,

	/// Cash balance micro.
	pub cash_balance_micro: EncodedAmount,

	/// Batch hash.
	pub batch_hash: Option<crate::ids::BatchHash>,

	/// Batch index.
	pub batch_index: Option<u32>,
}

/// One outcome displayed by event lifecycle metadata.
#[derive(Clone, Debug, serde::Deserialize)]
pub struct LifecycleOutcome {
	/// Token id.
	pub token_id: TokenId,

	/// Kind.
	pub kind: String,

	/// Label.
	pub label: String,

	/// Short label.
	#[serde(deserialize_with = "super::validation::required_nullable")]
	pub short_label: Option<String>,
}

/// A market and at least two outcomes described by event lifecycle metadata.
#[derive(Clone, Debug, serde::Deserialize)]
#[serde(try_from = "WireLifecycleMarket")]
pub struct LifecycleMarket {
	/// Market id.
	pub market_id: MarketId,

	/// Condition id.
	pub condition_id: ConditionId,

	/// State.
	pub state: LifecycleState,

	/// Reference price.
	pub reference_price: Option<String>,

	/// Currency.
	pub currency: Option<String>,

	/// Reference at.
	pub reference_at: Option<Timestamp>,

	/// Observe at.
	pub observe_at: Option<Timestamp>,

	/// Outcomes.
	pub outcomes: Vec<LifecycleOutcome>,
}

/// The event's active main market changed; this has no engine sequence.
#[derive(Clone, Debug, serde::Deserialize)]
#[serde(try_from = "WireCurrentMarketChanged")]
pub struct CurrentMarketChanged {
	/// Event id.
	pub event_id: EventId,

	/// Changed at.
	pub changed_at: Timestamp,

	/// Main market id.
	pub main_market_id: MarketId,

	/// Market.
	pub market: LifecycleMarket,
}

/// Reference and observation instants for an expected future market cycle.
#[derive(Clone, Debug, serde::Deserialize)]
pub struct ExpectedCycle {
	/// Reference at.
	pub reference_at: Timestamp,

	/// Observe at.
	pub observe_at: Timestamp,
}

/// Resolution plus a validated active, upcoming, expected or absent replacement.
#[derive(Clone, Debug, serde::Deserialize)]
#[serde(try_from = "WireMarketResolutionCompleted")]
pub struct MarketResolutionCompleted {
	/// Event id.
	pub event_id: EventId,

	/// Main market id.
	pub main_market_id: Option<MarketId>,

	/// Resolved market id.
	pub resolved_market_id: MarketId,

	/// Resolved condition id.
	pub resolved_condition_id: ConditionId,

	/// Winning token id.
	pub winning_token_id: TokenId,

	/// Resolved at.
	pub resolved_at: Timestamp,

	/// Replacement status.
	pub replacement_status: ReplacementStatus,

	/// Replacement.
	pub replacement: Option<LifecycleMarket>,

	/// Expected cycle.
	pub expected_cycle: Option<ExpectedCycle>,
}

#[derive(serde::Deserialize)]
struct WireTokensRedeemed {
	sequence: u64,
	condition_id: ConditionId,
	winning_token_id: TokenId,
	losing_token_id: TokenId,
	winning_shares_redeemed: EncodedAmount,
	losing_shares_zeroed: EncodedAmount,
	payout_micro: EncodedAmount,
	size_scale: Scale,
	batch_hash: Option<crate::ids::BatchHash>,
	batch_index: Option<u32>,
}

#[derive(serde::Deserialize)]
struct WireCollateralChanged {
	sequence: u64,
	amount_micro: EncodedAmount,
	cash_balance_micro: EncodedAmount,
	batch_hash: Option<crate::ids::BatchHash>,
	batch_index: Option<u32>,
}

#[derive(serde::Deserialize)]
struct WireLifecycleMarket {
	market_id: MarketId,
	condition_id: ConditionId,
	state: LifecycleState,

	#[serde(deserialize_with = "super::validation::required_nullable")]
	reference_price: Option<String>,

	#[serde(deserialize_with = "super::validation::required_nullable")]
	currency: Option<String>,

	#[serde(deserialize_with = "super::validation::required_nullable")]
	reference_at: Option<Timestamp>,

	#[serde(deserialize_with = "super::validation::required_nullable")]
	observe_at: Option<Timestamp>,
	outcomes: Vec<LifecycleOutcome>,
}

#[derive(serde::Deserialize)]
struct WireCurrentMarketChanged {
	event_id: EventId,
	changed_at: Timestamp,
	main_market_id: MarketId,
	market: LifecycleMarket,
}

#[derive(serde::Deserialize)]
struct WireMarketResolutionCompleted {
	event_id: EventId,

	#[serde(deserialize_with = "super::validation::required_nullable")]
	main_market_id: Option<MarketId>,
	resolved_market_id: MarketId,
	resolved_condition_id: ConditionId,
	winning_token_id: TokenId,
	resolved_at: Timestamp,
	replacement_status: ReplacementStatus,

	#[serde(deserialize_with = "super::validation::required_nullable")]
	replacement: Option<LifecycleMarket>,
	expected_cycle: Option<ExpectedCycle>,
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

fn validate_resolution(value: &MarketResolutionCompleted) -> Result<(), super::FrameDecodeError> {
	let valid = match (
		value.replacement_status,
		&value.replacement,
		&value.expected_cycle,
	) {
		(ReplacementStatus::Active, Some(market), None) => {
			market.state == LifecycleState::Active
				&& value.main_market_id.as_ref() == Some(&market.market_id)
				&& market.market_id != value.resolved_market_id
		},
		(ReplacementStatus::Upcoming, Some(market), None) => {
			market.state == LifecycleState::Provisioned
				&& market.market_id != value.resolved_market_id
		},
		(ReplacementStatus::Expected, None, Some(_)) | (ReplacementStatus::None, None, None) => {
			true
		},
		_ => false,
	};
	if !valid {
		return Err(super::FrameDecodeError::Invariant(
			super::FrameInvariant::ResolutionReplacement,
		));
	}

	Ok(())
}

impl TryFrom<WireTokensRedeemed> for TokensRedeemed {
	type Error = super::FrameDecodeError;

	fn try_from(raw: WireTokensRedeemed) -> Result<Self, Self::Error> {
		let value = Self {
			sequence: raw.sequence,
			condition_id: raw.condition_id,
			winning_token_id: raw.winning_token_id,
			losing_token_id: raw.losing_token_id,
			winning_shares_redeemed: raw.winning_shares_redeemed,
			losing_shares_zeroed: raw.losing_shares_zeroed,
			payout_micro: raw.payout_micro,
			size_scale: raw.size_scale,
			batch_hash: raw.batch_hash,
			batch_index: raw.batch_index,
		};
		provenance(&value.batch_hash, value.batch_index)?;

		Ok(value)
	}
}

impl TryFrom<WireCollateralChanged> for CollateralChanged {
	type Error = super::FrameDecodeError;

	fn try_from(raw: WireCollateralChanged) -> Result<Self, Self::Error> {
		let value = Self {
			sequence: raw.sequence,
			amount_micro: raw.amount_micro,
			cash_balance_micro: raw.cash_balance_micro,
			batch_hash: raw.batch_hash,
			batch_index: raw.batch_index,
		};
		provenance(&value.batch_hash, value.batch_index)?;

		Ok(value)
	}
}

impl TryFrom<WireLifecycleMarket> for LifecycleMarket {
	type Error = super::FrameDecodeError;

	fn try_from(raw: WireLifecycleMarket) -> Result<Self, Self::Error> {
		let value = Self {
			market_id: raw.market_id,
			condition_id: raw.condition_id,
			state: raw.state,
			reference_price: raw.reference_price,
			currency: raw.currency,
			reference_at: raw.reference_at,
			observe_at: raw.observe_at,
			outcomes: raw.outcomes,
		};
		if value.outcomes.iter().any(|outcome| outcome.kind.is_empty()) {
			return Err(super::FrameDecodeError::Invariant(
				super::FrameInvariant::OutcomeKind,
			));
		}
		if value.outcomes.len() < 2 {
			return Err(super::FrameDecodeError::Invariant(
				super::FrameInvariant::OutcomeCount,
			));
		}
		if value
			.reference_price
			.as_ref()
			.is_some_and(|price| !super::validation::decimal_text(price))
		{
			return Err(super::FrameDecodeError::Invariant(
				super::FrameInvariant::ReferencePrice,
			));
		}

		Ok(value)
	}
}

impl TryFrom<WireCurrentMarketChanged> for CurrentMarketChanged {
	type Error = super::FrameDecodeError;

	fn try_from(raw: WireCurrentMarketChanged) -> Result<Self, Self::Error> {
		let value = Self {
			event_id: raw.event_id,
			changed_at: raw.changed_at,
			main_market_id: raw.main_market_id,
			market: raw.market,
		};
		if value.market.state != LifecycleState::Active {
			return Err(super::FrameDecodeError::Invariant(
				super::FrameInvariant::CurrentMarketState,
			));
		}
		if value.market.market_id != value.main_market_id {
			return Err(super::FrameDecodeError::Invariant(
				super::FrameInvariant::CurrentMarketIdentity,
			));
		}

		Ok(value)
	}
}

impl TryFrom<WireMarketResolutionCompleted> for MarketResolutionCompleted {
	type Error = super::FrameDecodeError;

	fn try_from(raw: WireMarketResolutionCompleted) -> Result<Self, Self::Error> {
		let value = Self {
			event_id: raw.event_id,
			main_market_id: raw.main_market_id,
			resolved_market_id: raw.resolved_market_id,
			resolved_condition_id: raw.resolved_condition_id,
			winning_token_id: raw.winning_token_id,
			resolved_at: raw.resolved_at,
			replacement_status: raw.replacement_status,
			replacement: raw.replacement,
			expected_cycle: raw.expected_cycle,
		};
		validate_resolution(&value)?;

		Ok(value)
	}
}

//! Current portfolio activity variants and metadata.

use crate::{
	ids::{ConditionId, Exchange, OrderId, OrderStatus, Side, TokenId},
	units::Micro,
};

/// Whether recorded fills cover the entire order quantity or only part of it.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PortfolioOrderFillStatus {
	/// Full order quantity filled.
	Full,

	/// Partial order quantity filled.
	Partial,
}

/// Current AGARA account-history variants, discriminated by the wire `type` field.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Activity {
	/// Order with at least one recorded fill.
	Order(PortfolioOrderActivity),

	/// Complete-set split.
	Split(PortfolioSplitActivity),

	/// Complete-set merge.
	Merge(PortfolioMergeActivity),

	/// Resolved outcome redemption.
	Redeem(PortfolioRedeemActivity),

	/// External collateral deposit.
	Deposit(PortfolioDepositActivity),

	/// External collateral withdrawal.
	Withdrawal(PortfolioWithdrawalActivity),

	/// Confirmed liquidity-provider reward payout.
	LpPayout(PortfolioLpPayoutActivity),
}

/// Aggregate recorded fills for one order, including fees and current order status.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]

pub struct PortfolioOrderActivity {
	/// Exchange that owns this market, account balance or trading record.
	pub exchange: Exchange,

	/// Namespaced activity identifier; distinct from the order or transaction ID.
	pub id: String,

	/// Internal order UUID used by order reads and cancellation.
	pub order_id: OrderId,

	/// Current order status.
	pub status: OrderStatus,

	/// Whether the order filled completely or partially.
	pub fill_status: PortfolioOrderFillStatus,

	/// On-chain condition identifier used by position and settlement operations. None when
	/// unavailable.
	pub condition_id: Option<ConditionId>,

	/// On-chain outcome token identifier used by trading and quote subscriptions. None when
	/// unavailable.
	pub token_id: Option<TokenId>,

	/// Buy or sell direction for this order or fill.
	pub side: Side,

	/// Total filled quantity in micro shares.
	pub filled_shares_micro: Micro,

	/// Filled-share weighted price in micro collateral per share.
	pub average_fill_price_micro: Micro,

	/// Total filled notional in micro collateral.
	pub filled_amount_micro: Micro,

	/// Sum of fill fees in micro collateral.
	pub fees_micro: Micro,

	/// RFC3339 timestamp of the earliest recorded fill in this activity.
	pub created_at: String,
}

/// A confirmed collateral split that minted equal quantities of both outcomes.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]

pub struct PortfolioSplitActivity {
	/// Exchange that owns this market, account balance or trading record.
	pub exchange: Exchange,

	/// Namespaced activity identifier; distinct from the order or transaction ID.
	pub id: String,

	/// On-chain condition identifier used by position and settlement operations.
	pub condition_id: ConditionId,

	/// Micro shares minted for each outcome leg.
	pub shares_micro: Micro,

	/// Micro collateral consumed by the split.
	pub amount_micro: Micro,

	/// On-chain transaction hash confirming this activity.
	pub tx_hash: String,

	/// RFC3339 creation or recognition timestamp for this record.
	pub created_at: String,
}

/// A confirmed complete-set merge that released collateral.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]

pub struct PortfolioMergeActivity {
	/// Exchange that owns this market, account balance or trading record.
	pub exchange: Exchange,

	/// Namespaced activity identifier; distinct from the order or transaction ID.
	pub id: String,

	/// On-chain condition identifier used by position and settlement operations.
	pub condition_id: ConditionId,

	/// Micro shares burned from each outcome leg.
	pub shares_micro: Micro,

	/// Micro collateral released by the merge.
	pub amount_micro: Micro,

	/// On-chain transaction hash confirming this activity.
	pub tx_hash: String,

	/// RFC3339 creation or recognition timestamp for this record.
	pub created_at: String,
}

/// A confirmed winning-outcome redemption and its collateral proceeds.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]

pub struct PortfolioRedeemActivity {
	/// Exchange that owns this market, account balance or trading record.
	pub exchange: Exchange,

	/// Namespaced activity identifier; distinct from the order or transaction ID.
	pub id: String,

	/// On-chain condition identifier used by position and settlement operations.
	pub condition_id: ConditionId,

	/// On-chain outcome token identifier used by trading and quote subscriptions.
	pub token_id: TokenId,

	/// Winning outcome quantity redeemed, in micro shares.
	pub shares_micro: Micro,

	/// Redemption proceeds in micro collateral.
	pub amount_micro: Micro,

	/// On-chain transaction hash confirming this activity.
	pub tx_hash: String,

	/// RFC3339 creation or recognition timestamp for this record.
	pub created_at: String,
}

/// An external collateral transfer into the trading wallet.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]

pub struct PortfolioDepositActivity {
	/// Exchange that owns this market, account balance or trading record.
	pub exchange: Exchange,

	/// Namespaced activity identifier; distinct from the order or transaction ID.
	pub id: String,

	/// Positive deposited quantity in micro collateral; the activity type supplies the direction.
	pub amount_micro: Micro,

	/// Chain the transfer settled on.
	pub chain_id: i64,

	/// Lowercased collateral token address.
	pub token_address: String,

	/// On-chain transfer peer.
	pub counterparty_address: String,

	/// On-chain transaction hash confirming this activity.
	pub tx_hash: String,

	/// RFC3339 creation or recognition timestamp for this record.
	pub created_at: String,
}

/// An external collateral transfer out of the trading wallet.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]

pub struct PortfolioWithdrawalActivity {
	/// Exchange that owns this market, account balance or trading record.
	pub exchange: Exchange,

	/// Namespaced activity identifier; distinct from the order or transaction ID.
	pub id: String,

	/// Positive withdrawn quantity in micro collateral; the activity type supplies the direction.
	pub amount_micro: Micro,

	/// Chain the transfer settled on.
	pub chain_id: i64,

	/// Lowercased collateral token address.
	pub token_address: String,

	/// On-chain transfer peer.
	pub counterparty_address: String,

	/// On-chain transaction hash confirming this activity.
	pub tx_hash: String,

	/// RFC3339 creation or recognition timestamp for this record.
	pub created_at: String,
}

/// A confirmed liquidity-provider reward payout for one Eastern reward day.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]

pub struct PortfolioLpPayoutActivity {
	/// Exchange that owns this market, account balance or trading record.
	pub exchange: Exchange,

	/// Namespaced activity identifier; distinct from the order or transaction ID.
	pub id: String,

	/// Confirmed liquidity-provider payout in micro collateral.
	pub amount_micro: Micro,

	/// Reward date as YYYY-MM-DD in America/New_York.
	pub epoch_date: String,

	/// On-chain transaction hash; None until available.
	pub tx_hash: Option<String>,

	/// RFC3339 confirmation time of the reward payout.
	pub created_at: String,
}

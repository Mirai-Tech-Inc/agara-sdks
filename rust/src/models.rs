//! Wire request and response types. Field names and casing mirror the
//! router's JSON exactly; micro amounts use [`Micro`], timestamps are
//! RFC3339 strings.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::ids::{
	ConditionId, Exchange, FillRole, OrderHash, OrderId, OrderStatus, OrderType, PendingOperation,
	PositionOperation, RelayerState, Side, TimeInForce, TokenId, WalletId,
};
use crate::units::Micro;

/// Per-token display metadata returned alongside orders / positions.
#[derive(Clone, Debug, Deserialize)]
pub struct MarketMeta {
	pub market_title: String,
	pub outcome_name: String,
	pub logo_url: Option<String>,
	pub event_slug: String,
}

/// Map of token id → [`MarketMeta`], the `markets` sidecar on list
/// responses.
pub type Markets = HashMap<TokenId, MarketMeta>;

/// Keyset-pagination cursor envelope. Treat `next_cursor` as opaque and
/// keep re-requesting until it comes back `None`.
#[derive(Clone, Debug, Deserialize)]
pub struct CursorPagination {
	pub next_cursor: Option<String>,
	pub limit: u32,
}

// ── Orders ───────────────────────────────────────────────────────────

/// The ack returned when an order is accepted. Treat it as "we got it,"
/// not as a fill.
#[derive(Clone, Debug, Deserialize)]
pub struct CreateClobOrderResponse {
	pub order_id: OrderId,
	pub source: Exchange,
	pub status: OrderStatus,
	pub pending_operation: PendingOperation,
	pub as_of: String,
}

/// One entry in a signed-order batch response — accepted or rejected,
/// independently of the other entries.
#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum SignedOrderResult {
	Accepted {
		index: u32,
		#[serde(flatten)]
		ack: CreateClobOrderResponse,
	},
	Rejected {
		index: u32,
		code: String,
		message: String,
	},
}

/// Response to a signed-order batch submission.
#[derive(Clone, Debug, Deserialize)]
pub struct SignedOrderBatchResponse {
	pub results: Vec<SignedOrderResult>,
	pub as_of: String,
}

/// A full order record.
#[derive(Clone, Debug, Deserialize)]
pub struct Order {
	pub internal_id: OrderId,
	pub exchange: Exchange,
	pub token_id: TokenId,
	pub condition_id: Option<ConditionId>,
	pub side: Side,
	#[serde(rename = "type")]
	pub order_type: OrderType,
	pub price_micro: Option<Micro>,
	pub original_size_micro: Option<Micro>,
	pub collateral_amount_micro: Option<Micro>,
	pub size_matched_micro: Micro,
	pub avg_fill_price_micro: Option<Micro>,
	pub status: OrderStatus,
	pub error: Option<String>,
	pub expiration: String,
	pub created_at: String,
	pub cancel_requested_at: Option<String>,
}

/// A single order plus its display metadata.
#[derive(Clone, Debug, Deserialize)]
pub struct OrderResponse {
	pub order: Order,
	#[serde(default)]
	pub markets: Markets,
}

/// A page of orders (open and terminal), newest-first.
#[derive(Clone, Debug, Deserialize)]
pub struct OrdersListResponse {
	pub orders: Vec<Order>,
	#[serde(default)]
	pub markets: Markets,
	pub pagination: CursorPagination,
	pub as_of: String,
}

/// One fill (a leg of a trade), from an order's perspective.
#[derive(Clone, Debug, Deserialize)]
pub struct Fill {
	pub exchange: Exchange,
	pub trade_id: String,
	pub fill_id: Option<String>,
	pub order_id: Option<OrderId>,
	pub token_id: TokenId,
	pub side: Side,
	pub shares_micro: Micro,
	pub price_micro: Micro,
	pub fee_micro: Micro,
	pub role: Option<FillRole>,
	pub status: String,
	pub transaction_hash: Option<String>,
	pub executed_at: String,
}

/// The fills for one order.
#[derive(Clone, Debug, Deserialize)]
pub struct OrderTradesResponse {
	pub trades: Vec<Fill>,
	pub as_of: String,
}

/// Ack for a single cancel.
#[derive(Clone, Debug, Deserialize)]
pub struct CancelOrderResponse {
	pub order_id: OrderId,
	pub pending_operation: PendingOperation,
	pub as_of: String,
}

/// Ack for cancel-all.
#[derive(Clone, Debug, Deserialize)]
pub struct CancelAllOrdersResponse {
	pub wallet_ids: Vec<WalletId>,
	pub pending_operation: PendingOperation,
	pub as_of: String,
}

// ── Orderbook ────────────────────────────────────────────────────────

/// One price level in the REST orderbook — dollars and shares, not
/// micro-units.
#[derive(Clone, Copy, Debug, Deserialize)]
pub struct OrderbookLevel {
	pub price: f64,
	pub size: f64,
}

/// A REST orderbook snapshot for one outcome.
#[derive(Clone, Debug, Deserialize)]
pub struct Orderbook {
	pub bids: Vec<OrderbookLevel>,
	pub asks: Vec<OrderbookLevel>,
	pub timestamp: String,
	pub hash: String,
	pub tick_size: String,
}

impl Orderbook {
	/// Highest bid price, if any.
	pub fn best_bid(&self) -> Option<f64> {
		self.bids.first().map(|l| l.price)
	}

	/// Lowest ask price, if any.
	pub fn best_ask(&self) -> Option<f64> {
		self.asks.first().map(|l| l.price)
	}

	/// Midpoint between best bid and ask, if both exist.
	pub fn mid(&self) -> Option<f64> {
		match (self.best_bid(), self.best_ask()) {
			(Some(b), Some(a)) => Some((b + a) / 2.0),
			_ => None,
		}
	}

	/// Ask − bid, if both exist.
	pub fn spread(&self) -> Option<f64> {
		match (self.best_bid(), self.best_ask()) {
			(Some(b), Some(a)) => Some(a - b),
			_ => None,
		}
	}
}

// ── Portfolio ────────────────────────────────────────────────────────

/// Per-exchange balance + positions value + open commitments.
#[derive(Clone, Debug, Deserialize)]
pub struct PortfolioSummaryEntry {
	pub exchange: Exchange,
	pub cash_balance_micro: Micro,
	pub free_cash_micro: Micro,
	pub positions_value_micro: Micro,
	pub portfolio_value_micro: Micro,
	pub open_cost_basis_micro: Micro,
	pub open_unrealized_pnl_micro: Micro,
	pub as_of: String,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct PortfolioSummaryResponse {
	pub summaries: Vec<PortfolioSummaryEntry>,
}

/// One current position.
#[derive(Clone, Debug, Deserialize)]
pub struct Position {
	pub exchange: Exchange,
	pub condition_id: ConditionId,
	pub token_id: TokenId,
	pub shares_micro: Micro,
	pub avg_price_micro: Micro,
	pub current_price_micro: Micro,
	pub current_value_micro: Micro,
	pub to_win_micro: Micro,
	pub profit_loss_micro: Micro,
	pub profit_loss_percent: String,
	pub redeemable: bool,
	pub mergeable: bool,
	pub mergeable_shares_micro: Option<Micro>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct PositionsResponse {
	pub positions: Vec<Position>,
	#[serde(default)]
	pub unavailable_exchanges: Vec<Exchange>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct OpenOrdersResponse {
	pub orders: Vec<Order>,
	pub pagination: CursorPagination,
}

/// A page of recent fills, newest-first.
#[derive(Clone, Debug, Deserialize)]
pub struct TradesResponse {
	pub trades: Vec<Fill>,
	#[serde(default)]
	pub markets: Markets,
	pub pagination: CursorPagination,
	#[serde(default)]
	pub unavailable_exchanges: Vec<Exchange>,
	pub as_of: String,
}

/// One activity-feed row.
#[derive(Clone, Debug, Deserialize)]
pub struct Activity {
	pub exchange: Exchange,
	pub id: String,
	#[serde(rename = "type")]
	pub activity_type: String,
	pub condition_id: ConditionId,
	pub token_id: TokenId,
	pub side: Side,
	pub shares_micro: Micro,
	pub price_micro: Micro,
	pub amount_micro: Micro,
	pub realized_pnl_micro: Option<Micro>,
	pub occurred_at: String,
}

/// A page of activity-feed rows, newest-first.
#[derive(Clone, Debug, Deserialize)]
pub struct ActivitiesResponse {
	pub activities: Vec<Activity>,
	#[serde(default)]
	pub markets: Markets,
	pub pagination: CursorPagination,
	#[serde(default)]
	pub unavailable_exchanges: Vec<Exchange>,
	pub as_of: String,
}

/// Receipt for a split / merge / redeem.
#[derive(Clone, Debug, Deserialize)]
pub struct PositionOperationResponse {
	pub operation: PositionOperation,
	pub condition_id: ConditionId,
	pub relayer_transaction_id: String,
	pub transaction_hash: Option<String>,
	pub relayer_state: RelayerState,
	pub as_of: String,
}

/// Pending incentive balances, in micro-USDC.
#[derive(Clone, Debug, Deserialize)]
pub struct RebatesSummaryResponse {
	pub maker_rebate_micro: Micro,
	pub vip_rebate_micro: Micro,
	pub lp_incentive_micro: Micro,
	pub total_micro: Micro,
}

/// One market's LP-incentive standing.
#[derive(Clone, Debug, Deserialize)]
pub struct LpIncentiveMarket {
	pub market_id: String,
	pub question: String,
	pub event_slug: String,
	pub event_title: String,
	pub close_time: Option<String>,
	pub pool_micro: Micro,
	pub epoch_score: f64,
	pub market_total_score: f64,
	pub projected_payout_micro: Micro,
}

/// LP-incentive eligibility and per-market standing.
#[derive(Clone, Debug, Deserialize)]
pub struct LpIncentivesResponse {
	pub eligible: bool,
	pub markets: Vec<LpIncentiveMarket>,
}

/// Public market/event counts.
#[derive(Clone, Copy, Debug, Deserialize)]
pub struct StatusResponse {
	pub markets: i64,
	pub events: i64,
}

// ── Request bodies ───────────────────────────────────────────────────

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
	pub token_id: TokenId,
	pub side: Side,
	#[serde(rename = "type")]
	pub order_type: OrderType,
	pub time_in_force: TimeInForce,
	pub price_micro: Micro,
	pub shares_micro: Micro,
	pub post_only: bool,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub expiration_unix_seconds: Option<i64>,
	pub order_hash: OrderHash,
	pub signature: String,
	pub salt: String,
	pub maker: String,
	pub signer: String,
	pub chain_token_id: String,
	pub maker_amount: String,
	pub taker_amount: String,
	pub side_u8: u8,
	pub signature_type: u8,
	pub timestamp: String,
	pub metadata: String,
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

#[derive(Debug, Serialize)]
pub(crate) struct OpenOrdersListRequest {
	pub token_ids: Vec<TokenId>,
	pub exchanges: Vec<Exchange>,
	pub limit: u32,
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

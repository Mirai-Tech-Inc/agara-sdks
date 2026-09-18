//! Wire request and response types. Field names and casing mirror the
//! router's JSON exactly; micro amounts use [`Micro`], timestamps are
//! RFC3339 strings.

mod requests;

#[cfg(feature = "signing")]
pub(crate) use requests::limit_collateral;
pub(crate) use requests::{
	CreateOrderRequest, MergeRequest, OrdersListRequest, PositionsListRequest,
	SignedOrderBatchRequest, SplitRequest,
};

pub use requests::{OpenOrdersListRequest, OrderField, OrderValidationError, SignedOrderRequest};

pub use crate::activity::Activity;

pub use crate::incentives::{LpIncentiveMarket, LpIncentivesResponse};

use std::collections::HashMap;

use serde::{Deserialize, Deserializer, Serialize};

use crate::{
	ids::{
		ConditionId, Exchange, FillRole, MarketId, OrderId, OrderStatus, OrderType,
		PendingOperation, PositionOperation, RelayerState, Side, TokenId, WalletId,
	},
	problem::PublicFailure,
	units::Micro,
	values::Timestamp,
};

/// Map of token id → [`MarketMeta`], the `markets` sidecar on list
/// responses.
pub type Markets = HashMap<TokenId, MarketMeta>;

/// Per-token display metadata returned alongside orders / positions.
#[derive(Clone, Debug, Deserialize)]
pub struct MarketMeta {
	/// Stable Agara market UUID.
	pub market_id: MarketId,

	/// Current market/event lifecycle state reported by discovery.
	pub state: String,

	/// Shared negative-risk group identifier; None for an ungrouped market.
	pub neg_risk_id: Option<String>,

	/// Structured presentation metadata; it is not authoritative order or settlement state.
	pub display: MarketMetaDisplay,

	/// Human-readable question or title of the parent market.
	pub market_title: String,

	/// Human-readable name of the selected outcome.
	pub outcome_name: String,

	/// Artwork URL; None when no logo is available.
	pub logo_url: Option<String>,

	/// Parent event’s stable slug for discovery lookups and links.
	pub event_slug: String,
}

/// Keyset-pagination cursor envelope. Treat `next_cursor` as opaque and
/// keep re-requesting until it comes back `None`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CursorPagination {
	/// Opaque continuation token; None ends the walk even if the last page is empty.
	pub next_cursor: Option<String>,

	/// Applied page size or requested upper bound on returned entries.
	pub limit: u32,
}

/// The ack returned when an order is accepted. Treat it as "we got it,"
/// not as a fill.
#[derive(Clone, Debug, Deserialize)]
pub struct CreateClobOrderResponse {
	/// Internal order UUID used by order reads and cancellation.
	pub order_id: OrderId,

	/// Exchange whose wallet accepted this order for asynchronous submission.
	pub source: Exchange,

	/// Initial local order state after acceptance; not proof of a fill.
	pub status: OrderStatus,

	/// Asynchronous operation accepted by the server, not proof of completion.
	pub pending_operation: PendingOperation,

	/// RFC3339 timestamp at which this response or projection was observed.
	pub as_of: Timestamp,
}

/// One entry in a signed-order batch response — accepted or rejected,
/// independently of the other entries.
#[derive(Clone, Debug)]
pub enum SignedOrderResult {
	/// This indexed order was accepted for asynchronous submission.
	Accepted {
		/// Zero-based position of this entry in the submitted order batch.
		index: u32,

		/// Accepted-order acknowledgement; matching and settlement are still asynchronous.
		ack: CreateClobOrderResponse,
	},

	/// This indexed order failed validation without rejecting the other batch entries.
	Rejected {
		/// Zero-based position of this entry in the submitted order batch.
		index: u32,

		/// Canonical public failure metadata; do not interpret its display text as a code.
		failure: PublicFailure,
	},
}

/// Response to a signed-order batch submission.
#[derive(Clone, Debug, Deserialize)]
pub struct SignedOrderBatchResponse {
	/// Per-order results in submission order, each retaining its original index.
	pub results: Vec<SignedOrderResult>,

	/// RFC3339 timestamp at which this response or projection was observed.
	pub as_of: Timestamp,
}

/// A full order record.
#[derive(Clone, Debug, Deserialize)]
pub struct Order {
	/// Internal order UUID; distinct from an exchange-side order identifier.
	pub internal_id: OrderId,

	/// Exchange that owns this market, account balance or trading record.
	pub exchange: Exchange,

	/// On-chain outcome token identifier used by trading and quote subscriptions.
	pub token_id: TokenId,

	/// On-chain condition identifier used by position and settlement operations. None when
	/// unavailable.
	pub condition_id: Option<ConditionId>,

	/// Buy or sell direction for this order or fill.
	pub side: Side,

	/// LIMIT or MARKET execution request kind.
	#[serde(rename = "type")]
	pub order_type: OrderType,

	/// Limit price in micro collateral per share; None for a market order.
	pub price_micro: Option<Micro>,

	/// Original order quantity in micro shares; may be absent for collateral-sized orders.
	pub original_size_micro: Option<Micro>,

	/// Market-buy budget in micro collateral; None for a share-sized order.
	pub collateral_amount_micro: Option<Micro>,

	/// Cumulative filled quantity in micro shares.
	pub size_matched_micro: Micro,

	/// Filled-share weighted price in micro collateral per share; None before a fill.
	pub avg_fill_price_micro: Option<Micro>,

	/// Order display state; only is_terminal establishes order completion.
	pub status: OrderStatus,

	/// Authoritative order completion flag; display status alone cannot establish completion.
	pub is_terminal: bool,

	#[serde(
		default,
		alias = "error",
		deserialize_with = "deserialize_optional_failure"
	)]
	/// Canonical terminal-failure metadata; None when the order has no public failure.
	pub failure: Option<PublicFailure>,

	/// Order expiration timestamp in RFC3339 format.
	pub expiration: Timestamp,

	/// RFC3339 creation or recognition timestamp for this record.
	pub created_at: Timestamp,

	/// RFC3339 time cancellation was requested; None when no intent is recorded.
	pub cancel_requested_at: Option<Timestamp>,
}

/// A single order plus its display metadata.
#[derive(Clone, Debug, Deserialize)]
pub struct OrderResponse {
	/// Current order record, including authoritative terminality and any public failure.
	pub order: Order,

	/// Market metadata keyed by outcome token ID.
	pub markets: Markets,
}

/// A page of orders (open and terminal), newest-first.
#[derive(Clone, Debug, Deserialize)]
pub struct OrdersListResponse {
	/// Order rows for this response page.
	pub orders: Vec<Order>,

	/// Market metadata keyed by outcome token ID.
	pub markets: Markets,

	/// Applied page size and opaque continuation token for the next request.
	pub pagination: CursorPagination,

	/// RFC3339 timestamp at which this response or projection was observed.
	pub as_of: Timestamp,
}

/// One fill (a leg of a trade), from an order's perspective.
#[derive(Clone, Debug, Deserialize)]
pub struct Fill {
	/// Exchange that owns this market, account balance or trading record.
	pub exchange: Exchange,

	/// Trade identity reported by the exchange’s fill history.
	pub trade_id: String,

	/// Engine fill identity used to join streams and REST; None for Polymarket fills.
	pub fill_id: Option<String>,

	/// Internal order UUID used by order reads and cancellation. None when unavailable.
	pub order_id: Option<OrderId>,

	/// On-chain outcome token identifier used by trading and quote subscriptions.
	pub token_id: TokenId,

	/// Buy or sell direction for this order or fill.
	pub side: Side,

	/// Quantity executed on this fill leg in micro shares.
	pub shares_micro: Micro,

	/// Execution price for this order’s fill leg in micro collateral per share.
	pub price_micro: Micro,

	/// Fee for this fill’s order leg in micro collateral.
	pub fee_micro: Micro,

	/// Maker or taker role for this order’s fill leg; None when the venue does not report it.
	pub role: Option<FillRole>,

	/// Trade-settlement lifecycle state, distinct from the order’s matching status.
	pub status: String,

	/// On-chain transaction hash; None until available.
	pub transaction_hash: Option<String>,

	/// RFC3339 execution time of this fill.
	pub executed_at: Timestamp,
}

/// The fills for one order.
#[derive(Clone, Debug, Deserialize)]
pub struct OrderTradesResponse {
	/// Recorded fill legs in newest-first order.
	pub trades: Vec<Fill>,

	/// RFC3339 timestamp at which this response or projection was observed.
	pub as_of: Timestamp,
}

/// Ack for a single cancel.
#[derive(Clone, Debug, Deserialize)]
pub struct CancelOrderResponse {
	/// Internal order UUID used by order reads and cancellation.
	pub order_id: OrderId,

	/// Asynchronous operation accepted by the server, not proof of completion.
	pub pending_operation: PendingOperation,

	/// RFC3339 timestamp at which this response or projection was observed.
	pub as_of: Timestamp,
}

/// Ack for cancel-all.
#[derive(Clone, Debug, Deserialize)]
pub struct CancelAllOrdersResponse {
	/// Internal wallet UUIDs included in the accepted operation.
	pub wallet_ids: Vec<WalletId>,

	/// Asynchronous operation accepted by the server, not proof of completion.
	pub pending_operation: PendingOperation,

	/// RFC3339 timestamp at which this response or projection was observed.
	pub as_of: Timestamp,
}

/// One price level in the REST orderbook — dollars and shares, not
/// micro-units.
#[derive(Clone, Copy, Debug, Deserialize)]
pub struct OrderbookLevel {
	/// Whole collateral units per share as a JSON number; not micro units.
	pub price: f64,

	/// Whole shares available at this price as a JSON number; not micro shares.
	pub size: f64,
}

/// A REST orderbook snapshot for one outcome.
#[derive(Clone, Debug, Deserialize)]
pub struct Orderbook {
	/// Bid depth ordered from highest to lowest price.
	pub bids: Vec<OrderbookLevel>,

	/// Ask depth ordered from lowest to highest price.
	pub asks: Vec<OrderbookLevel>,

	/// RFC3339 time at which the REST snapshot was produced.
	pub timestamp: String,

	/// Decimal sequence watermark used as the REST baseline during stream recovery.
	pub hash: String,

	/// Minimum price increment as whole-unit decimal text.
	pub tick_size: String,
}

/// Per-exchange balance + positions value + open commitments.
#[derive(Clone, Debug, Deserialize)]
pub struct PortfolioSummaryEntry {
	/// Exchange that owns this market, account balance or trading record.
	pub exchange: Exchange,

	/// Total collateral cash balance in micro units.
	pub cash_balance_micro: Micro,

	/// Collateral available for new orders in micro units.
	pub free_cash_micro: Micro,

	/// Current or marked position value in micro collateral.
	pub positions_value_micro: Micro,

	/// Cash plus position value in micro collateral.
	pub portfolio_value_micro: Micro,

	/// Remaining position cost basis in micro collateral.
	pub open_cost_basis_micro: Micro,

	/// Unrealized gain or loss on held positions in micro collateral.
	pub open_unrealized_pnl_micro: Micro,

	/// RFC3339 timestamp at which this response or projection was observed.
	pub as_of: Timestamp,
}

/// Separate per-exchange balances and valuations; collateral pools remain distinct.
#[derive(Clone, Debug, Deserialize)]
pub struct PortfolioSummaryResponse {
	/// Separate balance and valuation entries for each returned exchange.
	pub summaries: Vec<PortfolioSummaryEntry>,
}

/// One current position.
#[derive(Clone, Debug, Deserialize)]
pub struct Position {
	/// Exchange that owns this market, account balance or trading record.
	pub exchange: Exchange,

	/// On-chain condition identifier used by position and settlement operations.
	pub condition_id: ConditionId,

	/// On-chain outcome token identifier used by trading and quote subscriptions.
	pub token_id: TokenId,

	/// Total held quantity of this outcome in micro shares.
	pub shares_micro: Micro,

	/// Held shares not committed to open orders, in micro shares.
	pub available_shares_micro: Micro,

	/// Average acquisition price in micro collateral per share.
	pub avg_price_micro: Micro,

	/// Current outcome valuation in micro collateral per share.
	pub current_price_micro: Micro,

	/// Current holding value in micro collateral.
	pub current_value_micro: Micro,

	/// Gross collateral payout if the held outcome wins, in micro units.
	pub to_win_micro: Micro,

	/// Holding gain or loss relative to cost basis, in micro collateral.
	pub profit_loss_micro: Micro,

	/// Exact decimal percentage gain or loss; not an integer micro amount.
	pub profit_loss_percent: String,

	/// Whether the venue reports this resolved position as redeemable.
	pub redeemable: bool,

	/// Whether a complete outcome set is available to merge.
	pub mergeable: bool,

	/// Mergeable quantity per outcome leg in micro shares; None when unreported.
	pub mergeable_shares_micro: Option<Micro>,
}

/// Complete position rows, display sidecars and exchange-availability diagnostics.
#[derive(Clone, Debug, Deserialize)]
pub struct PositionsResponse {
	/// Market metadata keyed by outcome token ID.
	pub markets: Markets,

	/// Event metadata keyed by event slug.
	pub events: HashMap<String, EventMeta>,

	/// RFC3339 timestamp at which this response or projection was observed.
	pub as_of: Timestamp,

	/// Current held outcome positions for the successfully read exchanges.
	pub positions: Vec<Position>,

	/// Exchanges that failed this read; require an empty list before treating all rows as complete.
	pub unavailable_exchanges: Vec<Exchange>,
}

/// One page of nonterminal orders with display sidecars and an opaque continuation.
#[derive(Clone, Debug, Deserialize)]
pub struct OpenOrdersResponse {
	/// Market metadata keyed by outcome token ID.
	pub markets: Markets,

	/// Event metadata keyed by event slug.
	pub events: HashMap<String, EventMeta>,

	/// RFC3339 timestamp at which this response or projection was observed.
	pub as_of: Timestamp,

	/// Order rows for this response page.
	pub orders: Vec<Order>,

	/// Applied page size and opaque continuation token for the next request.
	pub pagination: CursorPagination,
}

/// A page of recent fills, newest-first.
#[derive(Clone, Debug, Deserialize)]
pub struct TradesResponse {
	/// Event metadata keyed by event slug.
	pub events: HashMap<String, EventMeta>,

	/// Recorded fill legs in newest-first order.
	pub trades: Vec<Fill>,

	/// Market metadata keyed by outcome token ID.
	pub markets: Markets,

	/// Applied page size and opaque continuation token for the next request.
	pub pagination: CursorPagination,

	/// Exchanges that failed this read; require an empty list before treating all rows as complete.
	pub unavailable_exchanges: Vec<Exchange>,

	/// RFC3339 timestamp at which this response or projection was observed.
	pub as_of: Timestamp,
}

/// A page of activity-feed rows, newest-first.
#[derive(Clone, Debug, Deserialize)]
pub struct ActivitiesResponse {
	/// Event metadata keyed by event slug.
	pub events: HashMap<String, EventMeta>,

	/// Condition-level metadata keyed by on-chain condition ID.
	pub conditions: HashMap<String, ConditionMeta>,

	/// Account-history rows in newest-first order, discriminated by activity type.
	pub activities: Vec<Activity>,

	/// Market metadata keyed by outcome token ID.
	pub markets: Markets,

	/// Applied page size and opaque continuation token for the next request.
	pub pagination: CursorPagination,

	/// RFC3339 timestamp at which this response or projection was observed.
	pub as_of: Timestamp,
}

/// Receipt for a split / merge / redeem.
#[derive(Clone, Debug, Deserialize)]
pub struct RelayerPositionOperation {
	/// Position operation represented by this receipt.
	pub operation: PositionOperation,

	/// On-chain condition identifier used by position and settlement operations.
	pub condition_id: ConditionId,

	/// Provider relayer submission identifier, distinct from a chain transaction hash.
	pub relayer_transaction_id: String,

	/// On-chain transaction hash; None until available.
	pub transaction_hash: Option<String>,

	/// Observed mined or confirmed state of the relayer transaction.
	pub relayer_state: RelayerState,

	/// RFC3339 timestamp at which this response or projection was observed.
	pub as_of: Timestamp,
}

/// Pending incentive balances, in micro-USDC.
#[derive(Clone, Debug, Deserialize)]
pub struct RebatesSummaryResponse {
	/// Pending maker rebates in micro collateral.
	pub maker_rebate_micro: Micro,

	/// Pending VIP taker rebates in micro collateral.
	pub vip_rebate_micro: Micro,

	/// Pending liquidity-provider incentives in micro collateral.
	pub lp_incentive_micro: Micro,

	/// Sum of all pending rebate programs in micro collateral.
	pub total_micro: Micro,
}

/// Public market/event counts.
#[derive(Clone, Copy, Debug, Deserialize)]
pub struct StatusResponse {
	/// Number of markets known to the platform.
	pub markets: i64,

	/// Number of events known to the platform.
	pub events: i64,
}

/// An AGARA operation accepted for asynchronous batch settlement.
#[derive(Clone, Debug, Deserialize)]
pub struct PositionOperationAccepted {
	/// EIP-712 account-batch digest used to poll asynchronous settlement.
	pub batch_hash: String,

	/// Initial asynchronous batch state; poll by batch_hash to confirm settlement.
	pub status: String,

	/// RFC3339 timestamp at which this response or projection was observed.
	pub as_of: Timestamp,
}

/// Exchange-dependent split/merge result; AGARA requires batch reconciliation.
#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum PositionOperationResponse {
	/// AGARA accepted the operation for asynchronous account-batch settlement.
	PendingBatch(PositionOperationAccepted),

	/// Polymarket returned its relayer transaction receipt.
	Relayer(RelayerPositionOperation),
}
/// Unmodified CMS market/outcome display data carried beside trading records.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]

pub struct MarketMetaDisplay {
	/// Raw market-level display blob as stored by the CMS.
	pub market: serde_json::Value,

	/// Raw outcome-level display blob as stored by the CMS.
	pub outcome: serde_json::Value,
}

/// Unmodified CMS market display data for a condition-level activity.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]

pub struct ConditionMetaDisplay {
	/// Raw market-level display blob as stored by the CMS.
	pub market: serde_json::Value,
}

/// Unmodified CMS event display data carried beside trading records.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]

pub struct EventMetaDisplay {
	/// Raw CMS event display object, retained without interpreting unknown properties.
	pub event: serde_json::Value,
}

/// Condition-keyed market metadata for activities that do not identify one outcome.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]

pub struct ConditionMeta {
	/// Stable Agara market UUID.
	pub market_id: MarketId,

	/// Human-readable question or title of the parent market.
	pub market_title: String,

	/// Artwork URL; None when no logo is available.
	pub logo_url: Option<String>,

	/// Parent event’s stable slug for discovery lookups and links.
	pub event_slug: String,

	/// Structured presentation metadata; it is not authoritative order or settlement state.
	pub display: ConditionMetaDisplay,
}

/// Event title and raw display metadata keyed by event slug in response sidecars.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]

pub struct EventMeta {
	/// Human-readable title of the parent event.
	pub event_title: String,

	/// Structured presentation metadata; it is not authoritative order or settlement state.
	pub display: EventMetaDisplay,
}

fn deserialize_optional_failure<'de, D>(deserializer: D) -> Result<Option<PublicFailure>, D::Error>
where
	D: Deserializer<'de>,
{
	Option::<PublicFailure>::deserialize(deserializer)
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

impl<'de> Deserialize<'de> for SignedOrderResult {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		#[derive(Deserialize)]
		#[serde(tag = "outcome", rename_all = "snake_case")]
		enum Wire {
			Accepted {
				index: u32,

				#[serde(flatten)]
				ack: CreateClobOrderResponse,
			},
			Rejected {
				index: u32,
				failure: PublicFailure,
			},
		}

		match Wire::deserialize(deserializer)? {
			Wire::Accepted { index, ack } => Ok(Self::Accepted { index, ack }),
			Wire::Rejected { index, failure } => Ok(Self::Rejected { index, failure }),
		}
	}
}

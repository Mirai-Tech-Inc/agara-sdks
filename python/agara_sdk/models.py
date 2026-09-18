"""Trading wire models. Micro amounts are strings, timestamps RFC 3339 strings."""

# Eager NotRequired markers keep runtime optional keys accurate on Python 3.10.
from typing import Any, Literal

from typing_extensions import NotRequired, TypedDict


class Order(TypedDict):
    internal_id: str
    exchange: str
    token_id: str
    condition_id: str | None
    side: str
    type: str
    price_micro: str | None
    original_size_micro: str | None
    collateral_amount_micro: str | None
    size_matched_micro: str
    avg_fill_price_micro: str | None
    status: str
    is_terminal: bool
    failure: "Failure | None"
    expiration: str
    created_at: str
    cancel_requested_at: str | None


class CreateClobOrderResponse(TypedDict):
    order_id: str
    source: str
    status: str
    pending_operation: str
    as_of: str


class ClobOrderResponse(TypedDict):
    order: Order
    markets: "dict[str, MarketMeta]"


class MarketMeta(TypedDict):
    market_id: str
    market_title: str
    outcome_name: str
    logo_url: str | None
    event_slug: str
    state: str
    neg_risk_id: str | None
    display: "MarketMetaDisplay"


class MarketMetaDisplay(TypedDict):
    market: Any
    outcome: Any


class ClobOrdersListResponse(TypedDict):
    orders: list[Order]
    markets: dict[str, MarketMeta]
    pagination: "Pagination"
    as_of: str


class CancelClobOrderResponse(TypedDict):
    order_id: str
    pending_operation: str
    as_of: str


class CancelAllClobOrdersResponse(TypedDict):
    wallet_ids: list[str]
    pending_operation: str
    as_of: str


class PortfolioPositionsResponse(TypedDict):
    positions: "list[PortfolioPosition]"
    markets: dict[str, MarketMeta]
    events: "dict[str, EventMeta]"
    unavailable_exchanges: list[str]
    as_of: str


class PortfolioPosition(TypedDict):
    exchange: str
    condition_id: str
    token_id: str
    shares_micro: str
    available_shares_micro: str
    avg_price_micro: str
    current_price_micro: str
    current_value_micro: str
    to_win_micro: str
    profit_loss_micro: str
    profit_loss_percent: str
    redeemable: bool
    mergeable: bool
    mergeable_shares_micro: str | None


class EventMeta(TypedDict):
    event_title: str
    display: "EventMetaDisplay"


class EventMetaDisplay(TypedDict):
    event: Any


class PortfolioOpenOrdersResponse(TypedDict):
    orders: list[Order]
    markets: dict[str, MarketMeta]
    events: dict[str, EventMeta]
    pagination: "Pagination"
    as_of: str


class PortfolioSummaryResponse(TypedDict):
    summaries: "list[PortfolioSummaryEntry]"


class PortfolioSummaryEntry(TypedDict):
    exchange: str
    cash_balance_micro: str
    free_cash_micro: str
    positions_value_micro: str
    portfolio_value_micro: str
    open_cost_basis_micro: str
    open_unrealized_pnl_micro: str
    as_of: str


class PortfolioTradesResponse(TypedDict):
    trades: "list[Fill]"
    markets: dict[str, MarketMeta]
    events: dict[str, EventMeta]
    pagination: "Pagination"
    unavailable_exchanges: list[str]
    as_of: str


class Fill(TypedDict):
    exchange: str
    trade_id: str
    fill_id: str | None
    order_id: str | None
    token_id: str
    side: str
    shares_micro: str
    price_micro: str
    fee_micro: str
    role: str | None
    status: str
    transaction_hash: str | None
    executed_at: str


class PortfolioActivitiesResponse(TypedDict):
    activities: "list[Activity]"
    markets: dict[str, MarketMeta]
    conditions: "dict[str, ConditionMeta]"
    events: dict[str, EventMeta]
    pagination: "Pagination"
    as_of: str


class ConditionMeta(TypedDict):
    market_id: str
    market_title: str
    logo_url: str | None
    event_slug: str
    display: "ConditionMetaDisplay"


class ConditionMetaDisplay(TypedDict):
    market: Any


class PortfolioBridgeDepositAddressResponse(TypedDict):
    wallet_address: str
    deposit_addresses: "PortfolioBridgeDepositAddresses"
    note: str | None


class PortfolioBridgeDepositAddresses(TypedDict):
    evm: str
    svm: str
    btc: str


class PortfolioBridgeSupportedAssetsResponse(TypedDict):
    wallet_address: str
    supported_assets: "list[PortfolioBridgeSupportedAsset]"
    note: str | None


class PortfolioBridgeSupportedAsset(TypedDict):
    chain_id: str
    chain_name: str
    address_type: str
    token: "PortfolioBridgeSupportedToken"
    min_checkout_usd: str


class PortfolioBridgeSupportedToken(TypedDict):
    name: str
    symbol: str
    address: str
    decimals: int


class PortfolioBridgeDepositQuoteResponse(TypedDict):
    est_checkout_time_ms: int
    est_fee_breakdown: "PortfolioBridgeEstimatedFeeBreakdown"
    est_input_usd: str
    est_output_usd: str
    est_to_token_base_unit: str
    quote_id: str


class PortfolioBridgeEstimatedFeeBreakdown(TypedDict):
    app_fee_label: str
    app_fee_percent: str
    app_fee_usd: str
    fill_cost_percent: str
    fill_cost_usd: str
    gas_usd: str
    max_slippage: str
    min_received: str
    swap_impact: str
    swap_impact_usd: str
    total_impact: str
    total_impact_usd: str


class PortfolioBridgeDepositQuoteRequest(TypedDict):
    from_chain_id: str
    from_token_address: str
    from_amount_base_unit: str


class PortfolioBridgeWithdrawQuoteRequest(TypedDict):
    to_chain_id: str
    to_token_address: str
    recipient_address: str
    from_amount_base_unit: str


class RebatesSummaryResponse(TypedDict):
    maker_rebate_micro: str
    vip_rebate_micro: str
    lp_incentive_micro: str
    total_micro: str


class LpIncentivesResponse(TypedDict):
    eligible: bool
    epoch_date: str
    epoch_ends_at: str
    markets: "list[LpIncentiveMarket]"


class LpIncentiveMarket(TypedDict):
    market_id: str
    question: str
    event_slug: str
    event_title: str
    logo_url: str | None
    close_time: str | None
    phase: str
    pool_micro: str
    max_spread_micro: int
    min_shares_micro: str
    epoch_score: float
    market_total_score: float
    projected_payout_micro: str
    has_traded: bool


class LpIncentiveCategoriesResponse(TypedDict):
    categories: "list[LpIncentiveCategory]"


class LpIncentiveCategory(TypedDict):
    slug: str
    label: str
    market_count: int


class ClosedLpIncentivesResponse(TypedDict):
    eligible: bool
    markets: "list[ClosedLpIncentiveMarket]"
    limit: int
    page: int
    total: int


class ClosedLpIncentiveMarket(TypedDict):
    market_id: str
    question: str
    event_slug: str
    event_title: str
    category_slug: str | None
    category_label: str | None
    closed_at: str | None
    pool_micro: str
    epoch_score: float
    market_total_score: float
    potential_payout_micro: str
    earned_micro: str
    epoch_date: str
    credited: bool
    has_traded: bool


class LpIncentiveEarningsResponse(TypedDict):
    current_cycle_micro: str
    last_7_days_micro: str
    last_30_days_micro: str
    all_time_micro: str


class WalletPnlSnapshotDto(TypedDict):
    bucket_at: str
    mark_run_id: str
    mark_captured_at: str
    chain_id: int
    classification_version: int
    wallet_universe_generation: int
    custody_through_block: int
    custody_through_block_hash: str
    custody_through_block_time: str
    collateral_micro: str
    positions_value_micro: str
    cumulative_inflow_micro: str
    cumulative_outflow_micro: str
    lifetime_pnl_micro: str
    position_count: int


WalletPnlHistoryDto = TypedDict(
    "WalletPnlHistoryDto",
    {
        "from": "str",
        "to": "str",
        "start": "WalletPnlSnapshotDto | None",
        "end": "WalletPnlSnapshotDto | None",
        "lifetime_pnl_change_micro": "str | None",
        "points": "list[WalletPnlSnapshotDto]",
        "has_more": "bool",
    },
)


class RealizedPnlReportDto(TypedDict):
    currency: str
    amountScale: int
    timezone: str
    asOf: str
    coverageStartedAt: str
    reportingStartedAt: str
    throughEvent: "RealizedPnlThroughEventDto"
    pendingOperations: int
    granularity: str
    window: str
    currentBucketPartial: bool
    totals: "RealizedPnlTotalsDto"
    buckets: "list[RealizedPnlBucketDto]"


class RealizedPnlThroughEventDto(TypedDict):
    shardId: int
    eventSeq: str


class RealizedPnlTotalsDto(TypedDict):
    last7Days: "RealizedPnlCategoryTotalsDto"
    last30Days: "RealizedPnlCategoryTotalsDto"
    allTime: "RealizedPnlCategoryTotalsDto"


class RealizedPnlCategoryTotalsDto(TypedDict):
    sales: str
    directMerges: str
    redemptions: str
    total: str


class RealizedPnlBucketDto(TypedDict):
    timestamp: str
    sales: str
    directMerges: str
    redemptions: str
    total: str


class AccountBatchStatusDto(TypedDict):
    batch_hash: str
    status: str
    seq: int
    deadline_unix_seconds: int
    origin: str
    tx_hash: str | None
    executed_at: str | None
    failure: "Failure | None"
    superseded_by_batch_hash: str | None
    heals_batch_hash: str | None
    unwound_at: str | None
    created_at: str


class BatchGroupStatusDto(TypedDict):
    group_id: str
    op_count: int
    chunk_count: int | None
    completed_at: str | None
    chunks: "list[GroupChunkDto]"
    as_of: str


class GroupChunkDto(TypedDict):
    chunk_index: int
    batch_hash: str
    status: str
    tx_hash: str | None
    failure: "Failure | None"


class PortfolioOrderActivity(TypedDict):
    exchange: str
    id: str
    order_id: str
    status: str
    fill_status: str
    condition_id: str | None
    token_id: str | None
    side: str
    filled_shares_micro: str
    average_fill_price_micro: str
    filled_amount_micro: str
    fees_micro: str
    created_at: str


class PortfolioSplitActivity(TypedDict):
    exchange: str
    id: str
    condition_id: str
    shares_micro: str
    amount_micro: str
    tx_hash: str
    created_at: str


class PortfolioMergeActivity(TypedDict):
    exchange: str
    id: str
    condition_id: str
    shares_micro: str
    amount_micro: str
    tx_hash: str
    created_at: str


class PortfolioRedeemActivity(TypedDict):
    exchange: str
    id: str
    condition_id: str
    token_id: str
    shares_micro: str
    amount_micro: str
    tx_hash: str
    created_at: str


class PortfolioDepositActivity(TypedDict):
    exchange: str
    id: str
    amount_micro: str
    chain_id: int
    token_address: str
    counterparty_address: str
    tx_hash: str
    created_at: str


class PortfolioWithdrawalActivity(TypedDict):
    exchange: str
    id: str
    amount_micro: str
    chain_id: int
    token_address: str
    counterparty_address: str
    tx_hash: str
    created_at: str


class Pagination(TypedDict):
    next_cursor: str | None
    limit: int


class RecoveryWire(TypedDict):
    strategy: str
    after_seconds: NotRequired[int]
    resource: NotRequired[dict[str, str]]


class Failure(TypedDict):
    code: str
    title: str
    detail: NotRequired[str]
    recovery: RecoveryWire


class Status(TypedDict):
    markets: int
    events: int


class RawOrderbookLevel(TypedDict):
    price: float
    size: float


class RawOrderbook(TypedDict):
    bids: list[RawOrderbookLevel]
    asks: list[RawOrderbookLevel]
    timestamp: str
    hash: str
    tick_size: str


class LimitOrderRequest(TypedDict):
    token_id: str
    side: Literal["BUY", "SELL"]
    type: Literal["LIMIT"]
    time_in_force: Literal["GTC", "GTD", "FAK", "FOK"]
    price_micro: str
    shares_micro: str
    post_only: NotRequired[bool]
    expiration_unix_seconds: NotRequired[int]


class MarketBuyRequest(TypedDict):
    token_id: str
    side: Literal["BUY"]
    type: Literal["MARKET"]
    time_in_force: Literal["FAK", "FOK"]
    collateral_amount_micro: str
    post_only: NotRequired[bool]


class MarketSellRequest(TypedDict):
    token_id: str
    side: Literal["SELL"]
    type: Literal["MARKET"]
    time_in_force: Literal["FAK", "FOK"]
    shares_micro: str
    post_only: NotRequired[bool]


OrderRequest = LimitOrderRequest | MarketBuyRequest | MarketSellRequest


class SignedOrderRequest(LimitOrderRequest):
    salt: str
    maker: str
    chain_token_id: str
    maker_amount: str
    taker_amount: str
    side_u8: int
    timestamp: str
    metadata: str
    builder: str
    order_hash: str
    signature: str


class AcceptedSignedOrder(CreateClobOrderResponse):
    index: int
    outcome: Literal["accepted"]


class RejectedSignedOrder(TypedDict):
    index: int
    outcome: Literal["rejected"]
    failure: Failure


SignedOrderResult = AcceptedSignedOrder | RejectedSignedOrder


class SignedOrdersResponse(TypedDict):
    results: list[SignedOrderResult]
    as_of: str


class OrderTradesResponse(TypedDict):
    trades: list[Fill]
    as_of: str


class PositionOperationAccepted(TypedDict):
    batch_hash: str
    status: str
    as_of: str


class PositionOperationReceipt(TypedDict):
    operation: str
    condition_id: str
    relayer_transaction_id: str
    transaction_hash: str | None
    relayer_state: Literal["MINED", "CONFIRMED"]
    as_of: str


PositionOperationResponse = PositionOperationAccepted | PositionOperationReceipt
PortfolioBridgeWithdrawQuoteResponse = PortfolioBridgeDepositQuoteResponse


class SplitOp(TypedDict):
    kind: Literal["SPLIT"]
    market_id: str
    condition_id: str
    shares_micro: int


class MergeOp(TypedDict):
    kind: Literal["MERGE"]
    market_id: str
    condition_id: str
    shares_micro: int


class WithdrawOp(TypedDict):
    kind: Literal["WITHDRAW"]
    destination: str
    amount_micro: int


BatchOp = SplitOp | MergeOp | WithdrawOp


class BatchSubmission(TypedDict):
    ops: list[BatchOp]
    seq: int
    deadline_unix_seconds: int
    signature: str
    heals_batch_hash: NotRequired[str | None]


class BatchSupersedeSubmission(TypedDict):
    ops: list[BatchOp]
    deadline_unix_seconds: int
    signature: str


class BatchAccepted(TypedDict):
    batch_hash: str


class BatchSuperseded(TypedDict):
    outcome: Literal["SUPERSEDED"]
    batch: AccountBatchStatusDto


class BatchSupersedeRefused(TypedDict):
    outcome: Literal["REFUSED"]
    current: AccountBatchStatusDto


BatchSupersedeResult = BatchSuperseded | BatchSupersedeRefused


class OrderActivity(PortfolioOrderActivity):
    type: Literal["ORDER"]


class SplitActivity(PortfolioSplitActivity):
    type: Literal["SPLIT"]


class MergeActivity(PortfolioMergeActivity):
    type: Literal["MERGE"]


class RedeemActivity(PortfolioRedeemActivity):
    type: Literal["REDEEM"]


class DepositActivity(PortfolioDepositActivity):
    type: Literal["DEPOSIT"]


class WithdrawalActivity(PortfolioWithdrawalActivity):
    type: Literal["WITHDRAWAL"]


class LpPayoutActivity(TypedDict):
    type: Literal["LP_PAYOUT"]
    exchange: str
    id: str
    amount_micro: str
    epoch_date: str
    tx_hash: str | None
    created_at: str


Activity = (
    OrderActivity
    | SplitActivity
    | MergeActivity
    | RedeemActivity
    | DepositActivity
    | WithdrawalActivity
    | LpPayoutActivity
)

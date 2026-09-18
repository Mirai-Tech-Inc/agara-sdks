"""Catalogue wire types from platform a7e8c2d; numeric JSON fields retain wire units."""

# Eager NotRequired markers keep runtime optional keys accurate on Python 3.10.
from typing import Any, Literal

from typing_extensions import NotRequired, TypedDict


class CategoryResource(TypedDict):
    id: str
    slug: str
    label: str
    event_types: "CategoryResourceEventTypes"


class CategoryResourceEventTypes(TypedDict):
    game: bool
    multi_outcome: bool
    proposition: bool


class EventCanonicalUrlResponse(TypedDict):
    redirect_url: str | None
    canonical_slug: str


class EventDetail0(TypedDict):
    event_type: Literal["GAME"]
    display: "GameEventDisplay"
    game_state: "GameState | None"
    id: str
    slug: str
    title: str
    is_accepting_orders: bool
    is_live: bool
    is_group_root: bool
    start_time: str | None
    end_time: str | None
    volume_micro: "MicroUsd"
    main_market_id: str | None
    resolution_status: Literal["ACTIVE", "PROPOSED", "DISPUTED", "RESOLVED", "VOID"]
    state: NotRequired["EventLifecycleState"]
    categories: "list[EventDetail0CategoriesItem]"
    primary_category: "EventDetail0PrimaryCategory0 | None"
    markets: "list[MarketDetail]"
    sport_group: NotRequired[str]
    main_market_group: NotRequired["MainMarketGroup"]
    featured_spread_markets: NotRequired["list[MarketDetail]"]
    featured_totals_market: NotRequired["MarketDetail"]
    market_tabs: NotRequired["list[MarketTab]"]
    disputed_market_count: NotRequired[float]
    resolved_market_count: NotRequired[float]
    related_events: "list[EventListItem]"


class GameEventDisplay(TypedDict):
    image_url: NotRequired[str]
    icon_url: NotRequired[str]
    description: NotRequired[str]
    subtext: NotRequired[str]
    display_date: NotRequired[str]
    liquidity: NotRequired[float]
    has_three_way_markets: NotRequired[bool]
    has_prop_markets: NotRequired[bool]
    has_legacy_prop_markets: NotRequired[bool]
    market_categories: NotRequired[list[str]]
    tags: NotRequired[list[str]]
    resolution_source: NotRequired[str]
    layout: NotRequired["EventLayout"]
    game_unit_label: NotRequired["GameUnitLabel | None"]
    spreads_main_market_id: NotRequired[str | None]
    totals_main_market_id: NotRequired[str | None]
    polymarket: NotRequired["PolymarketEventDisplay"]
    agara_cms: NotRequired["AgaraCmsEventDisplay"]
    team_a: NotRequired["SportsTeam"]
    team_b: NotRequired["SportsTeam"]
    sport_slug: NotRequired[str]
    league_slug: NotRequired[str]
    format_label: NotRequired[str]
    spreads_main_line: NotRequired[float]
    totals_main_line: NotRequired[float]


class PolymarketEventDisplay(TypedDict):
    url: NotRequired[str]
    series: NotRequired["PolymarketEventDisplaySeries"]
    event_week: NotRequired[str | float]
    neg_risk: NotRequired[bool]
    parent_event_id: NotRequired[str]
    uma_resolution_statuses: NotRequired[list[str]]
    competitive: NotRequired[float]


class PolymarketEventDisplaySeries(TypedDict):
    id: str
    slug: str
    title: str


class SportsTeam(TypedDict):
    name: str
    abbreviation: str | None
    logo_url: str | None
    color: str | None
    record: str | None
    sport_slug: str | None
    league_slug: str | None
    provider_id: str | None
    source_team_id: str | None


class GameState(TypedDict):
    id: float
    score: NotRequired["GameStateScore"]
    match: NotRequired["GameStateMatch"]


class GameStateTennisScore(TypedDict):
    sport: Literal["tennis"]
    match: "GameStateTennisScoreMatch"
    sets: "list[GameStateTennisScoreSetsItem]"


class GameStateTennisScoreMatch(TypedDict):
    a: float
    b: float


class GameStateTennisScoreSetsItem(TypedDict):
    a: float
    b: float
    tiebreak: NotRequired["GameStateTennisScoreSetsItemTiebreak"]


class GameStateTennisScoreSetsItemTiebreak(TypedDict):
    a: float
    b: float


class GameStateDefaultScore(TypedDict):
    sport: Literal["default"]
    match: "GameStateDefaultScoreMatch"


class GameStateDefaultScoreMatch(TypedDict):
    a: float
    b: float


class GameStateMatch(TypedDict):
    lifecycle: "GameStateLifecycle"
    live_label: NotRequired[str]


class EventDetail0CategoriesItem(TypedDict):
    slug: str
    label: str
    root_slug: str
    path: "list[EventDetail0CategoriesItemPathItem]"


class EventDetail0CategoriesItemPathItem(TypedDict):
    slug: str
    label: str


class EventDetail0PrimaryCategory0(TypedDict):
    slug: str
    label: str


class MarketDetail(TypedDict):
    id: str
    state: NotRequired["MarketState"]
    exchange: "Exchange"
    slug: str
    question: str
    display_type: "DisplayType | None"
    source_market_type: str | None
    outcomes: "list[MarketOutcome]"
    is_accepting_orders: bool
    halted_at: str | None
    halted_reason: str | None
    volume_micro: "MicroUsd"
    liquidity: str
    volume_24h_micro: "MicroUsd"
    tick_size_micro: "MicroProbability | None"
    engine_config: "AgaraEngineConfig | None"
    min_order_size_micro: "MicroUsd | None"
    line: str | None
    resolution_status: "ResolutionStatus | None"
    resolution_outcome: str | None
    resolved_at: str | None
    resolution: "MarketResolution | None"
    archived_at: str | None
    condition_id: NotRequired[str | None]
    neg_risk_id: NotRequired[str | None]
    three_way_role: NotRequired["ThreeWayRole | None"]
    is_three_way: NotRequired[bool | None]
    game_number: NotRequired[float | None]
    display_label: str
    lp_incentive: NotRequired["MarketLpIncentive | None"]
    display: "MarketDisplay"


class MarketOutcome(TypedDict):
    id: str
    index: float
    label: str
    price_micro: "MicroProbability | None"
    token_id: str | None
    best_bid_micro: NotRequired["MicroProbability | None"]
    best_ask_micro: NotRequired["MicroProbability | None"]
    spread_micro: NotRequired["MicroProbability | None"]
    kind: NotRequired["OutcomeKind"]
    short_label: NotRequired[str]
    team_side: NotRequired["OutcomeTeamSide | None"]
    side: NotRequired[Literal["left", "right", None]]
    threshold: NotRequired[float | None]
    resolution_price_micro: NotRequired["MicroProbability | None"]
    bet_slip_title: NotRequired[str]
    resolution_label: NotRequired[str]
    display: NotRequired["OutcomeDisplay"]


class BaseOutcomeDisplay(TypedDict):
    label: NotRequired[str]
    kind: NotRequired["OutcomeKind"]
    short_label: NotRequired[str]
    team_side: NotRequired["OutcomeTeamSide | None"]
    side: NotRequired[Literal["left", "right", None]]
    threshold: NotRequired[float | None]
    bet_slip_title: NotRequired[str]
    resolution_label: NotRequired[str]
    polymarket: NotRequired["PolymarketOutcomeDisplay"]
    agara_cms: NotRequired["AgaraCmsOutcomeDisplay"]


class AgaraEngineConfig(TypedDict):
    tick_size: float
    price_scale: float
    size_scale: float
    min_price: float
    max_price: float


class MarketResolution(TypedDict):
    source: str
    reference_value: str | None
    observed_value: str | None
    observed_at: str | None


class MarketLpIncentive(TypedDict):
    pool_micro: "MicroUsd"
    max_spread_micro: float
    min_shares_micro: str


class BaseMarketDisplay(TypedDict):
    image_url: NotRequired[str]
    portfolio_title: NotRequired[str]
    description: NotRequired[str]
    resolution_source: NotRequired[str]
    resolution_criteria: NotRequired[str]
    resolution_sources: NotRequired["list[ResolutionSourceLink]"]
    player_name: NotRequired[str]
    stat_type: NotRequired[str]
    liquidity: NotRequired[float]
    cadence_seconds: NotRequired[float]
    reference_price: NotRequired[str]
    currency: NotRequired[str | None]
    reference_at: NotRequired[str]
    observe_at: NotRequired[str]
    band_index: NotRequired[float]
    band_lower: NotRequired[str | None]
    band_upper: NotRequired[str | None]
    band_label: NotRequired[str]
    pyth_feed_id: NotRequired[str]
    pyth_benchmark_symbol: NotRequired[str]
    tradingview_symbol: NotRequired[str]
    pyth_pro_symbol: NotRequired[str]
    security_symbol: NotRequired[str]
    security_provider: NotRequired[str]
    game_number: NotRequired[float | None]
    is_three_way: NotRequired[bool]
    three_way_role: NotRequired["ThreeWayRole"]
    display_label: NotRequired[str]
    polymarket: NotRequired["PolymarketMarketDisplay"]
    agara_cms: NotRequired["AgaraCmsMarketDisplay"]


class ResolutionSourceLink(TypedDict):
    title: str
    url: str


class PolymarketMarketDisplay(TypedDict):
    uma_resolution_statuses: NotRequired[list[str]]
    uma_end_date: NotRequired[str]
    resolved_by: NotRequired[str]
    deployed_at: NotRequired[str]


class MainMarketGroup(TypedDict):
    neg_risk_id: str | None
    markets: list[MarketDetail]


class MarketTab(TypedDict):
    key: str
    label: str
    order: float
    sections: "list[CategorySection]"


class CategorySection(TypedDict):
    key: str
    label: str
    order: float
    render_hint: "CategoryRenderHint"
    groups: "list[MarketGroup]"
    meta: NotRequired["CategorySectionMeta"]


class MarketGroup(TypedDict):
    key: str
    market_ids: list[str]


class CategorySectionMeta(TypedDict):
    selector_axis: NotRequired[Literal["line", "game_number"]]
    default_line: NotRequired[float | None]
    line_owner_side: NotRequired[Literal["first", "second"]]
    grid_cols_hint: NotRequired[Literal[2, 3, 4]]


class EventListItem0(TypedDict):
    event_type: Literal["GAME"]
    market_count: float
    display: GameEventDisplay
    game_state: GameState | None
    id: str
    slug: str
    title: str
    is_accepting_orders: bool
    is_live: bool
    is_group_root: bool
    start_time: str | None
    end_time: str | None
    volume_micro: "MicroUsd"
    primary_category: "EventListItem0PrimaryCategory0 | None"
    main_market_id: str | None
    resolution_status: Literal["ACTIVE", "PROPOSED", "DISPUTED", "RESOLVED", "VOID"]
    state: NotRequired["EventLifecycleState"]
    main_market_group: NotRequired[MainMarketGroup]


class EventListItem0PrimaryCategory0(TypedDict):
    slug: str
    label: str


class EventListItem1(TypedDict):
    event_type: Literal["MULTI_OUTCOME"]
    market_count: float
    display: "MultiOutcomeEventDisplay"
    listing_markets: NotRequired["list[ListingMarket]"]
    id: str
    slug: str
    title: str
    is_accepting_orders: bool
    is_live: bool
    is_group_root: bool
    start_time: str | None
    end_time: str | None
    volume_micro: "MicroUsd"
    primary_category: "EventListItem1PrimaryCategory0 | None"
    main_market_id: str | None
    resolution_status: Literal["ACTIVE", "PROPOSED", "DISPUTED", "RESOLVED", "VOID"]
    state: NotRequired["EventLifecycleState"]
    main_market_group: NotRequired[MainMarketGroup]


class MultiOutcomeEventDisplay(TypedDict):
    image_url: NotRequired[str]
    icon_url: NotRequired[str]
    description: NotRequired[str]
    subtext: NotRequired[str]
    display_date: NotRequired[str]
    liquidity: NotRequired[float]
    has_three_way_markets: NotRequired[bool]
    has_prop_markets: NotRequired[bool]
    has_legacy_prop_markets: NotRequired[bool]
    market_categories: NotRequired[list[str]]
    tags: NotRequired[list[str]]
    resolution_source: NotRequired[str]
    layout: NotRequired["EventLayout"]
    game_unit_label: NotRequired["GameUnitLabel | None"]
    spreads_main_market_id: NotRequired[str | None]
    totals_main_market_id: NotRequired[str | None]
    polymarket: NotRequired[PolymarketEventDisplay]
    agara_cms: NotRequired["AgaraCmsEventDisplay"]
    subcategory: NotRequired[str]
    candidate_count: NotRequired[float]
    active_candidate_count: NotRequired[float]
    top_candidate_name: NotRequired[str]
    top_candidate_price_micro: NotRequired["MicroProbability"]
    top_candidates: NotRequired["list[MultiOutcomeEventDisplayTopCandidatesItem]"]
    resolved_winner_name: NotRequired[str]
    is_future: NotRequired[bool]
    is_child_event: NotRequired[bool]


class MultiOutcomeEventDisplayTopCandidatesItem(TypedDict):
    name: str
    price_micro: NotRequired["MicroProbability"]
    volume_micro: "MicroUsd"


class ListingMarket(TypedDict):
    id: str
    state: NotRequired["MarketState"]
    slug: str
    exchange: "Exchange"
    question: str
    display_label: str
    pyth_feed_id: NotRequired[str]
    pyth_pro_symbol: NotRequired[str]
    security_symbol: NotRequired[str]
    security_provider: NotRequired[str]
    reference_price: NotRequired[str]
    currency: NotRequired[str | None]
    reference_at: NotRequired[str]
    observe_at: NotRequired[str]
    outcomes: "list[ListingMarketOutcome]"


class ListingMarketOutcome(TypedDict):
    label: str
    short_label: NotRequired[str]
    kind: NotRequired["OutcomeKind"]
    price_micro: "MicroProbability | None"
    token_id: str | None


class EventListItem1PrimaryCategory0(TypedDict):
    slug: str
    label: str


class EventListItem2(TypedDict):
    event_type: Literal["PROPOSITION"]
    market_count: float
    display: "PropositionEventDisplay"
    listing_markets: NotRequired[list[ListingMarket]]
    id: str
    slug: str
    title: str
    is_accepting_orders: bool
    is_live: bool
    is_group_root: bool
    start_time: str | None
    end_time: str | None
    volume_micro: "MicroUsd"
    primary_category: "EventListItem2PrimaryCategory0 | None"
    main_market_id: str | None
    resolution_status: Literal["ACTIVE", "PROPOSED", "DISPUTED", "RESOLVED", "VOID"]
    state: NotRequired["EventLifecycleState"]
    main_market_group: NotRequired[MainMarketGroup]


class PropositionEventDisplay(TypedDict):
    image_url: NotRequired[str]
    icon_url: NotRequired[str]
    description: NotRequired[str]
    subtext: NotRequired[str]
    display_date: NotRequired[str]
    liquidity: NotRequired[float]
    has_three_way_markets: NotRequired[bool]
    has_prop_markets: NotRequired[bool]
    has_legacy_prop_markets: NotRequired[bool]
    market_categories: NotRequired[list[str]]
    tags: NotRequired[list[str]]
    resolution_source: NotRequired[str]
    layout: NotRequired["EventLayout"]
    game_unit_label: NotRequired["GameUnitLabel | None"]
    spreads_main_market_id: NotRequired[str | None]
    totals_main_market_id: NotRequired[str | None]
    polymarket: NotRequired[PolymarketEventDisplay]
    agara_cms: NotRequired["AgaraCmsEventDisplay"]
    subcategory: NotRequired[str]


class EventListItem2PrimaryCategory0(TypedDict):
    slug: str
    label: str


class EventListItem3(TypedDict):
    event_type: Literal["RECURRING_PROPOSITION"]
    market_count: NotRequired[Any]
    display: "RecurringPropositionEventDisplay"
    listing_markets: NotRequired[list[ListingMarket]]
    id: str
    slug: str
    title: str
    is_accepting_orders: bool
    is_live: bool
    is_group_root: bool
    start_time: str | None
    end_time: str | None
    volume_micro: "MicroUsd"
    primary_category: "EventListItem3PrimaryCategory0 | None"
    main_market_id: str | None
    resolution_status: Literal["ACTIVE", "PROPOSED", "DISPUTED", "RESOLVED", "VOID"]
    state: NotRequired["EventLifecycleState"]
    main_market_group: NotRequired[MainMarketGroup]


class RecurringPropositionEventDisplay(TypedDict):
    image_url: NotRequired[str]
    icon_url: NotRequired[str]
    description: NotRequired[str]
    subtext: NotRequired[str]
    display_date: NotRequired[str]
    liquidity: NotRequired[float]
    has_three_way_markets: NotRequired[bool]
    has_prop_markets: NotRequired[bool]
    has_legacy_prop_markets: NotRequired[bool]
    market_categories: NotRequired[list[str]]
    tags: NotRequired[list[str]]
    resolution_source: NotRequired[str]
    layout: NotRequired["EventLayout"]
    game_unit_label: NotRequired["GameUnitLabel | None"]
    spreads_main_market_id: NotRequired[str | None]
    totals_main_market_id: NotRequired[str | None]
    polymarket: NotRequired[PolymarketEventDisplay]
    agara_cms: NotRequired["AgaraCmsEventDisplay"]
    subcategory: NotRequired[str]
    series_slug: NotRequired[str]
    cadence_seconds: NotRequired[float]


class EventListItem3PrimaryCategory0(TypedDict):
    slug: str
    label: str


class EventDetail1(TypedDict):
    event_type: Literal["MULTI_OUTCOME"]
    display: MultiOutcomeEventDisplay
    id: str
    slug: str
    title: str
    is_accepting_orders: bool
    is_live: bool
    is_group_root: bool
    start_time: str | None
    end_time: str | None
    volume_micro: "MicroUsd"
    main_market_id: str | None
    resolution_status: Literal["ACTIVE", "PROPOSED", "DISPUTED", "RESOLVED", "VOID"]
    state: NotRequired["EventLifecycleState"]
    categories: "list[EventDetail1CategoriesItem]"
    primary_category: "EventDetail1PrimaryCategory0 | None"
    markets: list[MarketDetail]
    sport_group: NotRequired[str]
    main_market_group: NotRequired[MainMarketGroup]
    featured_spread_markets: NotRequired[list[MarketDetail]]
    featured_totals_market: NotRequired[MarketDetail]
    market_tabs: NotRequired[list[MarketTab]]
    disputed_market_count: NotRequired[float]
    resolved_market_count: NotRequired[float]
    related_events: "list[EventListItem]"


class EventDetail1CategoriesItem(TypedDict):
    slug: str
    label: str
    root_slug: str
    path: "list[EventDetail1CategoriesItemPathItem]"


class EventDetail1CategoriesItemPathItem(TypedDict):
    slug: str
    label: str


class EventDetail1PrimaryCategory0(TypedDict):
    slug: str
    label: str


class EventDetail2(TypedDict):
    event_type: Literal["PROPOSITION"]
    display: PropositionEventDisplay
    id: str
    slug: str
    title: str
    is_accepting_orders: bool
    is_live: bool
    is_group_root: bool
    start_time: str | None
    end_time: str | None
    volume_micro: "MicroUsd"
    main_market_id: str | None
    resolution_status: Literal["ACTIVE", "PROPOSED", "DISPUTED", "RESOLVED", "VOID"]
    state: NotRequired["EventLifecycleState"]
    categories: "list[EventDetail2CategoriesItem]"
    primary_category: "EventDetail2PrimaryCategory0 | None"
    markets: list[MarketDetail]
    sport_group: NotRequired[str]
    main_market_group: NotRequired[MainMarketGroup]
    featured_spread_markets: NotRequired[list[MarketDetail]]
    featured_totals_market: NotRequired[MarketDetail]
    market_tabs: NotRequired[list[MarketTab]]
    disputed_market_count: NotRequired[float]
    resolved_market_count: NotRequired[float]
    related_events: "list[EventListItem]"


class EventDetail2CategoriesItem(TypedDict):
    slug: str
    label: str
    root_slug: str
    path: "list[EventDetail2CategoriesItemPathItem]"


class EventDetail2CategoriesItemPathItem(TypedDict):
    slug: str
    label: str


class EventDetail2PrimaryCategory0(TypedDict):
    slug: str
    label: str


class EventDetail3(TypedDict):
    event_type: Literal["RECURRING_PROPOSITION"]
    display: RecurringPropositionEventDisplay
    id: str
    slug: str
    title: str
    is_accepting_orders: bool
    is_live: bool
    is_group_root: bool
    start_time: str | None
    end_time: str | None
    volume_micro: "MicroUsd"
    main_market_id: str | None
    resolution_status: Literal["ACTIVE", "PROPOSED", "DISPUTED", "RESOLVED", "VOID"]
    state: NotRequired["EventLifecycleState"]
    categories: "list[EventDetail3CategoriesItem]"
    primary_category: "EventDetail3PrimaryCategory0 | None"
    markets: list[MarketDetail]
    sport_group: NotRequired[str]
    main_market_group: NotRequired[MainMarketGroup]
    featured_spread_markets: NotRequired[list[MarketDetail]]
    featured_totals_market: NotRequired[MarketDetail]
    market_tabs: NotRequired[list[MarketTab]]
    disputed_market_count: NotRequired[float]
    resolved_market_count: NotRequired[float]
    related_events: "list[EventListItem]"


class EventDetail3CategoriesItem(TypedDict):
    slug: str
    label: str
    root_slug: str
    path: "list[EventDetail3CategoriesItemPathItem]"


class EventDetail3CategoriesItemPathItem(TypedDict):
    slug: str
    label: str


class EventDetail3PrimaryCategory0(TypedDict):
    slug: str
    label: str


class EventsListResponse(TypedDict):
    events: "list[EventListItem]"
    pagination: "Pagination"
    sections: NotRequired["list[EventListSection]"]


class Pagination(TypedDict):
    next_cursor: str | None
    limit: float


class EventListSection(TypedDict):
    category: "EventListSectionCategory"
    event_ids: list[str]


class EventListSectionCategory(TypedDict):
    slug: str
    label: str


class MarketsListResponse(TypedDict):
    markets: "list[MarketsListItem]"
    pagination: Pagination


class MarketsListItem(TypedDict):
    id: str
    state: NotRequired["MarketState"]
    exchange: "Exchange"
    slug: str
    question: str
    display_type: "DisplayType | None"
    source_market_type: str | None
    outcomes: list[MarketOutcome]
    is_accepting_orders: bool
    halted_at: str | None
    halted_reason: str | None
    volume_micro: "MicroUsd"
    liquidity: str
    volume_24h_micro: "MicroUsd"
    tick_size_micro: "MicroProbability | None"
    engine_config: AgaraEngineConfig | None
    min_order_size_micro: "MicroUsd | None"
    line: str | None
    resolution_status: "ResolutionStatus | None"
    resolution_outcome: str | None
    resolved_at: str | None
    resolution: MarketResolution | None
    archived_at: str | None
    condition_id: NotRequired[str | None]
    neg_risk_id: NotRequired[str | None]
    three_way_role: NotRequired["ThreeWayRole | None"]
    is_three_way: NotRequired[bool | None]
    game_number: NotRequired[float | None]
    display_label: str
    lp_incentive: NotRequired[MarketLpIncentive | None]
    display: "MarketDisplay"
    event: "MarketsListEventRef"


class MarketsListEventRef(TypedDict):
    id: str
    slug: str
    title: str
    active: bool


class NextSessionResponse(TypedDict):
    mic: str
    session: "TradingDay | None"


class TradingDay(TypedDict):
    mic: str
    date: str
    is_trading_day: bool
    segments: "list[SessionSegment]"
    reason: str | None


class SessionSegment(TypedDict):
    open: str
    close: str


class SearchResponse(TypedDict):
    events: "list[SearchEventItem]"
    markets: "list[SearchMarketItem]"
    categories: "list[SearchCategoryItem]"


class SearchEventItem0(TypedDict):
    relevance_score: float
    event_type: Literal["GAME"]
    market_count: float
    display: GameEventDisplay
    game_state: GameState | None
    id: str
    slug: str
    title: str
    is_accepting_orders: bool
    is_live: bool
    is_group_root: bool
    start_time: str | None
    end_time: str | None
    volume_micro: "MicroUsd"
    primary_category: "SearchEventItem0PrimaryCategory0 | None"
    main_market_id: str | None
    resolution_status: Literal["ACTIVE", "PROPOSED", "DISPUTED", "RESOLVED", "VOID"]
    state: NotRequired["EventLifecycleState"]
    main_market_group: NotRequired[MainMarketGroup]


class SearchEventItem0PrimaryCategory0(TypedDict):
    slug: str
    label: str


class SearchEventItem1(TypedDict):
    relevance_score: float
    event_type: Literal["MULTI_OUTCOME"]
    market_count: float
    display: MultiOutcomeEventDisplay
    listing_markets: NotRequired[list[ListingMarket]]
    id: str
    slug: str
    title: str
    is_accepting_orders: bool
    is_live: bool
    is_group_root: bool
    start_time: str | None
    end_time: str | None
    volume_micro: "MicroUsd"
    primary_category: "SearchEventItem1PrimaryCategory0 | None"
    main_market_id: str | None
    resolution_status: Literal["ACTIVE", "PROPOSED", "DISPUTED", "RESOLVED", "VOID"]
    state: NotRequired["EventLifecycleState"]
    main_market_group: NotRequired[MainMarketGroup]


class SearchEventItem1PrimaryCategory0(TypedDict):
    slug: str
    label: str


class SearchEventItem2(TypedDict):
    relevance_score: float
    event_type: Literal["PROPOSITION"]
    market_count: float
    display: PropositionEventDisplay
    listing_markets: NotRequired[list[ListingMarket]]
    id: str
    slug: str
    title: str
    is_accepting_orders: bool
    is_live: bool
    is_group_root: bool
    start_time: str | None
    end_time: str | None
    volume_micro: "MicroUsd"
    primary_category: "SearchEventItem2PrimaryCategory0 | None"
    main_market_id: str | None
    resolution_status: Literal["ACTIVE", "PROPOSED", "DISPUTED", "RESOLVED", "VOID"]
    state: NotRequired["EventLifecycleState"]
    main_market_group: NotRequired[MainMarketGroup]


class SearchEventItem2PrimaryCategory0(TypedDict):
    slug: str
    label: str


class SearchEventItem3(TypedDict):
    relevance_score: float
    event_type: Literal["RECURRING_PROPOSITION"]
    market_count: NotRequired[Any]
    display: RecurringPropositionEventDisplay
    listing_markets: NotRequired[list[ListingMarket]]
    id: str
    slug: str
    title: str
    is_accepting_orders: bool
    is_live: bool
    is_group_root: bool
    start_time: str | None
    end_time: str | None
    volume_micro: "MicroUsd"
    primary_category: "SearchEventItem3PrimaryCategory0 | None"
    main_market_id: str | None
    resolution_status: Literal["ACTIVE", "PROPOSED", "DISPUTED", "RESOLVED", "VOID"]
    state: NotRequired["EventLifecycleState"]
    main_market_group: NotRequired[MainMarketGroup]


class SearchEventItem3PrimaryCategory0(TypedDict):
    slug: str
    label: str


class SearchMarketItem(TypedDict):
    relevance_score: float
    id: str
    slug: str
    question: str
    is_accepting_orders: bool
    volume_micro: "MicroUsd"
    event: "SearchMarketEventRef"
    outcomes: "list[SearchOutcome]"


class SearchMarketEventRef(TypedDict):
    slug: str
    title: str
    image_url: str | None


class SearchOutcome(TypedDict):
    label: str
    price_micro: "MicroProbability | None"


class SearchCategoryItem(TypedDict):
    slug: str
    label: str
    icon: str | None


class SecuritiesResponse(TypedDict):
    securities: "list[Security]"


class Security(TypedDict):
    symbol: str
    label: str
    mic: str
    asset_class: str
    active: bool


class TradingDaysResponse(TypedDict):
    mic: str
    days: list[TradingDay]


class TradingVenue(TypedDict):
    mic: str
    name: str
    timezone: str
    country: str
    currency: str
    default_segments: list[SessionSegment]
    active: bool


class TradingVenuesResponse(TypedDict):
    venues: list[TradingVenue]


class PricePointBody(TypedDict):
    value: float
    mantissa: str
    expo: float
    publish_time_micros: float


class ShimHistory(TypedDict):
    s: Literal["ok"]
    t: list[float]
    c: list[float]


class TokenHistoryResponse(TypedDict):
    s: Literal["ok"]
    t: list[float]
    c: list[float]
    latestTradeAt: float | None


class SecurityProvider(TypedDict):
    provider: str
    provider_symbol: str
    active: bool


class SecurityDetail(Security):
    providers: list[SecurityProvider]
    venue_timezone: str
    venue_default_segments: list[SessionSegment]


EventLayout = Literal["regular", "esports", "multi_outcome", "proposition", "recurring_proposition"]

GameUnitLabel = Literal["Game", "Map"]

Record_string_never = dict[str, Any]

AgaraCmsEventDisplay = Record_string_never

GameStateScore = GameStateTennisScore | GameStateDefaultScore

GameStateLifecycle = Literal[
    "scheduled", "delayed", "live", "intermission", "final", "cancelled", "postponed"
]

MicroUsd = str

EventLifecycleState = Literal[
    "DRAFT", "INACTIVE", "ACTIVE", "PROPOSED", "DISPUTED", "PAUSED", "RESOLVED", "VOID", "ARCHIVED"
]

MarketState = Literal[
    "DRAFT",
    "PROVISIONED",
    "ACTIVE",
    "INACTIVE",
    "TRADING_HALT",
    "PROPOSAL_PENDING",
    "PROPOSED",
    "DISPUTED",
    "RESOLVED",
    "VOID",
    "ARCHIVED",
]

Exchange = Literal["POLYMARKET", "AGARA"]

DisplayType = Literal[
    "BINARY", "MAP_MARKET", "MULTI_OUTCOME", "OVER_UNDER", "PLAYER_PROP", "SPREAD", "TERNARY"
]

MicroProbability = str

OutcomeKind = Literal[
    "team_home",
    "team_away",
    "draw",
    "yes",
    "no",
    "over",
    "under",
    "candidate",
    "side_a",
    "side_b",
    "unknown",
]

OutcomeTeamSide = Literal["home", "away"]

PolymarketOutcomeDisplay = Record_string_never

AgaraCmsOutcomeDisplay = Record_string_never

OutcomeDisplay = BaseOutcomeDisplay

ResolutionStatus = Literal["DISPUTED", "PROPOSED", "RESOLVED", "VOID"]

ThreeWayRole = Literal["home", "draw", "away"]

AgaraCmsMarketDisplay = Record_string_never

MarketDisplay = BaseMarketDisplay

CategoryRenderHint = Literal["card_list", "lined_single_card", "multi_outcome_grid"]

EventListItem = EventListItem0 | EventListItem1 | EventListItem2 | EventListItem3

EventDetail = EventDetail0 | EventDetail1 | EventDetail2 | EventDetail3

SearchEventItem = SearchEventItem0 | SearchEventItem1 | SearchEventItem2 | SearchEventItem3

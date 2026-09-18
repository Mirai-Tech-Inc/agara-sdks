use super::{
	BaseEventDetailCategories, BaseEventDetailPrimaryCategory, BaseEventListItemPrimaryCategory,
	MainMarketGroup, MarketDetail, MarketTab, StringOrNumber,
};

/// Fields shared by event-list cards, including order acceptance and selected market.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct BaseEventListItem {
	/// Stable Agara event UUID.
	pub id: String,

	/// Stable catalogue slug used in resource links and lookups.
	pub slug: String,

	/// Human-readable event title.
	pub title: String,

	/// Whether the catalogue currently permits orders; revalidate at submission time.
	pub is_accepting_orders: bool,

	/// Whether the event is currently marked live by its source.
	pub is_live: bool,

	/// Whether this event is a grouping parent for related child events.
	pub is_group_root: bool,

	/// RFC3339 event start time; None when the source has not supplied it.
	pub start_time: Option<String>,

	/// RFC3339 event end time; None when the source has not supplied it.
	pub end_time: Option<String>,

	/// Accumulated trading volume in micro collateral.
	pub volume_micro: crate::units::Micro,

	/// Preferred category for links; None when no placement is selected.
	pub primary_category: Option<BaseEventListItemPrimaryCategory>,

	/// Selected main-market UUID; None before a main market is assigned.
	pub main_market_id: Option<String>,

	/// Current resolution lifecycle classification, separate from order settlement.
	pub resolution_status: String,

	/// Current market/event lifecycle state reported by discovery. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub state: Option<String>,

	/// Selected main market and related legs, if the response includes grouping.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub main_market_group: Option<MainMarketGroup>,
}

/// Polymarket series identity retained inside source-specific display metadata.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PolymarketEventDisplaySeries {
	/// Source-side series identifier.
	pub id: String,

	/// Stable catalogue slug used in resource links and lookups.
	pub slug: String,

	/// Source-provided series title in Polymarket’s namespace.
	pub title: String,
}

/// Optional Polymarket display metadata that is separate from trading state.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PolymarketEventDisplay {
	/// Public URL supplied by the source. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub url: Option<String>,

	/// Source-specific recurring or competition series metadata. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub series: Option<PolymarketEventDisplaySeries>,

	/// Source-provided event week, preserving text or numeric representation. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub event_week: Option<StringOrNumber>,

	/// Whether source metadata marks this event as a negative-risk group. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub neg_risk: Option<bool>,

	/// Source-side identifier of the grouping parent event. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub parent_event_id: Option<String>,

	/// Source-reported UMA resolution stages retained for display. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub uma_resolution_statuses: Option<Vec<String>>,

	/// Source-provided competitiveness metric retained without reinterpretation. None when
	/// unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub competitive: Option<f64>,
}

/// Normalized competitor identity with optional provider and presentation metadata.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SportsTeam {
	/// Human-readable name supplied by the catalogue or provider.
	pub name: String,

	/// Compact competitor name, when supplied.
	pub abbreviation: Option<String>,

	/// Artwork URL; None when no logo is available.
	pub logo_url: Option<String>,

	/// Source-provided team display color, when available.
	pub color: Option<String>,

	/// Human-readable team record, when available.
	pub record: Option<String>,

	/// Stable sport classification slug, when supplied.
	pub sport_slug: Option<String>,

	/// Stable league classification slug, when supplied.
	pub league_slug: Option<String>,

	/// Provider identifier associated with the normalized competitor. None when unavailable.
	pub provider_id: Option<String>,

	/// Competitor identifier in the source provider’s namespace. None when unavailable.
	pub source_team_id: Option<String>,
}

/// Candidate summary with probability and accumulated volume in micro units.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct EventDisplayTopCandidates {
	/// Human-readable name supplied by the catalogue or provider.
	pub name: String,

	/// Outcome price in micro collateral per share; one whole unit equals 1,000,000 micros. None when
	/// unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub price_micro: Option<crate::units::Micro>,

	/// Accumulated trading volume in micro collateral.
	pub volume_micro: crate::units::Micro,
}

/// Sparse event presentation metadata with unrecognized CMS properties preserved.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct EventDisplay {
	/// Unrecognized CMS properties retained without assigning trading behavior to them.
	#[serde(flatten)]
	pub extensions: std::collections::BTreeMap<String, serde_json::Value>,

	/// Optional primary artwork URL.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub image_url: Option<String>,

	/// Optional icon artwork URL.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub icon_url: Option<String>,

	/// Human-readable description supplied by the catalogue. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub description: Option<String>,

	/// Secondary presentation text below the event title. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub subtext: Option<String>,

	/// Source-selected date text for presentation, not a timestamp contract. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub display_date: Option<String>,

	/// Source-supplied liquidity for display in whole units, not micro units. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub liquidity: Option<f64>,

	/// Whether the event contains home/draw/away moneyline legs. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub has_three_way_markets: Option<bool>,

	/// Whether current proposition-market sections are available. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub has_prop_markets: Option<bool>,

	/// Whether legacy proposition-market sections are available. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub has_legacy_prop_markets: Option<bool>,

	/// Source-provided market-category labels for presentation. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub market_categories: Option<Vec<String>>,

	/// Source-provided presentation and grouping tags. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub tags: Option<Vec<String>>,

	/// Human-readable resolution authority or source description. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub resolution_source: Option<String>,

	/// Event layout discriminator used by presentation clients. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub layout: Option<String>,

	/// Label for a game/map selector, typically Game or Map. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub game_unit_label: Option<String>,

	/// Selected main spread-market UUID, when configured.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub spreads_main_market_id: Option<String>,

	/// Selected main totals-market UUID, when configured.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub totals_main_market_id: Option<String>,

	/// Source-specific Polymarket presentation data; preserve its namespace. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub polymarket: Option<PolymarketEventDisplay>,

	/// Source-specific Agara CMS properties, retained without interpreting unknown keys. None when
	/// unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub agara_cms: Option<std::collections::BTreeMap<String, serde_json::Value>>,

	/// Normalized identity of the event’s first competitor, when supplied.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub team_a: Option<SportsTeam>,

	/// Normalized identity of the event’s second competitor, when supplied.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub team_b: Option<SportsTeam>,

	/// Stable sport classification slug, when supplied.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub sport_slug: Option<String>,

	/// Stable league classification slug, when supplied.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub league_slug: Option<String>,

	/// Human-readable competition format, such as a best-of series. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub format_label: Option<String>,

	/// Selected main spread in sport-specific whole units. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub spreads_main_line: Option<f64>,

	/// Selected main total in sport-specific whole units. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub totals_main_line: Option<f64>,

	/// Additional event presentation subcategory, when supplied.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub subcategory: Option<String>,

	/// Total candidates represented by a multi-outcome event. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub candidate_count: Option<i64>,

	/// Candidates whose markets remain active. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub active_candidate_count: Option<i64>,

	/// Leading candidate’s display name, when available.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub top_candidate_name: Option<String>,

	/// Leading candidate’s probability in micro units. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub top_candidate_price_micro: Option<crate::units::Micro>,

	/// Leading candidate summaries in presentation order. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub top_candidates: Option<Vec<EventDisplayTopCandidates>>,

	/// Display name of the resolved winner, when known.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub resolved_winner_name: Option<String>,

	/// Whether source metadata classifies this as a future event. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub is_future: Option<bool>,

	/// Whether this event belongs to a grouping parent. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub is_child_event: Option<bool>,

	/// Stable recurring-event series slug, when available.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub series_slug: Option<String>,

	/// Nominal interval between recurring market cycles, in seconds. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub cadence_seconds: Option<i64>,
}

/// Match-level score for the event’s first and second competitors.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GameStateScoreMatch {
	/// Score for the event’s first competitor.
	pub a: i64,

	/// Score for the event’s second competitor.
	pub b: i64,
}

/// Tiebreak points for the first and second competitors within one tennis set.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GameStateScoreSetsTiebreak {
	/// Score for the event’s first competitor.
	pub a: i64,

	/// Score for the event’s second competitor.
	pub b: i64,
}

/// Games won in one tennis set, with optional tiebreak details.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GameStateScoreSets {
	/// Score for the event’s first competitor.
	pub a: i64,

	/// Score for the event’s second competitor.
	pub b: i64,

	/// Tiebreak points for this set, when reported.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub tiebreak: Option<GameStateScoreSetsTiebreak>,
}

/// Sport-tagged match score and optional tennis set breakdown.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GameStateScore {
	/// Sport discriminator determining how to interpret the score fields.
	pub sport: String,

	/// Match-level score for the first and second competitors.
	pub r#match: GameStateScoreMatch,

	/// Tennis set scores in play order; absent for other sports or unavailable scores.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub sets: Option<Vec<GameStateScoreSets>>,
}

/// Live match lifecycle and optional human-readable period label.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GameStateMatch {
	/// Match stage, such as scheduled, live, intermission, final or cancelled.
	pub lifecycle: String,

	/// Human-readable current period or live-state label, when supplied.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub live_label: Option<String>,
}

/// Provider game identity and the currently available score/lifecycle updates.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GameState {
	/// Provider game identifier used to correlate live score updates.
	pub id: i64,

	/// Latest reported sport-specific score; absent before a usable score is available.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub score: Option<GameStateScore>,

	/// Latest reported match lifecycle; absent when the provider has not supplied it.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub r#match: Option<GameStateMatch>,
}

/// Lean outcome metadata used by optionally enriched listing cards.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ListingMarketOutcome {
	/// Human-readable label for this catalogue entry.
	pub label: String,

	/// Compact outcome label; None when no alternate label is supplied.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub short_label: Option<String>,

	/// Semantic outcome kind supplied by the catalogue. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub kind: Option<String>,

	/// Outcome price in micro collateral per share; one whole unit equals 1,000,000 micros. None when
	/// unavailable.
	pub price_micro: Option<crate::units::Micro>,

	/// On-chain outcome token identifier used by trading and quote subscriptions. None when
	/// unavailable.
	pub token_id: Option<String>,
}

/// Lean market identity, resolution-feed context and outcomes for listing enrichment.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ListingMarket {
	/// Stable Agara market UUID of this lean listing row.
	pub id: String,

	/// Current market/event lifecycle state reported by discovery. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub state: Option<String>,

	/// Stable catalogue slug used in resource links and lookups.
	pub slug: String,

	/// Exchange that owns this market, account balance or trading record.
	pub exchange: crate::ids::Exchange,

	/// The market’s human-readable resolution question.
	pub question: String,

	/// Server-computed market label suitable for a listing card.
	pub display_label: String,

	/// Pyth Hermes feed identifier pinned to the market, when configured.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub pyth_feed_id: Option<String>,

	/// Provider-native Pyth Pro symbol used for live price subscriptions. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub pyth_pro_symbol: Option<String>,

	/// Canonical underlying symbol used with security_provider for REST price reads. None when
	/// unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub security_symbol: Option<String>,

	/// Primary provider slug used with security_symbol for the settlement-lineage feed. None when
	/// unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub security_provider: Option<String>,

	/// Exact decimal reference value for the cycle; not a micro-unit integer. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub reference_price: Option<String>,

	/// ISO currency of the referenced venue or quote; None for a unitless instrument.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub currency: Option<String>,

	/// RFC3339 time at which the recurring cycle’s reference is observed. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub reference_at: Option<String>,

	/// RFC3339 time at which the recurring cycle’s final observation is taken. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub observe_at: Option<String>,

	/// Ordered outcome identities and available probability metadata.
	pub outcomes: Vec<ListingMarketOutcome>,
}

/// An event card with its type-specific display and optional lean market enrichment.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct EventListItem {
	/// Flattened common event-card fields; no extra nesting appears on the wire.
	#[serde(flatten)]
	pub base: BaseEventListItem,

	/// Event family, such as GAME, MULTI_OUTCOME, PROPOSITION or RECURRING_PROPOSITION.
	pub event_type: String,

	/// Number of markets represented by the event; None when this card omits the count.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub market_count: Option<i64>,

	/// Structured presentation metadata; it is not authoritative order or settlement state.
	pub display: EventDisplay,

	/// Current provider game score/lifecycle; None for non-game or unreported state.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub game_state: Option<GameState>,

	/// Lean market enrichment returned when include_markets is requested.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub listing_markets: Option<Vec<ListingMarket>>,
}

/// Shared event-detail fields with full markets and grouping references.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct BaseEventDetail {
	/// Stable Agara event UUID.
	pub id: String,

	/// Stable catalogue slug used in resource links and lookups.
	pub slug: String,

	/// Human-readable event title.
	pub title: String,

	/// Whether the catalogue currently permits orders; revalidate at submission time.
	pub is_accepting_orders: bool,

	/// Whether the event is currently marked live by its source.
	pub is_live: bool,

	/// Whether this event is a grouping parent for related child events.
	pub is_group_root: bool,

	/// RFC3339 event start time; None when the source has not supplied it.
	pub start_time: Option<String>,

	/// RFC3339 event end time; None when the source has not supplied it.
	pub end_time: Option<String>,

	/// Accumulated trading volume in micro collateral.
	pub volume_micro: crate::units::Micro,

	/// Selected main-market UUID; None before a main market is assigned.
	pub main_market_id: Option<String>,

	/// Current resolution lifecycle classification, separate from order settlement.
	pub resolution_status: String,

	/// Current market/event lifecycle state reported by discovery. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub state: Option<String>,

	/// Category placements or matches included in this response.
	pub categories: Vec<BaseEventDetailCategories>,

	/// Preferred category for links; None when no placement is selected.
	pub primary_category: Option<BaseEventDetailPrimaryCategory>,

	/// Full market rows for the event; all tab/group references resolve into this collection.
	pub markets: Vec<MarketDetail>,

	/// Optional sporting family used to group the event’s market tabs.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub sport_group: Option<String>,

	/// Selected main market and related legs, if the response includes grouping.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub main_market_group: Option<MainMarketGroup>,

	/// Featured spread markets included for event-detail presentation. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub featured_spread_markets: Option<Vec<MarketDetail>>,

	/// Featured totals market, when the event supplies one.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub featured_totals_market: Option<MarketDetail>,

	/// Ordered tab/section references whose market IDs resolve in this event’s markets. None when
	/// unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub market_tabs: Option<Vec<MarketTab>>,

	/// Number of included markets whose outcome is currently disputed. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub disputed_market_count: Option<i64>,

	/// Number of included markets that have completed resolution. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub resolved_market_count: Option<i64>,

	/// Related event cards supplied with event details.
	pub related_events: Vec<EventListItem>,
}

/// Full event discovery result with type-specific display and optional game state.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct EventDetail {
	/// Flattened common event-detail fields; no extra nesting appears on the wire.
	#[serde(flatten)]
	pub base: BaseEventDetail,

	/// Event family, such as GAME, MULTI_OUTCOME, PROPOSITION or RECURRING_PROPOSITION.
	pub event_type: String,

	/// Structured presentation metadata; it is not authoritative order or settlement state.
	pub display: EventDisplay,

	/// Current provider game score/lifecycle; None for non-game or unreported state.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub game_state: Option<GameState>,
}

/// Category heading for a grouped event-list section.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct EventListSectionCategory {
	/// Stable catalogue slug used in resource links and lookups.
	pub slug: String,

	/// Human-readable label for this catalogue entry.
	pub label: String,
}

/// Ordered event IDs grouped under a category heading.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct EventListSection {
	/// Category identity and display label for this grouped list section.
	pub category: EventListSectionCategory,

	/// Event UUIDs in section display order; resolve them from the enclosing event list.
	pub event_ids: Vec<String>,
}

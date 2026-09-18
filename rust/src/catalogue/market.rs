const GRID_TWO_COLUMNS: u8 = 2;
const GRID_THREE_COLUMNS: u8 = 3;
const GRID_FOUR_COLUMNS: u8 = 4;

/// Sparse outcome presentation metadata; trading identity remains in the outcome row.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct BaseOutcomeDisplay {
	/// Human-readable label for this catalogue entry. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub label: Option<String>,

	/// Semantic outcome kind supplied by the catalogue. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub kind: Option<String>,

	/// Compact outcome label; None when no alternate label is supplied.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub short_label: Option<String>,

	/// First or second competitor associated with this outcome. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub team_side: Option<String>,

	/// Presentation side associated with this outcome, when supplied.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub side: Option<String>,

	/// Sport/statistic threshold in native market units, not micro probability. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub threshold: Option<f64>,

	/// Outcome title chosen for an order-entry display. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub bet_slip_title: Option<String>,

	/// Human-readable resolution result for this outcome. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub resolution_label: Option<String>,

	/// Source-specific Polymarket presentation data; preserve its namespace. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub polymarket: Option<std::collections::BTreeMap<String, serde_json::Value>>,

	/// Source-specific Agara CMS properties, retained without interpreting unknown keys. None when
	/// unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub agara_cms: Option<std::collections::BTreeMap<String, serde_json::Value>>,
}

/// One market outcome with token identity, probability and optional live quote enrichment.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct MarketOutcome {
	/// Stable Agara outcome UUID; distinct from the outcome’s chain token_id.
	pub id: String,

	/// Zero-based position of this outcome within the market.
	pub index: i64,

	/// Human-readable label for this catalogue entry.
	pub label: String,

	/// Outcome price in micro collateral per share; one whole unit equals 1,000,000 micros. None when
	/// unavailable.
	pub price_micro: Option<crate::units::Micro>,

	/// On-chain outcome token identifier used by trading and quote subscriptions. None when
	/// unavailable.
	pub token_id: Option<String>,

	/// Best known bid in micro probability units; None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub best_bid_micro: Option<crate::units::Micro>,

	/// Best known ask in micro probability units; None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub best_ask_micro: Option<crate::units::Micro>,

	/// Ask-minus-bid spread in micro probability units; None when either side is missing.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub spread_micro: Option<crate::units::Micro>,

	/// Semantic outcome kind supplied by the catalogue. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub kind: Option<String>,

	/// Compact outcome label; None when no alternate label is supplied.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub short_label: Option<String>,

	/// First or second competitor associated with this outcome. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub team_side: Option<String>,

	/// Presentation side associated with this outcome, when supplied.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub side: Option<String>,

	/// Sport/statistic threshold in native market units, not micro probability. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub threshold: Option<f64>,

	/// Resolved payout price in micro units; None before a payout is established.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub resolution_price_micro: Option<crate::units::Micro>,

	/// Outcome title chosen for an order-entry display. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub bet_slip_title: Option<String>,

	/// Human-readable resolution result for this outcome. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub resolution_label: Option<String>,

	/// Structured presentation metadata; it is not authoritative order or settlement state. None when
	/// unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub display: Option<BaseOutcomeDisplay>,
}

/// AGARA price/size scales and tick limits in engine-native integer units.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AgaraEngineConfig {
	/// Smallest price increment in native engine price units.
	pub tick_size: i64,

	/// Native price units per one collateral unit; divide prices by this scale.
	pub price_scale: i64,

	/// Native quantity units per whole share; divide sizes by this scale.
	pub size_scale: i64,

	/// Lowest accepted price in native engine units.
	pub min_price: i64,

	/// Highest accepted price in native engine units.
	pub max_price: i64,
}

/// Canonical settlement evidence, populated only when a resolution attempt is available.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct MarketResolution {
	/// Canonical resolution source or provider identifier.
	pub source: String,

	/// Exact decimal reference used for settlement; None when not applicable.
	pub reference_value: Option<String>,

	/// Exact decimal final observation used for settlement, when available.
	pub observed_value: Option<String>,

	/// RFC3339 time of the settlement observation, when available.
	pub observed_at: Option<String>,
}

/// Current-epoch reward terms attached to a market-detail response.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct MarketLpIncentive {
	/// Reward pool in micro collateral for this market and reward epoch.
	pub pool_micro: crate::units::Micro,

	/// Maximum qualifying distance from midpoint in micro probability units.
	pub max_spread_micro: i64,

	/// Minimum qualifying resting quantity in micro shares.
	pub min_shares_micro: String,
}

/// Named public evidence source used to explain market resolution.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ResolutionSourceLink {
	/// Human-readable label for the linked resolution evidence.
	pub title: String,

	/// Public URL supplied by the source.
	pub url: String,
}

/// Optional Polymarket resolution/deployment metadata kept under its source namespace.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PolymarketMarketDisplay {
	/// Source-reported UMA resolution stages retained for display. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub uma_resolution_statuses: Option<Vec<String>>,

	/// Source-reported UMA resolution deadline, when present.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub uma_end_date: Option<String>,

	/// Source-provided resolver identity, when available.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub resolved_by: Option<String>,

	/// Source-reported deployment timestamp, when available.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub deployed_at: Option<String>,
}

/// Sparse market presentation and resolution-feed metadata.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct BaseMarketDisplay {
	/// Optional primary artwork URL.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub image_url: Option<String>,

	/// Optional market title specialized for portfolio presentation.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub portfolio_title: Option<String>,

	/// Human-readable description supplied by the catalogue. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub description: Option<String>,

	/// Human-readable resolution authority or source description. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub resolution_source: Option<String>,

	/// Human-readable conditions used to resolve the market. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub resolution_criteria: Option<String>,

	/// Named public evidence links for the market’s resolution. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub resolution_sources: Option<Vec<ResolutionSourceLink>>,

	/// Player identity for a player-proposition market. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub player_name: Option<String>,

	/// Statistic measured by a player-proposition market. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub stat_type: Option<String>,

	/// Source-supplied liquidity for display in whole units, not micro units. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub liquidity: Option<f64>,

	/// Nominal interval between recurring market cycles, in seconds. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub cadence_seconds: Option<i64>,

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

	/// Ordering index of this market’s strike or value band. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub band_index: Option<i64>,

	/// Exact decimal lower band boundary, when present.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub band_lower: Option<String>,

	/// Exact decimal upper band boundary, when present.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub band_upper: Option<String>,

	/// Human-readable label for a strike or value band. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub band_label: Option<String>,

	/// Pyth Hermes feed identifier pinned to the market, when configured.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub pyth_feed_id: Option<String>,

	/// Pyth historical benchmark symbol associated with the market. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub pyth_benchmark_symbol: Option<String>,

	/// Optional TradingView ticker used to identify the displayed underlying.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub tradingview_symbol: Option<String>,

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

	/// Per-game or per-map index for esports markets; None when not applicable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub game_number: Option<i64>,

	/// Whether this market is one leg of a three-way moneyline group. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub is_three_way: Option<bool>,

	/// Home, draw or away role within a three-way moneyline group. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub three_way_role: Option<String>,

	/// Server-computed market label suitable for a listing card. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub display_label: Option<String>,

	/// Source-specific Polymarket presentation data; preserve its namespace. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub polymarket: Option<PolymarketMarketDisplay>,

	/// Source-specific Agara CMS properties, retained without interpreting unknown keys. None when
	/// unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub agara_cms: Option<std::collections::BTreeMap<String, serde_json::Value>>,
}

/// Market identity, tradability, outcomes and resolution data returned by discovery.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct MarketDetail {
	/// Stable Agara market UUID used by market-detail lookups.
	pub id: String,

	/// Current market/event lifecycle state reported by discovery. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub state: Option<String>,

	/// Exchange that owns this market, account balance or trading record.
	pub exchange: crate::ids::Exchange,

	/// Stable catalogue slug used in resource links and lookups.
	pub slug: String,

	/// The market’s human-readable resolution question.
	pub question: String,

	/// Optional display classification for the market’s layout.
	pub display_type: Option<String>,

	/// Source-specific market taxonomy, distinct from the exchange identity. None when unavailable.
	pub source_market_type: Option<String>,

	/// Ordered outcome identities and available probability metadata.
	pub outcomes: Vec<MarketOutcome>,

	/// Whether the catalogue currently permits orders; revalidate at submission time.
	pub is_accepting_orders: bool,

	/// RFC3339 operational halt time; None when no administrative halt is recorded.
	pub halted_at: Option<String>,

	/// Public explanation of an operational halt; None when unreported.
	pub halted_reason: Option<String>,

	/// Accumulated trading volume in micro collateral.
	pub volume_micro: crate::units::Micro,

	/// Last synchronized liquidity as whole-unit decimal text; not a micro amount.
	pub liquidity: String,

	/// Trading volume over the past twenty-four hours in micro collateral.
	pub volume_24h_micro: crate::units::Micro,

	/// Minimum probability-price increment in micro units, when known.
	pub tick_size_micro: Option<crate::units::Micro>,

	/// Engine-native limits for an anchored AGARA market; None for other or unanchored markets.
	pub engine_config: Option<AgaraEngineConfig>,

	/// Minimum order notional in micro collateral; None when unavailable.
	pub min_order_size_micro: Option<crate::units::Micro>,

	/// Exact decimal handicap, total or threshold in sport-specific units, not micros. None when
	/// unavailable.
	pub line: Option<String>,

	/// Current resolution lifecycle classification, separate from order settlement. None when
	/// unavailable.
	pub resolution_status: Option<String>,

	/// Server-derived winning outcome label; None while unresolved or void.
	pub resolution_outcome: Option<String>,

	/// RFC3339 resolution timestamp; None while unresolved.
	pub resolved_at: Option<String>,

	/// Canonical settlement evidence; None on lean cards or before evidence is available.
	pub resolution: Option<MarketResolution>,

	/// RFC3339 archival timestamp; None when the market is not archived.
	pub archived_at: Option<String>,

	/// On-chain condition identifier used by position and settlement operations. None when
	/// unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub condition_id: Option<String>,

	/// Shared negative-risk group identifier; None for an ungrouped market.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub neg_risk_id: Option<String>,

	/// Home, draw or away role within a three-way moneyline group. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub three_way_role: Option<String>,

	/// Whether this market is one leg of a three-way moneyline group. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub is_three_way: Option<bool>,

	/// Per-game or per-map index for esports markets; None when not applicable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub game_number: Option<i64>,

	/// Server-computed market label suitable for a listing card.
	pub display_label: String,

	/// Current reward terms; absent on lean cards or when no pool is funded.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub lp_incentive: Option<MarketLpIncentive>,

	/// Structured presentation metadata; it is not authoritative order or settlement state.
	pub display: BaseMarketDisplay,
}

/// Parent-event identity embedded in a market-discovery row.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct MarketsListEventRef {
	/// Stable Agara UUID of the parent event.
	pub id: String,

	/// Stable catalogue slug used in resource links and lookups.
	pub slug: String,

	/// Human-readable parent-event title.
	pub title: String,

	/// Whether this registered resource is currently enabled.
	pub active: bool,
}

/// Market details with enough parent-event context to avoid a follow-up lookup.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct MarketsListItem {
	/// Flattened market-detail fields; no extra nesting appears on the wire.
	#[serde(flatten)]
	pub base: MarketDetail,

	/// Parent event context included with this market row.
	pub event: MarketsListEventRef,
}

/// Cursor-paginated market-discovery rows with parent-event context.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct MarketsListResponse {
	/// Matching market rows with parent-event context for this cursor page.
	pub markets: Vec<MarketsListItem>,

	/// Applied page size and opaque continuation token for the next request.
	pub pagination: crate::models::CursorPagination,
}

/// One category ancestor in the event’s navigation hierarchy.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct BaseEventDetailCategoriesPath {
	/// Stable catalogue slug used in resource links and lookups.
	pub slug: String,

	/// Human-readable label for this catalogue entry.
	pub label: String,
}

/// Category placement with root identity and its ordered ancestor path.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct BaseEventDetailCategories {
	/// Stable catalogue slug used in resource links and lookups.
	pub slug: String,

	/// Human-readable label for this catalogue entry.
	pub label: String,

	/// Stable slug of the root of this category hierarchy.
	pub root_slug: String,

	/// Category ancestors in navigation order.
	pub path: Vec<BaseEventDetailCategoriesPath>,
}

/// Preferred category identity used when linking to an event.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct BaseEventDetailPrimaryCategory {
	/// Stable catalogue slug used in resource links and lookups.
	pub slug: String,

	/// Human-readable label for this catalogue entry.
	pub label: String,
}

/// Selected main market or related three-way legs in display order.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct MainMarketGroup {
	/// Shared negative-risk group identifier; None for an ungrouped market.
	pub neg_risk_id: Option<String>,

	/// Selected market or sibling three-way legs in display order; usually one or three rows.
	pub markets: Vec<MarketDetail>,
}

/// Ordered market references forming one display group; IDs resolve in the event’s markets.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct MarketGroup {
	/// Stable display-group key, independent of its current ordering.
	pub key: String,

	/// Market UUIDs in display order; resolve them from the enclosing event’s markets.
	pub market_ids: Vec<String>,
}

/// Numeric column-count hint admitted by the catalogue’s multi-outcome layout contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GridColumnsHint {
	/// Suggest two columns for the multi-outcome grid.
	Two,

	/// Suggest three columns for the multi-outcome grid.
	Three,

	/// Suggest four columns for the multi-outcome grid.
	Four,
}

/// Optional line-selector and multi-outcome grid presentation hints.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CategorySectionMeta {
	/// Whether selection varies by numeric line or esports game_number. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub selector_axis: Option<String>,

	/// Preferred initial line; None when no anchor exists.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub default_line: Option<f64>,

	/// Whether the stored line belongs to the first or second competitor. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub line_owner_side: Option<String>,

	/// Suggested multi-outcome grid column count; presentation-only. None when unavailable.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub grid_cols_hint: Option<GridColumnsHint>,
}

/// One ordered market section with its display strategy and referenced groups.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CategorySection {
	/// Stable display-group key, independent of its current ordering.
	pub key: String,

	/// Human-readable label for this catalogue entry.
	pub label: String,

	/// Ascending presentation order within the enclosing tab or section.
	pub order: i64,

	/// Section layout: card_list, lined_single_card or multi_outcome_grid.
	pub render_hint: String,

	/// Ordered groups of market references included in the section.
	pub groups: Vec<MarketGroup>,

	/// Optional configuration used only by layouts that need extra rendering hints.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub meta: Option<CategorySectionMeta>,
}

/// An ordered event-detail tab containing market sections.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct MarketTab {
	/// Stable display-group key, independent of its current ordering.
	pub key: String,

	/// Human-readable label for this catalogue entry.
	pub label: String,

	/// Ascending presentation order within the enclosing tab or section.
	pub order: i64,

	/// Ordered category/market sections included in this page or tab.
	pub sections: Vec<CategorySection>,
}

/// Preferred category identity included with an event-list card.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct BaseEventListItemPrimaryCategory {
	/// Stable catalogue slug used in resource links and lookups.
	pub slug: String,

	/// Human-readable label for this catalogue entry.
	pub label: String,
}

impl GridColumnsHint {
	/// Return the wire-level numeric column count.
	pub const fn columns(self) -> u8 {
		match self {
			Self::Two => GRID_TWO_COLUMNS,
			Self::Three => GRID_THREE_COLUMNS,
			Self::Four => GRID_FOUR_COLUMNS,
		}
	}
}

impl serde::Serialize for GridColumnsHint {
	fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		serializer.serialize_u8(self.columns())
	}
}

impl<'de> serde::Deserialize<'de> for GridColumnsHint {
	fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		let count = <u8 as serde::Deserialize>::deserialize(deserializer)?;

		match count {
			GRID_TWO_COLUMNS => Ok(Self::Two),
			GRID_THREE_COLUMNS => Ok(Self::Three),
			GRID_FOUR_COLUMNS => Ok(Self::Four),
			_ => Err(serde::de::Error::invalid_value(
				serde::de::Unexpected::Unsigned(u64::from(count)),
				&"a grid column count of 2, 3, or 4",
			)),
		}
	}
}

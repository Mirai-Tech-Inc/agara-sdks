use super::{EventListItem, EventListSection};

/// An event-discovery page, its continuation cursor and optional category grouping.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct EventsListResponse {
	/// Event listing cards in the requested order for this cursor page.
	pub events: Vec<EventListItem>,

	/// Applied page size and opaque continuation token for the next request.
	pub pagination: crate::models::CursorPagination,

	/// Optional category groupings referencing IDs in the returned event list.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub sections: Option<Vec<EventListSection>>,
}

/// Canonical event slug and an optional redirect when the requested alias is stale.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct EventCanonicalUrlResponse {
	/// Canonical redirect URL; None when no redirect is needed.
	pub redirect_url: Option<String>,

	/// Canonical event slug after resolving aliases or stale source slugs.
	pub canonical_slug: String,
}

/// Event families that a catalogue category supports.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CategoryResourceEventTypes {
	/// Whether the category supports sporting or other game events.
	pub game: bool,

	/// Whether the category supports events with multiple candidate outcomes.
	pub multi_outcome: bool,

	/// Whether the category supports proposition events.
	pub proposition: bool,
}

/// Category identity, display label and supported event families.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CategoryResource {
	/// Stable catalogue category UUID.
	pub id: String,

	/// Stable catalogue slug used in resource links and lookups.
	pub slug: String,

	/// Human-readable label for this catalogue entry.
	pub label: String,

	/// Event families available in this category.
	pub event_types: CategoryResourceEventTypes,
}

/// An active intraday interval expressed as HH:MM in the venue’s local timezone.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SessionSegment {
	/// Session start as HH:MM in the venue’s timezone.
	pub open: String,

	/// Session end as HH:MM in the venue’s timezone.
	pub close: String,
}

/// Registered trading venue, local timezone and regular session schedule.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TradingVenue {
	/// Canonical listing-venue market identifier code, such as XNYS.
	pub mic: String,

	/// Human-readable name supplied by the catalogue or provider.
	pub name: String,

	/// IANA timezone used to interpret the venue’s wall-clock sessions.
	pub timezone: String,

	/// Venue country code.
	pub country: String,

	/// ISO currency used by the trading venue.
	pub currency: String,

	/// Regular daily trading intervals in venue-local wall-clock time.
	pub default_segments: Vec<SessionSegment>,

	/// Whether this registered resource is currently enabled.
	pub active: bool,
}

/// The registered trading-calendar venues.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TradingVenuesResponse {
	/// Registered venue records with their regular calendars.
	pub venues: Vec<TradingVenue>,
}

/// One venue/date session; an empty schedule means closed, while a missing row is a 404.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TradingDay {
	/// Canonical listing-venue market identifier code, such as XNYS.
	pub mic: String,

	/// Calendar session date in YYYY-MM-DD format.
	pub date: String,

	/// Whether this covered date has at least one active trading interval.
	pub is_trading_day: bool,

	/// Actual active intervals for this date; empty on a covered closed day.
	pub segments: Vec<SessionSegment>,

	/// Holiday, early-close or outage explanation; None on an ordinary covered day.
	pub reason: Option<String>,
}

/// Available daily sessions for one venue over an inclusive calendar range.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TradingDaysResponse {
	/// Canonical listing-venue market identifier code, such as XNYS.
	pub mic: String,

	/// Covered daily sessions within the requested inclusive range.
	pub days: Vec<TradingDay>,
}

/// Earliest trading session on or after the requested date, when coverage permits.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct NextSessionResponse {
	/// Canonical listing-venue market identifier code, such as XNYS.
	pub mic: String,

	/// Earliest covered trading day on or after the requested date; None if none is found.
	pub session: Option<TradingDay>,
}

/// Canonical underlying instrument identity linked to its listing venue.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Security {
	/// Canonical security symbol, such as BTC-USD, distinct from provider-native feed IDs.
	pub symbol: String,

	/// Human-readable label for this catalogue entry.
	pub label: String,

	/// Canonical listing-venue market identifier code, such as XNYS.
	pub mic: String,

	/// Instrument class such as equity, index, crypto, fx or commodity.
	pub asset_class: String,

	/// Whether this registered resource is currently enabled.
	pub active: bool,
}

/// Canonical underlyings available in the securities master.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SecuritiesResponse {
	/// Canonical underlying instruments returned by the securities master.
	pub securities: Vec<Security>,
}

/// One provider’s native feed identifier for a canonical security.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SecurityProvider {
	/// Provider slug paired with the canonical security symbol for price discovery.
	pub provider: String,

	/// Provider-native feed identifier for this canonical security.
	pub provider_symbol: String,

	/// Whether this registered resource is currently enabled.
	pub active: bool,
}

/// Canonical security plus provider mappings and its venue’s regular schedule.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SecurityDetail {
	/// Flattened canonical security identity and venue fields; no extra nesting appears on the wire.
	#[serde(flatten)]
	pub base: Security,

	/// Available provider mappings for the canonical security.
	pub providers: Vec<SecurityProvider>,

	/// IANA timezone of the security’s listing venue.
	pub venue_timezone: String,

	/// Regular listing-venue intervals in local HH:MM wall-clock time.
	pub venue_default_segments: Vec<SessionSegment>,
}

/// An event search hit with normal listing fields and comparable relevance.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SearchEventItem {
	/// Flattened event-list fields; no extra nesting appears on the wire.
	#[serde(flatten)]
	pub base: EventListItem,

	/// Comparable search relevance between zero and one; indirect matches may score zero.
	pub relevance_score: f64,
}

/// A normalized search relevance score shared by event and market hits.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SearchRelevance {
	/// Comparable search relevance between zero and one; indirect matches may score zero.
	pub relevance_score: f64,
}

/// Minimal parent-event context included with a market search hit.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SearchMarketEventRef {
	/// Stable catalogue slug used in resource links and lookups.
	pub slug: String,

	/// Human-readable parent-event title.
	pub title: String,

	/// Optional primary artwork URL.
	pub image_url: Option<String>,
}

/// Leading outcome label and optional probability in micro units.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SearchOutcome {
	/// Human-readable label for this catalogue entry.
	pub label: String,

	/// Outcome price in micro collateral per share; one whole unit equals 1,000,000 micros. None when
	/// unavailable.
	pub price_micro: Option<crate::units::Micro>,
}

/// A market search hit with its parent event and leading outcome prices.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SearchMarketItem {
	/// Flattened search relevance; no extra nesting appears on the wire.
	#[serde(flatten)]
	pub base: SearchRelevance,

	/// Stable Agara market UUID returned by the search index.
	pub id: String,

	/// Stable catalogue slug used in resource links and lookups.
	pub slug: String,

	/// The market’s human-readable resolution question.
	pub question: String,

	/// Whether the catalogue currently permits orders; revalidate at submission time.
	pub is_accepting_orders: bool,

	/// Accumulated trading volume in micro collateral.
	pub volume_micro: crate::units::Micro,

	/// Parent event context included with this market row.
	pub event: SearchMarketEventRef,

	/// Ordered outcome identities and available probability metadata.
	pub outcomes: Vec<SearchOutcome>,
}

/// Category identity included in discovery search results.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SearchCategoryRef {
	/// Stable catalogue slug used in resource links and lookups.
	pub slug: String,

	/// Human-readable label for this catalogue entry.
	pub label: String,
}

/// A matching catalogue category with optional icon metadata.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SearchCategoryItem {
	/// Flattened category identity; no extra nesting appears on the wire.
	#[serde(flatten)]
	pub base: SearchCategoryRef,

	/// Category icon metadata; None when not configured.
	pub icon: Option<String>,
}

/// Matching events, markets and categories from a single discovery query.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SearchResponse {
	/// Matching enriched event cards with normalized relevance scores.
	pub events: Vec<SearchEventItem>,

	/// Matching markets with parent-event context and leading outcomes.
	pub markets: Vec<SearchMarketItem>,

	/// Matching category identities and available icon metadata.
	pub categories: Vec<SearchCategoryItem>,
}

/// One historical underlying quote; mantissa and exponent retain exact decimal value.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PricePoint {
	/// Approximate whole-unit display price; use mantissa and expo for an exact value.
	pub value: f64,

	/// Exact signed integer price mantissa, encoded as decimal text.
	pub mantissa: String,

	/// Base-ten exponent; exact price is mantissa multiplied by 10 raised to this exponent.
	pub expo: i64,

	/// Provider publication time in microseconds since the Unix epoch.
	pub publish_time_micros: i64,
}

/// Parallel timestamp/value arrays for an underlying feed or outcome-price history.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PriceHistory {
	/// Price-history status discriminator; successful current responses use `ok`.
	pub s: String,

	/// Unix-second timestamps corresponding index-for-index with `c`.
	pub t: Vec<i64>,

	/// Whole-unit price values corresponding index-for-index with `t`.
	pub c: Vec<f64>,

	/// Outcome history’s latest trade time in Unix seconds; None when absent or untraded.
	#[serde(rename = "latestTradeAt")]
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub latest_trade_at: Option<i64>,
}

/// Filters and cursor pagination for AGARA market discovery.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, Default)]
pub struct MarketsQuery {
	/// Catalogue market source; current endpoint accepts lowercase agara.
	pub source: String,

	/// Parent event’s stable slug for discovery lookups and links. None omits this parameter.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub event_slug: Option<String>,

	/// Current market/event lifecycle state reported by discovery. None omits this parameter.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub state: Option<String>,

	/// Opaque continuation from the previous page; omit to start a new walk.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub cursor: Option<String>,

	/// Page size from 1 to 256; server default 64. None omits this parameter.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub limit: Option<i64>,
}

/// Complete event-discovery filters, cursor pagination and optional market enrichment.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, Default)]
pub struct EventsQuery {
	/// Page size from 1 to 100; server default 20. None omits this parameter.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub limit: Option<i64>,

	/// Opaque continuation from the previous page; omit to start a new walk.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub cursor: Option<String>,

	/// Category-tree slug; omit to search globally.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub category: Option<String>,

	/// Root hierarchy slug used to disambiguate category placement. None omits this parameter.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub root: Option<String>,

	/// Optional games or props grouping; omit to include all event families.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub event_type_bucket: Option<String>,

	/// Exchange filter, normalized by the server to AGARA or POLYMARKET. None omits this parameter.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub source: Option<String>,

	/// Registered event-filter key, such as sports_live or sports_futures. None omits this parameter.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub filter: Option<String>,

	/// Whether to omit ended events; resolved-state filters override this server-side.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub exclude_ended: Option<bool>,

	/// Resolution filter: all, active, proposed, disputed or resolved. None omits this parameter.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub resolution: Option<String>,

	/// Event order: time, volume or markets. None omits this parameter.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub sort: Option<String>,

	/// Request lean market/outcome enrichment with each listing card. None omits this parameter.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub include_markets: Option<bool>,
}

/// Canonical security/provider and Unix second for a historical quote.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, Default)]
pub struct PricePointQuery {
	/// Canonical security symbol, such as BTC-USD, distinct from provider-native feed IDs.
	pub symbol: String,

	/// Provider slug paired with the canonical security symbol for price discovery.
	pub provider: String,

	/// Historical lookup time in Unix seconds.
	pub at: i64,
}

/// Canonical security/provider and inclusive Unix-second history boundaries.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, Default)]
pub struct PriceTicksQuery {
	/// Canonical security symbol, such as BTC-USD, distinct from provider-native feed IDs.
	pub symbol: String,

	/// Provider slug paired with the canonical security symbol for price discovery.
	pub provider: String,

	/// Inclusive history start in Unix seconds.
	pub from: i64,

	/// Inclusive history end in Unix seconds.
	pub to: i64,
}

/// Outcome-token price history window and sampled-point count.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, Default)]
pub struct TokenHistoryQuery {
	/// On-chain outcome token identifier used by trading and quote subscriptions.
	pub token_id: String,

	/// Supported relative price-history range; server default 1d. None omits this parameter.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub range: Option<String>,

	/// Requested sampled points, from 2 to 1000; server default 400. None omits this parameter.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub points: Option<i64>,
}

/// Bounded search text, result count and supported catalogue source.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, Default)]
pub struct SearchQuery {
	/// Search text from 2 to 128 characters.
	pub q: String,

	/// Maximum hits per result family, from 1 to 25; server default 10. None omits this parameter.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub limit: Option<i64>,

	/// Supported catalogue source; current search accepts AGARA. None omits this parameter.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub source: Option<String>,
}

/// Inclusive YYYY-MM-DD calendar range, limited to 366 days.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, Default)]
pub struct CalendarRangeQuery {
	/// Inclusive range start as a real YYYY-MM-DD date.
	pub from: String,

	/// Inclusive range end as a real YYYY-MM-DD date.
	pub to: String,
}

/// Inclusive YYYY-MM-DD starting date for the next covered trading session.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, Default)]
pub struct NextSessionQuery {
	/// Earliest eligible session date in YYYY-MM-DD format.
	pub from: String,
}

/// Inclusive RFC3339 settled-PnL history boundaries and completed-bucket limit.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, Default)]
pub struct PnlHistoryQuery {
	/// Inclusive settled-PnL range start as RFC3339.
	pub from: String,

	/// Inclusive settled-PnL range end as RFC3339.
	pub to: String,

	/// Maximum completed buckets, from 1 to 2880; server default 672. None omits this parameter.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub limit: Option<i64>,
}

/// Provider metadata that preserves whether the wire supplied text or a number.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum StringOrNumber {
	/// Preserve source metadata encoded as a JSON string.
	Text(String),

	/// Preserve source metadata encoded as a JSON number.
	Number(f64),
}

//! LP opportunity filters, eligibility, and reward contracts.

/// Filters and ordering for current and upcoming LP-incentive opportunities.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]

pub struct LpIncentivesParams {
	/// Active root-category slug. None omits this parameter.
	pub category: Option<String>,

	/// Case-insensitive literal substring matched against market question and event title. None omits
	/// this parameter.
	pub search: Option<String>,

	/// Market field used for explicit sorting. None omits this parameter.
	pub sort_by: Option<LpIncentiveSortBy>,

	/// Sort direction. Defaults to `desc` when `sort_by` is supplied. None omits this parameter.
	pub sort_order: Option<LpIncentiveSortOrder>,
}

/// Current LP-market metric used by explicit sorting.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LpIncentiveSortBy {
	/// Order by market title.
	Market,

	/// Order by the event’s close time.
	CloseTime,

	/// Order by maximum qualifying spread.
	MaxSpread,

	/// Order by minimum qualifying share quantity.
	MinShares,

	/// Order by the current reward pool.
	RewardPool,

	/// Order by the caller’s current incentive share.
	MyShare,

	/// Order by the caller’s projected payout.
	ProjectedPayout,
}

/// Direction applied to the selected LP-incentive sorting field.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LpIncentiveSortOrder {
	/// Lower values or earlier entries first.
	Asc,

	/// Higher values or later entries first.
	Desc,
}

/// Page and filter parameters for the caller’s credited LP reward cycles.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ClosedLpIncentivesParams {
	/// Applied page size or requested upper bound on returned entries. None omits this parameter.
	pub limit: Option<u32>,

	/// 1-based page number. Defaults to 1. None omits this parameter.
	pub page: Option<u32>,

	/// Active root-category slug. None omits this parameter.
	pub category: Option<String>,

	/// Case-insensitive literal substring matched against market question and event title. None omits
	/// this parameter.
	pub search: Option<String>,

	/// Row field used for explicit sorting. Defaults to `date` (the credited reward day). None omits
	/// this parameter.
	pub sort_by: Option<ClosedLpIncentiveSortBy>,

	/// Sort direction. Defaults to `desc` when `sort_by` is supplied. None omits this parameter.
	pub sort_order: Option<LpIncentiveSortOrder>,
}

/// Credited reward-day or earned-amount ordering for historical LP cycles.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClosedLpIncentiveSortBy {
	/// Order by the credited reward date.
	Date,

	/// Order by credited reward amount.
	Earned,
}

/// Current reward epoch and its active/upcoming incentive markets.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]

pub struct LpIncentivesResponse {
	/// Always `true`: every caller sees the market list. Retained for API
	/// compatibility.
	pub eligible: bool,

	/// Reward date as YYYY-MM-DD in America/New_York.
	pub epoch_date: String,

	/// Exclusive RFC3339 end of the current Eastern reward day.
	pub epoch_ends_at: String,

	/// Active and upcoming incentive markets with the caller’s current-epoch standing.
	pub markets: Vec<LpIncentiveMarket>,
}

/// Root categories with at least one matching incentive market.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]

pub struct LpIncentiveCategoriesResponse {
	/// Root categories containing at least one matching incentive market.
	pub categories: Vec<LpIncentiveCategory>,
}

/// Root-category identity with its distinct matching-market count.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]

pub struct LpIncentiveCategory {
	/// Stable catalogue slug used in resource links and lookups.
	pub slug: String,

	/// Human-readable label for this catalogue entry.
	pub label: String,

	/// Distinct current incentive markets assigned under the root.
	pub market_count: i64,
}

/// One page of credited LP cycles with an unpaged matching-row total.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]

pub struct ClosedLpIncentivesResponse {
	/// Always `true`: every authenticated caller sees their reward rows.
	/// Retained for API compatibility.
	pub eligible: bool,

	/// Credited market reward-cycle rows in this result page.
	pub markets: Vec<ClosedLpIncentiveMarket>,

	/// Applied page size or requested upper bound on returned entries.
	pub limit: u32,

	/// 1-based page number this response covers.
	pub page: u32,

	/// Total rows matching the filters across all pages.
	pub total: u64,
}

/// Current projected LP earnings and trailing windows in micro collateral.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]

pub struct LpIncentiveEarningsResponse {
	/// Rewards earned so far in the current reward cycle, projected live from today's
	/// scores before the minimum payout is applied, in micro-collateral units.
	pub current_cycle_micro: crate::units::Micro,

	/// The current cycle's projection plus rewards credited over the 6 prior days.
	pub last_7_days_micro: crate::units::Micro,

	/// The current cycle's projection plus rewards credited over the 29 prior days.
	pub last_30_days_micro: crate::units::Micro,

	/// The current cycle's projection plus all rewards ever credited to the caller.
	pub all_time_micro: crate::units::Micro,
}

/// Whether an incentive opportunity is already active or scheduled to start.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LpIncentiveMarketPhase {
	/// The market currently participates in the active incentive window.
	Active,

	/// The market’s incentive opportunity is scheduled to begin later.
	Upcoming,
}

/// Current incentive terms and the caller’s projected standing for one market.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct LpIncentiveMarket {
	/// Stable Agara market UUID.
	pub market_id: String,

	/// The market’s human-readable resolution question.
	pub question: String,

	/// Parent event’s stable slug for discovery lookups and links.
	pub event_slug: String,

	/// Human-readable title of the parent event.
	pub event_title: String,

	/// Artwork URL; None when no logo is available.
	pub logo_url: Option<String>,

	/// RFC3339 event close time; None when unset.
	pub close_time: Option<String>,

	/// User-facing lifecycle bucket for this market.
	pub phase: LpIncentiveMarketPhase,

	/// Reward pool in micro collateral for this market and reward epoch.
	pub pool_micro: crate::units::Micro,

	/// Maximum qualifying distance from midpoint in micro probability units.
	pub max_spread_micro: u32,

	/// Minimum qualifying resting quantity in micro shares.
	pub min_shares_micro: crate::units::Micro,

	/// The caller's current epoch score: their summed per-sample shares.
	pub epoch_score: f64,

	/// Total epoch score across all makers (= the market's scored-sample count).
	pub market_total_score: f64,

	/// Projected reward in micro collateral before any minimum-payout rule.
	pub projected_payout_micro: crate::units::Micro,

	/// Whether the caller has traded this market at least once.
	pub has_traded: bool,
}

/// One historical market reward cycle with credited and pre-minimum payout amounts.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]

pub struct ClosedLpIncentiveMarket {
	/// Stable Agara market UUID.
	pub market_id: String,

	/// The market’s human-readable resolution question.
	pub question: String,

	/// Parent event’s stable slug for discovery lookups and links.
	pub event_slug: String,

	/// Human-readable title of the parent event.
	pub event_title: String,

	/// Root-category slug; None when no active root category is assigned.
	pub category_slug: Option<String>,

	/// Root-category display label; None when no active root category is assigned.
	pub category_label: Option<String>,

	/// RFC3339 market resolution time; None while unresolved.
	pub closed_at: Option<String>,

	/// Reward pool in micro collateral for this market and reward epoch.
	pub pool_micro: crate::units::Micro,

	/// The caller's score in this cycle.
	pub epoch_score: f64,

	/// Total market score in this cycle.
	pub market_total_score: f64,

	/// Pro-rata reward in micro collateral before any minimum-payout rule.
	pub potential_payout_micro: crate::units::Micro,

	/// Reward credited to the caller for this cycle, in micro collateral.
	pub earned_micro: crate::units::Micro,

	/// Reward date as YYYY-MM-DD in America/New_York.
	pub epoch_date: String,

	/// Whether settlement has credited this cycle.
	pub credited: bool,

	/// Whether the caller has traded this market at least once.
	pub has_traded: bool,
}

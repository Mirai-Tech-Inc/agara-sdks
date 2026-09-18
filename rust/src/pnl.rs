//! Feature-dependent settled and realized PnL contracts.

/// Settled lifetime PnL with the exact mark run and custody-block provenance.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]

pub struct WalletPnlSnapshotDto {
	/// Scheduled settled-PnL bucket time as RFC3339.
	pub bucket_at: String,

	/// UUID of the completed valuation mark run used by this snapshot.
	pub mark_run_id: String,

	/// RFC3339 completion time of the valuation mark capture.
	pub mark_captured_at: String,

	/// Chain identifier whose custody balances were included.
	pub chain_id: i64,

	/// Immutable custody classifier version.
	pub classification_version: i32,

	/// Wallet-universe generation included by the snapshot.
	pub wallet_universe_generation: i64,

	/// Last custody block included.
	pub custody_through_block: i64,

	/// Hash of the last custody block included.
	pub custody_through_block_hash: String,

	/// RFC3339 timestamp of the last included custody block.
	pub custody_through_block_time: String,

	/// Current collateral in micro-collateral units.
	pub collateral_micro: String,

	/// Current or marked position value in micro collateral.
	pub positions_value_micro: String,

	/// Cumulative external inflows in micro-collateral units.
	pub cumulative_inflow_micro: String,

	/// Cumulative external outflows in micro-collateral units.
	pub cumulative_outflow_micro: String,

	/// Full-lifetime settled PnL in micro-collateral units.
	pub lifetime_pnl_micro: String,

	/// Number of positive custody positions included in valuation.
	pub position_count: i32,
}

/// Completed settled-PnL buckets and step-held snapshots at the requested boundaries.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]

pub struct WalletPnlHistoryDto {
	/// Requested inclusive settled-PnL range start as RFC3339.
	pub from: String,

	/// Requested inclusive settled-PnL range end as RFC3339.
	pub to: String,

	/// Step-held snapshot at the range start, absent before the first completed run.
	pub start: Option<WalletPnlSnapshotDto>,

	/// Step-held snapshot at the range end, absent before the first completed run.
	pub end: Option<WalletPnlSnapshotDto>,

	/// End minus start lifetime PnL, present only with both boundaries; this is not attribution.
	pub lifetime_pnl_change_micro: Option<String>,

	/// Completed PnL buckets in the requested range.
	pub points: Vec<WalletPnlSnapshotDto>,

	/// Whether the range contains more completed buckets than returned.
	pub has_more: bool,
}

/// UTC bucket size; weekly buckets are reserved for the all-history window.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RealizedPnlBucketGranularityDto {
	/// One bucket per UTC hour.
	Hour,

	/// One bucket per UTC day.
	Day,

	/// One bucket per Monday-based UTC week.
	Week,
}

/// UTC calendar window for realized PnL, rather than a sliding-duration window.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]

pub enum RealizedPnlWindowDto {
	/// Current UTC day.
	#[serde(rename = "1d")]
	OneDay,

	/// Current UTC day plus the preceding six days.
	#[serde(rename = "7d")]
	SevenDays,

	/// Current UTC day plus the preceding twenty-nine days.
	#[serde(rename = "30d")]
	ThirtyDays,

	/// Complete reporting history.
	#[serde(rename = "all")]
	All,
}

/// Compatible realized-PnL bucket and calendar-window selections.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]

pub struct RealizedPnlReportQueryDto {
	/// Requested UTC bucket resolution.
	pub granularity: RealizedPnlBucketGranularityDto,

	/// Requested UTC calendar window.
	pub window: RealizedPnlWindowDto,
}

/// Exact category totals as decimal USDC strings with the report’s fractional precision.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RealizedPnlCategoryTotalsDto {
	/// Realized sale PnL as exact decimal USDC text, currently with twelve fractional digits.
	pub sales: String,

	/// Realized complete-set merge PnL as exact decimal USDC text.
	pub direct_merges: String,

	/// Realized redemption PnL as exact decimal USDC text.
	pub redemptions: String,

	/// Sum of category PnL as exact decimal USDC text, already expressed in whole USDC.
	pub total: String,
}

/// Realized-PnL category totals for seven-day, thirty-day and all-history windows.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RealizedPnlTotalsDto {
	/// Current UTC day plus the preceding six days.
	pub last_7_days: RealizedPnlCategoryTotalsDto,

	/// Current UTC day plus the preceding twenty-nine days.
	pub last_30_days: RealizedPnlCategoryTotalsDto,

	/// All realised PnL recognized at or after `reportingStartedAt` through `throughEvent`.
	pub all_time: RealizedPnlCategoryTotalsDto,
}

/// One UTC realized-PnL bucket, including zero-filled buckets without disposals.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RealizedPnlBucketDto {
	/// Inclusive RFC 3339 UTC bucket start.
	pub timestamp: String,

	/// Sale PnL for this UTC bucket as exact decimal USDC text.
	pub sales: String,

	/// Complete-set merge PnL for this UTC bucket as exact decimal USDC text.
	pub direct_merges: String,

	/// Redemption PnL for this UTC bucket as exact decimal USDC text.
	pub redemptions: String,

	/// Total PnL for this UTC bucket as exact decimal USDC text.
	pub total: String,
}

/// The durable engine-event frontier included in a realized-PnL report.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RealizedPnlThroughEventDto {
	/// Engine shard whose contiguous durable event prefix was included.
	pub shard_id: i32,

	/// Exact event sequence as a decimal string.
	pub event_seq: String,
}

/// Exact realized-PnL buckets and totals; camelCase on the wire with decimal USDC amounts.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RealizedPnlReportDto {
	/// Collateral currency of this report, currently USDC.
	pub currency: String,

	/// Fractional decimal digits in the USDC strings, currently twelve; amounts are already in USDC.
	pub amount_scale: u8,

	/// Bucket timezone, currently UTC.
	pub timezone: String,

	/// RFC3339 recognition time of the contiguous durable source prefix.
	pub as_of: String,

	/// Earliest RFC3339 instant for which activated reporting coverage is proven.
	pub coverage_started_at: String,

	/// RFC3339 start of recognized disposals included in the report’s totals and buckets.
	pub reporting_started_at: String,

	/// Last contiguous source event included.
	pub through_event: RealizedPnlThroughEventDto,

	/// Recognised wallet operations not yet applied; zero does not prove the projector is current.
	pub pending_operations: u32,

	/// Requested UTC bucket resolution.
	pub granularity: RealizedPnlBucketGranularityDto,

	/// Requested UTC calendar window.
	pub window: RealizedPnlWindowDto,

	/// Whether the current UTC bucket can still receive disposals.
	pub current_bucket_partial: bool,

	/// Seven-day, thirty-day, and all-time totals.
	pub totals: RealizedPnlTotalsDto,

	/// Ascending zero-filled UTC buckets for the requested window.
	pub buckets: Vec<RealizedPnlBucketDto>,
}

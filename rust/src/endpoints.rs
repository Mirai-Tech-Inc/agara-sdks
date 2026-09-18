use crate::{
	batches, bridge, catalogue,
	client::{AgaraClient, Authenticated},
	error::{AgaraError, PollError, Result},
	ids::{BatchGroupId, BatchHash, ConditionId, Exchange, MarketId},
	incentives, input, models, pnl,
	validation::{Field, IdentifierReason, ValidationError},
};

use core::time::Duration;

const HEX: &[u8; 16] = b"0123456789ABCDEF";

pub(crate) fn path_segment(value: &str) -> Result<String> {
	input::text(value, Field::PathSegment, input::MAX_TEXT_BYTES)?;
	if value == "." || value == ".." {
		return Err(ValidationError::identifier(
			Field::PathSegment,
			IdentifierReason::InvalidCharacter,
		)
		.into());
	}

	let mut encoded = String::new();
	for byte in value.bytes() {
		if byte.is_ascii_alphanumeric() || b"-._~".contains(&byte) {
			encoded.push(char::from(byte));
		} else {
			encoded.push('%');
			encoded.push(char::from(HEX[usize::from(byte >> 4)]));
			encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
		}
	}

	Ok(encoded)
}

impl AgaraClient<Authenticated> {
	/// Submit validated, presigned account operations and return the accepted batch digest. Scope:
	/// batches:submit; acceptance is not chain settlement.
	pub async fn submit_batch(
		&self,
		request: &batches::AccountBatchSubmission,
	) -> Result<batches::AccountBatchAccepted> {
		self.post("/trade/v1/batches", request).await
	}

	/// Read one account batch’s current status, failure and reconciliation links. Scope:
	/// batches:submit.
	pub async fn get_batch(&self, batch_hash: &str) -> Result<batches::AccountBatchStatusDto> {
		let batch_hash = BatchHash::new(batch_hash.to_owned())?;
		self.get(&std::format!("/trade/v1/batches/{batch_hash}"), &[]).await
	}

	/// Attempt to replace a pending batch with operations re-signed at the same account sequence.
	/// Inspect the typed superseded/refused outcome; scope: batches:submit.
	pub async fn supersede_batch(
		&self,
		batch_hash: &str,
		request: &batches::AccountBatchSupersedeSubmission,
	) -> Result<batches::AccountBatchSupersedeOutcome> {
		let batch_hash = BatchHash::new(batch_hash.to_owned())?;
		self.post(
			&std::format!("/trade/v1/batches/{batch_hash}/supersede"),
			request,
		)
		.await
	}

	/// Read a multi-batch group and its ordered chunks. Group completion and each chunk’s failure
	/// remain explicit; scope: batches:submit.
	pub async fn get_batch_group(&self, group_id: &str) -> Result<batches::BatchGroupStatusDto> {
		let group_id = BatchGroupId::new(group_id.to_owned())?;
		self.get(&std::format!("/trade/v1/batch-groups/{group_id}"), &[]).await
	}

	/// Read provider deposit addresses associated with the caller’s trading wallet. This request does
	/// not initiate a transfer.
	pub async fn get_deposit_address(
		&self,
	) -> Result<bridge::PortfolioBridgeDepositAddressResponse> {
		self.get("/trade/v1/portfolio/bridge/deposit/address", &[]).await
	}

	/// Read the currently supported deposit chain/token combinations and native token precision.
	pub async fn get_deposit_assets(
		&self,
	) -> Result<bridge::PortfolioBridgeSupportedAssetsResponse> {
		self.get("/trade/v1/portfolio/bridge/deposit/supported-assets", &[]).await
	}

	/// Estimate a deposit route from exact native base-unit inputs without executing the deposit.
	pub async fn get_deposit_quote(
		&self,
		request: &bridge::PortfolioBridgeDepositQuoteRequest,
	) -> Result<bridge::PortfolioBridgeDepositQuoteResponse> {
		self.post("/trade/v1/portfolio/bridge/deposit/quote", request).await
	}

	/// Read supported withdrawal chain/token combinations for the caller’s wallet.
	pub async fn get_withdraw_assets(
		&self,
	) -> Result<bridge::PortfolioBridgeSupportedAssetsResponse> {
		self.get("/trade/v1/portfolio/bridge/withdraw/supported-assets", &[]).await
	}

	/// Estimate a withdrawal route for a chain-specific recipient without submitting a withdrawal.
	pub async fn get_withdraw_quote(
		&self,
		request: &bridge::PortfolioBridgeWithdrawQuoteRequest,
	) -> Result<bridge::PortfolioBridgeDepositQuoteResponse> {
		self.post("/trade/v1/portfolio/bridge/withdraw/quote", request).await
	}

	/// Read one filtered page of the caller’s credited LP reward cycles, retaining total-row count.
	pub async fn get_closed_lp_incentives(
		&self,
		request: &incentives::ClosedLpIncentivesParams,
	) -> Result<incentives::ClosedLpIncentivesResponse> {
		self.get_query("/trade/v1/lp-incentives/closed", request).await
	}

	/// Read categories in which the caller has credited LP rewards.
	pub async fn get_closed_lp_categories(
		&self,
	) -> Result<incentives::LpIncentiveCategoriesResponse> {
		self.get("/trade/v1/lp-incentives/closed/categories", &[]).await
	}

	/// Read current projected and trailing LP earnings in micro collateral.
	pub async fn get_lp_earnings(&self) -> Result<incentives::LpIncentiveEarningsResponse> {
		self.get("/trade/v1/lp-incentives/earnings", &[]).await
	}

	/// Read settled lifetime PnL with mark and custody provenance. This route may be disabled or
	/// report that a completed snapshot is not ready.
	pub async fn get_pnl(&self) -> Result<pnl::WalletPnlSnapshotDto> {
		self.get("/trade/v1/portfolio/pnl", &[]).await
	}

	/// Read completed settled-PnL buckets and step-held RFC3339 range boundaries; this is not activity
	/// attribution.
	pub async fn get_pnl_history(
		&self,
		request: &catalogue::PnlHistoryQuery,
	) -> Result<pnl::WalletPnlHistoryDto> {
		self.get_query("/trade/v1/portfolio/pnl/history", request).await
	}

	/// Read realized disposal PnL as decimal USDC strings, currently with twelve fractional digits.
	/// Keep report precision and source frontier intact.
	pub async fn get_realized_pnl(
		&self,
		request: &pnl::RealizedPnlReportQueryDto,
	) -> Result<pnl::RealizedPnlReportDto> {
		self.get_query("/trade/v1/portfolio/pnl/realized", request).await
	}
}

impl<A> AgaraClient<A> {
	/// Read public root categories with current incentive-market counts.
	pub async fn get_lp_categories(&self) -> Result<incentives::LpIncentiveCategoriesResponse> {
		self.get("/trade/v1/lp-incentives/categories", &[]).await
	}

	/// Read filtered current and upcoming incentive markets. Anonymous callers receive the market list
	/// with zeroed personal standing.
	pub async fn list_lp_incentives(
		&self,
		request: &incentives::LpIncentivesParams,
	) -> Result<incentives::LpIncentivesResponse> {
		self.get_query("/trade/v1/lp-incentives", request).await
	}

	/// Read a covered venue/date session. A covered closed day has no intervals; missing coverage is a
	/// server not-found error.
	pub async fn get_trading_day(&self, mic: &str, date: &str) -> Result<catalogue::TradingDay> {
		let mic = input::mic(mic)?;
		let date = input::calendar(date, Field::Date)?;
		self.get(&std::format!("/api/v1/calendars/{mic}/days/{date}"), &[]).await
	}

	/// Read covered sessions for a validated, inclusive calendar range of at most 366 days.
	pub async fn get_trading_days(
		&self,
		mic: &str,
		request: &catalogue::CalendarRangeQuery,
	) -> Result<catalogue::TradingDaysResponse> {
		let mic = input::mic(mic)?;
		self.get_query(&std::format!("/api/v1/calendars/{mic}/days"), request).await
	}

	/// Read the first covered trading session on or after the requested date; the response may contain
	/// no session.
	pub async fn get_next_session(
		&self,
		mic: &str,
		request: &catalogue::NextSessionQuery,
	) -> Result<catalogue::NextSessionResponse> {
		let mic = input::mic(mic)?;
		self.get_query(
			&std::format!("/api/v1/calendars/{mic}/next-session"),
			request,
		)
		.await
	}

	/// Read registered trading venues and their local-time regular session schedules.
	pub async fn list_calendars(&self) -> Result<catalogue::TradingVenuesResponse> {
		self.get("/api/v1/calendars", &[]).await
	}

	/// Read one normalized venue code, its timezone and regular session schedule.
	pub async fn get_calendar(&self, mic: &str) -> Result<catalogue::TradingVenue> {
		let mic = input::mic(mic)?;
		self.get(&std::format!("/api/v1/calendars/{mic}"), &[]).await
	}

	/// Read category identity and supported event families by its hierarchy slug.
	pub async fn get_category(&self, slug: &str) -> Result<catalogue::CategoryResource> {
		let slug = path_segment(slug)?;
		self.get(&std::format!("/api/v1/categories/{slug}"), &[]).await
	}

	/// Resolve an event alias to its canonical slug and optional redirect URL.
	pub async fn get_canonical_event_url(
		&self,
		slug: &str,
	) -> Result<catalogue::EventCanonicalUrlResponse> {
		let slug = path_segment(slug)?;
		self.get(&std::format!("/api/v1/events/{slug}/canonical-url"), &[]).await
	}

	/// Read full event details, markets and optional lifecycle/display enrichment.
	pub async fn get_event(&self, slug: &str) -> Result<catalogue::EventDetail> {
		let slug = path_segment(slug)?;
		self.get(&std::format!("/api/v1/events/{slug}"), &[]).await
	}

	/// Read one event-discovery page using the complete filter set and optional market enrichment.
	pub async fn list_events(
		&self,
		request: &catalogue::EventsQuery,
	) -> Result<catalogue::EventsListResponse> {
		self.get_query("/api/v1/events", request).await
	}

	/// Read a catalogue market by UUID, including available trading limits and canonical resolution
	/// evidence.
	pub async fn get_market(&self, id: &str) -> Result<catalogue::MarketDetail> {
		let id = MarketId::new(id.to_owned())?;
		self.get(&std::format!("/api/v1/markets/{id}"), &[]).await
	}

	/// Read one filtered AGARA market page with parent-event context and an opaque continuation
	/// cursor.
	pub async fn list_markets(
		&self,
		request: &catalogue::MarketsQuery,
	) -> Result<catalogue::MarketsListResponse> {
		self.get_query("/api/v1/markets", request).await
	}

	/// Read one historical quote by canonical security symbol, provider and Unix second. Preserve
	/// mantissa/exponent for exact decimal interpretation.
	pub async fn get_price_point(
		&self,
		request: &catalogue::PricePointQuery,
	) -> Result<catalogue::PricePoint> {
		self.get_query("/api/v1/prices/point", request).await
	}

	/// Read underlying price history over an inclusive window of at most twenty-four hours, using
	/// Unix-second timestamps.
	pub async fn get_price_ticks(
		&self,
		request: &catalogue::PriceTicksQuery,
	) -> Result<catalogue::PriceHistory> {
		self.get_query("/api/v1/prices/ticks", request).await
	}

	/// Read sampled last-traded outcome probabilities and the latest trade’s Unix-second timestamp.
	pub async fn get_token_history(
		&self,
		request: &catalogue::TokenHistoryQuery,
	) -> Result<catalogue::PriceHistory> {
		self.get_query("/api/v1/prices/token-history", request).await
	}

	/// Search events, markets and categories with bounded text and a per-family result limit.
	pub async fn search(
		&self,
		request: &catalogue::SearchQuery,
	) -> Result<catalogue::SearchResponse> {
		self.get_query("/api/v1/search", request).await
	}

	/// Read canonical underlying instruments and their listing-venue identities.
	pub async fn list_securities(&self) -> Result<catalogue::SecuritiesResponse> {
		self.get("/api/v1/securities", &[]).await
	}

	/// Read a normalized security symbol, provider mappings and regular venue schedule.
	pub async fn get_security(&self, symbol: &str) -> Result<catalogue::SecurityDetail> {
		let symbol = path_segment(symbol)?;
		self.get(&std::format!("/api/v1/securities/{symbol}"), &[]).await
	}
}

impl AgaraClient<Authenticated> {
	/// Read the full position envelope, preserving display metadata and unavailable_exchanges for
	/// completeness checks.
	pub async fn get_positions(
		&self,
		conditions: Vec<ConditionId>,
		exchanges: Vec<Exchange>,
	) -> Result<models::PositionsResponse> {
		self.post(
			"/trade/v1/portfolio/positions/list",
			&models::PositionsListRequest { condition_ids: conditions, exchanges },
		)
		.await
	}

	/// Read one page of nonterminal orders with metadata and an opaque next cursor.
	pub async fn get_open_orders_page(
		&self,
		request: &models::OpenOrdersListRequest,
	) -> Result<models::OpenOrdersResponse> {
		self.post("/trade/v1/portfolio/open-orders/list", request).await
	}

	/// Poll the requested digest until settlement or a completed failure unwind, bounded by an
	/// absolute deadline. Inspect failures and replacement hashes; this method does not follow a
	/// successor automatically.
	pub async fn wait_for_batch(
		&self,
		batch_hash: &str,
		timeout: Duration,
		poll_interval: Duration,
	) -> Result<batches::AccountBatchStatusDto> {
		let deadline = input::deadline(timeout, poll_interval)?;
		loop {
			if tokio::time::Instant::now() >= deadline {
				return Err(PollError::DeadlineElapsed.into());
			}

			let batch = tokio::time::timeout_at(deadline, self.get_batch(batch_hash))
				.await
				.map_err(|_| AgaraError::from(PollError::DeadlineElapsed))??;
			if tokio::time::Instant::now() >= deadline {
				return Err(PollError::DeadlineElapsed.into());
			}

			if batch.is_terminal() {
				return Ok(batch);
			}
			tokio::time::sleep_until(
				tokio::time::Instant::now()
					.checked_add(poll_interval)
					.unwrap_or(deadline)
					.min(deadline),
			)
			.await;
		}
	}
}

use crate::{
	catalogue, incentives,
	validation::{Field, QueryReason, ValidationError},
};

const MAX_MARKETS: i64 = 256;
const MAX_EVENTS: i64 = 100;
const MAX_SEARCH_RESULTS: i64 = 25;
const MAX_SEARCH_CHARS: usize = 128;
const MAX_LP_SEARCH_CHARS: usize = 100;
const MAX_LP_PAGE: i64 = 100;
const MAX_HISTORY_POINTS: i64 = 1000;
const MAX_PNL_BUCKETS: i64 = 2880;
const MAX_CALENDAR_DAYS: i64 = 366;
const MAX_PRICE_SECONDS: i64 = 86_400;
const MARKET_STATES: &[&str] = &[
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
];

fn limit(value: Option<i64>, maximum: i64) -> Result<(), ValidationError> {
	if let Some(value) = value {
		super::number(value, Field::Limit, 1, maximum)?;
	}

	Ok(())
}

fn optional_text(value: Option<&str>, field: Field) -> Result<(), ValidationError> {
	if let Some(value) = value {
		super::text(value, field, super::MAX_TEXT_BYTES)?;
	}

	Ok(())
}

fn symbol_provider(symbol: &str, provider: &str) -> Result<(), ValidationError> {
	super::text(symbol, Field::Symbol, 64)?;
	super::text(provider, Field::Provider, 32)?;
	if !symbol.bytes().all(|c| c.is_ascii_alphanumeric() || b"._/-".contains(&c)) {
		return Err(ValidationError::query(
			Field::Symbol,
			QueryReason::unsupported_value(symbol.to_owned()),
		));
	}

	if !provider.bytes().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-') {
		return Err(ValidationError::query(
			Field::Provider,
			QueryReason::unsupported_value(provider.to_owned()),
		));
	}

	Ok(())
}

fn lp_text(category: Option<&str>, search: Option<&str>) -> Result<(), ValidationError> {
	optional_text(category, Field::Category)?;
	if let Some(search) = search {
		let count = search.trim().chars().count();
		if count > MAX_LP_SEARCH_CHARS {
			return Err(ValidationError::query(
				Field::Search,
				QueryReason::TooLong { max: MAX_LP_SEARCH_CHARS, actual: count },
			));
		}
	}

	Ok(())
}

impl catalogue::MarketsQuery {
	/// Validate the venue, lifecycle filter, page bound and opaque cursor before submission.
	pub fn validate(&self) -> Result<(), ValidationError> {
		super::choice(&self.source.to_ascii_uppercase(), Field::Source, &["AGARA"])?;
		if let Some(state) = &self.state {
			super::choice(&state.to_ascii_uppercase(), Field::State, MARKET_STATES)?;
		}

		optional_text(self.event_slug.as_deref(), Field::EventSlug)?;
		super::cursor(self.cursor.as_deref())?;
		limit(self.limit, MAX_MARKETS)
	}
}

impl catalogue::EventsQuery {
	/// Check the documented filters and 1–100 page size; filter availability remains server-owned.
	pub fn validate(&self) -> Result<(), ValidationError> {
		limit(self.limit, MAX_EVENTS)?;
		super::cursor(self.cursor.as_deref())?;
		optional_text(self.category.as_deref(), Field::Category)?;
		optional_text(self.root.as_deref(), Field::Root)?;
		optional_text(self.filter.as_deref(), Field::Filter)?;
		if let Some(value) = &self.source {
			super::choice(
				&value.to_ascii_uppercase(),
				Field::Source,
				&["AGARA", "POLYMARKET"],
			)?;
		}

		if let Some(value) = &self.event_type_bucket {
			super::choice(value, Field::EventTypeBucket, &["games", "props"])?;
		}

		if let Some(value) = &self.resolution {
			super::choice(
				value,
				Field::Resolution,
				&["all", "active", "proposed", "disputed", "resolved"],
			)?;
		}

		if let Some(value) = &self.sort {
			super::choice(value, Field::Sort, &["time", "volume", "markets"])?;
		}

		Ok(())
	}
}

impl catalogue::PricePointQuery {
	/// Validate canonical security/provider identifiers and a Unix-second timestamp through 2100-01-01.
	pub fn validate(&self) -> Result<(), ValidationError> {
		symbol_provider(&self.symbol, &self.provider)?;
		super::number(self.at, Field::At, 0, super::MAX_UNIX_SECONDS)
	}
}

impl catalogue::PriceTicksQuery {
	/// Validate an ordered, at-most-24-hour Unix-second window and provider identifiers.
	pub fn validate(&self) -> Result<(), ValidationError> {
		symbol_provider(&self.symbol, &self.provider)?;
		super::number(self.from, Field::From, 0, super::MAX_UNIX_SECONDS)?;
		super::number(self.to, Field::To, 0, super::MAX_UNIX_SECONDS)?;
		if self.from > self.to {
			return Err(ValidationError::query(
				Field::From,
				QueryReason::ReversedRange,
			));
		}

		if self.to - self.from > MAX_PRICE_SECONDS {
			return Err(ValidationError::query(
				Field::To,
				QueryReason::RangeTooWide { max: MAX_PRICE_SECONDS as u64 },
			));
		}

		Ok(())
	}
}

impl catalogue::TokenHistoryQuery {
	/// Reject unsupported history windows and out-of-range sample counts instead of clamping them.
	pub fn validate(&self) -> Result<(), ValidationError> {
		crate::ids::TokenId::new(self.token_id.clone())?;
		if let Some(value) = &self.range {
			super::choice(value, Field::Range, &["1h", "6h", "1d", "7d", "30d", "all"])?;
		}

		if let Some(points) = self.points {
			super::number(points, Field::Points, 2, MAX_HISTORY_POINTS)?;
		}

		Ok(())
	}
}

impl catalogue::SearchQuery {
	/// Validate a trimmed query of 2–128 UTF-16 code units, AGARA source and 1–25 result limit.
	pub fn validate(&self) -> Result<(), ValidationError> {
		let count = self.q.trim().encode_utf16().count();
		super::number(count as i64, Field::Query, 2, MAX_SEARCH_CHARS as i64)?;
		limit(self.limit, MAX_SEARCH_RESULTS)?;
		if let Some(source) = &self.source {
			super::choice(&source.to_ascii_uppercase(), Field::Source, &["AGARA"])?;
		}

		Ok(())
	}
}

impl catalogue::CalendarRangeQuery {
	/// Require real ordered dates with an inclusive span of at most 366 days.
	pub fn validate(&self) -> Result<(), ValidationError> {
		let from = super::calendar(&self.from, Field::From)?;
		let to = super::calendar(&self.to, Field::To)?;
		let days = to.days_since(from);
		if days < 0 {
			return Err(ValidationError::query(
				Field::From,
				QueryReason::ReversedRange,
			));
		}

		if days >= MAX_CALENDAR_DAYS {
			return Err(ValidationError::query(
				Field::To,
				QueryReason::RangeTooWide { max: MAX_CALENDAR_DAYS as u64 },
			));
		}

		Ok(())
	}
}

impl catalogue::NextSessionQuery {
	/// Require a real calendar date rather than an unchecked date-shaped string.
	pub fn validate(&self) -> Result<(), ValidationError> {
		super::calendar(&self.from, Field::From).map(|_| ())
	}
}

impl catalogue::PnlHistoryQuery {
	/// Check RFC 3339 instants by actual time, including offsets, and the 1–2880 bucket limit.
	pub fn validate(&self) -> Result<(), ValidationError> {
		let from = super::timestamp(&self.from, Field::From)?;
		let to = super::timestamp(&self.to, Field::To)?;
		if from > to {
			return Err(ValidationError::query(
				Field::From,
				QueryReason::ReversedRange,
			));
		}

		limit(self.limit, MAX_PNL_BUCKETS)
	}
}

impl incentives::LpIncentivesParams {
	/// Validate category/search input and require a sort field when specifying direction.
	pub fn validate(&self) -> Result<(), ValidationError> {
		lp_text(self.category.as_deref(), self.search.as_deref())?;
		if self.sort_order.is_some() && self.sort_by.is_none() {
			return Err(ValidationError::query(
				Field::SortOrder,
				QueryReason::IncompatibleFields { other: Field::SortBy },
			));
		}

		Ok(())
	}
}

impl incentives::ClosedLpIncentivesParams {
	/// Check 1-based pagination, a 1–100 page size, text bounds and coherent sorting.
	pub fn validate(&self) -> Result<(), ValidationError> {
		lp_text(self.category.as_deref(), self.search.as_deref())?;
		limit(self.limit.map(i64::from), MAX_LP_PAGE)?;
		if let Some(page) = self.page {
			super::number(i64::from(page), Field::Page, 1, i64::from(u32::MAX))?;
		}

		if self.sort_order.is_some() && self.sort_by.is_none() {
			return Err(ValidationError::query(
				Field::SortOrder,
				QueryReason::IncompatibleFields { other: Field::SortBy },
			));
		}

		Ok(())
	}
}

//! Validated temporal values used by query and stream contracts.

use core::fmt;

use chrono::{DateTime, FixedOffset, NaiveDate, SecondsFormat};
use serde::{Deserialize, Serialize};

use crate::validation::{Field, QueryReason, ValidationError};

const DATE_LENGTH: usize = 10;

/// A real Gregorian calendar date in canonical `YYYY-MM-DD` form.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct CalendarDate(NaiveDate);

/// A parsed RFC 3339 instant with its explicit UTC offset preserved.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Timestamp(DateTime<FixedOffset>);

impl CalendarDate {
	/// Reject malformed dates and impossible month/day combinations, including leap-day errors.
	pub fn new(value: &str) -> Result<Self, ValidationError> {
		if value.len() != DATE_LENGTH
			|| !value.bytes().enumerate().all(|(i, b)| {
				if i == 4 || i == 7 {
					b == b'-'
				} else {
					b.is_ascii_digit()
				}
			}) {
			return Err(ValidationError::query(
				Field::Date,
				QueryReason::InvalidDate { source: None },
			));
		}

		NaiveDate::parse_from_str(value, "%Y-%m-%d").map(Self).map_err(|source| {
			ValidationError::query(
				Field::Date,
				QueryReason::InvalidDate { source: Some(source) },
			)
		})
	}

	/// Borrow the validated date for calendar arithmetic.
	pub const fn as_date(&self) -> &NaiveDate {
		&self.0
	}

	/// Count calendar days from an earlier date; negative results identify reversed ranges.
	pub fn days_since(self, earlier: Self) -> i64 {
		self.0.signed_duration_since(earlier.0).num_days()
	}
}

impl Timestamp {
	/// Require a valid RFC 3339 instant, including a timezone offset.
	pub fn new(value: &str) -> Result<Self, ValidationError> {
		DateTime::parse_from_rfc3339(value).map(Self).map_err(|source| {
			ValidationError::query(Field::At, QueryReason::InvalidTimestamp { source })
		})
	}

	/// Borrow the parsed timestamp without reparsing its wire text.
	pub const fn as_datetime(&self) -> &DateTime<FixedOffset> {
		&self.0
	}

	/// Return whole seconds since the Unix epoch, independent of the original offset.
	pub const fn unix_seconds(self) -> i64 {
		self.0.timestamp()
	}
}

impl fmt::Display for CalendarDate {
	fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.0.format("%Y-%m-%d").fmt(formatter)
	}
}

impl fmt::Display for Timestamp {
	fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
		formatter.write_str(&self.0.to_rfc3339_opts(SecondsFormat::AutoSi, true))
	}
}

impl TryFrom<String> for CalendarDate {
	type Error = ValidationError;

	fn try_from(value: String) -> Result<Self, Self::Error> {
		Self::new(&value)
	}
}

impl From<CalendarDate> for String {
	fn from(value: CalendarDate) -> Self {
		value.to_string()
	}
}

impl TryFrom<String> for Timestamp {
	type Error = ValidationError;

	fn try_from(value: String) -> Result<Self, Self::Error> {
		Self::new(&value)
	}
}

impl From<Timestamp> for String {
	fn from(value: Timestamp) -> Self {
		value.to_string()
	}
}

mod queries;
mod requests;

use core::time::Duration;

use crate::{
	Micro,
	error::{AgaraError, PollError},
	ids::Exchange,
	validation::{
		AmountReason, ConfigurationError, Field, IdentifierReason, QueryReason, ValidationError,
	},
	values::{CalendarDate, Timestamp},
};

pub(crate) const MAX_LIST_LIMIT: i64 = 500;
pub(crate) const MAX_CURSOR_BYTES: usize = 16_384;
pub(crate) const MAX_TEXT_BYTES: usize = 1024;
pub(crate) const MAX_UNIX_SECONDS: i64 = 4_102_444_800;

pub(crate) trait Validate {
	fn validate_input(&self) -> Result<(), AgaraError>;
}

pub(crate) fn number(value: i64, field: Field, min: i64, max: i64) -> Result<(), ValidationError> {
	if value < min || value > max {
		return Err(ValidationError::query(
			field,
			QueryReason::OutOfRange {
				min: i128::from(min),
				max: i128::from(max),
				actual: i128::from(value),
			},
		));
	}

	Ok(())
}

pub(crate) fn text(value: &str, field: Field, max: usize) -> Result<(), ValidationError> {
	if value.trim().is_empty() {
		return Err(ValidationError::query(field, QueryReason::Required));
	}

	if value.len() > max {
		return Err(ValidationError::query(
			field,
			QueryReason::TooLong { max, actual: value.len() },
		));
	}

	if value.chars().any(char::is_control) {
		return Err(ValidationError::identifier(
			field,
			IdentifierReason::InvalidCharacter,
		));
	}

	Ok(())
}

pub(crate) fn cursor(value: Option<&str>) -> Result<(), ValidationError> {
	if let Some(value) = value {
		text(value, Field::Cursor, MAX_CURSOR_BYTES)?;
	}

	Ok(())
}

pub(crate) fn exchanges(values: &[Exchange]) -> Result<(), ValidationError> {
	if values.contains(&Exchange::Unknown) {
		return Err(ValidationError::query(
			Field::Exchange,
			QueryReason::unsupported_value("UNKNOWN"),
		));
	}

	Ok(())
}

pub(crate) fn positive(value: Micro, field: Field) -> Result<(), ValidationError> {
	if value.raw() <= 0 {
		return Err(ValidationError::amount(field, AmountReason::NonPositive));
	}

	Ok(())
}

pub(crate) fn choice(value: &str, field: Field, choices: &[&str]) -> Result<(), ValidationError> {
	if !choices.contains(&value) {
		return Err(ValidationError::query(
			field,
			QueryReason::unsupported_value(value.to_owned()),
		));
	}

	Ok(())
}

pub(crate) fn calendar(value: &str, field: Field) -> Result<CalendarDate, ValidationError> {
	CalendarDate::new(value).map_err(|error| error.with_field(field))
}

pub(crate) fn timestamp(value: &str, field: Field) -> Result<Timestamp, ValidationError> {
	Timestamp::new(value).map_err(|error| error.with_field(field))
}

pub(crate) fn mic(value: &str) -> Result<String, ValidationError> {
	let value = value.trim().to_ascii_uppercase();
	let mut chars = value.bytes();
	if !(2..=8).contains(&value.len())
		|| !chars.next().is_some_and(|c| c.is_ascii_uppercase())
		|| !chars.all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
	{
		return Err(ValidationError::identifier(
			Field::Mic,
			IdentifierReason::InvalidCharacter,
		));
	}

	Ok(value)
}

pub(crate) fn base_url(value: &str) -> Result<url::Url, ValidationError> {
	let url = url::Url::parse(value).map_err(ConfigurationError::InvalidBaseUrl)?;
	if !core::matches!(url.scheme(), "http" | "https") {
		return Err(ConfigurationError::UnsupportedScheme.into());
	}

	if url.host_str().is_none() {
		return Err(ConfigurationError::MissingHost.into());
	}

	if !url.username().is_empty() || url.password().is_some() {
		return Err(ConfigurationError::UrlCredentials.into());
	}

	if url.query().is_some() {
		return Err(ConfigurationError::UrlQuery.into());
	}

	if url.fragment().is_some() {
		return Err(ConfigurationError::UrlFragment.into());
	}

	Ok(url)
}

pub(crate) fn duration(value: Duration, field: Field) -> Result<(), ValidationError> {
	if value.is_zero() || tokio::time::Instant::now().checked_add(value).is_none() {
		return Err(ConfigurationError::InvalidTimeout { field }.into());
	}

	Ok(())
}

pub(crate) fn deadline(
	timeout: Duration,
	interval: Duration,
) -> Result<tokio::time::Instant, AgaraError> {
	duration(timeout, Field::PollTimeout)?;
	duration(interval, Field::PollInterval)?;

	tokio::time::Instant::now()
		.checked_add(timeout)
		.ok_or_else(|| PollError::InvalidDeadline.into())
}

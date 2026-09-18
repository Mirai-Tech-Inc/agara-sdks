//! Micro-unit encoding. The wire carries USDC and share amounts as
//! integer micro-units (1 unit = 1e-6) encoded as JSON strings; the API
//! also accepts a bare JSON number on input. Prices and sizes are
//! surfaced to callers as [`rust_decimal::Decimal`] dollars / shares.

use core::fmt;

use crate::validation::{AmountReason, Field, ValidationError};

use rust_decimal::{Decimal, prelude::ToPrimitive};

use serde::{
	Deserialize, Serialize, Serializer,
	de::{self, Deserializer, Visitor},
};

/// Micro-units per whole unit (dollar or share).
pub const MICRO: i64 = 1_000_000;

/// An integer micro-unit amount. Lossless on the wire; convert to
/// dollars / shares with [`Micro::as_decimal`].
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Micro(i64);

struct MicroVisitor;

impl Micro {
	/// Wrap a raw micro-unit integer.
	pub const fn new(raw: i64) -> Self {
		Self(raw)
	}

	/// The raw micro-unit integer.
	pub const fn raw(self) -> i64 {
		self.0
	}

	/// Convert whole units exactly, rejecting fractional micro-units and overflow.
	pub fn from_units(units: Decimal) -> crate::Result<Self> {
		let scaled = units
			.checked_mul(Decimal::from(MICRO))
			.ok_or_else(|| ValidationError::amount(Field::Amount, AmountReason::Overflow))?;
		if !scaled.fract().is_zero() {
			return Err(ValidationError::amount(Field::Amount, AmountReason::Precision).into());
		}
		scaled
			.to_i64()
			.map(Self)
			.ok_or_else(|| ValidationError::amount(Field::Amount, AmountReason::OutOfRange).into())
	}

	/// Add amounts without overflow.
	pub fn checked_add(self, other: Self) -> Option<Self> {
		self.0.checked_add(other.0).map(Self)
	}

	/// Subtract amounts without overflow.
	pub fn checked_sub(self, other: Self) -> Option<Self> {
		self.0.checked_sub(other.0).map(Self)
	}

	/// Convert micro-units back to whole units (dollars / shares).
	pub fn as_decimal(self) -> Decimal {
		Decimal::from(self.0) / Decimal::from(MICRO)
	}

	/// Whole units as an `f64` (lossy — prefer [`Micro::as_decimal`] for
	/// exactness).
	pub fn as_f64(self) -> f64 {
		self.0 as f64 / MICRO as f64
	}
}

impl fmt::Debug for Micro {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		core::write!(f, "Micro({})", self.0)
	}
}

impl fmt::Display for Micro {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		core::write!(f, "{}", self.0)
	}
}

impl Serialize for Micro {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		serializer.serialize_str(&self.0.to_string())
	}
}

impl<'de> Deserialize<'de> for Micro {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		deserializer.deserialize_any(MicroVisitor)
	}
}

impl Visitor<'_> for MicroVisitor {
	type Value = Micro;

	fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str("a micro-unit integer as a string or number")
	}

	fn visit_str<E: de::Error>(self, v: &str) -> Result<Micro, E> {
		v.parse::<i64>().map(Micro).map_err(de::Error::custom)
	}

	fn visit_i64<E: de::Error>(self, v: i64) -> Result<Micro, E> {
		Ok(Micro(v))
	}

	fn visit_u64<E: de::Error>(self, v: u64) -> Result<Micro, E> {
		i64::try_from(v).map(Micro).map_err(de::Error::custom)
	}

	fn visit_f64<E: de::Error>(self, v: f64) -> Result<Micro, E> {
		Err(de::Error::custom(std::format!(
			"floating-point micro amount {v} is not accepted; use an integer string"
		)))
	}
}

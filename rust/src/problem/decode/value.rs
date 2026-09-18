use core::fmt;

use serde::{
	Deserialize, Deserializer,
	de::{MapAccess, SeqAccess, Visitor},
};
use serde_json::{Map, Number, Value};

pub(super) struct StrictValue(pub(super) Value);

struct ValueVisitor;

impl<'de> Deserialize<'de> for StrictValue {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		deserializer.deserialize_any(ValueVisitor)
	}
}

impl<'de> Visitor<'de> for ValueVisitor {
	type Value = StrictValue;

	fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
		formatter.write_str("JSON without duplicate object properties")
	}

	fn visit_bool<E: serde::de::Error>(self, value: bool) -> Result<Self::Value, E> {
		Ok(StrictValue(Value::Bool(value)))
	}

	fn visit_i64<E: serde::de::Error>(self, value: i64) -> Result<Self::Value, E> {
		Ok(StrictValue(Value::Number(Number::from(value))))
	}

	fn visit_u64<E: serde::de::Error>(self, value: u64) -> Result<Self::Value, E> {
		Ok(StrictValue(Value::Number(Number::from(value))))
	}

	fn visit_f64<E: serde::de::Error>(self, value: f64) -> Result<Self::Value, E> {
		Number::from_f64(value)
			.map(|number| StrictValue(Value::Number(number)))
			.ok_or_else(|| serde::de::Error::custom("non-finite JSON number"))
	}

	fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
		Ok(StrictValue(Value::String(value.to_owned())))
	}

	fn visit_string<E: serde::de::Error>(self, value: String) -> Result<Self::Value, E> {
		Ok(StrictValue(Value::String(value)))
	}

	fn visit_none<E: serde::de::Error>(self) -> Result<Self::Value, E> {
		Ok(StrictValue(Value::Null))
	}

	fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
		Ok(StrictValue(Value::Null))
	}

	fn visit_seq<A: SeqAccess<'de>>(self, mut sequence: A) -> Result<Self::Value, A::Error> {
		let mut values = Vec::new();

		while let Some(StrictValue(value)) = sequence.next_element::<StrictValue>()? {
			values.push(value);
		}

		Ok(StrictValue(Value::Array(values)))
	}

	fn visit_map<A: MapAccess<'de>>(self, mut access: A) -> Result<Self::Value, A::Error> {
		let mut values = Map::new();

		while let Some((key, StrictValue(value))) = access.next_entry::<String, StrictValue>()? {
			if values.contains_key(&key) {
				return Err(serde::de::Error::custom(std::format!(
					"duplicate JSON property {key}"
				)));
			}

			values.insert(key, value);
		}

		Ok(StrictValue(Value::Object(values)))
	}
}

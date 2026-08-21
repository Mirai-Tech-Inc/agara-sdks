//! Public failure contracts shared by HTTP, asynchronous order state, and
//! WebSocket frames.

use core::time::Duration;

use serde::{Deserialize, Deserializer};
use serde_json::Value;

/// Sanitized recovery guidance. Unknown or malformed strategies are retained
/// but inert, so a future wire addition cannot make this SDK repeat an action.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum Recovery {
	None,
	Retry,
	RetryAfter { after_seconds: u32 },
	RefreshIdentityToken,
	CheckStatus { resource: Value },
	Unknown { strategy: String },
}

impl Recovery {
	/// Whether the server explicitly permits repeating the same request.
	pub fn is_retryable(&self) -> bool {
		matches!(self, Self::Retry | Self::RetryAfter { .. })
	}

	/// A body-level retry delay, when present.
	pub fn retry_after(&self) -> Option<Duration> {
		match self {
			Self::RetryAfter { after_seconds } => {
				Some(Duration::from_secs(u64::from(*after_seconds)))
			},
			_ => None,
		}
	}

	/// The wire strategy name, including future values.
	pub fn strategy(&self) -> &str {
		match self {
			Self::None => "none",
			Self::Retry => "retry",
			Self::RetryAfter { .. } => "retry_after",
			Self::RefreshIdentityToken => "refresh_identity_token",
			Self::CheckStatus { .. } => "check_status",
			Self::Unknown { strategy } => strategy,
		}
	}
}

impl<'de> Deserialize<'de> for Recovery {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		let value = Value::deserialize(deserializer)?;
		let Some(object) = value.as_object() else {
			return Ok(Self::Unknown { strategy: "unknown".to_owned() });
		};
		let strategy =
			object.get("strategy").and_then(Value::as_str).unwrap_or("unknown").to_owned();
		let exact = |fields: &[&str]| {
			object.len() == fields.len() && fields.iter().all(|field| object.contains_key(*field))
		};

		let recovery = match strategy.as_str() {
			"none" if exact(&["strategy"]) => Self::None,
			"retry" if exact(&["strategy"]) => Self::Retry,
			"refresh_identity_token" if exact(&["strategy"]) => Self::RefreshIdentityToken,
			"retry_after" if exact(&["strategy", "after_seconds"]) => object
				.get("after_seconds")
				.and_then(Value::as_u64)
				.filter(|seconds| *seconds <= 86_400)
				.and_then(|seconds| u32::try_from(seconds).ok())
				.map_or_else(
					|| Self::Unknown { strategy: strategy.clone() },
					|after_seconds| Self::RetryAfter { after_seconds },
				),
			"check_status" if exact(&["strategy", "resource"]) => {
				match object.get("resource").filter(|resource| is_recovery_resource(resource)) {
					Some(resource) => Self::CheckStatus { resource: resource.clone() },
					None => Self::Unknown { strategy: strategy.clone() },
				}
			},
			_ => Self::Unknown { strategy: strategy.clone() },
		};

		Ok(recovery)
	}
}

fn is_recovery_uuid(value: &str) -> bool {
	let bytes = value.as_bytes();
	if bytes.len() != 36
		|| !bytes.iter().enumerate().all(|(index, byte)| {
			if matches!(index, 8 | 13 | 18 | 23) {
				*byte == b'-'
			} else {
				byte.is_ascii_hexdigit()
			}
		}) {
		return false;
	}

	let compact = value.replace('-', "");
	if compact.bytes().all(|byte| byte == b'0')
		|| compact.bytes().all(|byte| matches!(byte, b'f' | b'F'))
	{
		return true;
	}

	matches!(compact.as_bytes()[12], b'1'..=b'8')
		&& matches!(
			compact.as_bytes()[16],
			b'8' | b'9' | b'a' | b'A' | b'b' | b'B'
		)
}

fn is_recovery_resource(value: &Value) -> bool {
	let Some(object) = value.as_object() else {
		return false;
	};
	match object.get("kind").and_then(Value::as_str) {
		Some("order") if object.len() == 2 => {
			object.get("order_id").and_then(Value::as_str).is_some_and(is_recovery_uuid)
		},
		Some("batch") if object.len() == 2 => {
			object.get("batch_hash").and_then(Value::as_str).is_some_and(|hash| {
				hash.len() == 66
					&& hash.starts_with("0x")
					&& hash.as_bytes()[2..].iter().all(u8::is_ascii_hexdigit)
			})
		},
		Some("group") if object.len() == 2 => {
			object.get("group_id").and_then(Value::as_str).is_some_and(is_recovery_uuid)
		},
		Some("wallet_status") => object.len() == 1,
		_ => false,
	}
}

/// Protocol-neutral public failure embedded in asynchronous responses.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct PublicFailure {
	pub code: String,
	pub title: String,
	#[serde(default)]
	pub detail: Option<String>,
	pub recovery: Recovery,
}

impl PublicFailure {
	/// Safe fallback for legacy responses that contained only a private reason.
	pub fn internal() -> Self {
		Self {
			code: "internal_error".to_owned(),
			title: "Internal server error".to_owned(),
			detail: None,
			recovery: Recovery::None,
		}
	}

	pub(crate) fn from_legacy_batch_code(code: Option<&str>) -> Self {
		let (code, title) = match code.unwrap_or_default().to_ascii_uppercase().as_str() {
			"DUPLICATE" | "DUPLICATE_ORDER" => ("duplicate_order", "Duplicate order"),
			"INVALID_SIGNATURE" => ("invalid_signature", "Invalid signature"),
			"INSUFFICIENT_BALANCE" => ("insufficient_balance", "Insufficient balance"),
			"INSUFFICIENT_SHARES" => ("insufficient_shares", "Insufficient shares"),
			"MARKET_CLOSED" => ("market_closed", "Market closed"),
			"ORDER_NOT_FILLABLE" => ("order_not_fillable", "Order cannot be filled"),
			"POST_ONLY_WOULD_CROSS" => ("post_only_would_cross", "Post-only order would cross"),
			_ => return Self::internal(),
		};

		Self {
			code: code.to_owned(),
			title: title.to_owned(),
			detail: None,
			recovery: Recovery::None,
		}
	}
}

/// Origin HTTP Problem Details.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct ProblemDetails {
	#[serde(rename = "type")]
	pub type_uri: String,
	pub title: String,
	pub status: u16,
	pub code: String,
	#[serde(default)]
	pub detail: Option<String>,
	#[serde(default)]
	pub request_id: Option<String>,
	pub recovery: Recovery,
	#[serde(default)]
	pub field_errors: Vec<Value>,
}

impl ProblemDetails {
	/// Safe user-facing summary, preferring allowlisted detail when present.
	pub fn message(&self) -> &str {
		self.detail.as_deref().unwrap_or(&self.title)
	}

	/// Whether this SDK version recognizes the code/recovery combination as a
	/// safe retry. Future codes remain inert even if they reuse a known strategy.
	pub fn is_retryable(&self) -> bool {
		matches!(
			self.code.as_str(),
			"dependency_unavailable"
				| "pnl_not_ready"
				| "price_stream_capacity_exceeded"
				| "price_temporarily_unavailable"
				| "rate_limited"
		) && self.recovery.is_retryable()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn unknown_and_malformed_recovery_are_inert() {
		for json in [
			r#"{"strategy":"future_strategy","after_seconds":1}"#,
			r#"{"strategy":"retry","unexpected":true}"#,
			r#"{"strategy":"retry_after","after_seconds":86401}"#,
		] {
			let recovery: Recovery = serde_json::from_str(json).unwrap();
			assert!(!recovery.is_retryable());
			assert!(matches!(recovery, Recovery::Unknown { .. }));
		}
	}

	#[test]
	fn retry_recovery_is_explicit() {
		let retry: Recovery = serde_json::from_str(r#"{"strategy":"retry"}"#).unwrap();
		let none: Recovery = serde_json::from_str(r#"{"strategy":"none"}"#).unwrap();
		assert!(retry.is_retryable());
		assert!(!none.is_retryable());
	}

	#[test]
	fn malformed_check_status_resource_is_inert() {
		for json in [
			r#"{"strategy":"check_status","resource":{"kind":"order","order_id":"bad"}}"#,
			r#"{"strategy":"check_status","resource":{"kind":"batch","batch_hash":"0x12"}}"#,
			r#"{"strategy":"check_status","resource":{"kind":"wallet_status","extra":true}}"#,
			r#"{"strategy":"check_status","resource":{"kind":"future","id":"abc"}}"#,
		] {
			let recovery: Recovery = serde_json::from_str(json).unwrap();
			assert!(matches!(recovery, Recovery::Unknown { .. }));
		}
	}
}

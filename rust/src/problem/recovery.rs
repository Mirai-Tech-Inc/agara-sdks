use core::time::Duration;

use std::borrow::Cow;

use serde::{Deserialize, Deserializer};
use serde_json::Value;

use super::{ProblemDecodeError, ProblemField, decode};

const MAX_RETRY_AFTER_SECONDS: u64 = 86_400;
const MAX_RECOVERY_PROPERTIES: usize = 8;
const HASH_TEXT_LENGTH: usize = 66;
const MAX_STRATEGY_LENGTH: usize = 64;
const UUID_TEXT_LENGTH: usize = 36;

/// Registered recovery templates; code metadata selects exactly one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecoveryStrategy {
	/// Repeating the action is not permitted.
	None,

	/// The request may be retried under explicit client policy.
	Retry,

	/// Retry only after the provided bounded delay.
	RetryAfter,

	/// Refresh the identity credential before taking another action.
	RefreshIdentityToken,

	/// Reconcile an existing resource instead of replaying a mutation.
	CheckStatus,
}

/// Durable resource kinds accepted by check-status recovery.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceKind {
	/// An existing order owned by the caller.
	Order,

	/// An accepted account batch identified by its digest.
	Batch,

	/// An account-batch group owned by the caller.
	Group,

	/// Browser-authenticated wallet onboarding state.
	WalletStatus,
}

/// Canonical thirty-two-byte recovery digest, preserving its wire spelling.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecoveryBatchHash(Cow<'static, str>);

/// Validated resource identity that a caller must reconcile.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RecoveryResource {
	/// Query an existing order's current state.
	Order {
		/// Canonical UUID parsed from the recovery reference.
		order_id: uuid::Uuid,
	},

	/// Query an already accepted account batch.
	Batch {
		/// Validated EIP-712 batch digest.
		batch_hash: RecoveryBatchHash,
	},

	/// Query an existing multi-batch group.
	Group {
		/// Canonical UUID parsed from the group reference.
		group_id: uuid::Uuid,
	},

	/// Read wallet onboarding state using the appropriate browser authentication.
	WalletStatus,
}

/// Strict known recovery shape or a forward-compatible inert value.
#[derive(Clone, Debug, PartialEq)]
pub enum Recovery {
	/// No automatic recovery is permitted.
	None,

	/// An explicit retry policy may repeat the request.
	Retry,

	/// The server requires a bounded delay before a permitted retry.
	RetryAfter {
		/// Delta-seconds, validated within the platform's one-day maximum.
		after_seconds: u32,
	},

	/// Obtain a fresh identity token before reconnecting or retrying.
	RefreshIdentityToken,

	/// Inspect an existing resource; do not replay the original mutation blindly.
	CheckStatus {
		/// Validated identity of the resource to reconcile.
		resource: RecoveryResource,
	},

	/// Future strategy or known-looking advice attached to an unknown code; always inert.
	Unknown {
		/// Bounded original strategy name, retained without enabling behavior.
		strategy: Cow<'static, str>,

		/// Original recovery object for diagnostics and future protocol consumers.
		raw: Value,
	},
}

impl RecoveryBatchHash {
	/// Validate an exact 0x-prefixed thirty-two-byte hexadecimal digest.
	pub fn new(value: Cow<'static, str>) -> Result<Self, ProblemDecodeError> {
		if value.len() != HASH_TEXT_LENGTH
			|| !value.starts_with("0x")
			|| !value.as_bytes()[2..].iter().all(u8::is_ascii_hexdigit)
		{
			return Err(ProblemDecodeError::InvalidBatchHash);
		}

		Ok(Self(value))
	}

	/// Borrow the validated digest in its original wire form.
	pub fn as_str(&self) -> &str {
		&self.0
	}
}

impl RecoveryResource {
	/// Parse one of the closed current resource shapes.
	pub fn parse(value: Value) -> Result<Self, ProblemDecodeError> {
		let object = decode::object(&value, ProblemField::RecoveryResource)?;
		let kind = decode::required_string(
			object,
			"kind",
			ProblemField::ResourceKind,
			MAX_STRATEGY_LENGTH,
		)?;

		match kind {
			"order" => {
				decode::exact_fields(object, &["kind", "order_id"])?;
				let raw = decode::required_string(
					object,
					"order_id",
					ProblemField::OrderId,
					UUID_TEXT_LENGTH,
				)?;

				Ok(Self::Order { order_id: decode::uuid(raw, ProblemField::OrderId)? })
			},
			"batch" => {
				decode::exact_fields(object, &["kind", "batch_hash"])?;
				let raw = decode::required_string(
					object,
					"batch_hash",
					ProblemField::BatchHash,
					HASH_TEXT_LENGTH,
				)?;

				Ok(Self::Batch { batch_hash: RecoveryBatchHash::new(Cow::Owned(raw.to_owned()))? })
			},
			"group" => {
				decode::exact_fields(object, &["kind", "group_id"])?;
				let raw = decode::required_string(
					object,
					"group_id",
					ProblemField::GroupId,
					UUID_TEXT_LENGTH,
				)?;

				Ok(Self::Group { group_id: decode::uuid(raw, ProblemField::GroupId)? })
			},
			"wallet_status" => {
				decode::exact_fields(object, &["kind"])?;

				Ok(Self::WalletStatus)
			},
			_ => Err(ProblemDecodeError::UnknownResourceKind),
		}
	}

	/// The resource discriminator selected by registered recovery metadata.
	pub const fn kind(&self) -> ResourceKind {
		match self {
			Self::Order { .. } => ResourceKind::Order,
			Self::Batch { .. } => ResourceKind::Batch,
			Self::Group { .. } => ResourceKind::Group,
			Self::WalletStatus => ResourceKind::WalletStatus,
		}
	}

	pub(super) fn validate(&self) -> Result<(), ProblemDecodeError> {
		match self {
			Self::Order { order_id } => {
				decode::uuid(&order_id.to_string(), ProblemField::OrderId).map(|_| ())
			},
			Self::Group { group_id } => {
				decode::uuid(&group_id.to_string(), ProblemField::GroupId).map(|_| ())
			},
			Self::Batch { .. } | Self::WalletStatus => Ok(()),
		}
	}
}

impl Recovery {
	/// Parse strict known templates and bounded future strategy objects.
	pub fn parse(value: Value) -> Result<Self, ProblemDecodeError> {
		let object = decode::object(&value, ProblemField::Recovery)?;
		let strategy = decode::required_string(
			object,
			"strategy",
			ProblemField::RecoveryStrategy,
			MAX_STRATEGY_LENGTH,
		)?;
		decode::code_name(strategy, ProblemField::RecoveryStrategy)?;

		match strategy {
			"none" | "retry" | "refresh_identity_token" => {
				decode::exact_fields(object, &["strategy"])?;

				Ok(match strategy {
					"none" => Self::None,
					"retry" => Self::Retry,
					_ => Self::RefreshIdentityToken,
				})
			},
			"retry_after" => {
				decode::exact_fields(object, &["strategy", "after_seconds"])?;
				let seconds =
					decode::required_unsigned(object, "after_seconds", ProblemField::AfterSeconds)?;
				decode::number_range(
					seconds,
					0,
					MAX_RETRY_AFTER_SECONDS,
					ProblemField::AfterSeconds,
				)?;

				Ok(Self::RetryAfter { after_seconds: seconds as u32 })
			},
			"check_status" => {
				decode::exact_fields(object, &["strategy", "resource"])?;
				let resource =
					decode::required(object, "resource", ProblemField::RecoveryResource)?;

				Ok(Self::CheckStatus { resource: RecoveryResource::parse(resource.clone())? })
			},
			_ => {
				decode::length_bound(
					object.len(),
					MAX_RECOVERY_PROPERTIES,
					ProblemField::Recovery,
				)?;
				let strategy = Cow::Owned(strategy.to_owned());

				Ok(Self::Unknown { strategy, raw: value })
			},
		}
	}

	/// Whether this recognized, valid recovery template permits a retry.
	pub fn is_retryable(&self) -> bool {
		self.validate().is_ok() && core::matches!(self, Self::Retry | Self::RetryAfter { .. })
	}

	/// A validated server-specified retry delay.
	pub fn retry_after(&self) -> Option<Duration> {
		match self {
			Self::RetryAfter { after_seconds }
				if u64::from(*after_seconds) <= MAX_RETRY_AFTER_SECONDS =>
			{
				Some(Duration::from_secs(u64::from(*after_seconds)))
			},
			_ => None,
		}
	}

	/// Original strategy name for display or serialization; unknown values do not authorize actions.
	pub fn strategy(&self) -> &str {
		match self {
			Self::None => "none",
			Self::Retry => "retry",
			Self::RetryAfter { .. } => "retry_after",
			Self::RefreshIdentityToken => "refresh_identity_token",
			Self::CheckStatus { .. } => "check_status",
			Self::Unknown { strategy, .. } => strategy,
		}
	}

	/// Registered strategy identity, absent for all inert future advice.
	pub const fn known_strategy(&self) -> Option<RecoveryStrategy> {
		match self {
			Self::None => Some(RecoveryStrategy::None),
			Self::Retry => Some(RecoveryStrategy::Retry),
			Self::RetryAfter { .. } => Some(RecoveryStrategy::RetryAfter),
			Self::RefreshIdentityToken => Some(RecoveryStrategy::RefreshIdentityToken),
			Self::CheckStatus { .. } => Some(RecoveryStrategy::CheckStatus),
			Self::Unknown { .. } => None,
		}
	}

	/// Check-status discriminator, absent for all other recovery templates.
	pub const fn resource_kind(&self) -> Option<ResourceKind> {
		match self {
			Self::CheckStatus { resource } => Some(resource.kind()),
			_ => None,
		}
	}

	pub(super) fn into_inert(self, raw: Value) -> Self {
		Self::Unknown { strategy: Cow::Owned(self.strategy().to_owned()), raw }
	}

	pub(super) fn validate(&self) -> Result<(), ProblemDecodeError> {
		match self {
			Self::RetryAfter { after_seconds } => decode::number_range(
				u64::from(*after_seconds),
				0,
				MAX_RETRY_AFTER_SECONDS,
				ProblemField::AfterSeconds,
			),
			Self::CheckStatus { resource } => resource.validate(),
			Self::Unknown { strategy, raw } => {
				decode::code_name(strategy, ProblemField::RecoveryStrategy)?;
				let object = decode::object(raw, ProblemField::Recovery)?;
				decode::length_bound(
					object.len(),
					MAX_RECOVERY_PROPERTIES,
					ProblemField::Recovery,
				)
			},
			_ => Ok(()),
		}
	}
}

impl<'de> Deserialize<'de> for Recovery {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		Self::parse(decode::deserialize_value(deserializer)?).map_err(serde::de::Error::custom)
	}
}

impl<'de> Deserialize<'de> for RecoveryResource {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		Self::parse(decode::deserialize_value(deserializer)?).map_err(serde::de::Error::custom)
	}
}

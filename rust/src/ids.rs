//! Validated resource identifiers and forward-compatible response classifications.

mod define;

use core::fmt;

use serde::{Deserialize, Serialize};

use crate::validation::{Field, IdentifierReason, ValidationError};

const MAX_TOKEN_DIGITS: usize = 78;
const U256_MAX_DECIMAL: &str =
	"115792089237316195423570985008687907853269984665640564039457584007913129639935";
const DIGEST_HEX_LENGTH: usize = 66;
#[cfg(feature = "signing")]
const HEX_DIGITS: &[u8; 16] = b"0123456789abcdef";

define::string_id!(
	/// An unsigned 256-bit outcome token identifier, serialized as canonical decimal text.
	TokenId, Field::TokenId, token_text
);
define::string_id!(
	/// A CTF condition identifier containing exactly 32 bytes, serialized as lowercase 0x hex.
	ConditionId, Field::ConditionId, digest_text
);
define::string_id!(
	/// A canonical UUID identifying a platform order, including when the venue is Polymarket.
	OrderId, Field::OrderId, uuid_text
);
define::string_id!(
	/// A 32-byte EIP-712 order digest, serialized as lowercase 0x hex.
	OrderHash, Field::OrderHash, digest_text
);
define::string_id!(
	/// A canonical UUID identifying a registered wallet.
	WalletId, Field::WalletId, uuid_text
);
define::string_id!(
	/// A catalogue event UUID used for event-scoped lifecycle subscriptions.
	EventId, Field::EventId, uuid_text
);
define::string_id!(
	/// A catalogue market UUID, distinct from its on-chain condition identifier.
	MarketId, Field::MarketId, uuid_text
);
define::string_id!(
	/// A 32-byte account-batch digest used for status and supersession requests.
	BatchHash, Field::BatchHash, digest_text
);
define::string_id!(
	/// A canonical UUID identifying an account-batch group.
	BatchGroupId, Field::BatchGroupId, uuid_text
);

/// The backend an order / position / balance lives on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Exchange {
	/// AGARA’s native matching engine and collateral wallet.
	Agara,

	/// Polymarket’s order venue and its separate collateral wallet.
	Polymarket,

	/// A backend this client version doesn't model — keeps a new venue in a
	/// server response from failing the whole decode.
	#[serde(other)]
	Unknown,
}

/// Order side.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Side {
	/// Acquire outcome shares in exchange for collateral.
	Buy,

	/// Sell held outcome shares for collateral.
	Sell,

	/// Unrecognized response value; request validators reject this variant.
	#[default]
	#[serde(other)]
	Unspecified,
}

/// Limit vs market order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderType {
	/// Execute at a specified limit price with a share-sized quantity.
	Limit,

	/// Execute against available depth using a buy budget or sell share quantity.
	Market,

	/// An order type this client version doesn't model.
	#[serde(other)]
	Unknown,
}

/// How long an order rests.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TimeInForce {
	/// Rest until filled or explicitly cancelled.
	Gtc,

	/// Rest until filled, cancelled or the required expiration timestamp.
	Gtd,

	/// Fill available quantity immediately and cancel the remainder.
	Fak,

	/// Execute the complete requested quantity immediately or reject the order.
	Fok,

	/// Unrecognized response value; request validators reject this variant.
	#[default]
	#[serde(other)]
	Unspecified,
}

/// Lifecycle status of an order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderStatus {
	/// Accepted locally and awaiting asynchronous submission.
	Pending,

	/// Submission is in progress and the venue has not yet acknowledged it.
	Submitting,

	/// The venue has delayed matching a marketable order.
	Delayed,

	/// The order awaits just-in-time bridge funding.
	AwaitingBridge,

	/// The order is resting on the venue’s book.
	Open,

	/// Some quantity filled; only the order’s is_terminal flag determines whether any remains live.
	PartiallyFilled,

	/// Requested quantity matched; durable order completion and trade settlement may still lag.
	Matched,

	/// Cancellation is reflected in the current order projection.
	Cancelled,

	/// The venue reports that the order reached its expiration.
	Expired,

	/// Submission was rejected; inspect the order’s typed public failure.
	Rejected,

	/// Processing failed; inspect the order’s typed public failure.
	Failed,

	/// Unrecognized display state; the order’s independent is_terminal flag may be true or false.
	#[serde(other)]
	Unknown,
}

/// The async engine operation an ack refers to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PendingOperation {
	/// An order submission was accepted for asynchronous processing.
	Submit,

	/// A single-order cancellation was accepted for asynchronous processing.
	Cancel,

	/// Cancellation of the caller’s open orders was accepted asynchronously.
	CancelAll,

	/// A pending operation this client version doesn't model.
	#[serde(other)]
	Unknown,
}

/// Which side of a fill an order was on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FillRole {
	/// This order supplied resting liquidity for the fill.
	Maker,

	/// This order consumed resting liquidity for the fill.
	Taker,

	/// A role this client version doesn't model.
	#[serde(other)]
	Unknown,
}

/// How a trade settled on-chain.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SettlementMode {
	/// Outcome tokens transfer between counterparties without minting or merging a complete set.
	Normal,

	/// Matching complementary buys mints a complete outcome set against collateral.
	Mint,

	/// Matching complementary sells merges a complete outcome set back into collateral.
	Merge,

	/// Unspecified or unrecognized settlement mode; no known settlement path is implied.
	#[default]
	#[serde(other)]
	Unspecified,
}

/// A collateral split / merge / redeem.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PositionOperation {
	/// Lock collateral and create equal shares of each outcome in a complete set.
	Split,

	/// Burn equal quantities of a complete outcome set to release collateral.
	Merge,

	/// Exchange resolved winning outcome shares for their collateral payout.
	Redeem,

	/// An operation this client version doesn't model.
	#[serde(other)]
	Unknown,
}

/// On-chain confirmation state of a relayer operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RelayerState {
	/// The relayer reports that the transaction was included in a block.
	Mined,

	/// The relayer reports that the transaction reached its confirmation threshold.
	Confirmed,

	/// A relayer state this client version doesn't model.
	#[serde(other)]
	Unknown,
}

fn uuid_text(value: String, field: Field) -> Result<String, ValidationError> {
	uuid::Uuid::parse_str(&value).map(|id| id.hyphenated().to_string()).map_err(|source| {
		ValidationError::identifier(field, IdentifierReason::InvalidUuid { source })
	})
}

fn digest_text(value: String, field: Field) -> Result<String, ValidationError> {
	if value.len() != DIGEST_HEX_LENGTH
		|| !value.starts_with("0x")
		|| !value.as_bytes()[2..].iter().all(u8::is_ascii_hexdigit)
	{
		return Err(ValidationError::identifier(
			field,
			IdentifierReason::InvalidDigest,
		));
	}

	Ok(value.to_ascii_lowercase())
}

fn token_text(value: String, field: Field) -> Result<String, ValidationError> {
	if value.is_empty() || !value.bytes().all(|b| b.is_ascii_digit()) {
		return Err(ValidationError::identifier(
			field,
			IdentifierReason::InvalidTokenId,
		));
	}

	let significant = value.trim_start_matches('0');
	let canonical = if significant.is_empty() {
		"0"
	} else {
		significant
	};
	if canonical.len() > MAX_TOKEN_DIGITS
		|| (canonical.len() == MAX_TOKEN_DIGITS && canonical > U256_MAX_DECIMAL)
	{
		return Err(ValidationError::identifier(
			field,
			IdentifierReason::InvalidTokenId,
		));
	}

	Ok(canonical.to_owned())
}

#[cfg(feature = "signing")]
fn digest_bytes(bytes: [u8; 32]) -> String {
	let mut text = String::with_capacity(DIGEST_HEX_LENGTH);
	text.push_str("0x");
	for byte in bytes {
		text.push(char::from(HEX_DIGITS[usize::from(byte >> 4)]));
		text.push(char::from(HEX_DIGITS[usize::from(byte & 0x0f)]));
	}

	text
}

impl OrderHash {
	#[cfg(feature = "signing")]
	pub(crate) fn from_bytes(bytes: [u8; 32]) -> Self {
		Self(digest_bytes(bytes))
	}
}

impl fmt::Display for Exchange {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(match self {
			Self::Agara => "AGARA",
			Self::Polymarket => "POLYMARKET",
			Self::Unknown => "UNKNOWN",
		})
	}
}

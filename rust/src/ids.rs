//! Domain newtypes and wire enums. Distinct id types stop a token id
//! being passed where a condition id is expected; the enums pin the
//! exact SCREAMING_SNAKE_CASE strings the router emits.

use core::fmt;

use serde::{Deserialize, Serialize};

macro_rules! string_id {
    ($(#[$doc:meta])* $name:ident) => {
        $(#[$doc])*
        #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            /// Wrap a raw identifier string.
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }

            /// Borrow the underlying string.
            pub fn as_str(&self) -> &str {
                &self.0
            }

            /// Consume into the owned string.
            pub fn into_inner(self) -> String {
                self.0
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}({:?})", stringify!($name), self.0)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self(value)
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self(value.to_owned())
            }
        }
    };
}

string_id!(
	/// An outcome-token id (matches `agara_market_outcomes.token_id`).
	TokenId
);
string_id!(
	/// A market/condition id on the CTF.
	ConditionId
);
string_id!(
	/// An order's internal UUID.
	OrderId
);
string_id!(
	/// An order's EIP-712 hash (`0x`-prefixed).
	OrderHash
);
string_id!(
	/// A wallet's internal UUID.
	WalletId
);

/// The backend an order / position / balance lives on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Exchange {
	Agara,
	Polymarket,
	/// A backend this client version doesn't model — keeps a new venue in a
	/// server response from failing the whole decode.
	#[serde(other)]
	Unknown,
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

/// Order side.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Side {
	Buy,
	Sell,
	/// Only seen on stream frames; never sent on a request.
	#[default]
	#[serde(other)]
	Unspecified,
}

/// Limit vs market order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderType {
	Limit,
	Market,
	/// An order type this client version doesn't model.
	#[serde(other)]
	Unknown,
}

/// How long an order rests.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TimeInForce {
	Gtc,
	Gtd,
	Fak,
	Fok,
	/// Only seen on stream frames; never sent on a request.
	#[default]
	#[serde(other)]
	Unspecified,
}

/// Lifecycle status of an order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderStatus {
	Pending,
	Submitting,
	Delayed,
	AwaitingBridge,
	Open,
	PartiallyFilled,
	Matched,
	Cancelled,
	Expired,
	Rejected,
	Failed,
	/// A status this client version doesn't model — treated as non-terminal
	/// so a new server state doesn't fail the decode or look "done".
	#[serde(other)]
	Unknown,
}

impl OrderStatus {
	/// Whether the order has reached a state it will never leave. Matches
	/// the engine's terminal set — a filled order rests at `MATCHED`;
	/// on-chain settlement to `CONFIRMED` is decoupled and asynchronous.
	pub fn is_terminal(self) -> bool {
		matches!(
			self,
			Self::Matched | Self::Cancelled | Self::Expired | Self::Rejected | Self::Failed
		)
	}
}

/// The async engine operation an ack refers to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PendingOperation {
	Submit,
	Cancel,
	CancelAll,
	/// A pending operation this client version doesn't model.
	#[serde(other)]
	Unknown,
}

/// Which side of a fill an order was on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FillRole {
	Maker,
	Taker,
	/// A role this client version doesn't model.
	#[serde(other)]
	Unknown,
}

/// How a trade settled on-chain.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SettlementMode {
	Normal,
	Mint,
	Merge,
	#[default]
	#[serde(other)]
	Unspecified,
}

/// A collateral split / merge / redeem.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PositionOperation {
	Split,
	Merge,
	Redeem,
	/// An operation this client version doesn't model.
	#[serde(other)]
	Unknown,
}

/// On-chain confirmation state of a relayer operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RelayerState {
	Mined,
	Confirmed,
	/// A relayer state this client version doesn't model.
	#[serde(other)]
	Unknown,
}

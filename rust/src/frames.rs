#![cfg(feature = "streaming")]

//! Typed stream channels and frames (feature `streaming`).
//!
//! Engine-native prices/sizes travel as integers with their own
//! `price_scale` / `size_scale` — divide to recover dollars / shares.
//! Future shapes retain their full envelope in [`Frame::Unknown`]; malformed known
//! shapes become [`Frame::Malformed`] with a precise decode error.

mod data;
mod decode;
mod lifecycle;
mod validation;

pub use data::{
	BestQuote, CrossMatchToggled, FeePolicyUpdated, Fill, Level, MarketCreated, MarketHalted,
	MarketResolved, MarketResumed, OrderAccepted, OrderCancelled, OrderRejected, OrderbookDelta,
	OrderbookSnapshot, OutcomeProposed, TokensMerged, TokensMinted, Trade,
};
pub use decode::{MAX_FRAME_BYTES, decode_frame, try_decode_frame, try_decode_text};
pub use lifecycle::{
	CollateralChanged, CurrentMarketChanged, ExpectedCycle, LifecycleMarket, LifecycleOutcome,
	LifecycleState, MarketResolutionCompleted, ReplacementStatus, TokensRedeemed,
};
pub use validation::{
	EncodedAmount, FrameDecodeError, FrameField, FrameFieldReason, FrameInvariant, Scale,
	SubjectViolation,
};

use serde_json::Value;

use crate::{
	ids::{ConditionId, EventId, TokenId},
	problem::PublicFailure,
	values::Timestamp,
};

/// One subscription request. Public channels need no token; `AccountEvents`
/// requires the client's bearer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Channel {
	/// Orderbook variant.
	Orderbook(TokenId),

	/// BestQuote variant.
	BestQuote(TokenId),

	/// MarketStatus variant.
	MarketStatus(ConditionId),

	/// EventMarketStatus variant.
	EventMarketStatus(EventId),

	/// Trades variant.
	Trades(ConditionId),

	/// AccountEvents variant.
	AccountEvents,
}

/// The server acknowledged a valid subscription subject.
#[derive(Clone, Debug)]
pub struct Subscribed {
	/// A channel with exactly its required typed subject.
	pub channel: Channel,
}

/// The server removed a subscription subject.
#[derive(Clone, Debug)]
pub struct Unsubscribed {
	/// A channel with exactly its required typed subject.
	pub channel: Channel,
}

/// Why a stream continuity boundary occurred; neither variant grants replay guarantees.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResetReason {
	/// The consumer missed messages while reading too slowly.
	Lagged,

	/// The upstream stream restarted.
	StreamReset,
}

/// A continuity boundary requiring channel-specific REST reconciliation or reseeding.
#[derive(Clone, Debug)]
pub struct SequenceReset {
	/// Affected typed subject.
	pub channel: Channel,

	/// Required diagnostic reason.
	pub reason: ResetReason,
}

/// Heartbeat.
#[derive(Clone, Debug, serde::Deserialize)]
pub struct Heartbeat {
	/// Server time.
	pub server_time: Timestamp,
}

/// Pong.
#[derive(Clone, Debug, serde::Deserialize)]
pub struct Pong {
	/// Server time.
	pub server_time: Timestamp,
}

/// A validated server failure and its bounded optional stream context.
#[derive(Clone, Debug)]
pub struct StreamError {
	/// Canonical failure; local transport failures use `Frame::ClientError` instead.
	pub failure: PublicFailure,

	/// Received action. Future actions remain visible but never execute automatically.
	pub action: WebSocketAction,

	/// Affected channel, when supplied.
	pub channel: Option<String>,

	/// Affected token, when supplied.
	pub token_id: Option<String>,

	/// Affected condition, when supplied.
	pub condition_id: Option<String>,

	/// Affected event UUID, when supplied.
	pub event_id: Option<String>,
}

/// Required recovery action on a WebSocket error frame.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum WebSocketAction {
	/// None variant.
	None,

	/// Resubscribe variant.
	Resubscribe,

	/// Reconnect variant.
	Reconnect,

	/// Unknown variant.
	Unknown(String),
}

/// Subscriptionlist.
#[derive(Clone, Debug, serde::Deserialize)]
pub struct SubscriptionList {
	/// Channels.
	pub channels: Vec<Subscription>,
}

/// A decoded server frame.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum Frame {
	/// A local connection event: reconcile these subjects before trusting incremental updates.
	ConnectionOpened(Vec<Channel>),

	/// OrderbookSnapshot variant.
	OrderbookSnapshot(OrderbookSnapshot),

	/// OrderbookDelta variant.
	OrderbookDelta(OrderbookDelta),

	/// BestQuote variant.
	BestQuote(BestQuote),

	/// MarketCreated variant.
	MarketCreated(MarketCreated),

	/// MarketHalted variant.
	MarketHalted(MarketHalted),

	/// MarketResumed variant.
	MarketResumed(MarketResumed),

	/// OutcomeProposed variant.
	OutcomeProposed(OutcomeProposed),

	/// MarketResolved variant.
	MarketResolved(MarketResolved),

	/// FeePolicyUpdated variant.
	FeePolicyUpdated(FeePolicyUpdated),

	/// CrossMatchToggled variant.
	CrossMatchToggled(CrossMatchToggled),

	/// Trade variant.
	Trade(Trade),

	/// Fill variant.
	Fill(Fill),

	/// OrderAccepted variant.
	OrderAccepted(OrderAccepted),

	/// OrderCancelled variant.
	OrderCancelled(OrderCancelled),

	/// OrderRejected variant.
	OrderRejected(OrderRejected),

	/// TokensMinted variant.
	TokensMinted(TokensMinted),

	/// TokensMerged variant.
	TokensMerged(TokensMerged),

	/// TokensRedeemed variant.
	TokensRedeemed(TokensRedeemed),

	/// CollateralDeposited variant.
	CollateralDeposited(CollateralChanged),

	/// CollateralWithdrawn variant.
	CollateralWithdrawn(CollateralChanged),

	/// CurrentMarketChanged variant.
	CurrentMarketChanged(CurrentMarketChanged),

	/// MarketResolutionCompleted variant.
	MarketResolutionCompleted(MarketResolutionCompleted),

	/// Subscribed variant.
	Subscribed(Subscribed),

	/// Unsubscribed variant.
	Unsubscribed(Unsubscribed),

	/// SequenceReset variant.
	SequenceReset(SequenceReset),

	/// Heartbeat variant.
	Heartbeat(Heartbeat),

	/// Pong variant.
	Pong(Pong),

	/// Error variant.
	Error(StreamError),

	/// SubscriptionList variant.
	SubscriptionList(SubscriptionList),

	/// A local stream failure, distinct from a server-originated failure.
	#[cfg(feature = "streaming")]
	ClientError(crate::stream::StreamClientError),

	/// A recognized frame violated its wire contract; the original value is preserved.
	Malformed {
		/// The precise decoding failure.
		error: FrameDecodeError,

		/// Original frame for diagnostics.
		raw: Value,
	},

	/// A wire shape this client version doesn't model.
	Unknown(Value),
}

/// One valid subject listed by the server; the wire discriminator is `channel`, not `name`.
#[derive(Clone, Debug)]
pub struct Subscription {
	/// Typed channel and its single applicable identifier.
	pub channel: Channel,
}

impl Channel {
	/// The server channel name; event and condition scopes both use `market_status`.
	pub fn name(&self) -> &'static str {
		match self {
			Self::Orderbook(_) => "orderbook",
			Self::BestQuote(_) => "best_quote",
			Self::MarketStatus(_) | Self::EventMarketStatus(_) => "market_status",
			Self::Trades(_) => "trades",
			Self::AccountEvents => "account_events",
		}
	}

	pub(crate) fn is_account(&self) -> bool {
		core::matches!(self, Self::AccountEvents)
	}

	pub(crate) fn to_wire(&self, token: Option<&str>) -> Value {
		let mut wire = serde_json::json!({ "name": self.name() });
		match self {
			Self::Orderbook(t) | Self::BestQuote(t) => {
				wire["token_id"] = Value::String(t.to_string());
			},
			Self::MarketStatus(c) | Self::Trades(c) => {
				wire["condition_id"] = Value::String(c.to_string());
			},
			Self::EventMarketStatus(event) => {
				wire["event_id"] = Value::String(event.to_string());
			},
			Self::AccountEvents => {
				if let Some(tok) = token {
					wire["token"] = Value::String(tok.to_owned());
				}
			},
		}

		wire
	}
}

impl OrderRejected {
	/// Deprecated compatibility summary. Match on `failure.code` for behavior;
	/// this never returns the old private provider diagnostic.
	pub fn reason(&self) -> &str {
		self.failure.detail.as_deref().unwrap_or(&self.failure.title)
	}
}

impl WebSocketAction {
	fn from_wire(value: Option<&str>) -> Self {
		match value {
			Some("none") | None => Self::None,
			Some("resubscribe") => Self::Resubscribe,
			Some("reconnect") => Self::Reconnect,
			Some(value) => Self::Unknown(value.to_owned()),
		}
	}
}

impl StreamError {
	/// The typed canonical or future server code.
	pub fn code(&self) -> &crate::problem::ProblemCode {
		&self.failure.code
	}

	/// Public detail when present, otherwise the validated title.
	pub fn message(&self) -> &str {
		self.failure.detail.as_deref().unwrap_or(&self.failure.title)
	}

	pub(crate) fn automatic_action(&self) -> Option<&WebSocketAction> {
		let expected = self.failure.websocket_action()?;
		match (expected, &self.action) {
			(crate::problem::WebSocketAction::Resubscribe, WebSocketAction::Resubscribe)
			| (crate::problem::WebSocketAction::Reconnect, WebSocketAction::Reconnect) => Some(&self.action),
			_ => None,
		}
	}
}

impl<'de> serde::Deserialize<'de> for Subscribed {
	fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		let raw = <Value as serde::Deserialize>::deserialize(deserializer)?;

		validation::subject(&raw).map(|channel| Self { channel }).map_err(serde::de::Error::custom)
	}
}

impl<'de> serde::Deserialize<'de> for Unsubscribed {
	fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		let raw = <Value as serde::Deserialize>::deserialize(deserializer)?;

		validation::subject(&raw).map(|channel| Self { channel }).map_err(serde::de::Error::custom)
	}
}

impl<'de> serde::Deserialize<'de> for Subscription {
	fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		let raw = <Value as serde::Deserialize>::deserialize(deserializer)?;

		validation::subject(&raw).map(|channel| Self { channel }).map_err(serde::de::Error::custom)
	}
}

impl<'de> serde::Deserialize<'de> for SequenceReset {
	fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		let raw = <Value as serde::Deserialize>::deserialize(deserializer)?;
		let channel = validation::subject(&raw).map_err(serde::de::Error::custom)?;
		let reason = serde_json::from_value(
			raw.get("reason").cloned().ok_or_else(|| serde::de::Error::missing_field("reason"))?,
		)
		.map_err(serde::de::Error::custom)?;

		Ok(Self { channel, reason })
	}
}

impl core::fmt::Display for Channel {
	fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		match self {
			Self::Orderbook(token) | Self::BestQuote(token) => {
				core::write!(formatter, "{}:{token}", self.name())
			},
			Self::MarketStatus(condition) | Self::Trades(condition) => {
				core::write!(formatter, "{}:{condition}", self.name())
			},
			Self::EventMarketStatus(event) => core::write!(formatter, "{}:{event}", self.name()),
			Self::AccountEvents => formatter.write_str(self.name()),
		}
	}
}

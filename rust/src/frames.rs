//! Typed stream channels and frames (feature `streaming`).
//!
//! Engine-native prices/sizes travel as integers with their own
//! `price_scale` / `size_scale` — divide to recover dollars / shares.
//! Unknown wire shapes decode to [`Frame::Unknown`] so a server-side
//! addition never crashes an old client.

use serde_json::Value;

use crate::ids::{ConditionId, FillRole, SettlementMode, Side, TimeInForce, TokenId};
use crate::units::Micro;

/// Stand-in for a missing `data` payload, so decoders can borrow rather
/// than clone.
static NULL_DATA: Value = Value::Null;

/// One subscription request. Public channels need no token; `AccountEvents`
/// requires the client's bearer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Channel {
	Orderbook(TokenId),
	BestQuote(TokenId),
	MarketStatus(ConditionId),
	Trades(ConditionId),
	AccountEvents,
}

impl Channel {
	pub(crate) fn name(&self) -> &'static str {
		match self {
			Self::Orderbook(_) => "orderbook",
			Self::BestQuote(_) => "best_quote",
			Self::MarketStatus(_) => "market_status",
			Self::Trades(_) => "trades",
			Self::AccountEvents => "account_events",
		}
	}

	pub(crate) fn is_account(&self) -> bool {
		matches!(self, Self::AccountEvents)
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
			Self::AccountEvents => {
				if let Some(tok) = token {
					wire["token"] = Value::String(tok.to_owned());
				}
			},
		}

		wire
	}
}

/// An engine-native price level.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Level {
	pub price: i64,
	pub size: i64,
}

#[derive(Clone, Debug)]
pub struct OrderbookSnapshot {
	pub token_id: TokenId,
	pub sequence: u64,
	pub bids: Vec<Level>,
	pub asks: Vec<Level>,
	pub tick_size: i64,
	pub price_scale: i64,
	pub size_scale: i64,
}

#[derive(Clone, Debug)]
pub struct OrderbookDelta {
	pub token_id: TokenId,
	pub sequence: u64,
	/// Each entry replaces the level at that price; `size == 0` removes it.
	pub bids: Vec<Level>,
	pub asks: Vec<Level>,
	pub price_scale: i64,
	pub size_scale: i64,
}

#[derive(Clone, Debug)]
pub struct BestQuote {
	pub token_id: TokenId,
	pub sequence: u64,
	pub bid: Option<Level>,
	pub ask: Option<Level>,
	pub price_scale: i64,
	pub size_scale: i64,
}

#[derive(Clone, Debug)]
pub struct MarketCreated {
	pub condition_id: ConditionId,
	pub sequence: u64,
	pub num_outcomes: i64,
	pub tick_size: i64,
	pub price_scale: i64,
	pub size_scale: i64,
	pub min_price: i64,
	pub max_price: i64,
	pub cross_match_enabled: bool,
}

#[derive(Clone, Debug)]
pub struct MarketHalted {
	pub condition_id: ConditionId,
	pub sequence: u64,
}

#[derive(Clone, Debug)]
pub struct MarketResumed {
	pub condition_id: ConditionId,
	pub sequence: u64,
}

#[derive(Clone, Debug)]
pub struct OutcomeProposed {
	pub condition_id: ConditionId,
	pub sequence: u64,
	pub proposed_token_id: TokenId,
}

#[derive(Clone, Debug)]
pub struct MarketResolved {
	pub condition_id: ConditionId,
	pub sequence: u64,
	pub winning_token_id: TokenId,
}

#[derive(Clone, Debug)]
pub struct FeePolicyUpdated {
	pub condition_id: ConditionId,
	pub sequence: u64,
}

#[derive(Clone, Debug)]
pub struct CrossMatchToggled {
	pub condition_id: ConditionId,
	pub sequence: u64,
	pub enabled: bool,
}

#[derive(Clone, Debug)]
pub struct Trade {
	pub condition_id: ConditionId,
	pub sequence: u64,
	pub fill_id: String,
	pub taker_token_id: TokenId,
	pub maker_token_id: TokenId,
	pub side: Side,
	pub price: i64,
	pub size: i64,
	pub price_scale: i64,
	pub size_scale: i64,
	pub settlement_mode: SettlementMode,
}

#[derive(Clone, Debug)]
pub struct Fill {
	pub sequence: u64,
	pub role: FillRole,
	pub fill_id: String,
	pub order_id: String,
	pub order_hash: String,
	pub token_id: TokenId,
	pub side: Side,
	pub price: i64,
	pub size: i64,
	pub price_scale: i64,
	pub size_scale: i64,
	pub settlement_mode: SettlementMode,
	pub fee_micro: Micro,
}

#[derive(Clone, Debug)]
pub struct OrderAccepted {
	pub sequence: u64,
	pub order_id: String,
	pub order_hash: String,
	pub token_id: TokenId,
	pub side: Side,
	pub price: i64,
	pub remaining_size: i64,
	pub original_size: i64,
	pub price_scale: i64,
	pub size_scale: i64,
	pub tif: TimeInForce,
}

#[derive(Clone, Debug)]
pub struct OrderCancelled {
	pub sequence: u64,
	pub order_id: String,
	pub order_hash: String,
	pub token_id: TokenId,
	pub side: Side,
	pub price: i64,
	pub remaining_size: i64,
	pub price_scale: i64,
	pub size_scale: i64,
	/// `USER`, `FAK_REMAINDER`, `SELF_TRADE_PREVENTION`, `MARKET_RESOLVED`.
	pub reason: String,
}

#[derive(Clone, Debug)]
pub struct OrderRejected {
	/// Router-originated, so `sequence` is 0 — dedupe by `order_id`.
	pub sequence: u64,
	pub order_id: String,
	pub order_hash: Option<String>,
	pub token_id: Option<TokenId>,
	pub reason: String,
}

#[derive(Clone, Debug)]
pub struct TokensMinted {
	pub sequence: u64,
	pub condition_id: ConditionId,
	pub size: i64,
	pub size_scale: i64,
}

#[derive(Clone, Debug)]
pub struct TokensMerged {
	pub sequence: u64,
	pub condition_id: ConditionId,
	pub size: i64,
	pub size_scale: i64,
}

#[derive(Clone, Debug)]
pub struct Subscribed {
	pub channel: String,
	pub token_id: Option<String>,
	pub condition_id: Option<String>,
}

#[derive(Clone, Debug)]
pub struct Unsubscribed {
	pub channel: String,
	pub token_id: Option<String>,
	pub condition_id: Option<String>,
}

#[derive(Clone, Debug)]
pub struct SequenceReset {
	pub channel: String,
	/// `lagged` (slow read) or `stream_reset` (upstream blip). Recovery is
	/// identical: discard local state for the subject and rebuild.
	pub reason: Option<String>,
	pub token_id: Option<String>,
	pub condition_id: Option<String>,
}

#[derive(Clone, Debug)]
pub struct Heartbeat {
	pub server_time: String,
}

#[derive(Clone, Debug)]
pub struct Pong {
	pub server_time: String,
}

#[derive(Clone, Debug)]
pub struct StreamError {
	pub code: String,
	pub message: String,
	/// `resubscribe`, `reconnect`, or `None` (terminal). In callback mode
	/// the client acts on this automatically.
	pub action: Option<String>,
	pub channel: Option<String>,
	pub token_id: Option<String>,
	pub condition_id: Option<String>,
}

#[derive(Clone, Debug)]
pub struct SubscriptionList {
	pub channels: Vec<Value>,
}

/// A decoded server frame.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum Frame {
	OrderbookSnapshot(OrderbookSnapshot),
	OrderbookDelta(OrderbookDelta),
	BestQuote(BestQuote),
	MarketCreated(MarketCreated),
	MarketHalted(MarketHalted),
	MarketResumed(MarketResumed),
	OutcomeProposed(OutcomeProposed),
	MarketResolved(MarketResolved),
	FeePolicyUpdated(FeePolicyUpdated),
	CrossMatchToggled(CrossMatchToggled),
	Trade(Trade),
	Fill(Fill),
	OrderAccepted(OrderAccepted),
	OrderCancelled(OrderCancelled),
	OrderRejected(OrderRejected),
	TokensMinted(TokensMinted),
	TokensMerged(TokensMerged),
	Subscribed(Subscribed),
	Unsubscribed(Unsubscribed),
	SequenceReset(SequenceReset),
	Heartbeat(Heartbeat),
	Pong(Pong),
	Error(StreamError),
	SubscriptionList(SubscriptionList),
	/// A wire shape this client version doesn't model.
	Unknown(Value),
}

/// Decode one parsed server frame. Unknown shapes fall through to
/// [`Frame::Unknown`].
pub fn decode_frame(raw: Value) -> Frame {
	match str_at(&raw, "op").as_deref() {
		Some("update") => decode_update(raw),
		Some("subscribed") => Frame::Subscribed(Subscribed {
			channel: str_at(&raw, "channel").unwrap_or_default(),
			token_id: str_at(&raw, "token_id"),
			condition_id: str_at(&raw, "condition_id"),
		}),
		Some("unsubscribed") => Frame::Unsubscribed(Unsubscribed {
			channel: str_at(&raw, "channel").unwrap_or_default(),
			token_id: str_at(&raw, "token_id"),
			condition_id: str_at(&raw, "condition_id"),
		}),
		Some("sequence_reset") => Frame::SequenceReset(SequenceReset {
			channel: str_at(&raw, "channel").unwrap_or_default(),
			reason: str_at(&raw, "reason"),
			token_id: str_at(&raw, "token_id"),
			condition_id: str_at(&raw, "condition_id"),
		}),
		Some("heartbeat") => Frame::Heartbeat(Heartbeat {
			server_time: str_at(&raw, "server_time").unwrap_or_default(),
		}),
		Some("pong") => {
			Frame::Pong(Pong { server_time: str_at(&raw, "server_time").unwrap_or_default() })
		},
		Some("error") => Frame::Error(StreamError {
			code: str_at(&raw, "code").unwrap_or_default(),
			message: str_at(&raw, "message").unwrap_or_default(),
			action: str_at(&raw, "action"),
			channel: str_at(&raw, "channel"),
			token_id: str_at(&raw, "token_id"),
			condition_id: str_at(&raw, "condition_id"),
		}),
		Some("subscription_list") => Frame::SubscriptionList(SubscriptionList {
			channels: raw.get("channels").and_then(Value::as_array).cloned().unwrap_or_default(),
		}),
		_ => Frame::Unknown(raw),
	}
}

fn decode_update(raw: Value) -> Frame {
	let channel = str_at(&raw, "channel").unwrap_or_default();
	let sequence = u64_at(&raw, "sequence");
	// Borrow the payload — every decoder only reads it. Cloning the whole
	// `data` object (the full book on a snapshot) per frame would be waste
	// on the hottest path.
	let data = raw.get("data").unwrap_or(&NULL_DATA);
	match channel.as_str() {
		"orderbook" => decode_orderbook(&raw, sequence, data),
		"best_quote" => Frame::BestQuote(BestQuote {
			token_id: TokenId::new(str_at(&raw, "token_id").unwrap_or_default()),
			sequence,
			bid: optional_level(data.get("bid")),
			ask: optional_level(data.get("ask")),
			price_scale: i64_at(data, "price_scale"),
			size_scale: i64_at(data, "size_scale"),
		}),
		"market_status" => decode_market_status(&raw, sequence, data),
		"trades" => decode_trade(&raw, sequence, data),
		"account_events" => decode_account_event(&raw, sequence, data),
		_ => Frame::Unknown(raw),
	}
}

fn decode_orderbook(raw: &Value, sequence: u64, data: &Value) -> Frame {
	let token_id = TokenId::new(str_at(raw, "token_id").unwrap_or_default());
	let bids = array_levels(data.get("bids"));
	let asks = array_levels(data.get("asks"));
	match str_at(data, "kind").as_deref() {
		Some("snapshot") => Frame::OrderbookSnapshot(OrderbookSnapshot {
			token_id,
			sequence,
			bids,
			asks,
			tick_size: i64_at(data, "tick_size"),
			price_scale: i64_at(data, "price_scale"),
			size_scale: i64_at(data, "size_scale"),
		}),
		Some("delta") => Frame::OrderbookDelta(OrderbookDelta {
			token_id,
			sequence,
			bids,
			asks,
			price_scale: i64_at(data, "price_scale"),
			size_scale: i64_at(data, "size_scale"),
		}),
		_ => Frame::Unknown(raw.clone()),
	}
}

fn decode_market_status(raw: &Value, sequence: u64, data: &Value) -> Frame {
	let cid = ConditionId::new(str_at(raw, "condition_id").unwrap_or_default());
	match str_at(data, "kind").as_deref() {
		Some("market_created") => Frame::MarketCreated(MarketCreated {
			condition_id: cid,
			sequence,
			num_outcomes: i64_at(data, "num_outcomes"),
			tick_size: i64_at(data, "tick_size"),
			price_scale: i64_at(data, "price_scale"),
			size_scale: i64_at(data, "size_scale"),
			min_price: i64_at(data, "min_price"),
			max_price: i64_at(data, "max_price"),
			cross_match_enabled: bool_at(data, "cross_match_enabled"),
		}),
		Some("market_halted") => Frame::MarketHalted(MarketHalted { condition_id: cid, sequence }),
		Some("market_resumed") => {
			Frame::MarketResumed(MarketResumed { condition_id: cid, sequence })
		},
		Some("outcome_proposed") => Frame::OutcomeProposed(OutcomeProposed {
			condition_id: cid,
			sequence,
			proposed_token_id: TokenId::new(str_at(data, "proposed_token_id").unwrap_or_default()),
		}),
		Some("market_resolved") => Frame::MarketResolved(MarketResolved {
			condition_id: cid,
			sequence,
			winning_token_id: TokenId::new(str_at(data, "winning_token_id").unwrap_or_default()),
		}),
		Some("fee_policy_updated") => {
			Frame::FeePolicyUpdated(FeePolicyUpdated { condition_id: cid, sequence })
		},
		Some("cross_match_toggled") => Frame::CrossMatchToggled(CrossMatchToggled {
			condition_id: cid,
			sequence,
			enabled: bool_at(data, "enabled"),
		}),
		_ => Frame::Unknown(raw.clone()),
	}
}

fn decode_trade(raw: &Value, sequence: u64, data: &Value) -> Frame {
	if str_at(data, "kind").as_deref() != Some("trade") {
		return Frame::Unknown(raw.clone());
	}

	Frame::Trade(Trade {
		condition_id: ConditionId::new(str_at(raw, "condition_id").unwrap_or_default()),
		sequence,
		fill_id: str_at(data, "fill_id").unwrap_or_default(),
		taker_token_id: TokenId::new(str_at(data, "taker_token_id").unwrap_or_default()),
		maker_token_id: TokenId::new(str_at(data, "maker_token_id").unwrap_or_default()),
		side: enum_at(data, "side"),
		price: i64_at(data, "price"),
		size: i64_at(data, "size"),
		price_scale: i64_at(data, "price_scale"),
		size_scale: i64_at(data, "size_scale"),
		settlement_mode: enum_at(data, "settlement_mode"),
	})
}

fn decode_account_event(raw: &Value, sequence: u64, data: &Value) -> Frame {
	match str_at(data, "kind").as_deref() {
		Some("fill") => Frame::Fill(Fill {
			sequence,
			role: serde_json::from_value(data.get("role").cloned().unwrap_or(Value::Null))
				.unwrap_or(FillRole::Taker),
			fill_id: str_at(data, "fill_id").unwrap_or_default(),
			order_id: str_at(data, "order_id").unwrap_or_default(),
			order_hash: str_at(data, "order_hash").unwrap_or_default(),
			token_id: TokenId::new(str_at(data, "token_id").unwrap_or_default()),
			side: enum_at(data, "side"),
			price: i64_at(data, "price"),
			size: i64_at(data, "size"),
			price_scale: i64_at(data, "price_scale"),
			size_scale: i64_at(data, "size_scale"),
			settlement_mode: enum_at(data, "settlement_mode"),
			fee_micro: Micro::new(i64_at(data, "fee_micro")),
		}),
		Some("order_accepted") => Frame::OrderAccepted(OrderAccepted {
			sequence,
			order_id: str_at(data, "order_id").unwrap_or_default(),
			order_hash: str_at(data, "order_hash").unwrap_or_default(),
			token_id: TokenId::new(str_at(data, "token_id").unwrap_or_default()),
			side: enum_at(data, "side"),
			price: i64_at(data, "price"),
			remaining_size: i64_at(data, "remaining_size"),
			original_size: i64_at(data, "original_size"),
			price_scale: i64_at(data, "price_scale"),
			size_scale: i64_at(data, "size_scale"),
			tif: enum_at(data, "tif"),
		}),
		Some("order_cancelled") => Frame::OrderCancelled(OrderCancelled {
			sequence,
			order_id: str_at(data, "order_id").unwrap_or_default(),
			order_hash: str_at(data, "order_hash").unwrap_or_default(),
			token_id: TokenId::new(str_at(data, "token_id").unwrap_or_default()),
			side: enum_at(data, "side"),
			price: i64_at(data, "price"),
			remaining_size: i64_at(data, "remaining_size"),
			price_scale: i64_at(data, "price_scale"),
			size_scale: i64_at(data, "size_scale"),
			reason: str_at(data, "reason").unwrap_or_default(),
		}),
		Some("order_rejected") => Frame::OrderRejected(OrderRejected {
			sequence,
			order_id: str_at(data, "order_id").unwrap_or_default(),
			order_hash: str_at(data, "order_hash"),
			token_id: str_at(data, "token_id").map(TokenId::new),
			reason: str_at(data, "reason").unwrap_or_default(),
		}),
		Some("tokens_minted") => Frame::TokensMinted(TokensMinted {
			sequence,
			condition_id: ConditionId::new(str_at(data, "condition_id").unwrap_or_default()),
			size: i64_at(data, "size"),
			size_scale: i64_at(data, "size_scale"),
		}),
		Some("tokens_merged") => Frame::TokensMerged(TokensMerged {
			sequence,
			condition_id: ConditionId::new(str_at(data, "condition_id").unwrap_or_default()),
			size: i64_at(data, "size"),
			size_scale: i64_at(data, "size_scale"),
		}),
		_ => Frame::Unknown(raw.clone()),
	}
}

fn str_at(v: &Value, key: &str) -> Option<String> {
	v.get(key).and_then(Value::as_str).map(str::to_owned)
}

fn i64_at(v: &Value, key: &str) -> i64 {
	v.get(key)
		.and_then(|x| x.as_i64().or_else(|| x.as_str().and_then(|s| s.parse().ok())))
		.unwrap_or(0)
}

fn u64_at(v: &Value, key: &str) -> u64 {
	v.get(key)
		.and_then(|x| x.as_u64().or_else(|| x.as_str().and_then(|s| s.parse().ok())))
		.unwrap_or(0)
}

fn bool_at(v: &Value, key: &str) -> bool {
	v.get(key).and_then(Value::as_bool).unwrap_or(false)
}

fn enum_at<T: serde::de::DeserializeOwned + Default>(v: &Value, key: &str) -> T {
	v.get(key).cloned().and_then(|x| serde_json::from_value(x).ok()).unwrap_or_default()
}

fn level_from_array(v: &Value) -> Option<Level> {
	let arr = v.as_array()?;
	Some(Level {
		price: arr.first().and_then(Value::as_i64).unwrap_or(0),
		size: arr.get(1).and_then(Value::as_i64).unwrap_or(0),
	})
}

fn array_levels(v: Option<&Value>) -> Vec<Level> {
	v.and_then(Value::as_array)
		.map(|arr| arr.iter().filter_map(level_from_array).collect())
		.unwrap_or_default()
}

fn optional_level(v: Option<&Value>) -> Option<Level> {
	let obj = v?;
	if obj.is_null() {
		return None;
	}

	Some(Level { price: i64_at(obj, "price"), size: i64_at(obj, "size") })
}

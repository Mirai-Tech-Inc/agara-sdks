mod tests;

use super::{
	BestQuote, CollateralChanged, CrossMatchToggled, CurrentMarketChanged, FeePolicyUpdated, Fill,
	Frame, FrameDecodeError, FrameField, FrameFieldReason, Heartbeat, MarketCreated, MarketHalted,
	MarketResolutionCompleted, MarketResolved, MarketResumed, OrderAccepted, OrderCancelled,
	OrderRejected, OrderbookDelta, OrderbookSnapshot, OutcomeProposed, Pong, SequenceReset,
	StreamError, SubjectViolation, Subscribed, SubscriptionList, TokensMerged, TokensMinted,
	TokensRedeemed, Trade, Unsubscribed, WebSocketAction,
};

use crate::problem::PublicFailure;

use serde_json::Value;
/// Maximum decoded WebSocket text bytes accepted by the SDK.
pub const MAX_FRAME_BYTES: usize = 8 * 1024 * 1024;

/// Decode bounded JSON text without discarding duplicate properties.
/// Unknown future messages remain inspectable; malformed known frames return a typed error.
pub fn try_decode_text(text: &str) -> Result<Frame, super::FrameDecodeError> {
	let raw = crate::problem::parse_json_value_with_limit(text.as_bytes(), MAX_FRAME_BYTES)
		.map_err(|source| super::FrameDecodeError::Json(std::sync::Arc::new(source)))?;

	try_decode_frame(raw)
}

/// Decode a parsed frame, retaining typed malformed-known errors and untouched future frames.
pub fn decode_frame(raw: Value) -> Frame {
	match try_decode_frame(raw.clone()) {
		Ok(frame) => frame,
		Err(error) => Frame::Malformed { error, raw },
	}
}

/// Decode a current frame, distinguishing malformed known data from unknown future messages.
/// Unknown operations, channels and update kinds return `Ok(Frame::Unknown(raw))`.
pub fn try_decode_frame(raw: Value) -> Result<Frame, super::FrameDecodeError> {
	let op = raw
		.get("op")
		.and_then(Value::as_str)
		.filter(|value| !value.is_empty())
		.ok_or(super::FrameDecodeError::Envelope)?;
	match op {
		"update" => decode_update(raw),
		"subscribed" | "unsubscribed" | "sequence_reset" => {
			let channel = super::validation::text(&raw, FrameField::Channel)?;
			if !known_channel(Some(channel)) {
				return Ok(Frame::Unknown(raw));
			}
			super::validation::validate_subject(&raw, true)?;
			match op {
				"subscribed" => decode::<Subscribed>(raw, Frame::Subscribed),
				"unsubscribed" => decode::<Unsubscribed>(raw, Frame::Unsubscribed),
				_ => {
					if !core::matches!(
						raw.get("reason").and_then(Value::as_str),
						Some("lagged" | "stream_reset")
					) {
						return Err(FrameDecodeError::Field {
							field: FrameField::ResetReason,
							reason: FrameFieldReason::UnknownVariant,
						});
					}
					decode::<SequenceReset>(raw, Frame::SequenceReset)
				},
			}
		},
		"heartbeat" => decode::<Heartbeat>(raw, Frame::Heartbeat),
		"pong" => decode::<Pong>(raw, Frame::Pong),
		"subscription_list" => {
			let entries =
				raw.get("channels").and_then(Value::as_array).ok_or(FrameDecodeError::Field {
					field: FrameField::Channels,
					reason: FrameFieldReason::ExpectedArray,
				})?;
			for entry in entries {
				super::validation::validate_subject(entry, true)?;
			}
			decode::<SubscriptionList>(raw, Frame::SubscriptionList)
		},
		"error" => decode_error(raw),
		_ => Ok(Frame::Unknown(raw)),
	}
}

fn known_channel(channel: Option<&str>) -> bool {
	core::matches!(
		channel,
		Some("orderbook" | "best_quote" | "market_status" | "trades" | "account_events")
	)
}

fn decode<T: serde::de::DeserializeOwned>(
	payload: Value,
	wrap: fn(T) -> Frame,
) -> Result<Frame, super::FrameDecodeError> {
	super::validation::validate_payload(&payload)?;
	serde_json::from_value(payload)
		.map(wrap)
		.map_err(|source| super::FrameDecodeError::Payload(std::sync::Arc::new(source)))
}

fn decode_error(raw: Value) -> Result<Frame, super::FrameDecodeError> {
	let Some(failure) = raw.get("failure") else {
		if raw.get("code").is_some() {
			return Ok(Frame::Unknown(raw));
		}
		return Err(FrameDecodeError::Field {
			field: FrameField::Failure,
			reason: FrameFieldReason::Missing,
		});
	};
	let failure = PublicFailure::parse(failure.clone())
		.map_err(|source| FrameDecodeError::Problem(std::sync::Arc::new(source)))?;
	super::validation::validate_subject(&raw, false)?;
	let action = super::validation::text(&raw, FrameField::Action)?;
	if action.chars().count() > 64 {
		return Err(FrameDecodeError::Field {
			field: FrameField::Action,
			reason: FrameFieldReason::TooLong { limit: 64, actual: action.chars().count() },
		});
	}

	Ok(Frame::Error(StreamError {
		failure,
		action: WebSocketAction::from_wire(Some(action)),
		channel: str_at(&raw, "channel"),
		token_id: str_at(&raw, "token_id"),
		condition_id: str_at(&raw, "condition_id"),
		event_id: str_at(&raw, "event_id"),
	}))
}

fn decode_update(raw: Value) -> Result<Frame, super::FrameDecodeError> {
	let channel_name = super::validation::text(&raw, FrameField::Channel)?;
	if !known_channel(Some(channel_name)) {
		return Ok(Frame::Unknown(raw));
	}
	let data = raw.get("data").ok_or(FrameDecodeError::Field {
		field: FrameField::Data,
		reason: FrameFieldReason::Missing,
	})?;
	let object = data.as_object().ok_or(FrameDecodeError::Field {
		field: FrameField::Data,
		reason: FrameFieldReason::ExpectedObject,
	})?;
	let kind = object.get("kind");
	if kind.is_some_and(|kind| !kind.is_string()) {
		return Err(FrameDecodeError::Field {
			field: FrameField::Kind,
			reason: FrameFieldReason::ExpectedString,
		});
	}
	if kind.and_then(Value::as_str) == Some("") {
		return Err(FrameDecodeError::Field {
			field: FrameField::Kind,
			reason: FrameFieldReason::Empty,
		});
	}
	let recognized = core::matches!(
		(channel_name, kind.and_then(Value::as_str)),
		("orderbook", Some("snapshot" | "delta"))
			| ("best_quote", None)
			| (
				"market_status",
				Some(
					"market_created"
						| "market_halted" | "market_resumed"
						| "outcome_proposed"
						| "market_resolved"
						| "fee_policy_updated"
						| "cross_match_toggled"
						| "current_market_changed"
						| "market_resolution_completed"
				)
			) | ("trades", Some("trade"))
			| (
				"account_events",
				Some(
					"fill"
						| "order_accepted" | "order_cancelled"
						| "order_rejected" | "tokens_minted"
						| "tokens_merged" | "tokens_redeemed"
						| "collateral_deposited"
						| "collateral_withdrawn"
				)
			)
	);
	if !recognized {
		if kind.is_none() && channel_name != "best_quote" {
			return Err(FrameDecodeError::Field {
				field: FrameField::Kind,
				reason: FrameFieldReason::Missing,
			});
		}
		return Ok(Frame::Unknown(raw));
	}
	if super::validation::has_future_enum(&raw["data"]) {
		return Ok(Frame::Unknown(raw));
	}
	let Some(mut payload) = raw.get("data").and_then(Value::as_object).cloned() else {
		return Err(FrameDecodeError::Field {
			field: FrameField::Data,
			reason: FrameFieldReason::ExpectedObject,
		});
	};
	for field in [
		FrameField::Sequence,
		FrameField::TokenId,
		FrameField::ConditionId,
		FrameField::EventId,
	] {
		let key = field.as_str();
		if let Some(value) = raw.get(key) {
			if payload.get(key).is_some_and(|existing| existing != value) {
				return Err(FrameDecodeError::IdentityConflict { field });
			}
			payload.insert(key.to_owned(), value.clone());
		}
	}

	let channel = str_at(&raw, "channel").unwrap_or_default();
	let kind =
		raw.get("data").and_then(|v| v.get("kind")).and_then(Value::as_str).unwrap_or_default();
	if known_channel(Some(&channel)) {
		super::validation::validate_subject(&raw, true)?;
	}
	let event_scope = raw.get("event_id").is_some_and(|value| !value.is_null());
	let event_kind = core::matches!(
		kind,
		"current_market_changed" | "market_resolution_completed"
	);
	if channel == "market_status" && event_kind != event_scope {
		return Err(FrameDecodeError::Subject(SubjectViolation::LifecycleScope));
	}
	if event_scope && raw.get("sequence").is_some() {
		return Err(FrameDecodeError::Field {
			field: FrameField::Sequence,
			reason: FrameFieldReason::Unexpected,
		});
	}
	if !event_scope {
		let sequence = raw.get("sequence").ok_or(FrameDecodeError::Field {
			field: FrameField::Sequence,
			reason: FrameFieldReason::Missing,
		})?;
		if sequence.as_u64().is_none() {
			return Err(FrameDecodeError::Field {
				field: FrameField::Sequence,
				reason: FrameFieldReason::ExpectedUnsignedInteger,
			});
		}
	}
	if kind == "order_rejected" {
		let failure = payload.get("failure").ok_or(FrameDecodeError::Field {
			field: FrameField::Failure,
			reason: FrameFieldReason::Missing,
		})?;
		PublicFailure::parse(failure.clone())
			.map_err(|source| FrameDecodeError::Problem(std::sync::Arc::new(source)))?;
	}
	let payload = Value::Object(payload);
	match (channel.as_str(), kind) {
		("orderbook", "snapshot") => decode::<OrderbookSnapshot>(payload, Frame::OrderbookSnapshot),
		("orderbook", "delta") => decode::<OrderbookDelta>(payload, Frame::OrderbookDelta),
		("best_quote", "") => decode::<BestQuote>(payload, Frame::BestQuote),
		("market_status", "market_created") => {
			decode::<MarketCreated>(payload, Frame::MarketCreated)
		},
		("market_status", "market_halted") => decode::<MarketHalted>(payload, Frame::MarketHalted),
		("market_status", "market_resumed") => {
			decode::<MarketResumed>(payload, Frame::MarketResumed)
		},
		("market_status", "outcome_proposed") => {
			decode::<OutcomeProposed>(payload, Frame::OutcomeProposed)
		},
		("market_status", "market_resolved") => {
			decode::<MarketResolved>(payload, Frame::MarketResolved)
		},
		("market_status", "fee_policy_updated") => {
			decode::<FeePolicyUpdated>(payload, Frame::FeePolicyUpdated)
		},
		("market_status", "cross_match_toggled") => {
			decode::<CrossMatchToggled>(payload, Frame::CrossMatchToggled)
		},
		("market_status", "current_market_changed") => {
			decode::<CurrentMarketChanged>(payload, Frame::CurrentMarketChanged)
		},
		("market_status", "market_resolution_completed") => {
			decode::<MarketResolutionCompleted>(payload, Frame::MarketResolutionCompleted)
		},
		("trades", "trade") => decode::<Trade>(payload, Frame::Trade),
		("account_events", "fill") => decode::<Fill>(payload, Frame::Fill),
		("account_events", "order_accepted") => {
			decode::<OrderAccepted>(payload, Frame::OrderAccepted)
		},
		("account_events", "order_cancelled") => {
			decode::<OrderCancelled>(payload, Frame::OrderCancelled)
		},
		("account_events", "order_rejected") => {
			decode::<OrderRejected>(payload, Frame::OrderRejected)
		},
		("account_events", "tokens_minted") => decode::<TokensMinted>(payload, Frame::TokensMinted),
		("account_events", "tokens_merged") => decode::<TokensMerged>(payload, Frame::TokensMerged),
		("account_events", "tokens_redeemed") => {
			decode::<TokensRedeemed>(payload, Frame::TokensRedeemed)
		},
		("account_events", "collateral_deposited") => {
			decode::<CollateralChanged>(payload, Frame::CollateralDeposited)
		},
		("account_events", "collateral_withdrawn") => {
			decode::<CollateralChanged>(payload, Frame::CollateralWithdrawn)
		},
		_ => Ok(Frame::Unknown(raw)),
	}
}

fn str_at(v: &Value, key: &str) -> Option<String> {
	v.get(key).and_then(Value::as_str).map(str::to_owned)
}

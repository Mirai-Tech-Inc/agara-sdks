//! Deserializer coverage against the exact router wire shapes. No
//! network — these pin the JSON contract the SDK decodes.

use agara_sdk::Micro;
use agara_sdk::ids::{Exchange, FillRole, OrderStatus, PendingOperation, Side};
use agara_sdk::models::{
	CreateClobOrderResponse, Fill, Order, Orderbook, SignedOrderResult, StatusResponse,
};

#[test]
fn micro_accepts_string_and_number() {
	let from_str: Micro = serde_json::from_str(r#""1000000""#).unwrap();
	let from_num: Micro = serde_json::from_str("1000000").unwrap();
	assert_eq!(from_str, from_num);
	assert_eq!(from_str.raw(), 1_000_000);
	assert_eq!(from_str.as_decimal(), rust_decimal::Decimal::ONE);
}

#[test]
fn micro_round_trips_dollars() {
	let m = Micro::from_units("0.6".parse().unwrap());
	assert_eq!(m.raw(), 600_000);
	assert_eq!(serde_json::to_string(&m).unwrap(), r#""600000""#);
}

#[test]
fn create_order_ack_decodes() {
	let ack: CreateClobOrderResponse = serde_json::from_str(
		r#"{"order_id":"abc","source":"AGARA","status":"OPEN",
            "pending_operation":"SUBMIT","as_of":"2026-07-08T00:00:00Z"}"#,
	)
	.unwrap();
	assert_eq!(ack.order_id.as_str(), "abc");
	assert_eq!(ack.source, Exchange::Agara);
	assert_eq!(ack.status, OrderStatus::Open);
	assert_eq!(ack.pending_operation, PendingOperation::Submit);
}

#[test]
fn signed_batch_result_decodes_both_arms() {
	let accepted: SignedOrderResult = serde_json::from_str(
		r#"{"index":0,"outcome":"accepted","order_id":"o1","source":"AGARA",
            "status":"OPEN","pending_operation":"SUBMIT","as_of":"t"}"#,
	)
	.unwrap();
	match accepted {
		SignedOrderResult::Accepted { index, ack } => {
			assert_eq!(index, 0);
			assert_eq!(ack.order_id.as_str(), "o1");
		},
		_ => panic!("expected accepted"),
	}

	let rejected: SignedOrderResult = serde_json::from_str(
		r#"{"index":1,"outcome":"rejected","failure":{"code":"duplicate_order",
            "title":"Duplicate order","detail":"An order with this identity already exists.",
            "recovery":{"strategy":"none"}}}"#,
	)
	.unwrap();
	match rejected {
		SignedOrderResult::Rejected { index, failure } => {
			assert_eq!(index, 1);
			assert_eq!(failure.code, "duplicate_order");
			assert_eq!(failure.recovery.strategy(), "none");
		},
		_ => panic!("expected rejected"),
	}
}

#[test]
fn order_decodes_with_nullable_fields() {
	let order: Order = serde_json::from_str(
		r#"{"internal_id":"id","exchange":"AGARA","token_id":"tok","condition_id":null,
            "side":"BUY","type":"LIMIT","price_micro":"600000","original_size_micro":"1000000",
            "collateral_amount_micro":null,"size_matched_micro":"0","avg_fill_price_micro":null,
			"status":"PARTIALLY_FILLED","failure":null,"expiration":"t","created_at":"t",
            "cancel_requested_at":null}"#,
	)
	.unwrap();
	assert_eq!(order.side, Side::Buy);
	assert_eq!(order.status, OrderStatus::PartiallyFilled);
	assert_eq!(order.price_micro.unwrap().raw(), 600_000);
	assert!(order.condition_id.is_none());
	assert!(order.avg_fill_price_micro.is_none());
	assert!(order.failure.is_none());
}

#[test]
fn legacy_order_error_becomes_safe_internal_failure() {
	let order: Order = serde_json::from_str(
		r#"{"internal_id":"id","exchange":"AGARA","token_id":"tok","condition_id":null,
            "side":"BUY","type":"LIMIT","price_micro":"600000","original_size_micro":"1000000",
            "collateral_amount_micro":null,"size_matched_micro":"0","avg_fill_price_micro":null,
            "status":"REJECTED","error":"private provider diagnostic","expiration":"t","created_at":"t",
            "cancel_requested_at":null}"#,
	)
	.unwrap();
	let failure = order.failure.unwrap();
	assert_eq!(failure.code, "internal_error");
	assert!(failure.detail.is_none());
}

#[test]
fn legacy_signed_batch_rejection_discards_message() {
	let rejected: SignedOrderResult = serde_json::from_str(
		r#"{"index":1,"outcome":"rejected","code":"DUPLICATE","message":"private"}"#,
	)
	.unwrap();
	match rejected {
		SignedOrderResult::Rejected { failure, .. } => {
			assert_eq!(failure.code, "duplicate_order");
			assert!(failure.detail.is_none());
		},
		_ => panic!("expected rejected"),
	}
}

#[test]
fn orderbook_decodes_float_levels() {
	let book: Orderbook = serde_json::from_str(
        r#"{"bids":[{"price":0.6,"size":100.0},{"price":0.59,"size":50.0}],
            "asks":[{"price":0.62,"size":80.0}],"timestamp":"t","hash":"0xabc","tick_size":"0.01"}"#,
    )
    .unwrap();
	assert_eq!(book.best_bid(), Some(0.6));
	assert_eq!(book.best_ask(), Some(0.62));
	assert_eq!(book.mid(), Some((0.6 + 0.62) / 2.0));
	assert!((book.spread().unwrap() - 0.02).abs() < 1e-9);
}

#[test]
fn fill_decodes_with_string_micros_and_nullable_role() {
	let fill: Fill = serde_json::from_str(
		r#"{"exchange":"AGARA","trade_id":"t1","fill_id":"f1","order_id":"o1","token_id":"tok",
            "side":"SELL","shares_micro":"1000000","price_micro":"550000","fee_micro":"1000",
            "role":"MAKER","status":"MATCHED","transaction_hash":null,"executed_at":"t"}"#,
	)
	.unwrap();
	assert_eq!(fill.role, Some(FillRole::Maker));
	assert_eq!(fill.fee_micro.raw(), 1000);
	assert_eq!(fill.price_micro.raw(), 550_000);
}

#[test]
fn status_decodes() {
	let s: StatusResponse = serde_json::from_str(r#"{"markets":42,"events":7}"#).unwrap();
	assert_eq!(s.markets, 42);
	assert_eq!(s.events, 7);
}

#[test]
fn unknown_response_enum_values_decode_to_unknown_not_error() {
	// A future server enum value must not fail the whole response decode.
	let order: Order = serde_json::from_str(
		r#"{"internal_id":"id","exchange":"KALSHI","token_id":"tok","condition_id":null,
            "side":"BUY","type":"SPREAD","price_micro":null,"original_size_micro":null,
            "collateral_amount_micro":null,"size_matched_micro":"0","avg_fill_price_micro":null,
			"status":"SETTLING","failure":null,"expiration":"t","created_at":"t",
            "cancel_requested_at":null}"#,
	)
	.unwrap();
	assert_eq!(order.exchange, Exchange::Unknown);
	assert_eq!(order.order_type, agara_sdk::ids::OrderType::Unknown);
	assert_eq!(order.status, OrderStatus::Unknown);
	assert!(!order.status.is_terminal());
}

#[test]
fn micro_saturates_on_overflow_and_rejects_out_of_range_numbers() {
	// A fat-finger amount saturates instead of wrapping to zero.
	let huge = Micro::from_units("10000000000000".parse().unwrap());
	assert_eq!(huge.raw(), i64::MAX);

	// A bare JSON number above i64::MAX is rejected, matching the string path.
	assert!(serde_json::from_str::<Micro>("9999999999999999999").is_err());
}

#[cfg(feature = "streaming")]
mod stream {
	use agara_sdk::Frame;
	use agara_sdk::frames::decode_frame;

	fn decode(json: &str) -> Frame {
		decode_frame(serde_json::from_str(json).unwrap())
	}

	#[test]
	fn orderbook_snapshot_uses_array_levels() {
		let frame = decode(
			r#"{"op":"update","channel":"orderbook","token_id":"tok","sequence":5,
                "data":{"kind":"snapshot","bids":[[60,100],[59,50]],"asks":[[62,80]],
                "tick_size":1,"price_scale":100,"size_scale":1}}"#,
		);
		match frame {
			Frame::OrderbookSnapshot(s) => {
				assert_eq!(s.sequence, 5);
				assert_eq!(s.bids.len(), 2);
				assert_eq!(s.bids[0].price, 60);
				assert_eq!(s.bids[0].size, 100);
				assert_eq!(s.asks[0].price, 62);
				assert_eq!(s.price_scale, 100);
			},
			other => panic!("expected snapshot, got {other:?}"),
		}
	}

	#[test]
	fn best_quote_decodes_nullable_sides() {
		let frame = decode(
			r#"{"op":"update","channel":"best_quote","token_id":"tok","sequence":9,
                "data":{"bid":{"price":60,"size":100},"ask":null,"price_scale":100,"size_scale":1}}"#,
		);
		match frame {
			Frame::BestQuote(q) => {
				assert_eq!(q.bid.unwrap().price, 60);
				assert!(q.ask.is_none());
			},
			other => panic!("expected best_quote, got {other:?}"),
		}
	}

	#[test]
	fn order_rejected_tolerates_null_hash_and_token() {
		let frame = decode(
			r#"{"op":"update","channel":"account_events","sequence":0,
                "data":{"kind":"order_rejected","order_id":"o1","order_hash":null,
				"token_id":null,"failure":{"code":"insufficient_shares",
                "title":"Insufficient shares","detail":"Available shares are lower than the requested amount.",
                "recovery":{"strategy":"none"}}}}"#,
		);
		match frame {
			Frame::OrderRejected(r) => {
				assert_eq!(r.order_id, "o1");
				assert!(r.order_hash.is_none());
				assert!(r.token_id.is_none());
				assert_eq!(r.failure.code, "insufficient_shares");
				assert_eq!(
					r.reason(),
					"Available shares are lower than the requested amount."
				);
			},
			other => panic!("expected order_rejected, got {other:?}"),
		}
	}

	#[test]
	fn error_frame_carries_action() {
		let frame = decode(
			r#"{"op":"error","failure":{"code":"slow_consumer",
                "title":"Client is reading too slowly","detail":"Reconnect after draining messages.",
                "recovery":{"strategy":"none"}},"action":"reconnect"}"#,
		);
		match frame {
			Frame::Error(e) => {
				assert_eq!(e.code, "slow_consumer");
				assert_eq!(e.action, agara_sdk::frames::WebSocketAction::Reconnect);
				assert_eq!(e.failure.unwrap().code, "slow_consumer");
			},
			other => panic!("expected error, got {other:?}"),
		}
	}

	#[test]
	fn unknown_error_action_is_inert() {
		let frame = decode(
			r#"{"op":"error","failure":{"code":"future_code","title":"Future",
                "recovery":{"strategy":"future_recovery"}},"action":"future_action"}"#,
		);
		match frame {
			Frame::Error(e) => assert!(matches!(
				e.action,
				agara_sdk::frames::WebSocketAction::Unknown(ref action) if action == "future_action"
			)),
			other => panic!("expected error, got {other:?}"),
		}
	}

	#[test]
	fn future_failure_cannot_activate_known_reconnect_action() {
		let frame = decode(
			r#"{"op":"error","failure":{"code":"future_code","title":"Future",
                "recovery":{"strategy":"none"}},"action":"reconnect"}"#,
		);
		match frame {
			Frame::Error(e) => {
				assert_eq!(e.action, agara_sdk::frames::WebSocketAction::Reconnect);
			},
			other => panic!("expected error, got {other:?}"),
		}
	}

	#[test]
	fn unknown_op_falls_through() {
		let frame = decode(r#"{"op":"brand_new_op","foo":1}"#);
		assert!(matches!(frame, Frame::Unknown(_)));
	}

	#[test]
	fn unknown_account_event_kind_returns_full_envelope() {
		let frame = decode(
			r#"{"op":"update","channel":"account_events","sequence":3,
                "data":{"kind":"brand_new_event","foo":1}}"#,
		);
		match frame {
			// The whole envelope is preserved (op/channel/sequence), not just data.
			Frame::Unknown(v) => {
				assert_eq!(v.get("op").and_then(|x| x.as_str()), Some("update"));
				assert_eq!(
					v.get("channel").and_then(|x| x.as_str()),
					Some("account_events")
				);
			},
			other => panic!("expected unknown, got {other:?}"),
		}
	}
}

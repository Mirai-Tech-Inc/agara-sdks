#![allow(missing_docs)]

use agara_sdk::{Micro, activity::Activity, models::PositionOperationResponse};
use rust_decimal::Decimal;

#[rstest::rstest]
#[case("0.0000001")]
#[case("10000000000000")]
#[case("79228162514264337593543950335")]
fn exact_micro_rejects_precision_loss_and_overflow(#[case] value: &str) {
	// Arrange
	let value: Decimal = value.parse().unwrap();
	// Act
	let result = Micro::from_units(value);
	// Assert
	core::assert!(result.is_err());
}

#[rstest::rstest]
#[case("1.1")]
#[case("1e20")]
#[case("9223372036854775808")]
fn micro_rejects_inexact_numeric_wire(#[case] value: &str) {
	// Arrange
	// Act
	let result = serde_json::from_str::<Micro>(value);
	// Assert
	core::assert!(result.is_err());
}

#[test]
fn pending_batch_and_relayer_receipts_are_distinct() {
	// Arrange
	let pending = serde_json::json!({"batch_hash":"0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","status":"PENDING","as_of":"2026-09-18T00:00:00Z"});
	let relayer = serde_json::json!({"operation":"MERGE","condition_id":"0x1111111111111111111111111111111111111111111111111111111111111111","relayer_transaction_id":"r1","transaction_hash":null,"relayer_state":"MINED","as_of":"2026-09-18T00:00:00Z"});
	// Act
	let pending: PositionOperationResponse = serde_json::from_value(pending).unwrap();
	let relayer: PositionOperationResponse = serde_json::from_value(relayer).unwrap();
	// Assert
	core::assert!(core::matches!(
		pending,
		PositionOperationResponse::PendingBatch(_)
	));
	core::assert!(core::matches!(
		relayer,
		PositionOperationResponse::Relayer(_)
	));
}

#[rstest::rstest]
#[case("ORDER", serde_json::json!({"order_id":"11111111-1111-4111-8111-111111111111","status":"PARTIALLY_FILLED","fill_status":"PARTIAL","condition_id":null,"token_id":null,"side":"SELL","filled_shares_micro":"12","average_fill_price_micro":"40","filled_amount_micro":"4","fees_micro":"0"}))]
#[case("SPLIT", serde_json::json!({"condition_id":"0x1111111111111111111111111111111111111111111111111111111111111111","shares_micro":"10","amount_micro":"10","tx_hash":"0x1"}))]
#[case("MERGE", serde_json::json!({"condition_id":"0x1111111111111111111111111111111111111111111111111111111111111111","shares_micro":"10","amount_micro":"10","tx_hash":"0x1"}))]
#[case("REDEEM", serde_json::json!({"condition_id":"0x1111111111111111111111111111111111111111111111111111111111111111","token_id":"123","shares_micro":"10","amount_micro":"10","tx_hash":"0x1"}))]
#[case("DEPOSIT", serde_json::json!({"amount_micro":"10","chain_id":8453,"token_address":"0x1","counterparty_address":"0x2","tx_hash":"0x3"}))]
#[case("WITHDRAWAL", serde_json::json!({"amount_micro":"10","chain_id":8453,"token_address":"0x1","counterparty_address":"0x2","tx_hash":"0x3"}))]
#[case("LP_PAYOUT", serde_json::json!({"amount_micro":"10","epoch_date":"2026-09-17","tx_hash":null}))]
fn all_current_activity_variants_decode(#[case] kind: &str, #[case] mut value: serde_json::Value) {
	// Arrange
	value["type"] = kind.into();
	value["exchange"] = "AGARA".into();
	value["id"] = "activity1".into();
	value["created_at"] = "2026-09-17T00:00:00Z".into();
	// Act
	let result = serde_json::from_value::<Activity>(value);
	// Assert
	core::assert!(result.is_ok(), "{kind}: {result:?}");
}

#[cfg(feature = "streaming")]
#[test]
fn all_source_derived_frames_have_named_types_and_preserve_context() {
	// Arrange
	let fixtures: Vec<serde_json::Value> =
		serde_json::from_str(core::include_str!("fixtures/stream_frames.json")).unwrap();
	// Act
	for fixture in fixtures {
		let frame = agara_sdk::frames::decode_frame(fixture["value"].clone());
		// Assert
		core::assert!(
			!core::matches!(
				frame,
				agara_sdk::Frame::Unknown(_) | agara_sdk::Frame::Malformed { .. }
			),
			"{}",
			fixture["name"]
		);
		if let agara_sdk::Frame::TokensMinted(ref frame) = frame {
			core::assert_eq!(frame.batch_index, Some(0));
			core::assert!(frame.batch_hash.is_some());
		}
		if let agara_sdk::Frame::CurrentMarketChanged(ref frame) = frame {
			core::assert_eq!(frame.market.outcomes.len(), 2);
		}
		if let agara_sdk::Frame::Error(ref frame) = frame
			&& fixture["value"].get("event_id").is_some()
		{
			core::assert!(frame.event_id.is_some());
		}
	}
}

#[cfg(feature = "streaming")]
#[test]
fn malformed_known_frame_does_not_invent_zero_cash() {
	// Arrange
	let value = serde_json::json!({"op":"update","channel":"account_events","sequence":1,"data":{"kind":"collateral_deposited","amount_micro":"1000000"}});
	// Act
	let frame = agara_sdk::frames::decode_frame(value);
	// Assert
	core::assert!(core::matches!(frame, agara_sdk::Frame::Malformed { .. }));
}

#[cfg(feature = "streaming")]
#[test]
fn fragmented_sse_preserves_mantissa_and_timestamp() {
	// Arrange
	let text = b": keep-alive\r\n\r\nid: 42\r\nretry: 1234\r\ndata: {\"parsed\":[{\"id\":\"Crypto.BTC/USD\",\"price\":{\"price\":\"6457812345678\",\"expo\":-8,\"publish_time_ms\":1780038840123}}]}\r\n\r\n";
	let mut parser = agara_sdk::prices::SseDecoder::default();
	let mut events = Vec::new();
	// Act
	for byte in text {
		events.extend(parser.push(&[*byte]).unwrap());
	}
	// Assert
	core::assert_eq!(events.len(), 1);
	core::assert_eq!(events[0].id.as_deref(), Some("42"));
	core::assert_eq!(
		events[0].frame.parsed[0].price.price.as_str(),
		"6457812345678"
	);
	core::assert_eq!(
		events[0].frame.parsed[0].price.publish_time_ms,
		1_780_038_840_123
	);
}

#[test]
fn canonical_invalid_and_future_problems_cannot_enable_retry() {
	// Arrange
	let manifest: serde_json::Value =
		serde_json::from_str(core::include_str!("fixtures/problem_manifest.json")).unwrap();
	// Act
	for fixture in manifest["fixtures"].as_array().unwrap() {
		if fixture["schema"] != "origin-problem-details.schema.json" {
			continue;
		}
		let parsed = serde_json::from_value::<agara_sdk::ProblemDetails>(fixture["value"].clone());
		// Assert
		if fixture["contract_valid"] == false
			|| fixture["value"]["code"].as_str().unwrap_or_default().starts_with("future_")
		{
			core::assert!(
				!parsed.is_ok_and(|problem| problem.is_retryable()),
				"{}",
				fixture["name"]
			);
		}
	}
}

#[rstest::rstest]
#[case("settlements_pending", 409, "retry", true)]
#[case("dependency_unavailable", 503, "retry", true)]
#[case("dependency_unavailable", 502, "retry", false)]
#[case("rate_limited", 429, "retry", false)]
#[case("future_capacity", 503, "retry", false)]
fn only_registered_problem_combinations_retry(
	#[case] code: &str,
	#[case] status: u16,
	#[case] strategy: &str,
	#[case] expected: bool,
) {
	// Arrange
	let registry: serde_json::Value =
		serde_json::from_str(core::include_str!("../src/problem/registry_snapshot.json")).unwrap();
	let row = registry["codes"].as_array().unwrap().iter().find(|row| row["code"] == code);
	let title = row.map(|row| row["title"].as_str().unwrap()).unwrap_or("Future capacity");
	let mut value = serde_json::json!({"type":format!("urn:agara:problem:{}",code.replace('_',"-")),"code":code,"title":title,"status":status,"request_id":"8051f907-4eeb-4ca9-bd84-b447ea65a68c","recovery":{"strategy":strategy}});
	if let Some(detail) = registry["public_details"][code].as_str() {
		value["detail"] = detail.into();
	}
	// Act
	let parsed = serde_json::from_value::<agara_sdk::ProblemDetails>(value);
	// Assert
	core::assert_eq!(parsed.is_ok_and(|problem| problem.is_retryable()), expected);
}

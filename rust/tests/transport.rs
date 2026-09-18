#![allow(missing_docs)]

#[path = "support/http.rs"]
mod http;

use core::time::Duration;

use agara_sdk::{
	AgaraClient, Micro, RetryPolicy,
	client::Anonymous,
	ids::{ConditionId, OrderId, Side, TokenId},
};

fn client(server: &http::Server, retry: bool) -> AgaraClient {
	AgaraClient::builder()
		.base_url(server.url.clone().into())
		.token("agt_fixture".into())
		.retry(
			RetryPolicy::builder()
				.max_retries(u32::from(retry))
				.initial_backoff(Duration::from_millis(1))
				.jitter(0.0)
				.build()
				.unwrap(),
		)
		.build()
		.unwrap()
}

fn ack() -> serde_json::Value {
	serde_json::json!({"order_id":"11111111-1111-4111-8111-111111111111","source":"AGARA","status":"PENDING","pending_operation":"SUBMIT","as_of":"2026-09-18T00:00:00Z"})
}

fn order(terminal: bool) -> serde_json::Value {
	serde_json::json!({"order":{"internal_id":"11111111-1111-4111-8111-111111111111","exchange":"AGARA","token_id":"123","condition_id":null,"side":"SELL","type":"MARKET","price_micro":null,"original_size_micro":"1000000","collateral_amount_micro":null,"size_matched_micro":"500000","avg_fill_price_micro":"400000","status":"PARTIALLY_FILLED","is_terminal":terminal,"expiration":"1970-01-01T00:00:00Z","created_at":"2026-09-18T00:00:00Z","cancel_requested_at":null},"markets":{}})
}

#[rstest::rstest]
#[case(Side::Buy)]
#[case(Side::Sell)]
#[tokio::test]
async fn market_fak_preserves_both_directions(#[case] side: Side) {
	// Arrange
	let server = http::Server::new(std::vec![http::json(202, &ack())]);
	let api = client(&server, false);
	// Act
	let result = api
		.place_market_order()
		.token_id(TokenId::new("123").unwrap())
		.side(side)
		.maybe_shares((side == Side::Sell).then_some(rust_decimal::Decimal::ONE))
		.maybe_collateral_amount((side == Side::Buy).then_some(rust_decimal::Decimal::ONE))
		.call()
		.await
		.unwrap();
	let requests = server.finish();
	let body: serde_json::Value =
		serde_json::from_str(requests[0].split("\r\n\r\n").nth(1).unwrap()).unwrap();
	// Assert
	core::assert_eq!(
		result.order_id.as_str(),
		"11111111-1111-4111-8111-111111111111"
	);
	core::assert_eq!(body["time_in_force"], "FAK");
	core::assert_eq!(
		body[if side == Side::Buy {
			"collateral_amount_micro"
		} else {
			"shares_micro"
		}],
		"1000000"
	);
}

#[tokio::test]
async fn ambiguous_mutation_transport_failure_is_never_replayed() {
	// Arrange
	let server = http::Server::new(std::vec![None, http::json(202, &ack())]);
	let api = client(&server, true);
	// Act
	let result = api
		.place_market_order()
		.token_id(TokenId::new("123").unwrap())
		.side(Side::Buy)
		.collateral_amount(rust_decimal::Decimal::ONE)
		.call()
		.await;
	// Assert
	core::assert!(result.is_err());
	core::assert_eq!(server.finish().len(), 1);
}

#[tokio::test]
async fn pending_position_batch_and_recovery_resource_survive_transport() {
	// Arrange
	let hash = std::format!("0x{}", "a".repeat(64));
	let problem = http::problem(
		"dependency_outcome_unknown",
		serde_json::json!({"strategy":"check_status","resource":{"kind":"batch","batch_hash":hash}}),
	);
	let server = http::Server::new(std::vec![http::json(502, &problem)]);
	let api = client(&server, true);
	// Act
	let error = api
		.split_position(
			ConditionId::new("0x1111111111111111111111111111111111111111111111111111111111111111")
				.unwrap(),
			Micro::new(1_000_000),
		)
		.await
		.unwrap_err();
	// Assert
	let problem = error.problem().unwrap();
	core::assert_eq!(
		problem.request_id.map(|id| id.to_string()),
		Some("8051f907-4eeb-4ca9-bd84-b447ea65a68c".to_owned())
	);
	core::assert!(
		core::matches!(&problem.recovery, agara_sdk::Recovery::CheckStatus { resource: agara_sdk::problem::RecoveryResource::Batch { batch_hash } } if batch_hash.as_str() == hash)
	);
	core::assert_eq!(server.finish().len(), 1);
}

#[tokio::test]
async fn default_all_positions_cannot_hide_unavailable_inventory() {
	// Arrange
	let body = serde_json::json!({"positions":[],"markets":{},"events":{},"as_of":"2026-09-18T00:00:00Z","unavailable_exchanges":["AGARA"]});
	let server = http::Server::new(std::vec![http::json(200, &body), http::json(200, &body)]);
	let api = client(&server, false);
	// Act
	let snapshot = api.get_positions(std::vec![], std::vec![]).await.unwrap();
	let convenience = api.list_positions(std::vec![], std::vec![]).await;
	// Assert
	core::assert_eq!(snapshot.unavailable_exchanges.len(), 1);
	core::assert!(convenience.is_err());
	core::assert_eq!(server.finish().len(), 2);
}

#[tokio::test]
async fn wait_uses_authoritative_terminality_and_bound_deadline() {
	// Arrange
	let server = http::Server::new(std::vec![
		http::json(200, &order(false)),
		http::json(200, &order(true)),
	]);
	let api = client(&server, false);
	// Act
	let order = api
		.wait_for_terminal(
			&OrderId::new("11111111-1111-4111-8111-111111111111").unwrap(),
			Duration::from_secs(1),
			Duration::from_millis(1),
		)
		.await
		.unwrap();
	// Assert
	core::assert!(order.is_terminal);
	core::assert_eq!(server.finish().len(), 2);
}

#[tokio::test]
async fn anonymous_public_client_sends_no_authentication() {
	// Arrange
	let server = http::Server::new(std::vec![http::json(
		200,
		&serde_json::json!({"markets":4,"events":2}),
	)]);
	let api =
		AgaraClient::<Anonymous>::anonymous().base_url(server.url.clone().into()).call().unwrap();
	// Act
	let status = api.get_status().await.unwrap();
	let requests = server.finish();
	// Assert
	core::assert_eq!(status.markets, 4);
	core::assert!(!requests[0].to_lowercase().contains("authorization:"));
}

#[tokio::test]
async fn retry_after_above_cap_is_returned_without_early_retry() {
	// Arrange
	let body = http::problem(
		"rate_limited",
		serde_json::json!({"strategy":"retry_after","after_seconds":120}),
	);
	let server = http::Server::new(std::vec![http::json(429, &body)]);
	let api = client(&server, true);
	// Act
	let error = api.get_status().await.unwrap_err();
	// Assert
	core::assert_eq!(error.retry_after(), Some(Duration::from_secs(120)));
	core::assert_eq!(server.finish().len(), 1);
}

#[tokio::test]
async fn invalid_domain_identifiers_never_reach_transport() {
	// Arrange
	let server = http::Server::new(std::vec![]);
	// Act
	let token = TokenId::new("%2e%2e");
	let order = OrderId::new("not-a-uuid");
	// Assert
	core::assert!(core::matches!(
		token,
		Err(agara_sdk::validation::ValidationError::Identifier(_))
	));
	core::assert!(core::matches!(
		order,
		Err(agara_sdk::validation::ValidationError::Identifier(_))
	));
	core::assert!(server.finish().is_empty());
}

#[tokio::test]
async fn order_wait_deadline_interrupts_a_slow_http_read() {
	// Arrange
	let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
	let base = std::format!("http://{}", listener.local_addr().unwrap());
	let server = std::thread::spawn(move || {
		let (_stream, _) = listener.accept().unwrap();
		std::thread::sleep(Duration::from_millis(150));
	});
	let api = AgaraClient::builder()
		.token("agt_fixture".into())
		.base_url(base.into())
		.timeout(Duration::from_secs(10))
		.build()
		.unwrap();
	let start = std::time::Instant::now();
	// Act
	let result = api
		.wait_for_terminal(
			&OrderId::new("11111111-1111-4111-8111-111111111111").unwrap(),
			Duration::from_millis(20),
			Duration::from_millis(1),
		)
		.await;
	// Assert
	core::assert!(result.is_err());
	core::assert!(start.elapsed() < Duration::from_millis(100));
	server.join().unwrap();
}

#[tokio::test]
async fn cursor_cycles_fail_without_returning_partial_orders() {
	// Arrange
	let mut page: serde_json::Value = serde_json::from_str(core::include_str!(
		"fixtures/http/get_open_orders_page.json"
	))
	.unwrap();
	page["pagination"]["next_cursor"] = "same_cursor".into();
	let server = http::Server::new(std::vec![http::json(200, &page), http::json(200, &page)]);
	let api = client(&server, false);
	// Act
	let result = api.list_open_orders(std::vec![TokenId::new("123").unwrap()], std::vec![]).await;
	let requests = server.finish();
	// Assert
	core::assert!(result.is_err());
	core::assert_eq!(requests.len(), 2);
	for request in requests {
		let body: serde_json::Value =
			serde_json::from_str(request.split("\r\n\r\n").nth(1).unwrap()).unwrap();
		core::assert_eq!(body["token_ids"], serde_json::json!(["123"]));
	}
}

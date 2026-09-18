#![allow(missing_docs)]

#[path = "support/http.rs"]
mod http;

use core::time::Duration;

use agara_sdk::{
	AgaraClient, AgaraError, RetryPolicy,
	error::{PaginationError, PollError, ResponseError},
	ids::OrderId,
};

fn client(server: &http::Server) -> AgaraClient {
	AgaraClient::builder()
		.token("fixture".into())
		.base_url(server.url.clone().into())
		.retry(
			RetryPolicy::builder()
				.max_retries(1)
				.initial_backoff(Duration::from_millis(1))
				.jitter(0.0)
				.build()
				.unwrap(),
		)
		.build()
		.unwrap()
}

#[rstest::rstest]
#[case("", "application/json", "empty")]
#[case("{", "application/json", "json")]
#[case("<html>gateway</html>", "text/html", "media")]
#[tokio::test]
async fn response_decoding_errors_keep_distinct_categories_and_never_retry(
	#[case] body: &str,
	#[case] media: &str,
	#[case] expected: &str,
) {
	// Arrange
	let response = Some(std::format!(
		"HTTP/1.1 200 OK\r\nContent-Type: {media}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
		body.len()
	));
	let server = http::Server::new(std::vec![response]);
	let api = client(&server);
	// Act
	let error = api.get_status().await.unwrap_err();
	// Assert
	match (expected, &error) {
		("empty", AgaraError::Response(ResponseError::EmptyBody)) => {},
		("json", AgaraError::Response(ResponseError::Json { source })) => {
			core::assert!(source.is_eof())
		},
		("media", AgaraError::Response(ResponseError::UnexpectedContentType { content_type })) => {
			core::assert_eq!(content_type.as_deref(), Some("text/html"))
		},
		_ => core::panic!("unexpected classification: {error:?}"),
	}
	core::assert!(!error.is_retryable());
	core::assert_eq!(server.finish().len(), 1);
}

#[tokio::test]
async fn duplicate_problem_code_is_rejected_before_classification_or_retry() {
	// Arrange
	let body = http::problem(
		"dependency_unavailable",
		serde_json::json!({"strategy":"retry"}),
	);
	let body = body.to_string();
	let duplicate = std::format!("{{\"code\":\"internal_error\",{}", &body[1..]);
	let response = Some(std::format!(
		"HTTP/1.1 503 Unavailable\r\nContent-Type: application/problem+json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{duplicate}",
		duplicate.len()
	));
	let server = http::Server::new(std::vec![response]);
	let api = client(&server);
	// Act
	let error = api.get_status().await.unwrap_err();
	// Assert
	core::assert!(core::matches!(
		error,
		AgaraError::Response(ResponseError::Problem { status: 503, .. })
	));
	core::assert!(!error.is_retryable());
	core::assert_eq!(server.finish().len(), 1);
}

#[tokio::test]
async fn real_http_status_cannot_be_overridden_by_problem_json() {
	// Arrange
	let body = http::problem(
		"dependency_unavailable",
		serde_json::json!({"strategy":"retry"}),
	);
	let server = http::Server::new(std::vec![http::json(502, &body)]);
	let api = client(&server);
	// Act
	let error = api.get_status().await.unwrap_err();
	// Assert
	core::assert_eq!(error.status_code(), Some(502));
	core::assert!(core::matches!(
		error,
		AgaraError::Response(ResponseError::Problem { status: 502, .. })
	));
	core::assert!(!error.is_retryable());
	core::assert_eq!(server.finish().len(), 1);
}

#[tokio::test]
async fn missing_completeness_is_a_decode_error_not_a_synthetic_server_error() {
	// Arrange
	let body =
		serde_json::json!({"positions":[],"markets":{},"events":{},"as_of":"2026-09-18T00:00:00Z"});
	let server = http::Server::new(std::vec![http::json(200, &body)]);
	let api = client(&server);
	// Act
	let error = api.list_positions(std::vec![], std::vec![]).await.unwrap_err();
	// Assert
	core::assert!(core::matches!(
		error,
		AgaraError::Response(ResponseError::Json { .. })
	));
	core::assert_eq!(error.status_code(), None);
	core::assert_eq!(server.finish().len(), 1);
}

#[tokio::test]
async fn explicit_partial_inventory_is_matchable_without_parsing_text() {
	// Arrange
	let body = serde_json::json!({"positions":[],"markets":{},"events":{},"as_of":"2026-09-18T00:00:00Z","unavailable_exchanges":["AGARA"]});
	let server = http::Server::new(std::vec![http::json(200, &body)]);
	let api = client(&server);
	// Act
	let error = api.list_positions(std::vec![], std::vec![]).await.unwrap_err();
	// Assert
	core::assert!(
		core::matches!(&error, AgaraError::Response(ResponseError::PartialData { exchanges }) if exchanges == &[agara_sdk::ids::Exchange::Agara])
	);
	core::assert_eq!(error.status_code(), None);
	core::assert_eq!(server.finish().len(), 1);
}

#[tokio::test]
async fn cursor_cycles_return_a_pagination_error_without_partial_success() {
	// Arrange
	let body = serde_json::json!({"orders":[],"markets":{},"events":{},"as_of":"2026-09-18T00:00:00Z","pagination":{"next_cursor":"same","limit":500}});
	let server = http::Server::new(std::vec![http::json(200, &body), http::json(200, &body)]);
	let api = client(&server);
	// Act
	let error = api.list_open_orders(std::vec![], std::vec![]).await.unwrap_err();
	// Assert
	core::assert!(core::matches!(
		error,
		AgaraError::Pagination(PaginationError::CursorCycle)
	));
	core::assert_eq!(server.finish().len(), 2);
}

#[tokio::test]
async fn expired_wait_never_issues_an_extra_order_read() {
	// Arrange
	let body: serde_json::Value =
		serde_json::from_str(core::include_str!("fixtures/http/get_order.json")).unwrap();
	let server = http::Server::new(std::vec![http::json(200, &body)]);
	let api = client(&server);
	let order = OrderId::new("11111111-1111-4111-8111-111111111111").unwrap();
	// Act
	let error = api
		.wait_for_terminal(&order, Duration::from_millis(50), Duration::from_secs(1))
		.await
		.unwrap_err();
	// Assert
	core::assert!(core::matches!(
		error,
		AgaraError::Poll(PollError::DeadlineElapsed)
	));
	core::assert_eq!(server.finish().len(), 1);
}

#[rstest::rstest]
#[case("0", 120)]
#[case("999999999999999999999999999999", 0)]
#[case("Fri, 18 Sep 2099 12:00:00 GMT", 0)]
#[case("NaN", 0)]
#[case("-1", 0)]
#[tokio::test]
async fn header_hints_cannot_shorten_body_delays_or_enable_unsafe_fallback(
	#[case] header: &str,
	#[case] seconds: u32,
) {
	// Arrange
	let body = http::problem(
		"rate_limited",
		serde_json::json!({"strategy":"retry_after","after_seconds":seconds}),
	)
	.to_string();
	let response = Some(std::format!(
		"HTTP/1.1 429 Rate Limited\r\nRetry-After: {header}\r\nContent-Type: application/problem+json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
		body.len()
	));
	let server = http::Server::new(std::vec![response]);
	let api = client(&server);
	// Act
	let error = api.get_status().await.unwrap_err();
	// Assert
	core::assert!(core::matches!(error, AgaraError::RateLimited { .. }));
	if seconds == 120 {
		core::assert_eq!(error.retry_after(), Some(Duration::from_secs(120)));
	} else {
		core::assert!(!error.is_retryable());
		core::assert!(core::matches!(
			error.http_failure().unwrap().retry_hint(),
			Some(agara_sdk::error::RetryAfter::Unusable { .. })
		));
	}
	core::assert_eq!(server.finish().len(), 1);
}

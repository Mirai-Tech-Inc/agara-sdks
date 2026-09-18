#![allow(missing_docs)]

#[path = "support/http.rs"]
mod http;

use core::time::Duration;

use agara_sdk::{
	AgaraClient, AgaraError, Micro, RetryPolicy, bridge, catalogue,
	ids::{BatchHash, ConditionId, Exchange, OrderId, TokenId},
	models::{OpenOrdersListRequest, OrderResponse, PositionsResponse, TradesResponse},
	retry::{Jitter, RetryLimit},
	validation::{
		ConfigurationError, Field, IdentifierReason, QueryReason, RetryPolicyError, ValidationError,
	},
	values::{CalendarDate, Timestamp},
};

const UUID: &str = "11111111-1111-4111-8111-111111111111";
const HASH: &str = "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const U256_MAX: &str =
	"115792089237316195423570985008687907853269984665640564039457584007913129639935";

fn client(server: &http::Server) -> AgaraClient {
	AgaraClient::builder()
		.token("fixture".into())
		.base_url(server.url.clone().into())
		.build()
		.unwrap()
}

#[rstest::rstest]
#[case("")]
#[case("-1")]
#[case("+1")]
#[case("1.0")]
#[case("0x01")]
#[case("123\n")]
#[case("１２３")]
#[case("115792089237316195423570985008687907853269984665640564039457584007913129639936")]
fn token_constructor_and_deserializer_enforce_same_u256_domain(#[case] value: &str) {
	// Arrange
	let wire = serde_json::to_string(value).unwrap();
	// Act
	let direct = TokenId::new(value);
	let decoded = serde_json::from_str::<TokenId>(&wire);
	// Assert
	core::assert!(
		core::matches!(direct, Err(ValidationError::Identifier(error)) if error.field == Field::TokenId && core::matches!(error.reason, IdentifierReason::InvalidTokenId))
	);
	core::assert!(decoded.is_err());
}

#[test]
fn valid_identifier_boundaries_are_canonical_without_precision_loss() {
	// Arrange
	let uppercase = HASH.to_ascii_uppercase().replacen("0X", "0x", 1);
	// Act
	let max = TokenId::new(U256_MAX).unwrap();
	let zero = TokenId::new("000").unwrap();
	let batch = BatchHash::new(uppercase).unwrap();
	let order = OrderId::new(UUID.replace('-', "")).unwrap();
	// Assert
	core::assert_eq!(max.as_str(), U256_MAX);
	core::assert_eq!(zero.as_str(), "0");
	core::assert_eq!(batch.as_str(), HASH);
	core::assert_eq!(order.as_str(), UUID);
}

#[rstest::rstest]
#[case("not-a-uuid")]
#[case("11111111-1111-4111-8111-11111111111g")]
#[case("../orders")]
fn uuid_identifiers_reject_invalid_wire_values(#[case] value: &str) {
	// Arrange
	let wire = serde_json::json!(value);
	// Act
	let result = serde_json::from_value::<OrderId>(wire);
	// Assert
	core::assert!(result.is_err());
}

#[rstest::rstest]
#[case("0x1")]
#[case("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
#[case("0xgggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggg")]
fn hashes_require_exact_hex_width(#[case] value: &str) {
	// Arrange
	let wire = serde_json::json!(value);
	// Act
	let result = serde_json::from_value::<ConditionId>(wire);
	// Assert
	core::assert!(result.is_err());
}

#[rstest::rstest]
#[case("2026-02-29")]
#[case("2026-04-31")]
#[case("2026-9-18")]
#[case("2026-09-18T00:00:00Z")]
fn calendar_dates_are_real_and_canonical(#[case] value: &str) {
	// Arrange
	let wire = serde_json::json!(value);
	// Act
	let result = serde_json::from_value::<CalendarDate>(wire);
	// Assert
	core::assert!(result.is_err());
	core::assert!(CalendarDate::new("2024-02-29").is_ok());
}

#[test]
fn timestamps_and_range_order_compare_instants_instead_of_text() {
	// Arrange
	let query = catalogue::PnlHistoryQuery {
		from: "2026-09-18T02:00:00+02:00".into(),
		to: "2026-09-18T00:00:00Z".into(),
		limit: Some(1),
	};
	// Act
	let valid = query.validate();
	let invalid = Timestamp::new("2026-09-18T00:00:00");
	// Assert
	core::assert!(valid.is_ok());
	core::assert!(
		core::matches!(invalid, Err(ValidationError::Query(error)) if core::matches!(error.reason, QueryReason::InvalidTimestamp { .. }))
	);
}

#[rstest::rstest]
#[case("ftp://example.test")]
#[case("https://user:password@example.test")]
#[case("https://example.test?token=value")]
#[case("https://example.test#fragment")]
#[case("not a URL")]
fn client_rejects_ambiguous_service_configuration(#[case] url: &str) {
	// Arrange
	// Act
	let result =
		AgaraClient::builder().token("fixture".into()).base_url(url.to_owned().into()).build();
	// Assert
	core::assert!(core::matches!(
		result,
		Err(AgaraError::Validation(ValidationError::Configuration(_)))
	));
}

#[rstest::rstest]
#[case(Duration::ZERO)]
#[case(Duration::MAX)]
fn invalid_timeout_is_a_typed_configuration_error(#[case] timeout: Duration) {
	// Arrange
	// Act
	let result = AgaraClient::builder().token("fixture".into()).timeout(timeout).build();
	// Assert
	core::assert!(core::matches!(
		result,
		Err(AgaraError::Validation(ValidationError::Configuration(
			ConfigurationError::InvalidTimeout { field: Field::Timeout }
		)))
	));
}

#[rstest::rstest]
#[case(f64::NAN)]
#[case(f64::INFINITY)]
#[case(f64::NEG_INFINITY)]
#[case(-0.001)]
#[case(1.001)]
fn invalid_retry_jitter_cannot_be_constructed(#[case] value: f64) {
	// Arrange
	// Act
	let jitter = Jitter::new(value);
	let policy = RetryPolicy::builder().jitter(value).build();
	// Assert
	core::assert!(core::matches!(jitter, Err(RetryPolicyError::InvalidJitter)));
	core::assert!(core::matches!(
		policy,
		Err(ValidationError::RetryPolicy(
			RetryPolicyError::InvalidJitter
		))
	));
}

#[test]
fn retry_limits_and_backoff_relationships_are_bounded() {
	// Arrange
	// Act
	let retries = RetryLimit::new(u32::MAX);
	let zero = RetryPolicy::builder().initial_backoff(Duration::ZERO).build();
	let reversed = RetryPolicy::builder()
		.initial_backoff(Duration::from_secs(2))
		.max_backoff(Duration::from_secs(1))
		.build();
	// Assert
	core::assert!(core::matches!(
		retries,
		Err(RetryPolicyError::ExcessiveAttempts)
	));
	core::assert!(core::matches!(
		zero,
		Err(ValidationError::RetryPolicy(
			RetryPolicyError::ZeroInitialBackoff
		))
	));
	core::assert!(core::matches!(
		reversed,
		Err(ValidationError::RetryPolicy(
			RetryPolicyError::MaxBelowInitial
		))
	));
}

#[tokio::test]
async fn mutated_query_dtos_and_invalid_operations_fail_before_any_network_request() {
	// Arrange
	let server = http::Server::new(std::vec![]);
	let api = client(&server);
	let mut markets = catalogue::MarketsQuery { source: "agara".into(), ..Default::default() };
	markets.validate().unwrap();
	markets.limit = Some(257);
	let mut orders = OpenOrdersListRequest {
		token_ids: std::vec![],
		exchanges: std::vec![],
		limit: 1,
		cursor: None,
	};
	orders.exchanges.push(Exchange::Unknown);
	// Act
	let results = [
		api.list_markets(&markets).await.map(|_| ()),
		api.get_open_orders_page(&orders).await.map(|_| ()),
		api.get_trading_day("XNYS", "2026-02-29").await.map(|_| ()),
		api.get_batch("0x123").await.map(|_| ()),
		api.get_batch_group("not-a-uuid").await.map(|_| ()),
		api.get_market("../x").await.map(|_| ()),
		api.get_event("..").await.map(|_| ()),
		api.get_positions(std::vec![], std::vec![Exchange::Unknown]).await.map(|_| ()),
		api.list_trades(0, None).await.map(|_| ()),
		api.merge_position(ConditionId::new(HASH).unwrap(), Micro::new(-1)).await.map(|_| ()),
		api.split_position(ConditionId::new(HASH).unwrap(), Micro::new(0)).await.map(|_| ()),
		api.get_deposit_quote(&bridge::PortfolioBridgeDepositQuoteRequest {
			from_chain_id: "solana".into(),
			from_token_address: "mint".into(),
			from_amount_base_unit: "01".into(),
		})
		.await
		.map(|_| ()),
	];
	// Assert
	for result in results {
		core::assert!(
			core::matches!(result, Err(AgaraError::Validation(_))),
			"{result:?}"
		);
	}
	core::assert!(server.finish().is_empty());
}

#[rstest::rstest]
#[case("positions")]
#[case("trades")]
fn missing_availability_never_decodes_as_complete_portfolio(#[case] kind: &str) {
	// Arrange
	let fixture = if kind == "positions" {
		core::include_str!("fixtures/http/get_positions.json")
	} else {
		core::include_str!("fixtures/http/list_trades.json")
	};
	let mut body: serde_json::Value = serde_json::from_str(fixture).unwrap();
	body.as_object_mut().unwrap().remove("unavailable_exchanges");
	// Act
	let result = if kind == "positions" {
		serde_json::from_value::<PositionsResponse>(body).map(|_| ())
	} else {
		serde_json::from_value::<TradesResponse>(body).map(|_| ())
	};
	// Assert
	core::assert!(result.is_err());
}

#[rstest::rstest]
#[case("expiration", serde_json::json!("0"))]
#[case("created_at", serde_json::json!("yesterday"))]
#[case("is_terminal", serde_json::json!("true"))]
fn malformed_order_state_cannot_decode(#[case] field: &str, #[case] value: serde_json::Value) {
	// Arrange
	let mut body: serde_json::Value =
		serde_json::from_str(core::include_str!("fixtures/http/get_order.json")).unwrap();
	body["order"][field] = value;
	// Act
	let result = serde_json::from_value::<OrderResponse>(body);
	// Assert
	core::assert!(result.is_err());
}

#[tokio::test]
async fn local_failures_retain_machine_matchable_categories() {
	// Arrange
	let server = http::Server::new(std::vec![]);
	let api = client(&server);
	// Act
	let result = api
		.wait_for_terminal(
			&OrderId::new(UUID).unwrap(),
			Duration::ZERO,
			Duration::from_millis(1),
		)
		.await;
	// Assert
	core::assert!(core::matches!(
		result,
		Err(AgaraError::Validation(ValidationError::Configuration(
			ConfigurationError::InvalidTimeout { field: Field::PollTimeout }
		)))
	));
	core::assert!(server.finish().is_empty());
}

#[rstest::rstest]
#[case(
	agara_sdk::pnl::RealizedPnlBucketGranularityDto::Week,
	agara_sdk::pnl::RealizedPnlWindowDto::SevenDays
)]
#[case(
	agara_sdk::pnl::RealizedPnlBucketGranularityDto::Day,
	agara_sdk::pnl::RealizedPnlWindowDto::All
)]
#[tokio::test]
async fn incompatible_realized_pnl_windows_fail_before_io(
	#[case] granularity: agara_sdk::pnl::RealizedPnlBucketGranularityDto,
	#[case] window: agara_sdk::pnl::RealizedPnlWindowDto,
) {
	// Arrange
	let server = http::Server::new(std::vec![]);
	let api = client(&server);
	let query = agara_sdk::pnl::RealizedPnlReportQueryDto { granularity, window };
	// Act
	let result = api.get_realized_pnl(&query).await;
	// Assert
	core::assert!(
		core::matches!(result, Err(AgaraError::Validation(ValidationError::Query(error))) if error.field == Field::Granularity && core::matches!(error.reason, QueryReason::IncompatibleFields { other: Field::Window }))
	);
	core::assert!(server.finish().is_empty());
}

#[tokio::test]
async fn redirect_cannot_transparently_replay_a_mutation() {
	// Arrange
	let destination = http::Server::new(std::vec![]);
	let redirect = Some(std::format!(
		"HTTP/1.1 307 Temporary Redirect\r\nLocation: {}/replayed\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
		destination.url
	));
	let server = http::Server::new(std::vec![redirect]);
	let api = client(&server);
	// Act
	let result = api.split_position(ConditionId::new(HASH).unwrap(), Micro::new(1_000_000)).await;
	// Assert
	core::assert!(core::matches!(
		result,
		Err(AgaraError::UnexpectedStatus { status: 307, .. })
	));
	core::assert_eq!(server.finish().len(), 1);
	core::assert!(destination.finish().is_empty());
}

#[test]
fn search_bounds_match_server_utf16_length() {
	// Arrange
	let query = catalogue::SearchQuery { q: "😀".repeat(65), ..Default::default() };
	// Act
	let result = query.validate();
	// Assert
	core::assert!(
		core::matches!(result, Err(ValidationError::Query(error)) if error.field == Field::Query && core::matches!(error.reason, QueryReason::OutOfRange { max: 128, actual: 130, .. }))
	);
}

#[tokio::test]
async fn unsigned_gtd_leaves_venue_specific_lifetime_policy_to_server() {
	// Arrange
	let body: serde_json::Value =
		serde_json::from_str(core::include_str!("fixtures/http/place_limit_order.json")).unwrap();
	let server = http::Server::new(std::vec![http::json(202, &body)]);
	let api = client(&server);
	// Act
	let result = api
		.place_limit_order()
		.token_id(TokenId::new("123").unwrap())
		.side(agara_sdk::ids::Side::Buy)
		.price("0.5".parse().unwrap())
		.shares("1".parse().unwrap())
		.time_in_force(agara_sdk::ids::TimeInForce::Gtd)
		.expiration_unix_seconds(1)
		.call()
		.await;
	// Assert
	core::assert!(result.is_ok(), "{result:?}");
	core::assert_eq!(server.finish().len(), 1);
}

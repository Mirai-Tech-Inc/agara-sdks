#![allow(missing_docs)]
#[path = "support/http.rs"]
mod http;
use agara_sdk::{
	AgaraClient, Micro, batches, bridge, catalogue,
	ids::{ConditionId, OrderHash, OrderId, OrderType, Side, TimeInForce, TokenId},
	incentives, models, pnl,
};
const HASH: &str = "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
fn signature() -> String {
	std::format!("0x{:064x}{:064x}1b", 1, 1)
}
fn future_deadline() -> u64 {
	std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() + 3600
}
fn signed() -> models::SignedOrderRequest {
	models::SignedOrderRequest {
		token_id: TokenId::new("123").unwrap(),
		side: Side::Buy,
		order_type: OrderType::Limit,
		time_in_force: TimeInForce::Gtc,
		price_micro: Micro::new(500000),
		shares_micro: Micro::new(1000000),
		post_only: false,
		expiration_unix_seconds: None,
		order_hash: OrderHash::new(HASH).unwrap(),
		signature: signature(),
		salt: "1".into(),
		maker: "0x1111111111111111111111111111111111111111".into(),
		chain_token_id: "123".into(),
		maker_amount: "500000".into(),
		taker_amount: "1000000".into(),
		side_u8: 0,
		timestamp: "0".into(),
		metadata: HASH.into(),
		builder: HASH.into(),
	}
}
fn batch_request() -> batches::AccountBatchSubmission {
	batches::AccountBatchSubmission {
		ops: std::vec![batches::BatchOpDto::Split {
			market_id: "11111111-1111-4111-8111-111111111111".into(),
			condition_id: "0x1111111111111111111111111111111111111111111111111111111111111111"
				.into(),
			shares_micro: 1000000,
		}],
		seq: 4,
		deadline_unix_seconds: future_deadline(),
		signature: signature(),
		heals_batch_hash: None,
	}
}
#[rstest::rstest]
#[case("get_status")]
#[case("place_limit_order")]
#[case("place_signed_order")]
#[case("place_signed_orders")]
#[case("list_orders")]
#[case("get_order")]
#[case("cancel_order")]
#[case("get_order_by_hash")]
#[case("get_order_trades")]
#[case("cancel_all_orders")]
#[case("submit_batch")]
#[case("get_batch")]
#[case("supersede_batch")]
#[case("get_batch_group")]
#[case("get_orderbook")]
#[case("list_activities")]
#[case("get_deposit_address")]
#[case("get_deposit_assets")]
#[case("get_deposit_quote")]
#[case("get_withdraw_assets")]
#[case("get_withdraw_quote")]
#[case("get_open_orders_page")]
#[case("get_positions")]
#[case("split_position")]
#[case("merge_position")]
#[case("get_portfolio_summary")]
#[case("list_trades")]
#[case("get_rebates")]
#[case("list_lp_incentives")]
#[case("get_lp_categories")]
#[case("get_closed_lp_categories")]
#[case("get_closed_lp_incentives")]
#[case("get_lp_earnings")]
#[case("get_pnl")]
#[case("get_pnl_history")]
#[case("get_realized_pnl")]
#[case("get_trading_day")]
#[case("get_trading_days")]
#[case("get_next_session")]
#[case("list_calendars")]
#[case("get_calendar")]
#[case("get_category")]
#[case("get_canonical_event_url")]
#[case("get_event")]
#[case("list_events")]
#[case("get_market")]
#[case("list_markets")]
#[case("get_price_point")]
#[case("get_price_ticks")]
#[case("get_token_history")]
#[case("search")]
#[case("list_securities")]
#[case("get_security")]
#[tokio::test]
async fn all_rest_endpoints_match_contract(#[case] name: &str) {
	// Arrange
	let coverage: serde_json::Value =
		serde_json::from_str(core::include_str!("../coverage.json")).unwrap();
	let operation = coverage["operations"]
		.as_array()
		.unwrap()
		.iter()
		.find(|op| op["public_method"] == name)
		.unwrap();
	let fixture = std::fs::read_to_string(std::format!(
		"{}/tests/fixtures/http/{name}.json",
		core::env!("CARGO_MANIFEST_DIR")
	))
	.unwrap();
	let body: serde_json::Value = serde_json::from_str(&fixture).unwrap();
	let server = http::Server::new(std::vec![http::json(200, &body)]);
	let api = AgaraClient::builder()
		.base_url(server.url.clone().into())
		.token("agt_fixture".into())
		.build()
		.unwrap();
	// Act
	match name {
		"get_status" => {
			api.get_status().await.unwrap();
		},
		"place_limit_order" => {
			api.place_limit_order()
				.token_id(TokenId::new("123").unwrap())
				.side(Side::Buy)
				.price("0.5".parse().unwrap())
				.shares("1".parse().unwrap())
				.call()
				.await
				.unwrap();
		},
		"place_signed_order" => {
			api.place_signed_order(&signed()).await.unwrap();
		},
		"place_signed_orders" => {
			api.place_signed_orders(std::vec![signed(), signed()]).await.unwrap();
		},
		"list_orders" => {
			api.list_orders(50, None).await.unwrap();
		},
		"get_order" => {
			api.get_order(&OrderId::new("11111111-1111-4111-8111-111111111111").unwrap())
				.await
				.unwrap();
		},
		"cancel_order" => {
			api.cancel_order(&OrderId::new("11111111-1111-4111-8111-111111111111").unwrap())
				.await
				.unwrap();
		},
		"get_order_by_hash" => {
			api.get_order_by_hash(&OrderHash::new(HASH).unwrap()).await.unwrap();
		},
		"get_order_trades" => {
			api.get_order_trades(&OrderId::new("11111111-1111-4111-8111-111111111111").unwrap())
				.await
				.unwrap();
		},
		"cancel_all_orders" => {
			api.cancel_all_orders().await.unwrap();
		},
		"submit_batch" => {
			api.submit_batch(&batch_request()).await.unwrap();
		},
		"get_batch" => {
			api.get_batch(HASH).await.unwrap();
		},
		"supersede_batch" => {
			api.supersede_batch(
				HASH,
				&batches::AccountBatchSupersedeSubmission {
					ops: batch_request().ops,
					deadline_unix_seconds: future_deadline(),
					signature: signature(),
				},
			)
			.await
			.unwrap();
		},
		"get_batch_group" => {
			api.get_batch_group("11111111-1111-4111-8111-111111111111").await.unwrap();
		},
		"get_orderbook" => {
			api.get_orderbook(&TokenId::new("123").unwrap()).await.unwrap();
		},
		"list_activities" => {
			api.list_activities(50, None).await.unwrap();
		},
		"get_deposit_address" => {
			api.get_deposit_address().await.unwrap();
		},
		"get_deposit_assets" => {
			api.get_deposit_assets().await.unwrap();
		},
		"get_deposit_quote" => {
			api.get_deposit_quote(&bridge::PortfolioBridgeDepositQuoteRequest {
				from_chain_id: "8453".into(),
				from_token_address: "0x2".into(),
				from_amount_base_unit: "1000000".into(),
			})
			.await
			.unwrap();
		},
		"get_withdraw_assets" => {
			api.get_withdraw_assets().await.unwrap();
		},
		"get_withdraw_quote" => {
			api.get_withdraw_quote(&bridge::PortfolioBridgeWithdrawQuoteRequest {
				to_chain_id: "8453".into(),
				to_token_address: "0x2".into(),
				recipient_address: "0x3".into(),
				from_amount_base_unit: "1000000".into(),
			})
			.await
			.unwrap();
		},
		"get_open_orders_page" => {
			api.get_open_orders_page(&models::OpenOrdersListRequest {
				token_ids: std::vec![],
				exchanges: std::vec![],
				limit: 50,
				cursor: None,
			})
			.await
			.unwrap();
		},
		"get_positions" => {
			api.get_positions(std::vec![], std::vec![]).await.unwrap();
		},
		"split_position" => {
			api.split_position(
				ConditionId::new(
					"0x1111111111111111111111111111111111111111111111111111111111111111",
				)
				.unwrap(),
				Micro::new(1000000),
			)
			.await
			.unwrap();
		},
		"merge_position" => {
			api.merge_position(
				ConditionId::new(
					"0x1111111111111111111111111111111111111111111111111111111111111111",
				)
				.unwrap(),
				Micro::new(1000000),
			)
			.await
			.unwrap();
		},
		"get_portfolio_summary" => {
			api.get_portfolio_summary(&[]).await.unwrap();
		},
		"list_trades" => {
			api.list_trades(50, None).await.unwrap();
		},
		"get_rebates" => {
			api.get_rebates().await.unwrap();
		},
		"list_lp_incentives" => {
			api.list_lp_incentives(&incentives::LpIncentivesParams {
				category: Some("crypto".into()),
				search: Some("BTC USD".into()),
				sort_by: Some(incentives::LpIncentiveSortBy::MinShares),
				sort_order: Some(incentives::LpIncentiveSortOrder::Asc),
			})
			.await
			.unwrap();
		},
		"get_lp_categories" => {
			api.get_lp_categories().await.unwrap();
		},
		"get_closed_lp_categories" => {
			api.get_closed_lp_categories().await.unwrap();
		},
		"get_closed_lp_incentives" => {
			api.get_closed_lp_incentives(&incentives::ClosedLpIncentivesParams {
				limit: Some(20),
				page: Some(1),
				category: None,
				search: None,
				sort_by: None,
				sort_order: None,
			})
			.await
			.unwrap();
		},
		"get_lp_earnings" => {
			api.get_lp_earnings().await.unwrap();
		},
		"get_pnl" => {
			api.get_pnl().await.unwrap();
		},
		"get_pnl_history" => {
			api.get_pnl_history(&catalogue::PnlHistoryQuery {
				from: "2026-09-01T00:00:00Z".into(),
				to: "2026-09-17T00:00:00Z".into(),
				limit: Some(10),
			})
			.await
			.unwrap();
		},
		"get_realized_pnl" => {
			api.get_realized_pnl(&pnl::RealizedPnlReportQueryDto {
				granularity: pnl::RealizedPnlBucketGranularityDto::Day,
				window: pnl::RealizedPnlWindowDto::SevenDays,
			})
			.await
			.unwrap();
		},
		"get_trading_day" => {
			api.get_trading_day("XNAS", "2026-09-17").await.unwrap();
		},
		"get_trading_days" => {
			api.get_trading_days(
				"XNAS",
				&catalogue::CalendarRangeQuery {
					from: "2026-09-17".into(),
					to: "2026-09-17".into(),
				},
			)
			.await
			.unwrap();
		},
		"get_next_session" => {
			api.get_next_session(
				"XNAS",
				&catalogue::NextSessionQuery { from: "2026-09-17".into() },
			)
			.await
			.unwrap();
		},
		"list_calendars" => {
			api.list_calendars().await.unwrap();
		},
		"get_calendar" => {
			api.get_calendar("XNAS").await.unwrap();
		},
		"get_category" => {
			api.get_category("crypto").await.unwrap();
		},
		"get_canonical_event_url" => {
			api.get_canonical_event_url("event").await.unwrap();
		},
		"get_event" => {
			api.get_event("event").await.unwrap();
		},
		"list_events" => {
			api.list_events(&catalogue::EventsQuery {
				category: Some("crypto".into()),
				include_markets: Some(true),
				..Default::default()
			})
			.await
			.unwrap();
		},
		"get_market" => {
			api.get_market("11111111-1111-4111-8111-111111111111").await.unwrap();
		},
		"list_markets" => {
			api.list_markets(&catalogue::MarketsQuery {
				source: "agara".into(),
				state: Some("ACTIVE".into()),
				..Default::default()
			})
			.await
			.unwrap();
		},
		"get_price_point" => {
			api.get_price_point(&catalogue::PricePointQuery {
				symbol: "BTC-USD".into(),
				provider: "pyth-pro".into(),
				at: 1789632000,
			})
			.await
			.unwrap();
		},
		"get_price_ticks" => {
			api.get_price_ticks(&catalogue::PriceTicksQuery {
				symbol: "BTC-USD".into(),
				provider: "pyth-pro".into(),
				from: 1789632000,
				to: 1789632060,
			})
			.await
			.unwrap();
		},
		"get_token_history" => {
			api.get_token_history(&catalogue::TokenHistoryQuery {
				token_id: "123".into(),
				range: Some("1d".into()),
				points: Some(20),
			})
			.await
			.unwrap();
		},
		"search" => {
			api.search(&catalogue::SearchQuery {
				q: "BTC USD".into(),
				limit: Some(5),
				source: Some("agara".into()),
			})
			.await
			.unwrap();
		},
		"list_securities" => {
			api.list_securities().await.unwrap();
		},
		"get_security" => {
			api.get_security("BTC-USD").await.unwrap();
		},
		_ => core::panic!("unmapped endpoint"),
	}
	let requests = server.finish();
	// Assert
	core::assert_eq!(requests.len(), 1);
	let request = &requests[0];
	core::assert!(
		request.starts_with(&std::format!("{} ", operation["method"].as_str().unwrap())),
		"{name}: {request:?}"
	);
	let mut expected = operation["path"].as_str().unwrap().to_owned();
	for (key, value) in [
		("order_id", "11111111-1111-4111-8111-111111111111"),
		("order_hash", HASH),
		("batch_hash", HASH),
		("group_id", "11111111-1111-4111-8111-111111111111"),
		("token_id", "123"),
		("mic", "XNAS"),
		("date", "2026-09-17"),
		("id", "11111111-1111-4111-8111-111111111111"),
		("symbol", "BTC-USD"),
	] {
		expected = expected.replace(&std::format!("{{{key}}}"), value);
	}
	expected = expected.replace(
		"{slug}",
		if name == "get_category" {
			"crypto"
		} else {
			"event"
		},
	);
	core::assert_eq!(
		request
			.lines()
			.next()
			.unwrap()
			.split_whitespace()
			.nth(1)
			.unwrap()
			.split('?')
			.next()
			.unwrap(),
		expected
	);
	core::assert!(request.to_lowercase().contains("authorization: bearer agt_fixture"));
	if core::matches!(name, "place_limit_order" | "place_signed_order") {
		let sent: serde_json::Value =
			serde_json::from_str(request.split("\r\n\r\n").nth(1).unwrap()).unwrap();
		core::assert_eq!(sent["shares_micro"], "1000000");
		core::assert_eq!(sent["price_micro"], "500000");
		core::assert!(sent.get("collateral_amount_micro").is_none());
	}
	if name == "list_lp_incentives" {
		core::assert!(request.contains("sort_by=min_shares"));
		core::assert!(request.contains("search=BTC+USD"));
	}
	if name == "get_realized_pnl" {
		core::assert!(request.contains("granularity=day"));
		core::assert!(request.contains("window=7d"));
	}
	if name == "get_price_point" {
		core::assert!(request.contains("provider=pyth-pro"));
	}
}

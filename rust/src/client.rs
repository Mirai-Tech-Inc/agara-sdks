//! The async HTTP trading client.

use core::time::Duration;

use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};
use reqwest::{Method, RequestBuilder, Response};
use rust_decimal::Decimal;
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::error::{AgaraError, Result};
use crate::ids::{
	ConditionId, Exchange, OrderHash, OrderId, OrderType, Side, TimeInForce, TokenId,
};
use crate::models::{
	ActivitiesResponse, CancelAllOrdersResponse, CancelOrderResponse, CreateClobOrderResponse,
	CreateOrderRequest, LpIncentivesResponse, MergeRequest, OpenOrdersListRequest,
	OpenOrdersResponse, Order, OrderResponse, OrderTradesResponse, Orderbook, OrdersListRequest,
	OrdersListResponse, PortfolioSummaryEntry, PortfolioSummaryResponse, Position,
	PositionOperationResponse, PositionsListRequest, PositionsResponse, RebatesSummaryResponse,
	SignedOrderBatchRequest, SignedOrderBatchResponse, SignedOrderRequest, SplitRequest,
	StatusResponse, TradesResponse,
};
use crate::problem::ProblemDetails;
use crate::retry::RetryPolicy;
use crate::units::Micro;

/// The default trading API host (the agara sandbox).
pub const DEFAULT_BASE_URL: &str = "https://app.sandbox.agara.xyz";

/// Page size used when walking keyset pagination internally. Matches the
/// server's max page size so a full set comes back in the fewest calls.
const LIST_PAGE_SIZE: u32 = 500;

/// Consecutive explicitly retryable failures tolerated inside
/// `wait_for_terminal` before it gives up.
const MAX_CONSECUTIVE_SERVER_ERRORS: u32 = 3;

/// Async client for the agara trading API.
///
/// One `AgaraClient` multiplexes concurrent requests over a pooled,
/// keep-alive HTTP/2 connection — clone it freely across tasks (it is
/// cheap: the inner `reqwest::Client` is an `Arc`).
#[derive(Clone, Debug)]
pub struct AgaraClient {
	base_url: String,
	http: reqwest::Client,
	retry: RetryPolicy,
}

#[bon::bon]
impl AgaraClient {
	/// Build a client. `token` is a personal access token (`agt_…`).
	#[builder]
	pub fn new(
		token: String,
		#[builder(default = DEFAULT_BASE_URL.to_owned())] base_url: String,
		#[builder(default = Duration::from_secs(10))] timeout: Duration,
		#[builder(default = RetryPolicy::none())] retry: RetryPolicy,
	) -> Result<Self> {
		let mut headers = HeaderMap::new();
		let mut auth = HeaderValue::from_str(&format!("Bearer {token}"))
			.map_err(|_| AgaraError::Validation("token is not a valid header value".to_owned()))?;
		auth.set_sensitive(true);
		headers.insert(AUTHORIZATION, auth);
		headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
		headers.insert(ACCEPT, HeaderValue::from_static("application/json"));

		let http = reqwest::Client::builder().timeout(timeout).default_headers(headers).build()?;

		Ok(Self { base_url: base_url.trim_end_matches('/').to_owned(), http, retry })
	}

	/// Place a limit order. Pass exactly one of `shares` or
	/// `collateral_amount` (the latter is BUY-only, a USDC budget).
	/// Returns the accepted-order ack; poll [`AgaraClient::get_order`] or
	/// [`AgaraClient::wait_for_terminal`] to track progression.
	#[builder]
	pub async fn place_limit_order(
		&self,
		token_id: TokenId,
		side: Side,
		price: Decimal,
		shares: Option<Decimal>,
		collateral_amount: Option<Decimal>,
		#[builder(default = TimeInForce::Gtc)] time_in_force: TimeInForce,
		#[builder(default = false)] post_only: bool,
		expiration_unix_seconds: Option<i64>,
	) -> Result<CreateClobOrderResponse> {
		if shares.is_some() == collateral_amount.is_some() {
			return Err(AgaraError::Validation(
				"set exactly one of shares or collateral_amount".to_owned(),
			));
		}

		if collateral_amount.is_some() && side == Side::Sell {
			return Err(AgaraError::Validation(
				"collateral_amount is BUY-only; use shares for SELL".to_owned(),
			));
		}

		if time_in_force == TimeInForce::Gtd && expiration_unix_seconds.is_none() {
			return Err(AgaraError::Validation(
				"GTD orders require expiration_unix_seconds".to_owned(),
			));
		}

		if price <= Decimal::ZERO {
			return Err(AgaraError::Validation("price must be > 0".to_owned()));
		}

		client_validate_positive("shares", shares)?;
		client_validate_positive("collateral_amount", collateral_amount)?;

		let body = CreateOrderRequest {
			token_id,
			side,
			order_type: OrderType::Limit,
			time_in_force,
			price_micro: Some(Micro::from_units(price)),
			collateral_amount_micro: collateral_amount.map(Micro::from_units),
			shares_micro: shares.map(Micro::from_units),
			post_only,
			expiration_unix_seconds,
		};

		self.post("/trade/v1/orders", &body).await
	}

	/// Place a market order. BUY takes `collateral_amount` (a USDC budget
	/// the server walks the asks against); SELL takes `shares`. FAK fills
	/// what it can and cancels the rest; FOK rejects unless it fully
	/// fills.
	#[builder]
	pub async fn place_market_order(
		&self,
		token_id: TokenId,
		side: Side,
		shares: Option<Decimal>,
		collateral_amount: Option<Decimal>,
		#[builder(default = TimeInForce::Fak)] time_in_force: TimeInForce,
	) -> Result<CreateClobOrderResponse> {
		if !matches!(time_in_force, TimeInForce::Fak | TimeInForce::Fok) {
			return Err(AgaraError::Validation(
				"market orders require FAK or FOK time_in_force".to_owned(),
			));
		}

		match side {
			Side::Buy => {
				if collateral_amount.is_none() || shares.is_some() {
					return Err(AgaraError::Validation(
						"market BUY requires collateral_amount and no shares".to_owned(),
					));
				}
			},
			Side::Sell => {
				if shares.is_none() || collateral_amount.is_some() {
					return Err(AgaraError::Validation(
						"market SELL requires shares and no collateral_amount".to_owned(),
					));
				}
			},
			Side::Unspecified => {
				return Err(AgaraError::Validation(
					"side must be BUY or SELL".to_owned(),
				));
			},
		}

		client_validate_positive("shares", shares)?;
		client_validate_positive("collateral_amount", collateral_amount)?;

		let body = CreateOrderRequest {
			token_id,
			side,
			order_type: OrderType::Market,
			time_in_force,
			price_micro: None,
			collateral_amount_micro: collateral_amount.map(Micro::from_units),
			shares_micro: shares.map(Micro::from_units),
			post_only: false,
			expiration_unix_seconds: None,
		};

		self.post("/trade/v1/orders", &body).await
	}

	/// Snapshot of bid/ask depth for one outcome (public — no scope).
	pub async fn get_orderbook(&self, token_id: &TokenId) -> Result<Orderbook> {
		self.get(&format!("/trade/v1/orderbook/{token_id}"), &[]).await
	}

	/// Submit a pre-signed LIMIT order. Build `request` with
	/// `SignedOrder::to_request_body` (feature `signing`). Scope
	/// `orders:place_signed`.
	pub async fn place_signed_order(
		&self,
		request: &SignedOrderRequest,
	) -> Result<CreateClobOrderResponse> {
		self.post("/trade/v1/orders/signed", request).await
	}

	/// Submit up to 32 pre-signed LIMIT orders in one call. Each is
	/// accepted or rejected independently. Scope `orders:place_signed`.
	pub async fn place_signed_orders(
		&self,
		orders: Vec<SignedOrderRequest>,
	) -> Result<SignedOrderBatchResponse> {
		if orders.is_empty() || orders.len() > 32 {
			return Err(AgaraError::Validation(
				"signed-order batch must hold 1..=32 orders".to_owned(),
			));
		}

		let body = SignedOrderBatchRequest { orders };
		self.post("/trade/v1/orders/signed/batch", &body).await
	}

	/// One page of your orders (open and terminal), newest-first. Omit
	/// `cursor` for the first page; pass `pagination.next_cursor` back to
	/// walk the rest. Scope `orders:read`.
	pub async fn list_orders(
		&self,
		limit: u32,
		cursor: Option<String>,
	) -> Result<OrdersListResponse> {
		let body = OrdersListRequest { limit, cursor };
		self.post("/trade/v1/orders/list", &body).await
	}

	/// Look up one order by its internal UUID. Scope `orders:read`.
	pub async fn get_order(&self, order_id: &OrderId) -> Result<OrderResponse> {
		self.get(&format!("/trade/v1/orders/{order_id}"), &[]).await
	}

	/// Look up one order by its EIP-712 hash. Scope `orders:read`.
	pub async fn get_order_by_hash(&self, order_hash: &OrderHash) -> Result<OrderResponse> {
		self.get(&format!("/trade/v1/orders/by-hash/{order_hash}"), &[]).await
	}

	/// The fills for one order, newest-first. Scope `orders:read`.
	pub async fn get_order_trades(&self, order_id: &OrderId) -> Result<OrderTradesResponse> {
		self.get(&format!("/trade/v1/orders/{order_id}/trades"), &[]).await
	}

	/// Cancel one order. Async on the engine — poll [`AgaraClient::get_order`]
	/// until the status is `CANCELLED`. Scope `orders:cancel`.
	pub async fn cancel_order(&self, order_id: &OrderId) -> Result<CancelOrderResponse> {
		self.delete(&format!("/trade/v1/orders/{order_id}")).await
	}

	/// Cancel every open order across all your wallets. Scope
	/// `orders:cancel_all`.
	pub async fn cancel_all_orders(&self) -> Result<CancelAllOrdersResponse> {
		self.post_empty("/trade/v1/orders/cancel-all").await
	}

	/// Split USDC collateral into a YES + NO pair on-chain. Blocks until
	/// the transaction confirms. Scope `positions:split`.
	pub async fn split_position(
		&self,
		condition_id: ConditionId,
		collateral_amount_micro: Micro,
	) -> Result<PositionOperationResponse> {
		let body = SplitRequest { condition_id, collateral_amount_micro };
		self.post("/trade/v1/portfolio/positions/split", &body).await
	}

	/// Merge a complete YES + NO pair back into USDC on-chain. Scope
	/// `positions:merge`.
	pub async fn merge_position(
		&self,
		condition_id: ConditionId,
		shares_micro: Micro,
	) -> Result<PositionOperationResponse> {
		let body = MergeRequest { condition_id, shares_micro };
		self.post("/trade/v1/portfolio/positions/merge", &body).await
	}

	/// Per-exchange balances. `exchanges` restricts the fan-out; empty
	/// queries every exchange you're onboarded on. Scope `portfolio:read`.
	pub async fn get_portfolio_summary(
		&self,
		exchanges: &[Exchange],
	) -> Result<Vec<PortfolioSummaryEntry>> {
		let query = exchanges_query(exchanges);
		let resp: PortfolioSummaryResponse =
			self.get("/trade/v1/portfolio/summary", &query).await?;

		Ok(resp.summaries)
	}

	/// Every current position across both backends, in one shot.
	/// `condition_ids` filters server-side; `exchanges` restricts the
	/// fan-out. If you scoped to specific `exchanges` and a requested one
	/// is unavailable, this errors rather than returning a partial set.
	/// Scope `portfolio:read`.
	pub async fn list_positions(
		&self,
		condition_ids: Vec<ConditionId>,
		exchanges: Vec<Exchange>,
	) -> Result<Vec<Position>> {
		let requested = exchanges.clone();
		let body = PositionsListRequest { condition_ids, exchanges };
		let resp: PositionsResponse =
			self.post("/trade/v1/portfolio/positions/list", &body).await?;

		let blocked: Vec<Exchange> =
			resp.unavailable_exchanges.iter().copied().filter(|e| requested.contains(e)).collect();
		if !blocked.is_empty() {
			let names: Vec<String> = blocked.iter().map(|e| e.to_string()).collect();
			return Err(AgaraError::Server {
				status: 502,
				message: format!(
					"positions unavailable for requested exchange(s): {}",
					names.join(", ")
				),
				problem: None,
			});
		}

		Ok(resp.positions)
	}

	/// Every resting order across both backends, newest-first — walks the
	/// server's pagination internally and returns the complete set. Scope
	/// `portfolio:read`.
	pub async fn list_open_orders(
		&self,
		token_ids: Vec<TokenId>,
		exchanges: Vec<Exchange>,
	) -> Result<Vec<Order>> {
		// The filter vecs are invariant across pages; only the cursor
		// advances, so build the body once and move the cursor in place.
		let mut body =
			OpenOrdersListRequest { token_ids, exchanges, limit: LIST_PAGE_SIZE, cursor: None };
		let mut orders = Vec::new();
		loop {
			let page: OpenOrdersResponse =
				self.post("/trade/v1/portfolio/open-orders/list", &body).await?;
			orders.extend(page.orders);
			match page.pagination.next_cursor {
				Some(next) if Some(&next) != body.cursor.as_ref() => body.cursor = Some(next),
				_ => break,
			}
		}

		Ok(orders)
	}

	/// One page of recent fills, newest-first. Scope `portfolio:read`.
	pub async fn list_trades(&self, limit: u32, cursor: Option<String>) -> Result<TradesResponse> {
		let query = limit_cursor_query(limit, cursor);
		self.get("/trade/v1/portfolio/trades", &query).await
	}

	/// One page of the activity feed (fills + realized P&L), newest-first.
	/// Scope `portfolio:read`.
	pub async fn list_activities(
		&self,
		limit: u32,
		cursor: Option<String>,
	) -> Result<ActivitiesResponse> {
		let query = limit_cursor_query(limit, cursor);
		self.get("/trade/v1/portfolio/activities", &query).await
	}

	/// Pending incentive balances (maker rebate / VIP / LP), in
	/// micro-USDC. Scope `portfolio:read`.
	pub async fn get_rebates(&self) -> Result<RebatesSummaryResponse> {
		self.get("/trade/v1/portfolio/rebates", &[]).await
	}

	/// LP-incentive eligibility and per-market standing. Scope
	/// `portfolio:read`.
	pub async fn get_lp_incentives(&self) -> Result<LpIncentivesResponse> {
		self.get("/trade/v1/lp-incentives", &[]).await
	}

	/// Public market/event counts — a cheap liveness probe. No scope.
	pub async fn get_status(&self) -> Result<StatusResponse> {
		self.get("/trade/v1/status", &[]).await
	}

	/// Poll until the order reaches a terminal status or `timeout`
	/// elapses; returns the final order either way (check
	/// [`OrderStatus::is_terminal`]). Failures carrying explicit `retry` or
	/// `retry_after` recovery are retried up to 3 consecutive times.
	pub async fn wait_for_terminal(
		&self,
		order_id: &OrderId,
		timeout: Duration,
		poll_interval: Duration,
	) -> Result<Order> {
		let deadline = tokio::time::Instant::now() + timeout;
		let mut consecutive_server_errors = 0u32;
		loop {
			let mut delay = poll_interval;
			match self.get_order(order_id).await {
				Ok(resp) => {
					consecutive_server_errors = 0;
					let terminal = resp.order.status.is_terminal();
					if terminal || tokio::time::Instant::now() >= deadline {
						return Ok(resp.order);
					}
				},
				Err(e) if e.is_retryable() => {
					consecutive_server_errors += 1;
					if let Some(retry_after) = e.retry_after() {
						delay = delay.max(retry_after);
					}
					if consecutive_server_errors >= MAX_CONSECUTIVE_SERVER_ERRORS
						|| tokio::time::Instant::now() >= deadline
					{
						return Err(e);
					}
				},
				Err(e) => return Err(e),
			}

			tokio::time::sleep(delay).await;
		}
	}
}

impl AgaraClient {
	async fn get<T: DeserializeOwned>(&self, path: &str, query: &[(&str, String)]) -> Result<T> {
		let req = self.http.request(Method::GET, self.url(path)).query(query);
		self.execute(req).await
	}

	async fn delete<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
		let req = self.http.request(Method::DELETE, self.url(path));
		self.execute(req).await
	}

	async fn post<B: Serialize, T: DeserializeOwned>(&self, path: &str, body: &B) -> Result<T> {
		let req = self.http.request(Method::POST, self.url(path)).json(body);
		self.execute(req).await
	}

	async fn post_empty<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
		let req = self.http.request(Method::POST, self.url(path));
		self.execute(req).await
	}

	fn url(&self, path: &str) -> String {
		format!("{}{path}", self.base_url)
	}

	async fn execute<T: DeserializeOwned>(&self, req: RequestBuilder) -> Result<T> {
		// Common case (retries disabled): send the request directly, no clone.
		if self.retry.max_retries == 0 {
			return self.try_once(req).await;
		}

		let mut attempt = 0u32;
		loop {
			attempt += 1;
			let cloned = req.try_clone().expect("request bodies are always cloneable");
			match self.try_once(cloned).await {
				Ok(v) => return Ok(v),
				Err(e) => {
					if attempt > self.retry.max_retries || !e.is_retryable() {
						return Err(e);
					}

					let delay = e
						.retry_after()
						.filter(|_| self.retry.respect_retry_after)
						.unwrap_or_else(|| self.retry.backoff(attempt));
					tokio::time::sleep(delay).await;
				},
			}
		}
	}

	async fn try_once<T: DeserializeOwned>(&self, req: RequestBuilder) -> Result<T> {
		let resp = req.send().await?;
		let status = resp.status();
		if status.is_success() {
			let bytes = resp.bytes().await?;
			return serde_json::from_slice(&bytes)
				.map_err(|e| AgaraError::Decode(format!("{e} (body: {} bytes)", bytes.len())));
		}

		let retry_after = parse_retry_after(&resp);
		let bytes = resp.bytes().await.unwrap_or_default();
		let value = serde_json::from_slice::<serde_json::Value>(&bytes).ok();
		let problem = value
			.as_ref()
			.and_then(|value| serde_json::from_value::<ProblemDetails>(value.clone()).ok())
			.filter(|problem| problem.status == status.as_u16());
		let message = problem.as_ref().map_or_else(
			|| {
				value
					.as_ref()
					.and_then(|value| value.get("error"))
					.and_then(serde_json::Value::as_str)
					.map(str::to_owned)
					.unwrap_or_else(|| {
						String::from_utf8_lossy(&bytes).trim().chars().take(300).collect()
					})
			},
			|problem| problem.message().to_owned(),
		);

		Err(AgaraError::from_status(
			status.as_u16(),
			message,
			retry_after,
			problem,
		))
	}
}

fn client_validate_positive(name: &str, value: Option<Decimal>) -> Result<()> {
	if let Some(v) = value
		&& v <= Decimal::ZERO
	{
		return Err(AgaraError::Validation(format!("{name} must be > 0")));
	}

	Ok(())
}

fn exchanges_query(exchanges: &[Exchange]) -> Vec<(&'static str, String)> {
	if exchanges.is_empty() {
		return Vec::new();
	}

	let csv = exchanges.iter().map(|e| e.to_string()).collect::<Vec<_>>().join(",");
	vec![("exchanges", csv)]
}

fn limit_cursor_query(limit: u32, cursor: Option<String>) -> Vec<(&'static str, String)> {
	let mut query = vec![("limit", limit.to_string())];
	if let Some(c) = cursor {
		query.push(("cursor", c));
	}

	query
}

fn parse_retry_after(resp: &Response) -> Option<Duration> {
	for key in ["retry-after", "x-ratelimit-reset"] {
		if let Some(raw) = resp.headers().get(key)
			&& let Ok(text) = raw.to_str()
			&& let Ok(secs) = text.trim().parse::<f64>()
			&& secs.is_finite()
			&& secs >= 0.0
		{
			return Some(Duration::from_secs_f64(secs));
		}
	}

	None
}

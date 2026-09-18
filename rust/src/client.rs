//! Public and authenticated HTTP clients with validated request boundaries.

mod transport;

use core::time::Duration;

use std::borrow::Cow;

use reqwest::{
	Response,
	header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue},
};

use rust_decimal::Decimal;

use crate::{
	error::{AgaraError, PaginationError, PollError, ResponseError, Result},
	ids::{ConditionId, Exchange, OrderHash, OrderId, OrderType, Side, TimeInForce, TokenId},
	input,
	models::{
		ActivitiesResponse, CancelAllOrdersResponse, CancelOrderResponse, CreateClobOrderResponse,
		CreateOrderRequest, LpIncentivesResponse, MergeRequest, OpenOrdersListRequest,
		OpenOrdersResponse, Order, OrderResponse, OrderTradesResponse, Orderbook,
		OrdersListRequest, OrdersListResponse, PortfolioSummaryEntry, PortfolioSummaryResponse,
		Position, PositionOperationResponse, PositionsListRequest, PositionsResponse,
		RebatesSummaryResponse, SignedOrderBatchRequest, SignedOrderBatchResponse,
		SignedOrderRequest, SplitRequest, StatusResponse, TradesResponse,
	},
	retry::RetryPolicy,
	units::Micro,
	validation::{ConfigurationError, Field, OrderInputReason, ValidationError},
};

/// The default trading API host (the agara sandbox).
pub const DEFAULT_BASE_URL: &str = "https://app.sandbox.agara.xyz";

const LIST_PAGE_SIZE: u32 = 500;

const MAX_AUTO_PAGES: usize = 1000;

const MAX_CONSECUTIVE_SERVER_ERRORS: u32 = 3;

/// Client state with access to authenticated trader operations.
#[derive(Clone, Debug)]
pub struct Authenticated;

/// Client state restricted to public reads.
#[derive(Clone, Debug)]
pub struct Anonymous;

/// Async client for the agara trading API.
///
/// One `AgaraClient` multiplexes concurrent requests over a pooled,
/// keep-alive HTTP/2 connection — clone it freely across tasks (it is
/// cheap: the inner `reqwest::Client` is an `Arc`).
#[derive(Clone, Debug)]
pub struct AgaraClient<A = Authenticated> {
	auth: core::marker::PhantomData<A>,
	authorization: Option<HeaderValue>,
	base_url: reqwest::Url,
	http: reqwest::Client,
	timeout: Duration,
	retry: RetryPolicy,
}

pub(crate) fn parse_retry_after(resp: &Response) -> Option<crate::error::RetryAfter> {
	let mut delay: Option<Duration> = None;
	for key in ["retry-after", "x-ratelimit-reset"] {
		for raw in resp.headers().get_all(key) {
			let parsed = raw
				.to_str()
				.ok()
				.and_then(|text| text.trim().parse::<f64>().ok())
				.filter(|value| value.is_finite() && *value >= 0.0)
				.and_then(|seconds| Duration::try_from_secs_f64(seconds).ok());
			let Some(parsed) = parsed else {
				return Some(crate::error::RetryAfter::Unusable { value: raw.clone() });
			};
			delay = Some(delay.map_or(parsed, |current| current.max(parsed)));
		}
	}

	delay.map(crate::error::RetryAfter::Delay)
}

pub(crate) fn validate_limit_options(
	side: Side,
	tif: TimeInForce,
	post_only: bool,
	expiration: Option<i64>,
) -> Result<()> {
	if side == Side::Unspecified {
		return Err(ValidationError::order(Field::Side, OrderInputReason::Unsupported).into());
	}
	if tif == TimeInForce::Unspecified {
		return Err(
			ValidationError::order(Field::TimeInForce, OrderInputReason::Unsupported).into(),
		);
	}
	if post_only && core::matches!(tif, TimeInForce::Fak | TimeInForce::Fok) {
		return Err(
			ValidationError::order(Field::PostOnly, OrderInputReason::PostOnlyTimeInForce).into(),
		);
	}
	match (tif, expiration) {
		(TimeInForce::Gtd, None) => {
			return Err(
				ValidationError::order(Field::Expiration, OrderInputReason::Required).into(),
			);
		},
		(TimeInForce::Gtd, Some(expiration)) if expiration < 0 => {
			return Err(ValidationError::order(
				Field::Expiration,
				OrderInputReason::NegativeExpiration,
			)
			.into());
		},
		(TimeInForce::Gtd, Some(_)) => {},
		(_, Some(_)) => {
			return Err(
				ValidationError::order(Field::Expiration, OrderInputReason::Forbidden).into(),
			);
		},
		_ => {},
	}

	Ok(())
}

fn exchanges_query(exchanges: &[Exchange]) -> Result<Vec<(&'static str, String)>> {
	input::exchanges(exchanges)?;
	if exchanges.is_empty() {
		return Ok(Vec::new());
	}

	let csv = exchanges.iter().map(|e| e.to_string()).collect::<Vec<_>>().join(",");
	Ok(std::vec![("exchanges", csv)])
}

fn limit_cursor_query(limit: u32, cursor: Option<String>) -> Result<Vec<(&'static str, String)>> {
	input::number(i64::from(limit), Field::Limit, 1, input::MAX_LIST_LIMIT)?;
	input::cursor(cursor.as_deref())?;
	let mut query = std::vec![("limit", limit.to_string())];
	if let Some(c) = cursor {
		query.push(("cursor", c));
	}

	Ok(query)
}

#[bon::bon]
impl AgaraClient<Authenticated> {
	/// Build an authenticated trader client using a PAT or supported identity bearer credential.
	/// Injected HTTP client configuration remains caller-owned; SDK request validation and deadlines
	/// still apply.
	#[builder]
	pub fn new(
		token: Cow<'static, str>,
		http: Option<reqwest::Client>,
		#[builder(default = Cow::Borrowed(DEFAULT_BASE_URL))] base_url: Cow<'static, str>,
		#[builder(default = Duration::from_secs(10))] timeout: Duration,
		#[builder(default = RetryPolicy::none())] retry: RetryPolicy,
	) -> Result<Self> {
		if token.trim().is_empty() {
			return Err(ValidationError::from(ConfigurationError::EmptyToken).into());
		}

		if token.chars().any(char::is_whitespace) {
			return Err(ValidationError::identifier(
				Field::ApiToken,
				crate::validation::IdentifierReason::Whitespace,
			)
			.into());
		}
		let base_url = input::base_url(&base_url)?;
		input::duration(timeout, Field::Timeout)?;
		let mut headers = HeaderMap::new();
		let mut auth = HeaderValue::from_str(&std::format!("Bearer {token}")).map_err(|error| {
			ValidationError::from(ConfigurationError::InvalidTokenHeader(error))
		})?;
		auth.set_sensitive(true);
		headers.insert(AUTHORIZATION, auth.clone());
		headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
		headers.insert(ACCEPT, HeaderValue::from_static("application/json"));

		let http = match http {
			Some(http) => http,
			None => reqwest::Client::builder()
				.redirect(reqwest::redirect::Policy::none())
				.connect_timeout(timeout)
				.default_headers(headers)
				.build()?,
		};

		Ok(Self {
			base_url,
			http,
			timeout,
			retry,
			authorization: Some(auth),
			auth: core::marker::PhantomData,
		})
	}

	/// Place a LIMIT order with an exact share quantity. Market tick and notional rules are enforced by the server.
	#[builder]
	pub async fn place_limit_order(
		&self,
		token_id: TokenId,
		side: Side,
		price: Decimal,
		shares: Decimal,
		#[builder(default = TimeInForce::Gtc)] time_in_force: TimeInForce,
		#[builder(default = false)] post_only: bool,
		expiration_unix_seconds: Option<i64>,
	) -> Result<CreateClobOrderResponse> {
		let price_micro = Micro::from_units(price)?;
		let shares_micro = Micro::from_units(shares)?;
		let body = CreateOrderRequest {
			token_id,
			side,
			order_type: OrderType::Limit,
			time_in_force,
			price_micro: Some(price_micro),
			collateral_amount_micro: None,
			shares_micro: Some(shares_micro),
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
		let body = CreateOrderRequest {
			token_id,
			side,
			order_type: OrderType::Market,
			time_in_force,
			price_micro: None,
			collateral_amount_micro: collateral_amount.map(Micro::from_units).transpose()?,
			shares_micro: shares.map(Micro::from_units).transpose()?,
			post_only: false,
			expiration_unix_seconds: None,
		};

		self.post("/trade/v1/orders", &body).await
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

	/// Submit one to thirty-two independently validated signed LIMIT orders. Shared wallet/provider
	/// failures may reject the entire request; inspect each nested failure. Scope:
	/// orders:place_signed.
	pub async fn place_signed_orders(
		&self,
		orders: Vec<SignedOrderRequest>,
	) -> Result<SignedOrderBatchResponse> {
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
		self.get(
			&std::format!(
				"/trade/v1/orders/{}",
				crate::endpoints::path_segment(order_id.as_str())?
			),
			&[],
		)
		.await
	}

	/// Look up one order by its EIP-712 hash. Scope `orders:read`.
	pub async fn get_order_by_hash(&self, order_hash: &OrderHash) -> Result<OrderResponse> {
		self.get(
			&std::format!(
				"/trade/v1/orders/by-hash/{}",
				crate::endpoints::path_segment(order_hash.as_str())?
			),
			&[],
		)
		.await
	}

	/// The fills for one order, newest-first. Scope `orders:read`.
	pub async fn get_order_trades(&self, order_id: &OrderId) -> Result<OrderTradesResponse> {
		self.get(
			&std::format!(
				"/trade/v1/orders/{}/trades",
				crate::endpoints::path_segment(order_id.as_str())?
			),
			&[],
		)
		.await
	}

	/// Cancel one order. Async on the engine — poll [`AgaraClient::get_order`]
	/// until `is_terminal` is true. Scope `orders:cancel`.
	pub async fn cancel_order(&self, order_id: &OrderId) -> Result<CancelOrderResponse> {
		self.delete(&std::format!(
			"/trade/v1/orders/{}",
			crate::endpoints::path_segment(order_id.as_str())?
		))
		.await
	}

	/// Cancel every open order across all your wallets. Scope
	/// `orders:cancel_all`.
	pub async fn cancel_all_orders(&self) -> Result<CancelAllOrdersResponse> {
		self.post_empty("/trade/v1/orders/cancel-all").await
	}

	/// Split USDC collateral into a YES + NO pair. The
	/// accepted response carries a batch hash on AGARA. Scope `positions:split`.
	pub async fn split_position(
		&self,
		condition_id: ConditionId,
		collateral_amount_micro: Micro,
	) -> Result<PositionOperationResponse> {
		let body = SplitRequest { condition_id, collateral_amount_micro };
		self.post("/trade/v1/portfolio/positions/split", &body).await
	}

	/// Merge a complete YES/NO set back into collateral. AGARA returns an asynchronous batch
	/// acceptance; Polymarket returns a relayer receipt. Scope: positions:merge.
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
		let query = exchanges_query(exchanges)?;
		let resp: PortfolioSummaryResponse =
			self.get("/trade/v1/portfolio/summary", &query).await?;

		Ok(resp.summaries)
	}

	/// Read current positions across requested exchanges, with empty filters selecting all onboarded
	/// exchanges. Any requested unavailable exchange produces PartialData; use get_positions to retain
	/// a partial envelope and its metadata. Scope: portfolio:read.
	pub async fn list_positions(
		&self,
		condition_ids: Vec<ConditionId>,
		exchanges: Vec<Exchange>,
	) -> Result<Vec<Position>> {
		let requested = exchanges.clone();
		let body = PositionsListRequest { condition_ids, exchanges };
		let resp: PositionsResponse =
			self.post("/trade/v1/portfolio/positions/list", &body).await?;

		let blocked: Vec<Exchange> = resp
			.unavailable_exchanges
			.iter()
			.copied()
			.filter(|e| requested.is_empty() || requested.contains(e))
			.collect();
		if !blocked.is_empty() {
			return Err(ResponseError::PartialData { exchanges: blocked }.into());
		}

		Ok(resp.positions)
	}

	/// Walk every cursor page of nonterminal orders, newest first, up to the configured safety bound.
	/// An empty page with a cursor does not end the walk. Scope: portfolio:read.
	pub async fn list_open_orders(
		&self,
		token_ids: Vec<TokenId>,
		exchanges: Vec<Exchange>,
	) -> Result<Vec<Order>> {
		let mut body =
			OpenOrdersListRequest { token_ids, exchanges, limit: LIST_PAGE_SIZE, cursor: None };
		let mut orders = Vec::new();
		let mut seen = std::collections::HashSet::new();
		loop {
			if seen.len() >= MAX_AUTO_PAGES {
				return Err(PaginationError::PageLimitExceeded { limit: MAX_AUTO_PAGES }.into());
			}

			let page: OpenOrdersResponse =
				self.post("/trade/v1/portfolio/open-orders/list", &body).await?;
			orders.extend(page.orders);
			match page.pagination.next_cursor {
				Some(next) => {
					if !seen.insert(next.clone()) {
						return Err(PaginationError::CursorCycle.into());
					}
					body.cursor = Some(next);
				},
				None => break,
			}
		}

		Ok(orders)
	}

	/// One page of recent fills, newest-first. Scope `portfolio:read`.
	pub async fn list_trades(&self, limit: u32, cursor: Option<String>) -> Result<TradesResponse> {
		let query = limit_cursor_query(limit, cursor)?;
		self.get("/trade/v1/portfolio/trades", &query).await
	}

	/// Read one newest-first page of orders with fills, splits, merges, redemptions, deposits,
	/// withdrawals and LP payouts. Realized PnL is a separate report; scope: portfolio:read.
	pub async fn list_activities(
		&self,
		limit: u32,
		cursor: Option<String>,
	) -> Result<ActivitiesResponse> {
		let query = limit_cursor_query(limit, cursor)?;
		self.get("/trade/v1/portfolio/activities", &query).await
	}

	/// Pending incentive balances (maker rebate / VIP / LP), in
	/// micro-USDC. Scope `portfolio:read`.
	pub async fn get_rebates(&self) -> Result<RebatesSummaryResponse> {
		self.get("/trade/v1/portfolio/rebates", &[]).await
	}

	/// Poll until the authoritative is_terminal flag is true, within an absolute deadline. Up to three
	/// consecutive explicitly retryable failures are tolerated; order completion is separate from
	/// trade settlement.
	pub async fn wait_for_terminal(
		&self,
		order_id: &OrderId,
		timeout: Duration,
		poll_interval: Duration,
	) -> Result<Order> {
		let deadline = input::deadline(timeout, poll_interval)?;
		let mut consecutive_server_errors = 0u32;
		loop {
			if tokio::time::Instant::now() >= deadline {
				return Err(PollError::DeadlineElapsed.into());
			}

			let mut delay = poll_interval;
			match tokio::time::timeout_at(deadline, self.get_order(order_id))
				.await
				.map_err(|_| AgaraError::from(PollError::DeadlineElapsed))?
			{
				Ok(resp) => {
					if tokio::time::Instant::now() >= deadline {
						return Err(PollError::DeadlineElapsed.into());
					}

					consecutive_server_errors = 0;
					let terminal = resp.order.is_terminal;
					if terminal {
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

			let next =
				tokio::time::Instant::now().checked_add(delay).unwrap_or(deadline).min(deadline);
			tokio::time::sleep_until(next).await;
		}
	}
}

#[bon::bon]
impl AgaraClient<Anonymous> {
	/// Build an anonymous client that sends no credentials.
	#[builder]
	pub fn anonymous(
		#[builder(default = Cow::Borrowed(DEFAULT_BASE_URL))] base_url: Cow<'static, str>,
		#[builder(default = Duration::from_secs(10))] timeout: Duration,
		#[builder(default = RetryPolicy::none())] retry: RetryPolicy,
	) -> Result<Self> {
		let base_url = input::base_url(&base_url)?;
		input::duration(timeout, Field::Timeout)?;
		let http = reqwest::Client::builder()
			.redirect(reqwest::redirect::Policy::none())
			.connect_timeout(timeout)
			.build()?;
		Ok(Self {
			base_url,
			http,
			timeout,
			retry,
			authorization: None,
			auth: core::marker::PhantomData,
		})
	}
}

impl<A> AgaraClient<A> {
	/// Read whole-unit REST book depth and its decimal sequence hash. Use the hash as a state fence
	/// when rebuilding a live book; configured credentials are retained.
	pub async fn get_orderbook(&self, token_id: &TokenId) -> Result<Orderbook> {
		self.get(
			&std::format!(
				"/trade/v1/orderbook/{}",
				crate::endpoints::path_segment(token_id.as_str())?
			),
			&[],
		)
		.await
	}
	/// Read public current/upcoming LP opportunities; configured credentials enable the caller’s
	/// personal standing.
	pub async fn get_lp_incentives(&self) -> Result<LpIncentivesResponse> {
		self.get("/trade/v1/lp-incentives", &[]).await
	}
	/// Read public platform market/event counts; configured credentials are retained.
	pub async fn get_status(&self) -> Result<StatusResponse> {
		self.get("/trade/v1/status", &[]).await
	}
}

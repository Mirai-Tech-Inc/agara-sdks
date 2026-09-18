"""Blocking current-trader client. Responses preserve the complete wire envelope."""

from __future__ import annotations

import time
from collections.abc import Iterator
from typing import TYPE_CHECKING, Any, Literal, cast

import requests

from . import catalogue, models
from ._http import check_deadline, finite_positive, request_timeout, require_auth, validate_settings
from ._requests import (
    DEFAULT_BASE_URL,
    _compact,
    _limit_body,
    _market_body,
    _segment,
    _validate_batch,
    _validate_order,
    batch_is_terminal,
)
from .amounts import Amount, positive_micro
from .errors import AgaraError, ProtocolError, TransportError, _raise_api_error
from .prices import PriceEvent, iter_prices

if TYPE_CHECKING:
    from .signing import SignedOrder, SignedOrderEntry


class AgaraClient:
    """One requests session per thread. Injected sessions remain owned by the caller."""

    def __init__(
        self,
        token: str | None = None,
        base_url: str = DEFAULT_BASE_URL,
        timeout: float = 10.0,
        session: requests.Session | None = None,
    ):
        validate_settings(base_url, timeout)
        self._token = token
        self.base_url = base_url.rstrip("/")
        self.timeout = timeout
        self._owns_session = session is None
        self.session = session if session is not None else requests.Session()
        self._headers = {"Accept": "application/json", "Content-Type": "application/json"}
        if token:
            self._headers["Authorization"] = f"Bearer {token}"

    def _request(
        self,
        method: str,
        path: str,
        json: Any = None,
        params: Any = None,
        deadline: float | None = None,
    ) -> Any:
        require_auth(path, self._token)
        timeout = request_timeout(self.timeout, deadline)
        try:
            response = self.session.request(
                method,
                self.base_url + path,
                json=json,
                params=params,
                headers=self._headers,
                timeout=timeout,
                allow_redirects=False,
            )
        except requests.RequestException as exc:
            raise TransportError(method, exc) from exc
        check_deadline(deadline)
        if not 200 <= response.status_code < 300:
            try:
                error_body = response.json()
            except ValueError:
                error_body = response.text
            _raise_api_error(response.status_code, error_body, response.headers)
        if not response.content:
            return None
        try:
            value = response.json()
        except ValueError as exc:
            raise ProtocolError("successful response contained invalid JSON") from exc
        if not isinstance(value, dict):
            raise ProtocolError("successful response was not a JSON object")
        return value

    def get_status(self) -> models.Status:
        """GET /trade/v1/status."""
        return cast(models.Status, self._request("GET", "/trade/v1/status"))

    def submit_order(self, order: models.OrderRequest) -> models.CreateClobOrderResponse:
        """Submit an explicit micro-unit LIMIT or MARKET request."""
        _validate_order(order)
        return cast(
            models.CreateClobOrderResponse,
            self._request("POST", "/trade/v1/orders", json=dict(order)),
        )

    def submit_signed_order(
        self, order: models.SignedOrderRequest
    ) -> models.CreateClobOrderResponse:
        """POST /trade/v1/orders/signed."""
        _validate_order(order)
        return cast(
            models.CreateClobOrderResponse,
            self._request("POST", "/trade/v1/orders/signed", json=dict(order)),
        )

    def submit_signed_orders(
        self, orders: list[models.SignedOrderRequest]
    ) -> models.SignedOrdersResponse:
        """Submit independently validated orders; shared dependency failures fail the request."""
        if not 1 <= len(orders) <= 32:
            raise ValueError("signed batch must contain 1 to 32 orders")
        for order in orders:
            _validate_order(order)
        return cast(
            models.SignedOrdersResponse,
            self._request("POST", "/trade/v1/orders/signed/batch", json={"orders": orders}),
        )

    def list_orders(
        self, limit: int = 500, cursor: str | None = None
    ) -> models.ClobOrdersListResponse:
        """POST /trade/v1/orders/list."""
        return cast(
            models.ClobOrdersListResponse,
            self._request(
                "POST", "/trade/v1/orders/list", json=_compact(limit=limit, cursor=cursor)
            ),
        )

    def get_order(self, order_id: str) -> models.ClobOrderResponse:
        """GET /trade/v1/orders/{order_id}."""
        return cast(
            models.ClobOrderResponse, self._request("GET", f"/trade/v1/orders/{_segment(order_id)}")
        )

    def get_order_by_hash(self, order_hash: str) -> models.ClobOrderResponse:
        """GET /trade/v1/orders/by-hash/{order_hash}."""
        return cast(
            models.ClobOrderResponse,
            self._request("GET", f"/trade/v1/orders/by-hash/{_segment(order_hash)}"),
        )

    def get_order_trades(self, order_id: str) -> models.OrderTradesResponse:
        """GET /trade/v1/orders/{order_id}/trades."""
        return cast(
            models.OrderTradesResponse,
            self._request("GET", f"/trade/v1/orders/{_segment(order_id)}/trades"),
        )

    def cancel_order(self, order_id: str) -> models.CancelClobOrderResponse:
        """DELETE /trade/v1/orders/{order_id}."""
        return cast(
            models.CancelClobOrderResponse,
            self._request("DELETE", f"/trade/v1/orders/{_segment(order_id)}"),
        )

    def cancel_all_orders(self) -> models.CancelAllClobOrdersResponse:
        """POST /trade/v1/orders/cancel-all."""
        return cast(
            models.CancelAllClobOrdersResponse, self._request("POST", "/trade/v1/orders/cancel-all")
        )

    def submit_batch(self, batch: models.BatchSubmission) -> models.BatchAccepted:
        """POST /trade/v1/batches."""
        _validate_batch(batch)
        return cast(
            models.BatchAccepted, self._request("POST", "/trade/v1/batches", json=dict(batch))
        )

    def get_batch(self, batch_hash: str) -> models.AccountBatchStatusDto:
        """GET /trade/v1/batches/{batch_hash}."""
        return cast(
            models.AccountBatchStatusDto,
            self._request("GET", f"/trade/v1/batches/{_segment(batch_hash)}"),
        )

    def supersede_batch(
        self, batch_hash: str, batch: models.BatchSupersedeSubmission
    ) -> models.BatchSupersedeResult:
        """POST /trade/v1/batches/{batch_hash}/supersede."""
        _validate_batch(batch)
        return cast(
            models.BatchSupersedeResult,
            self._request(
                "POST", f"/trade/v1/batches/{_segment(batch_hash)}/supersede", json=dict(batch)
            ),
        )

    def get_batch_group(self, group_id: str) -> models.BatchGroupStatusDto:
        """GET /trade/v1/batch-groups/{group_id}."""
        return cast(
            models.BatchGroupStatusDto,
            self._request("GET", f"/trade/v1/batch-groups/{_segment(group_id)}"),
        )

    def get_orderbook(self, token_id: str) -> models.RawOrderbook:
        """Depth in whole price/share units (JSON numbers); use WebSocket scales for native integers."""
        return cast(
            models.RawOrderbook, self._request("GET", f"/trade/v1/orderbook/{_segment(token_id)}")
        )

    def list_activities(
        self, limit: int = 50, cursor: str | None = None
    ) -> models.PortfolioActivitiesResponse:
        """GET /trade/v1/portfolio/activities."""
        return cast(
            models.PortfolioActivitiesResponse,
            self._request(
                "GET", "/trade/v1/portfolio/activities", params=_compact(limit=limit, cursor=cursor)
            ),
        )

    def get_deposit_address(self) -> models.PortfolioBridgeDepositAddressResponse:
        """GET /trade/v1/portfolio/bridge/deposit/address."""
        return cast(
            models.PortfolioBridgeDepositAddressResponse,
            self._request("GET", "/trade/v1/portfolio/bridge/deposit/address"),
        )

    def get_deposit_supported_assets(self) -> models.PortfolioBridgeSupportedAssetsResponse:
        """GET /trade/v1/portfolio/bridge/deposit/supported-assets."""
        return cast(
            models.PortfolioBridgeSupportedAssetsResponse,
            self._request("GET", "/trade/v1/portfolio/bridge/deposit/supported-assets"),
        )

    def get_deposit_quote(
        self, request: models.PortfolioBridgeDepositQuoteRequest
    ) -> models.PortfolioBridgeDepositQuoteResponse:
        """POST /trade/v1/portfolio/bridge/deposit/quote."""
        return cast(
            models.PortfolioBridgeDepositQuoteResponse,
            self._request("POST", "/trade/v1/portfolio/bridge/deposit/quote", json=dict(request)),
        )

    def get_withdraw_supported_assets(self) -> models.PortfolioBridgeSupportedAssetsResponse:
        """GET /trade/v1/portfolio/bridge/withdraw/supported-assets."""
        return cast(
            models.PortfolioBridgeSupportedAssetsResponse,
            self._request("GET", "/trade/v1/portfolio/bridge/withdraw/supported-assets"),
        )

    def get_withdraw_quote(
        self, request: models.PortfolioBridgeWithdrawQuoteRequest
    ) -> models.PortfolioBridgeWithdrawQuoteResponse:
        """POST /trade/v1/portfolio/bridge/withdraw/quote."""
        return cast(
            models.PortfolioBridgeWithdrawQuoteResponse,
            self._request("POST", "/trade/v1/portfolio/bridge/withdraw/quote", json=dict(request)),
        )

    def list_open_orders(
        self,
        *,
        token_ids: list[str] | None = None,
        exchanges: list[str] | None = None,
        limit: int = 500,
        cursor: str | None = None,
    ) -> models.PortfolioOpenOrdersResponse:
        """Return one complete page including metadata and cursor."""
        return cast(
            models.PortfolioOpenOrdersResponse,
            self._request(
                "POST",
                "/trade/v1/portfolio/open-orders/list",
                json=_compact(
                    token_ids=token_ids or [], exchanges=exchanges or [], limit=limit, cursor=cursor
                ),
            ),
        )

    def list_positions(
        self, condition_ids: list[str] | None = None, exchanges: list[str] | None = None
    ) -> models.PortfolioPositionsResponse:
        """Return positions and availability; unavailable_exchanges must be empty before reconciliation."""
        return cast(
            models.PortfolioPositionsResponse,
            self._request(
                "POST",
                "/trade/v1/portfolio/positions/list",
                json={"condition_ids": condition_ids or [], "exchanges": exchanges or []},
            ),
        )

    def split_position(
        self, *, condition_id: str, collateral_amount_micro: int | str
    ) -> models.PositionOperationResponse:
        """AGARA returns a batch acceptance; POLYMARKET returns a relayer receipt."""
        return cast(
            models.PositionOperationResponse,
            self._request(
                "POST",
                "/trade/v1/portfolio/positions/split",
                json={
                    "condition_id": condition_id,
                    "collateral_amount_micro": str(positive_micro(collateral_amount_micro)),
                },
            ),
        )

    def merge_position(
        self, *, condition_id: str, shares_micro: int | str
    ) -> models.PositionOperationResponse:
        """AGARA returns a batch acceptance; POLYMARKET returns a relayer receipt."""
        return cast(
            models.PositionOperationResponse,
            self._request(
                "POST",
                "/trade/v1/portfolio/positions/merge",
                json={
                    "condition_id": condition_id,
                    "shares_micro": str(positive_micro(shares_micro)),
                },
            ),
        )

    def get_portfolio_summary(
        self, exchanges: list[str] | None = None
    ) -> models.PortfolioSummaryResponse:
        """GET /trade/v1/portfolio/summary."""
        return cast(
            models.PortfolioSummaryResponse,
            self._request(
                "GET",
                "/trade/v1/portfolio/summary",
                params=_compact(exchanges=",".join(exchanges) if exchanges else None),
            ),
        )

    def list_trades(
        self, limit: int = 500, cursor: str | None = None
    ) -> models.PortfolioTradesResponse:
        """GET /trade/v1/portfolio/trades."""
        return cast(
            models.PortfolioTradesResponse,
            self._request(
                "GET", "/trade/v1/portfolio/trades", params=_compact(limit=limit, cursor=cursor)
            ),
        )

    def get_rebates(self) -> models.RebatesSummaryResponse:
        """GET /trade/v1/portfolio/rebates."""
        return cast(
            models.RebatesSummaryResponse, self._request("GET", "/trade/v1/portfolio/rebates")
        )

    def list_lp_incentives(
        self,
        *,
        category: str | None = None,
        search: str | None = None,
        sort_by: str | None = None,
        sort_order: Literal["asc", "desc"] | None = None,
    ) -> models.LpIncentivesResponse:
        """GET /trade/v1/lp-incentives."""
        return cast(
            models.LpIncentivesResponse,
            self._request(
                "GET",
                "/trade/v1/lp-incentives",
                params=_compact(
                    category=category, search=search, sort_by=sort_by, sort_order=sort_order
                ),
            ),
        )

    def list_lp_incentive_categories(self) -> models.LpIncentiveCategoriesResponse:
        """GET /trade/v1/lp-incentives/categories."""
        return cast(
            models.LpIncentiveCategoriesResponse,
            self._request("GET", "/trade/v1/lp-incentives/categories"),
        )

    def list_closed_lp_incentives(
        self,
        *,
        limit: int = 20,
        page: int = 1,
        category: str | None = None,
        search: str | None = None,
        sort_by: Literal["date", "earned"] | None = None,
        sort_order: Literal["asc", "desc"] | None = None,
    ) -> models.ClosedLpIncentivesResponse:
        """GET /trade/v1/lp-incentives/closed."""
        return cast(
            models.ClosedLpIncentivesResponse,
            self._request(
                "GET",
                "/trade/v1/lp-incentives/closed",
                params=_compact(
                    limit=limit,
                    page=page,
                    category=category,
                    search=search,
                    sort_by=sort_by,
                    sort_order=sort_order,
                ),
            ),
        )

    def list_closed_lp_incentive_categories(self) -> models.LpIncentiveCategoriesResponse:
        """GET /trade/v1/lp-incentives/closed/categories."""
        return cast(
            models.LpIncentiveCategoriesResponse,
            self._request("GET", "/trade/v1/lp-incentives/closed/categories"),
        )

    def get_lp_incentive_earnings(self) -> models.LpIncentiveEarningsResponse:
        """GET /trade/v1/lp-incentives/earnings."""
        return cast(
            models.LpIncentiveEarningsResponse,
            self._request("GET", "/trade/v1/lp-incentives/earnings"),
        )

    def get_portfolio_pnl(self) -> models.WalletPnlSnapshotDto:
        """GET /trade/v1/portfolio/pnl."""
        return cast(models.WalletPnlSnapshotDto, self._request("GET", "/trade/v1/portfolio/pnl"))

    def get_portfolio_pnl_history(
        self, *, from_: str, to: str, limit: int = 672
    ) -> models.WalletPnlHistoryDto:
        """GET /trade/v1/portfolio/pnl/history."""
        return cast(
            models.WalletPnlHistoryDto,
            self._request(
                "GET",
                "/trade/v1/portfolio/pnl/history",
                params={"from": from_, "to": to, "limit": limit},
            ),
        )

    def get_realized_pnl(
        self,
        *,
        granularity: Literal["hour", "day", "week"],
        window: Literal["1d", "7d", "30d", "all"],
    ) -> models.RealizedPnlReportDto:
        """GET /trade/v1/portfolio/pnl/realized."""
        return cast(
            models.RealizedPnlReportDto,
            self._request(
                "GET",
                "/trade/v1/portfolio/pnl/realized",
                params={"granularity": granularity, "window": window},
            ),
        )

    def get_trading_day(self, mic: str, date: str) -> catalogue.TradingDay:
        """GET /api/v1/calendars/{mic}/days/{date}."""
        return cast(
            catalogue.TradingDay,
            self._request("GET", f"/api/v1/calendars/{_segment(mic)}/days/{_segment(date)}"),
        )

    def list_trading_days(self, mic: str, *, from_: str, to: str) -> catalogue.TradingDaysResponse:
        """GET /api/v1/calendars/{mic}/days."""
        return cast(
            catalogue.TradingDaysResponse,
            self._request(
                "GET", f"/api/v1/calendars/{_segment(mic)}/days", params={"from": from_, "to": to}
            ),
        )

    def get_next_session(self, mic: str, *, from_: str) -> catalogue.NextSessionResponse:
        """GET /api/v1/calendars/{mic}/next-session."""
        return cast(
            catalogue.NextSessionResponse,
            self._request(
                "GET", f"/api/v1/calendars/{_segment(mic)}/next-session", params={"from": from_}
            ),
        )

    def list_calendars(self) -> catalogue.TradingVenuesResponse:
        """GET /api/v1/calendars."""
        return cast(catalogue.TradingVenuesResponse, self._request("GET", "/api/v1/calendars"))

    def get_calendar(self, mic: str) -> catalogue.TradingVenue:
        """GET /api/v1/calendars/{mic}."""
        return cast(
            catalogue.TradingVenue, self._request("GET", f"/api/v1/calendars/{_segment(mic)}")
        )

    def get_category(self, slug: str) -> catalogue.CategoryResource:
        """GET /api/v1/categories/{slug}."""
        return cast(
            catalogue.CategoryResource, self._request("GET", f"/api/v1/categories/{_segment(slug)}")
        )

    def get_event_canonical_url(self, slug: str) -> catalogue.EventCanonicalUrlResponse:
        """GET /api/v1/events/{slug}/canonical-url."""
        return cast(
            catalogue.EventCanonicalUrlResponse,
            self._request("GET", f"/api/v1/events/{_segment(slug)}/canonical-url"),
        )

    def get_event(self, slug: str) -> catalogue.EventDetail:
        """GET /api/v1/events/{slug}."""
        return cast(catalogue.EventDetail, self._request("GET", f"/api/v1/events/{_segment(slug)}"))

    def list_events(
        self,
        *,
        category: str | None = None,
        root: str | None = None,
        event_type_bucket: Literal["games", "props"] | None = None,
        filter: str | None = None,
        source: str | None = None,
        exclude_ended: bool | None = None,
        resolution: Literal["all", "active", "proposed", "disputed", "resolved"] | None = None,
        sort: Literal["time", "volume", "markets"] | None = None,
        cursor: str | None = None,
        limit: int = 20,
        include_markets: bool | None = None,
    ) -> catalogue.EventsListResponse:
        """GET /api/v1/events."""
        return cast(
            catalogue.EventsListResponse,
            self._request(
                "GET",
                "/api/v1/events",
                params=_compact(
                    category=category,
                    root=root,
                    event_type_bucket=event_type_bucket,
                    filter=filter,
                    source=source,
                    exclude_ended=exclude_ended,
                    resolution=resolution,
                    sort=sort,
                    cursor=cursor,
                    limit=limit,
                    include_markets=include_markets,
                ),
            ),
        )

    def get_market(self, market_id: str) -> catalogue.MarketDetail:
        """GET /api/v1/markets/{market_id}."""
        return cast(
            catalogue.MarketDetail, self._request("GET", f"/api/v1/markets/{_segment(market_id)}")
        )

    def list_markets(
        self,
        *,
        source: str = "agara",
        event_slug: str | None = None,
        state: str | None = None,
        cursor: str | None = None,
        limit: int = 64,
    ) -> catalogue.MarketsListResponse:
        """GET /api/v1/markets."""
        return cast(
            catalogue.MarketsListResponse,
            self._request(
                "GET",
                "/api/v1/markets",
                params=_compact(
                    source=source, event_slug=event_slug, state=state, cursor=cursor, limit=limit
                ),
            ),
        )

    def get_price_point(self, *, symbol: str, provider: str, at: int) -> catalogue.PricePointBody:
        """GET /api/v1/prices/point."""
        return cast(
            catalogue.PricePointBody,
            self._request(
                "GET",
                "/api/v1/prices/point",
                params={"symbol": symbol, "provider": provider, "at": at},
            ),
        )

    def get_price_ticks(
        self, *, symbol: str, provider: str, from_: int, to: int
    ) -> catalogue.ShimHistory:
        """GET /api/v1/prices/ticks."""
        return cast(
            catalogue.ShimHistory,
            self._request(
                "GET",
                "/api/v1/prices/ticks",
                params={"symbol": symbol, "provider": provider, "from": from_, "to": to},
            ),
        )

    def get_token_history(
        self, token_id: str, *, range: str = "1d", points: int = 400
    ) -> catalogue.TokenHistoryResponse:
        """GET /api/v1/prices/token-history."""
        return cast(
            catalogue.TokenHistoryResponse,
            self._request(
                "GET",
                "/api/v1/prices/token-history",
                params={"token_id": token_id, "range": range, "points": points},
            ),
        )

    def search(self, q: str, *, limit: int = 10, source: str = "AGARA") -> catalogue.SearchResponse:
        """GET /api/v1/search."""
        return cast(
            catalogue.SearchResponse,
            self._request(
                "GET", "/api/v1/search", params={"q": q, "limit": limit, "source": source}
            ),
        )

    def list_securities(self) -> catalogue.SecuritiesResponse:
        """GET /api/v1/securities."""
        return cast(catalogue.SecuritiesResponse, self._request("GET", "/api/v1/securities"))

    def get_security(self, symbol: str) -> catalogue.SecurityDetail:
        """GET /api/v1/securities/{symbol}."""
        return cast(
            catalogue.SecurityDetail, self._request("GET", f"/api/v1/securities/{_segment(symbol)}")
        )

    def place_order(
        self,
        *,
        token_id: str,
        side: str,
        price: Amount,
        shares: Amount,
        time_in_force: str = "GTC",
        post_only: bool = False,
        expiration_unix_seconds: int | None = None,
    ) -> models.CreateClobOrderResponse:
        """Place a LIMIT order with exact whole-unit decimal amounts."""
        return self.submit_order(
            _limit_body(
                token_id=token_id,
                side=side,
                price=price,
                shares=shares,
                time_in_force=time_in_force,
                post_only=post_only,
                expiration_unix_seconds=expiration_unix_seconds,
            )
        )

    def place_market_order(
        self,
        *,
        token_id: str,
        side: str,
        shares: Amount | None = None,
        collateral_amount: Amount | None = None,
        time_in_force: str = "FAK",
    ) -> models.CreateClobOrderResponse:
        """BUY spends collateral_amount; SELL sells shares. Acceptance is not completion."""
        return self.submit_order(
            _market_body(
                token_id=token_id,
                side=side,
                shares=shares,
                collateral_amount=collateral_amount,
                time_in_force=time_in_force,
            )
        )

    def place_signed_order(
        self,
        *,
        signed_order: SignedOrder,
        token_id: str,
        side: Literal["BUY", "SELL"],
        price_micro: int,
        shares_micro: int,
        time_in_force: str = "GTC",
        post_only: bool = False,
        expiration_unix_seconds: int | None = None,
    ) -> models.CreateClobOrderResponse:
        """Submit a locally signed order after verifying the request matches its economics."""
        return self.submit_signed_order(
            signed_order.to_request_body(
                token_id_string=token_id,
                side_string=side,
                price_micro=price_micro,
                shares_micro=shares_micro,
                time_in_force=time_in_force,
                post_only=post_only,
                expiration_unix_seconds=expiration_unix_seconds,
            )
        )

    def place_signed_orders(self, *, orders: list[SignedOrderEntry]) -> models.SignedOrdersResponse:
        """Submit 1–32 signed orders, preserving each nested acceptance or failure."""
        return self.submit_signed_orders([entry.to_request_body() for entry in orders])

    def wait_for_terminal(
        self, order_id: str, timeout: float = 30.0, poll_interval: float = 1.0
    ) -> models.Order:
        """Wait for is_terminal=true; trade settlement is a separate lifecycle. Raise on timeout."""
        deadline = _deadline(timeout, poll_interval)
        errors = 0
        while True:
            try:
                order = cast(
                    models.ClobOrderResponse,
                    self._request(
                        "GET", f"/trade/v1/orders/{_segment(order_id)}", deadline=deadline
                    ),
                )["order"]
                errors = 0
                if order.get("is_terminal") is True:
                    return order
                if not isinstance(order.get("is_terminal"), bool):
                    raise ProtocolError("order response omitted authoritative is_terminal")
                delay = poll_interval
            except AgaraError as exc:
                errors += 1
                if not exc.is_retryable or errors >= 3:
                    raise
                delay = max(poll_interval, exc.retry_after or 0)
            time.sleep(_remaining_sleep(deadline, delay))

    def wait_for_batch(
        self,
        batch_hash: str,
        timeout: float = 60.0,
        poll_interval: float = 1.0,
        *,
        follow_superseded: bool = True,
    ) -> models.AccountBatchStatusDto:
        """Wait for settlement or a completed failure; optionally follow replacement batches."""
        deadline = _deadline(timeout, poll_interval)
        visited = {batch_hash}
        while True:
            batch = cast(
                models.AccountBatchStatusDto,
                self._request(
                    "GET", f"/trade/v1/batches/{_segment(batch_hash)}", deadline=deadline
                ),
            )
            successor = batch["superseded_by_batch_hash"]
            if follow_superseded and successor:
                if successor in visited:
                    raise ProtocolError("batch successor cycle")
                visited.add(successor)
                batch_hash = successor
            elif batch_is_terminal(batch):
                return batch
            time.sleep(_remaining_sleep(deadline, poll_interval))

    def wait_for_batch_group(
        self, group_id: str, timeout: float = 60.0, poll_interval: float = 1.0
    ) -> models.BatchGroupStatusDto:
        """Wait for the group's completed_at marker, retaining per-chunk failures."""
        deadline = _deadline(timeout, poll_interval)
        while True:
            group = cast(
                models.BatchGroupStatusDto,
                self._request(
                    "GET", f"/trade/v1/batch-groups/{_segment(group_id)}", deadline=deadline
                ),
            )
            if group["completed_at"] is not None:
                return group
            time.sleep(_remaining_sleep(deadline, poll_interval))

    def follow_position_operation(
        self,
        operation: models.PositionOperationResponse,
        timeout: float = 60.0,
        poll_interval: float = 1.0,
    ) -> models.AccountBatchStatusDto | models.PositionOperationReceipt:
        """Follow an AGARA split/merge acceptance; retain the exchange-specific receipt otherwise."""
        if "batch_hash" in operation:
            return self.wait_for_batch(
                cast(models.PositionOperationAccepted, operation)["batch_hash"],
                timeout,
                poll_interval,
            )
        return operation

    def stream_prices(self, symbols: list[str]) -> Iterator[PriceEvent]:
        """Stream bounded, demand-driven multi-symbol Hermes price events; close the iterator to stop."""
        return iter_prices(
            self.session, self.base_url, self._headers, self.timeout, symbols=symbols
        )

    def stream_pyth_price(self, symbol: str) -> Iterator[PriceEvent]:
        """Stream one Pyth Pro symbol using its provider symbol, e.g. Crypto.BTC/USD."""
        return iter_prices(self.session, self.base_url, self._headers, self.timeout, symbol=symbol)

    def close(self) -> None:
        if self._owns_session:
            self.session.close()

    def __enter__(self) -> AgaraClient:
        return self

    def __exit__(self, *_exc: Any) -> None:
        self.close()


def _deadline(timeout: float, poll_interval: float) -> float:
    finite_positive(timeout, "timeout")
    finite_positive(poll_interval, "poll_interval")
    return time.monotonic() + timeout


def _remaining_sleep(deadline: float, requested: float) -> float:
    remaining = deadline - time.monotonic()
    if remaining <= 0:
        raise TimeoutError("operation did not complete before timeout")
    return min(remaining, requested)

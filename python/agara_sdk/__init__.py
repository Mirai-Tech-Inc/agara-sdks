"""Agara Trading API — minimal Python client.

See https://app.sandbox.agara.xyz/docs/sdks/python for usage.
Python 3.10+.
"""

from __future__ import annotations

import time
from dataclasses import dataclass
from typing import TYPE_CHECKING, Any, NoReturn, Optional

import requests

if TYPE_CHECKING:
    from agara_sdk.signing import SignedOrder, SignedOrderEntry


__all__ = [
    "AgaraClient",
    "AgaraError",
    "AuthError",
    "BadRequestError",
    "ForbiddenError",
    "NotFoundError",
    "ConflictError",
    "RejectedError",
    "RateLimitedError",
    "ServerError",
    "Orderbook",
    "OrderbookLevel",
    "DEFAULT_BASE_URL",
    "TERMINAL_STATUSES",
    "micro_to_float",
]

__version__ = "0.7.0"


DEFAULT_BASE_URL = "https://app.sandbox.agara.xyz"

MICRO = 1_000_000
TERMINAL_STATUSES = frozenset(
    {"MATCHED", "CANCELLED", "EXPIRED", "REJECTED", "FAILED"}
)

#: Consecutive transient 5xx tolerated inside `wait_for_terminal` before
#: it gives up. Shared by the sync and async clients.
_MAX_CONSECUTIVE_SERVER_ERRORS = 3

#: Page size the list-all helpers request per round-trip while walking the
#: server's keyset-cursor pagination. Matches the server's max page size, so a
#: full book is fetched in the fewest requests. Shared by the sync and async clients.
_LIST_PAGE_SIZE = 500


class AgaraError(Exception):
    """Base for every error this SDK raises."""

    def __init__(self, status_code: int, message: str, retry_after: Optional[float] = None):
        super().__init__(f"[{status_code}] {message}")
        self.status_code = status_code
        self.message = message
        #: Seconds the server asked the caller to wait before retrying, from the
        #: `Retry-After` (or `x-ratelimit-reset`) header. `None` when absent.
        self.retry_after = retry_after


class BadRequestError(AgaraError):
    """400 — malformed request body or invalid parameter values."""


class AuthError(AgaraError):
    """401 — missing / invalid / revoked / expired token."""


class ForbiddenError(AgaraError):
    """403 — token is valid but lacks the required scope. Re-issue
    with broader scopes from the FE; the bearer itself is fine."""


class NotFoundError(AgaraError):
    """404 — order, market, or token id doesn't exist or isn't yours."""


class ConflictError(AgaraError):
    """409 — e.g. cancelling an order already in a terminal state, or
    submitting a signed order whose `order_hash` already exists."""


class RejectedError(AgaraError):
    """422 — order rejected (insufficient balance / shares,
    FOK couldn't fill, post-only would cross, market halted)."""


class RateLimitedError(AgaraError):
    """429 — per-tier token bucket exhausted. `retry_after` carries the
    server's `Retry-After` hint (seconds) when present; back off and retry."""


class ServerError(AgaraError):
    """5xx — temporary platform problem. Safe to retry with backoff."""


@dataclass
class OrderbookLevel:
    price: float  # dollars per share (not micro)
    size: float   # shares (not micro-shares)


@dataclass
class Orderbook:
    bids: list[OrderbookLevel]
    asks: list[OrderbookLevel]
    timestamp: str
    hash: str
    tick_size: str

    @property
    def best_bid(self) -> Optional[float]:
        return self.bids[0].price if self.bids else None

    @property
    def best_ask(self) -> Optional[float]:
        return self.asks[0].price if self.asks else None

    @property
    def mid(self) -> Optional[float]:
        bb, ba = self.best_bid, self.best_ask
        return (bb + ba) / 2 if bb is not None and ba is not None else None

    @property
    def spread(self) -> Optional[float]:
        bb, ba = self.best_bid, self.best_ask
        return (ba - bb) if bb is not None and ba is not None else None


def _to_micro_str(value: float) -> str:
    """Dollars (or shares) → micro-encoded string for the wire."""
    return str(int(round(value * MICRO)))


def micro_to_float(value: str | int | None) -> Optional[float]:
    """Micro-encoded string (or int) → dollars (or shares).

    Returns None on None (handy for nullable response fields like
    `avg_fill_price_micro` before the first fill).
    """
    if value is None:
        return None
    return int(value) / MICRO


def _parse_retry_after(headers: Optional[Any]) -> Optional[float]:
    """Read the retry hint (seconds) from response headers. Prefers
    `Retry-After`, falls back to `x-ratelimit-reset`; both are emitted as
    integer seconds by the router. Returns None when absent or unparseable."""
    if headers is None:
        return None
    for key in ("retry-after", "x-ratelimit-reset"):
        raw = headers.get(key)
        if raw is None:
            continue
        try:
            return float(raw)
        except (TypeError, ValueError):
            continue
    return None


def _raise_api_error(
    status_code: int, message: str, headers: Optional[Any] = None
) -> NoReturn:
    """Map an HTTP status to the matching typed exception and raise it.
    Shared by the sync and async clients so the mapping lives once."""
    if status_code == 400:
        raise BadRequestError(status_code, message)
    if status_code == 401:
        raise AuthError(status_code, message)
    if status_code == 403:
        raise ForbiddenError(status_code, message)
    if status_code == 404:
        raise NotFoundError(status_code, message)
    if status_code == 409:
        raise ConflictError(status_code, message)
    if status_code == 422:
        raise RejectedError(status_code, message)
    if status_code == 429:
        raise RateLimitedError(status_code, message, _parse_retry_after(headers))
    if 500 <= status_code < 600:
        raise ServerError(status_code, message)
    raise AgaraError(status_code, message)


def _parse_orderbook(data: dict[str, Any]) -> Orderbook:
    return Orderbook(
        bids=[OrderbookLevel(lvl["price"], lvl["size"]) for lvl in data["bids"]],
        asks=[OrderbookLevel(lvl["price"], lvl["size"]) for lvl in data["asks"]],
        timestamp=data["timestamp"],
        hash=data["hash"],
        tick_size=data["tick_size"],
    )


def _build_limit_order_body(
    *,
    token_id: str,
    side: str,
    price: float,
    shares: Optional[float],
    collateral_amount: Optional[float],
    time_in_force: str,
    post_only: bool,
    expiration_unix_seconds: Optional[int],
) -> dict[str, Any]:
    if (shares is None) == (collateral_amount is None):
        raise ValueError("set exactly one of shares or collateral_amount")
    if collateral_amount is not None and side == "SELL":
        raise ValueError(
            "collateral_amount is only valid for BUY orders; use shares for SELL"
        )
    if time_in_force == "GTD" and expiration_unix_seconds is None:
        raise ValueError("GTD orders require expiration_unix_seconds")
    if price <= 0:
        raise ValueError("price must be > 0")
    if shares is not None and shares <= 0:
        raise ValueError("shares must be > 0")
    if collateral_amount is not None and collateral_amount <= 0:
        raise ValueError("collateral_amount must be > 0")

    body: dict[str, Any] = {
        "token_id": token_id,
        "side": side,
        "type": "LIMIT",
        "time_in_force": time_in_force,
        "price_micro": _to_micro_str(price),
        "post_only": post_only,
    }
    if shares is not None:
        body["shares_micro"] = _to_micro_str(shares)
    if collateral_amount is not None:
        body["collateral_amount_micro"] = _to_micro_str(collateral_amount)
    if expiration_unix_seconds is not None:
        body["expiration_unix_seconds"] = expiration_unix_seconds
    return body


def _build_market_order_body(
    *,
    token_id: str,
    side: str,
    shares: Optional[float],
    collateral_amount: Optional[float],
    time_in_force: str,
) -> dict[str, Any]:
    if side not in ("BUY", "SELL"):
        raise ValueError("side must be 'BUY' or 'SELL'")
    if time_in_force not in ("FAK", "FOK"):
        raise ValueError("market orders require FAK or FOK time_in_force")
    if side == "BUY":
        if collateral_amount is None:
            raise ValueError("market BUY orders require collateral_amount")
        if shares is not None:
            raise ValueError("market BUY orders must not set shares")
    else:
        if shares is None:
            raise ValueError("market SELL orders require shares")
        if collateral_amount is not None:
            raise ValueError("market SELL orders must not set collateral_amount")
    if shares is not None and shares <= 0:
        raise ValueError("shares must be > 0")
    if collateral_amount is not None and collateral_amount <= 0:
        raise ValueError("collateral_amount must be > 0")

    body: dict[str, Any] = {
        "token_id": token_id,
        "side": side,
        "type": "MARKET",
        "time_in_force": time_in_force,
        "post_only": False,
    }
    if shares is not None:
        body["shares_micro"] = _to_micro_str(shares)
    if collateral_amount is not None:
        body["collateral_amount_micro"] = _to_micro_str(collateral_amount)
    return body


class AgaraClient:
    """Trading API client.

    Pass a personal access token at construction. All methods are
    blocking; for an async-native equivalent backed by `httpx`, use
    `agara_sdk.aio.AsyncAgaraClient` (the `[async]` extra).

    Thread safety: each instance wraps a single `requests.Session`,
    which is **not safe to share across threads** — concurrent
    requests on the same client can interleave and surface as mixed-up
    responses. The fix is one client per thread (or wrap calls in your
    own lock). If you're running a single-threaded bot loop, you don't
    need to think about this.
    """

    def __init__(
        self,
        token: str,
        base_url: str = DEFAULT_BASE_URL,
        timeout: float = 10.0,
        session: Optional[requests.Session] = None,
    ):
        self.base_url = base_url.rstrip("/")
        self.timeout = timeout
        self.session = session or requests.Session()
        self.session.headers.update(
            {
                "Authorization": f"Bearer {token}",
                "Content-Type": "application/json",
                "Accept": "application/json",
            }
        )

    def _request(
        self,
        method: str,
        path: str,
        json: Optional[dict[str, Any]] = None,
        params: Optional[dict[str, Any]] = None,
    ) -> Any:
        url = f"{self.base_url}{path}"
        resp = self.session.request(
            method, url, json=json, params=params, timeout=self.timeout
        )

        if resp.ok:
            return resp.json() if resp.content else None

        # Body should be { "error": "..." }; fall back to status text if not.
        try:
            err_msg = resp.json().get("error", resp.reason or "")
        except ValueError:
            err_msg = resp.text or resp.reason or ""

        _raise_api_error(resp.status_code, err_msg, resp.headers)

    def get_orderbook(self, token_id: str) -> Orderbook:
        """Snapshot of bid/ask depth for one outcome."""
        data = self._request("GET", f"/trade/v1/orderbook/{token_id}")
        return _parse_orderbook(data)

    def place_order(
        self,
        *,
        token_id: str,
        side: str,                          # "BUY" or "SELL"
        price: float,                       # dollars per share
        shares: Optional[float] = None,
        collateral_amount: Optional[float] = None,  # dollars (BUY only)
        time_in_force: str = "GTC",         # "GTC" | "FAK" | "FOK" | "GTD"
        post_only: bool = False,
        expiration_unix_seconds: Optional[int] = None,
    ) -> dict[str, Any]:
        """Place a limit order. Returns the accepted-order ack —
        treat it as "we got it," not as a fill. Poll with `get_order`
        or `wait_for_terminal` to track progression."""
        body = _build_limit_order_body(
            token_id=token_id,
            side=side,
            price=price,
            shares=shares,
            collateral_amount=collateral_amount,
            time_in_force=time_in_force,
            post_only=post_only,
            expiration_unix_seconds=expiration_unix_seconds,
        )
        return self._request("POST", "/trade/v1/orders", json=body)

    def place_signed_order(
        self,
        *,
        token_id: str,
        side: str,                          # "BUY" or "SELL"
        price_micro: int,                   # μUSDC per share, in (0, 1_000_000)
        shares_micro: int,                  # μshares
        signed_order: "SignedOrder",        # from `agara_sdk.signing.sign_limit_order`
        time_in_force: str = "GTC",         # "GTC" only for v1
        post_only: bool = False,
        expiration_unix_seconds: Optional[int] = None,
    ) -> dict[str, Any]:
        """Place a pre-signed LIMIT order. The bot/server holds the
        EOA private key and signed `signed_order` locally — the router
        validates the signature against the wallet's EOA and skips
        Privy entirely. Scope: `orders:place_signed`."""
        body = signed_order.to_request_body(
            token_id_string=token_id,
            side_string=side,
            price_micro=price_micro,
            shares_micro=shares_micro,
            time_in_force=time_in_force,
            post_only=post_only,
            expiration_unix_seconds=expiration_unix_seconds,
        )
        return self._request("POST", "/trade/v1/orders/signed", json=body)

    def place_signed_orders(
        self,
        *,
        orders: "list[SignedOrderEntry]",   # up to 32, from `agara_sdk.signing`
    ) -> dict[str, Any]:
        """Place up to 32 pre-signed LIMIT orders in one call. Each order is
        validated and accepted independently: the response `results` array
        carries one entry per submitted order, in request order, each either
        `accepted` (with the same fields as `place_signed_order`) or `rejected`
        (with a `code` and `message`). A duplicate `order_hash` is reported
        `rejected` rather than failing the batch. Scope: `orders:place_signed`."""
        body = {"orders": [entry.to_request_body() for entry in orders]}
        return self._request("POST", "/trade/v1/orders/signed/batch", json=body)

    def place_market_order(
        self,
        *,
        token_id: str,
        side: str,                                  # "BUY" or "SELL"
        shares: Optional[float] = None,             # required for SELL
        collateral_amount: Optional[float] = None,  # required for BUY (USDC budget)
        time_in_force: str = "FAK",                 # "FAK" or "FOK"
    ) -> dict[str, Any]:
        """Place a market order.

        BUY: pass `collateral_amount` (USDC budget). The server walks
        the asks until the budget is exhausted, then signs a LIMIT BUY
        at the deepest level the walk consumed (or `max_price` if the
        book has no in-range asks). The chain envelope caps
        `makerAmount` at the budget so over-fills are impossible, and
        fills at earlier, better-priced levels pay out at those better
        prices on chain.

        SELL: pass `shares`. The server signs a LIMIT SELL at the
        market's `min_price` floor; fills against any bid above the
        floor pay out at the better bid price.

        FAK fills what's available and cancels the remainder; FOK
        rejects without placing if the order can't fully fill.
        Returns the accepted-order ack; poll with `get_order` to track
        fills."""
        body = _build_market_order_body(
            token_id=token_id,
            side=side,
            shares=shares,
            collateral_amount=collateral_amount,
            time_in_force=time_in_force,
        )
        return self._request("POST", "/trade/v1/orders", json=body)

    def list_orders(
        self,
        limit: int = 500,
        cursor: Optional[str] = None,
    ) -> dict[str, Any]:
        """List your orders, newest first — open and terminal — keyset-
        paginated. Returns the raw envelope `{"orders": [...],
        "pagination": {"next_cursor": ..., "limit": ...}, "as_of": ...}`.
        Omit `cursor` for the first page; pass the response's
        `pagination.next_cursor` back as `cursor` to walk the rest,
        stopping when it comes back `None`. Treat the cursor as opaque
        and keep `limit` the same across pages."""
        body: dict[str, Any] = {"limit": limit}
        if cursor is not None:
            body["cursor"] = cursor
        return self._request("POST", "/trade/v1/orders/list", json=body)

    def get_order(self, order_id: str) -> dict[str, Any]:
        """Look up one order by its internal UUID."""
        return self._request("GET", f"/trade/v1/orders/{order_id}")

    def get_order_by_hash(self, order_hash: str) -> dict[str, Any]:
        """Look up one order by its EIP-712 `order_hash`. For signed orders
        you know the hash before submitting, so this finds the order when
        you have the hash but not the `order_id` — you missed the placement
        response, got a `409` resubmitting, or are reconciling after a
        reconnect. Raises `NotFoundError` if no such order is yours. Scope:
        `orders:read`."""
        return self._request("GET", f"/trade/v1/orders/by-hash/{order_hash}")

    def get_order_trades(self, order_id: str) -> dict[str, Any]:
        """List the fills for one order, newest-first, from that order's
        perspective. Returns the raw envelope `{"trades": [...],
        "as_of": ...}`. Each trade carries `role` (`MAKER` or `TAKER`,
        or `null` for Polymarket orders — an order can be the maker on
        some fills and the taker on others), `side`, `price_micro`, and
        `fee_micro` for this order's leg of the fill, plus `fill_id` (the
        stream-to-REST join key; `null` for Polymarket fills). Unpaginated:
        an order's fills are returned in full. Scope: `orders:read`."""
        return self._request("GET", f"/trade/v1/orders/{order_id}/trades")

    def cancel_order(self, order_id: str) -> dict[str, Any]:
        """Cancel one order. Async — poll `get_order` until the
        status becomes `CANCELLED` to confirm."""
        return self._request("DELETE", f"/trade/v1/orders/{order_id}")

    def cancel_all_orders(self) -> dict[str, Any]:
        """Cancel every open order across all your wallets."""
        return self._request("POST", "/trade/v1/orders/cancel-all")

    def get_portfolio_summary(
        self,
        exchanges: Optional[list[str]] = None,
    ) -> list[dict[str, Any]]:
        """Per-exchange balance + positions value + open commitments.
        Returns one entry per backend you're onboarded on (`POLYMARKET`,
        `AGARA`) — separate USDC pools don't add coherently, so the
        server returns them side-by-side and lets the caller pick.

        `exchanges` restricts the fan-out (e.g. `["AGARA"]` skips the
        Polymarket round-trips entirely); `None` or `[]` queries every
        exchange you're onboarded on.
        Scope: `portfolio:read`."""
        params = {"exchanges": ",".join(exchanges)} if exchanges else None
        data = self._request("GET", "/trade/v1/portfolio/summary", params=params)
        return data["summaries"] if data else []

    def list_positions(
        self,
        condition_ids: Optional[list[str]] = None,
        exchanges: Optional[list[str]] = None,
    ) -> list[dict[str, Any]]:
        """Every current position across both backends, in one shot —
        the server returns the complete set, no pagination. Each row
        carries its own `exchange` so positions from token-id collisions
        across chains are still disambiguable.

        `condition_ids` filters server-side; pass `None` (or `[]`) for
        everything. `exchanges` restricts the fan-out (e.g. `["AGARA"]`
        skips Polymarket entirely); `None` or `[]` queries every
        exchange you're onboarded on.

        If a backend is unreachable the server returns the rest, naming the
        failed one in `unavailable_exchanges`. When you scoped to specific
        `exchanges`, a requested backend coming back unavailable raises
        `ServerError` rather than silently returning a partial/empty set — so
        you can't mistake "backend down" for "no positions".
        Scope: `portfolio:read`."""
        data = self._request(
            "POST",
            "/trade/v1/portfolio/positions/list",
            json={
                "condition_ids": condition_ids or [],
                "exchanges": exchanges or [],
            },
        )
        if not data:
            return []
        requested = exchanges or []
        blocked = [e for e in (data.get("unavailable_exchanges") or []) if e in requested]
        if blocked:
            raise ServerError(
                502, f"positions unavailable for requested exchange(s): {', '.join(blocked)}"
            )
        return data["positions"]

    def list_open_orders(
        self,
        token_ids: Optional[list[str]] = None,
        exchanges: Optional[list[str]] = None,
    ) -> list[dict[str, Any]]:
        """Every resting order across both backends, newest-first. Walks
        the server's keyset-cursor pagination internally and returns the
        complete set — a truncated snapshot would make a reconciler
        treat unseen live orders as stale. Distinct from `list_orders`,
        which also returns terminal orders; this returns only orders
        that could still fill.

        `exchanges` restricts the fan-out (e.g. `["AGARA"]` skips
        Polymarket entirely); `None` or `[]` queries every exchange you
        are onboarded on.
        Scope: `portfolio:read`."""
        orders: list[dict[str, Any]] = []
        cursor: Optional[str] = None
        while True:
            body: dict[str, Any] = {
                "token_ids": token_ids or [],
                "exchanges": exchanges or [],
                "limit": _LIST_PAGE_SIZE,
            }
            if cursor is not None:
                body["cursor"] = cursor
            data = self._request(
                "POST", "/trade/v1/portfolio/open-orders/list", json=body
            )
            if not data:
                break
            orders.extend(data.get("orders", []))
            next_cursor = (data.get("pagination") or {}).get("next_cursor")
            # `== cursor` guards a server that hands back a non-advancing
            # token; without an integer offset to compare, that's the
            # only stall we can detect.
            if next_cursor is None or next_cursor == cursor:
                break
            cursor = next_cursor
        return orders

    def list_activities(
        self,
        limit: Optional[int] = None,
        cursor: Optional[str] = None,
    ) -> dict[str, Any]:
        """Activity feed, newest-first — keyset-paginated. Like
        `list_trades` but each row also includes realized P&L (`null` on
        rows that don't realize — BUY / SPLIT / MERGE). Returns the raw
        envelope `{"activities": [...], "pagination": {"next_cursor": ...,
        "limit": ...}, "unavailable_exchanges": [...], "as_of": ...}`.
        Omit `cursor` for the first page; pass the response's
        `pagination.next_cursor` back as `cursor`, stopping when it comes
        back `None`. An empty `activities` page can still carry a non-null
        cursor, so keep paging until it is `None` — don't stop on an empty
        page. Scope: `portfolio:read`."""
        params: dict[str, Any] = {}
        if limit is not None:
            params["limit"] = limit
        if cursor is not None:
            params["cursor"] = cursor
        # `_request` returns None on a 204 / empty body; surface that as
        # an empty dict so callers can still do `data.get("activities", [])`.
        data = self._request(
            "GET", "/trade/v1/portfolio/activities", params=params or None
        )
        return data if data else {}

    def list_trades(
        self,
        limit: int = 500,
        cursor: Optional[str] = None,
    ) -> dict[str, Any]:
        """Recent fill history, newest-first — keyset-paginated. Returns
        the raw envelope `{"trades": [...], "pagination": {"next_cursor":
        ..., "limit": ...}, "unavailable_exchanges": [...], "as_of": ...}`.
        Omit `cursor` for the first page; pass the response's
        `pagination.next_cursor` back as `cursor` to walk the rest,
        stopping when it comes back `None`. An empty `trades` page can
        still carry a non-null cursor, so keep paging until it is `None`
        — don't stop on an empty page. Scope: `portfolio:read`."""
        params: dict[str, Any] = {"limit": limit}
        if cursor is not None:
            params["cursor"] = cursor
        # `_request` returns None on a 204 / empty body; surface that as
        # an empty dict so callers can still do `data.get("trades", [])`.
        data = self._request("GET", "/trade/v1/portfolio/trades", params=params)
        return data if data else {}

    def wait_for_terminal(
        self,
        order_id: str,
        timeout: float = 30.0,
        poll_interval: float = 1.0,
    ) -> dict[str, Any]:
        """Block until the order reaches a terminal status or timeout
        elapses. Returns the final order detail either way — callers
        should check `order["status"] in TERMINAL_STATUSES`.

        Transient 5xx during polling is retried up to 3 consecutive
        times before giving up — `wait_for_terminal` is meant to
        absorb routine flakes, not surface them. Other exception
        types still propagate immediately."""
        deadline = time.monotonic() + timeout
        consecutive_server_errors = 0
        while True:
            try:
                order = self.get_order(order_id)["order"]
                consecutive_server_errors = 0
            except ServerError:
                consecutive_server_errors += 1
                if consecutive_server_errors >= _MAX_CONSECUTIVE_SERVER_ERRORS:
                    raise
                if time.monotonic() >= deadline:
                    raise
                time.sleep(poll_interval)
                continue
            if order["status"] in TERMINAL_STATUSES:
                return order
            if time.monotonic() >= deadline:
                return order
            time.sleep(poll_interval)

    def close(self) -> None:
        self.session.close()

    def __enter__(self) -> "AgaraClient":
        return self

    def __exit__(self, *_exc: Any) -> None:
        self.close()

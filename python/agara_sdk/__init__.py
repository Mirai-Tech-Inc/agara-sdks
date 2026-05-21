"""Agara Trading API — minimal Python client.

See https://d3r180aqvl5ynd.cloudfront.net/docs/sdks/python for usage.
Python 3.10+.
"""

from __future__ import annotations

import time
from dataclasses import dataclass
from typing import Any, Optional

import requests


__all__ = [
    "AgaraClient",
    "AgaraError",
    "AuthError",
    "BadRequestError",
    "ForbiddenError",
    "NotFoundError",
    "ConflictError",
    "RejectedError",
    "ServerError",
    "Orderbook",
    "OrderbookLevel",
    "TERMINAL_STATUSES",
    "micro_to_float",
]

__version__ = "0.2.0"


MICRO = 1_000_000
TERMINAL_STATUSES = frozenset(
    {"CONFIRMED", "CANCELLED", "EXPIRED", "REJECTED", "FAILED"}
)


class AgaraError(Exception):
    """Base for every error this SDK raises."""

    def __init__(self, status_code: int, message: str):
        super().__init__(f"[{status_code}] {message}")
        self.status_code = status_code
        self.message = message


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
    """409 — e.g. cancelling an order already in a terminal state."""


class RejectedError(AgaraError):
    """422 — order rejected (insufficient balance / shares,
    FOK couldn't fill, post-only would cross, market halted)."""


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


class AgaraClient:
    """Trading API client.

    Pass a personal access token at construction. All methods are
    blocking; for async, use the OpenAPI spec at
    `/trade/v1/openapi.json` with an async generator.

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
        base_url: str = "https://d3r180aqvl5ynd.cloudfront.net",
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

        sc = resp.status_code
        if sc == 400:
            raise BadRequestError(sc, err_msg)
        if sc == 401:
            raise AuthError(sc, err_msg)
        if sc == 403:
            raise ForbiddenError(sc, err_msg)
        if sc == 404:
            raise NotFoundError(sc, err_msg)
        if sc == 409:
            raise ConflictError(sc, err_msg)
        if sc == 422:
            raise RejectedError(sc, err_msg)
        if 500 <= sc < 600:
            raise ServerError(sc, err_msg)
        raise AgaraError(sc, err_msg)

    def get_orderbook(self, token_id: str) -> Orderbook:
        """Snapshot of bid/ask depth for one outcome."""
        data = self._request("GET", f"/trade/v1/orderbook/{token_id}")
        return Orderbook(
            bids=[OrderbookLevel(lvl["price"], lvl["size"]) for lvl in data["bids"]],
            asks=[OrderbookLevel(lvl["price"], lvl["size"]) for lvl in data["asks"]],
            timestamp=data["timestamp"],
            hash=data["hash"],
            tick_size=data["tick_size"],
        )

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

        return self._request("POST", "/trade/v1/orders", json=body)

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

        BUY: pass `collateral_amount` (the USDC budget). The server
        translates to a LIMIT BUY at the worst-acceptable band edge for
        `budget / max_price` shares — the chain envelope caps the
        signed `makerAmount` at the budget, and the contract refunds
        any price improvement on fills below `max_price`.

        SELL: pass `shares`. The server translates to a LIMIT SELL at
        the min-price band edge for the requested shares; fills against
        any in-range bid pay out the better price.

        Market orders are FAK (fill what's available, cancel
        remainder) or FOK (fill entirely or reject). Returns the
        accepted-order ack; poll with `get_order` to track fills."""
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

        return self._request("POST", "/trade/v1/orders", json=body)

    def list_orders(
        self,
        limit: int = 100,
        offset: int = 0,
    ) -> dict[str, Any]:
        """List your orders, newest first. Returns both open and terminal."""
        return self._request(
            "POST",
            "/trade/v1/orders/list",
            json={"limit": limit, "offset": offset},
        )

    def get_order(self, order_id: str) -> dict[str, Any]:
        """Look up one order by its internal UUID."""
        return self._request("GET", f"/trade/v1/orders/{order_id}")

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
        limit: int = 100,
        offset: int = 0,
    ) -> list[dict[str, Any]]:
        """Your current positions, concatenated across both backends.
        Each row carries its own `exchange` so positions from token-id
        collisions across chains are still disambiguable.

        `condition_ids` filters server-side; pass `None` (or `[]`) for
        everything. `exchanges` restricts the fan-out (e.g. `["AGARA"]`
        skips Polymarket entirely); `None` or `[]` queries every
        exchange you're onboarded on. Pagination is applied per-backend
        before the merge, so very large position counts may need
        multiple calls.
        Scope: `portfolio:read`."""
        data = self._request(
            "POST",
            "/trade/v1/portfolio/positions/list",
            json={
                "condition_ids": condition_ids or [],
                "exchanges": exchanges or [],
                "limit": limit,
                "offset": offset,
            },
        )
        return data["positions"] if data else []

    def list_open_orders(
        self,
        token_ids: Optional[list[str]] = None,
        exchanges: Optional[list[str]] = None,
        limit: int = 100,
        offset: int = 0,
    ) -> list[dict[str, Any]]:
        """Resting orders across both backends, newest-first. Distinct
        from `list_orders` which returns *all* orders (including
        terminal); this returns only orders that could still fill.

        `exchanges` restricts the fan-out (e.g. `["AGARA"]` skips
        Polymarket entirely); `None` or `[]` queries every exchange you
        are onboarded on.
        Scope: `portfolio:read`."""
        data = self._request(
            "POST",
            "/trade/v1/portfolio/open-orders/list",
            json={
                "token_ids": token_ids or [],
                "exchanges": exchanges or [],
                "limit": limit,
                "offset": offset,
            },
        )
        return data["orders"] if data else []

    def list_activities(self, limit: Optional[int] = None) -> list[dict[str, Any]]:
        """Activity feed sorted newest-first. Like `list_trades` but
        each row also includes realized P&L (`null` on rows that don't
        realize — BUY / SPLIT / MERGE). Use this when you want the
        same stream the user sees in their browser activity tab.
        Scope: `portfolio:read`."""
        params = {"limit": limit} if limit is not None else None
        data = self._request("GET", "/trade/v1/portfolio/activities", params=params)
        return data["activities"] if data else []

    def list_trades(self) -> list[dict[str, Any]]:
        """Recent fill history. Sorted newest-first by the server."""
        # `_request` returns None on 204 / empty body; a fresh
        # account with no fills can legitimately come back that
        # way. Treat it as "no trades yet" rather than crashing.
        data = self._request("GET", "/trade/v1/portfolio/trades")
        return data["trades"] if data else []

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
                if consecutive_server_errors >= 3:
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

"""Async-native trading client, backed by `httpx`.

`AsyncAgaraClient` mirrors the blocking `agara_sdk.AgaraClient` method
for method — same arguments, same return shapes, same exception
hierarchy — but every call is a coroutine you `await`. Use it from an
asyncio event loop when you want concurrent RPCs without thread pools.

    pip install 'agara-sdk[async]'

    import asyncio
    from agara_sdk.aio import AsyncAgaraClient

    async def main():
        async with AsyncAgaraClient(token="agt_...") as client:
            book = await client.get_orderbook(token_id)
            resp = await client.place_order(
                token_id=token_id, side="BUY", price=0.60, shares=1.0,
            )
            final = await client.wait_for_terminal(resp["order_id"])

    asyncio.run(main())

Unlike the sync client's single `requests.Session` — which is not safe
to share across threads — one `AsyncAgaraClient` is safe to use from
many concurrent tasks on the same event loop; `httpx.AsyncClient`
multiplexes them over its connection pool.
"""

from __future__ import annotations

import asyncio
from typing import TYPE_CHECKING, Any, Optional

try:
    import httpx
except ImportError as exc:
    raise ImportError(
        "agara_sdk.aio requires the `httpx` extra: pip install 'agara-sdk[async]'"
    ) from exc

from agara_sdk import (
    DEFAULT_BASE_URL,
    TERMINAL_STATUSES,
    Orderbook,
    ServerError,
    _MAX_CONSECUTIVE_SERVER_ERRORS,
    _build_limit_order_body,
    _build_market_order_body,
    _parse_orderbook,
    _raise_api_error,
)

if TYPE_CHECKING:
    from agara_sdk.signing import SignedOrder


__all__ = ["AsyncAgaraClient"]


class AsyncAgaraClient:
    """Async trading API client. See module docstring for the usage
    pattern; every method below is the awaitable twin of the same-named
    method on `agara_sdk.AgaraClient`."""

    def __init__(
        self,
        token: str,
        base_url: str = DEFAULT_BASE_URL,
        timeout: float = 10.0,
        client: Optional[httpx.AsyncClient] = None,
    ):
        self.base_url = base_url.rstrip("/")
        self.timeout = timeout
        headers = {
            "Authorization": f"Bearer {token}",
            "Content-Type": "application/json",
            "Accept": "application/json",
        }
        if client is None:
            self._client = httpx.AsyncClient(timeout=timeout, headers=headers)
            self._owns_client = True
        else:
            client.headers.update(headers)
            self._client = client
            self._owns_client = False

    async def _request(
        self,
        method: str,
        path: str,
        json: Optional[dict[str, Any]] = None,
        params: Optional[dict[str, Any]] = None,
    ) -> Any:
        resp = await self._client.request(
            method, f"{self.base_url}{path}", json=json, params=params
        )

        if resp.is_success:
            return resp.json() if resp.content else None

        # Body should be { "error": "..." }; fall back to status text if not.
        try:
            err_msg = resp.json().get("error", resp.reason_phrase or "")
        except ValueError:
            err_msg = resp.text or resp.reason_phrase or ""

        _raise_api_error(resp.status_code, err_msg)

    async def get_orderbook(self, token_id: str) -> Orderbook:
        """Snapshot of bid/ask depth for one outcome."""
        data = await self._request("GET", f"/trade/v1/orderbook/{token_id}")
        return _parse_orderbook(data)

    async def place_order(
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
        """Place a limit order. Returns the accepted-order ack — treat
        it as "we got it," not as a fill. Poll with `get_order` or
        `wait_for_terminal` to track progression."""
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
        return await self._request("POST", "/trade/v1/orders", json=body)

    async def place_signed_order(
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
        """Place a pre-signed LIMIT order. The bot/server holds the EOA
        private key and signed `signed_order` locally — the router
        validates the signature against the wallet's EOA and skips Privy
        entirely. Scope: `orders:place_signed`."""
        body = signed_order.to_request_body(
            token_id_string=token_id,
            side_string=side,
            price_micro=price_micro,
            shares_micro=shares_micro,
            time_in_force=time_in_force,
            post_only=post_only,
            expiration_unix_seconds=expiration_unix_seconds,
        )
        return await self._request("POST", "/trade/v1/orders/signed", json=body)

    async def place_market_order(
        self,
        *,
        token_id: str,
        side: str,                                  # "BUY" or "SELL"
        shares: Optional[float] = None,             # required for SELL
        collateral_amount: Optional[float] = None,  # required for BUY (USDC budget)
        time_in_force: str = "FAK",                 # "FAK" or "FOK"
    ) -> dict[str, Any]:
        """Place a market order. BUY takes `collateral_amount` (a USDC
        budget the server walks the asks against); SELL takes `shares`.
        FAK fills what's available and cancels the rest; FOK rejects
        without placing if it can't fully fill. Returns the
        accepted-order ack; poll with `get_order` to track fills."""
        body = _build_market_order_body(
            token_id=token_id,
            side=side,
            shares=shares,
            collateral_amount=collateral_amount,
            time_in_force=time_in_force,
        )
        return await self._request("POST", "/trade/v1/orders", json=body)

    async def list_orders(self, limit: int = 100, offset: int = 0) -> dict[str, Any]:
        """List your orders, newest first. Returns both open and terminal."""
        return await self._request(
            "POST",
            "/trade/v1/orders/list",
            json={"limit": limit, "offset": offset},
        )

    async def get_order(self, order_id: str) -> dict[str, Any]:
        """Look up one order by its internal UUID."""
        return await self._request("GET", f"/trade/v1/orders/{order_id}")

    async def cancel_order(self, order_id: str) -> dict[str, Any]:
        """Cancel one order. Async on the engine too — poll `get_order`
        until the status becomes `CANCELLED` to confirm."""
        return await self._request("DELETE", f"/trade/v1/orders/{order_id}")

    async def cancel_all_orders(self) -> dict[str, Any]:
        """Cancel every open order across all your wallets."""
        return await self._request("POST", "/trade/v1/orders/cancel-all")

    async def get_portfolio_summary(
        self,
        exchanges: Optional[list[str]] = None,
    ) -> list[dict[str, Any]]:
        """Per-exchange balance + positions value + open commitments.
        `exchanges` restricts the fan-out (e.g. `["AGARA"]`); `None` or
        `[]` queries every exchange you're onboarded on.
        Scope: `portfolio:read`."""
        params = {"exchanges": ",".join(exchanges)} if exchanges else None
        data = await self._request("GET", "/trade/v1/portfolio/summary", params=params)
        return data["summaries"] if data else []

    async def list_positions(
        self,
        condition_ids: Optional[list[str]] = None,
        exchanges: Optional[list[str]] = None,
        limit: int = 100,
        offset: int = 0,
    ) -> list[dict[str, Any]]:
        """Your current positions, concatenated across both backends.
        `condition_ids` filters server-side; `exchanges` restricts the
        fan-out. `None` or `[]` means "everything"/"every exchange".
        Scope: `portfolio:read`."""
        data = await self._request(
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

    async def list_open_orders(
        self,
        token_ids: Optional[list[str]] = None,
        exchanges: Optional[list[str]] = None,
        limit: int = 100,
        offset: int = 0,
    ) -> list[dict[str, Any]]:
        """Resting orders across both backends, newest-first. Distinct
        from `list_orders`, which also returns terminal orders; this
        returns only orders that could still fill.
        Scope: `portfolio:read`."""
        data = await self._request(
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

    async def list_activities(
        self, limit: Optional[int] = None
    ) -> list[dict[str, Any]]:
        """Activity feed, newest-first — like `list_trades` but each row
        also carries realized P&L (`null` on BUY / SPLIT / MERGE rows).
        Scope: `portfolio:read`."""
        params = {"limit": limit} if limit is not None else None
        data = await self._request(
            "GET", "/trade/v1/portfolio/activities", params=params
        )
        return data["activities"] if data else []

    async def list_trades(self) -> list[dict[str, Any]]:
        """Recent fill history, newest-first."""
        data = await self._request("GET", "/trade/v1/portfolio/trades")
        return data["trades"] if data else []

    async def wait_for_terminal(
        self,
        order_id: str,
        timeout: float = 30.0,
        poll_interval: float = 1.0,
    ) -> dict[str, Any]:
        """Await until the order reaches a terminal status or `timeout`
        elapses. Returns the final order detail either way — callers
        should check `order["status"] in TERMINAL_STATUSES`.

        Transient 5xx during polling is retried up to 3 consecutive
        times before giving up; other exceptions propagate immediately."""
        loop = asyncio.get_running_loop()
        deadline = loop.time() + timeout
        consecutive_server_errors = 0
        while True:
            try:
                order = (await self.get_order(order_id))["order"]
                consecutive_server_errors = 0
            except ServerError:
                consecutive_server_errors += 1
                if consecutive_server_errors >= _MAX_CONSECUTIVE_SERVER_ERRORS:
                    raise
                if loop.time() >= deadline:
                    raise
                await asyncio.sleep(poll_interval)
                continue
            if order["status"] in TERMINAL_STATUSES:
                return order
            if loop.time() >= deadline:
                return order
            await asyncio.sleep(poll_interval)

    async def aclose(self) -> None:
        """Close the underlying connection pool. A no-op when the client
        was constructed with an injected `httpx.AsyncClient` — the caller
        owns that one's lifecycle."""
        if self._owns_client:
            await self._client.aclose()

    async def __aenter__(self) -> "AsyncAgaraClient":
        return self

    async def __aexit__(self, *_exc: Any) -> None:
        await self.aclose()

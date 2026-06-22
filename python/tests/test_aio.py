"""Smoke tests for the async-native AsyncAgaraClient.

We inject an `httpx.MockTransport` so the network never happens and
assert against the request the client *would* have made — same grain
as the sync `test_client.py`, just over `httpx` instead of `requests`.
The body-building + status-mapping logic is shared with the sync
client, so these tests focus on the async dispatch path: awaiting,
header propagation, response unwrapping, and `wait_for_terminal`'s
poll loop.
"""

from __future__ import annotations

import json as _json
from typing import Callable

import httpx
import pytest

from agara_sdk import (
    AgaraError,
    AuthError,
    BadRequestError,
    ConflictError,
    ForbiddenError,
    NotFoundError,
    RateLimitedError,
    RejectedError,
    ServerError,
    TERMINAL_STATUSES,
)
from agara_sdk.aio import AsyncAgaraClient


BASE_URL = "https://api.example.test"
TOKEN = "agt_test_token"
TOKEN_ID = "21742633143463906290569050155826241533067272736897614950488156847949938836455"

Handler = Callable[[httpx.Request], httpx.Response]


def _client(handler: Handler) -> AsyncAgaraClient:
    transport = httpx.MockTransport(handler)
    injected = httpx.AsyncClient(transport=transport)
    return AsyncAgaraClient(token=TOKEN, base_url=BASE_URL, client=injected)


@pytest.fixture
def no_sleep(monkeypatch: pytest.MonkeyPatch) -> None:
    """Make `wait_for_terminal`'s poll delay instant so the loop tests
    don't actually sleep between polls."""

    async def _instant(_seconds: float) -> None:
        return None

    monkeypatch.setattr("agara_sdk.aio.asyncio.sleep", _instant)


@pytest.mark.asyncio
async def test_authorization_header_is_set_on_every_request() -> None:
    seen: dict[str, str] = {}

    def handler(request: httpx.Request) -> httpx.Response:
        seen["auth"] = request.headers.get("Authorization", "")
        return httpx.Response(200, json={"order": {"id": "abc", "status": "OPEN"}})

    async with _client(handler) as client:
        await client.get_order("abc")

    assert seen["auth"] == f"Bearer {TOKEN}"


@pytest.mark.asyncio
async def test_get_order_trades_hits_subresource_path() -> None:
    seen: dict[str, str] = {}

    def handler(request: httpx.Request) -> httpx.Response:
        seen["path"] = request.url.path
        return httpx.Response(
            200,
            json={
                "trades": [{"trade_id": "t1", "role": "TAKER", "side": "BUY"}],
                "as_of": "2026-06-13T00:00:00Z",
            },
        )

    async with _client(handler) as client:
        result = await client.get_order_trades("abc")

    assert seen["path"] == "/trade/v1/orders/abc/trades"
    assert result["trades"][0]["role"] == "TAKER"


@pytest.mark.asyncio
async def test_get_orderbook_returns_typed_dataclass() -> None:
    def handler(request: httpx.Request) -> httpx.Response:
        assert request.url.path == f"/trade/v1/orderbook/{TOKEN_ID}"
        return httpx.Response(
            200,
            json={
                "bids": [{"price": 0.60, "size": 100.0}],
                "asks": [{"price": 0.62, "size": 80.0}],
                "timestamp": "2026-05-12T10:23:45.678Z",
                "hash": "1234",
                "tick_size": "0.01",
            },
        )

    async with _client(handler) as client:
        book = await client.get_orderbook(TOKEN_ID)

    assert book.best_bid == 0.60
    assert book.best_ask == 0.62
    assert book.mid == 0.61
    assert book.tick_size == "0.01"


@pytest.mark.asyncio
async def test_place_order_encodes_dollars_to_micro_strings() -> None:
    seen: dict = {}

    def handler(request: httpx.Request) -> httpx.Response:
        assert request.method == "POST"
        assert request.url.path == "/trade/v1/orders"
        seen["body"] = _json.loads(request.content)
        return httpx.Response(202, json={"order_id": "ord-1", "status": "PENDING_NEW"})

    async with _client(handler) as client:
        resp = await client.place_order(
            token_id=TOKEN_ID, side="BUY", price=0.60, shares=1.0
        )

    assert resp["order_id"] == "ord-1"
    assert seen["body"] == {
        "token_id": TOKEN_ID,
        "side": "BUY",
        "type": "LIMIT",
        "time_in_force": "GTC",
        "price_micro": "600000",
        "shares_micro": "1000000",
        "post_only": False,
    }


@pytest.mark.asyncio
async def test_place_market_order_buy_sends_collateral() -> None:
    seen: dict = {}

    def handler(request: httpx.Request) -> httpx.Response:
        seen["body"] = _json.loads(request.content)
        return httpx.Response(202, json={"order_id": "ord-mk-1"})

    async with _client(handler) as client:
        await client.place_market_order(
            token_id=TOKEN_ID, side="BUY", collateral_amount=5.0
        )

    assert seen["body"]["type"] == "MARKET"
    assert seen["body"]["time_in_force"] == "FAK"
    assert seen["body"]["collateral_amount_micro"] == "5000000"


@pytest.mark.asyncio
async def test_validation_errors_raise_before_dispatch() -> None:
    called = False

    def handler(request: httpx.Request) -> httpx.Response:
        nonlocal called
        called = True
        return httpx.Response(202, json={})

    async with _client(handler) as client:
        with pytest.raises(ValueError, match="exactly one"):
            await client.place_order(token_id=TOKEN_ID, side="BUY", price=0.5)

    assert called is False


@pytest.mark.parametrize(
    "status, exc",
    [
        (400, BadRequestError),
        (401, AuthError),
        (403, ForbiddenError),
        (404, NotFoundError),
        (409, ConflictError),
        (422, RejectedError),
        (429, RateLimitedError),
        (500, ServerError),
        (503, ServerError),
    ],
)
@pytest.mark.asyncio
async def test_http_status_maps_to_typed_exception(status: int, exc: type) -> None:
    def handler(request: httpx.Request) -> httpx.Response:
        return httpx.Response(status, json={"error": "boom"})

    async with _client(handler) as client:
        with pytest.raises(exc) as info:
            await client.get_order("abc")

    assert info.value.status_code == status
    assert info.value.message == "boom"
    assert isinstance(info.value, AgaraError)


@pytest.mark.asyncio
async def test_rate_limited_carries_retry_after() -> None:
    def handler(request: httpx.Request) -> httpx.Response:
        return httpx.Response(
            429, json={"error": "rate_limited"}, headers={"Retry-After": "7"}
        )

    async with _client(handler) as client:
        with pytest.raises(RateLimitedError) as info:
            await client.get_order("abc")

    assert info.value.retry_after == 7.0


@pytest.mark.asyncio
async def test_list_trades_returns_empty_dict_when_body_is_empty() -> None:
    def handler(request: httpx.Request) -> httpx.Response:
        return httpx.Response(204)

    async with _client(handler) as client:
        assert await client.list_trades() == {}


@pytest.mark.asyncio
async def test_list_trades_returns_envelope() -> None:
    def handler(request: httpx.Request) -> httpx.Response:
        assert request.url.params.get("limit") == "500"
        return httpx.Response(
            200,
            json={
                "trades": [{"id": "t1"}],
                "pagination": {"next_cursor": "c", "limit": 500},
                "unavailable_exchanges": [],
            },
        )

    async with _client(handler) as client:
        resp = await client.list_trades()

    assert resp["trades"][0]["id"] == "t1"
    assert resp["pagination"]["next_cursor"] == "c"


@pytest.mark.asyncio
async def test_list_orders_threads_cursor() -> None:
    seen: dict = {}

    def handler(request: httpx.Request) -> httpx.Response:
        seen["body"] = _json.loads(request.content)
        return httpx.Response(
            200,
            json={
                "orders": [],
                "pagination": {"next_cursor": None, "limit": 250},
                "as_of": "2026-06-13T00:00:00Z",
            },
        )

    async with _client(handler) as client:
        resp = await client.list_orders(limit=250, cursor="abc")

    assert seen["body"] == {"limit": 250, "cursor": "abc"}
    assert resp["pagination"]["next_cursor"] is None


@pytest.mark.asyncio
async def test_list_positions_unwraps_and_sends_filters() -> None:
    seen: dict = {}

    def handler(request: httpx.Request) -> httpx.Response:
        seen["body"] = _json.loads(request.content)
        return httpx.Response(200, json={"positions": [{"id": "p1"}]})

    async with _client(handler) as client:
        positions = await client.list_positions(
            condition_ids=["0xabc"], exchanges=["AGARA"]
        )

    assert [p["id"] for p in positions] == ["p1"]
    assert seen["body"] == {
        "condition_ids": ["0xabc"],
        "exchanges": ["AGARA"],
    }


@pytest.mark.asyncio
async def test_list_open_orders_walks_every_page() -> None:
    bodies: list[dict] = []

    def handler(request: httpx.Request) -> httpx.Response:
        body = _json.loads(request.content)
        bodies.append(body)
        if "cursor" not in body:
            return httpx.Response(
                200,
                json={"orders": [{"id": "o1"}], "pagination": {"next_cursor": "cur-2", "limit": 500}},
            )
        return httpx.Response(
            200,
            json={"orders": [{"id": "o2"}], "pagination": {"next_cursor": None, "limit": 500}},
        )

    async with _client(handler) as client:
        orders = await client.list_open_orders()

    assert [o["id"] for o in orders] == ["o1", "o2"]
    # Page 1 sends no cursor; page 2 echoes the page-1 token.
    assert "cursor" not in bodies[0]
    assert bodies[1]["cursor"] == "cur-2"


@pytest.mark.asyncio
async def test_wait_for_terminal_polls_until_terminal(no_sleep: None) -> None:
    statuses = ["OPEN", "OPEN", "CONFIRMED"]
    calls = 0

    def handler(request: httpx.Request) -> httpx.Response:
        nonlocal calls
        status = statuses[min(calls, len(statuses) - 1)]
        calls += 1
        return httpx.Response(200, json={"order": {"id": "abc", "status": status}})

    async with _client(handler) as client:
        result = await client.wait_for_terminal("abc", timeout=10.0, poll_interval=1.0)

    assert result["status"] == "CONFIRMED"
    assert result["status"] in TERMINAL_STATUSES
    assert calls == 3


@pytest.mark.asyncio
async def test_wait_for_terminal_raises_after_three_consecutive_server_errors(
    no_sleep: None,
) -> None:
    def handler(request: httpx.Request) -> httpx.Response:
        return httpx.Response(502, json={"error": "boom"})

    async with _client(handler) as client:
        with pytest.raises(ServerError) as info:
            await client.wait_for_terminal("abc", timeout=10.0, poll_interval=0.1)

    assert info.value.status_code == 502


@pytest.mark.asyncio
async def test_injected_client_is_not_closed_by_aclose() -> None:
    def handler(request: httpx.Request) -> httpx.Response:
        return httpx.Response(200, json={"order": {"status": "OPEN"}})

    injected = httpx.AsyncClient(transport=httpx.MockTransport(handler))
    client = AsyncAgaraClient(token=TOKEN, base_url=BASE_URL, client=injected)
    await client.aclose()

    # The caller owns the injected client's lifecycle, so it stays open.
    assert injected.is_closed is False
    await injected.aclose()

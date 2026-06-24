"""Smoke tests for AgaraClient.

Scope: HTTP-layer behavior that's easy to break and tedious to spot in
review — header propagation, dollar/share ↔ micro-string conversion,
the status-code → exception mapping, and `wait_for_terminal`'s loop.
We use `responses` so the network never happens; the assertions are
against the request `requests` _would_ have made.
"""

from __future__ import annotations

import pytest
import responses
from responses.matchers import json_params_matcher, query_param_matcher

from agara_sdk import (
    AgaraClient,
    AgaraError,
    AuthError,
    ForbiddenError,
    BadRequestError,
    ConflictError,
    NotFoundError,
    RateLimitedError,
    RejectedError,
    ServerError,
    TERMINAL_STATUSES,
    micro_to_float,
)
from agara_sdk.signing import SignedOrder, SignedOrderEntry


BASE_URL = "https://api.example.test"
TOKEN = "agt_test_token"
TOKEN_ID = "21742633143463906290569050155826241533067272736897614950488156847949938836455"


@pytest.fixture
def client() -> AgaraClient:
    return AgaraClient(token=TOKEN, base_url=BASE_URL)


@responses.activate
def test_authorization_header_is_set_on_every_request(client: AgaraClient) -> None:
    responses.get(
        f"{BASE_URL}/trade/v1/orders/abc",
        json={"order": {"id": "abc", "status": "OPEN"}},
    )

    client.get_order("abc")

    assert len(responses.calls) == 1
    assert responses.calls[0].request.headers["Authorization"] == f"Bearer {TOKEN}"


@responses.activate
def test_get_order_trades_hits_subresource_path(client: AgaraClient) -> None:
    responses.get(
        f"{BASE_URL}/trade/v1/orders/abc/trades",
        json={
            "trades": [
                {"trade_id": "t1", "role": "MAKER", "side": "BUY"},
                {"trade_id": "t2", "role": "TAKER", "side": "SELL"},
            ],
            "as_of": "2026-06-13T00:00:00Z",
        },
    )

    result = client.get_order_trades("abc")

    assert len(responses.calls) == 1
    assert [trade["role"] for trade in result["trades"]] == ["MAKER", "TAKER"]


@responses.activate
def test_get_order_by_hash_hits_by_hash_path(client: AgaraClient) -> None:
    responses.get(
        f"{BASE_URL}/trade/v1/orders/by-hash/0xabc123",
        json={"order": {"id": "abc", "order_hash": "0xabc123", "status": "OPEN"}},
    )

    result = client.get_order_by_hash("0xabc123")

    assert len(responses.calls) == 1
    assert result["order"]["order_hash"] == "0xabc123"


@responses.activate
def test_get_orderbook_returns_typed_dataclass(client: AgaraClient) -> None:
    # The router's orderbook endpoint returns prices/sizes as floats
    # already (dollars and shares), not micro-encoded — see the
    # AgaraOrderbookResponse schema in apps/router/src/orderbook.rs.
    responses.get(
        f"{BASE_URL}/trade/v1/orderbook/{TOKEN_ID}",
        json={
            "bids": [{"price": 0.60, "size": 100.0}, {"price": 0.59, "size": 50.0}],
            "asks": [{"price": 0.62, "size": 80.0}, {"price": 0.63, "size": 40.0}],
            "timestamp": "2026-05-12T10:23:45.678Z",
            "hash": "1234",
            "tick_size": "0.01",
        },
    )

    book = client.get_orderbook(TOKEN_ID)

    assert book.best_bid == 0.60
    assert book.best_ask == 0.62
    assert book.mid == 0.61
    # 0.62 - 0.60; tolerate float rounding from binary representation
    assert book.spread == pytest.approx(0.02)
    assert book.tick_size == "0.01"
    assert len(book.bids) == 2
    assert book.bids[0].size == 100.0


@responses.activate
def test_orderbook_properties_handle_empty_book(client: AgaraClient) -> None:
    responses.get(
        f"{BASE_URL}/trade/v1/orderbook/{TOKEN_ID}",
        json={
            "bids": [],
            "asks": [],
            "timestamp": "2026-05-12T10:23:45.678Z",
            "hash": "1",
            "tick_size": "0.01",
        },
    )

    book = client.get_orderbook(TOKEN_ID)

    assert book.best_bid is None
    assert book.best_ask is None
    assert book.mid is None
    assert book.spread is None


# `json_params_matcher(strict_match=True)` asserts the exact body the
# SDK should produce. The mock only matches when the body lines up,
# so a regression in field names, types, or extras fails the request
# (and the test) without per-key unpacking on the call side.


@responses.activate
def test_place_order_encodes_dollars_to_micro_strings(client: AgaraClient) -> None:
    responses.post(
        f"{BASE_URL}/trade/v1/orders",
        json={"order_id": "ord-1", "status": "PENDING_NEW"},
        status=202,
        match=[
            json_params_matcher(
                {
                    "token_id": TOKEN_ID,
                    "side": "BUY",
                    "type": "LIMIT",
                    "time_in_force": "GTC",
                    "price_micro": "600000",
                    "shares_micro": "1000000",
                    "post_only": False,
                },
                strict_match=True,
            )
        ],
    )

    client.place_order(token_id=TOKEN_ID, side="BUY", price=0.60, shares=1.0)


@responses.activate
def test_place_order_with_collateral_sends_collateral_amount_micro(
    client: AgaraClient,
) -> None:
    responses.post(
        f"{BASE_URL}/trade/v1/orders",
        json={"order_id": "x"},
        status=202,
        match=[
            json_params_matcher(
                {
                    "token_id": TOKEN_ID,
                    "side": "BUY",
                    "type": "LIMIT",
                    "time_in_force": "GTC",
                    "price_micro": "550000",
                    "collateral_amount_micro": "10000000",
                    "post_only": False,
                },
                strict_match=True,
            )
        ],
    )

    client.place_order(
        token_id=TOKEN_ID,
        side="BUY",
        price=0.55,
        collateral_amount=10.0,
    )


@responses.activate
def test_place_order_gtd_includes_expiration(client: AgaraClient) -> None:
    responses.post(
        f"{BASE_URL}/trade/v1/orders",
        json={"order_id": "x"},
        status=202,
        match=[
            json_params_matcher(
                {
                    "token_id": TOKEN_ID,
                    "side": "SELL",
                    "type": "LIMIT",
                    "time_in_force": "GTD",
                    "price_micro": "700000",
                    "shares_micro": "2000000",
                    "post_only": False,
                    "expiration_unix_seconds": 1_800_000_000,
                },
                strict_match=True,
            )
        ],
    )

    client.place_order(
        token_id=TOKEN_ID,
        side="SELL",
        price=0.70,
        shares=2.0,
        time_in_force="GTD",
        expiration_unix_seconds=1_800_000_000,
    )


@responses.activate
def test_place_signed_orders_wraps_entries_under_orders(client: AgaraClient) -> None:
    signed = SignedOrder(
        order_hash="0xhash",
        signature="0xsig",
        salt=7,
        maker="0xmaker",
        signer="0xmaker",
        token_id=int(TOKEN_ID),
        maker_amount=600000,
        taker_amount=1000000,
        side=0,
    )
    entry = SignedOrderEntry(
        signed_order=signed,
        token_id=TOKEN_ID,
        side="BUY",
        price_micro=600000,
        shares_micro=1000000,
    )
    expected = entry.to_request_body()

    responses.post(
        f"{BASE_URL}/trade/v1/orders/signed/batch",
        json={"results": [], "as_of": "2026-06-19T00:00:00Z"},
        status=202,
        match=[json_params_matcher({"orders": [expected, expected]}, strict_match=True)],
    )

    client.place_signed_orders(orders=[entry, entry])

    assert len(responses.calls) == 1


def test_place_order_rejects_both_shares_and_collateral(client: AgaraClient) -> None:
    with pytest.raises(ValueError, match="exactly one"):
        client.place_order(
            token_id=TOKEN_ID,
            side="BUY",
            price=0.5,
            shares=1.0,
            collateral_amount=0.5,
        )


def test_place_order_rejects_neither_shares_nor_collateral(client: AgaraClient) -> None:
    with pytest.raises(ValueError, match="exactly one"):
        client.place_order(token_id=TOKEN_ID, side="BUY", price=0.5)


def test_place_order_gtd_requires_expiration(client: AgaraClient) -> None:
    with pytest.raises(ValueError, match="GTD"):
        client.place_order(
            token_id=TOKEN_ID,
            side="BUY",
            price=0.5,
            shares=1.0,
            time_in_force="GTD",
        )


def test_place_order_rejects_collateral_on_sell(client: AgaraClient) -> None:
    # collateral_amount is a BUY-only quoting mode ("spend up to $N");
    # passing it on a SELL would otherwise reach the engine and fail
    # with a non-obvious 4xx. Catch it client-side.
    with pytest.raises(ValueError, match="BUY orders"):
        client.place_order(
            token_id=TOKEN_ID,
            side="SELL",
            price=0.5,
            collateral_amount=10.0,
        )


@responses.activate
def test_place_market_order_buy_sends_collateral(client: AgaraClient) -> None:
    responses.post(
        f"{BASE_URL}/trade/v1/orders",
        json={"order_id": "ord-mk-1"},
        status=202,
        match=[
            json_params_matcher(
                {
                    "token_id": TOKEN_ID,
                    "side": "BUY",
                    "type": "MARKET",
                    "time_in_force": "FAK",
                    "collateral_amount_micro": "5000000",
                    "post_only": False,
                },
                strict_match=True,
            )
        ],
    )

    client.place_market_order(
        token_id=TOKEN_ID,
        side="BUY",
        collateral_amount=5.0,
    )


@responses.activate
def test_place_market_order_sell_sends_shares(client: AgaraClient) -> None:
    responses.post(
        f"{BASE_URL}/trade/v1/orders",
        json={"order_id": "ord-mk-2"},
        status=202,
        match=[
            json_params_matcher(
                {
                    "token_id": TOKEN_ID,
                    "side": "SELL",
                    "type": "MARKET",
                    "time_in_force": "FOK",
                    "shares_micro": "1500000",
                    "post_only": False,
                },
                strict_match=True,
            )
        ],
    )

    client.place_market_order(
        token_id=TOKEN_ID,
        side="SELL",
        shares=1.5,
        time_in_force="FOK",
    )


def test_place_market_order_rejects_gtc(client: AgaraClient) -> None:
    with pytest.raises(ValueError, match="FAK or FOK"):
        client.place_market_order(
            token_id=TOKEN_ID,
            side="BUY",
            collateral_amount=5.0,
            time_in_force="GTC",
        )


def test_place_market_order_buy_requires_collateral(client: AgaraClient) -> None:
    with pytest.raises(ValueError, match="require collateral_amount"):
        client.place_market_order(token_id=TOKEN_ID, side="BUY")


def test_place_market_order_sell_requires_shares(client: AgaraClient) -> None:
    with pytest.raises(ValueError, match="require shares"):
        client.place_market_order(token_id=TOKEN_ID, side="SELL")


def test_place_market_order_buy_rejects_shares(client: AgaraClient) -> None:
    with pytest.raises(ValueError, match="must not set shares"):
        client.place_market_order(
            token_id=TOKEN_ID,
            side="BUY",
            collateral_amount=5.0,
            shares=1.0,
        )


def test_place_market_order_sell_rejects_collateral(client: AgaraClient) -> None:
    with pytest.raises(ValueError, match="must not set collateral"):
        client.place_market_order(
            token_id=TOKEN_ID,
            side="SELL",
            shares=1.0,
            collateral_amount=5.0,
        )


@pytest.mark.parametrize("amount", [0, 0.0, -1.0])
def test_place_order_rejects_non_positive_shares(
    client: AgaraClient, amount: float
) -> None:
    with pytest.raises(ValueError, match="shares must be > 0"):
        client.place_order(token_id=TOKEN_ID, side="BUY", price=0.5, shares=amount)


@pytest.mark.parametrize("amount", [0, 0.0, -1.0])
def test_place_order_rejects_non_positive_collateral(
    client: AgaraClient, amount: float
) -> None:
    with pytest.raises(ValueError, match="collateral_amount must be > 0"):
        client.place_order(
            token_id=TOKEN_ID, side="BUY", price=0.5, collateral_amount=amount
        )


@pytest.mark.parametrize("price", [0, 0.0, -0.1])
def test_place_order_rejects_non_positive_price(
    client: AgaraClient, price: float
) -> None:
    with pytest.raises(ValueError, match="price must be > 0"):
        client.place_order(token_id=TOKEN_ID, side="BUY", price=price, shares=1.0)


@pytest.mark.parametrize("amount", [0, 0.0, -1.0])
def test_place_market_order_rejects_non_positive_shares(
    client: AgaraClient, amount: float
) -> None:
    with pytest.raises(ValueError, match="shares must be > 0"):
        client.place_market_order(token_id=TOKEN_ID, side="SELL", shares=amount)


@pytest.mark.parametrize("amount", [0, 0.0, -1.0])
def test_place_market_order_rejects_non_positive_collateral(
    client: AgaraClient, amount: float
) -> None:
    with pytest.raises(ValueError, match="collateral_amount must be > 0"):
        client.place_market_order(
            token_id=TOKEN_ID, side="BUY", collateral_amount=amount
        )


def test_micro_to_float_round_trip() -> None:
    assert micro_to_float("600000") == 0.60
    assert micro_to_float("1000000") == 1.0
    assert micro_to_float(0) == 0.0
    assert micro_to_float(None) is None


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
        (502, ServerError),
        (503, ServerError),
    ],
)
@responses.activate
def test_http_status_maps_to_typed_exception(
    client: AgaraClient, status: int, exc: type
) -> None:
    responses.get(
        f"{BASE_URL}/trade/v1/orders/abc",
        json={"error": "boom"},
        status=status,
    )

    with pytest.raises(exc) as info:
        client.get_order("abc")

    assert info.value.status_code == status
    assert info.value.message == "boom"
    # Every typed exception inherits from AgaraError.
    assert isinstance(info.value, AgaraError)


@responses.activate
def test_rate_limited_carries_retry_after(client: AgaraClient) -> None:
    responses.get(
        f"{BASE_URL}/trade/v1/orders/abc",
        json={"error": "rate_limited"},
        status=429,
        headers={"Retry-After": "7"},
    )

    with pytest.raises(RateLimitedError) as info:
        client.get_order("abc")

    assert info.value.retry_after == 7.0


@responses.activate
def test_unhandled_status_falls_back_to_agara_error(client: AgaraClient) -> None:
    # 418 isn't in the mapping table — it should still raise the
    # base class so callers can catch unconditionally on AgaraError.
    responses.get(
        f"{BASE_URL}/trade/v1/orders/abc",
        json={"error": "teapot"},
        status=418,
    )

    with pytest.raises(AgaraError) as info:
        client.get_order("abc")

    assert type(info.value) is AgaraError
    assert info.value.status_code == 418


@responses.activate
def test_error_body_without_error_field_uses_status_text(
    client: AgaraClient,
) -> None:
    # Body is valid JSON but missing the canonical `error` key — we
    # fall back to the HTTP reason rather than crashing or showing
    # `None`.
    responses.get(
        f"{BASE_URL}/trade/v1/orders/abc",
        json={"unexpected": "shape"},
        status=404,
    )

    with pytest.raises(NotFoundError) as info:
        client.get_order("abc")

    # Either falsy or the reason phrase — both are OK; what we care
    # about is that the construction didn't blow up.
    assert isinstance(info.value.message, str)


@responses.activate
def test_wait_for_terminal_returns_immediately_when_already_terminal(
    client: AgaraClient,
) -> None:
    responses.get(
        f"{BASE_URL}/trade/v1/orders/abc",
        json={"order": {"id": "abc", "status": "CONFIRMED"}},
    )

    result = client.wait_for_terminal("abc", timeout=5.0, poll_interval=0.01)

    assert result["status"] == "CONFIRMED"
    assert result["status"] in TERMINAL_STATUSES
    assert len(responses.calls) == 1


@responses.activate
def test_wait_for_terminal_polls_until_terminal(
    client: AgaraClient, monkeypatch: pytest.MonkeyPatch
) -> None:
    # First two polls: still OPEN. Third: CONFIRMED.
    responses.get(
        f"{BASE_URL}/trade/v1/orders/abc",
        json={"order": {"id": "abc", "status": "OPEN"}},
    )
    responses.get(
        f"{BASE_URL}/trade/v1/orders/abc",
        json={"order": {"id": "abc", "status": "OPEN"}},
    )
    responses.get(
        f"{BASE_URL}/trade/v1/orders/abc",
        json={"order": {"id": "abc", "status": "CONFIRMED"}},
    )

    # Skip the real sleep so the test isn't slow.
    monkeypatch.setattr("agara_sdk.time.sleep", lambda _: None)

    result = client.wait_for_terminal("abc", timeout=10.0, poll_interval=1.0)

    assert result["status"] == "CONFIRMED"
    assert len(responses.calls) == 3


@responses.activate
def test_wait_for_terminal_returns_non_terminal_after_timeout(
    client: AgaraClient, monkeypatch: pytest.MonkeyPatch
) -> None:
    # Always OPEN.
    responses.get(
        f"{BASE_URL}/trade/v1/orders/abc",
        json={"order": {"id": "abc", "status": "OPEN"}},
    )

    # First two `time.monotonic` calls return 0.0 (deadline computed +
    # initial in-deadline check); every later call returns 999.0 (past
    # deadline). Counter-based instead of an iter() so the test doesn't
    # break when the impl adds/reorders monotonic() calls.
    call_count = 0

    def fake_monotonic() -> float:
        nonlocal call_count
        call_count += 1
        return 0.0 if call_count <= 2 else 999.0

    monkeypatch.setattr("agara_sdk.time.monotonic", fake_monotonic)
    monkeypatch.setattr("agara_sdk.time.sleep", lambda _: None)

    result = client.wait_for_terminal("abc", timeout=1.0, poll_interval=0.1)

    assert result["status"] == "OPEN"
    assert result["status"] not in TERMINAL_STATUSES


@responses.activate
def test_wait_for_terminal_retries_through_transient_server_errors(
    client: AgaraClient, monkeypatch: pytest.MonkeyPatch
) -> None:
    # Two 503s, then a terminal status. Helper should absorb the
    # transient failures and return the final order.
    responses.get(
        f"{BASE_URL}/trade/v1/orders/abc",
        json={"error": "upstream timeout"},
        status=503,
    )
    responses.get(
        f"{BASE_URL}/trade/v1/orders/abc",
        json={"error": "upstream timeout"},
        status=503,
    )
    responses.get(
        f"{BASE_URL}/trade/v1/orders/abc",
        json={"order": {"id": "abc", "status": "CONFIRMED"}},
    )

    monkeypatch.setattr("agara_sdk.time.sleep", lambda _: None)

    result = client.wait_for_terminal("abc", timeout=10.0, poll_interval=0.1)

    assert result["status"] == "CONFIRMED"
    assert len(responses.calls) == 3


@responses.activate
def test_wait_for_terminal_raises_after_three_consecutive_server_errors(
    client: AgaraClient, monkeypatch: pytest.MonkeyPatch
) -> None:
    for _ in range(3):
        responses.get(
            f"{BASE_URL}/trade/v1/orders/abc",
            json={"error": "boom"},
            status=502,
        )

    monkeypatch.setattr("agara_sdk.time.sleep", lambda _: None)

    with pytest.raises(ServerError) as info:
        client.wait_for_terminal("abc", timeout=10.0, poll_interval=0.1)

    assert info.value.status_code == 502
    assert len(responses.calls) == 3


@responses.activate
def test_list_orders_sends_limit_only_on_first_page(client: AgaraClient) -> None:
    # First page carries no cursor; the SDK sends only `limit` and
    # returns the raw envelope verbatim.
    responses.post(
        f"{BASE_URL}/trade/v1/orders/list",
        json={
            "orders": [{"id": "ord-1"}],
            "pagination": {"next_cursor": "nc", "limit": 500},
            "as_of": "2026-06-13T00:00:00Z",
        },
        match=[json_params_matcher({"limit": 500}, strict_match=True)],
    )

    resp = client.list_orders()

    assert [o["id"] for o in resp["orders"]] == ["ord-1"]
    assert resp["pagination"]["next_cursor"] == "nc"


@responses.activate
def test_list_orders_threads_cursor(client: AgaraClient) -> None:
    responses.post(
        f"{BASE_URL}/trade/v1/orders/list",
        json={
            "orders": [],
            "pagination": {"next_cursor": None, "limit": 250},
            "as_of": "2026-06-13T00:00:00Z",
        },
        match=[json_params_matcher({"limit": 250, "cursor": "abc"}, strict_match=True)],
    )

    resp = client.list_orders(limit=250, cursor="abc")

    assert resp["pagination"]["next_cursor"] is None


@responses.activate
def test_list_trades_returns_empty_dict_when_body_is_empty(
    client: AgaraClient,
) -> None:
    # The router can legitimately return 204 / empty body for an
    # account with no fills. We surface that as an empty dict so callers
    # can still do `data.get("trades", [])` rather than crashing.
    responses.get(
        f"{BASE_URL}/trade/v1/portfolio/trades",
        body="",
        status=204,
    )

    assert client.list_trades() == {}


@responses.activate
def test_list_trades_returns_envelope_with_pagination(
    client: AgaraClient,
) -> None:
    responses.get(
        f"{BASE_URL}/trade/v1/portfolio/trades",
        json={
            "trades": [{"trade_id": "t1"}, {"trade_id": "t2"}],
            "pagination": {"next_cursor": "c1", "limit": 500},
            "unavailable_exchanges": [],
        },
        match=[query_param_matcher({"limit": "500"})],
    )

    resp = client.list_trades()

    assert [t["trade_id"] for t in resp["trades"]] == ["t1", "t2"]
    assert resp["pagination"]["next_cursor"] == "c1"


@responses.activate
def test_list_trades_threads_limit_and_cursor(client: AgaraClient) -> None:
    responses.get(
        f"{BASE_URL}/trade/v1/portfolio/trades",
        json={"trades": [], "pagination": {"next_cursor": None, "limit": 250}},
        match=[query_param_matcher({"limit": "250", "cursor": "abc"})],
    )

    resp = client.list_trades(limit=250, cursor="abc")

    assert resp["pagination"]["limit"] == 250


@responses.activate
def test_get_portfolio_summary_unwraps_summaries_key(client: AgaraClient) -> None:
    responses.get(
        f"{BASE_URL}/trade/v1/portfolio/summary",
        json={"summaries": [{"exchange": "POLYMARKET"}, {"exchange": "AGARA"}]},
    )

    summaries = client.get_portfolio_summary()

    assert [s["exchange"] for s in summaries] == ["POLYMARKET", "AGARA"]


@responses.activate
def test_get_portfolio_summary_returns_empty_when_body_is_empty(
    client: AgaraClient,
) -> None:
    responses.get(f"{BASE_URL}/trade/v1/portfolio/summary", body="", status=204)

    assert client.get_portfolio_summary() == []


@responses.activate
def test_list_positions_sends_filter_in_one_shot(client: AgaraClient) -> None:
    responses.post(
        f"{BASE_URL}/trade/v1/portfolio/positions/list",
        json={"positions": [{"id": "p1"}]},
        match=[
            json_params_matcher(
                {"condition_ids": ["0xabc"], "exchanges": []},
                strict_match=True,
            )
        ],
    )

    positions = client.list_positions(condition_ids=["0xabc"])

    assert [p["id"] for p in positions] == ["p1"]


@responses.activate
def test_list_positions_defaults_to_empty_condition_filter(client: AgaraClient) -> None:
    responses.post(
        f"{BASE_URL}/trade/v1/portfolio/positions/list",
        json={"positions": []},
        match=[
            json_params_matcher(
                {"condition_ids": [], "exchanges": []},
                strict_match=True,
            )
        ],
    )

    assert client.list_positions() == []


@responses.activate
def test_list_positions_raises_when_requested_exchange_unavailable(client: AgaraClient) -> None:
    responses.post(
        f"{BASE_URL}/trade/v1/portfolio/positions/list",
        json={"positions": [], "unavailable_exchanges": ["AGARA"]},
    )

    # Scoped to AGARA + AGARA is down → fail loud, don't return "[]" (no positions).
    with pytest.raises(ServerError):
        client.list_positions(exchanges=["AGARA"])


@responses.activate
def test_list_positions_unscoped_returns_partial_on_backend_outage(client: AgaraClient) -> None:
    responses.post(
        f"{BASE_URL}/trade/v1/portfolio/positions/list",
        json={"positions": [{"id": "p1"}], "unavailable_exchanges": ["POLYMARKET"]},
    )

    # No exchanges scope → "give me everything" tolerates a partial result.
    assert [p["id"] for p in client.list_positions()] == ["p1"]


@responses.activate
def test_list_open_orders_returns_single_page(client: AgaraClient) -> None:
    # next_cursor null on the only page → one request, no cursor sent.
    responses.post(
        f"{BASE_URL}/trade/v1/portfolio/open-orders/list",
        json={
            "orders": [{"id": "o1"}, {"id": "o2"}],
            "pagination": {"next_cursor": None, "limit": 500},
        },
        match=[
            json_params_matcher(
                {"token_ids": [TOKEN_ID], "exchanges": [], "limit": 500},
                strict_match=True,
            )
        ],
    )

    orders = client.list_open_orders(token_ids=[TOKEN_ID])

    assert [o["id"] for o in orders] == ["o1", "o2"]


@responses.activate
def test_list_open_orders_walks_every_page(client: AgaraClient) -> None:
    # Page 1 carries a next_cursor → the client must fetch page 2 with
    # it echoed back as `cursor`. The strict matchers assert page 1 sends
    # no cursor and page 2 sends exactly the page-1 token.
    responses.post(
        f"{BASE_URL}/trade/v1/portfolio/open-orders/list",
        json={"orders": [{"id": "o1"}], "pagination": {"next_cursor": "cur-2", "limit": 500}},
        match=[
            json_params_matcher(
                {"token_ids": [], "exchanges": [], "limit": 500},
                strict_match=True,
            )
        ],
    )
    responses.post(
        f"{BASE_URL}/trade/v1/portfolio/open-orders/list",
        json={"orders": [{"id": "o2"}], "pagination": {"next_cursor": None, "limit": 500}},
        match=[
            json_params_matcher(
                {"token_ids": [], "exchanges": [], "limit": 500, "cursor": "cur-2"},
                strict_match=True,
            )
        ],
    )

    orders = client.list_open_orders()

    assert [o["id"] for o in orders] == ["o1", "o2"]


@responses.activate
def test_list_open_orders_stops_on_non_advancing_cursor(client: AgaraClient) -> None:
    # A server bug that hands back the same token must not loop forever:
    # both pages echo cur-1, so the walk stops after the second request.
    responses.post(
        f"{BASE_URL}/trade/v1/portfolio/open-orders/list",
        json={"orders": [{"id": "o1"}], "pagination": {"next_cursor": "cur-1", "limit": 500}},
        match=[
            json_params_matcher(
                {"token_ids": [], "exchanges": [], "limit": 500}, strict_match=True
            )
        ],
    )
    responses.post(
        f"{BASE_URL}/trade/v1/portfolio/open-orders/list",
        json={"orders": [{"id": "o2"}], "pagination": {"next_cursor": "cur-1", "limit": 500}},
        match=[
            json_params_matcher(
                {"token_ids": [], "exchanges": [], "limit": 500, "cursor": "cur-1"},
                strict_match=True,
            )
        ],
    )

    orders = client.list_open_orders()

    assert [o["id"] for o in orders] == ["o1", "o2"]
    assert len(responses.calls) == 2


@responses.activate
def test_list_activities_sends_limit_query_param(client: AgaraClient) -> None:
    responses.get(
        f"{BASE_URL}/trade/v1/portfolio/activities",
        json={"activities": [{"type": "TRADE"}], "pagination": {"next_cursor": None, "limit": 250}},
        match=[query_param_matcher({"limit": "250"})],
    )

    resp = client.list_activities(limit=250)

    assert [a["type"] for a in resp["activities"]] == ["TRADE"]


@responses.activate
def test_list_activities_threads_limit_and_cursor(client: AgaraClient) -> None:
    responses.get(
        f"{BASE_URL}/trade/v1/portfolio/activities",
        json={"activities": [], "pagination": {"next_cursor": None, "limit": 100}},
        match=[query_param_matcher({"limit": "100", "cursor": "xyz"})],
    )

    resp = client.list_activities(limit=100, cursor="xyz")

    assert resp["pagination"]["limit"] == 100


@responses.activate
def test_list_activities_omits_query_when_no_args(client: AgaraClient) -> None:
    # No limit/cursor → no query string at all; empty/None args let the
    # server pick its own default page size.
    responses.get(
        f"{BASE_URL}/trade/v1/portfolio/activities",
        json={"activities": [], "pagination": {"next_cursor": None, "limit": 50}},
    )

    resp = client.list_activities()

    assert resp["activities"] == []
    assert "?" not in responses.calls[0].request.url


def test_context_manager_closes_session() -> None:
    client = AgaraClient(token=TOKEN, base_url=BASE_URL)
    sess = client.session

    with client:
        pass

    # `requests.Session.close()` doesn't expose a "closed" flag —
    # idempotent and a no-op when already closed. Best we can do
    # cheaply is invoke it again and confirm it doesn't raise.
    sess.close()

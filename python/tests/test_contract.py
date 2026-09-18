"""Every selected operation is exercised through both real HTTP transports, offline."""

from __future__ import annotations

import asyncio
import inspect
import json
import types
from decimal import Decimal, localcontext
from pathlib import Path
from typing import Any, Literal, Union, get_args, get_origin
from unittest.mock import Mock
from urllib.parse import parse_qs, urlsplit

import httpx
import pytest
import requests
import responses
from typing_extensions import NotRequired, Required, get_type_hints

from agara_sdk import (
    AgaraClient,
    AgaraError,
    AuthError,
    ProtocolError,
    TransportError,
    micro_to_decimal,
    to_micro,
)
from agara_sdk.aio import AsyncAgaraClient

ROOT = Path(__file__).resolve().parents[1]
OPS = [
    op
    for op in json.loads((ROOT / "contracts/trader-manifest.json").read_text())["operations"]
    if op["protocol"] == "HTTP"
]
BASE = "https://offline.invalid"
ORDER = {
    "token_id": "123",
    "side": "BUY",
    "type": "LIMIT",
    "time_in_force": "GTC",
    "price_micro": "550000",
    "shares_micro": "2000000",
    "post_only": False,
}
SIGNED = {
    **ORDER,
    "salt": "1",
    "maker": "0x" + "11" * 20,
    "chain_token_id": "123",
    "maker_amount": "1100000",
    "taker_amount": "2000000",
    "side_u8": 0,
    "timestamp": "0",
    "metadata": "0x" + "00" * 32,
    "builder": "0x" + "00" * 32,
    "order_hash": "0x" + "22" * 32,
    "signature": "0x" + "33" * 65,
}
BATCH = {
    "ops": [
        {
            "kind": "MERGE",
            "market_id": "00000000-0000-4000-8000-000000000001",
            "condition_id": "0x" + "44" * 32,
            "shares_micro": 1000000,
        }
    ],
    "seq": 4,
    "deadline_unix_seconds": 1900000000,
    "signature": "0x" + "33" * 65,
}
DEPOSIT = {
    "from_chain_id": "1",
    "from_token_address": "0x" + "44" * 20,
    "from_amount_base_unit": "1000000",
}
WITHDRAW = {
    "to_chain_id": "137",
    "to_token_address": "0x" + "44" * 20,
    "recipient_address": "0x" + "55" * 20,
    "from_amount_base_unit": "1000000",
}
SPECIAL = {
    "submit_order": {"order": ORDER},
    "submit_signed_order": {"order": SIGNED},
    "submit_signed_orders": {"orders": [SIGNED]},
    "submit_batch": {"batch": BATCH},
    "supersede_batch": {
        "batch_hash": "0x" + "22" * 32,
        "batch": {k: v for k, v in BATCH.items() if k != "seq"},
    },
    "get_deposit_quote": {"request": DEPOSIT},
    "get_withdraw_quote": {"request": WITHDRAW},
    "get_portfolio_summary": {"exchanges": ["AGARA", "POLYMARKET"]},
    "list_positions": {"condition_ids": ["condition"], "exchanges": ["AGARA"]},
    "list_open_orders": {
        "token_ids": ["123"],
        "exchanges": ["AGARA"],
        "limit": 4,
        "cursor": "opaque",
    },
    "split_position": {"condition_id": "condition", "collateral_amount_micro": 1234567},
    "merge_position": {"condition_id": "condition", "shares_micro": 1234567},
    "get_realized_pnl": {"granularity": "day", "window": "7d"},
    "list_events": {
        "category": "crypto",
        "root": "finance",
        "event_type_bucket": "props",
        "filter": "sports_futures",
        "source": "AGARA",
        "exclude_ended": False,
        "resolution": "active",
        "sort": "time",
        "cursor": "opaque",
        "limit": 4,
        "include_markets": True,
    },
    "list_markets": {
        "source": "agara",
        "event_slug": "btc",
        "state": "ACTIVE",
        "cursor": "opaque",
        "limit": 4,
    },
}
VALUES = {
    "order_id": "00000000-0000-4000-8000-000000000001",
    "order_hash": "0x" + "22" * 32,
    "batch_hash": "0x" + "22" * 32,
    "group_id": "00000000-0000-4000-8000-000000000002",
    "token_id": "123",
    "mic": "XNYS",
    "date": "2026-09-17",
    "slug": "btc",
    "market_id": "00000000-0000-4000-8000-000000000003",
    "symbol": "BTC-USD",
    "provider": "pyth-pro",
    "from_": "2026-09-01",
    "to": "2026-09-17",
    "limit": 4,
    "cursor": "opaque",
    "page": 2,
    "category": "crypto",
    "search": "Bitcoin",
    "sort_by": "date",
    "sort_order": "desc",
    "q": "Bitcoin",
    "source": "AGARA",
    "range": "1d",
    "points": 32,
    "at": 1784022000,
}


def arguments(name):
    if name in SPECIAL:
        return SPECIAL[name]
    kwargs = {
        key: VALUES[key]
        for key in inspect.signature(getattr(AgaraClient, name)).parameters
        if key != "self"
    }
    if name == "get_price_ticks":
        kwargs.update(from_=1784021000, to=1784022000)
    if name == "list_lp_incentives":
        kwargs["sort_by"] = "reward_pool"
    return kwargs


def wire_expected(op, kwargs):
    path = op["path"]
    for key in [
        "order_id",
        "order_hash",
        "batch_hash",
        "group_id",
        "token_id",
        "mic",
        "date",
        "slug",
        "market_id",
        "symbol",
    ]:
        path = path.replace(
            "{" + ("id" if key == "market_id" else key) + "}", str(kwargs.get(key, ""))
        )
    name = op["python_method"]
    pathkeys = {field.split("}")[0] for field in op["path"].split("{")[1:]}
    data = {
        ("from" if k == "from_" else k): v
        for k, v in kwargs.items()
        if k not in pathkeys and not (k == "market_id" and "id" in pathkeys)
    }
    if name in {"submit_order", "submit_signed_order"}:
        data = kwargs["order"]
    elif name in {"submit_batch", "supersede_batch"}:
        data = kwargs["batch"]
    elif name in {"get_deposit_quote", "get_withdraw_quote"}:
        data = kwargs["request"]
    elif name in {"split_position", "merge_position"}:
        data = {k: (str(v) if k.endswith("_micro") else v) for k, v in data.items()}
    if op["method"] == "GET":
        if name == "get_portfolio_summary":
            data = {"exchanges": ",".join(kwargs["exchanges"])}
        data = {k: str(v).lower() if isinstance(v, bool) else str(v) for k, v in data.items()}
    return path, data


def sample(typ):
    origin = get_origin(typ)
    if origin in (NotRequired, Required):
        return sample(get_args(typ)[0])
    if origin in (types.UnionType, Union):
        return sample(get_args(typ)[0])
    if origin is Literal:
        return get_args(typ)[0]
    if origin is list:
        return []
    if origin is dict:
        return {}
    if typ is Any:
        return {}
    if typ is str:
        return "wire-value"
    if typ is int:
        return 1
    if typ is float:
        return 0.5
    if typ is bool:
        return True
    if typ is type(None):
        return None
    if hasattr(typ, "__required_keys__"):
        return {k: sample(t) for k, t in get_type_hints(typ).items()}
    raise AssertionError(typ)


@pytest.mark.parametrize("op", OPS, ids=lambda op: op["operation_id"])
@responses.activate
def test_every_sync_operation(op):
    kwargs = arguments(op["python_method"])
    path, data = wire_expected(op, kwargs)
    payload = sample(get_type_hints(getattr(AgaraClient, op["python_method"]))["return"])
    payload["future_extension"] = {"preserved": True}
    responses.add(op["method"], BASE + path, json=payload, status=200)
    with AgaraClient("agt_test", base_url=BASE) as client:
        actual = getattr(client, op["python_method"])(**kwargs)
    assert actual == payload
    sent = responses.calls[0].request
    assert sent.headers["Authorization"] == "Bearer agt_test"
    if op["method"] == "GET":
        assert parse_qs(urlsplit(sent.url).query) == {k: [v] for k, v in data.items()}
    elif data:
        assert json.loads(sent.body) == data
    else:
        assert sent.body is None


@pytest.mark.asyncio
@pytest.mark.parametrize("op", OPS, ids=lambda op: op["operation_id"])
async def test_every_async_operation(op):
    kwargs = arguments(op["python_method"])
    path, data = wire_expected(op, kwargs)
    payload = sample(get_type_hints(getattr(AgaraClient, op["python_method"]))["return"])
    payload["future_extension"] = {"preserved": True}

    def handler(request):
        assert request.method == op["method"]
        assert request.url.path == path
        assert request.headers["Authorization"] == "Bearer agt_test"
        if op["method"] == "GET":
            assert dict(request.url.params) == data
        elif data:
            assert json.loads(request.content) == data
        else:
            assert not request.content
        return httpx.Response(200, json=payload)

    async with httpx.AsyncClient(transport=httpx.MockTransport(handler)) as transport:
        async with AsyncAgaraClient("agt_test", base_url=BASE, client=transport) as client:
            assert await getattr(client, op["python_method"])(**kwargs) == payload
        assert not transport.is_closed


def test_complete_manifest_and_sync_async_signature_parity():
    assert len(OPS) == 53
    for name in {op["python_method"] for op in OPS} | {
        "place_order",
        "place_market_order",
        "wait_for_terminal",
        "wait_for_batch",
        "wait_for_batch_group",
        "follow_position_operation",
    }:
        assert inspect.signature(getattr(AgaraClient, name)) == inspect.signature(
            getattr(AsyncAgaraClient, name)
        )


@pytest.mark.parametrize(
    "value,expected",
    [
        ("9007199254.740993", 9007199254740993),
        (Decimal("0.000001"), 1),
        (2, 2000000),
        ("-0.05", -50000),
    ],
)
def test_amounts_are_exact_with_arbitrary_decimal_context(value, expected):
    with localcontext() as ctx:
        ctx.prec = 3
        assert to_micro(value) == expected
        assert micro_to_decimal(str(expected)) == Decimal(str(value))


@pytest.mark.parametrize("bad", [0.1, True, "NaN", "Infinity", "0.0000005", "9223372036854.775808"])
def test_amounts_reject_silent_rounding(bad):
    with pytest.raises((TypeError, ValueError)):
        to_micro(bad)


@pytest.mark.parametrize(
    "side,size", [("BUY", {"collateral_amount": "10.123456"}), ("SELL", {"shares": "2.5"})]
)
@responses.activate
def test_market_fak_both_directions(side, size):
    responses.post(
        BASE + "/trade/v1/orders",
        json={
            "order_id": "id",
            "source": "AGARA",
            "status": "PENDING",
            "pending_operation": "SUBMIT",
            "as_of": "2026-09-17T00:00:00Z",
        },
    )
    AgaraClient("token", BASE).place_market_order(token_id="123", side=side, **size)
    wire = json.loads(responses.calls[0].request.body)
    assert wire["time_in_force"] == "FAK"
    assert wire["type"] == "MARKET"
    assert "price_micro" not in wire
    key = "collateral_amount_micro" if side == "BUY" else "shares_micro"
    assert wire[key] == ("10123456" if side == "BUY" else "2500000")


@responses.activate
def test_limit_budget_option_removed_and_exact_shares():
    client = AgaraClient("token", BASE)
    with pytest.raises(TypeError):
        client.place_order(token_id="123", side="BUY", price="0.55", collateral_amount="10")
    with pytest.raises(ValueError):
        client.place_order(token_id="123", side="BUY", price="0.9999996", shares="1")
    assert not responses.calls


@pytest.mark.parametrize("bad", [float("nan"), float("inf"), 0, -1, True])
def test_invalid_timeout_rejected(bad):
    with pytest.raises(ValueError):
        AgaraClient(timeout=bad)
    with pytest.raises(ValueError):
        AsyncAgaraClient(timeout=bad)


@pytest.mark.parametrize(
    "bad", ["ws://host", "host", "https://u:p@host", "https://host?a=b", "https://host#x"]
)
def test_invalid_base_url_rejected(bad):
    with pytest.raises(ValueError):
        AgaraClient(base_url=bad)
    with pytest.raises(ValueError):
        AsyncAgaraClient(base_url=bad)


def test_injected_session_ownership_and_token_privacy():
    session = requests.Session()
    session.close = Mock()
    with AgaraClient("secret", BASE, session=session):
        pass
    session.close.assert_not_called()
    assert "Authorization" not in session.headers
    with pytest.raises(AuthError):
        AgaraClient(base_url=BASE).get_order("id")


@pytest.mark.asyncio
async def test_async_timeout_and_cancellation_do_not_retry_mutations():
    calls = []

    async def handler(request):
        calls.append(request)
        await asyncio.sleep(10)
        return httpx.Response(200, json={})

    async with httpx.AsyncClient(transport=httpx.MockTransport(handler)) as transport:
        client = AsyncAgaraClient("token", BASE, timeout=0.01, client=transport)
        with pytest.raises((TimeoutError, TransportError)):
            await client.submit_order(ORDER)
        task = asyncio.create_task(client.submit_order(ORDER))
        await asyncio.sleep(0)
        task.cancel()
        with pytest.raises(asyncio.CancelledError):
            await task
    assert len(calls) <= 2


@responses.activate
def test_transport_write_ambiguity_and_no_retry():
    responses.post(BASE + "/trade/v1/orders", body=requests.Timeout("lost ack"))
    with pytest.raises(TransportError) as caught:
        AgaraClient("token", BASE).submit_order(ORDER)
    assert caught.value.mutation_outcome_unknown
    assert len(responses.calls) == 1


@responses.activate
def test_current_terminal_authority_and_partial_positions():
    responses.get(
        BASE + "/trade/v1/orders/id", json={"order": {"status": "MATCHED", "is_terminal": False}}
    )
    responses.get(
        BASE + "/trade/v1/orders/id",
        json={"order": {"status": "PARTIALLY_FILLED", "is_terminal": True}},
    )
    result = AgaraClient("token", BASE).wait_for_terminal("id", poll_interval=0.001)
    assert result == {"status": "PARTIALLY_FILLED", "is_terminal": True}
    payload = {
        "positions": [],
        "markets": {},
        "events": {},
        "unavailable_exchanges": ["AGARA"],
        "as_of": "2026-09-17T00:00:00Z",
    }
    responses.post(BASE + "/trade/v1/portfolio/positions/list", json=payload)
    assert AgaraClient("token", BASE).list_positions() == payload


@pytest.mark.parametrize("bad", [None, 1, "true"])
@responses.activate
def test_terminal_flag_never_guessed(bad):
    responses.get(
        BASE + "/trade/v1/orders/id", json={"order": {"status": "MATCHED", "is_terminal": bad}}
    )
    with pytest.raises(ProtocolError):
        AgaraClient("token", BASE).wait_for_terminal("id")


@pytest.mark.asyncio
async def test_async_lifecycle_and_full_deadline():
    states = iter(
        [
            {"status": "MATCHED", "is_terminal": False},
            {"status": "PARTIALLY_FILLED", "is_terminal": True},
        ]
    )

    def handler(request):
        return httpx.Response(200, json={"order": next(states)})

    async with httpx.AsyncClient(transport=httpx.MockTransport(handler)) as transport:
        client = AsyncAgaraClient("token", BASE, client=transport)
        assert (await client.wait_for_terminal("id", poll_interval=0.001))["is_terminal"]

    async def slow(request):
        assert request.extensions["timeout"]["read"] <= 0.02
        await asyncio.sleep(1)
        return httpx.Response(200, json={"order": {"is_terminal": True}})

    async with httpx.AsyncClient(transport=httpx.MockTransport(slow)) as transport:
        with pytest.raises(TimeoutError):
            await AsyncAgaraClient("token", BASE, client=transport).wait_for_terminal(
                "id", timeout=0.02
            )


FIXTURES = json.loads((ROOT / "tests/fixtures/problem-contracts.json").read_text())["fixtures"]
PROBLEMS = [
    f
    for f in FIXTURES
    if f["schema"] == "origin-problem-details.schema.json"
    and f["schema_valid"]
    and f["contract_valid"]
]


@pytest.mark.parametrize("fixture", PROBLEMS, ids=lambda f: f["name"])
@responses.activate
def test_canonical_http_problems_sync(fixture):
    body = fixture["value"]
    responses.get(BASE + "/trade/v1/status", json=body, status=body["status"])
    with pytest.raises(AgaraError) as caught:
        AgaraClient(base_url=BASE).get_status()
    error = caught.value
    assert error.code == body["code"]
    assert error.title == body["title"]
    assert error.request_id == body.get("request_id")
    if body["code"].startswith("future"):
        assert not error.is_retryable


@pytest.mark.asyncio
@pytest.mark.parametrize("fixture", PROBLEMS, ids=lambda f: f["name"])
async def test_canonical_http_problems_async(fixture):
    body = fixture["value"]
    async with httpx.AsyncClient(
        transport=httpx.MockTransport(lambda r: httpx.Response(body["status"], json=body))
    ) as transport:
        with pytest.raises(AgaraError) as caught:
            await AsyncAgaraClient(base_url=BASE, client=transport).get_status()
        assert caught.value.code == body["code"]
        if body["code"].startswith("future"):
            assert not caught.value.is_retryable


RESPONSE_FIXTURES = json.loads((ROOT / "tests/fixtures/responses.json").read_text())["responses"]


@pytest.mark.parametrize("name,payload", RESPONSE_FIXTURES.items())
@responses.activate
def test_nonempty_source_dto_fixtures_sync(name, payload):
    operation = next(op for op in OPS if op["python_method"] == name)
    kwargs = arguments(name)
    path, _ = wire_expected(operation, kwargs)
    responses.add(operation["method"], BASE + path, json=payload)
    result = getattr(AgaraClient("token", BASE), name)(**kwargs)
    assert result == payload
    if name == "list_activities":
        assert {item["type"] for item in result["activities"]} == {
            "ORDER",
            "SPLIT",
            "MERGE",
            "REDEEM",
            "DEPOSIT",
            "WITHDRAWAL",
            "LP_PAYOUT",
        }
    if name == "list_positions":
        assert result["positions"][0]["shares_micro"] == "9007199254740993"
        assert result["unavailable_exchanges"] == ["POLYMARKET"]
    if name == "get_realized_pnl":
        assert result["amountScale"] == 12


@pytest.mark.asyncio
@pytest.mark.parametrize("name,payload", RESPONSE_FIXTURES.items())
async def test_nonempty_source_dto_fixtures_async(name, payload):
    async with httpx.AsyncClient(
        transport=httpx.MockTransport(lambda r: httpx.Response(200, json=payload))
    ) as transport:
        result = await getattr(AsyncAgaraClient("token", BASE, client=transport), name)(
            **arguments(name)
        )
        assert result == payload


@pytest.mark.parametrize(
    "status",
    [400, 401, 403, 404, 405, 409, 410, 413, 415, 422, 424, 425, 426, 429, 500, 502, 503, 504],
)
@responses.activate
def test_http_status_and_retry_after_retained(status):
    responses.get(
        BASE + "/trade/v1/status",
        body="upstream error",
        status=status,
        headers={"Retry-After": "2"},
    )
    with pytest.raises(AgaraError) as caught:
        AgaraClient(base_url=BASE).get_status()
    assert caught.value.status_code == status
    assert caught.value.retry_after == 2
    assert len(responses.calls) == 1


@pytest.mark.asyncio
@pytest.mark.parametrize(
    "status",
    [400, 401, 403, 404, 405, 409, 410, 413, 415, 422, 424, 425, 426, 429, 500, 502, 503, 504],
)
async def test_async_http_status_and_retry_after_retained(status):
    async with httpx.AsyncClient(
        transport=httpx.MockTransport(
            lambda r: httpx.Response(status, text="upstream error", headers={"Retry-After": "2"})
        )
    ) as transport:
        with pytest.raises(AgaraError) as caught:
            await AsyncAgaraClient(base_url=BASE, client=transport).get_status()
        assert caught.value.status_code == status
        assert caught.value.retry_after == 2


def test_signed_micro_range_includes_both_i64_endpoints():
    assert to_micro("-9223372036854.775808") == -(2**63)
    assert to_micro("9223372036854.775807") == 2**63 - 1
    with pytest.raises(ValueError):
        to_micro("-9223372036854.775809")

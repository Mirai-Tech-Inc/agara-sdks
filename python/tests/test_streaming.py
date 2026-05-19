"""Tests for the unified streaming client.

Scope:
- wire-shape → dataclass mapping (`decode_frame`)
- channel factories
- callback dispatch (via `_dispatch` against synthetic frames)
- subscription set management + per-call cap
- reconnect backoff curve

The transport (`websockets.connect`) and full reconnect lifecycle are
exercised by the smoke-test example, not here.
"""

from __future__ import annotations

import asyncio
import json
from typing import Any

import pytest

from agara_sdk import streaming


# ── decoder: public channels ─────────────────────────────────────────


def test_orderbook_snapshot_decodes():
    frame = streaming.decode_frame({
        "op": "update",
        "channel": "orderbook",
        "token_id": "21742",
        "sequence": 12345,
        "data": {
            "kind": "snapshot",
            "bids": [["0.60", "100"], ["0.59", "50"]],
            "asks": [["0.62", "80"]],
            "tick_size": "0.01",
        },
    })
    assert isinstance(frame, streaming.OrderbookSnapshot)
    assert frame.token_id == "21742"
    assert frame.sequence == 12345
    assert frame.bids == [streaming.Level(0.60, 100.0), streaming.Level(0.59, 50.0)]
    assert frame.asks == [streaming.Level(0.62, 80.0)]
    assert frame.tick_size == "0.01"


def test_orderbook_delta_decodes():
    frame = streaming.decode_frame({
        "op": "update",
        "channel": "orderbook",
        "token_id": "21742",
        "sequence": 12346,
        "data": {
            "kind": "delta",
            "bids": [["0.60", "120"]],
            "asks": [["0.62", "0"]],
        },
    })
    assert isinstance(frame, streaming.OrderbookDelta)
    assert frame.bids == [streaming.Level(0.60, 120.0)]
    assert frame.asks == [streaming.Level(0.62, 0.0)]


def test_best_quote_handles_empty_side():
    frame = streaming.decode_frame({
        "op": "update",
        "channel": "best_quote",
        "token_id": "21742",
        "sequence": 99,
        "data": {
            "bid": {"price": "0.60", "size": "120"},
            "ask": None,
        },
    })
    assert isinstance(frame, streaming.BestQuote)
    assert frame.bid == streaming.Level(0.60, 120.0)
    assert frame.ask is None


def test_market_resolved_decodes():
    frame = streaming.decode_frame({
        "op": "update",
        "channel": "market_status",
        "condition_id": "0x21",
        "sequence": 12349,
        "data": {"kind": "market_resolved", "winning_outcome": 0},
    })
    assert isinstance(frame, streaming.MarketResolved)
    assert frame.condition_id == "0x21"
    assert frame.winning_outcome == 0


def test_market_halted_decodes_without_data_fields():
    frame = streaming.decode_frame({
        "op": "update",
        "channel": "market_status",
        "condition_id": "0x21",
        "sequence": 12350,
        "data": {"kind": "market_halted"},
    })
    assert isinstance(frame, streaming.MarketHalted)


def test_trade_decodes():
    frame = streaming.decode_frame({
        "op": "update",
        "channel": "trades",
        "condition_id": "0x21",
        "sequence": 12351,
        "data": {
            "kind": "trade",
            "fill_id": "42",
            "outcome": 0,
            "side": "buy",
            "price": "0.60",
            "size": "120.00",
            "settlement_mode": "normal",
        },
    })
    assert isinstance(frame, streaming.Trade)
    assert frame.condition_id == "0x21"
    assert frame.sequence == 12351
    assert frame.fill_id == "42"
    assert frame.outcome == 0
    assert frame.side == "buy"
    assert frame.price == 0.60
    assert frame.size == 120.0
    assert frame.settlement_mode == "normal"


def test_trade_mint_settlement_decodes():
    frame = streaming.decode_frame({
        "op": "update",
        "channel": "trades",
        "condition_id": "0x21",
        "sequence": 12352,
        "data": {
            "kind": "trade",
            "fill_id": "43",
            "outcome": 1,
            "side": "buy",
            "price": "0.50",
            "size": "10.00",
            "settlement_mode": "mint",
        },
    })
    assert isinstance(frame, streaming.Trade)
    assert frame.settlement_mode == "mint"


def test_unknown_trade_kind_falls_through():
    frame = streaming.decode_frame({
        "op": "update",
        "channel": "trades",
        "condition_id": "0x21",
        "sequence": 12353,
        "data": {"kind": "frobnicated"},
    })
    assert isinstance(frame, streaming.UnknownFrame)


def test_unknown_market_status_kind_falls_through():
    frame = streaming.decode_frame({
        "op": "update",
        "channel": "market_status",
        "condition_id": "0x21",
        "sequence": 12350,
        "data": {"kind": "frobnicated"},
    })
    assert isinstance(frame, streaming.UnknownFrame)


# ── decoder: account_events ──────────────────────────────────────────


def test_fill_taker_decodes():
    frame = streaming.decode_frame({
        "op": "update",
        "channel": "account_events",
        "sequence": 42,
        "data": {
            "kind": "fill",
            "role": "taker",
            "fill_id": "fill-1",
            "order_id": "11111111-1111-1111-1111-111111111111",
            "engine_seq_num": "9001",
            "market_id": "m-1",
            "outcome": 0,
            "side": "buy",
            "price_micro": "600000",
            "shares_micro": "1000000",
            "settlement_mode": "normal",
            "fee_micro_usdc": "1234",
        },
    })
    assert isinstance(frame, streaming.Fill)
    assert frame.role == "taker"
    assert frame.fill_id == "fill-1"
    assert frame.engine_seq_num == 9001
    assert frame.price_micro == 600_000
    assert frame.shares_micro == 1_000_000
    assert frame.fee_micro_usdc == 1234
    assert frame.settlement_mode == "normal"
    assert frame.sequence == 42


def test_fill_with_null_order_id_decodes():
    frame = streaming.decode_frame({
        "op": "update",
        "channel": "account_events",
        "sequence": 43,
        "data": {
            "kind": "fill",
            "role": "maker",
            "fill_id": "fill-2",
            "order_id": None,
            "engine_seq_num": "9002",
            "market_id": "m-1",
            "outcome": 1,
            "side": "sell",
            "price_micro": "400000",
            "shares_micro": "5000000",
            "settlement_mode": "normal",
            "fee_micro_usdc": "0",
        },
    })
    assert isinstance(frame, streaming.Fill)
    assert frame.order_id is None


def test_order_accepted_and_cancelled_decode():
    accepted = streaming.decode_frame({
        "op": "update",
        "channel": "account_events",
        "sequence": 50,
        "data": {
            "kind": "order_accepted",
            "order_id": "11111111-1111-1111-1111-111111111111",
            "engine_seq_num": "100",
            "market_id": "m-1",
            "outcome": 0,
            "side": "buy",
            "price_micro": "600000",
            "remaining_shares_micro": "5000000",
            "tif": "gtc",
        },
    })
    assert isinstance(accepted, streaming.OrderAccepted)
    assert accepted.tif == "gtc"

    cancelled = streaming.decode_frame({
        "op": "update",
        "channel": "account_events",
        "sequence": 51,
        "data": {
            "kind": "order_cancelled",
            "order_id": "11111111-1111-1111-1111-111111111111",
            "engine_seq_num": "100",
            "market_id": "m-1",
            "outcome": 0,
            "side": "buy",
            "price_micro": "600000",
            "remaining_shares_micro": "5000000",
            "reason": "user",
        },
    })
    assert isinstance(cancelled, streaming.OrderCancelled)
    assert cancelled.reason == "user"


def test_tokens_minted_and_merged_decode():
    minted = streaming.decode_frame({
        "op": "update",
        "channel": "account_events",
        "sequence": 60,
        "data": {
            "kind": "tokens_minted",
            "engine_seq_num": "200",
            "market_id": "m-1",
            "shares_micro": "10000000",
        },
    })
    assert isinstance(minted, streaming.TokensMinted)
    assert minted.shares_micro == 10_000_000

    merged = streaming.decode_frame({
        "op": "update",
        "channel": "account_events",
        "sequence": 61,
        "data": {
            "kind": "tokens_merged",
            "engine_seq_num": "201",
            "market_id": "m-1",
            "shares_micro": "5000000",
        },
    })
    assert isinstance(merged, streaming.TokensMerged)
    assert merged.shares_micro == 5_000_000


def test_unknown_account_event_kind_falls_through():
    frame = streaming.decode_frame({
        "op": "update",
        "channel": "account_events",
        "sequence": 70,
        "data": {"kind": "frobnicated"},
    })
    assert isinstance(frame, streaming.UnknownFrame)


# ── decoder: control frames ──────────────────────────────────────────


def test_subscribed_ack():
    frame = streaming.decode_frame({
        "op": "subscribed",
        "channel": "orderbook",
        "token_id": "21742",
    })
    assert isinstance(frame, streaming.Subscribed)
    assert frame.token_id == "21742"
    assert frame.condition_id is None


def test_sequence_reset():
    frame = streaming.decode_frame({
        "op": "sequence_reset",
        "channel": "orderbook",
        "token_id": "21742",
    })
    assert isinstance(frame, streaming.SequenceReset)


def test_heartbeat_and_pong():
    hb = streaming.decode_frame({"op": "heartbeat", "server_time": "2026-05-13T00:00:00Z"})
    pong = streaming.decode_frame({"op": "pong", "server_time": "2026-05-13T00:00:00Z"})
    assert isinstance(hb, streaming.Heartbeat)
    assert isinstance(pong, streaming.Pong)


def test_error_frame():
    frame = streaming.decode_frame({
        "op": "error",
        "code": "unknown_token",
        "message": "no agara market+outcome for token_id 123",
        "channel": "orderbook",
        "token_id": "123",
    })
    assert isinstance(frame, streaming.StreamError)
    assert frame.code == "unknown_token"


def test_unknown_op_yields_unknown_frame():
    frame = streaming.decode_frame({"op": "noop_added_in_v2"})
    assert isinstance(frame, streaming.UnknownFrame)
    assert frame.raw == {"op": "noop_added_in_v2"}


# ── channel factories ────────────────────────────────────────────────


def test_public_factories_wire_shape():
    assert streaming.orderbook("t").to_wire(None) == {
        "name": "orderbook",
        "token_id": "t",
    }
    assert streaming.best_quote("t").to_wire(None) == {
        "name": "best_quote",
        "token_id": "t",
    }
    assert streaming.market_status("c").to_wire(None) == {
        "name": "market_status",
        "condition_id": "c",
    }
    assert streaming.trades("c").to_wire(None) == {
        "name": "trades",
        "condition_id": "c",
    }


def test_account_events_factory_requires_token():
    ch = streaming.account_events()
    assert ch.needs_token is True
    assert ch.is_account is True
    with pytest.raises(ValueError):
        ch.to_wire(None)
    assert ch.to_wire("agt_pat_abc") == {
        "name": "account_events",
        "token": "agt_pat_abc",
    }


def test_ws_url_translates_scheme():
    assert (
        streaming._ws_url("https://example.com", "/trade/v1/market-stream")
        == "wss://example.com/trade/v1/market-stream"
    )
    assert (
        streaming._ws_url("http://localhost:8100/", "/trade/v1/account-stream")
        == "ws://localhost:8100/trade/v1/account-stream"
    )
    assert (
        streaming._ws_url("wss://example.com", "/trade/v1/market-stream")
        == "wss://example.com/trade/v1/market-stream"
    )


# ── client: subscription set management ──────────────────────────────


def test_subscribe_dedups():
    client = streaming.AgaraStreamClient()
    ch = streaming.orderbook("t")
    # Synchronously add by appending to the list — bypasses send-on-open
    # path so we don't need a connection.
    asyncio.run(client.subscribe([ch]))
    asyncio.run(client.subscribe([ch]))
    assert client._subscriptions == [ch]


def test_subscribe_cap_rejects_oversize_set():
    client = streaming.AgaraStreamClient()
    too_many = [
        streaming.orderbook(str(i))
        for i in range(streaming.MAX_SUBSCRIPTIONS_PER_CONNECTION + 1)
    ]
    with pytest.raises(ValueError):
        asyncio.run(client.subscribe(too_many))


def test_account_events_subscribe_without_token_raises():
    client = streaming.AgaraStreamClient()  # no token
    with pytest.raises(ValueError):
        asyncio.run(client.subscribe([streaming.account_events()]))


def test_unsubscribe_removes_from_set():
    client = streaming.AgaraStreamClient()
    a = streaming.orderbook("a")
    b = streaming.orderbook("b")
    asyncio.run(client.subscribe([a, b]))
    asyncio.run(client.unsubscribe([a]))
    assert client._subscriptions == [b]


# ── client: callback dispatch ────────────────────────────────────────


@pytest.mark.asyncio
async def test_trade_dispatch_routes_to_on_trade():
    client = streaming.AgaraStreamClient()
    captured: list[streaming.Trade] = []

    @client.on_trade
    async def _(t: streaming.Trade) -> None:
        captured.append(t)

    await client._dispatch(streaming.decode_frame({
        "op": "update",
        "channel": "trades",
        "condition_id": "0x21",
        "sequence": 1,
        "data": {
            "kind": "trade",
            "fill_id": "42",
            "outcome": 0,
            "side": "buy",
            "price": "0.60",
            "size": "120.00",
            "settlement_mode": "normal",
        },
    }))
    assert len(captured) == 1
    assert captured[0].fill_id == "42"
    assert captured[0].condition_id == "0x21"
    assert captured[0].price == 0.60


@pytest.mark.asyncio
async def test_fill_dispatch_routes_to_on_fill():
    client = streaming.AgaraStreamClient(token="agt_pat_x")
    captured: list[streaming.Fill] = []

    @client.on_fill
    async def _(f: streaming.Fill) -> None:
        captured.append(f)

    await client._dispatch(streaming.decode_frame({
        "op": "update",
        "channel": "account_events",
        "sequence": 1,
        "data": {
            "kind": "fill",
            "role": "taker",
            "fill_id": "f1",
            "order_id": None,
            "engine_seq_num": "1",
            "market_id": "m",
            "outcome": 0,
            "side": "buy",
            "price_micro": "600000",
            "shares_micro": "1000000",
            "settlement_mode": "normal",
            "fee_micro_usdc": "0",
        },
    }))
    assert len(captured) == 1
    assert captured[0].fill_id == "f1"


@pytest.mark.asyncio
async def test_orderbook_snapshot_and_delta_route_separately():
    client = streaming.AgaraStreamClient()
    snaps: list[streaming.OrderbookSnapshot] = []
    deltas: list[streaming.OrderbookDelta] = []

    @client.on_orderbook_snapshot
    async def _(s: streaming.OrderbookSnapshot) -> None:
        snaps.append(s)

    @client.on_orderbook_delta
    async def _(d: streaming.OrderbookDelta) -> None:
        deltas.append(d)

    await client._dispatch(streaming.decode_frame({
        "op": "update",
        "channel": "orderbook",
        "token_id": "t",
        "sequence": 1,
        "data": {
            "kind": "snapshot",
            "bids": [["0.60", "100"]],
            "asks": [],
            "tick_size": "0.01",
        },
    }))
    await client._dispatch(streaming.decode_frame({
        "op": "update",
        "channel": "orderbook",
        "token_id": "t",
        "sequence": 2,
        "data": {"kind": "delta", "bids": [["0.60", "0"]], "asks": []},
    }))
    assert len(snaps) == 1 and snaps[0].sequence == 1
    assert len(deltas) == 1 and deltas[0].sequence == 2


@pytest.mark.asyncio
async def test_sequence_reset_routes_to_handler():
    client = streaming.AgaraStreamClient()
    resets: list[streaming.SequenceReset] = []

    @client.on_sequence_reset
    async def _(r: streaming.SequenceReset) -> None:
        resets.append(r)

    await client._dispatch(streaming.decode_frame({
        "op": "sequence_reset",
        "channel": "account_events",
    }))
    assert len(resets) == 1
    assert resets[0].channel == "account_events"


@pytest.mark.asyncio
async def test_unknown_op_falls_through_to_on_unknown():
    client = streaming.AgaraStreamClient()
    seen: list[streaming.UnknownFrame] = []

    @client.on_unknown
    async def _(u: streaming.UnknownFrame) -> None:
        seen.append(u)

    await client._dispatch(streaming.decode_frame({"op": "noop_added_in_v2"}))
    assert len(seen) == 1
    assert seen[0].raw == {"op": "noop_added_in_v2"}


@pytest.mark.asyncio
async def test_handler_exception_routes_to_on_error():
    client = streaming.AgaraStreamClient()
    errors: list[streaming.StreamError] = []

    @client.on_trade
    async def _(t: streaming.Trade) -> None:
        raise RuntimeError("boom")

    @client.on_error
    async def _(err: streaming.StreamError) -> None:
        errors.append(err)

    await client._dispatch(streaming.decode_frame({
        "op": "update",
        "channel": "trades",
        "condition_id": "0x21",
        "sequence": 1,
        "data": {
            "kind": "trade",
            "fill_id": "f1",
            "outcome": 0,
            "side": "buy",
            "price": "0.60",
            "size": "1.00",
            "settlement_mode": "normal",
        },
    }))
    assert len(errors) == 1
    assert errors[0].code == "handler_exception"
    assert "RuntimeError" in errors[0].message


# ── reconnect backoff ────────────────────────────────────────────────


def test_backoff_grows_and_caps():
    cfg = streaming.Reconnect(initial_delay=1.0, max_delay=8.0, jitter=0.0)
    assert streaming._backoff(cfg, 1) == 1.0
    assert streaming._backoff(cfg, 2) == 2.0
    assert streaming._backoff(cfg, 3) == 4.0
    assert streaming._backoff(cfg, 4) == 8.0
    assert streaming._backoff(cfg, 10) == 8.0  # capped


def test_endpoint_routing():
    """A public channel routes to `market`, account_events to `account`."""
    assert streaming._endpoint_for(streaming.orderbook("t")) == "market"
    assert streaming._endpoint_for(streaming.trades("c")) == "market"
    assert streaming._endpoint_for(streaming.market_status("c")) == "market"
    assert streaming._endpoint_for(streaming.best_quote("t")) == "market"
    assert streaming._endpoint_for(streaming.account_events()) == "account"


def test_grouping_partitions_correctly():
    chs = [
        streaming.orderbook("t"),
        streaming.trades("c"),
        streaming.account_events(),
    ]
    grouped = streaming._group_by_endpoint(chs)
    assert {c.name for c in grouped["market"]} == {"orderbook", "trades"}
    assert {c.name for c in grouped["account"]} == {"account_events"}

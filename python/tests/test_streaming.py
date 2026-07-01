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
            "bids": [[60, 100], [59, 50]],
            "asks": [[62, 80]],
            "tick_size": 1,
            "price_scale": 100,
            "size_scale": 1,
        },
    })
    assert isinstance(frame, streaming.OrderbookSnapshot)
    assert frame.token_id == "21742"
    assert frame.sequence == 12345
    assert frame.bids == [streaming.Level(60, 100), streaming.Level(59, 50)]
    assert frame.asks == [streaming.Level(62, 80)]
    assert frame.tick_size == 1
    assert frame.price_scale == 100
    assert frame.size_scale == 1


def test_orderbook_delta_decodes():
    frame = streaming.decode_frame({
        "op": "update",
        "channel": "orderbook",
        "token_id": "21742",
        "sequence": 12346,
        "data": {
            "kind": "delta",
            "bids": [[60, 120]],
            "asks": [[62, 0]],
            "price_scale": 100,
            "size_scale": 1,
        },
    })
    assert isinstance(frame, streaming.OrderbookDelta)
    assert frame.bids == [streaming.Level(60, 120)]
    assert frame.asks == [streaming.Level(62, 0)]
    assert frame.price_scale == 100
    assert frame.size_scale == 1


def test_best_quote_handles_empty_side():
    frame = streaming.decode_frame({
        "op": "update",
        "channel": "best_quote",
        "token_id": "21742",
        "sequence": 99,
        "data": {
            "bid": {"price": 60, "size": 120},
            "ask": None,
            "price_scale": 100,
            "size_scale": 1,
        },
    })
    assert isinstance(frame, streaming.BestQuote)
    assert frame.bid == streaming.Level(60, 120)
    assert frame.ask is None
    assert frame.price_scale == 100
    assert frame.size_scale == 1


def test_market_resolved_decodes():
    frame = streaming.decode_frame({
        "op": "update",
        "channel": "market_status",
        "condition_id": "0x21",
        "sequence": 12349,
        "data": {"kind": "market_resolved", "winning_token_id": "yes-token"},
    })
    assert isinstance(frame, streaming.MarketResolved)
    assert frame.condition_id == "0x21"
    assert frame.winning_token_id == "yes-token"


def test_outcome_proposed_decodes():
    frame = streaming.decode_frame({
        "op": "update",
        "channel": "market_status",
        "condition_id": "0x21",
        "sequence": 12350,
        "data": {"kind": "outcome_proposed", "proposed_token_id": "yes-token"},
    })
    assert isinstance(frame, streaming.OutcomeProposed)
    assert frame.proposed_token_id == "yes-token"


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
            "taker_token_id": "yes-token",
            "maker_token_id": "yes-token",
            "side": "BUY",
            "price": 60,
            "size": 120,
            "price_scale": 100,
            "size_scale": 1,
            "settlement_mode": "NORMAL",
        },
    })
    assert isinstance(frame, streaming.Trade)
    assert frame.condition_id == "0x21"
    assert frame.sequence == 12351
    assert frame.fill_id == "42"
    assert frame.taker_token_id == "yes-token"
    assert frame.maker_token_id == "yes-token"
    assert frame.side == "BUY"
    assert frame.price == 60
    assert frame.size == 120
    assert frame.price_scale == 100
    assert frame.size_scale == 1
    assert frame.settlement_mode == "NORMAL"


def test_trade_mint_settlement_decodes_with_split_tokens():
    # MINT fills touch two outcomes — taker buys YES, maker effectively
    # buys NO via the collateral split. The two token_ids differ.
    frame = streaming.decode_frame({
        "op": "update",
        "channel": "trades",
        "condition_id": "0x21",
        "sequence": 12352,
        "data": {
            "kind": "trade",
            "fill_id": "43",
            "taker_token_id": "yes-token",
            "maker_token_id": "no-token",
            "side": "BUY",
            "price": 50,
            "size": 10,
            "price_scale": 100,
            "size_scale": 1,
            "settlement_mode": "MINT",
        },
    })
    assert isinstance(frame, streaming.Trade)
    assert frame.settlement_mode == "MINT"
    assert frame.taker_token_id != frame.maker_token_id


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
            "role": "TAKER",
            "fill_id": "fill-1",
            "order_id": "11111111-1111-1111-1111-111111111111",
            "order_hash": "0xabc123",
            "token_id": "yes-token",
            "side": "BUY",
            "price": 60,
            "size": 1_000_000,
            "price_scale": 100,
            "size_scale": 1_000_000,
            "settlement_mode": "NORMAL",
            "fee_micro": "1234",
        },
    })
    assert isinstance(frame, streaming.Fill)
    assert frame.role == "TAKER"
    assert frame.fill_id == "fill-1"
    assert frame.order_hash == "0xabc123"
    assert frame.token_id == "yes-token"
    assert frame.price == 60
    assert frame.size == 1_000_000
    assert frame.price_scale == 100
    assert frame.size_scale == 1_000_000
    assert frame.fee_micro == 1234
    assert frame.settlement_mode == "NORMAL"
    assert frame.sequence == 42


def test_fill_maker_role_decodes():
    frame = streaming.decode_frame({
        "op": "update",
        "channel": "account_events",
        "sequence": 43,
        "data": {
            "kind": "fill",
            "role": "MAKER",
            "fill_id": "fill-2",
            "order_id": "22222222-2222-2222-2222-222222222222",
            "order_hash": "0xdef456",
            "token_id": "no-token",
            "side": "SELL",
            "price": 40,
            "size": 5_000_000,
            "price_scale": 100,
            "size_scale": 1_000_000,
            "settlement_mode": "NORMAL",
            "fee_micro": "0",
        },
    })
    assert isinstance(frame, streaming.Fill)
    assert frame.order_id == "22222222-2222-2222-2222-222222222222"
    assert frame.role == "MAKER"
    assert frame.token_id == "no-token"


def test_order_accepted_and_cancelled_decode():
    accepted = streaming.decode_frame({
        "op": "update",
        "channel": "account_events",
        "sequence": 50,
        "data": {
            "kind": "order_accepted",
            "order_id": "11111111-1111-1111-1111-111111111111",
            "order_hash": "0xabc123",
            "token_id": "yes-token",
            "side": "BUY",
            "price": 60,
            "remaining_size": 5_000_000,
            "price_scale": 100,
            "size_scale": 1_000_000,
            "tif": "GTC",
        },
    })
    assert isinstance(accepted, streaming.OrderAccepted)
    assert accepted.order_hash == "0xabc123"
    assert accepted.tif == "GTC"
    assert accepted.price == 60
    assert accepted.remaining_size == 5_000_000
    assert accepted.token_id == "yes-token"

    cancelled = streaming.decode_frame({
        "op": "update",
        "channel": "account_events",
        "sequence": 51,
        "data": {
            "kind": "order_cancelled",
            "order_id": "11111111-1111-1111-1111-111111111111",
            "order_hash": "0xabc123",
            "token_id": "yes-token",
            "side": "BUY",
            "price": 60,
            "remaining_size": 5_000_000,
            "price_scale": 100,
            "size_scale": 1_000_000,
            "reason": "USER",
        },
    })
    assert isinstance(cancelled, streaming.OrderCancelled)
    assert cancelled.order_hash == "0xabc123"
    assert cancelled.reason == "USER"
    assert cancelled.remaining_size == 5_000_000
    assert cancelled.token_id == "yes-token"


def test_order_rejected_decode():
    rejected = streaming.decode_frame({
        "op": "update",
        "channel": "account_events",
        "sequence": 0,
        "data": {
            "kind": "order_rejected",
            "order_id": "11111111-1111-1111-1111-111111111111",
            "order_hash": "0xabc123",
            "token_id": "yes-token",
            "reason": "insufficient balance",
        },
    })
    assert isinstance(rejected, streaming.OrderRejected)
    assert rejected.sequence == 0
    assert rejected.order_id == "11111111-1111-1111-1111-111111111111"
    assert rejected.order_hash == "0xabc123"
    assert rejected.token_id == "yes-token"
    assert rejected.reason == "insufficient balance"


def test_order_rejected_decode_null_hash_and_token():
    # order_hash / token_id are best-effort at the router and may arrive null.
    rejected = streaming.decode_frame({
        "op": "update",
        "channel": "account_events",
        "sequence": 0,
        "data": {
            "kind": "order_rejected",
            "order_id": "11111111-1111-1111-1111-111111111111",
            "order_hash": None,
            "token_id": None,
            "reason": "engine rejected order",
        },
    })
    assert isinstance(rejected, streaming.OrderRejected)
    assert rejected.order_hash is None
    assert rejected.token_id is None
    assert rejected.reason == "engine rejected order"


def test_tokens_minted_and_merged_decode():
    minted = streaming.decode_frame({
        "op": "update",
        "channel": "account_events",
        "sequence": 60,
        "data": {
            "kind": "tokens_minted",
            "condition_id": "0xcond",
            "size": 10_000_000,
            "size_scale": 1_000_000,
        },
    })
    assert isinstance(minted, streaming.TokensMinted)
    assert minted.size == 10_000_000
    assert minted.size_scale == 1_000_000
    assert minted.condition_id == "0xcond"

    merged = streaming.decode_frame({
        "op": "update",
        "channel": "account_events",
        "sequence": 61,
        "data": {
            "kind": "tokens_merged",
            "condition_id": "0xcond",
            "size": 5_000_000,
            "size_scale": 1_000_000,
        },
    })
    assert isinstance(merged, streaming.TokensMerged)
    assert merged.size == 5_000_000
    assert merged.condition_id == "0xcond"


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
        "reason": "lagged",
    })
    assert isinstance(frame, streaming.SequenceReset)
    assert frame.reason == "lagged"


def test_sequence_reset_reason_optional():
    frame = streaming.decode_frame({"op": "sequence_reset", "channel": "trades"})
    assert isinstance(frame, streaming.SequenceReset)
    assert frame.reason is None


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
    assert frame.action is None


def test_error_frame_carries_action():
    frame = streaming.decode_frame({
        "op": "error",
        "code": "subject_unavailable",
        "message": "subject feed ended; resubscribe to resume",
        "action": "resubscribe",
        "channel": "orderbook",
        "token_id": "123",
    })
    assert isinstance(frame, streaming.StreamError)
    assert frame.action == "resubscribe"


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
            "taker_token_id": "yes-token",
            "maker_token_id": "yes-token",
            "side": "BUY",
            "price": 60,
            "size": 120,
            "price_scale": 100,
            "size_scale": 1,
            "settlement_mode": "NORMAL",
        },
    }))
    assert len(captured) == 1
    assert captured[0].fill_id == "42"
    assert captured[0].condition_id == "0x21"
    assert captured[0].price == 60


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
            "role": "TAKER",
            "fill_id": "f1",
            "order_id": "33333333-3333-3333-3333-333333333333",
            "order_hash": "0x789abc",
            "token_id": "yes-token",
            "side": "BUY",
            "price": 60,
            "size": 1_000_000,
            "price_scale": 100,
            "size_scale": 1_000_000,
            "settlement_mode": "NORMAL",
            "fee_micro": "0",
        },
    }))
    assert len(captured) == 1
    assert captured[0].fill_id == "f1"
    assert captured[0].order_id == "33333333-3333-3333-3333-333333333333"


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
            "bids": [[60, 100]],
            "asks": [],
            "tick_size": 1,
            "price_scale": 100,
            "size_scale": 1,
        },
    }))
    await client._dispatch(streaming.decode_frame({
        "op": "update",
        "channel": "orderbook",
        "token_id": "t",
        "sequence": 2,
        "data": {
            "kind": "delta",
            "bids": [[60, 0]],
            "asks": [],
            "price_scale": 100,
            "size_scale": 1,
        },
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
            "taker_token_id": "yes-token",
            "maker_token_id": "yes-token",
            "side": "BUY",
            "price": 60,
            "size": 1,
            "price_scale": 100,
            "size_scale": 1,
            "settlement_mode": "NORMAL",
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


class _FakeEndpoint:
    """Stand-in for `_Endpoint` capturing sends/closes for action tests."""

    def __init__(self) -> None:
        self.sent: list[dict[str, Any]] = []
        self.closed = False
        self._open = True

    @property
    def is_open(self) -> bool:
        return self._open

    async def send(self, payload: dict[str, Any]) -> None:
        self.sent.append(payload)

    async def close(self) -> None:
        self.closed = True
        self._open = False


@pytest.mark.asyncio
async def test_resubscribe_action_replays_endpoint_subscriptions():
    client = streaming.AgaraStreamClient()
    fake = _FakeEndpoint()
    client._market = fake  # type: ignore[assignment]
    client._subscriptions = [streaming.orderbook("tok"), streaming.trades("cond")]

    await client._handle_error_action(
        streaming.StreamError(
            code="subject_unavailable",
            message="gone",
            action="resubscribe",
            channel="orderbook",
            token_id="tok",
        )
    )

    assert len(fake.sent) == 1
    op = fake.sent[0]
    assert op["op"] == "subscribe"
    assert {c["name"] for c in op["channels"]} == {"orderbook", "trades"}
    assert not fake.closed


@pytest.mark.asyncio
async def test_reconnect_action_closes_affected_endpoint():
    client = streaming.AgaraStreamClient(token="jwt")
    fake_account = _FakeEndpoint()
    fake_market = _FakeEndpoint()
    client._account = fake_account  # type: ignore[assignment]
    client._market = fake_market  # type: ignore[assignment]

    await client._handle_error_action(
        streaming.StreamError(
            code="unauthorized",
            message="token rejected",
            action="reconnect",
            channel="account_events",
        )
    )

    assert fake_account.closed
    assert not fake_market.closed
    assert fake_account.sent == []

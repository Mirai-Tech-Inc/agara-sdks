import asyncio
import json
from pathlib import Path

import pytest
from websockets.asyncio.server import serve

from agara_sdk import streaming

FIXTURES = json.loads((Path(__file__).parent / "fixtures/websocket.json").read_text())


@pytest.mark.parametrize("fixture", FIXTURES, ids=lambda f: f["name"])
def test_all_current_update_and_error_shapes(fixture):
    frame = streaming.decode_frame(fixture["value"])
    assert not isinstance(frame, streaming.UnknownFrame)
    raw = fixture["value"]
    if raw.get("event_id"):
        assert frame.event_id == raw["event_id"]
    if raw.get("data", {}).get("batch_hash"):
        assert frame.batch_hash == raw["data"]["batch_hash"]
        assert frame.batch_index == raw["data"]["batch_index"]


def test_event_subscription_and_reseed_contract():
    assert streaming.event_market_status("event").to_wire(None) == {
        "name": "market_status",
        "event_id": "event",
    }
    for op in ["subscribed", "unsubscribed", "sequence_reset"]:
        frame = streaming.decode_frame(
            {"op": op, "channel": "market_status", "event_id": "event", "reason": "lagged"}
        )
        assert frame.event_id == "event"
    assert streaming.SequenceReset("account_events").requires_rest_reseed
    assert streaming.SequenceReset("market_status", event_id="event").requires_rest_reseed
    assert not streaming.SequenceReset("orderbook", token_id="123").requires_rest_reseed
    with pytest.raises(ValueError):
        streaming.Channel("market_status", condition_id="c", event_id="e")


@pytest.mark.asyncio
async def test_real_websocket_reconnects_and_resubscribes_then_cleanly_stops():
    subscriptions = []

    async def handler(socket):
        subscriptions.append(json.loads(await socket.recv()))
        await socket.send(
            json.dumps(
                {
                    "op": "update",
                    "channel": "best_quote",
                    "token_id": "123",
                    "sequence": len(subscriptions),
                    "data": {
                        "bid": None,
                        "ask": None,
                        "price_scale": 1000000,
                        "size_scale": 1000000,
                    },
                }
            )
        )
        if len(subscriptions) == 1:
            await socket.close()
        else:
            await socket.wait_closed()

    async with serve(handler, "127.0.0.1", 0) as server:
        port = server.sockets[0].getsockname()[1]
        client = streaming.AgaraStreamClient(
            base_url=f"http://127.0.0.1:{port}",
            reconnect=streaming.Reconnect(
                initial_delay=0.001, max_delay=0.01, jitter=0, max_attempts=3
            ),
            max_queue_size=2,
        )
        received = []

        @client.on_best_quote
        async def on_quote(frame):
            received.append(frame)
            if len(received) == 2:
                await client.stop()

        await client.subscribe([streaming.best_quote("123")])
        await asyncio.wait_for(client.run(), 2)
    assert len(received) == 2
    assert subscriptions[0] == subscriptions[1]
    assert client._queue is None
    assert not client._market.is_open


@pytest.mark.asyncio
async def test_bounded_queue_cancellation_does_not_deadlock():
    async def handler(socket):
        await socket.recv()
        for _ in range(20):
            await socket.send(
                json.dumps({"op": "heartbeat", "server_time": "2026-09-17T00:00:00Z"})
            )
        await socket.wait_closed()

    async with serve(handler, "127.0.0.1", 0) as server:
        port = server.sockets[0].getsockname()[1]
        client = streaming.AgaraStreamClient(base_url=f"http://127.0.0.1:{port}", max_queue_size=1)
        await client.subscribe([streaming.orderbook("123")])
        async with client:
            await asyncio.sleep(0.01)
            assert client._queue.qsize() == 1
            await asyncio.wait_for(client.stop(), 2.0)
        assert not client._market.is_open


@pytest.mark.asyncio
async def test_dynamic_account_endpoint_and_run_cancellation():
    paths = []

    async def handler(socket):
        paths.append(socket.request.path)
        await socket.recv()
        await socket.wait_closed()

    async with serve(handler, "127.0.0.1", 0) as server:
        port = server.sockets[0].getsockname()[1]
        client = streaming.AgaraStreamClient(token="token", base_url=f"http://127.0.0.1:{port}")
        await client.subscribe([streaming.orderbook("123")])
        ready = asyncio.Event()

        @client.on_connect
        async def connected(context):
            if context.endpoint == "market":
                await client.subscribe([streaming.account_events()])
            else:
                ready.set()

        task = asyncio.create_task(client.run())
        await asyncio.wait_for(ready.wait(), 1)
        task.cancel()
        with pytest.raises(asyncio.CancelledError):
            await asyncio.wait_for(task, 1)
    assert set(paths) == {"/trade/v1/market-stream", "/trade/v1/account-stream"}
    assert client._queue is None


@pytest.mark.asyncio
async def test_accept_then_close_respects_max_attempts():
    attempts = []

    async def handler(socket):
        attempts.append(1)
        await socket.recv()
        await socket.close()

    async with serve(handler, "127.0.0.1", 0) as server:
        port = server.sockets[0].getsockname()[1]
        client = streaming.AgaraStreamClient(
            base_url=f"http://127.0.0.1:{port}",
            reconnect=streaming.Reconnect(
                initial_delay=0.001, max_delay=0.01, jitter=0, max_attempts=2
            ),
        )
        await client.subscribe([streaming.orderbook("123")])
        await asyncio.wait_for(client.run(), 1)
    assert len(attempts) == 2


@pytest.mark.asyncio
async def test_policy_close_stops_without_reconnecting():
    attempts = []
    errors = []

    async def handler(socket):
        attempts.append(1)
        await socket.recv()
        await socket.close(code=1008, reason="policy")

    async with serve(handler, "127.0.0.1", 0) as server:
        port = server.sockets[0].getsockname()[1]
        client = streaming.AgaraStreamClient(base_url=f"http://127.0.0.1:{port}")

        @client.on_error
        async def on_error(error):
            errors.append(error)

        await client.subscribe([streaming.orderbook("123")])
        await asyncio.wait_for(client.run(), 1)
    assert len(attempts) == 1
    assert errors[0].code == "policy_close"


@pytest.mark.asyncio
async def test_callback_stop_with_full_queue_finishes():
    async def handler(socket):
        await socket.recv()
        for _ in range(30):
            await socket.send(
                json.dumps({"op": "heartbeat", "server_time": "2026-09-17T00:00:00Z"})
            )
        await socket.wait_closed()

    async with serve(handler, "127.0.0.1", 0) as server:
        port = server.sockets[0].getsockname()[1]
        client = streaming.AgaraStreamClient(base_url=f"http://127.0.0.1:{port}", max_queue_size=1)

        @client.on_heartbeat
        async def heartbeat(frame):
            await asyncio.sleep(0.01)
            assert client._queue.full()
            await client.stop()

        await client.subscribe([streaming.orderbook("123")])
        await asyncio.wait_for(client.run(), 2)
    assert client._queue is None


@pytest.mark.parametrize("bad", [True, 1.5, None])
def test_malformed_native_units_never_silently_round(bad):
    frame = streaming.decode_frame(
        {
            "op": "update",
            "channel": "best_quote",
            "sequence": 1,
            "token_id": "123",
            "data": {
                "bid": {"price": bad, "size": 1},
                "ask": None,
                "price_scale": 100,
                "size_scale": 1000000,
            },
        }
    )
    assert isinstance(frame, streaming.UnknownFrame)

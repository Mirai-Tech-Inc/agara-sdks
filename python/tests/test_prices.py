from __future__ import annotations

import asyncio
import json
from decimal import Decimal

import httpx
import pytest
import responses

from agara_sdk import AgaraClient, ProtocolError
from agara_sdk.aio import AsyncAgaraClient
from agara_sdk.prices import MAX_EVENT_BYTES, SseByteDecoder

PRICE = {
    "parsed": [
        {
            "id": "Crypto.BTC/USD",
            "price": {"price": "6434371609685", "expo": -8, "publish_time_ms": 1753500000000},
        }
    ]
}
WIRE = ("data: " + json.dumps(PRICE) + "\r\n\r\n").encode()
BASE = "https://offline.invalid"


def test_chunk_split_utf8_crlf_keepalive_and_precise_mantissa():
    raw = {**PRICE, "extension": "界"}
    wire = (
        ": keep-alive\r\n\r\ndata: " + json.dumps(raw, ensure_ascii=False) + "\r\n\r\n"
    ).encode()
    for size in [1, 2, 7, 8192]:
        decoder = SseByteDecoder()
        frames = []
        for offset in range(0, len(wire), size):
            frames.extend(decoder.feed(wire[offset : offset + size]))
        assert len(frames) == 1
        assert frames[0].raw == raw
        assert frames[0].prices[0].value == Decimal("64343.71609685")
        assert frames[0].prices[0].publish_time_ms == 1753500000000


def test_oversized_unterminated_line_is_bounded():
    decoder = SseByteDecoder()
    with pytest.raises(ProtocolError):
        for _ in range(MAX_EVENT_BYTES // 8192 + 1):
            list(decoder.feed(b"x" * 8192))
    assert len(decoder._buffer) <= MAX_EVENT_BYTES


@pytest.mark.parametrize(
    "field,bad",
    [
        ("price", 1.5),
        ("price", True),
        ("expo", 1.5),
        ("expo", True),
        ("publish_time_ms", 1.5),
        ("publish_time_ms", True),
    ],
)
def test_malformed_numbers_are_never_coerced(field, bad):
    data = json.loads(json.dumps(PRICE))
    data["parsed"][0]["price"][field] = bad
    with pytest.raises(ProtocolError):
        list(SseByteDecoder().feed(("data: " + json.dumps(data) + "\n\n").encode()))


@pytest.mark.parametrize("bad", [{}, {"parsed": []}, {"parsed": "bad"}])
def test_missing_known_shape_is_not_an_empty_price_tick(bad):
    with pytest.raises(ProtocolError):
        list(SseByteDecoder().feed(("data: " + json.dumps(bad) + "\n\n").encode()))


@pytest.mark.parametrize(
    "method,arguments,path,query",
    [
        (
            "stream_prices",
            [["Crypto.BTC/USD", "Crypto.ETH/USD"]],
            "/api/v1/prices/stream",
            "symbols=Crypto.BTC%2FUSD%2CCrypto.ETH%2FUSD",
        ),
        (
            "stream_pyth_price",
            ["Crypto.BTC/USD"],
            "/api/v1/prices/pyth-pro/stream",
            "symbol=Crypto.BTC%2FUSD",
        ),
    ],
)
@responses.activate
def test_both_sync_price_feeds(method, arguments, path, query):
    responses.get(BASE + path, body=WIRE, content_type="text/event-stream")
    with AgaraClient(base_url=BASE) as client:
        frames = list(getattr(client, method)(*arguments))
    assert frames[0].prices[0].symbol == "Crypto.BTC/USD"
    assert responses.calls[0].request.url == BASE + path + "?" + query


@pytest.mark.asyncio
@pytest.mark.parametrize(
    "method,arguments,path",
    [
        ("stream_prices", [["Crypto.BTC/USD"]], "/api/v1/prices/stream"),
        ("stream_pyth_price", ["Crypto.BTC/USD"], "/api/v1/prices/pyth-pro/stream"),
    ],
)
async def test_both_async_price_feeds_and_early_close(method, arguments, path):
    class Feed(httpx.AsyncByteStream):
        closed = False

        async def __aiter__(self):
            yield WIRE[:11]
            yield WIRE[11:]
            await asyncio.sleep(60)

        async def aclose(self):
            self.closed = True

    feed = Feed()

    def handler(request):
        assert request.url.path == path
        assert request.extensions["timeout"]["read"] >= 30
        return httpx.Response(200, headers={"content-type": "text/event-stream"}, stream=feed)

    async with httpx.AsyncClient(transport=httpx.MockTransport(handler)) as transport:
        client = AsyncAgaraClient(base_url=BASE, client=transport)
        stream = getattr(client, method)(*arguments)
        assert (await anext(stream)).prices[0].mantissa == 6434371609685
        await stream.aclose()
        assert feed.closed


@pytest.mark.asyncio
async def test_async_price_stream_oversized_error_is_bounded():
    class Feed(httpx.AsyncByteStream):
        async def __aiter__(self):
            for _ in range(MAX_EVENT_BYTES // 8192 + 1):
                yield b"x" * 8192

    async with httpx.AsyncClient(
        transport=httpx.MockTransport(lambda r: httpx.Response(503, stream=Feed()))
    ) as transport:
        with pytest.raises(ProtocolError):
            await anext(
                AsyncAgaraClient(base_url=BASE, client=transport).stream_pyth_price(
                    "Crypto.BTC/USD"
                )
            )

"""Bounded SSE parsing for both current Pyth Pro price producers."""

from __future__ import annotations

import json
import re
from collections.abc import AsyncIterator, Iterator
from dataclasses import dataclass
from decimal import Decimal, localcontext
from typing import TYPE_CHECKING, Any

import requests

from .errors import ProtocolError, TransportError, _raise_api_error

if TYPE_CHECKING:
    import httpx

MAX_EVENT_BYTES = 1_048_576


@dataclass(frozen=True)
class Price:
    symbol: str
    mantissa: int
    expo: int
    publish_time_ms: int

    @property
    def value(self) -> Decimal:
        with localcontext() as context:
            context.prec = max(40, len(str(abs(self.mantissa))) + abs(self.expo) + 1)
            return Decimal(self.mantissa).scaleb(self.expo)


@dataclass(frozen=True)
class PriceEvent:
    """Unknown provider fields remain available in raw; keepalives are omitted."""

    prices: tuple[Price, ...]
    raw: dict[str, Any]
    event: str = "message"
    id: str | None = None


class SseDecoder:
    """Incremental SSE line parser with a hard event-size cap."""

    def __init__(self) -> None:
        self._data: list[str] = []
        self._size = 0
        self._event = "message"
        self._id: str | None = None

    def feed(self, line: str) -> PriceEvent | None:
        self._size += len(line.encode("utf-8"))
        if self._size > MAX_EVENT_BYTES:
            raise ProtocolError("SSE event exceeds maximum size")
        if not line:
            data, event, event_id = self._data, self._event, self._id
            self._data, self._size, self._event = [], 0, "message"
            if not data:
                return None
            try:
                raw = json.loads("\n".join(data))
                if not isinstance(raw, dict):
                    raise ValueError("object required")
                entries = raw["parsed"]
                if not isinstance(entries, list) or not entries:
                    raise ValueError("parsed must be a nonempty list")
                prices = tuple(_price(item) for item in entries)
            except (ValueError, KeyError, TypeError) as exc:
                raise ProtocolError("invalid price SSE event") from exc
            return PriceEvent(prices, raw, event, event_id)
        if line.startswith(":"):
            return None
        field, _, value = line.partition(":")
        value = value.removeprefix(" ")
        if field == "data":
            self._data.append(value)
        elif field == "event":
            self._event = value
        elif field == "id" and "\x00" not in value:
            self._id = value
        return None


def _route(symbols: list[str] | None, symbol: str | None) -> tuple[str, dict[str, str]]:
    if (symbols is None) == (symbol is None):
        raise ValueError("provide symbols or symbol")
    if symbols is not None:
        symbols = list(dict.fromkeys(symbols))
        if not 1 <= len(symbols) <= 64 or any(not _valid_symbol(s) for s in symbols):
            raise ValueError("provide 1 to 64 nonempty provider symbols")
        return "/api/v1/prices/stream", {"symbols": ",".join(symbols)}
    if not symbol or not _valid_symbol(symbol):
        raise ValueError("provide a valid provider symbol")
    return "/api/v1/prices/pyth-pro/stream", {"symbol": symbol}


def iter_prices(
    session: requests.Session,
    base_url: str,
    headers: dict[str, str],
    timeout: float,
    *,
    symbols: list[str] | None = None,
    symbol: str | None = None,
) -> Iterator[PriceEvent]:
    path, params = _route(symbols, symbol)
    try:
        with session.get(
            base_url + path,
            params=params,
            headers={**headers, "Accept": "text/event-stream"},
            timeout=(timeout, max(timeout, 30.0)),
            stream=True,
            allow_redirects=False,
        ) as response:
            if response.status_code != 200:
                chunks = bytearray()
                for chunk in response.iter_content(chunk_size=8192):
                    if len(chunks) + len(chunk) > MAX_EVENT_BYTES:
                        raise ProtocolError("SSE error body exceeds maximum size")
                    chunks.extend(chunk)
                body = _error_body(bytes(chunks))
                _raise_api_error(response.status_code, body, response.headers)
            if "text/event-stream" not in response.headers.get("content-type", ""):
                raise ProtocolError("expected text/event-stream")
            decoder = SseByteDecoder()
            for chunk in response.iter_content(chunk_size=8192):
                yield from decoder.feed(chunk)
    except requests.RequestException as exc:
        raise TransportError("GET", exc) from exc


async def aiter_prices(
    client: httpx.AsyncClient,
    base_url: str,
    headers: dict[str, str],
    timeout: float,
    *,
    symbols: list[str] | None = None,
    symbol: str | None = None,
) -> AsyncIterator[PriceEvent]:
    import httpx

    path, params = _route(symbols, symbol)
    try:
        async with client.stream(
            "GET",
            base_url + path,
            params=params,
            headers={**headers, "Accept": "text/event-stream"},
            timeout=httpx.Timeout(timeout, read=max(timeout, 30.0)),
            follow_redirects=False,
        ) as response:
            if response.status_code != 200:
                chunks = bytearray()
                async for chunk in response.aiter_bytes():
                    if len(chunks) + len(chunk) > MAX_EVENT_BYTES:
                        raise ProtocolError("SSE error body exceeds maximum size")
                    chunks.extend(chunk)
                body = _error_body(bytes(chunks))
                _raise_api_error(response.status_code, body, response.headers)
            if "text/event-stream" not in response.headers.get("content-type", ""):
                raise ProtocolError("expected text/event-stream")
            decoder = SseByteDecoder()
            async for chunk in response.aiter_bytes():
                for frame in decoder.feed(chunk):
                    yield frame
    except httpx.TransportError as exc:
        raise TransportError("GET", exc) from exc


def _valid_symbol(value: str) -> bool:
    return isinstance(value, str) and re.fullmatch(r"[A-Za-z0-9._/-]{1,64}", value) is not None


def _price(value: Any) -> Price:
    if not isinstance(value, dict) or not isinstance(value.get("id"), str):
        raise ValueError("invalid price entry")
    data = value["price"]
    mantissa, expo, timestamp = data["price"], data["expo"], data["publish_time_ms"]
    if not isinstance(mantissa, str) or re.fullmatch(r"-?[0-9]+", mantissa) is None:
        raise ValueError("price must be an integer string")
    if type(expo) is not int or type(timestamp) is not int or timestamp < 0:
        raise ValueError("expo and publish_time_ms must be integers")
    return Price(value["id"], int(mantissa), expo, timestamp)


def _error_body(value: bytes) -> Any:
    try:
        return json.loads(value)
    except (ValueError, UnicodeDecodeError):
        return value.decode("utf-8", errors="replace")


class SseByteDecoder:
    """Bounds unterminated lines before decoding UTF-8; handles split CRLF and code points."""

    def __init__(self) -> None:
        self._lines = SseDecoder()
        self._buffer = bytearray()
        self._cr = False

    def feed(self, chunk: bytes) -> Iterator[PriceEvent]:
        start = 0
        for index, byte in enumerate(chunk):
            if byte not in (10, 13):
                continue
            self._append(chunk[start:index])
            start = index + 1
            if byte == 10 and self._cr:
                self._cr = False
                continue
            self._cr = byte == 13
            try:
                line = self._buffer.decode("utf-8")
            except UnicodeDecodeError as exc:
                raise ProtocolError("invalid UTF-8 in price stream") from exc
            self._buffer.clear()
            event = self._lines.feed(line)
            if event is not None:
                yield event
        self._append(chunk[start:])

    def _append(self, value: bytes) -> None:
        if value:
            self._cr = False
        if len(self._buffer) + len(value) + self._lines._size > MAX_EVENT_BYTES:
            raise ProtocolError("SSE event exceeds maximum size")
        self._buffer.extend(value)

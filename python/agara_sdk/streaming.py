"""WebSocket client for the agara market and account streams.

Async-only; requires `websockets`:

    pip install 'agara-sdk[streaming]'

One client supports two ergonomic modes.

**Callback mode** — register handlers, then `await client.run()`. Auto-
reconnects on disconnect; replays the subscription set on every
reconnect. Recommended for bots.

    from agara_sdk import streaming

    client = streaming.AgaraStreamClient(token="agt_pat_...")

    @client.on_trade
    async def _(t: streaming.Trade) -> None:
        print(f"trade {t.fill_id} {t.side} {t.size}@{t.price}")

    @client.on_fill
    async def _(f: streaming.Fill) -> None:
        print(f"my fill {f.fill_id}")

    await client.subscribe([
        streaming.trades("0x..."),
        streaming.account_events(),
    ])
    await client.run()

**Iterator mode** — `async with` + `async for`. Single connection per
endpoint, no reconnect. Recommended for quick scripts and custom
dispatch pipelines.

    async with streaming.AgaraStreamClient() as ws:
        await ws.subscribe([streaming.orderbook("21742...")])
        async for frame in ws:
            match frame:
                case streaming.OrderbookSnapshot(bids=bids, asks=asks):
                    ...

Public channels (`orderbook`, `best_quote`, `market_status`, `trades`)
need no auth. The `account_events` channel needs a bearer token (Privy
JWT or PAT with `account:stream` scope) — pass it to the constructor.

The client transparently maintains up to two underlying WebSocket
connections — one for public channels, one for account events — and
routes subscriptions and frames to the right one. You don't need to
think about which endpoint serves which channel.
"""

from __future__ import annotations

import asyncio
import json
import random
import time
from dataclasses import dataclass
from typing import (
    Any,
    AsyncGenerator,
    AsyncIterator,
    Awaitable,
    Callable,
    Iterable,
    Literal,
    Optional,
    TypeVar,
    Union,
)
from urllib.parse import urlsplit, urlunsplit


DEFAULT_BASE_URL = "https://app.sandbox.agara.xyz"

_MARKET_PATH = "/trade/v1/market-stream"
_ACCOUNT_PATH = "/trade/v1/account-stream"

_PUBLIC_CHANNELS = frozenset({"orderbook", "best_quote", "market_status", "trades"})
_ACCOUNT_CHANNEL = "account_events"

#: Per-connection cap the server enforces; the SDK validates per-call so
#: callers get a clear `ValueError` instead of an async per-channel
#: `too_many_subscriptions` error frame. Necessary-but-not-sufficient
#: (server still caps the rolling total across multiple subscribe calls).
MAX_SUBSCRIPTIONS_PER_CONNECTION = 64

#: Application-level keepalive. The SDK sends `{"op": "ping"}` every
#: `_PING_INTERVAL_S` and treats a connection with no frame of any kind
#: for `_SILENCE_TIMEOUT_S` as wedged — a proxy that stopped forwarding,
#: a half-open socket — and drops it so the runner reconnects. The
#: server's ~10s `heartbeat` keeps a healthy connection well under the
#: timeout, so prolonged silence is a reliable dead-connection signal
#: that protocol-level pongs alone can miss.
_PING_INTERVAL_S = 25.0
_SILENCE_TIMEOUT_S = 35.0


# ── Channel descriptor ───────────────────────────────────────────────


@dataclass(frozen=True)
class Channel:
    """One channel-subscription descriptor. Build via the factories
    (`orderbook`, `best_quote`, `market_status`, `trades`,
    `account_events`)."""

    name: str
    token_id: Optional[str] = None
    condition_id: Optional[str] = None
    needs_token: bool = False

    @property
    def is_account(self) -> bool:
        return self.name == _ACCOUNT_CHANNEL

    def to_wire(self, bearer: Optional[str]) -> dict[str, Any]:
        wire: dict[str, Any] = {"name": self.name}
        if self.token_id is not None:
            wire["token_id"] = self.token_id
        if self.condition_id is not None:
            wire["condition_id"] = self.condition_id
        if self.needs_token:
            if not bearer:
                raise ValueError(
                    f"channel {self.name!r} requires a bearer token; "
                    "construct the client with `token=...` or call `set_token(...)`"
                )
            wire["token"] = bearer
        return wire


def orderbook(token_id: str) -> Channel:
    return Channel(name="orderbook", token_id=token_id)


def best_quote(token_id: str) -> Channel:
    return Channel(name="best_quote", token_id=token_id)


def market_status(condition_id: str) -> Channel:
    return Channel(name="market_status", condition_id=condition_id)


def trades(condition_id: str) -> Channel:
    return Channel(name="trades", condition_id=condition_id)


def account_events() -> Channel:
    return Channel(name=_ACCOUNT_CHANNEL, needs_token=True)


# ── Frame types ──────────────────────────────────────────────────────


@dataclass(frozen=True)
class Level:
    """Engine-native level. Convert to dollars / shares with the
    enclosing frame's `price_scale` / `size_scale`."""

    price: int
    size: int


@dataclass(frozen=True)
class OrderbookSnapshot:
    token_id: str
    sequence: int
    bids: list[Level]
    asks: list[Level]
    tick_size: int
    price_scale: int
    size_scale: int


@dataclass(frozen=True)
class OrderbookDelta:
    token_id: str
    sequence: int
    # Each entry replaces the level at that price; `size == 0` removes.
    bids: list[Level]
    asks: list[Level]
    price_scale: int
    size_scale: int


@dataclass(frozen=True)
class BestQuote:
    token_id: str
    sequence: int
    bid: Optional[Level]
    ask: Optional[Level]
    price_scale: int
    size_scale: int


@dataclass(frozen=True)
class MarketCreated:
    condition_id: str
    sequence: int
    num_outcomes: int
    tick_size: int
    price_scale: int
    size_scale: int
    min_price: int
    max_price: int
    cross_match_enabled: bool


@dataclass(frozen=True)
class MarketHalted:
    condition_id: str
    sequence: int


@dataclass(frozen=True)
class MarketResumed:
    condition_id: str
    sequence: int


@dataclass(frozen=True)
class OutcomeProposed:
    condition_id: str
    sequence: int
    proposed_token_id: str


@dataclass(frozen=True)
class MarketResolved:
    condition_id: str
    sequence: int
    winning_token_id: str


@dataclass(frozen=True)
class FeePolicyUpdated:
    condition_id: str
    sequence: int


@dataclass(frozen=True)
class CrossMatchToggled:
    condition_id: str
    sequence: int
    enabled: bool


@dataclass(frozen=True)
class Trade:
    condition_id: str
    sequence: int
    fill_id: str
    # Token traded on each side. NORMAL fills have
    # `taker_token_id == maker_token_id`. MINT / MERGE fills touch two
    # outcomes — taker on one side, maker on the other — so the two
    # differ. Consumers can render normal trades against either token;
    # mint / merge need both to fully attribute the collateral split.
    taker_token_id: str
    maker_token_id: str
    side: Literal["BUY", "SELL", "UNSPECIFIED"]
    # Engine-native; convert with `price / price_scale`. Self-describing
    # — scales travel with the event.
    price: int
    size: int
    price_scale: int
    size_scale: int
    settlement_mode: Literal["NORMAL", "MINT", "MERGE", "UNSPECIFIED"]


@dataclass(frozen=True)
class Fill:
    # Envelope-level sequence — monotonic across the whole account
    # event log for the subscribing user. Use it to detect gaps after a
    # `sequence_reset`.
    sequence: int
    role: Literal["TAKER", "MAKER"]
    fill_id: str
    # Agara UUID for the side this fill is rendered against (taker
    # order's UUID on `role="taker"`, maker order's on `role="maker"`).
    order_id: str
    # EIP-712 hash of the order this leg belongs to. For signed orders
    # it's the hash you computed before submitting — your correlation key.
    order_hash: str
    # The outcome token traded — YES or NO. The role-relative side:
    # on NORMAL fills both roles trade the same token; on MINT / MERGE
    # the two roles touch different tokens.
    token_id: str
    side: Literal["BUY", "SELL", "UNSPECIFIED"]
    # Engine-native units; convert with `price / price_scale` and
    # `size / size_scale`.
    price: int
    size: int
    price_scale: int
    size_scale: int
    settlement_mode: Literal["NORMAL", "MINT", "MERGE", "UNSPECIFIED"]
    fee_micro: int


@dataclass(frozen=True)
class OrderAccepted:
    sequence: int
    order_id: str
    order_hash: str
    token_id: str
    side: Literal["BUY", "SELL", "UNSPECIFIED"]
    price: int
    remaining_size: int
    price_scale: int
    size_scale: int
    tif: Literal["GTC", "FAK", "FOK", "UNSPECIFIED"]


@dataclass(frozen=True)
class OrderCancelled:
    sequence: int
    order_id: str
    order_hash: str
    token_id: str
    side: Literal["BUY", "SELL", "UNSPECIFIED"]
    price: int
    remaining_size: int
    price_scale: int
    size_scale: int
    reason: Literal["USER", "FAK_REMAINDER", "SELF_TRADE_PREVENTION", "MARKET_RESOLVED", "UNSPECIFIED"]


@dataclass(frozen=True)
class OrderRejected:
    # An accepted order failed at submit time (no balance, no shares,
    # post-only cross, unfillable, or a venue/infra failure). Router-originated,
    # so `sequence` is 0 (out-of-band) — dedupe by `order_id`.
    sequence: int
    order_id: str
    order_hash: str | None
    token_id: str | None
    reason: str


@dataclass(frozen=True)
class TokensMinted:
    sequence: int
    # Per-market identifier — mint affects both outcomes by
    # construction (one collateral → one YES + one NO).
    condition_id: str
    size: int
    size_scale: int


@dataclass(frozen=True)
class TokensMerged:
    sequence: int
    condition_id: str
    size: int
    size_scale: int


@dataclass(frozen=True)
class Subscribed:
    channel: str
    token_id: Optional[str] = None
    condition_id: Optional[str] = None


@dataclass(frozen=True)
class Unsubscribed:
    channel: str
    token_id: Optional[str] = None
    condition_id: Optional[str] = None


@dataclass(frozen=True)
class SequenceReset:
    channel: str
    # "lagged" (slow read) or "stream_reset" (brief upstream blip).
    # Recovery is identical either way: discard local state for the
    # subject and rebuild from the next update.
    reason: Optional[str] = None
    token_id: Optional[str] = None
    condition_id: Optional[str] = None


@dataclass(frozen=True)
class Heartbeat:
    server_time: str


@dataclass(frozen=True)
class Pong:
    server_time: str


@dataclass(frozen=True)
class StreamError:
    code: str
    message: str
    # "resubscribe" (one feed stopped; resubscribe the subject),
    # "reconnect" (reopen the socket with fresh credentials), or None
    # (terminal — the request was rejected and retrying won't help).
    # In callback mode the client acts on this automatically.
    action: Optional[str] = None
    channel: Optional[str] = None
    token_id: Optional[str] = None
    condition_id: Optional[str] = None


@dataclass(frozen=True)
class SubscriptionList:
    channels: list[dict[str, Any]]


@dataclass(frozen=True)
class ConnectContext:
    """Fires (in callback mode) every time an underlying endpoint
    connects. `endpoint` is `"market"` for the public stream and
    `"account"` for the account_events stream. `is_reconnect` is False
    on the first successful connection of a `run()` call for that
    endpoint, True thereafter."""

    endpoint: Literal["market", "account"]
    is_reconnect: bool
    attempt: int


@dataclass(frozen=True)
class UnknownFrame:
    """Catch-all so a server-side wire addition doesn't crash old clients."""

    raw: dict[str, Any]


AccountEvent = Union[Fill, OrderAccepted, OrderCancelled, OrderRejected, TokensMinted, TokensMerged]


Frame = Union[
    OrderbookSnapshot,
    OrderbookDelta,
    BestQuote,
    MarketCreated,
    MarketHalted,
    MarketResumed,
    OutcomeProposed,
    MarketResolved,
    FeePolicyUpdated,
    CrossMatchToggled,
    Trade,
    Fill,
    OrderAccepted,
    OrderCancelled,
    OrderRejected,
    TokensMinted,
    TokensMerged,
    Subscribed,
    Unsubscribed,
    SequenceReset,
    Heartbeat,
    Pong,
    StreamError,
    SubscriptionList,
    UnknownFrame,
]


@dataclass(frozen=True)
class Reconnect:
    """Backoff config for callback-mode reconnect. Set
    `max_attempts=None` for forever; `0` to disable reconnect entirely
    (one shot)."""

    initial_delay: float = 0.25
    max_delay: float = 8.0
    jitter: float = 0.2
    max_attempts: Optional[int] = None


# ── Frame decoder ────────────────────────────────────────────────────


def _level(entry: list[Any]) -> Level:
    return Level(price=int(entry[0]), size=int(entry[1]))


def _optional_level(entry: Optional[dict[str, Any]]) -> Optional[Level]:
    if entry is None:
        return None
    return Level(price=int(entry["price"]), size=int(entry["size"]))


def _int(value: Any) -> int:
    return int(value) if value is not None else 0


def _opt_str(value: Any) -> Optional[str]:
    return str(value) if value is not None else None


_MARKET_STATUS_DECODERS: dict[str, Callable[[str, int, dict[str, Any]], Frame]] = {
    "market_created": lambda cid, seq, d: MarketCreated(
        condition_id=cid,
        sequence=seq,
        num_outcomes=int(d["num_outcomes"]),
        tick_size=int(d["tick_size"]),
        price_scale=int(d["price_scale"]),
        size_scale=int(d["size_scale"]),
        min_price=int(d["min_price"]),
        max_price=int(d["max_price"]),
        cross_match_enabled=bool(d["cross_match_enabled"]),
    ),
    "market_halted": lambda cid, seq, _: MarketHalted(cid, seq),
    "market_resumed": lambda cid, seq, _: MarketResumed(cid, seq),
    "outcome_proposed": lambda cid, seq, d: OutcomeProposed(
        condition_id=cid, sequence=seq, proposed_token_id=str(d["proposed_token_id"])
    ),
    "market_resolved": lambda cid, seq, d: MarketResolved(
        condition_id=cid, sequence=seq, winning_token_id=str(d["winning_token_id"])
    ),
    "fee_policy_updated": lambda cid, seq, _: FeePolicyUpdated(cid, seq),
    "cross_match_toggled": lambda cid, seq, d: CrossMatchToggled(
        condition_id=cid, sequence=seq, enabled=bool(d["enabled"])
    ),
}


def _decode_fill(seq: int, d: dict[str, Any]) -> Fill:
    return Fill(
        sequence=seq,
        role=d["role"],
        fill_id=str(d["fill_id"]),
        order_id=str(d["order_id"]),
        order_hash=str(d["order_hash"]),
        token_id=str(d["token_id"]),
        side=d["side"],
        price=_int(d["price"]),
        size=_int(d["size"]),
        price_scale=_int(d["price_scale"]),
        size_scale=_int(d["size_scale"]),
        settlement_mode=d["settlement_mode"],
        fee_micro=_int(d["fee_micro"]),
    )


def _decode_order_accepted(seq: int, d: dict[str, Any]) -> OrderAccepted:
    return OrderAccepted(
        sequence=seq,
        order_id=str(d["order_id"]),
        order_hash=str(d["order_hash"]),
        token_id=str(d["token_id"]),
        side=d["side"],
        price=_int(d["price"]),
        remaining_size=_int(d["remaining_size"]),
        price_scale=_int(d["price_scale"]),
        size_scale=_int(d["size_scale"]),
        tif=d["tif"],
    )


def _decode_order_cancelled(seq: int, d: dict[str, Any]) -> OrderCancelled:
    return OrderCancelled(
        sequence=seq,
        order_id=str(d["order_id"]),
        order_hash=str(d["order_hash"]),
        token_id=str(d["token_id"]),
        side=d["side"],
        price=_int(d["price"]),
        remaining_size=_int(d["remaining_size"]),
        price_scale=_int(d["price_scale"]),
        size_scale=_int(d["size_scale"]),
        reason=d["reason"],
    )


def _decode_order_rejected(seq: int, d: dict[str, Any]) -> OrderRejected:
    order_hash = d.get("order_hash")
    token_id = d.get("token_id")
    return OrderRejected(
        sequence=seq,
        order_id=str(d["order_id"]),
        order_hash=str(order_hash) if order_hash is not None else None,
        token_id=str(token_id) if token_id is not None else None,
        reason=str(d.get("reason", "")),
    )


def _decode_tokens_minted(seq: int, d: dict[str, Any]) -> TokensMinted:
    return TokensMinted(
        sequence=seq,
        condition_id=str(d["condition_id"]),
        size=_int(d["size"]),
        size_scale=_int(d["size_scale"]),
    )


def _decode_tokens_merged(seq: int, d: dict[str, Any]) -> TokensMerged:
    return TokensMerged(
        sequence=seq,
        condition_id=str(d["condition_id"]),
        size=_int(d["size"]),
        size_scale=_int(d["size_scale"]),
    )


_ACCOUNT_EVENT_DECODERS: dict[str, Callable[[int, dict[str, Any]], AccountEvent]] = {
    "fill": _decode_fill,
    "order_accepted": _decode_order_accepted,
    "order_cancelled": _decode_order_cancelled,
    "order_rejected": _decode_order_rejected,
    "tokens_minted": _decode_tokens_minted,
    "tokens_merged": _decode_tokens_merged,
}


def decode_frame(raw: dict[str, Any]) -> Frame:
    """Decode one parsed server frame. Unknown shapes fall through to
    [`UnknownFrame`] so a server-side wire addition doesn't crash old
    clients."""
    op = raw.get("op")
    if op == "update":
        return _decode_update(raw)
    if op == "subscribed":
        return Subscribed(
            channel=raw["channel"],
            token_id=raw.get("token_id"),
            condition_id=raw.get("condition_id"),
        )
    if op == "unsubscribed":
        return Unsubscribed(
            channel=raw["channel"],
            token_id=raw.get("token_id"),
            condition_id=raw.get("condition_id"),
        )
    if op == "sequence_reset":
        return SequenceReset(
            channel=raw["channel"],
            reason=raw.get("reason"),
            token_id=raw.get("token_id"),
            condition_id=raw.get("condition_id"),
        )
    if op == "heartbeat":
        return Heartbeat(server_time=raw["server_time"])
    if op == "pong":
        return Pong(server_time=raw["server_time"])
    if op == "error":
        return StreamError(
            code=raw["code"],
            message=raw["message"],
            action=raw.get("action"),
            channel=raw.get("channel"),
            token_id=raw.get("token_id"),
            condition_id=raw.get("condition_id"),
        )
    if op == "subscription_list":
        return SubscriptionList(channels=list(raw.get("channels", [])))
    return UnknownFrame(raw=raw)


def _decode_update(raw: dict[str, Any]) -> Frame:
    channel = raw["channel"]
    sequence = int(raw["sequence"])
    data = raw.get("data") or {}
    if channel == "orderbook":
        token_id = raw["token_id"]
        bids = [_level(lvl) for lvl in data.get("bids", [])]
        asks = [_level(lvl) for lvl in data.get("asks", [])]
        kind = data.get("kind")
        if kind == "snapshot":
            return OrderbookSnapshot(
                token_id=token_id,
                sequence=sequence,
                bids=bids,
                asks=asks,
                tick_size=_int(data["tick_size"]),
                price_scale=_int(data["price_scale"]),
                size_scale=_int(data["size_scale"]),
            )
        if kind == "delta":
            return OrderbookDelta(
                token_id=token_id,
                sequence=sequence,
                bids=bids,
                asks=asks,
                price_scale=_int(data["price_scale"]),
                size_scale=_int(data["size_scale"]),
            )
        return UnknownFrame(raw=raw)
    if channel == "best_quote":
        return BestQuote(
            token_id=raw["token_id"],
            sequence=sequence,
            bid=_optional_level(data.get("bid")),
            ask=_optional_level(data.get("ask")),
            price_scale=_int(data["price_scale"]),
            size_scale=_int(data["size_scale"]),
        )
    if channel == "market_status":
        kind = data.get("kind")
        decoder = _MARKET_STATUS_DECODERS.get(kind) if kind else None
        if decoder is None:
            return UnknownFrame(raw=raw)
        return decoder(raw["condition_id"], sequence, data)
    if channel == "trades":
        if data.get("kind") != "trade":
            return UnknownFrame(raw=raw)
        return Trade(
            condition_id=raw["condition_id"],
            sequence=sequence,
            fill_id=str(data["fill_id"]),
            taker_token_id=str(data["taker_token_id"]),
            maker_token_id=str(data["maker_token_id"]),
            side=data["side"],
            price=_int(data["price"]),
            size=_int(data["size"]),
            price_scale=_int(data["price_scale"]),
            size_scale=_int(data["size_scale"]),
            settlement_mode=data["settlement_mode"],
        )
    if channel == _ACCOUNT_CHANNEL:
        kind = data.get("kind")
        decoder = _ACCOUNT_EVENT_DECODERS.get(kind) if kind else None
        if decoder is None:
            return UnknownFrame(raw=raw)
        return decoder(sequence, data)
    return UnknownFrame(raw=raw)


# ── URL + backoff helpers ────────────────────────────────────────────


def _ws_url(base_url: str, path: str) -> str:
    parts = urlsplit(base_url)
    if not parts.scheme:
        return base_url.rstrip("/") + path
    scheme = {"http": "ws", "https": "wss"}.get(parts.scheme, parts.scheme)
    full_path = parts.path.rstrip("/") + path
    return urlunsplit((scheme, parts.netloc, full_path, parts.query, parts.fragment))


def _backoff(cfg: Reconnect, attempt: int) -> float:
    base = min(cfg.max_delay, cfg.initial_delay * (2 ** (attempt - 1)))
    if cfg.jitter <= 0:
        return base
    spread = base * cfg.jitter
    return max(0.0, base + random.uniform(-spread, spread))


# ── Internal: per-endpoint connection ────────────────────────────────


T = TypeVar("T")
Handler = Callable[[T], Awaitable[None]]


@dataclass
class _Disconnected:
    endpoint: Literal["market", "account"]


@dataclass
class _RunnerStopped:
    """Sentinel pushed when a callback-mode runner permanently exits
    (max_attempts reached or `stop()` called). Dispatch counts down and
    exits when none remain."""

    endpoint: Literal["market", "account"]


_EndpointKind = Literal["market", "account"]


def _endpoint_for(channel: Channel) -> _EndpointKind:
    return "account" if channel.is_account else "market"


def _endpoint_kind_for_name(channel: Optional[str]) -> _EndpointKind:
    return "account" if channel == _ACCOUNT_CHANNEL else "market"


def _group_by_endpoint(channels: Iterable[Channel]) -> dict[_EndpointKind, list[Channel]]:
    out: dict[_EndpointKind, list[Channel]] = {"market": [], "account": []}
    for ch in channels:
        out[_endpoint_for(ch)].append(ch)
    return out


class _Endpoint:
    """One physical WebSocket. Owns the connect/send/read lifecycle for
    a single URL. The client composes one or both of these."""

    def __init__(self, url: str, kind: _EndpointKind) -> None:
        self._url = url
        self._kind: _EndpointKind = kind
        self._ws: Any = None
        self._reader: Optional[asyncio.Task[None]] = None
        self._keepalive: Optional[asyncio.Task[None]] = None
        self._last_rx: float = 0.0

    @property
    def is_open(self) -> bool:
        return self._ws is not None

    async def open(self, queue: "asyncio.Queue[Any]") -> None:
        try:
            import websockets
        except ImportError as exc:
            raise RuntimeError(
                "agara_sdk.streaming requires the `websockets` extra: "
                "`pip install 'agara-sdk[streaming]'`"
            ) from exc
        self._ws = await websockets.connect(self._url)
        self._last_rx = time.monotonic()
        self._reader = asyncio.create_task(self._read_loop(queue))
        self._keepalive = asyncio.create_task(self._keepalive_loop())

    async def close(self) -> None:
        for task in (self._keepalive, self._reader):
            if task is not None:
                task.cancel()
                try:
                    await task
                except (asyncio.CancelledError, Exception):
                    pass
        self._keepalive = None
        self._reader = None
        if self._ws is not None:
            try:
                await self._ws.close()
            except Exception:
                pass
            self._ws = None

    async def send(self, payload: dict[str, Any]) -> None:
        if self._ws is None:
            raise RuntimeError(
                f"endpoint {self._kind!r} is not connected"
            )
        await self._ws.send(json.dumps(payload))

    async def _read_loop(self, queue: "asyncio.Queue[Any]") -> None:
        from websockets.exceptions import ConnectionClosed

        try:
            async for raw in self._ws:
                self._last_rx = time.monotonic()
                try:
                    parsed = json.loads(raw)
                except json.JSONDecodeError:
                    continue
                await queue.put(decode_frame(parsed))
        except ConnectionClosed:
            pass
        except asyncio.CancelledError:
            return
        except Exception as exc:
            await queue.put(
                StreamError(code="transport", message=repr(exc))
            )
        finally:
            await queue.put(_Disconnected(endpoint=self._kind))

    async def _keepalive_loop(self) -> None:
        """Send an application `ping` on an interval and drop a wedged
        connection — one whose frames (including the server heartbeat)
        have stopped arriving — so the caller reconnects. Closing the
        socket ends `_read_loop`, which emits `_Disconnected`."""
        try:
            while True:
                await asyncio.sleep(_PING_INTERVAL_S)
                if self._ws is None:
                    return
                if time.monotonic() - self._last_rx > _SILENCE_TIMEOUT_S:
                    await self._ws.close()
                    return
                try:
                    await self._ws.send(json.dumps({"op": "ping"}))
                except Exception:
                    return
        except asyncio.CancelledError:
            return


# ── Client ───────────────────────────────────────────────────────────


class AgaraStreamClient:
    """Async WebSocket client for the agara market and account streams.

    Two ergonomic modes share one client.

    **Callback mode** — register handlers with the `on_*` decorators,
    then `await client.run()`. Auto-reconnects on disconnect; replays
    the subscription set on every reconnect.

    **Iterator mode** — `async with client as ws:`, then
    `async for frame in ws:`. Single connection per endpoint, no
    reconnect; the iterator ends when both endpoints have closed.

    Pass `token=...` to subscribe to `account_events`. Public channels
    need no auth.
    """

    def __init__(
        self,
        token: Optional[str] = None,
        base_url: str = DEFAULT_BASE_URL,
        reconnect: Optional[Reconnect] = None,
    ) -> None:
        self._market = _Endpoint(_ws_url(base_url, _MARKET_PATH), "market")
        self._account = _Endpoint(_ws_url(base_url, _ACCOUNT_PATH), "account")
        self._token = token
        self._reconnect = reconnect or Reconnect()
        self._subscriptions: list[Channel] = []
        self._handlers: dict[type, list[Handler[Any]]] = {}
        self._queue: Optional["asyncio.Queue[Any]"] = None
        self._stopped = False
        self._mode: Optional[Literal["callback", "iterator"]] = None
        self._open_endpoints: set[_EndpointKind] = set()
        self._connection_count: dict[_EndpointKind, int] = {"market": 0, "account": 0}
        self._consecutive_failures: dict[_EndpointKind, int] = {"market": 0, "account": 0}
        self._runners: dict[_EndpointKind, asyncio.Task[None]] = {}
        self._disconnect_events: dict[_EndpointKind, asyncio.Event] = {}

    def set_token(self, token: Optional[str]) -> None:
        """Replace the bearer used for `account_events`. Takes effect on
        the next (re)connect of the account endpoint."""
        self._token = token

    # ── Subscription management ──

    async def subscribe(self, channels: Iterable[Channel]) -> None:
        """Add channels to the desired subscription set.

        - Before `run()` / `__aenter__`: just records; the channels are
          sent on first connect.
        - In iterator mode: opens any newly-needed endpoint on demand
          and sends the subscribe op.
        - In callback mode mid-`run()`: sends to already-open endpoints
          immediately. Channels for an endpoint that isn't part of the
          run's initial set won't connect until the next `run()`.
        """
        materialized = list(channels)
        new = [c for c in materialized if c not in self._subscriptions]
        if not new:
            return
        if len(self._subscriptions) + len(new) > MAX_SUBSCRIPTIONS_PER_CONNECTION:
            raise ValueError(
                f"subscription set would exceed MAX_SUBSCRIPTIONS_PER_CONNECTION "
                f"({MAX_SUBSCRIPTIONS_PER_CONNECTION})"
            )
        for ch in new:
            if ch.is_account and not self._token:
                raise ValueError(
                    "subscribing to account_events requires a bearer token; "
                    "construct the client with `token=...` or call `set_token(...)`"
                )
        self._subscriptions.extend(new)
        by_kind = _group_by_endpoint(new)
        for kind, group in by_kind.items():
            if not group:
                continue
            ep = self._endpoint(kind)
            if ep.is_open:
                await ep.send(self._build_op("subscribe", group))
                continue
            if self._mode == "iterator":
                await self._open_endpoint(kind)
            # callback-mode + endpoint-not-open: the runner will pick
            # this up on its next connect (or never, if the runner for
            # this endpoint wasn't part of run()'s initial set).

    async def unsubscribe(self, channels: Iterable[Channel]) -> None:
        """Remove channels from the subscription set. Sends `unsubscribe`
        to any endpoint that currently holds the channel; never opens an
        endpoint just to unsubscribe."""
        removed = [c for c in channels if c in self._subscriptions]
        for c in removed:
            self._subscriptions.remove(c)
        if not removed:
            return
        by_kind = _group_by_endpoint(removed)
        for kind, group in by_kind.items():
            if not group:
                continue
            ep = self._endpoint(kind)
            if ep.is_open:
                await ep.send(self._build_op("unsubscribe", group))

    async def ping(self) -> None:
        """Send `ping` on every connected endpoint."""
        await self._broadcast({"op": "ping"})

    async def list_subscriptions(self) -> None:
        """Send `list` on every connected endpoint. Each one replies with
        its own `subscription_list` frame."""
        await self._broadcast({"op": "list"})

    # ── Handler registration ──

    def _register(self, cls: type, fn: Handler[Any]) -> Handler[Any]:
        self._handlers.setdefault(cls, []).append(fn)
        return fn

    def on_orderbook_snapshot(
        self, fn: Handler[OrderbookSnapshot]
    ) -> Handler[OrderbookSnapshot]:
        return self._register(OrderbookSnapshot, fn)

    def on_orderbook_delta(self, fn: Handler[OrderbookDelta]) -> Handler[OrderbookDelta]:
        return self._register(OrderbookDelta, fn)

    def on_best_quote(self, fn: Handler[BestQuote]) -> Handler[BestQuote]:
        return self._register(BestQuote, fn)

    def on_market_created(self, fn: Handler[MarketCreated]) -> Handler[MarketCreated]:
        return self._register(MarketCreated, fn)

    def on_market_halted(self, fn: Handler[MarketHalted]) -> Handler[MarketHalted]:
        return self._register(MarketHalted, fn)

    def on_market_resumed(self, fn: Handler[MarketResumed]) -> Handler[MarketResumed]:
        return self._register(MarketResumed, fn)

    def on_outcome_proposed(self, fn: Handler[OutcomeProposed]) -> Handler[OutcomeProposed]:
        return self._register(OutcomeProposed, fn)

    def on_market_resolved(self, fn: Handler[MarketResolved]) -> Handler[MarketResolved]:
        return self._register(MarketResolved, fn)

    def on_fee_policy_updated(
        self, fn: Handler[FeePolicyUpdated]
    ) -> Handler[FeePolicyUpdated]:
        return self._register(FeePolicyUpdated, fn)

    def on_cross_match_toggled(
        self, fn: Handler[CrossMatchToggled]
    ) -> Handler[CrossMatchToggled]:
        return self._register(CrossMatchToggled, fn)

    def on_trade(self, fn: Handler[Trade]) -> Handler[Trade]:
        return self._register(Trade, fn)

    def on_fill(self, fn: Handler[Fill]) -> Handler[Fill]:
        return self._register(Fill, fn)

    def on_order_accepted(self, fn: Handler[OrderAccepted]) -> Handler[OrderAccepted]:
        return self._register(OrderAccepted, fn)

    def on_order_cancelled(self, fn: Handler[OrderCancelled]) -> Handler[OrderCancelled]:
        return self._register(OrderCancelled, fn)

    def on_order_rejected(self, fn: Handler[OrderRejected]) -> Handler[OrderRejected]:
        return self._register(OrderRejected, fn)

    def on_tokens_minted(self, fn: Handler[TokensMinted]) -> Handler[TokensMinted]:
        return self._register(TokensMinted, fn)

    def on_tokens_merged(self, fn: Handler[TokensMerged]) -> Handler[TokensMerged]:
        return self._register(TokensMerged, fn)

    def on_connect(self, fn: Handler[ConnectContext]) -> Handler[ConnectContext]:
        return self._register(ConnectContext, fn)

    def on_sequence_reset(self, fn: Handler[SequenceReset]) -> Handler[SequenceReset]:
        return self._register(SequenceReset, fn)

    def on_subscribed(self, fn: Handler[Subscribed]) -> Handler[Subscribed]:
        return self._register(Subscribed, fn)

    def on_unsubscribed(self, fn: Handler[Unsubscribed]) -> Handler[Unsubscribed]:
        return self._register(Unsubscribed, fn)

    def on_heartbeat(self, fn: Handler[Heartbeat]) -> Handler[Heartbeat]:
        return self._register(Heartbeat, fn)

    def on_pong(self, fn: Handler[Pong]) -> Handler[Pong]:
        return self._register(Pong, fn)

    def on_subscription_list(
        self, fn: Handler[SubscriptionList]
    ) -> Handler[SubscriptionList]:
        return self._register(SubscriptionList, fn)

    def on_error(self, fn: Handler[StreamError]) -> Handler[StreamError]:
        return self._register(StreamError, fn)

    def on_unknown(self, fn: Handler[UnknownFrame]) -> Handler[UnknownFrame]:
        return self._register(UnknownFrame, fn)

    # ── Iterator mode ──

    async def __aenter__(self) -> "AgaraStreamClient":
        if self._mode is not None:
            raise RuntimeError(
                "AgaraStreamClient is already active; one mode per instance"
            )
        self._mode = "iterator"
        self._queue = asyncio.Queue()
        # Open only the endpoints that have subscriptions queued up. If
        # the user subscribes to a new endpoint mid-iteration, that
        # endpoint is opened on demand by `subscribe()`.
        needed = self._needed_endpoints()
        for kind in needed:
            await self._open_endpoint(kind)
        return self

    async def __aexit__(self, *_exc: object) -> None:
        await self._close_all()
        self._mode = None
        self._queue = None
        self._open_endpoints.clear()

    def __aiter__(self) -> AsyncIterator[Frame]:
        return self._frames()

    async def _frames(self) -> AsyncGenerator[Frame, None]:
        assert self._queue is not None, "use `async with` before iterating"
        while self._open_endpoints:
            item = await self._queue.get()
            if isinstance(item, _Disconnected):
                self._open_endpoints.discard(item.endpoint)
                continue
            yield item

    # ── Callback mode ──

    async def run(self) -> None:
        """Connect, dispatch frames to handlers, and reconnect on drop
        until `stop()` is called or `max_attempts` consecutive failures
        are reached on each endpoint. Replays the subscription set on
        every (re)connect.

        Endpoints are opened lazily — `market` opens iff a public
        subscription exists; `account` opens iff `account_events` is
        subscribed."""
        if self._mode is not None:
            raise RuntimeError(
                "AgaraStreamClient is already active; one mode per instance"
            )
        self._mode = "callback"
        self._queue = asyncio.Queue()
        self._stopped = False
        try:
            needed = self._needed_endpoints()
            if not needed:
                raise RuntimeError(
                    "run() requires at least one subscription; "
                    "call subscribe() before run()"
                )
            for kind in needed:
                self._disconnect_events[kind] = asyncio.Event()
                self._runners[kind] = asyncio.create_task(self._runner(kind))
            await self._dispatch_loop(needed)
        finally:
            for t in self._runners.values():
                t.cancel()
            for t in self._runners.values():
                try:
                    await t
                except (asyncio.CancelledError, Exception):
                    pass
            self._runners.clear()
            self._disconnect_events.clear()
            await self._close_all()
            self._mode = None
            self._queue = None
            self._open_endpoints.clear()

    async def stop(self) -> None:
        """Signal `run()` to exit after the current frame; close all
        open endpoints."""
        self._stopped = True
        await self._close_all()

    # ── Internals ──

    def _needed_endpoints(self) -> set[_EndpointKind]:
        kinds: set[_EndpointKind] = set()
        for ch in self._subscriptions:
            kinds.add(_endpoint_for(ch))
        return kinds

    def _endpoint(self, kind: _EndpointKind) -> _Endpoint:
        return self._market if kind == "market" else self._account

    async def _open_endpoint(self, kind: _EndpointKind) -> None:
        ep = self._endpoint(kind)
        assert self._queue is not None
        await ep.open(self._queue)
        self._open_endpoints.add(kind)
        await self._replay_subscriptions(kind)

    async def _replay_subscriptions(self, kind: _EndpointKind) -> None:
        """Send the endpoint's current subscription set as one `subscribe`
        op — on first connect, on reconnect, and to re-establish a feed
        after a `resubscribe` action."""
        ep = self._endpoint(kind)
        my_subs = [c for c in self._subscriptions if _endpoint_for(c) == kind]
        if ep.is_open and my_subs:
            await ep.send(self._build_op("subscribe", my_subs))

    async def _close_endpoint(self, kind: _EndpointKind) -> None:
        await self._endpoint(kind).close()
        self._open_endpoints.discard(kind)

    async def _close_all(self) -> None:
        await asyncio.gather(
            self._market.close(),
            self._account.close(),
            return_exceptions=True,
        )

    def _build_op(self, op: str, channels: list[Channel]) -> dict[str, Any]:
        return {
            "op": op,
            "channels": [c.to_wire(self._token) for c in channels],
        }

    async def _broadcast(self, payload: dict[str, Any]) -> None:
        for ep in (self._market, self._account):
            if ep.is_open:
                await ep.send(payload)

    async def _runner(self, kind: _EndpointKind) -> None:
        """Per-endpoint connect → drain → reconnect loop. Emits
        `_RunnerStopped(kind)` on permanent exit so the dispatch loop
        knows when no more frames will arrive."""
        ep = self._endpoint(kind)
        assert self._queue is not None
        try:
            while not self._stopped:
                try:
                    await self._open_endpoint(kind)
                except asyncio.CancelledError:
                    raise
                except Exception as exc:
                    await self._dispatch(StreamError(code="transport", message=repr(exc)))
                    self._consecutive_failures[kind] += 1
                    if self._stop_after_failures(kind):
                        return
                    await asyncio.sleep(
                        _backoff(self._reconnect, self._consecutive_failures[kind])
                    )
                    continue
                self._connection_count[kind] += 1
                is_reconnect = self._connection_count[kind] > 1
                self._consecutive_failures[kind] = 0
                await self._dispatch(
                    ConnectContext(
                        endpoint=kind,
                        is_reconnect=is_reconnect,
                        attempt=self._connection_count[kind],
                    )
                )
                event = self._disconnect_events[kind]
                event.clear()
                await event.wait()
                await ep.close()
                if self._stopped:
                    return
                self._consecutive_failures[kind] += 1
                if self._stop_after_failures(kind):
                    return
                await asyncio.sleep(
                    _backoff(self._reconnect, self._consecutive_failures[kind])
                )
        finally:
            await self._queue.put(_RunnerStopped(endpoint=kind))

    def _stop_after_failures(self, kind: _EndpointKind) -> bool:
        cap = self._reconnect.max_attempts
        return cap is not None and self._consecutive_failures[kind] >= cap

    async def _dispatch_loop(self, kinds: set[_EndpointKind]) -> None:
        """Drain the queue, dispatch frames, exit once every runner has
        signaled it permanently stopped."""
        assert self._queue is not None
        remaining = set(kinds)
        while remaining:
            item = await self._queue.get()
            if isinstance(item, _Disconnected):
                self._open_endpoints.discard(item.endpoint)
                ev = self._disconnect_events.get(item.endpoint)
                if ev is not None:
                    ev.set()
                continue
            if isinstance(item, _RunnerStopped):
                remaining.discard(item.endpoint)
                continue
            await self._dispatch(item)
            if isinstance(item, StreamError) and item.action:
                await self._handle_error_action(item)

    async def _handle_error_action(self, err: StreamError) -> None:
        """Callback-mode auto-recovery for `error` frames that carry an
        `action`. Runs after user handlers, so an `on_error` handler can
        refresh the token (via `set_token`) before a `reconnect`.

        - `resubscribe`: one feed stopped; replay the affected
          endpoint's subscription set to re-establish it.
        - `reconnect`: drop the affected endpoint so the runner reopens
          it with backoff (and whatever token is now set).
        """
        kind = _endpoint_kind_for_name(err.channel)
        if err.action == "resubscribe":
            await self._replay_subscriptions(kind)
        elif err.action == "reconnect":
            await self._endpoint(kind).close()

    async def _dispatch(self, payload: Any) -> None:
        handlers = self._handlers.get(type(payload))
        if not handlers:
            return
        for fn in handlers:
            try:
                await fn(payload)
            except asyncio.CancelledError:
                raise
            except Exception as exc:
                if isinstance(payload, StreamError):
                    return
                err_handlers = self._handlers.get(StreamError)
                if not err_handlers:
                    return
                err = StreamError(
                    code="handler_exception",
                    message=f"{type(exc).__name__}: {exc}",
                )
                for ef in err_handlers:
                    try:
                        await ef(err)
                    except Exception:
                        pass


__all__ = [
    "DEFAULT_BASE_URL",
    "MAX_SUBSCRIPTIONS_PER_CONNECTION",
    "AccountEvent",
    "AgaraStreamClient",
    "BestQuote",
    "Channel",
    "ConnectContext",
    "CrossMatchToggled",
    "FeePolicyUpdated",
    "Fill",
    "Frame",
    "Heartbeat",
    "Level",
    "MarketCreated",
    "MarketHalted",
    "MarketResolved",
    "MarketResumed",
    "OrderAccepted",
    "OrderCancelled",
    "OrderRejected",
    "OrderbookDelta",
    "OrderbookSnapshot",
    "OutcomeProposed",
    "Pong",
    "Reconnect",
    "SequenceReset",
    "StreamError",
    "Subscribed",
    "SubscriptionList",
    "TokensMerged",
    "TokensMinted",
    "Trade",
    "Unsubscribed",
    "UnknownFrame",
    "account_events",
    "best_quote",
    "decode_frame",
    "market_status",
    "orderbook",
    "trades",
]

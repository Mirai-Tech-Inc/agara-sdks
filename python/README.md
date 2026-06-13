# agara-sdk (Python)

Minimal Python client for the agara prediction-markets trading API.

## Install

```bash
pip install agara-sdk
```

Requires Python 3.10+.

## Quickstart

```python
from agara_sdk import AgaraClient

with AgaraClient(token="agt_…") as client:
    # Read the orderbook
    book = client.get_orderbook(token_id)
    print(f"best bid {book.best_bid} / ask {book.best_ask}")

    # Place a limit order
    resp = client.place_order(
        token_id=token_id,
        side="BUY",
        price=0.60,    # dollars per share
        shares=1.0,    # shares
    )
    order_id = resp["order_id"]

    # Wait for it to fill (or time out)
    final = client.wait_for_terminal(order_id, timeout=30.0)
    print(f"final status: {final['status']}")
```

See [examples/trading.py](./examples/trading.py) for a complete
working script.

## Async

Prefer `async`/`await`? The `[async]` extra adds `AsyncAgaraClient`, an
async-native client backed by `httpx` that mirrors `AgaraClient` method
for method — same arguments, same return shapes, same exceptions:

```bash
pip install 'agara-sdk[async]'
```

```python
import asyncio
from agara_sdk.aio import AsyncAgaraClient

async def main():
    async with AsyncAgaraClient(token="agt_…") as client:
        book = await client.get_orderbook(token_id)
        resp = await client.place_order(
            token_id=token_id, side="BUY", price=0.60, shares=1.0,
        )
        final = await client.wait_for_terminal(resp["order_id"], timeout=30.0)
        print(final["status"])

asyncio.run(main())
```

One `AsyncAgaraClient` is safe to use from many concurrent tasks on the
same event loop — unlike the sync client's `requests.Session`, you don't
need one per worker.

## What this SDK does

- Wraps `Authorization: Bearer agt_…` so you set the token once.
- Translates dollar / share amounts to the API's micro-encoded string
  format outbound, parses them back inbound. You think in dollars
  and shares; the wire details stay hidden.
- Maps HTTP status codes to a small exception hierarchy:
  `AuthError` (401, 403), `NotFoundError`, `ConflictError`,
  `RejectedError` (422), `ServerError` (5xx). All inherit from
  `AgaraError`.
- Provides `wait_for_terminal` for the place-and-poll pattern.
- Implements the context-manager protocol so `with AgaraClient(...) as c:`
  closes the underlying connection cleanly.

## Streaming

The `[streaming]` extra adds an async WebSocket client with typed
frames, callback or iterator dispatch, and auto-reconnect:

```bash
pip install 'agara-sdk[streaming]'
```

```python
import asyncio, os
from agara_sdk import streaming

async def main():
    client = streaming.AgaraStreamClient(token=os.environ["AGARA_TOKEN"])

    @client.on_trade
    async def _(t: streaming.Trade):
        print(f"trade {t.side} {t.size}@{t.price}")

    @client.on_fill
    async def _(f: streaming.Fill):
        print(f"my fill {f.fill_id}")

    await client.subscribe([
        streaming.trades("0x2174…"),
        streaming.account_events(),
    ])
    await client.run()

asyncio.run(main())
```

One client transparently handles both the public market stream
(orderbook, best_quote, trades, market_status) and the private
account_events stream. Full reference at
[`/docs/sdks/python/reference#streaming`](https://d3r180aqvl5ynd.cloudfront.net/docs/sdks/python/reference#streaming).

## What it doesn't do (compose on top)

- Automatic retries on REST. 5xx surfaces as `ServerError`; pick your
  backoff.

## Getting a token

API tokens are issued from the web app's **Settings → API tokens**
page. See [the authentication guide](https://d3r180aqvl5ynd.cloudfront.net/docs/authentication)
for full details on scopes and revocation.

The recommended trading-bot scope set:

```
portfolio:read
orders:read
orders:place
orders:cancel
orders:cancel_all
```

Reading the orderbook is public — no scope needed.

## Reference

### `AgaraClient(token, base_url="https://d3r180aqvl5ynd.cloudfront.net", timeout=10.0, session=None)`

Constructor. `session` lets you inject a configured `requests.Session`
(custom retries, connection pooling, etc.); a default one is created
if you don't.

**Thread safety.** Each `AgaraClient` wraps a single `requests.Session`,
which is **not safe to share across threads** — concurrent requests can
interleave and surface as mixed-up responses. If you're running a
multi-threaded bot, create one client per thread (or wrap calls in your
own lock). A single-threaded loop doesn't need to think about this.

### Orderbook

```python
book = client.get_orderbook(token_id)        # → Orderbook
book.bids, book.asks                          # list[OrderbookLevel]
book.best_bid, book.best_ask, book.mid, book.spread
```

### Orders

```python
client.place_order(
    *,
    token_id,
    side,                       # "BUY" | "SELL"
    price,                      # dollars per share
    shares=None,                # shares (set this OR collateral_amount)
    collateral_amount=None,     # dollars (BUY only)
    time_in_force="GTC",        # "GTC" | "FAK" | "FOK" | "GTD"
    post_only=False,
    expiration_unix_seconds=None,
)                                # → dict (see API docs)

client.list_orders(limit=500, cursor=None)    # → dict: `orders`, `pagination.next_cursor`, …
client.get_order(order_id)                    # → dict with `order`
client.cancel_order(order_id)                 # → dict
client.cancel_all_orders()                    # → dict
```

### Trades

```python
client.list_trades(limit=500, cursor=None)    # → dict: `trades`, `pagination.next_cursor`, …
```

### Helpers

```python
client.wait_for_terminal(
    order_id,
    timeout=30.0,
    poll_interval=1.0,
)                                # → final order dict; check status against TERMINAL_STATUSES

from agara_sdk import micro_to_float
micro_to_float("600000")          # 0.60
micro_to_float(None)              # None — handy for nullable response fields
```

### Exceptions

```python
from agara_sdk import (
    AgaraError,        # base — also catches uncategorized statuses
    BadRequestError,   # 400 — malformed body or invalid parameters
    AuthError,         # 401 / 403
    NotFoundError,     # 404
    ConflictError,     # 409 — e.g. cancel of an already-terminal order
    RejectedError,     # 422 — engine rejected the order
    ServerError,       # 5xx — retryable
)
```

Every exception has `.status_code` and `.message` attributes.

## Development

```bash
cd python
pip install -e ".[dev]"      # editable install
python examples/trading.py   # smoke test
```

To publish:

```bash
pip install hatch
hatch build
hatch publish
```

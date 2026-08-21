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
- Parses the API's Problem Details (`code`, `title`, optional `detail`,
  `request_id`, `recovery`, and `field_errors`) and maps statuses to an
  exception hierarchy:
  `BadRequestError` (400), `AuthError` (401), `ForbiddenError` (403),
  `NotFoundError` (404), `MethodNotAllowedError` (405),
  `ConflictError` (409), `GoneError` (410), `PayloadTooLargeError` (413),
  `UnsupportedMediaTypeError` (415), `RejectedError` (422),
  `FailedDependencyError` (424), `TooEarlyError` (425),
  `UpgradeRequiredError` (426), `RateLimitedError` (429), and
  `ServerError` (5xx). All inherit from `AgaraError`.
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
[`/docs/sdks/python/reference#streaming`](https://app.sandbox.agara.xyz/docs/sdks/python/reference#streaming).

Stream failures carry `failure.code`, `failure.title`, optional
`failure.detail`, and `failure.recovery`, plus a required `action` of
`"none"`, `"resubscribe"`, or `"reconnect"`. Callback mode performs the
last two actions automatically. Unknown actions and recovery strategies
are exposed for diagnostics but never acted on. An `OrderRejected` event
uses the same typed `failure`; route market-maker behavior on
`event.failure.code`, not a text reason.

## What it doesn't do (compose on top)

- Automatic retries on general REST calls. Check `error.is_retryable` and
  honor `error.retry_after` before repeating a request. A 5xx status alone
  is not permission to retry: for example, `dependency_unavailable` is
  retryable, while `internal_error` and `feature_not_configured` are not.

## Getting a token

API tokens are issued from the web app's **Profile → API tokens**
page. See [the authentication guide](https://app.sandbox.agara.xyz/docs/authentication)
for full details on scopes and revocation.

The recommended trading-bot scope set:

```
portfolio:read
orders:read
orders:place
orders:cancel
orders:cancel_all
```

Add `account:stream` for the account-events WebSocket,
`orders:place_signed` for locally-signed orders, and
`positions:split` / `positions:merge` for on-chain split/merge.
Reading the orderbook is public — no scope needed.

Locally-signed orders: from 0.9.0 `agara_sdk.signing` signs the
nine-field maker-guard `Order` (no `signer`, no `signatureType`).
Earlier releases sign the retired ten-field order, which the
maker-guard router rejects with a 400 hash mismatch. Upgrading
requires no call-site changes.

## Reference

### `AgaraClient(token, base_url="https://app.sandbox.agara.xyz", timeout=10.0, session=None)`

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
client.get_order_trades(order_id)             # → dict: `trades` (unpaginated), `as_of`
client.cancel_order(order_id)                 # → dict
client.cancel_all_orders()                    # → dict
```

Terminal order records now carry `order["failure"]` instead of
`order["error"]`. Signed-batch rejected entries likewise carry
`result["failure"]` instead of free-form `code` / `message` fields:

```python
result = client.place_signed_orders(orders=quotes)["results"][0]
if result["outcome"] == "rejected":
    failure = result["failure"]
    if failure["code"] == "post_only_would_cross":
        # Reprice this quote; repeating it unchanged cannot succeed.
        ...
```

For a safe transition, this SDK reads legacy order and batch failures,
maps known legacy batch codes, and converts every free-form diagnostic to
an `internal_error` without retaining the diagnostic.

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
    AuthError,         # 401 — missing / invalid / revoked / expired token
    ForbiddenError,    # 403 — token valid but lacks the required scope
    NotFoundError,     # 404
    ConflictError,     # 409 — e.g. cancel of an already-terminal order
    RejectedError,     # 422 — engine rejected the order
    RateLimitedError,  # 429 — per-tier bucket exhausted; see .retry_after
    ServerError,       # 5xx — inspect .is_retryable
)
```

Every exception has `.status_code`, `.message`, `.problem`, `.code`,
`.title`, `.detail`, `.request_id`, `.recovery`, `.field_errors`,
`.retry_after`, and `.is_retryable`. `problem` and contract fields are
`None` only when reading a legacy non-Problem response. `Retry-After`
headers take precedence over the body delay. Unknown or malformed
recovery strategies have `recovery.known == False` and are inert.

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

# Agara Python SDK

Python 3.10+, version **0.11.0**. Synchronous and async clients cover all **53 current trader REST operations**, the two router WebSocket endpoints, and both catalogue price SSE feeds. Contracts are pinned to platform `a7e8c2dc1ab3b16f4133d0952347d5cc1bbd1b71` (2026-09-17); conditional PnL routes remain available in the SDK even when a deployment disables them.

```sh
pip install 'agara-sdk[async,signing,streaming]'
```

The base install uses `requests`. The `async`, `signing`, and `streaming` extras provide `httpx`, local EIP-712 signing, and WebSockets respectively. Importing `agara_sdk` needs only the base dependencies. Select your deployment explicitly for production; the default URL is the Agara sandbox.

## Discover and read

```python
from agara_sdk import AgaraClient

with AgaraClient() as public:
    markets = public.list_markets(source="agara", state="ACTIVE", limit=20)
    print(markets["markets"])

with AgaraClient(token="agt_...", base_url="https://app.sandbox.agara.xyz") as client:
    portfolio = client.list_positions(exchanges=["AGARA"])
    if portfolio["unavailable_exchanges"]:
        raise RuntimeError("Wait for a complete position response before reconciling")
    print(portfolio["positions"], portfolio["markets"], portfolio["events"])
```

Anonymous clients can access public catalogue, status, book and LP discovery routes. Private operations fail locally without a token. Use the scopes required by each endpoint; REST scopes are recorded in [the coverage manifest](contracts/trader-manifest.json). HTTP sessions and async clients supplied by the caller are never closed or given persistent authorization headers by the SDK. A synchronous client is intended for one thread; async clients support concurrent tasks. Context managers close owned transports.

Responses are complete wire dictionaries annotated with `TypedDict` models in `agara_sdk.models` and `agara_sdk.catalogue`. Unknown fields are retained. Micro-unit fields remain decimal integer **strings**; native engine units, integer JSON fields, and whole-unit numeric book/catalogue fields retain their distinct wire representations. Use `micro_to_decimal()` for exact conversion. Realized PnL has its own `amountScale` (currently 12), not the six-decimal trading scale.

## Exact orders

```python
from decimal import Decimal
from agara_sdk import AgaraClient, to_micro

with AgaraClient("agt_...") as client:
    accepted = client.place_order(
        token_id="123", side="BUY", price=Decimal("0.55"), shares="2", post_only=True
    )
    terminal = client.wait_for_terminal(accepted["order_id"], timeout=30)
    assert terminal["is_terminal"]
    fills = client.get_order_trades(accepted["order_id"])

assert to_micro("9007199254.740993") == 9007199254740993
```

Whole-unit money inputs accept `Decimal`, decimal strings, or integers. Floats, non-finite decimals, sub-micro fractions, and values outside signed 64-bit micros are rejected without rounding. `submit_order()` accepts explicit typed wire requests. LIMIT orders always require shares. MARKET BUY requires `collateral_amount`; MARKET SELL requires `shares`. Both support FAK and FOK. GTD requires an expiration; other time-in-force values reject it. Server validation still determines live market limits, wallet availability and whether the expiration is sufficiently far in the future.

`wait_for_terminal()` uses only the server's boolean `is_terminal`: MATCHED may still be nonterminal, while a partially filled FAK can be terminal. Missing or malformed terminality fails closed. A timeout raises `TimeoutError`; the helper never cancels the order. Order completion is separate from trade settlement, which appears on trade status and transaction hashes.

Local `sign_limit_order()` uses the current nine-field Agara CTF Exchange Order domain. It validates the salt, chain integers and addresses and binds token, side, price and shares to the resulting `SignedOrder`. Passing different economics to `place_signed_order()` fails before I/O. `SignedOrderEntry` and `place_signed_orders()` submit 1–32 entries. Results discriminate accepted and rejected entries; rejections contain nested `failure` objects. Shared wallet/provider/storage failures may reject the whole request.

## Account batches and split/merge

`split_position()` and `merge_position()` return an AGARA batch acceptance (`batch_hash`, `status`, `as_of`) or the POLYMARKET operation receipt. Neither HTTP 201 nor HTTP 202 alone establishes chain confirmation. `follow_position_operation()` follows an AGARA acceptance with `wait_for_batch()` and retains a POLYMARKET receipt unchanged.

```python
from agara_sdk.batch_signing import BatchContracts, BatchDomain, RoutedOperation, sign_batch

# Obtain verified deployment addresses, the wallet's account seq, and market routing first.
signed = sign_batch(
    private_key=private_key,
    domain=BatchDomain(account_address, chain_id, implementation_version=1),
    contracts=BatchContracts(ctf_address, neg_risk_adapter_address, collateral_address),
    operations=[
        RoutedOperation(
            {
                "kind": "MERGE",
                "market_id": market_id,
                "condition_id": condition_id,
                "shares_micro": 1_000_000,
            },
            route="Ctf",
        )
    ],
    seq=account_seq,
    deadline_unix_seconds=deadline,
)
accepted = client.submit_batch(signed.to_request_body())
result = client.wait_for_batch(accepted["batch_hash"])
```

Presigned PAT batches admit **SPLIT, MERGE and WITHDRAW**, up to 20 operations. Neg-risk supports MERGE, but not SPLIT. REDEEM and Across operations are excluded because they are not admitted by this presigned trader path. Calldata is derived from typed operations and bound to the account's chain and implementation version. Deployment addresses and CTF/neg-risk routing are explicit; the SDK does not guess them. The existing published single- and multi-call digest vectors are tested unchanged.

`submit_batch()`, `get_batch()`, `supersede_batch()`, `get_batch_group()`, and their wait helpers require `batches:submit`. For superseding, re-sign at the same seq and pass `SignedBatch.to_supersede_body()`; inspect the `SUPERSEDED`/`REFUSED` outcome. `wait_for_batch()` follows successor hashes by default, rejects cycles and waits for `unwound_at` on FAILED batches. It returns terminal failures as well as SETTLED results; inspect `status` and `failure`. A group completes only when `completed_at` is set; inspect its individual chunks.

## Async parity

```python
from agara_sdk.aio import AsyncAgaraClient

async with AsyncAgaraClient("agt_...") as client:
    positions = await client.list_positions()
    trades = await client.list_trades(limit=100)
```

All REST methods and wait helpers have the same names, arguments and result types on both clients. Async cancellation propagates and closes active SSE contexts. Polling caps HTTP timeouts to the remaining deadline and rejects late responses; async HTTP also has an overall deadline. Synchronous `requests` connect/read timeouts are inactivity bounds, so a slowly trickling server can delay the exception beyond the nominal overall deadline. No HTTP mutation is automatically retried. Transport errors flag `mutation_outcome_unknown`: reconcile the precomputed order or batch hash before resubmitting.

## Streaming and recovery

```python
from agara_sdk import streaming

stream = streaming.AgaraStreamClient(token="agt_...", max_queue_size=1024)
await stream.subscribe(
    [
        streaming.orderbook("123"),
        streaming.best_quote("123"),
        streaming.trades(condition_id),
        streaming.market_status(condition_id),
        streaming.event_market_status(event_id),
        streaming.account_events(),
    ]
)


@stream.on_current_market_changed
async def changed(frame):
    print(frame.event_id, frame.data["market"])


@stream.on_sequence_reset
async def reset(frame):
    print(frame.channel, frame.requires_rest_reseed)


await stream.run()
```

All six subscription forms, 21 active update kinds, and the router's supported book delta shape have named types. Event-scoped updates intentionally have **no sequence**. Acknowledgements, resets and errors retain `event_id`. Account updates retain batch provenance, original order size, redemption and collateral movements. Unknown/malformed frames remain `UnknownFrame` and cannot trigger automatic recovery. Callback mode reconnects and replays subscriptions; `Reconnect(max_attempts=0)` disables reconnecting. Iterator mode (`async with stream`, `async for frame in stream`) is a single connection per endpoint; reconnect explicitly after it ends. Call `stop()` or cancel `run()` to cleanly stop all readers, keepalives and backoff tasks.

The application queue is bounded (1024 by default), with WebSocket receive limits and backpressure. Recovery does not prove that your local state is current:

| Feed | After connection loss or sequence reset |
| --- | --- |
| orderbook | Discard the local book and buffer updates. After the first post-reset snapshot arrives, keep buffering and fetch the REST book. Parse its `hash` as the baseline sequence, discard buffered frames with `sequence <= hash`, and apply newer frames in arrival order: snapshots replace the baseline and deltas modify it. Another reset restarts this procedure; use the same snapshot-then-REST order after resubscribing. |
| best_quote | Clear the quote; accept the next replacement quote. |
| account_events | Buffer live events and fully seed orders, open orders, positions, summary, trades and activities through REST, walking every page. Confirm order transitions against REST, deduplicate fills by `fill_id` and sequence-zero rejections by `order_id`, and refetch positions/summary after fills or inventory changes. Restart an incomplete trades walk without a cursor; retry incomplete positions. |
| condition lifecycle | Buffer lifecycle updates and fetch the current market through REST. Apply the entire ordered buffer and poll if REST disagrees; stale REST must never override a later stop event. |
| trades | Resume from the next trade. Public trades missed during the gap are not replayed. |
| event lifecycle | Fetch current event/market state; subsequent notifications do not provide a replacement snapshot. |

A post-reset book snapshot establishes that updates are flowing again; its arrival alone is not a state fence. The subsequent REST `hash` supplies that fence. For a fill, wait for `fill_id` in the order’s trades and for the order row to reflect all listed fills before rebuilding inventory. Require `unavailable_exchanges` to be empty on every page when reconciling multiple exchanges.

Engine sequence values need not be consecutive for a particular subscription. Do not treat every numerical gap as lost messages. HTTP order/trade projections can lag live events. The SDK exposes the data and reset signals needed for application-specific reconciliation; it does not silently declare a reconstructed account complete.

Both price SSE feeds are available on both HTTP clients:

```python
from contextlib import closing

with AgaraClient() as client:
    with closing(client.stream_prices(["Crypto.BTC/USD", "Crypto.ETH/USD"])) as prices:
        for event in prices:
            for price in event.prices:
                print(price.symbol, price.value, price.publish_time_ms)
```

`stream_pyth_price(symbol)` uses the single-symbol endpoint. Symbols are provider symbols, while REST `get_price_point()` uses the canonical security symbol plus provider. SSE retains integer-string mantissas, exponents and millisecond publication times; `Price.value` is exact Decimal. Parsing bounds data before line completion and supports split UTF-8/CRLF. Keepalives are ignored. Close/aclose generators on early exit. EOF ends the feed; reconnect by opening a new iterator and treat the latest quote as a replacement. Neither producer promises replay or an event cursor.

## Coverage

See [contracts/trader-manifest.json](contracts/trader-manifest.json) for every named method and source handler. Public methods include market/event/category discovery, search, calendars and sessions, securities, price points/ticks/token history; orders, signed orders, cancellation, complete portfolio envelopes and seven activity variants; deposit addresses/assets/quotes and withdrawal quotes; current/closed LP markets, categories, earnings, rebates; settled PnL, PnL history and realized PnL. Feature-disabled endpoints return the server's typed failure. JWT-only administration/funding execution, internal/admin/faucet routes and retired redemption endpoints are intentionally outside this trader SDK.

Pagination is explicit: reuse `pagination.next_cursor` until it is `None`, even when a page's list is empty. Open-order and order methods return one page; no metadata or availability information is discarded.

## Migration from 0.9 / 0.10

- Replace floats with decimal strings or `Decimal`; remove LIMIT collateral budgets and supply shares.
- Read `list_positions()["positions"]`, `list_open_orders()["orders"]`, and `get_portfolio_summary()["summaries"]`. These methods now return envelopes. Inspect availability and page cursors.
- `get_orderbook()` returns the typed wire envelope (`bids`, `asks`, `hash`, `tick_size`, `timestamp`), rather than a float convenience object.
- Replace `micro_to_float()` with `micro_to_decimal()`. `TERMINAL_STATUSES` is removed; use `is_terminal` and handle timeout exceptions.
- Follow AGARA split/merge batch acceptances; a position-operation response is not an unconditional confirmed receipt.
- Use nested `failure` for signed-batch rejections and stream errors. `AgaraError` exposes code, detail, request ID, field errors, recovery and Retry-After. Unknown recovery is inert; never infer safe retry solely from HTTP status.

## Development

```sh
python -m venv .venv
.venv/bin/pip install -e '.[dev]'
.venv/bin/ruff check .
.venv/bin/ruff format --check .
.venv/bin/mypy agara_sdk
.venv/bin/pytest -q
.venv/bin/python -m build
```

`tools/generate_async.py` derives async REST method bodies from the synchronous client; run it and Ruff after changing public methods. Regression tests use both HTTP mock transports, independent source-derived response fixtures, the platform's canonical failure fixtures, unchanged signing goldens, and loopback WebSocket servers. Examples are imported and type-checked without executing trades. No live trading is required for these checks.

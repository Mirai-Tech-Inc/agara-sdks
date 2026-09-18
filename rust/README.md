# agara-sdk

Async Rust SDK for Agara API traders. Rust 2024, MSRV 1.97.0. The current contract is pinned to platform commit `a7e8c2dc1ab3b16f4133d0952347d5cc1bbd1b71`; its public trader contracts were verified unchanged against `origin/main` at `93baa346f9ee04a4ff4b5a05bbd3da718887908b` on 2026-09-18.

The SDK covers 53 public/PAT REST operations, both router WebSocket endpoints, all six subscription forms and 22 supported update shapes, and both price SSE endpoints. [`coverage.json`](coverage.json) maps REST routes to public methods and producer-shaped HTTP fixtures. Browser wallet administration, browser signer configuration, JWT-only withdrawal and merge-all submission, token administration, faucets, internal/admin APIs, and retired redeem routes are outside this trader contract.

```toml
[dependencies]
agara-sdk = "0.4"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

`signing` and `streaming` are enabled by default. Disable default features for a REST-only client; signing and streaming can also be enabled independently.

## Public reads and authentication

```rust,no_run
use agara_sdk::{AgaraClient, client::Anonymous, catalogue::MarketsQuery};

# async fn example() -> agara_sdk::Result<()> {
let public = AgaraClient::<Anonymous>::anonymous()
    .base_url("https://app.sandbox.agara.xyz".into())
    .call()?;
let markets = public.list_markets(&MarketsQuery {
    source: "agara".to_owned(),
    state: Some("ACTIVE".to_owned()),
    ..Default::default()
}).await?;

let trader = AgaraClient::builder()
    .token(std::env::var("AGARA_TOKEN").expect("set AGARA_TOKEN").into())
    .build()?;
let snapshot = trader.get_positions(vec![], vec![]).await?;
if !snapshot.unavailable_exchanges.is_empty() {
    // Defer inventory decisions until every requested exchange is available.
    return Ok(());
}
# Ok(())
# }
```

`AgaraClient<Anonymous>` exposes public reads. Authenticated operations require `AgaraClient<Authenticated>` and the appropriate existing PAT scopes. Optional `.http(reqwest_client)` injects a pooled transport, including custom certificates or headers. SDK-owned HTTP clients reject redirects to prevent implicit mutation replays. An injected transport must also use `reqwest::redirect::Policy::none()`; its redirect and default-header configuration remains the caller’s responsibility. REST requests retain a per-attempt timeout; SSE uses bounded connection and idle reads through the same transport.

## Orders and exact units

Use `TokenId::new`, `OrderId::new` and other validated identifier constructors. Token IDs must fit unsigned 256-bit decimal integers, order/wallet/event/market/group IDs must be UUIDs, and condition/order/batch hashes must contain exactly 32 bytes. Constructors and deserialization enforce the same invariants and normalize the representation. Every URL path argument is encoded as a single segment. `Micro::from_units(Decimal)` is fallible: more than six decimal places or an overflowing amount is rejected. No amount is silently rounded, saturated or converted through floating point.

```rust,no_run
use agara_sdk::{AgaraClient, ids::{Side, TokenId}};

# async fn example(trader: &AgaraClient) -> agara_sdk::Result<()> {
let accepted = trader.place_limit_order()
    .token_id(TokenId::new("123")?)
    .side(Side::Buy)
    .price("0.50".parse().unwrap())
    .shares("2".parse().unwrap())
    .call().await?;
let order = trader.wait_for_terminal(
    &accepted.order_id,
    core::time::Duration::from_secs(30),
    core::time::Duration::from_millis(250),
).await?;
assert!(order.is_terminal);
# Ok(())
# }
```

LIMIT orders take shares. MARKET BUY takes collateral; MARKET SELL takes shares. FAK and FOK are supported in both directions. Market-specific ticks, notional bounds and current balances remain server-authoritative. A placement acknowledgment confirms acceptance, not a fill. Polling trusts `Order.is_terminal`, including partially filled terminal FAK orders, and enforces a wall-clock deadline.

Position split/merge returns `PositionOperationResponse`: AGARA returns `PendingBatch`; Polymarket returns a relayer receipt. Inspect the batch with `get_batch` or `wait_for_batch`. Failed batches with an unfinished unwind remain pending for reconciliation.

## Signed orders and account batches

`signing::sign_limit_order()` uses the nine-field `Agara CTF Exchange` EIP-712 order. It accepts explicit salt, timestamp, metadata and builder fields. `EngineDomain::new` validates the chain and exchange; deployment contract addresses must be supplied by the caller. `SignedOrder::to_request_body()` is a builder that rejects any mismatch between the signed token/side/price/shares and the request envelope.

`batch_signing::compose::compose` supports the presigned trader gauntlet's SPLIT, MERGE and WITHDRAW operations. Neg-risk MERGE uses the configured adapter; neg-risk SPLIT is rejected. `sign_batch()` composes calls, signs the `AgaraAccount` domain bound to the account and implementation version, and returns both the digest and typed submission. Submit once with `submit_batch`; use `get_batch`, `supersede_batch`, `get_batch_group` and `wait_for_batch` to reconcile. Supersede returns either the successor or a typed refusal containing the current batch. An ambiguous submission must be reconciled using its existing digest before any new submission.

Canonical published single-call and multi-call calldata/digest vectors are checked unchanged in `tests/batch_goldens.rs`.

## Read coverage

Typed responses retain available shares, market lifecycle state, display sidecars, events, condition metadata, timestamps and exchange availability. `get_positions` returns the full snapshot. `list_positions` is a convenience that fails on incomplete availability, including default-all queries. `get_open_orders_page` retains metadata and pagination; `list_open_orders` walks bounded pages, failing on cursor cycles. Order, fill and activity pages expose opaque cursors.

LP methods cover filters, thresholds, active/upcoming phases, categories, closed reward cycles and earnings. PnL methods cover settled headline/history and exact realized reports. PnL is deployment-dependent: an unmounted route may return 404, and an incomplete projection may return typed 425 recovery. History preserves `has_more`; do not treat a truncated history as complete.

Catalogue methods cover market/event lookup and listing, canonical event links, categories, search, trading calendars, securities, pinned prices, price ticks and outcome token history. A missing calendar day is unavailable data, not a closed trading day.

## Errors and retries

`AgaraError::problem()` exposes full Problem Details, code, request ID, typed field errors and `Recovery`, including typed order/batch/group status identities. Unknown failure codes or strategies never activate automatic recovery. HTTP retrying is opt-in and restricted to reads, including read-only POST list routes. Writes are never automatically replayed after transport, 429 or 5xx failures. When both headers and the canonical body specify delays, the longest delay applies. A server delay above the 60-second automatic retry ceiling is returned without an early retry. Unparseable, unrepresentable or HTTP-date retry hints are retained as `RetryAfter::Unusable` and disable automatic retry; the caller can reconcile and schedule a later read.

Local failures have distinct `Validation`, `OrderValidation`, `BatchValidation`, `Signing`, `Compose`, `Response`, `Poll` and `Pagination` variants. Match the nested enum and field instead of parsing diagnostic text. HTTP failures retain the actual status, original bounded body and validated problem; malformed problem metadata is a protocol error. The response body bound is 16 MiB. Known problem codes are enums; future codes are preserved without enabling recovery.

Request DTOs are mutable data carriers. Every HTTP submission revalidates their ranges and cross-field constraints, even after earlier validation or deserialization. Signature envelopes validate amount/side/token consistency, scalar bounds, signature shape and deadlines; cryptographic verification additionally requires the correct signing domain and holder. Market-specific constraints and current balances remain authoritative on the server. Timestamps on order responses use `values::Timestamp`; required exchange-availability fields cannot silently default to an empty list.

`RetryPolicy` and `Reconnect` have private fields and fallible builders. Jitter must be finite within `[0, 1]`, delays positive and ordered, and retry counts bounded. For example:

```rust
use agara_sdk::RetryPolicy;

let retry = RetryPolicy::builder()
    .max_retries(3)
    .jitter(0.2)
    .build()?;
# Ok::<(), agara_sdk::validation::ValidationError>(())
```

## Streaming and reconciliation

`AgaraStreamClient` supports orderbook, best quote, condition market status, event market status, trades and authenticated account events. `connect` returns a bounded `StreamHandle`; `next` and `run` share the same reconnect policy. Dropping or closing the handle aborts its tasks, including when the reader queue is full or reconnect is sleeping. Policy close code 1008 and identity-refresh failures stop automatic reconnects. Unknown updates remain `Frame::Unknown` with their complete envelope. Malformed known payloads are `Frame::Malformed` with a typed decoding error and original value. Local transport or queue failures are `Frame::ClientError`, separate from server `Frame::Error`; handle all three explicitly. No malformed payload fabricates zero balances or prices. Stream construction is fallible: `AgaraStreamClient::builder().build()?`.

Each new connection emits a local `Frame::ConnectionOpened` listing the subscribed subjects. This is a reconciliation signal, not a server sequence reset. After connection changes or server resets:

- Replace local books with the next complete book snapshot before trading from them.
- Re-read market/event lifecycle state through REST; lifecycle notifications have no snapshot replay.
- Buffer account events while fetching complete portfolio/order/fill snapshots. Preserve availability markers, correlate order hashes and fill IDs, and allow for REST persistence lag. Account events do not replay a complete account snapshot.

Native WebSocket price/size integers use the frame's scales. REST orderbook levels are whole-unit decimal numbers. Batch-related account events retain `batch_hash` and `batch_index`. Event-level lifecycle notifications have an event ID and no engine sequence.

`stream_price(symbol)` and `stream_prices(symbols)` expose typed SSE price events. Mantissa strings, exponent and provider publication milliseconds are preserved. The incremental decoder handles comments, multiline data, UTF-8 fragments and CRLF. EOF, malformed events, admission problems and idle timeout are surfaced; explicitly reconnect and compare publish timestamps. No historical replay is promised. Drop the stream to release its response.

## Verification

```sh
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo test --no-default-features
RUSTDOCFLAGS='-D warnings' cargo doc --all-features --no-deps
cargo package --locked
```

The suite uses loopback HTTP/WebSocket and source-derived fixtures. It makes no live trades or wallet mutations. Version 0.4 intentionally makes decimal conversion, identifier construction and configuration builders fallible. It removes LIMIT collateral-budget input, validates signed economics at submission, replaces generic error strings with typed variants, and removes status-only completion inference. Use the server’s `Order.is_terminal` flag and validated batch completion evidence.

Public clients cannot call authenticated operations:

```compile_fail,E0599
use agara_sdk::{AgaraClient, client::Anonymous};

async fn public_cannot_cancel() -> agara_sdk::Result<()> {
    let public = AgaraClient::<Anonymous>::anonymous().call()?;
    public.cancel_all_orders().await?;
    Ok(())
}
```

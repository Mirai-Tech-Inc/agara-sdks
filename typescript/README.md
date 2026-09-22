# @agara/sdk

TypeScript client for Agara public discovery, PAT trading, account batches, portfolio,
LP rewards, P&L, router WebSockets, and price SSE. Alpha; not yet published by this change.
Targets platform commit `8cf944265642737148bdb2ae128c697b776cebec`.

Requires Node.js 22+ or a modern browser with fetch, AbortController, WebSocket,
Web Streams, and bigint. Core HTTP needs only `lossless-json`. Install the optional
`viem` peer to use local signing:

```sh
npm install @agara/sdk
npm install viem # when importing @agara/sdk/signing
```

```ts
import { PublicClient, AgaraClient, parseUnits } from '@agara/sdk';

const publicClient = new PublicClient();
const markets = await publicClient.listMarkets({ source: 'agara', limit: 20 });
const client = new AgaraClient({ token: process.env.AGARA_TOKEN! });
const accepted = await client.placeOrder({
  token_id: 'YOUR_TOKEN_ID', type: 'LIMIT', side: 'BUY', time_in_force: 'GTC',
  price_micro: parseUnits('0.60').toString(),
  shares_micro: parseUnits('2').toString(),
});
const completed = await client.waitForOrder(accepted.order_id);
console.log(completed.status, completed.is_terminal);
```

`PublicClient` exposes anonymous endpoints; `TraderClient` adds every PAT endpoint;
`AgaraClient` adds polling and cursor helpers. Public client construction requires no
credentials. Configure `baseUrl`, `catalogueBaseUrl`, an injected `fetch`, default
`timeoutMs`, and `maxResponseBytes`. Each request accepts an optional final
`{ signal, timeoutMs, onResponse }`. Responses preserve complete envelopes and metadata;
response observers receive HTTP status, headers and request ID.

## Coverage and wire contracts

All **53 REST operations**, both router WebSocket endpoints, and both price SSE feeds
in [the checked-in scope](contracts/manifest.json) have named APIs. The
[method inventory](contracts/endpoints.json) maps each REST method to its path and auth.
Concrete request/response types derive from scoped OpenAPI snapshots, with source-backed
corrections documented in [contracts/README.md](contracts/README.md). Import specific
schema types through `TradingSchemas` / `CatalogueSchemas` from `@agara/sdk/types`;
common domain types such as `Order`, `Fill`, `BatchSubmission` are also exported.

Included families: orderbooks/orders/signed batches; account batch submission/status/
supersede/group status; complete portfolio reads, split/merge, rebates, P&L; bridge
asset/address/quote reads; LP incentives; events, markets, categories, search,
calendars, securities, price point/ticks/token history. Feature-gated P&L and disabled
providers return their structured server failure; method presence does not enable them.

JWT onboarding/account administration, token CRUD, standalone withdrawals, merge-all
creation, browser configuration, home/navigation presentation, faucets, admin/internal
routes, and retired redemption endpoints are intentionally excluded. PAT account-batch
`WITHDRAW` is supported and distinct from the excluded JWT withdrawal HTTP endpoint.
Across batch operations require server signing and are not accepted by presigned helpers.

## Exact amounts and completion

Use integer micro strings or bigint; avoid floating-point accounting. `parseUnits('1.23')`
returns `1230000n`; `formatUnits(1230000n)` returns `'1.23'`. Excess decimal places throw
rather than round. Native JSON integers beyond Number's safe range decode as bigint;
64-bit numeric response fields are declared `number | bigint`. Micro amount strings,
token IDs, hashes, provider prices, and P&L decimal strings remain strings. Unsafe
JavaScript integer inputs are rejected. Use bigint for batch integer fields; transport
serializes them as exact JSON numbers, not floating-point numbers or quoted strings.

Order acceptance is asynchronous. `waitForOrder` uses `is_terminal`, including final
PARTIALLY_FILLED FAK orders and interim MATCHED states that are not terminal. It does
not wait for trade settlement: inspect `getOrderTrades` / `listTrades` separately.
When the polling loop expires, `WaitTimeoutError` carries `.latest`. An in-flight read that
times out can instead throw `TransportError`; inspect its cause. A timeout does not establish
that the order failed or that a submitted mutation was cancelled.

AGARA split/merge returns 201 `{ batch_hash, status: 'PENDING', as_of }`; POLYMARKET
returns its 202 operation receipt. `waitForPositionOperation` handles that union.
`waitForBatch` returns on SETTLED, FAILED_DIVERGENT, or FAILED after `unwound_at` is
present. Inspect the returned state/failure: completion does not imply success.
`waitForBatchGroup` uses `completed_at` and preserves per-chunk attempts and failures.

## Signing

```ts
import { signOrder, signBatch } from '@agara/sdk/signing';
import { privateKeyToAccount } from 'viem/accounts';
```

Inject a signer supporting `signTypedData`; the SDK does not store keys. `signOrder`
binds maker, token, side, price and shares in one input and returns the complete POST
body plus `order_hash`. Nine-field Agara CTF Exchange EIP-712 domain and low-s signer
behavior are covered by cross-platform goldens. Raw signed submissions also reject
inconsistent request/envelope economics before I/O. `hashOrder` requires an explicit
salt; `signOrder` generates a cryptographically random nonzero salt when omitted.

Account batches use `composeBatchCalls` and `signBatch`: supply explicit account,
implementation version, chain ID, contract context and each market's Ctf/NegRisk route.
Helpers canonically encode SPLIT/MERGE/WITHDRAW calls and sign AgaraAccount's batch.
Neg-risk splits, self withdrawals, zero salts and unsupported operations fail locally.
The two published platform batch calldata/digest vectors are checked without reblessing.
`submitBatch`, `getBatch`, `supersedeBatch`, `getBatchGroup` are typed wrappers. Supply
live sequence and deployment addresses from trusted configuration; the SDK never guesses.

## Errors and retries

`AgaraError` retains `.status`, `.body`, `.headers`, `.problem`, `.code`, `.requestId`,
`.recovery` and `.retryAfter`. Canonical recovery is validated against the checked-in
registry; unknown codes/strategies remain inspectable and cannot enable automatic action.
Transport failures have `.outcomeUnknown` for mutations. No HTTP call is automatically
retried. Polling helpers only repeat safe reads when registered recovery permits it.
Reconcile ambiguous signed-order submission by its known hash; reconcile account batches
by `batchHash`. Never blindly replay an ambiguous mutation.

Request validation checks known request shapes, precision, LIMIT/MARKET sizing, GTD,
post-only, signatures and batch bounds. Response validation rejects malformed known
shapes. It is not a replacement for server checks of current balance, market grids,
account permissions, deployment policy or the complete domain rules.

## Pagination and partial availability

`orderPages`, `openOrderPages`, `tradePages`, and `activityPages` are async page iterators.
They preserve full envelopes and filters, continue through empty pages with cursors,
and reject cursor cycles. `paginate(fetchPage)` adapts other cursor-based methods.
`listPositions` is unpaginated and returns its entire envelope, including
`unavailable_exchanges`. `assertComplete(page)` optionally rejects partial snapshots.
Do not interpret unavailable backends as empty holdings. Iterators accept cancellation.

## Streaming

```ts
import { marketStream, accountStream, priceStream, pythProStream } from '@agara/sdk/streaming';
const stream = marketStream({ channels: [{ name: 'market_status', event_id: 'EVENT_UUID' }] });
for await (const event of stream) console.log(event);
```

Six subscription forms: orderbook/token, best_quote/token, trades/condition,
market_status/condition, market_status/event, and account_events/token. The decoder
covers 21 current emitted updates plus orderbook delta, nested failures, all controls,
and optional batch provenance. Event lifecycle frames have no engine sequence.
Unknown updates are `{ op: 'unknown', raw }`; malformed known frames throw ProtocolError.
Native 64-bit sequence/size values remain exact (`number | bigint`); no sequence
contiguity is invented for filtered feeds.

The async iterator emits connection/frame/gap events. A bounded queue fails loudly on
backpressure. Reconnection uses bounded exponential jitter and replays subscriptions;
authentication failure stops instead of looping a stale token. AbortSignal, `.close()`,
and breaking the iterator release sockets/timers. At most 64 subscriptions are allowed.
The router sends ten-second heartbeat frames; an idle watchdog detects stalled streams.

A gap requires application reconciliation: orderbook state needs its REST fence and a
fresh snapshot; best quotes await a fresh snapshot; account/trades/condition/event
lifecycle channels require current REST state. Streams do not replay missed fills or
turn a reconnected transport into a reconciled portfolio. See the platform's channel
recovery contract before resuming trading after a gap.

Price SSE supports multi-symbol and single Pyth Pro feeds with actual
`{parsed:[{id,price:{price,expo,publish_time_ms}}]}` frames. It incrementally handles
UTF-8/chunk boundaries, CRLF, comments, multiline data, bounded events, cancellation,
and bounded reconnection. It does not promise historical replay. Excessive server
retry delays surface instead of retrying early; customize `maxRetryDelayMs` if needed.

## Development

```sh
npm ci
npm run generate
npm run check
npm run test:pack
```

`npm run docs:check` validates public TSDoc as part of `npm run check` and CI. The check follows
all four package exports, including public members and handwritten request/option properties.
Use a meaningful `/** ... */` summary, `@param name - description` for public callable parameters,
and relevant `@returns`, `@throws`, `@remarks`, `@defaultValue` or `@example` sections. Explain units,
defaults and asynchronous completion when callers need them; private implementation helpers do
not need API doc comments. Generated OpenAPI schema declarations retain their upstream JSDoc
and are exempt from the curated public-documentation gate.

REST method documentation lives in `scripts/endpoint-docs.mjs`. Update that metadata and run
`npm run generate`; editing `src/endpoints.ts` alone will be overwritten. The generator rejects
missing method/parameter documentation. The package smoke check verifies that public comments
survive declaration emission and packing, so installed consumers receive IntelliSense guidance.

Generation is offline from committed snapshots. Tests use mock transports, canonical
problem fixtures, current wire payloads and signing goldens; no live trades are sent.

Because generation is offline, a platform field rename is invisible here until a consumer hits
it. `npm run drift:check` compares the committed contracts against the OpenAPI a deployment
serves and fails on a difference that breaks a correct exchange:

```sh
AGARA_DRIFT_BASE_URL=https://app.sandbox.agara.xyz npm run drift:check
```

It needs network, so it is not part of `npm run check`; CI runs it on a schedule and on demand.
Deliberate deviations live in [contracts/drift-allowlist.json](contracts/drift-allowlist.json).

See [design references](docs/design.md), [migration notes](docs/migration.md), and
[examples](examples/). Publishing is a separate explicit action.

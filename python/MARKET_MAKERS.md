# Market-maker guide: pre-signed orders on Agara

This guide is for liquidity providers / bots that want to **sign orders locally with
their own key** and submit them to the Agara CLOB, bypassing the browser/Privy signing
path. It documents the exact EIP-712 payload, the request body for
`POST /trade/v1/orders/signed`, the server-side validation rules, and a worked example.

> The pre-signed path has been exercised end-to-end against the forked CTF exchange: all
> three CTF match types settle on-chain with both sides locally signed, plus batch,
> post_only/GTD, and the validation-rejection cases (summarized in section 10).

---

## 1. Two ways to place orders

| Path | Endpoint | Who signs | Use it when |
|---|---|---|---|
| **Unsigned** | `POST /trade/v1/orders` | the server, via the holder's Privy key | browser users; bots that don't hold a key |
| **Pre-signed** | `POST /trade/v1/orders/signed` (+ `/batch`) | **you**, locally, with the holder EOA key | market makers running a bot; no Privy round-trip per order |

A market maker almost always wants the **pre-signed** path: lower latency (no server-side
Privy call), full control of the order envelope, and batch submission (up to 32 orders/call).

Pre-signed supports **`LIMIT` only**. For `MARKET` orders use the unsigned path (the server
walks the book and signs); see `place_market_order` in the SDK.

---

## 2. Prerequisites (one-time)

1. **An onboarded AgaraAccount.** You need a deposit wallet (an `AgaraAccount` smart account)
   deployed on-chain with approvals set, plus a wallet row + engine registration in the
   backend. This happens through the normal onboarding flow (`/trade/v1/wallet/register`
   → `/trade/v1/wallet/setup`). After it completes, `GET /trade/v1/wallet/status` returns:
   - `deposit_wallet_address` — your **maker** address (the `AgaraAccount`).
   - `eoa_address` — your **holder** EOA; the signature must recover to this.
2. **The holder EOA private key.** For Privy-managed embedded wallets, export it from
   Privy's export modal (the runner's "Export private key" button calls
   `useExportWallet`). This key signs every order. Treat it as live funds.
3. **A personal access token (PAT)** with the `orders:place_signed` scope (below).
4. **Funds**: collateral (USDC) in the AgaraAccount on-chain so settlement can pull it, and
   an engine balance so the matching engine accepts your orders. (For sell/merge orders you
   also need the relevant outcome positions.)

---

## 3. Auth — issue a PAT

PAT creation itself requires a Privy identity JWT (it can't be done with a PAT):

```bash
curl -X POST $ROUTER/trade/v1/auth/tokens \
  -H "authorization: Bearer <PRIVY_JWT>" -H 'content-type: application/json' \
  -d '{
    "label": "maker-bot",
    "exchange": "AGARA",
    "scopes": ["orders:place_signed","orders:cancel","orders:cancel_all","orders:read","portfolio:read"],
    "expires_at": null
  }'
```

The response `token` (`agt_…`) is shown once. Pass it to the SDK client. Relevant scopes:

| Scope | Grants |
|---|---|
| `orders:place_signed` | `POST /orders/signed` and `/orders/signed/batch` |
| `orders:place` | `POST /orders` (unsigned / market) |
| `orders:cancel` / `orders:cancel_all` | cancel one / all |
| `orders:read` / `portfolio:read` | order + portfolio reads |

---

## 4. The signed order — EIP-712

The thing you sign is the CTF exchange `Order` struct. It is **not** the request body; it's
the typed-data message whose digest your key signs. The request body (section 5) carries the
order fields **plus** the resulting `order_hash` and `signature` so the server can recompute
the digest and recover the signer.

### Domain

```
EIP712Domain {
  name:              "Agara CTF Exchange"     // fixed
  version:           "1"                       // fixed
  chainId:           84532                     // the chain the exchange lives on
  verifyingContract: <CTFExchange address>     // the forked exchange contract
}
```

`chainId` and `verifyingContract` **must** match the server's configured exchange, or the
server's recomputed hash won't match yours and the order is rejected (`order_hash does not
match the recomputed EIP-712 digest`). Get both from your operator: the chain id the
exchange is deployed on (e.g. Base Sepolia is `84532`) and the deployed CTFExchange address.

### Type

The fork uses a **10-field** `ORDER_TYPEHASH` (note: no `signatureType` field — that was
dropped from the on-chain struct):

```
Order(
  uint256 salt,
  address maker,
  address signer,
  uint256 tokenId,
  uint256 makerAmount,
  uint256 takerAmount,
  uint8   side,
  uint256 timestamp,
  bytes32 metadata,
  bytes32 builder
)
```

### Field semantics

| Field | Meaning |
|---|---|
| `salt` | uniqueness nonce (u256). The contract dedups on `order_hash`; salt's only job is to make each order's hash unique. Use a random u128. |
| `maker` | **your AgaraAccount** (`deposit_wallet_address`). Holds the funds/positions. |
| `signer` | **must equal `maker`** for ERC-1271 orders. |
| `tokenId` | the CTF position id (u256) — the outcome you're trading. |
| `makerAmount` | what you give. **BUY:** μUSDC collateral. **SELL:** μshares. |
| `takerAmount` | what you want. **BUY:** μshares. **SELL:** μUSDC collateral. |
| `side` | `0` = BUY, `1` = SELL. |
| `timestamp` | usually `0`. |
| `metadata` | `bytes32`, usually zero. |
| `builder` | `bytes32`, usually zero. |

Amount math (μ = ×10⁻⁶; price is dollars/share in (0,1)):

```
collateral_micro = shares_micro * price_micro / 1_000_000   # integer division; must be > 0
BUY : makerAmount = collateral_micro,  takerAmount = shares_micro
SELL: makerAmount = shares_micro,      takerAmount = collateral_micro
```

The signature is a flat 65-byte `r‖s‖v` ECDSA over the typed-data digest. On-chain,
`AgaraAccount.isValidSignature` validates it as `ecrecover(hash, sig) == holder`. The server
mirrors that check before accepting the order. **Must be canonical low-s** (EIP-2) — the
account's `ECDSA.tryRecover` rejects high-s, and the server rejects it up front too.

---

## 5. Request body — `POST /trade/v1/orders/signed`

An example body (BUY 100 YES @ $0.50). The `…`-marked fields are derived per order — fill
`maker`/`signer` with your AgaraAccount, and `order_hash`/`signature` from `sign_limit_order`:

```json
{
  "token_id": "<outcome token id>",
  "side": "BUY",
  "type": "LIMIT",
  "time_in_force": "GTC",
  "price_micro": "500000",
  "shares_micro": "100000000",
  "post_only": false,
  "order_hash": "0x… (keccak256 EIP-712 digest you signed)",
  "signature": "0x… (your 65-byte r‖s‖v signature)",
  "salt": "42",
  "maker": "0x<your AgaraAccount>",
  "signer": "0x<your AgaraAccount>",
  "chain_token_id": "<same u256 as token_id>",
  "maker_amount": "50000000",
  "taker_amount": "100000000",
  "side_u8": 0,
  "signature_type": 3,
  "timestamp": "0",
  "metadata": "0x0000000000000000000000000000000000000000000000000000000000000000",
  "builder": "0x0000000000000000000000000000000000000000000000000000000000000000"
}
```

(`maker_amount = shares × price = 100 × $0.50 = 50_000000` μUSDC; `taker_amount =
100_000000` μshares — see the amount math in section 4.)

| Field | Type | Notes |
|---|---|---|
| `token_id` | string | the outcome id the router looks up (`agara_market_outcomes.token_id`). |
| `side` | `"BUY"`/`"SELL"` | engine side. |
| `type` | `"LIMIT"` | only LIMIT is accepted here. |
| `time_in_force` | `"GTC"`/`"FOK"`/`"GTD"` | `GTD` requires `expiration_unix_seconds`. |
| `price_micro` | string | μUSDC/share, in `(0, 1_000_000)`. Must satisfy the market grid (section 7). |
| `shares_micro` | string | μshares. Must satisfy the size grid. |
| `post_only` | bool | reject (don't rest) if it would cross. |
| `expiration_unix_seconds` | int | only for `GTD`. |
| `order_hash` | 0x bytes32 | keccak256 EIP-712 digest you signed. |
| `signature` | 0x 65-byte | `r‖s‖v`, canonical low-s. |
| `salt` | string (u256) | same salt you signed. |
| `maker` | address | = your deposit wallet. |
| `signer` | address | = `maker`. |
| `chain_token_id` | string (u256 dec or 0x-hex) | the `tokenId` you signed; must numerically equal `token_id`. |
| `maker_amount` / `taker_amount` | string (u256) | as signed (see §4 math). |
| `side_u8` | int | `0` BUY / `1` SELL. |
| `signature_type` | int | **must be `3`** (ERC-1271). |
| `timestamp` / `metadata` / `builder` | as signed | usually `0` / zero / zero. |

Response: `202 Accepted` with the order ack (`order_id`, `status: PENDING`, …). The ack means
"received", not "filled" — poll `GET /trade/v1/orders/{order_id}` (SDK `get_order` /
`wait_for_terminal`) to track fills.

### Batch — `POST /trade/v1/orders/signed/batch`

Body is `{"orders": [ <body above>, … ]}`, **1..=32** orders. Each is validated and accepted
**independently**: the response `results[]` carries one entry per order in request order,
each either `{"outcome":"accepted", "order_id":…}` or `{"outcome":"rejected","code":…,
"message":…}`. A duplicate `order_hash` is reported rejected rather than failing the batch.

---

## 6. SDK usage (Python)

```python
from agara_sdk import AgaraClient
from agara_sdk import signing

client = AgaraClient(token="agt_…", base_url="https://<router>")
domain = signing.EngineDomain(chain_id=84532, exchange_contract="0x<CTFExchange>")

KEY  = "0x<holder_eoa_key>"          # signs; must recover to your holder EOA
ACCT = "0x<your_AgaraAccount>"        # maker == signer == deposit wallet
token_id = 12345...  # the outcome's u256 token id, from the market's outcomes

# single order
so = signing.sign_limit_order(
    private_key=KEY, domain=domain, deposit_wallet_address=ACCT,
    token_id=token_id, side="BUY",
    price_micro=500_000, shares_micro=100_000_000,   # salt defaults to random u128
)
ack = client.place_signed_order(
    token_id=str(token_id), side="BUY",
    price_micro=500_000, shares_micro=100_000_000, signed_order=so,
    time_in_force="GTC", post_only=False,
)

# batch (bid ladder)
from agara_sdk.signing import SignedOrderEntry
entries = []
for px in (200_000, 150_000, 100_000):
    s = signing.sign_limit_order(private_key=KEY, domain=domain,
        deposit_wallet_address=ACCT, token_id=token_id, side="BUY",
        price_micro=px, shares_micro=10_000_000)
    entries.append(SignedOrderEntry(signed_order=s, token_id=str(token_id),
        side="BUY", price_micro=px, shares_micro=10_000_000))
res = client.place_signed_orders(orders=entries)
```

`sign_limit_order` needs the `[signing]` extra (`pip install 'agara-sdk[signing]'`, pulls in
`eth-account`). It enforces `0 < price_micro < 1_000_000`, `shares_micro > 0`, and
non-zero collateral; everything else is enforced server-side.

---

## 7. Market grid constraints

Each market has an engine grid that your `price_micro`/`shares_micro` must satisfy, or the
**engine** rejects the order (asynchronously — the order lands `REJECTED`). The SDK only
checks `price ∈ (0,1)`; the grid is the market's. Fetch it from the market config; a typical
binary market looks like:

```
tick_size = 1, price_scale = 100, size_scale = 100, min_price = 1, max_price = 99
```

- engine price = `price_micro * price_scale / 1_000_000` → must be an **integer** in
  `[min_price, max_price]` and a multiple of `tick_size`. With `price_scale=100` that means
  `price_micro` is a multiple of `10_000` (one cent) within `[10_000, 990_000]`.
- engine size = `shares_micro * size_scale / 1_000_000` → must be an integer. With
  `size_scale=100` that means `shares_micro` is a multiple of `10_000`.
- `price_scale * size_scale` must divide `1_000_000` (collateral scale) — a market-config
  invariant, not something you set.

---

## 8. Validation rules & exact errors (all return `400`)

The server runs these before the order reaches the engine (verified by the test suite):

| Check | Error message on failure |
|---|---|
| `type == LIMIT` | `pre-signed endpoint accepts LIMIT orders only` |
| `chain_token_id` numerically equals `token_id` | `chain_token_id does not match token_id` |
| `signature_type == 3` | `agara only accepts ERC-1271 (signatureType = 3) on this endpoint` |
| `maker == signer` | `ERC-1271 orders require maker == signer` |
| `maker == your deposit-wallet address` | `maker must equal the wallet's deposit-wallet address` |
| `order_hash` equals server-recomputed EIP-712 digest | `order_hash does not match the recomputed EIP-712 digest` |
| signature is 65 bytes, canonical | `signature must be 65 bytes` / `signature s is not canonical (high-s)` |
| `ecrecover(hash, sig) == holder EOA` | `signature does not recover to the wallet's EOA` |
| market is AGARA & accepting orders | `pre-signed endpoint is agara-only` / not-accepting error |

A wrong `chainId`/`verifyingContract` surfaces as the **hash-mismatch** error — that's the
first thing to check if every order is rejected.

---

## 9. Match types & on-chain settlement

The engine matches your resting/crossing orders; the chain-settler then lands a single
`matchOrders` call on the CTF exchange. Three settlement shapes, all driven identically from
your side (you just sign BUY/SELL on YES/NO):

| Shape | How it arises | On-chain effect |
|---|---|---|
| **NORMAL** | BUY YES × SELL YES (same token) | outcome token transfers maker→taker; USDC the other way |
| **MINT** | BUY YES × BUY NO (complementary) | exchange splits collateral, mints a YES+NO pair |
| **MERGE** | SELL YES × SELL NO (complementary) | exchange merges a YES+NO pair back to collateral |

(A SELL YES is economically a BUY NO, so "complementary" crosses are detected automatically;
`cross_match_enabled` must be on for the market.)

---

## 10. Verified end-to-end

These payloads were exercised end-to-end with two AgaraAccounts both signing locally — **all
cases pass**:

| Test | Result |
|---|---|
| unsigned `place_order` (Privy path) | accepted, rests, cancel |
| pre-signed rest + cancel | accepted, rests @ signed price |
| pre-signed **NORMAL** match (both sides signed) | settled on-chain |
| pre-signed **MERGE** match (both sides signed) | settled on-chain |
| pre-signed **MINT** match (both sides signed) | settled on-chain |
| pre-signed **batch** (3-level ladder) | all accepted |
| pre-signed `post_only` (rests) + `GTD` (future expiry) | accepted |
| reject: tampered signature | `400 signature does not recover to the wallet's EOA` |
| reject: `maker` ≠ deposit wallet | `400 maker must equal the wallet's deposit-wallet address` |
| reject: wrong domain (exchange addr) | `400 order_hash does not match the recomputed EIP-712 digest` |

Each match minted/transferred/merged the full order size between the two accounts and moved
the corresponding USDC, confirmed by reading ConditionalTokens balances after each fill.

---

## 11. Gotchas checklist

- **Domain must match the operator's exchange** (`chainId`, `verifyingContract`) or every
  order hash-mismatches.
- **`maker == signer == your deposit wallet`**, and the **signature must recover to the
  holder EOA** — not the deposit wallet.
- **Canonical low-s** signatures only. (`eth-account` produces these by default.)
- **Grid alignment**: `price_micro` and `shares_micro` must fit the market's tick/scale or
  the engine rejects asynchronously.
- **`chain_token_id` must equal `token_id`** numerically (one is the lookup string, one is
  the u256 you signed).
- **Fresh salt per order**; the contract dedups on `order_hash`.
- The `202` ack is "received", not "filled" — poll order status for fills.
- Fund the **AgaraAccount on-chain** (USDC for buys, positions for sells/merges) and the
  **engine balance**, or orders are rejected for insufficient balance.

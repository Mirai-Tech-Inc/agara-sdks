# agara-sdk (Rust)

Latency-focused async Rust client for the agara prediction-markets
trading API. Fully-typed responses, newtype ids, EIP-712 order signing,
and a typed WebSocket stream — the same surface as the Python SDK.

## Install

```toml
[dependencies]
agara-sdk = "0.1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

Default features enable `signing` (EIP-712, pulls `alloy`) and
`streaming` (WebSocket, pulls `tokio-tungstenite`). Turn them off for a
REST-only build:

```toml
agara-sdk = { version = "0.1", default-features = false }
```

## Quickstart

```rust
use agara_sdk::{AgaraClient, ids::{Side, TokenId}};
use rust_decimal_macros::dec;

#[tokio::main]
async fn main() -> agara_sdk::Result<()> {
    let client = AgaraClient::builder()
        .token(std::env::var("AGARA_TOKEN").unwrap())
        .build()?;

    let token = TokenId::new("21742…36455");
    let book = client.get_orderbook(&token).await?;
    println!("best bid {:?} / ask {:?}", book.best_bid(), book.best_ask());

    let ack = client
        .place_limit_order()
        .token_id(token)
        .side(Side::Buy)
        .price(dec!(0.60))
        .shares(dec!(1))
        .call()
        .await?;

    let final_order = client
        .wait_for_terminal(&ack.order_id, std::time::Duration::from_secs(30), std::time::Duration::from_secs(1))
        .await?;
    println!("final status: {:?}", final_order.status);
    Ok(())
}
```

## What this SDK does

- **One bearer token.** Construct with a personal access token (`agt_…`);
  every request carries it.
- **Typed everything.** Responses are structs, not JSON maps; ids are
  distinct newtypes (`TokenId`, `ConditionId`, `OrderId`, `OrderHash`) so
  you can't cross them.
- **Micro-units hidden.** Prices/sizes are `rust_decimal::Decimal` at the
  boundary; the wire's integer micro-encoding is handled for you
  (`Micro`).
- **Typed errors keyed by status.** `AgaraError::{BadRequest, Auth,
  Forbidden, NotFound, Conflict, Rejected, RateLimited, Server, …}`;
  `RateLimited` carries `retry_after`.
- **Opt-in retries.** Pass a `RetryPolicy` (exponential backoff + jitter,
  honors `Retry-After`) to the builder; off by default.
- **`wait_for_terminal`** for the place-and-poll pattern.

## Signing (feature `signing`)

Bots that hold their own EOA key sign orders locally and skip the Privy
round-trip. The digest matches `crates/chain-client`'s golden byte-for-byte,
so a pre-signed order validates on-chain.

```rust
use agara_sdk::{EngineDomain, sign_limit_order, ids::{Side, TokenId, TimeInForce}, Micro};
use alloy::primitives::{Address, U256};

let signed = sign_limit_order()
    .private_key(&key)
    .domain(EngineDomain { chain_id: 84532, exchange_contract: exchange })
    .deposit_wallet_address(wallet)
    .token_id(U256::from_str_radix("21742…", 10).unwrap())
    .side(Side::Buy)
    .price_micro(Micro::new(600_000))
    .shares_micro(Micro::new(1_000_000))
    .call()?;

let body = signed.to_request_body(
    TokenId::new("21742…"), Side::Buy, Micro::new(600_000), Micro::new(1_000_000),
    TimeInForce::Gtc, false, None,
);
client.place_signed_order(&body).await?;
```

Batch up to 32 with `client.place_signed_orders(vec![...])`.

## Streaming (feature `streaming`)

```rust
use agara_sdk::{AgaraStreamClient, Channel, Frame, ids::{ConditionId, TokenId}};

let mut client = AgaraStreamClient::builder()
    .token(std::env::var("AGARA_TOKEN").unwrap())
    .build();
client.subscribe([
    Channel::Orderbook(TokenId::new("21742…")),
    Channel::Trades(ConditionId::new("0x2174…")),
    Channel::AccountEvents,
])?;

let mut stream = client.connect().await?;
while let Some(frame) = stream.next().await {
    match frame {
        Frame::Trade(t) => println!("trade {:?} {}@{}", t.side, t.size, t.price),
        Frame::Fill(f) => println!("my fill {}", f.fill_id),
        Frame::SequenceReset(r) => eprintln!("reset on {} — rebuild local state", r.channel),
        _ => {}
    }
}
```

Or push-style: `stream.run(|frame| async move { … }).await`. Both
auto-reconnect with backoff, replay subscriptions on reconnect, and act
on server `error` frames. One client transparently manages the public
market stream and the private account stream.

## Getting a token

Issued from the web app's **Profile → API tokens** page. Recommended
trading-bot scopes: `portfolio:read orders:read orders:place
orders:cancel orders:cancel_all`; add `account:stream` for the account
stream, `orders:place_signed` for signed orders, and `positions:split` /
`positions:merge` for on-chain split/merge. Reading the orderbook is
public.

## Development

```bash
cargo test --all-features
cargo clippy --all-targets --all-features -- -D warnings
```

# SDKs

Language-specific client libraries for the agara trading API. Each
subdirectory is independently publishable to its language's package
registry.

| Language | Path | Status | Registry |
| --- | --- | --- | --- |
| Python | [`python/`](./python/) | alpha | `agara-sdk` on PyPI (when published) |
| Rust | [`rust/`](./rust/) | alpha | `agara-sdk` on crates.io (when published) |
| TypeScript | — | planned | `@agara/sdk` on npm |

All SDKs target the same HTTP API. The canonical reference for what
each endpoint does — request shape, response shape, errors — lives at
[`https://app.sandbox.agara.xyz/docs`](https://app.sandbox.agara.xyz/docs).
When in doubt, the API docs win; SDKs are a thin convenience layer
over them.

The auto-generated OpenAPI 3 spec at `https://app.sandbox.agara.xyz/trade/v1/openapi.json`
drives any future generated clients we publish alongside the
hand-written ones.

## Conventions across all SDKs

- **Naming.** Method names mirror the HTTP verb intent
  (`place_order`, `get_orderbook`, `cancel_order`, `list_trades`), not
  the raw URL path.
- **Amounts.** Take dollars / shares in arguments, convert to micro
  units internally; parse response micro strings back to floats for
  consumers. Users think in dollars; the wire format stays hidden.
- **Errors.** A hierarchy keyed by HTTP status with typed Problem Details
  (`code`, `title`, optional `detail`, request id, and recovery). Retry
  behavior follows recovery guidance; SDKs never assume every 5xx is
  retryable, and future recovery/action values are inert.
- **Auth.** Personal access tokens (`agt_…`) only; no Privy JWTs
  through the SDKs (those are browser-side).

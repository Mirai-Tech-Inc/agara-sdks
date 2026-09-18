# SDKs

Language-specific client libraries for the agara trading API. Each
subdirectory is independently publishable to its language's package
registry.

| Language | Path | Status | Registry |
| --- | --- | --- | --- |
| Python | [`python/`](./python/) | alpha | `agara-sdk` on PyPI (when published) |
| Rust | [`rust/`](./rust/) | alpha | `agara-sdk` on crates.io (when published) |
| TypeScript | [`typescript/`](./typescript/) | alpha | `@agara/sdk` on npm (when published) |

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
- **Amounts.** Prefer exact decimal strings or integer micro units for trading
  and accounting. Floating-point conversion is an explicit display convenience,
  not an exact amount representation. Each language documents its available APIs
  and migration guidance: [Python](python/README.md), [Rust](rust/README.md),
  [TypeScript](typescript/README.md).
- **Errors.** Preserve HTTP status and structured problem information; use
  documented recovery guidance rather than assuming every server failure is
  retryable. See each language's exception and compatibility documentation.
- **Auth.** Public reads and personal access tokens (`agt_…`); no Privy JWTs
  through the SDKs (those are browser-side).

# Contract sources

Platform `e136dacf2e5442d1a8e2a05d7c77bf70b8f3aa6f` (2026-09-22).
`trading.json` and `catalogue.json` are scoped snapshots of the platform's checked-in
`apps/web/content/api-specs` documents, limited to the 40 operations in `manifest.json`.
Unused component schemas are removed. Preserve arbitrary metadata within documented JSON fields.

The environment-projected trading document omits mounted paths, including both bridge
withdrawal paths, which respond on every deployment checked. These source-backed additions
preserve the actual router contract:

- AGARA acceptance status is `PENDING`: `apps/router/src/handlers.rs:763`.
- Both bridge withdrawal paths: `apps/router/src/handlers/portfolio.rs` and
  `crates/service/src/portfolio.rs`, which resolve the AGARA wallet.

Problem fixtures and code registry come unchanged from `specs/problem-contracts`.
Run `npm run generate` after deliberately updating contract snapshots. Generated types
preserve integer wire values as `number | bigint` for 64-bit numeric fields; unsafe
JavaScript numbers are rejected and JSON parsing preserves large integers as bigint.

Nothing in this repository notices when the platform changes a field, so
`npm run drift:check` compares these snapshots against the OpenAPI a deployment serves
(`AGARA_DRIFT_BASE_URL`, plus `AGARA_DRIFT_GATE_COOKIE` for a gated environment). It fails
on a difference that breaks a correct exchange: a required response field the server no
longer sends, a request field the server now requires, or a response enum carrying a value
the snapshot rejects. Everything else prints as a warning. The deliberate deviations listed
below are recorded in `drift-allowlist.json`; keep that list short, because it is what stops
the report from being read.

The trading `Exchange` enum carries AGARA only: this package is Agara-only, so the other
exchange's identifier is not shipped even though the platform domain still defines it.
Runtime schema references are namespaced by service so catalogue and trading names
cannot overwrite each other. The runtime checker covers the schema forms used here;
it is not offered as a general-purpose JSON Schema validator.

`problem-details.json` preserves the canonical public details in
`crates/problem-contracts/src/generated_metadata.rs`; known titles/details/recovery
are validated against that registry. `batch-goldens.json` copies the unchanged values
from `crates/account-batch/tests/golden_vectors.rs`. `ws-fixtures.json` covers router
producers plus canonical failure fixtures; event lifecycle examples include every
field required by `packages/schemas/src/market-lifecycle.ts`.

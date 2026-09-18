# Contract sources

Platform `a7e8c2dc1ab3b16f4133d0952347d5cc1bbd1b71` (2026-09-17).
`trading.json` and `catalogue.json` are scoped snapshots of the platform's checked-in
`apps/web/content/api-specs` documents, limited to the 53 operations in `manifest.json`.
Unused component schemas are removed. Preserve arbitrary metadata within documented JSON fields.

The environment-projected trading document omits mounted multi-exchange paths. These
source-backed additions preserve the actual router contract:

- Three deposit paths and their DTOs: `apps/router/src/handlers/portfolio.rs:511-586`
  and `crates/service/src/portfolio.rs:835-923`.
- Split/merge's POLYMARKET 202 receipt: `handlers/portfolio.rs:39-138` and
  `crates/service/src/position_operations.rs:154-169`.
- AGARA acceptance status is `PENDING`: `apps/router/src/handlers.rs:763`.

Problem fixtures and code registry come unchanged from `specs/problem-contracts`.
Run `npm run generate` after deliberately updating contract snapshots. Generated types
preserve integer wire values as `number | bigint` for 64-bit numeric fields; unsafe
JavaScript numbers are rejected and JSON parsing preserves large integers as bigint.

The trading `Exchange` enum is restored to AGARA/POLYMARKET from the actual domain;
environment documentation projects it to AGARA even on shared multi-exchange DTOs.
Runtime schema references are namespaced by service so catalogue and trading names
cannot overwrite each other. The runtime checker covers the schema forms used here;
it is not offered as a general-purpose JSON Schema validator.

`problem-details.json` preserves the canonical public details in
`crates/problem-contracts/src/generated_metadata.rs`; known titles/details/recovery
are validated against that registry. `batch-goldens.json` copies the unchanged values
from `crates/account-batch/tests/golden_vectors.rs`. `ws-fixtures.json` covers router
producers plus canonical failure fixtures; event lifecycle examples include every
field required by `packages/schemas/src/market-lifecycle.ts`.

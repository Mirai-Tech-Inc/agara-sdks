/**
 * Exact-amount clients, trading workflows and typed HTTP failures for Agara API traders.
 *
 * @remarks
 * Use `PublicClient` for anonymous reads, `TraderClient` for PAT operations, and `AgaraClient`
 * for polling and page iteration. Optional signing and streaming APIs have separate package exports.
 *
 * @packageDocumentation
 */
export * from "./amounts.js";
export { PublicClient, TraderClient } from "./endpoints.js";
export * from "./errors.js";
export type { ClientOptions, RequestOptions, ResponseMetadata } from "./transport.js";
export { ResponseObserverError } from "./transport.js";
export type * from "./types.js";
export type { WaitOptions } from "./workflows.js";
export { AgaraClient, assertComplete, paginate } from "./workflows.js";

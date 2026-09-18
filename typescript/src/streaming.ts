/**
 * Router WebSocket and provider-price SSE APIs, available from `@agara/sdk/streaming`.
 *
 * @remarks
 * Stream recovery notifications require callers to reconcile application state. Reopening a
 * connection does not replay missed account activity or guarantee historical price delivery.
 *
 * @packageDocumentation
 */
export type { PriceEvent, PriceStreamOptions, SseEvent } from "./sse.js";
export { parseSse, priceStream, pythProStream } from "./sse.js";
export type { StreamEvent, StreamOptions, WebSocketLike } from "./stream-client.js";
export {
  AgaraStream,
  accountStream,
  marketStream,
  StreamBackpressureError,
  StreamClosedError,
} from "./stream-client.js";
export { decodeFrame, decodePriceFrame, validateSubscription } from "./stream-decode.js";
export * from "./stream-types.js";

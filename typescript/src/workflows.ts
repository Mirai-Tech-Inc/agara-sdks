import { TraderClient } from "./endpoints.js";
import {
  AgaraError,
  PaginationError,
  PartialAvailabilityError,
  ProtocolError,
  WaitTimeoutError,
} from "./errors.js";
import type { RequestOptions } from "./transport.js";
import type { BatchStatus, Order, PositionOperationResult, TradingSchemas } from "./types.js";
/**
 * Polling settings; inherited `timeoutMs` is the overall positive deadline (default 30000 ms).
 * An inherited response observer runs for each successfully validated status read.
 */
export interface WaitOptions extends RequestOptions {
  /**
   * Nonnegative delay between status reads, in milliseconds; server retry advice may lengthen it.
   *
   * @defaultValue 1000
   */
  pollIntervalMs?: number;
  /**
   * Consecutive recoverable server errors at which polling fails; a successful read resets the count.
   * Transport failures and unregistered recovery are returned immediately.
   *
   * @defaultValue 3
   */
  maxTransientErrors?: number;
}
/**
 * Require all requested exchanges to be available before using a portfolio snapshot as complete.
 *
 * @typeParam T - A response envelope carrying exchange availability diagnostics.
 * @param value - Snapshot whose unavailable exchanges must be an empty list.
 * @returns The same response object, without cloning or changing its rows.
 * @throws `PartialAvailabilityError` listing each unavailable exchange.
 * @example
 * ```ts
 * const snapshot = assertComplete(await client.listPositions({}));
 * ```
 */
export function assertComplete<T extends { unavailable_exchanges: readonly string[] }>(
  value: T,
): T {
  if (value.unavailable_exchanges.length)
    throw new PartialAvailabilityError(value.unavailable_exchanges);
  return value;
}
export async function delay(ms: number, signal?: AbortSignal): Promise<void> {
  signal?.throwIfAborted();
  await new Promise<void>((resolve, reject) => {
    const done = () => {
      signal?.removeEventListener("abort", abort);
      resolve();
    };
    const timer = setTimeout(done, ms);
    const abort = () => {
      clearTimeout(timer);
      signal?.removeEventListener("abort", abort);
      reject(signal?.reason);
    };
    signal?.addEventListener("abort", abort, { once: true });
  });
}
/**
 * Yield complete cursor-page envelopes, preserving empty pages and their sidecar metadata.
 *
 * @remarks
 * The first call receives an undefined cursor. Iteration stops only when `next_cursor` is null
 * or absent. A malformed/repeated cursor rejects after that page has been yielded; pages already
 * consumed are not a complete collection. Pass the same signal to `fetchPage` to abort an in-flight
 * read; this iterator checks cancellation before requesting each page.
 *
 * @typeParam T - A page envelope exposing `pagination.next_cursor`.
 * @param fetchPage - Fetches the next page while preserving all filters other than its cursor.
 * @param options - Cancellation signal and maximum page count, defaulting to 10000 pages.
 * @returns An async iterator of pages, rather than individual records.
 * @throws `PaginationError` for an invalid/repeated cursor or an exhausted page budget.
 * @throws The fetcher's original error or the abort signal's reason.
 */
export async function* paginate<T extends { pagination: { next_cursor?: string | null } }>(
  fetchPage: (cursor: string | undefined) => Promise<T>,
  options: {
    /** Checked before each fetch; also pass it to the fetcher to cancel in-flight work. */
    signal?: AbortSignal;
    /** Maximum pages to yield before failing when a continuation still exists; default 10000. */
    maxPages?: number;
  } = {},
): AsyncGenerator<T> {
  let cursor: string | undefined;
  const seen = new Set<string>();
  const maxPages = options.maxPages ?? 10000;
  for (let page = 0; page < maxPages; page++) {
    options.signal?.throwIfAborted();
    const result = await fetchPage(cursor);
    yield result;
    const next = result.pagination.next_cursor;
    if (next == null) return;
    if (typeof next !== "string" || !next.length || seen.has(next))
      throw new PaginationError("Server returned an invalid or repeated cursor");
    seen.add(next);
    cursor = next;
  }
  throw new PaginationError("Pagination exceeded maxPages");
}

/**
 * Authenticated trader client with safe-read polling and cursor-page iteration helpers.
 *
 * @remarks
 * Requires the same personal access token and transport settings as `TraderClient`. Submission
 * methods are never retried. Polling repeats reads only, using registered recovery advice;
 * a terminal response can report a failed operation and must still be inspected.
 */
export class AgaraClient extends TraderClient {
  /**
   * Read an order until the server's authoritative `is_terminal` flag is true.
   *
   * @remarks
   * A terminal partially filled, cancelled or rejected order is a valid result. This does not wait
   * for trade settlement; inspect fills separately. The last observed order is attached to a
   * `WaitTimeoutError` when the polling loop exhausts its deadline. A timed-out in-flight HTTP read
   * can instead reject with `TransportError`; its cause identifies the transport failure.
   *
   * @param orderId - Internal order UUID returned by order placement.
   * @param options - Overall wait timeout, polling interval, cancellation and transient-error budget.
   * @returns The last observed terminal order, whose status and failure still require inspection.
   * @throws `WaitTimeoutError` when the polling loop runs out of time without observing completion.
   * @throws `AgaraError` for nonrecoverable server failures or exhausted transient-error allowance.
   * @throws `TransportError` for failed or aborted HTTP reads; cancellation between reads throws its reason.
   * @throws `RangeError` for an invalid polling deadline or interval.
   */
  async waitForOrder(orderId: string, options: WaitOptions = {}): Promise<Order> {
    return this.wait(
      async (request) => {
        const value = (await this.getOrder(orderId, request)).order;
        if (typeof value.is_terminal !== "boolean")
          throw new ProtocolError("Order is missing authoritative is_terminal", value);
        return value;
      },
      (value) => value.is_terminal,
      options,
    );
  }
  /**
   * Poll an account batch until settled, divergent failure, or failure with a completed unwind.
   *
   * @remarks
   * Completion is not proof of success: inspect `status` and `failure`. `FAILED` remains pending
   * while `unwound_at` is null; `FAILED_DIVERGENT` returns for caller intervention.
   *
   * @param hash - Existing EIP-712 batch digest as 0x-prefixed hex.
   * @param options - Overall wait timeout, polling interval, cancellation and transient-error budget.
   * @returns The last observed batch meeting the completion predicate, including failed batches.
   * @throws `WaitTimeoutError` with the latest snapshot when the polling loop deadline expires.
   * @throws `AgaraError` for nonrecoverable server failures or exhausted transient-error allowance.
   * @throws `TransportError` for failed or aborted HTTP reads; cancellation between reads throws its reason.
   */
  async waitForBatch(hash: string, options: WaitOptions = {}): Promise<BatchStatus> {
    return this.wait(
      (request) => this.getBatch(hash, request),
      (value) =>
        value.status === "SETTLED" ||
        value.status === "FAILED_DIVERGENT" ||
        (value.status === "FAILED" && value.unwound_at != null),
      options,
    );
  }
  /**
   * Reconcile the account batch that a split or merge accepted.
   *
   * @param result - The acceptance returned by `splitPosition` or `mergePosition`.
   * @param options - Batch wait settings.
   * @returns Batch status after polling; a terminal state does not by itself mean success.
   * @throws The same polling errors as `waitForBatch`.
   */
  async waitForPositionOperation(
    result: PositionOperationResult,
    options: WaitOptions = {},
  ): Promise<BatchStatus> {
    return this.waitForBatch(result.batch_hash, options);
  }
  /**
   * Iterate order-history page envelopes while retaining the original request filters.
   *
   * @param body - Initial filters and optional starting cursor; amounts and IDs retain wire units.
   * @param options - Per-read timeout, cancellation and successful-response observer.
   * @returns Async pages with order rows, metadata and cursors; already yielded pages may be partial.
   * @throws `PaginationError` for invalid/repeated cursors or the 10000-page bound; read errors propagate.
   */
  orderPages(body: TradingSchemas["ClobOrdersListRequest"] = {}, options: RequestOptions = {}) {
    return paginate(
      (cursor) => this.listOrders({ ...body, ...(cursor ? { cursor } : {}) }, options),
      options,
    );
  }
  /**
   * Iterate nonterminal-order pages with their market/event metadata and stable token/exchange filters.
   *
   * @param body - Initial open-order filters, page size and optional starting cursor.
   * @param options - Per-read timeout, cancellation and successful-response observer.
   * @returns Complete page envelopes rather than a flattened list or an atomic portfolio snapshot.
   * @throws `PaginationError` for invalid/repeated cursors or the 10000-page bound; read errors propagate.
   */
  openOrderPages(
    body: TradingSchemas["PortfolioOpenOrdersListRequest"] = {},
    options: RequestOptions = {},
  ) {
    return paginate(
      (cursor) => this.listOpenOrders({ ...body, ...(cursor ? { cursor } : {}) }, options),
      options,
    );
  }
  /**
   * Iterate fill-history pages while preserving each page's availability and market/event metadata.
   *
   * @param query - Initial page size and optional starting cursor.
   * @param options - Per-read timeout, cancellation and successful-response observer.
   * @returns Full fill pages; use `assertComplete` before treating unavailable venues as empty.
   * @throws `PaginationError` for invalid/repeated cursors or the 10000-page bound; read errors propagate.
   */
  tradePages(query: Parameters<TraderClient["listTrades"]>[0] = {}, options: RequestOptions = {}) {
    return paginate(
      (cursor) => this.listTrades({ ...query, ...(cursor ? { cursor } : {}) }, options),
      options,
    );
  }
  /**
   * Iterate account-activity pages, including orders and collateral/position activity variants.
   *
   * @param query - Initial page size and optional starting cursor.
   * @param options - Per-read timeout, cancellation and successful-response observer.
   * @returns Full activity envelopes with their market, condition and event sidecars.
   * @throws `PaginationError` for invalid/repeated cursors or the 10000-page bound; read errors propagate.
   */
  activityPages(
    query: Parameters<TraderClient["listActivities"]>[0] = {},
    options: RequestOptions = {},
  ) {
    return paginate(
      (cursor) => this.listActivities({ ...query, ...(cursor ? { cursor } : {}) }, options),
      options,
    );
  }
  private async wait<T>(
    read: (options: RequestOptions) => Promise<T>,
    terminal: (value: T) => boolean,
    options: WaitOptions,
  ): Promise<T> {
    const duration = options.timeoutMs ?? 30000,
      interval = options.pollIntervalMs ?? 1000,
      maxErrors = options.maxTransientErrors ?? 3;
    if (!Number.isFinite(duration) || duration <= 0 || !Number.isFinite(interval) || interval < 0)
      throw new RangeError("Invalid polling duration");
    const deadline = performance.now() + duration;
    let latest: T | undefined;
    let errors = 0;
    while (performance.now() < deadline) {
      options.signal?.throwIfAborted();
      let pause = interval;
      try {
        latest = await read({ ...options, timeoutMs: Math.max(1, deadline - performance.now()) });
        errors = 0;
        if (terminal(latest)) return latest;
      } catch (error) {
        if (!(error instanceof AgaraError) || !error.isRetryable || ++errors >= maxErrors)
          throw error;
        pause = Math.max(pause, (error.retryAfter ?? 0) * 1000);
      }
      const remaining = deadline - performance.now();
      if (remaining <= 0) break;
      await delay(Math.min(pause, remaining), options.signal);
    }
    throw new WaitTimeoutError(latest);
  }
}

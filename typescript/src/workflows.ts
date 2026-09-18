import { TraderClient } from "./endpoints.js";
import {
  AgaraError,
  PaginationError,
  PartialAvailabilityError,
  ProtocolError,
  WaitTimeoutError,
} from "./errors.js";
import type { RequestOptions } from "./transport.js";
import type {
  BatchGroupStatus,
  BatchStatus,
  Order,
  PositionOperationResult,
  TradingSchemas,
} from "./types.js";
export interface WaitOptions extends RequestOptions {
  pollIntervalMs?: number;
  maxTransientErrors?: number;
}
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
export async function* paginate<T extends { pagination: { next_cursor?: string | null } }>(
  fetchPage: (cursor: string | undefined) => Promise<T>,
  options: { signal?: AbortSignal; maxPages?: number } = {},
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

export class AgaraClient extends TraderClient {
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
  async waitForBatchGroup(id: string, options: WaitOptions = {}): Promise<BatchGroupStatus> {
    return this.wait(
      (request) => this.getBatchGroup(id, request),
      (value) => value.completed_at != null,
      options,
    );
  }
  async waitForPositionOperation(
    result: PositionOperationResult,
    options: WaitOptions = {},
  ): Promise<BatchStatus | TradingSchemas["PortfolioPositionOperationResponse"]> {
    return "batch_hash" in result ? this.waitForBatch(result.batch_hash, options) : result;
  }
  orderPages(body: TradingSchemas["ClobOrdersListRequest"] = {}, options: RequestOptions = {}) {
    return paginate(
      (cursor) => this.listOrders({ ...body, ...(cursor ? { cursor } : {}) }, options),
      options,
    );
  }
  openOrderPages(
    body: TradingSchemas["PortfolioOpenOrdersListRequest"] = {},
    options: RequestOptions = {},
  ) {
    return paginate(
      (cursor) => this.listOpenOrders({ ...body, ...(cursor ? { cursor } : {}) }, options),
      options,
    );
  }
  tradePages(query: Parameters<TraderClient["listTrades"]>[0] = {}, options: RequestOptions = {}) {
    return paginate(
      (cursor) => this.listTrades({ ...query, ...(cursor ? { cursor } : {}) }, options),
      options,
    );
  }
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

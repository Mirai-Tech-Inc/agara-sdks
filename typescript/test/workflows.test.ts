import { expect, it, vi } from "vitest";
import {
  AgaraError,
  PaginationError,
  PartialAvailabilityError,
  WaitTimeoutError,
} from "../src/errors.js";
import { problemRegistry } from "../src/generated/problems.js";
import type { BatchStatus, Order } from "../src/types.js";
import { AgaraClient, assertComplete, paginate } from "../src/workflows.js";
import { traderOptions } from "./helpers.js";

it("uses is_terminal rather than the displayed status in both FAK directions", async () => {
  const c = new AgaraClient(traderOptions);
  const order = vi
    .spyOn(c, "getOrder")
    .mockResolvedValueOnce({
      order: { status: "MATCHED", is_terminal: false } as Order,
      markets: {},
    })
    .mockResolvedValueOnce({
      order: { status: "PARTIALLY_FILLED", is_terminal: true } as Order,
      markets: {},
    });
  const result = await c.waitForOrder("id", { pollIntervalMs: 0 });
  expect(result.status).toBe("PARTIALLY_FILLED");
  expect(order).toHaveBeenCalledTimes(2);
});
it("does not finish a failed batch before unwind", async () => {
  const c = new AgaraClient(traderOptions);
  const read = vi
    .spyOn(c, "getBatch")
    .mockResolvedValueOnce({ status: "FAILED", unwound_at: null } as BatchStatus)
    .mockResolvedValueOnce({ status: "FAILED", unwound_at: "2026-09-17T00:00:00Z" } as BatchStatus);
  expect((await c.waitForBatch("hash", { pollIntervalMs: 0 })).unwound_at).not.toBeNull();
  expect(read).toHaveBeenCalledTimes(2);
});
it("keeps metadata and empty pages with continuation cursors", async () => {
  const pages = [
    { orders: [], pagination: { next_cursor: "c1" }, as_of: "a" },
    { orders: [1], pagination: { next_cursor: null }, as_of: "b" },
  ];
  const seen = [];
  for await (const page of paginate(
    async () => pages.shift() ?? { orders: [], pagination: { next_cursor: null }, as_of: "z" },
  ))
    seen.push(page);
  expect(seen).toHaveLength(2);
  expect(seen[0]?.as_of).toBe("a");
});
it("detects multi-step cursor cycles", async () => {
  let n = 0;
  const cursors = ["a", "b", "a"];
  await expect(
    (async () => {
      for await (const _page of paginate(async () => ({
        pagination: { next_cursor: cursors[n++] ?? null },
      }))) {
      }
    })(),
  ).rejects.toBeInstanceOf(PaginationError);
});
it("exposes incomplete portfolios without losing rows", () => {
  const page = { positions: [], unavailable_exchanges: ["AGARA"] };
  expect(() => assertComplete(page)).toThrow(PartialAvailabilityError);
  expect(page.unavailable_exchanges).toEqual(["AGARA"]);
});
it("times out with the latest order instead of returning a false terminal result", async () => {
  const c = new AgaraClient(traderOptions);
  vi.spyOn(c, "getOrder").mockResolvedValue({
    order: { status: "OPEN", is_terminal: false } as Order,
    markets: {},
  });
  await expect(c.waitForOrder("id", { timeoutMs: 5, pollIntervalMs: 1 })).rejects.toBeInstanceOf(
    WaitTimeoutError,
  );
});

// Built from the registry rather than written out: an off-by-one title or status makes the body fail
// problem validation, which would silently make the error non-retryable instead of failing loudly.
function failure(code: keyof typeof problemRegistry) {
  const { http_status: status, urn, title, recovery } = problemRegistry[code];
  // Websocket-only codes carry no HTTP status and cannot stand in for an HTTP failure.
  if (status === null) throw new TypeError(`${code} is not an HTTP problem`);

  return new AgaraError(status, { type: urn, title, status, code, recovery });
}
// `pnl_not_ready` carries recovery strategy "retry", which is what lets the polling loop read again.
const transient = () => failure("pnl_not_ready");
const open = { order: { status: "OPEN", is_terminal: false } as Order, markets: {} };
const done = { order: { status: "MATCHED", is_terminal: true } as Order, markets: {} };

it("tolerates fewer consecutive transient failures than maxTransientErrors", async () => {
  const c = new AgaraClient(traderOptions);
  // Without this the whole group passes vacuously: an unretryable error also reaches a terminal read.
  expect(transient().isRetryable).toBe(true);
  const read = vi
    .spyOn(c, "getOrder")
    .mockRejectedValueOnce(transient())
    .mockRejectedValueOnce(transient())
    .mockResolvedValueOnce(done);

  expect(await c.waitForOrder("id", { pollIntervalMs: 0, maxTransientErrors: 3 })).toEqual(
    done.order,
  );
  expect(read).toHaveBeenCalledTimes(3);
});

it("gives up on the maxTransientErrors-th consecutive transient failure", async () => {
  const c = new AgaraClient(traderOptions);
  const read = vi.spyOn(c, "getOrder").mockRejectedValue(transient());

  await expect(c.waitForOrder("id", { pollIntervalMs: 0, maxTransientErrors: 2 })).rejects.toThrow(
    AgaraError,
  );
  // Two failures is the limit, so the second read is the one that surfaces.
  expect(read).toHaveBeenCalledTimes(2);
});

it("resets the transient allowance after any successful read", async () => {
  const c = new AgaraClient(traderOptions);
  // Without a reset, these five failures would exhaust an allowance of 2 long before the last read.
  const read = vi
    .spyOn(c, "getOrder")
    .mockRejectedValueOnce(transient())
    .mockResolvedValueOnce(open)
    .mockRejectedValueOnce(transient())
    .mockResolvedValueOnce(open)
    .mockRejectedValueOnce(transient())
    .mockResolvedValueOnce(done);

  expect(await c.waitForOrder("id", { pollIntervalMs: 0, maxTransientErrors: 2 })).toEqual(
    done.order,
  );
  expect(read).toHaveBeenCalledTimes(6);
});

it("never retries a failure whose recovery does not permit it", async () => {
  const c = new AgaraClient(traderOptions);
  const fatal = failure("batch_not_found");
  expect(fatal.isRetryable).toBe(false);
  const read = vi.spyOn(c, "getBatch").mockRejectedValue(fatal);

  // A generous allowance must not turn a non-retryable failure into repeated reads.
  await expect(
    c.waitForBatch(`0x${"11".repeat(32)}`, { pollIntervalMs: 0, maxTransientErrors: 50 }),
  ).rejects.toBe(fatal);
  expect(read).toHaveBeenCalledTimes(1);
});

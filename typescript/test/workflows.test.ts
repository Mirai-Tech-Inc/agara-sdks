import { expect, it, vi } from "vitest";
import { PaginationError, PartialAvailabilityError, WaitTimeoutError } from "../src/errors.js";
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
  const page = { positions: [], unavailable_exchanges: ["POLYMARKET"] };
  expect(() => assertComplete(page)).toThrow(PartialAvailabilityError);
  expect(page.unavailable_exchanges).toEqual(["POLYMARKET"]);
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

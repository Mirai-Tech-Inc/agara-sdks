import { privateKeyToAccount } from "viem/accounts";
import { describe, expect, it, vi } from "vitest";
import { formatUnits, parseUnits } from "../src/amounts.js";
import { ProtocolError, TransportError } from "../src/errors.js";
import { AgaraClient, PublicClient } from "../src/index.js";
import { parseJson, stringifyJson } from "../src/json.js";
import { signOrder } from "../src/signing.js";
import { ResponseObserverError } from "../src/transport.js";
import { contract, type Schema, sample, traderOptions } from "./helpers.js";

interface Endpoint {
  name: string;
  service: string;
  method: string;
  path: string;
  operationId: string;
}
const endpoints = contract("endpoints") as unknown as Endpoint[];
const specs = { trading: contract("trading"), catalogue: contract("catalogue") };
const key = privateKeyToAccount(`0x${"11".repeat(32)}`);
const signed = await signOrder(
  {
    domain: { chainId: 84532, exchangeContract: "0x1b42FF8DdB251074637d3A9872D72f51e3AbB23d" },
    maker: "0x279640887C3806d4FBd424bb0B58F0430CE661C1",
    tokenId: "2",
    side: "BUY",
    priceMicro: 500000n,
    sharesMicro: 2000000n,
    salt: 1n,
  },
  key,
);
describe("complete named endpoint contract", () => {
  it("covers the scope exactly", () => {
    expect(endpoints).toHaveLength(48);
    const scope = contract("manifest").operations as Endpoint[];
    expect(new Set(endpoints.map((e) => `${e.method} ${e.path}`))).toEqual(
      new Set(
        scope
          .filter((e) => (e as unknown as { protocol: string }).protocol === "HTTP")
          .map((e) => `${e.method} ${e.path}`),
      ),
    );
  });
  for (const endpoint of endpoints)
    it(`${endpoint.name}: ${endpoint.method} ${endpoint.path}`, async () => {
      const spec = specs[endpoint.service === "router" ? "trading" : "catalogue"];
      const definitions = (spec.components as { schemas: Record<string, Schema> }).schemas;
      const operation = (spec.paths as Record<string, Record<string, Schema>>)[endpoint.path]?.[
        endpoint.method.toLowerCase()
      ] as Schema;
      const success = Object.entries(operation.responses as Record<string, Schema>).find(
        ([status]) => status.startsWith("2"),
      );
      if (!success) throw Error("No success");
      const responseSchema = (success[1].content as { "application/json": { schema: Schema } })[
        "application/json"
      ].schema;
      const responseBody = sample(responseSchema, definitions);
      let actualUrl: URL | undefined, actualInit: RequestInit | undefined;
      const fetcher: typeof fetch = async (url, init) => {
        actualUrl = new URL(String(url));
        actualInit = init;
        return new Response(stringifyJson(responseBody), {
          status: Number(success[0]),
          headers: { "content-type": "application/json", "x-request-id": "audit" },
        });
      };
      const client = new AgaraClient({ ...traderOptions, fetch: fetcher });
      const args: unknown[] = [];
      let path = endpoint.path;
      const parameters = (operation.parameters ?? []) as {
        name: string;
        in: string;
        required?: boolean;
        schema: Schema;
      }[];
      for (const parameter of parameters.filter((p) => p.in === "path")) {
        const value = parameter.name === "date" ? "2026-09-17" : "value with / delimiter";
        args.push(value);
        path = path.replace(`{${parameter.name}}`, encodeURIComponent(value));
      }
      let body: unknown;
      if (operation.requestBody) {
        body = sample(
          (operation.requestBody as { content: { "application/json": { schema: Schema } } })
            .content["application/json"].schema,
          definitions,
        );
        if (endpoint.name === "placeOrder")
          body = {
            token_id: "2",
            type: "LIMIT",
            side: "BUY",
            time_in_force: "GTC",
            price_micro: "500000",
            shares_micro: "2000000",
          };
        if (endpoint.name === "placeSignedOrder") body = signed;
        if (endpoint.name === "placeSignedOrders") body = { orders: [signed] };
        if (endpoint.name === "submitBatch" || endpoint.name === "supersedeBatch")
          body = {
            ops: [
              {
                kind: "SPLIT",
                market_id: "11111111-1111-4111-8111-111111111111",
                condition_id: `0x${"11".repeat(32)}`,
                shares_micro: 1000000n,
              },
            ],
            ...(endpoint.name === "submitBatch" ? { seq: 0n } : {}),
            deadline_unix_seconds: 1900000000n,
            signature: `0x${"11".repeat(65)}`,
          };
        args.push(body);
      }
      const query = Object.fromEntries(
        parameters
          .filter((p) => p.in === "query")
          .map((p) => [p.name, sample(p.schema, definitions)]),
      );
      if (parameters.some((p) => p.in === "query")) args.push(query);
      const method = (
        client as unknown as Record<string, (...args: unknown[]) => Promise<unknown>>
      )[endpoint.name];
      if (!method) throw Error("Missing wrapper");
      expect(await method.apply(client, args)).toEqual(responseBody);
      expect(actualUrl?.pathname).toBe(path);
      expect(actualInit?.method).toBe(endpoint.method);
      for (const [key, value] of Object.entries(query))
        expect(actualUrl?.searchParams.get(key)).toBe(
          Array.isArray(value) ? value.join(",") : String(value),
        );
      if (body !== undefined)
        expect(parseJson(String(actualInit?.body))).toEqual(parseJson(stringifyJson(body)));
    });
});
it("anonymous requests omit credentials and encode safe paths", async () => {
  const fetcher = vi.fn<typeof fetch>(async () => new Response('{"status":"ok"}'));
  const client = new PublicClient({ fetch: fetcher });
  expect(() => client.getMarket("..")).toThrow();
  expect(fetcher).not.toHaveBeenCalled();
});
it("rejects unsafe integer and amount rounding", () => {
  expect(() => stringifyJson({ n: Number.MAX_SAFE_INTEGER + 1 })).toThrow();
  expect(parseJson('{"n":9007199254740993}')).toEqual({ n: 9007199254740993n });
  expect(parseUnits("9007199254.740993")).toBe(9007199254740993n);
  expect(formatUnits(9007199254740993n)).toBe("9007199254.740993");
  expect(() => parseUnits("0.0000005")).toThrow();
});
it("does not retry ambiguous mutations", async () => {
  const fetcher = vi.fn<typeof fetch>().mockRejectedValue(Error("network gone"));
  const c = new AgaraClient({ ...traderOptions, fetch: fetcher });
  await expect(c.cancelAllOrders()).rejects.toMatchObject({ outcomeUnknown: true });
  expect(fetcher).toHaveBeenCalledTimes(1);
});
it("bounds body reads when injected fetch ignores abort", async () => {
  const c = new PublicClient({
    timeoutMs: 10,
    fetch: async () => new Response(new ReadableStream({ start() {} })),
  });
  await expect(c.getStatus()).rejects.toBeInstanceOf(TransportError);
});
it("rejects malformed successful orders instead of claiming completion", async () => {
  const c = new AgaraClient({
    ...traderOptions,
    fetch: async () => new Response('{"order":{"is_terminal":"false"},"markets":{}}'),
  });
  await expect(c.getOrder("uuid")).rejects.toBeInstanceOf(ProtocolError);
});
it("preserves response status and ids on canonical errors", async () => {
  const fixture = (
    contract("problem-fixtures").fixtures as { name: string; value: unknown }[]
  ).find((f) => f.name === "known origin problem")?.value;
  const c = new AgaraClient({
    ...traderOptions,
    fetch: async () => new Response(JSON.stringify(fixture), { status: 422 }),
  });
  await expect(c.getOrder("uuid")).rejects.toMatchObject({
    code: "insufficient_balance",
    requestId: "8051f907-4eeb-4ca9-bd84-b447ea65a68c",
  });
});
it("observer errors retain the successful response and do not suggest retry", async () => {
  const c = new AgaraClient({
    ...traderOptions,
    onResponse: () => {
      throw Error("observer");
    },
    fetch: async () =>
      new Response(
        '{"wallet_ids":[],"pending_operation":"CANCEL_ALL","as_of":"2026-09-17T00:00:00Z"}',
        { status: 202 },
      ),
  });
  await expect(c.cancelAllOrders()).rejects.toBeInstanceOf(ResponseObserverError);
});
it("validates bounds during construction", () => {
  expect(() => new PublicClient({ timeoutMs: NaN })).toThrow();
  expect(() => new PublicClient({ maxResponseBytes: -1 })).toThrow();
});
it("preserves mixed accepted/rejected signed-batch results", async () => {
  const as_of = "2026-09-17T00:00:00Z";
  const response = {
    results: [
      {
        index: 0,
        outcome: "accepted",
        order_id: "11111111-1111-4111-8111-111111111111",
        source: "AGARA",
        status: "PENDING",
        pending_operation: "SUBMIT",
        as_of,
      },
      {
        index: 1,
        outcome: "rejected",
        failure: {
          code: "duplicate_order",
          title: "Duplicate order",
          detail: "An order with this identity already exists.",
          recovery: { strategy: "none" },
        },
      },
    ],
    as_of,
  };
  const c = new AgaraClient({
    ...traderOptions,
    fetch: async () => new Response(JSON.stringify(response), { status: 202 }),
  });
  expect(await c.placeSignedOrders({ orders: [signed, signed] })).toEqual(response);
});
it("preserves POLYMARKET availability and a complete positions envelope", async () => {
  const response = {
    positions: [],
    markets: {},
    events: {},
    unavailable_exchanges: ["POLYMARKET"],
    as_of: "2026-09-17T00:00:00Z",
  };
  const c = new AgaraClient({
    ...traderOptions,
    fetch: async () => new Response(JSON.stringify(response)),
  });
  expect(await c.listPositions({ condition_ids: [], exchanges: ["AGARA", "POLYMARKET"] })).toEqual(
    response,
  );
});
it.each(["splitPosition", "mergePosition"] as const)(
  "preserves %s's POLYMARKET 202 completed receipt",
  async (name) => {
    const response = {
      operation: name === "splitPosition" ? "SPLIT" : "MERGE",
      condition_id: `0x${"aa".repeat(32)}`,
      relayer_transaction_id: "receipt",
      transaction_hash: null,
      relayer_state: "CONFIRMED",
      as_of: "2026-09-17T00:00:00Z",
    };
    const c = new AgaraClient({
      ...traderOptions,
      fetch: async () => new Response(JSON.stringify(response), { status: 202 }),
    });
    const result =
      name === "splitPosition"
        ? await c.splitPosition({
            condition_id: response.condition_id,
            collateral_amount_micro: "1000000",
          })
        : await c.mergePosition({ condition_id: response.condition_id, shares_micro: "1000000" });
    expect(result).toEqual(response);
    expect(await c.waitForPositionOperation(result)).toEqual(response);
  },
);
it("does not label read-only POST lists as ambiguous mutations", async () => {
  const c = new AgaraClient({
    ...traderOptions,
    fetch: async () => {
      throw Error("network gone");
    },
  });
  await expect(c.listOrders({})).rejects.toMatchObject({ outcomeUnknown: false });
});

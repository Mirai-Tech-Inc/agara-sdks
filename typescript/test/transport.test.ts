import { privateKeyToAccount } from "viem/accounts";
import { describe, expect, it, vi } from "vitest";
import { formatUnits, parseUnits } from "../src/amounts.js";
import { PartialAvailabilityError, ProtocolError, TransportError } from "../src/errors.js";
import { AgaraClient, assertComplete, PublicClient } from "../src/index.js";
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
const CATALOGUE = "https://catalogue.test";
const specs = { trading: contract("trading"), catalogue: contract("catalogue") };
function successBody(name: string): Record<string, unknown> {
  const endpoint = endpoints.find((e) => e.name === name);
  if (!endpoint) throw Error(`Unknown endpoint ${name}`);
  const spec = specs[endpoint.service === "router" ? "trading" : "catalogue"];
  const definitions = (spec.components as { schemas: Record<string, Schema> }).schemas;
  const operation = (spec.paths as Record<string, Record<string, Schema>>)[endpoint.path]?.[
    endpoint.method.toLowerCase()
  ] as Schema;
  const success = Object.entries(operation.responses as Record<string, Schema>).find(([status]) =>
    status.startsWith("2"),
  );
  if (!success) throw Error(`No success response for ${name}`);
  const schema = (success[1].content as { "application/json": { schema: Schema } })[
    "application/json"
  ].schema;

  return sample(schema, definitions) as Record<string, unknown>;
}
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
    expect(endpoints).toHaveLength(38);
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
      const client = new AgaraClient({
        ...traderOptions,
        catalogueBaseUrl: CATALOGUE,
        fetch: fetcher,
      });
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
        if (endpoint.name === "submitBatch")
          body = {
            ops: [
              {
                kind: "SPLIT",
                market_id: "11111111-1111-4111-8111-111111111111",
                condition_id: `0x${"11".repeat(32)}`,
                shares_micro: 1000000n,
              },
            ],
            seq: 0n,
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
      // Which deployment answers, and whether the token travels, follow from the endpoint's service
      // in the contract. A catalogue read that reached the trading origin, or carried the bearer,
      // would leak the token to a host the caller pointed somewhere else entirely.
      expect(actualUrl?.origin).toBe(
        endpoint.service === "api" ? CATALOGUE : traderOptions.baseUrl,
      );
      expect(new Headers(actualInit?.headers).get("authorization")).toBe(
        endpoint.service === "api" ? null : `Bearer ${traderOptions.token}`,
      );
      for (const [key, value] of Object.entries(query))
        expect(actualUrl?.searchParams.get(key)).toBe(
          Array.isArray(value) ? value.join(",") : String(value),
        );
      if (body !== undefined)
        expect(parseJson(String(actualInit?.body))).toEqual(parseJson(stringifyJson(body)));
    });
});
it("sends catalogue reads to the trading base URL until one is given for them", async () => {
  const seen: string[] = [];
  const fetcher: typeof fetch = async (url) => {
    seen.push(String(url));
    throw Error("recorded");
  };
  await expect(
    new PublicClient({ baseUrl: "https://one.test/", fetch: fetcher }).getCategory("crypto"),
  ).rejects.toThrow();
  await expect(
    new PublicClient({
      baseUrl: "https://one.test",
      catalogueBaseUrl: "https://two.test/gateway/",
      fetch: fetcher,
    }).getCategory("crypto"),
  ).rejects.toThrow();

  // A trailing slash on either base would otherwise produce a doubled separator the server 404s on.
  expect(seen).toEqual([
    "https://one.test/api/v1/categories/crypto",
    "https://two.test/gateway/api/v1/categories/crypto",
  ]);
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
it("runs the client observer before the per-request one, and neither on a failure", async () => {
  const order = stringifyJson(successBody("getOrder"));
  let ok = true;
  const calls: string[] = [];
  const c = new AgaraClient({
    ...traderOptions,
    onResponse: (m) => calls.push(`client ${m.status} ${m.requestId} ${m.headers.get("x-kind")}`),
    fetch: async () =>
      ok
        ? new Response(order, { headers: { "x-request-id": "audit", "x-kind": "order" } })
        : new Response('{"type":"urn:agara:problem:x","title":"x","status":500,"code":"x"}', {
            status: 500,
          }),
  });

  await c.getOrder("uuid", { onResponse: (m) => calls.push(`request ${m.status}`) });
  // A metering observer that ran after the caller's own hook would report requests the caller had
  // already acted on, and one that ran on failures would count responses that carry no result.
  expect(calls).toEqual(["client 200 audit order", "request 200"]);
  ok = false;
  await expect(c.getOrder("uuid", { onResponse: () => calls.push("request") })).rejects.toThrow();
  expect(calls).toHaveLength(2);
});
it("reports every status read a wait performs, not just the last", async () => {
  const seen: number[] = [];
  const envelope = successBody("getOrder");
  const withTerminal = (is_terminal: boolean) =>
    stringifyJson({ ...envelope, order: { ...(envelope.order as object), is_terminal } });
  const bodies = [withTerminal(false), withTerminal(false), withTerminal(true)];
  const c = new AgaraClient({
    ...traderOptions,
    onResponse: (m) => seen.push(m.status),
    fetch: async () => new Response(bodies.shift(), { status: 200 }),
  });

  expect((await c.waitForOrder("uuid", { pollIntervalMs: 0 })).is_terminal).toBe(true);
  // One observation per read: a caller metering its own request volume must see the polling reads,
  // which are the ones that multiply.
  expect(seen).toEqual([200, 200, 200]);
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
// Every deployment reached so far answers PENDING with each acquisition cost null, so the shape a
// caught-up account actually returns has never been decoded. A contract that rejected it would
// surface only once accounting had caught up, against real money.
it.each(["AVAILABLE", "PENDING", undefined])("decodes a %s portfolio unchanged", async (status) => {
  const large = "9007199254740993";
  const costs = {
    positions_value_micro: large,
    portfolio_value_micro: large,
    open_cost_basis_micro: "250000",
    open_unrealized_pnl_micro: "-125000",
  };
  const summary = {
    ...(successBody("getPortfolioSummary").summaries as Record<string, unknown>[])[0],
    ...(status === undefined ? {} : { accounting_status: status }),
    ...(status === "AVAILABLE" ? costs : {}),
  };
  const envelope = { ...successBody("getPortfolioSummary"), summaries: [summary] };
  const c = new AgaraClient({
    ...traderOptions,
    fetch: async () => new Response(stringifyJson(envelope)),
  });

  const result = await c.getPortfolioSummary();
  expect(result.summaries?.[0]).toEqual(summary);
  // Exact beyond a double's range: a decode through plain JSON would round it to ...92.
  if (status === "AVAILABLE") expect(result.summaries?.[0]?.portfolio_value_micro).toBe(large);
});
it.each(["AVAILABLE", "PENDING", undefined])("decodes a %s position unchanged", async (status) => {
  const priced = {
    avg_price_micro: "450000",
    current_price_micro: "520000",
    current_value_micro: "5200000",
    open_cost_basis_micro: "4500000",
  };
  const position = {
    ...(successBody("listPositions").positions as Record<string, unknown>[])[0],
    ...(status === undefined ? {} : { accounting_status: status }),
    ...(status === "AVAILABLE" ? priced : {}),
  };
  const envelope = { ...successBody("listPositions"), positions: [position] };
  const c = new AgaraClient({
    ...traderOptions,
    fetch: async () => new Response(stringifyJson(envelope)),
  });

  expect(
    (await c.listPositions({ condition_ids: [`0x${"11".repeat(32)}`] })).positions?.[0],
  ).toEqual(position);
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
it("preserves an unavailable exchange rather than reporting empty holdings", async () => {
  // The dangerous reading of this envelope is "no positions". It means the exchange could not be
  // consulted, so the envelope must survive intact and assertComplete must refuse it.
  const response = {
    positions: [],
    markets: {},
    events: {},
    unavailable_exchanges: ["AGARA"],
    as_of: "2026-09-17T00:00:00Z",
  };
  const c = new AgaraClient({
    ...traderOptions,
    fetch: async () => new Response(JSON.stringify(response)),
  });
  const page = await c.listPositions({
    condition_ids: [`0x${"aa".repeat(32)}`],
    exchanges: ["AGARA"],
  });
  expect(page).toEqual(response);
  expect(() => assertComplete(page)).toThrow(PartialAvailabilityError);
});
it("refuses a positions read that names no condition", async () => {
  const fetcher = vi.fn<typeof fetch>();
  const c = new AgaraClient({ ...traderOptions, fetch: fetcher });

  // The server answers an empty list with an empty envelope, which reads exactly like holding
  // nothing, so this has to fail before the request rather than return a believable zero.
  await expect(c.listPositions({ condition_ids: [] })).rejects.toBeInstanceOf(TypeError);
  await expect(
    c.listPositions({ condition_ids: [] as unknown as string[], exchanges: ["AGARA"] }),
  ).rejects.toThrow(/at least one condition_id/);
  expect(fetcher).not.toHaveBeenCalled();
});
it.each(["splitPosition", "mergePosition"] as const)(
  "reconciles %s's accepted batch through its digest",
  async (name) => {
    const batchHash = `0x${"bb".repeat(32)}`;
    const accepted = { batch_hash: batchHash, status: "PENDING", as_of: "2026-09-17T00:00:00Z" };
    const settled = {
      batch_hash: batchHash,
      status: "SETTLED",
      seq: 1,
      deadline_unix_seconds: 1900000000,
      origin: "PRESIGNED",
      tx_hash: `0x${"cc".repeat(32)}`,
      executed_at: "2026-09-17T00:00:05Z",
      failure: null,
      superseded_by_batch_hash: null,
      heals_batch_hash: null,
      unwound_at: null,
      created_at: "2026-09-17T00:00:00Z",
    };
    const c = new AgaraClient({
      ...traderOptions,
      fetch: async (url) =>
        new Response(JSON.stringify(String(url).includes("/batches/") ? settled : accepted), {
          status: String(url).includes("/batches/") ? 200 : 201,
        }),
    });
    const result =
      name === "splitPosition"
        ? await c.splitPosition({
            condition_id: `0x${"aa".repeat(32)}`,
            collateral_amount_micro: "1000000",
          })
        : await c.mergePosition({ condition_id: `0x${"aa".repeat(32)}`, shares_micro: "1000000" });
    expect(result).toEqual(accepted);
    expect(await c.waitForPositionOperation(result)).toEqual(settled);
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

// Replays responses recorded from a real deployment through the client.
//
// The endpoint mapping test in transport.test.ts builds its response body with sample() from the
// same snapshot the validator reads, so it agrees with itself whatever the server actually sends.
// That is why two LP incentive breaks reached a consumer with the suite green: a renamed field and
// a removed required field are both invisible to a fixture derived from the schema under test.
//
// These fixtures are recorded output (scripts/record-fixtures.mjs), so they are an independent
// source. A snapshot that drifts from a real payload fails here, offline, on every run.

import fs from "node:fs";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";
import { AgaraClient, ProtocolError } from "../src/index.js";
import { traderOptions } from "./helpers.js";

interface Recording {
  name: string;
  request: string;
  collection: string | null;
  elements: number | null;
  allowEmpty: boolean;
}

const dir = fileURLToPath(new URL("fixtures/deployment/", import.meta.url));

const provenance = JSON.parse(fs.readFileSync(`${dir}provenance.json`, "utf8")) as {
  deployment: string;
  platform_commit: string;
  recordings: Recording[];
};

function fixture(name: string): Record<string, unknown> {
  return JSON.parse(fs.readFileSync(`${dir}${name}.json`, "utf8"));
}

function replayFor(name: string): (client: AgaraClient) => Promise<unknown> {
  const call = replay[name];
  if (!call) throw Error(`No replay registered for the recording '${name}'`);

  return call;
}

function elements(body: Record<string, unknown>, collection: string): Record<string, unknown>[] {
  const value = body[collection];
  if (!Array.isArray(value)) throw Error(`Recording's '${collection}' is not an array`);

  return value as Record<string, unknown>[];
}

function clientReturning(body: unknown, status = 200): AgaraClient {
  return new AgaraClient({
    ...traderOptions,
    catalogueBaseUrl: traderOptions.baseUrl,
    fetch: async () =>
      new Response(JSON.stringify(body), {
        status,
        headers: { "content-type": "application/json", "x-request-id": "recorded" },
      }),
  });
}

// One call per recorded fixture. Arguments only have to satisfy local request validation; the
// injected fetch returns the recording regardless.
const replay: Record<string, (client: AgaraClient) => Promise<unknown>> = {
  listMarkets: (c) => c.listMarkets({ source: "agara", state: "ACTIVE", limit: 5 }),
  listEvents: (c) => c.listEvents({ limit: 5, include_markets: "true" }),
  listSecurities: (c) => c.listSecurities(),
  listCalendars: (c) => c.listCalendars(),
  listTradingDays: (c) => c.listTradingDays("XHKG", { from: "2026-09-01", to: "2026-09-30" }),
  search: (c) => c.search({ q: "inflation", limit: 5 }),
  listLpIncentives: (c) => c.listLpIncentives({}),
  listLpIncentiveCategories: (c) => c.listLpIncentiveCategories(),
  getLpIncentiveEarnings: (c) => c.getLpIncentiveEarnings(),
  listClosedLpIncentives: (c) => c.listClosedLpIncentives({ limit: 5 }),
  listClosedLpIncentiveCategories: (c) => c.listClosedLpIncentiveCategories(),
  listPositions: (c) => c.listPositions({ condition_ids: [`0x${"11".repeat(32)}`] }),
  getPortfolioSummary: (c) => c.getPortfolioSummary(),
  listTrades: (c) => c.listTrades({ limit: 10 }),
  listActivities: (c) => c.listActivities({ limit: 10 }),
  getRebates: (c) => c.getRebates(),
  getRealizedPnl: (c) => c.getRealizedPnl({ granularity: "day", window: "30d" }),
  listOrders: (c) => c.listOrders({ limit: 10 }),
  listOpenOrders: (c) => c.listOpenOrders({ limit: 10 }),
};

describe("recorded deployment responses", () => {
  it("covers every recording with a replay", () => {
    expect(new Set(Object.keys(replay))).toEqual(new Set(provenance.recordings.map((r) => r.name)));
  });

  for (const recording of provenance.recordings)
    it(`${recording.name} parses real output from ${recording.request}`, async () => {
      const body = fixture(recording.name);
      const result = (await replayFor(recording.name)(clientReturning(body))) as Record<
        string,
        unknown
      >;
      expect(result).toBeTruthy();

      if (!recording.collection) return;
      const collection = elements(result, recording.collection);
      expect(collection.length).toBe(recording.elements ?? 0);
      if (!recording.allowEmpty) expect(collection.length).toBeGreaterThan(0);
    });
});

// Without this, a fixture recorded against an empty collection would pass while exercising none of
// the element schema, which is the failure mode that let has_traded through.
describe("recorded collections exercise their element schema", () => {
  const populated = provenance.recordings.filter((r) => r.collection && (r.elements ?? 0) > 0);

  it("has at least one populated collection to check", () => {
    expect(populated.length).toBeGreaterThan(0);
  });

  for (const recording of populated)
    it(`${recording.name} rejects an element missing a required field`, async () => {
      const body = fixture(recording.name);
      const key = recording.collection as string;
      const first = elements(body, key)[0];
      if (!first) throw Error(`Recording '${recording.name}' has no element to damage`);

      // Which keys the schema requires is not knowable from the payload, so drop each in turn and
      // assert at least one is enforced. Picking a single key by position would pass or fail on
      // whatever order the recording happened to have.
      const rejected: string[] = [];
      for (const field of Object.keys(first)) {
        const damaged = structuredClone(body);
        const element = elements(damaged, key)[0];
        if (!element) continue;
        delete element[field];
        try {
          await replayFor(recording.name)(clientReturning(damaged));
        } catch (error) {
          if (error instanceof ProtocolError) rejected.push(field);
        }
      }

      expect(rejected.length).toBeGreaterThan(0);
    });

  it("names the element schemas no recording pins", () => {
    const unpinned = provenance.recordings
      .filter((r) => r.collection && (r.elements ?? 0) === 0)
      .map((r) => `${r.name}.${r.collection}`);

    // Not a failure: no environment has LP incentive data, so these four cannot be recorded at all
    // rather than merely not having been. Asserted so the gap is visible in review rather than
    // passing silently, and so that the list shrinks the day rewards are configured somewhere.
    // contracts/drift-allowlist.json has no bearing here; `npm run drift:check` is what covers these
    // element schemas against a live deployment, and it is the only thing that does.
    expect(unpinned).toEqual([
      "listLpIncentives.markets",
      "listLpIncentiveCategories.categories",
      "listClosedLpIncentives.markets",
      "listClosedLpIncentiveCategories.categories",
    ]);
  });
});

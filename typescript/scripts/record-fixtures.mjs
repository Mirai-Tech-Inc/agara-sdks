// Records real responses from a deployment into test/fixtures/deployment/, so the offline suite has
// payloads that did not come from the contracts it validates.
//
// Why: the endpoint mapping test builds its response body with sample() from the same snapshot the
// validator reads, so it agrees with itself no matter what the server actually sends. Recorded
// payloads are an independent source, and a snapshot that drifts from one fails offline.
//
// Read-only: every operation here is a GET or a read-only POST list. Place any orders or positions
// you want represented before running it.
//
// Usage:
//   AGARA_RECORD_BASE_URL=https://app.dev.agara.xyz AGARA_RECORD_TOKEN=agt_... \
//     AGARA_RECORD_CONDITION_IDS=0x...[,0x...] [AGARA_RECORD_GATE_COOKIE=agara_gate=...] \
//     node scripts/record-fixtures.mjs

import fs from "node:fs/promises";

const BASE_URL = (process.env.AGARA_RECORD_BASE_URL ?? "https://app.sandbox.agara.xyz").replace(
  /\/$/,
  "",
);
const TOKEN = process.env.AGARA_RECORD_TOKEN;
const GATE = process.env.AGARA_RECORD_GATE_COOKIE;
const CONDITION_IDS = (process.env.AGARA_RECORD_CONDITION_IDS ?? "").split(",").filter(Boolean);
const OUT = new URL("../test/fixtures/deployment/", import.meta.url);
// Trading days exist only around the present, so a pinned range would eventually record nothing.
const day = (offset) => {
  const now = new Date();

  return new Date(Date.UTC(now.getUTCFullYear(), now.getUTCMonth() + offset, offset ? 0 : 1))
    .toISOString()
    .slice(0, 10);
};

// Collections whose element schema is worth pinning. `collection` is the response key the fixture
// test asserts is non-empty; omit it for a non-list response.
const RECORDINGS = [
  {
    name: "listMarkets",
    path: "/api/v1/markets?source=agara&state=ACTIVE&limit=5",
    collection: "markets",
  },
  { name: "listEvents", path: "/api/v1/events?limit=5&include_markets=true", collection: "events" },
  { name: "listSecurities", path: "/api/v1/securities", collection: "securities" },
  { name: "listCalendars", path: "/api/v1/calendars", collection: "venues" },
  {
    name: "listTradingDays",
    // An equity venue: XCRYPTO trades continuously and has no trading-day rows at all.
    path: `/api/v1/calendars/XHKG/days?from=${day(0)}&to=${day(1)}`,
    collection: "days",
    auth: false,
  },
  { name: "search", path: "/api/v1/search?q=inflation&limit=5", auth: false },
  {
    name: "listLpIncentives",
    path: "/trade/v1/lp-incentives",
    collection: "markets",
    allowEmpty: true,
  },
  {
    name: "listLpIncentiveCategories",
    path: "/trade/v1/lp-incentives/categories",
    collection: "categories",
    auth: false,
    allowEmpty: true,
  },
  { name: "getLpIncentiveEarnings", path: "/trade/v1/lp-incentives/earnings" },
  {
    name: "listClosedLpIncentives",
    path: "/trade/v1/lp-incentives/closed?limit=5",
    collection: "markets",
    allowEmpty: true,
  },
  {
    name: "listClosedLpIncentiveCategories",
    path: "/trade/v1/lp-incentives/closed/categories",
    collection: "categories",
    allowEmpty: true,
  },
  {
    name: "listPositions",
    path: "/trade/v1/portfolio/positions/list",
    method: "POST",
    body: { condition_ids: CONDITION_IDS },
    collection: "positions",
  },
  { name: "getPortfolioSummary", path: "/trade/v1/portfolio/summary", collection: "summaries" },
  { name: "listTrades", path: "/trade/v1/portfolio/trades?limit=10", collection: "trades" },
  {
    name: "listActivities",
    path: "/trade/v1/portfolio/activities?limit=10",
    collection: "activities",
  },
  { name: "getRebates", path: "/trade/v1/portfolio/rebates" },
  { name: "getRealizedPnl", path: "/trade/v1/portfolio/pnl/realized?granularity=day&window=30d" },
  {
    name: "listOrders",
    path: "/trade/v1/orders/list",
    method: "POST",
    body: { limit: 10 },
    collection: "orders",
  },
  {
    name: "listOpenOrders",
    path: "/trade/v1/portfolio/open-orders/list",
    method: "POST",
    body: { limit: 10 },
    collection: "orders",
  },
];

if (!TOKEN) {
  console.error("AGARA_RECORD_TOKEN is required (a PAT with portfolio:read and orders:read).");
  process.exit(2);
}
if (!CONDITION_IDS.length) {
  console.error(
    "AGARA_RECORD_CONDITION_IDS is required: listPositions refuses an empty filter, and a market " +
      "the token's wallet holds nothing in records an empty fixture.",
  );
  process.exit(2);
}

await fs.mkdir(OUT, { recursive: true });

const provenance = [];
let failed = 0;

for (const recording of RECORDINGS) {
  const method = recording.method ?? "GET";
  const headers = { accept: "application/json" };
  if (recording.auth !== false) headers.authorization = `Bearer ${TOKEN}`;
  if (GATE) headers.cookie = GATE;
  if (recording.body) headers["content-type"] = "application/json";

  const response = await fetch(`${BASE_URL}${recording.path}`, {
    method,
    headers,
    body: recording.body ? JSON.stringify(recording.body) : undefined,
  });
  const text = await response.text();

  if (!response.ok) {
    console.error(`  skip ${recording.name}: HTTP ${response.status} ${text.slice(0, 160)}`);
    failed += 1;
    continue;
  }

  const parsed = JSON.parse(text);
  const count = recording.collection ? (parsed[recording.collection]?.length ?? null) : null;
  if (recording.collection && !count && !recording.allowEmpty) {
    console.error(
      `  skip ${recording.name}: '${recording.collection}' is empty, which would record a fixture ` +
        "that cannot exercise the element schema. Create some state and retry.",
    );
    failed += 1;
    continue;
  }

  await fs.writeFile(
    new URL(`${recording.name}.json`, OUT),
    `${JSON.stringify(parsed, null, 2)}\n`,
  );
  provenance.push({
    name: recording.name,
    request: `${method} ${recording.path}`,
    collection: recording.collection ?? null,
    elements: count,
    allowEmpty: Boolean(recording.allowEmpty),
  });
  console.log(`  recorded ${recording.name}${count === null ? "" : ` (${count} element(s))`}`);
}

const manifest = JSON.parse(
  await fs.readFile(new URL("../contracts/manifest.json", import.meta.url), "utf8"),
);
await fs.writeFile(
  new URL("provenance.json", OUT),
  `${JSON.stringify(
    {
      comment:
        "Responses recorded from a real deployment. Not derived from contracts/, so they fail offline when a snapshot drifts from what the server sends. Re-record with scripts/record-fixtures.mjs.",
      recorded_at: new Date().toISOString(),
      deployment: BASE_URL,
      platform_commit: manifest.platform_commit,
      recordings: provenance,
    },
    null,
    2,
  )}\n`,
);

console.log(`\n${provenance.length} fixture(s) written to test/fixtures/deployment/`);
if (failed) {
  console.error(`${failed} recording(s) skipped.`);
  process.exit(1);
}

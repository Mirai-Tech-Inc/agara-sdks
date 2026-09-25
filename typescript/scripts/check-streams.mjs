// Decodes every frame a live deployment sends, for each subscription form.
//
// Why this exists: there is no machine-readable contract for the WebSocket protocol to diff against.
// `/trade/v1/asyncapi.json` does not exist, and neither stream path nor any frame schema appears in
// the OpenAPI document, so `check-drift.mjs` cannot cover streams. Frame shapes are pinned only by
// contracts/ws-fixtures.json, hand-copied from the router, and the offline suite replays those
// fixtures through the same decoder that produced them — it agrees with itself no matter what the
// router now sends.
//
// So this check inverts the question: rather than comparing schemas, it makes the deployment speak
// and requires the pinned decoder to accept every frame. A shape change surfaces as a ProtocolError.
//
// Usage: AGARA_STREAM_BASE_URL=https://app.dev.agara.xyz [AGARA_STREAM_TOKEN=agt_...] \
//          [AGARA_STREAM_GATE_COOKIE=agara_gate=...] node scripts/check-streams.mjs
// Exit 0 clean, 1 on a decode failure or a silent channel, 2 when the deployment is unreachable.

import { PublicClient } from "../dist/index.js";
import { accountStream, marketStream } from "../dist/streaming.js";

const BASE_URL = (process.env.AGARA_STREAM_BASE_URL ?? "https://app.sandbox.agara.xyz").replace(
  /\/$/,
  "",
);
const TOKEN = process.env.AGARA_STREAM_TOKEN;
const GATE = process.env.AGARA_STREAM_GATE_COOKIE;
// The router heartbeats every ten seconds, so a window shorter than that can see nothing at all.
const WINDOW_MS = Number(process.env.AGARA_STREAM_WINDOW_MS ?? 25_000);

const gatedFetch = (input, init = {}) => {
  if (!GATE) return fetch(input, init);
  const headers = new Headers(init.headers);
  headers.set("cookie", GATE);

  return fetch(input, { ...init, headers });
};

// Every form needs a live subject, so discover one rather than hardcoding ids that rot.
let subject;
try {
  const client = new PublicClient({
    baseUrl: BASE_URL,
    catalogueBaseUrl: BASE_URL,
    fetch: gatedFetch,
    timeoutMs: 30_000,
  });
  const markets = await client.listMarkets({ source: "agara", state: "ACTIVE", limit: 25 });
  const market = markets.markets.find((m) => m.condition_id && m.outcomes?.[0]?.token_id);
  if (!market) throw new Error("no ACTIVE market with a condition and token");
  const events = await client.listEvents({ limit: 5 });
  subject = {
    slug: market.slug,
    tokenId: market.outcomes[0].token_id,
    conditionId: market.condition_id,
    eventId: events.events?.[0]?.id ?? null,
  };
} catch (error) {
  console.error(`Could not reach ${BASE_URL} to discover a subject: ${error.message}`);
  process.exit(2);
}

console.log(`Stream frame check against ${BASE_URL}`);
console.log(`  subject: ${subject.slug}`);

const channelSets = [
  ["orderbook", [{ name: "orderbook", token_id: subject.tokenId, depth: 10 }]],
  ["best_quote", [{ name: "best_quote", token_id: subject.tokenId }]],
  ["trades", [{ name: "trades", condition_id: subject.conditionId }]],
  ["market_status/condition", [{ name: "market_status", condition_id: subject.conditionId }]],
];
if (subject.eventId) {
  channelSets.push(["market_status/event", [{ name: "market_status", event_id: subject.eventId }]]);
}

// One connection carrying every market-side form, to check multiplexing as well.
channelSets.push(["multiplexed", channelSets.flatMap(([, channels]) => channels)]);

async function drain(label, stream) {
  const result = { label, frames: 0, ops: {}, kinds: {}, unknown: 0, gaps: 0, failure: null };
  const timer = setTimeout(() => stream.close(), WINDOW_MS);
  try {
    for await (const event of stream) {
      if (event.type === "gap") {
        result.gaps += 1;
        continue;
      }
      if (event.type !== "frame") continue;
      result.frames += 1;
      const op = event.frame.op ?? "(none)";
      result.ops[op] = (result.ops[op] ?? 0) + 1;
      if (op === "unknown") result.unknown += 1;
      const kind = event.frame.data?.kind;
      if (kind) result.kinds[kind] = (result.kinds[kind] ?? 0) + 1;
    }
  } catch (error) {
    // A ProtocolError here is the signal this check exists for: the router sent a frame the pinned
    // decoder rejects. Anything else (auth, transport) is reported the same way and still fails.
    result.failure = `${error?.constructor?.name}: ${error?.message}`;
  } finally {
    clearTimeout(timer);
  }

  return result;
}

const results = [];
for (const [label, channels] of channelSets) {
  results.push(await drain(label, marketStream({ baseUrl: BASE_URL, channels })));
}
if (TOKEN) {
  results.push(await drain("account_events", accountStream({ baseUrl: BASE_URL, token: TOKEN })));
} else {
  console.log("  (no AGARA_STREAM_TOKEN; the account channel is not checked)");
}

const failures = [];
for (const r of results) {
  const kinds = Object.keys(r.kinds);
  console.log(
    `  ${r.label.padEnd(24)} frames=${String(r.frames).padStart(3)} ops=${JSON.stringify(r.ops)}` +
      `${kinds.length ? ` kinds=${kinds.join(",")}` : ""}${r.unknown ? ` unknown=${r.unknown}` : ""}`,
  );
  if (r.failure) failures.push(`${r.label}: ${r.failure}`);
  // Silence is a failure, not a pass: heartbeats alone should arrive inside the window.
  else if (r.frames === 0) failures.push(`${r.label}: no frames in ${WINDOW_MS}ms`);
}

const unknown = results.reduce((sum, r) => sum + r.unknown, 0);
if (unknown) {
  console.log(
    `\n${unknown} frame(s) decoded as op "unknown". Not a failure: an unrecognised update kind is\n` +
      "the forward-compatibility path. It does mean the router emits something this package does not\n" +
      "model yet, so refresh contracts/ws-fixtures.json and the decoder when convenient.",
  );
}

if (failures.length) {
  console.log("\nFAILURES:");
  for (const f of failures) console.log(`  ! ${f}`);
  console.log(
    "\nA ProtocolError means the deployment sent a frame the pinned decoder rejects: reconcile\n" +
      "src/stream-decode.ts and contracts/ws-fixtures.json against the router's producers.",
  );
  process.exit(1);
}

console.log("\nEvery frame decoded.");

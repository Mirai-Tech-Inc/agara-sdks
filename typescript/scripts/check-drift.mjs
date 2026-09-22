// Compares the committed contract snapshots against the OpenAPI a live deployment serves, for the
// operations in contracts/manifest.json only.
//
// Why this exists: the snapshots are a hand-scoped copy of the platform's specs, so nothing local
// notices when the platform changes a field. Twice that shipped a broken client — a renamed field
// killed getLpIncentiveEarnings outright, and a removed required field left listLpIncentives
// throwing for any non-empty page while an empty-array fixture kept the suite green.
//
// Usage: AGARA_DRIFT_BASE_URL=https://app.dev.agara.xyz node scripts/check-drift.mjs
// Exits 1 on a breaking difference, 0 otherwise. Non-breaking differences print as warnings.

import fs from "node:fs/promises";

const BASE_URL = (process.env.AGARA_DRIFT_BASE_URL ?? "https://app.sandbox.agara.xyz").replace(
  /\/$/,
  "",
);
// Private deployments sit behind a shared-passphrase cookie gate; /trade/v1 and /api/v1 are not
// gated today, but send it when supplied so this keeps working if that changes.
const GATE = process.env.AGARA_DRIFT_GATE_COOKIE;
const TIMEOUT_MS = Number(process.env.AGARA_DRIFT_TIMEOUT_MS ?? 30_000);

const url = (path) => new URL(path, `${BASE_URL}/`).toString();

async function fetchSpec(path) {
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), TIMEOUT_MS);
  try {
    const response = await fetch(url(path), {
      signal: controller.signal,
      headers: GATE ? { cookie: GATE } : undefined,
    });
    if (!response.ok) throw new Error(`${path} responded ${response.status}`);

    return await response.json();
  } finally {
    clearTimeout(timer);
  }
}

function deref(spec, node, seen, depth) {
  if (!node || depth > 8) return node;
  if (node.$ref) {
    const key = node.$ref.replace("#/components/schemas/", "");
    if (seen.has(key)) return { circular: key };

    return deref(spec, spec.components?.schemas?.[key], new Set([...seen, key]), depth + 1);
  }

  return node;
}

// A tagged union's variants must be compared like for like. Merging them into one bag makes every
// field of a variant the other document lacks look like a removed field, which buried the two real
// breaks under forty false ones. Prefer the single-valued `kind` discriminator as the label.
function variantLabel(spec, variant, index, depth) {
  const node = deref(spec, variant, new Set(), depth);
  for (const name of ["kind", "op", "type"]) {
    const values = deref(spec, node?.properties?.[name], new Set(), depth)?.enum;
    if (values?.length === 1) return String(values[0]);
  }

  return `#${index}`;
}

// Flattens a schema into { "path": "required"|"optional", "path#enum": "a|b" } so two documents can
// be compared without depending on how either one factors its $refs.
function flatten(spec, schema, prefix = "", depth = 0, out = {}, seen = new Set()) {
  const node = deref(spec, schema, seen, depth);
  if (!node || depth > 5) return out;
  if (node.enum) out[`${prefix}#enum`] = [...node.enum].map(String).sort().join("|");

  const variants = node.oneOf ?? node.anyOf;
  if (variants) {
    for (const [index, variant] of variants.entries()) {
      flatten(
        spec,
        variant,
        `${prefix}<${variantLabel(spec, variant, index, depth)}>`,
        depth + 1,
        out,
        seen,
      );
    }
  }
  for (const variant of node.allOf ?? []) flatten(spec, variant, prefix, depth + 1, out, seen);
  if (node.items) flatten(spec, node.items, `${prefix}[]`, depth + 1, out, seen);
  if (node.properties) {
    const required = new Set(node.required ?? []);
    for (const [name, value] of Object.entries(node.properties)) {
      const path = prefix ? `${prefix}.${name}` : name;
      out[path] = required.has(name) ? "required" : "optional";
      flatten(spec, value, path, depth + 1, out, seen);
    }
  }

  return out;
}

// One line per union variant that only one document knows about, instead of one per field of it.
function collapseVariants(differences) {
  const wholeVariant = new Map();
  for (const d of differences) {
    const match = /^(request|response) (.*?<[^>]+>)\./.exec(d.difference);
    if (!match) continue;
    const key = `${d.operation}|${match[1]} ${match[2]}`;
    const side = /deployment absent$/.test(d.difference)
      ? "deployment"
      : /snapshot absent,/.test(d.difference)
        ? "snapshot"
        : null;
    if (!side) {
      wholeVariant.set(key, null);
      continue;
    }
    const seenSide = wholeVariant.get(key);
    wholeVariant.set(key, seenSide === undefined || seenSide === side ? side : null);
  }

  const emitted = new Set();
  const out = [];
  for (const d of differences) {
    const match = /^(request|response) (.*?<[^>]+>)\./.exec(d.difference);
    const key = match ? `${d.operation}|${match[1]} ${match[2]}` : null;
    const side = key ? wholeVariant.get(key) : null;
    if (!key || !side) {
      out.push(d);
      continue;
    }
    if (emitted.has(key)) continue;
    emitted.add(key);
    out.push({
      operation: d.operation,
      difference: `${match[1]} variant ${match[2]}: present only in the ${side === "deployment" ? "snapshot" : "deployment"}`,
    });
  }

  return out;
}

const requestSchema = (op) => op?.requestBody?.content?.["application/json"]?.schema;

function responseSchema(op) {
  const code = Object.keys(op?.responses ?? {}).find((c) => c.startsWith("2"));

  return code ? op.responses[code].content?.["application/json"]?.schema : undefined;
}

const members = (value) => new Set(value ? value.split("|") : []);
const superset = (a, b) => [...b].every((v) => a.has(v));

// A difference is breaking when it makes a correct server exchange fail in the client:
// a required response field the server no longer sends, a request field the server now requires,
// or a response enum that can carry a value the snapshot does not admit.
function classify(kind, key, mine, theirs) {
  if (key.endsWith("#enum")) {
    if (mine === theirs) return null;
    // A response enum the snapshot does not cover means the server can send a value the client
    // rejects. The reverse (a wider snapshot) still accepts everything the server sends.
    if (kind === "response") return superset(members(mine), members(theirs)) ? "warn" : "break";

    // Request enums never break a correct exchange: a wider snapshot earns a server 422, and a
    // narrower one refuses calls the API would serve. Both want a human, neither wants a red build.
    return "warn";
  }
  if (kind === "response") {
    if (theirs === undefined) return mine === "required" ? "break" : "warn";
    if (mine === undefined) return "warn";

    return mine === "optional" && theirs === "required" ? "warn" : "break";
  }
  if (theirs === "required" && mine !== "required") return "break";
  if (mine === undefined || theirs === undefined) return "warn";

  return "warn";
}

const [manifest, snapshots, allowlist] = await Promise.all([
  fs.readFile(new URL("../contracts/manifest.json", import.meta.url), "utf8").then(JSON.parse),
  Promise.all(
    ["trading", "catalogue"].map((name) =>
      fs.readFile(new URL(`../contracts/${name}.json`, import.meta.url), "utf8").then(JSON.parse),
    ),
  ).then(([trading, catalogue]) => ({ router: trading, api: catalogue })),
  fs
    .readFile(new URL("../contracts/drift-allowlist.json", import.meta.url), "utf8")
    .then(JSON.parse)
    .catch(() => ({ deliberate: [] })),
]);

const inventory = JSON.parse(
  await fs.readFile(new URL("../contracts/endpoints.json", import.meta.url), "utf8"),
);

let live;
try {
  const [trading, catalogue] = await Promise.all([
    fetchSpec("trade/v1/openapi.json"),
    fetchSpec("api/v1/openapi.json"),
  ]);
  live = { router: trading, api: catalogue };
} catch (error) {
  console.error(`Could not read the deployed OpenAPI from ${BASE_URL}: ${error.message}`);
  console.error("Set AGARA_DRIFT_BASE_URL to a reachable deployment.");
  process.exit(2);
}

// Entries match by exact `difference` or by `pattern` (a regex), and by exact `operation` or "*".
const allowRules = allowlist.deliberate.map((entry) => ({
  operation: entry.operation ?? "*",
  exact: entry.difference,
  pattern: entry.pattern ? new RegExp(entry.pattern) : null,
}));
const isAllowed = (operation, difference) =>
  allowRules.some(
    (rule) =>
      (rule.operation === "*" || rule.operation === operation) &&
      (rule.exact
        ? rule.exact === difference
        : rule.pattern
          ? rule.pattern.test(difference)
          : false),
  );
const breaks = [];
const warns = [];
let identical = 0;

for (const entry of inventory) {
  if (entry.protocol !== "HTTP") continue;
  const service = entry.service === "router" ? "router" : "api";
  const method = entry.method.toLowerCase();
  const mine = snapshots[service].paths?.[entry.path]?.[method];
  const theirs = live[service].paths?.[entry.path]?.[method];

  const record = (level, difference) => {
    if (isAllowed(entry.name, difference)) return;
    (level === "break" ? breaks : warns).push({ operation: entry.name, difference });
  };

  if (!mine) {
    record("break", "absent from the committed snapshot");
    continue;
  }
  if (!theirs) {
    record("warn", `absent from the deployment (${entry.method} ${entry.path})`);
    continue;
  }

  const before = breaks.length + warns.length;

  const params = (op) =>
    Object.fromEntries((op.parameters ?? []).map((p) => [`${p.in}:${p.name}`, p]));
  const mineParams = params(mine);
  const theirParams = params(theirs);
  for (const key of new Set([...Object.keys(mineParams), ...Object.keys(theirParams)])) {
    const a = mineParams[key];
    const b = theirParams[key];
    if (!a) {
      record("warn", `param ${key}: only on the deployment`);
      continue;
    }
    if (!b) {
      record("warn", `param ${key}: only in the snapshot`);
      continue;
    }
    if (!a.required && b.required) record("break", `param ${key}: now required by the deployment`);
    const mineEnum = flatten(snapshots[service], a.schema)["#enum"];
    const theirEnum = flatten(live[service], b.schema)["#enum"];
    if (mineEnum !== theirEnum) {
      const level = classify("request", "#enum", mineEnum, theirEnum);
      if (level) record(level, `param ${key} enum: [${mineEnum ?? "-"}] vs [${theirEnum ?? "-"}]`);
    }
  }

  for (const [kind, pick] of [
    ["request", requestSchema],
    ["response", responseSchema],
  ]) {
    const a = pick(mine) ? flatten(snapshots[service], pick(mine)) : {};
    const b = pick(theirs) ? flatten(live[service], pick(theirs)) : {};
    for (const key of new Set([...Object.keys(a), ...Object.keys(b)])) {
      if (a[key] === b[key]) continue;
      const level = classify(kind, key, a[key], b[key]);
      if (!level) continue;
      record(
        level,
        `${kind} ${key}: snapshot ${a[key] ?? "absent"}, deployment ${b[key] ?? "absent"}`,
      );
    }
  }

  if (breaks.length + warns.length === before) identical += 1;
}

const collapsedWarns = collapseVariants(warns);
const collapsedBreaks = collapseVariants(breaks);

const total = inventory.filter((e) => e.protocol === "HTTP").length;
console.log(`Contract drift against ${BASE_URL}`);
console.log(`  snapshot pinned at:  platform ${manifest.platform_commit}`);
console.log(`  operations compared: ${total}`);
console.log(`  no difference:       ${identical}`);
console.log(`  allowlisted rules:   ${allowRules.length}`);
console.log(`  warnings:            ${collapsedWarns.length}`);
console.log(`  breaking:            ${collapsedBreaks.length}`);

if (collapsedWarns.length) {
  console.log("\nNon-breaking differences (the client still works; review when convenient):");
  for (const w of collapsedWarns) console.log(`  ~ ${w.operation}: ${w.difference}`);
}

if (collapsedBreaks.length) {
  console.log("\nBREAKING differences (a correct server exchange will fail in the client):");
  for (const b of collapsedBreaks) console.log(`  ! ${b.operation}: ${b.difference}`);
  console.log(
    "\nRefresh the affected schemas in contracts/, run `npm run generate`, and bump\n" +
      "`platform_commit` in contracts/manifest.json. If a difference is deliberate, record it\n" +
      "in contracts/drift-allowlist.json with the reason.",
  );
  process.exit(1);
}

console.log("\nNo breaking drift.");

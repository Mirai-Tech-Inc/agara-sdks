import fs from "node:fs/promises";
import { endpointDocs } from "./endpoint-docs.mjs";

const directory = new URL("../", import.meta.url);
const names = {
  status: "getStatus",
  create_order: "placeOrder",
  create_signed_order: "placeSignedOrder",
  create_signed_order_batch: "placeSignedOrders",
  list_orders: "listOrders",
  get_order: "getOrder",
  cancel_order: "cancelOrder",
  get_order_by_hash: "getOrderByHash",
  get_order_trades: "getOrderTrades",
  cancel_all_orders: "cancelAllOrders",
  create_batch: "submitBatch",
  get_batch: "getBatch",
  supersede_batch: "supersedeBatch",
  get_batch_group: "getBatchGroup",
  agara_orderbook: "getOrderbook",
  portfolio_activities: "listActivities",
  portfolio_bridge_deposit_address: "getBridgeDepositAddress",
  portfolio_bridge_deposit_supported_assets: "getBridgeDepositAssets",
  portfolio_bridge_deposit_quote: "quoteBridgeDeposit",
  portfolio_bridge_withdraw_supported_assets: "getBridgeWithdrawAssets",
  portfolio_bridge_withdraw_quote: "quoteBridgeWithdrawal",
  portfolio_open_orders_list: "listOpenOrders",
  portfolio_positions_list: "listPositions",
  portfolio_positions_split: "splitPosition",
  portfolio_positions_merge: "mergePosition",
  portfolio_summary: "getPortfolioSummary",
  portfolio_trades: "listTrades",
  portfolio_rebates: "getRebates",
  lp_incentives: "listLpIncentives",
  lp_incentive_categories: "listLpIncentiveCategories",
  closed_lp_incentives: "listClosedLpIncentives",
  closed_lp_incentive_categories: "listClosedLpIncentiveCategories",
  lp_incentive_earnings: "getLpIncentiveEarnings",
  portfolio_pnl: "getPnl",
  portfolio_pnl_history: "getPnlHistory",
  portfolio_realized_pnl: "getRealizedPnl",
  getApiV1CalendarsByMicDaysByDate: "getTradingDay",
  getApiV1CalendarsByMicDays: "listTradingDays",
  getApiV1CalendarsByMicNextSession: "getNextSession",
  getApiV1Calendars: "listCalendars",
  getApiV1CalendarsByMic: "getCalendar",
  getApiV1CategoriesBySlug: "getCategory",
  getApiV1EventsBySlugCanonicalUrl: "getEventCanonicalUrl",
  getApiV1EventsBySlug: "getEvent",
  getApiV1Events: "listEvents",
  getApiV1MarketsById: "getMarket",
  getApiV1Markets: "listMarkets",
  getApiV1PricesPoint: "getPricePoint",
  getApiV1PricesTicks: "getPriceTicks",
  getApiV1PricesTokenHistory: "getTokenHistory",
  getApiV1Search: "search",
  getApiV1Securities: "listSecurities",
  getApiV1SecuritiesBySymbol: "getSecurity",
};
const manifest = JSON.parse(
  await fs.readFile(new URL("contracts/manifest.json", directory), "utf8"),
);
const schemas = {};
for (const name of ["trading", "catalogue"])
  schemas[name] = JSON.parse(
    await fs.readFile(new URL(`contracts/${name}.json`, directory), "utf8"),
  );
let output =
  'import { Transport, type ClientOptions, type RequestOptions } from "./transport.js";\nimport type { TradingOperations, CatalogueOperations, RequestBody, Query, Success, OrderRequest, SignedOrderRequest, SignedOrderBatchRequest, BatchSubmission, BatchSupersedeSubmission } from "./types.js";\n';
const wrappers = { public: [], private: [] };
const inventory = [];
const runtime = {};
for (const endpoint of manifest.operations.filter((o) => o.protocol === "HTTP")) {
  const family = endpoint.service === "router" ? "trading" : "catalogue";
  const operation = schemas[family].paths[endpoint.path][endpoint.method.toLowerCase()];
  const id = operation.operationId;
  const name = names[id];
  if (!name) throw new Error(`Missing name for ${id}`);
  const operationType = `${family === "trading" ? "TradingOperations" : "CatalogueOperations"}["${id}"]`;
  const args = [];
  const params = operation.parameters ?? [];
  for (const p of params.filter((p) => p.in === "path")) args.push(`${p.name}: string`);
  if (operation.requestBody)
    args.push(
      `body: ${id === "create_signed_order_batch" ? "SignedOrderBatchRequest" : id === "create_batch" ? "BatchSubmission" : id === "supersede_batch" ? "BatchSupersedeSubmission" : id === "create_order" ? "OrderRequest" : id === "create_signed_order" ? "SignedOrderRequest" : `RequestBody<${operationType}>`}`,
    );
  const query = params.filter((p) => p.in === "query");
  if (query.length)
    args.push(`query: Query<${operationType}>${query.some((p) => p.required) ? "" : " = {}"}`);
  args.push("options: RequestOptions = {}");
  const url = endpoint.path.replace(/\{(.*?)\}/g, (_, p) => `\${this.pathPart(${p})}`);
  const auth = !endpoint.auth.startsWith("anonymous");
  const wrapper = `${methodComment(name, endpoint, operation, params, query)}  ${name}(${args.join(", ")}): Promise<Success<${operationType}>> {\n    return this.request("${endpoint.method}", \`${url}\`, ${operation.requestBody ? "body" : "undefined"}, ${query.length ? "query" : "undefined"}, options, ${auth}, "${name}");\n  }\n`;
  wrappers[auth ? "private" : "public"].push(wrapper);
  inventory.push({ name, ...endpoint, operationId: id });
  runtime[name] = renameRefs(
    {
      responses: Object.fromEntries(
        Object.entries(operation.responses ?? {})
          .filter(([status]) => status.startsWith("2"))
          .map(([status, response]) => [status, response.content?.["application/json"]?.schema]),
      ),
      body: operation.requestBody?.content?.["application/json"]?.schema,
      query: Object.fromEntries(
        query.map((p) => [p.name, { ...p.schema, required: !!p.required }]),
      ),
    },
    family,
  );
}
if (inventory.length !== Object.keys(endpointDocs).length)
  throw new Error("Endpoint documentation inventory differs from generated client methods");
output += `${docComment(
  [
    "Public discovery and trading-data client; construction does not require credentials.",
    "",
    "@remarks",
    "Configure service URLs, request deadlines and optional credentials through ClientOptions.",
    "Methods preserve full response envelopes. HTTP requests are sent once with redirects disabled;",
    "server failures, transport failures and malformed success responses have distinct error classes.",
    "An optional PAT personalizes eligible trading-data routes such as LP standing.",
  ],
  "",
)}export class PublicClient extends Transport {\n${wrappers.public.join("\n")}\n}\n`;
output += `${docComment(
  [
    "Personal-access-token client for order, portfolio and account-batch operations.",
    "",
    "@remarks",
    "Each method documents its required PAT scope. Acceptance of an asynchronous operation is not",
    "proof of a fill or settlement. Reconcile uncertain submissions using their existing identities;",
    "this client never automatically replays HTTP mutations. AgaraClient adds polling and pagination.",
  ],
  "",
)}export class TraderClient extends PublicClient {\n${docComment([
  "Configure an authenticated client without sending a request or validating the token remotely.",
  "",
  "@param options - Required PAT plus optional URLs, fetch transport, deadlines and response observers.",
  "@throws TypeError for invalid URL or token text.",
  "@throws RangeError for invalid default timeout or byte limit.",
])} constructor(options: ClientOptions & { /** Personal access token with each requested operation's scopes. */ token: string }) { super(options); }\n${wrappers.private.join("\n")}\n}\n`;
await fs.writeFile(new URL("src/endpoints.ts", directory), output);
await fs.writeFile(
  new URL("contracts/endpoints.json", directory),
  `${JSON.stringify(inventory, null, 2)}\n`,
);
function renameRefs(value, family) {
  if (Array.isArray(value)) return value.map((v) => renameRefs(v, family));
  if (value && typeof value === "object")
    return Object.fromEntries(
      Object.entries(value).map(([key, v]) => [
        key,
        key === "$ref"
          ? v.replace("#/components/schemas/", `#/components/schemas/${family}.`)
          : renameRefs(v, family),
      ]),
    );
  return value;
}
const definitions = Object.fromEntries(
  Object.entries(schemas).flatMap(([family, schema]) =>
    Object.entries(schema.components.schemas).map(([name, value]) => [
      `${family}.${name}`,
      renameRefs(value, family),
    ]),
  ),
);
function compact(v, preserveKeys = false) {
  if (Array.isArray(v)) return v.map((item) => compact(item));
  if (v && typeof v === "object")
    return Object.fromEntries(
      Object.entries(v)
        .filter(
          ([k]) =>
            preserveKeys ||
            !["description", "example", "examples", "title", "default", "deprecated"].includes(k),
        )
        .map(([k, x]) => [k, compact(x, k === "properties")]),
    );
  return v;
}
await fs.writeFile(
  new URL("src/generated/requests.ts", directory),
  `export const requestContracts = ${JSON.stringify(compact(runtime))};\nexport const definitions = ${JSON.stringify(compact(definitions))};\n`,
);

function methodComment(name, endpoint, operation, params, query) {
  const docs = endpointDocs[name];
  if (!docs?.summary || !docs.returns || !docs.remarks)
    throw new Error(`Missing endpoint documentation for ${name}`);
  const scopes = (operation.security ?? []).flatMap(
    (security) => security.personal_access_token ?? [],
  );
  const fallbackScopes = {
    OrdersPlace: "orders:place",
    OrdersPlaceSigned: "orders:place_signed",
    OrdersRead: "orders:read",
    OrdersCancel: "orders:cancel",
    OrdersCancelAll: "orders:cancel_all",
    BatchesSubmit: "batches:submit",
    PortfolioRead: "portfolio:read",
    PositionsSplit: "positions:split",
    PositionsMerge: "positions:merge",
  };
  if (!endpoint.auth.startsWith("anonymous") && !scopes.length) {
    const scope = fallbackScopes[endpoint.auth.split(": ").at(-1)];
    if (!scope) throw new Error(`Missing PAT scope documentation for ${name}`);
    scopes.push(scope);
  }
  const authentication = endpoint.auth.startsWith("anonymous")
    ? "Callable without credentials."
    : `Requires a personal access token with scope ${scopes.map((scope) => `\`${scope}\``).join(", ")}.`;
  const mutation =
    !["GET", "HEAD"].includes(endpoint.method) &&
    ![
      "listOrders",
      "listOpenOrders",
      "listPositions",
      "quoteBridgeDeposit",
      "quoteBridgeWithdrawal",
    ].includes(name);
  const lines = [docs.summary, "", "@remarks", `${authentication} ${docs.remarks}`, ""];
  for (const param of params.filter((param) => param.in === "path")) {
    const description = docs.params?.[param.name];
    if (!description) throw new Error(`Missing ${name}.${param.name} documentation`);
    lines.push(`@param ${param.name} - ${description}`);
  }
  if (operation.requestBody) {
    if (!docs.body) throw new Error(`Missing ${name} body documentation`);
    lines.push(`@param body - ${docs.body}`);
  }
  if (query.length) {
    if (!docs.query) throw new Error(`Missing ${name} query documentation`);
    lines.push(`@param query - ${docs.query}`);
  }
  lines.push(
    "@param options - Per-request abort signal, timeout override and successful-response observer.",
    `@returns ${docs.returns}`,
    "@throws TypeError for invalid local request shapes or text inputs.",
    "@throws RangeError for invalid local numeric inputs or timeout overrides.",
    "@throws AgaraError for an unsuccessful HTTP response; inspect its status and validated recovery.",
    mutation
      ? "@throws TransportError for failed or aborted I/O; a sent mutation may still complete."
      : "@throws TransportError for failed or aborted I/O; inspect its cause for the underlying failure.",
    "@throws ProtocolError when a successful response is malformed or exceeds the response-byte bound.",
    "@throws ResponseObserverError when a callback throws after receiving a valid success response.",
  );
  return docComment(lines);
}

function docComment(paragraphs, indent = "  ") {
  const lines = [];
  const width = 100 - indent.length - 3;
  for (const paragraph of paragraphs) {
    if (paragraph.includes("*/"))
      throw new Error("Endpoint documentation cannot terminate its comment");
    if (!paragraph) {
      lines.push("");
      continue;
    }
    let line = "";
    for (const word of paragraph.split(/\s+/)) {
      if (line && line.length + word.length + 1 > width) {
        lines.push(line);
        line = "";
      }
      line += `${line ? " " : ""}${word}`;
    }
    lines.push(line);
  }
  return `${indent}/**\n${lines.map((line) => `${indent} *${line ? ` ${line}` : ""}`).join("\n")}\n${indent} */\n`;
}

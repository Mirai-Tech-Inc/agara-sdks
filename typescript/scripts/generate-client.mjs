import fs from "node:fs/promises";

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
  const wrapper = `  ${name}(${args.join(", ")}): Promise<Success<${operationType}>> {\n    return this.request("${endpoint.method}", \`${url}\`, ${operation.requestBody ? "body" : "undefined"}, ${query.length ? "query" : "undefined"}, options, ${auth}, "${name}");\n  }\n`;
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
output += `export class PublicClient extends Transport {\n${wrappers.public.join("\n")}\n}\n`;
output += `export class TraderClient extends PublicClient {\n constructor(options: ClientOptions & { token: string }) { super(options); }\n${wrappers.private.join("\n")}\n}\n`;
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

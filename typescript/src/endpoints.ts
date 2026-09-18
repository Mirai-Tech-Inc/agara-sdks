import { type ClientOptions, type RequestOptions, Transport } from "./transport.js";
import type {
  BatchSubmission,
  BatchSupersedeSubmission,
  CatalogueOperations,
  OrderRequest,
  Query,
  RequestBody,
  SignedOrderBatchRequest,
  SignedOrderRequest,
  Success,
  TradingOperations,
} from "./types.js";
export class PublicClient extends Transport {
  getStatus(options: RequestOptions = {}): Promise<Success<TradingOperations["status"]>> {
    return this.request(
      "GET",
      `/trade/v1/status`,
      undefined,
      undefined,
      options,
      false,
      "getStatus",
    );
  }

  getOrderbook(
    token_id: string,
    options: RequestOptions = {},
  ): Promise<Success<TradingOperations["agara_orderbook"]>> {
    return this.request(
      "GET",
      `/trade/v1/orderbook/${this.pathPart(token_id)}`,
      undefined,
      undefined,
      options,
      false,
      "getOrderbook",
    );
  }

  listLpIncentives(
    query: Query<TradingOperations["lp_incentives"]> = {},
    options: RequestOptions = {},
  ): Promise<Success<TradingOperations["lp_incentives"]>> {
    return this.request(
      "GET",
      `/trade/v1/lp-incentives`,
      undefined,
      query,
      options,
      false,
      "listLpIncentives",
    );
  }

  listLpIncentiveCategories(
    options: RequestOptions = {},
  ): Promise<Success<TradingOperations["lp_incentive_categories"]>> {
    return this.request(
      "GET",
      `/trade/v1/lp-incentives/categories`,
      undefined,
      undefined,
      options,
      false,
      "listLpIncentiveCategories",
    );
  }

  getTradingDay(
    mic: string,
    date: string,
    options: RequestOptions = {},
  ): Promise<Success<CatalogueOperations["getApiV1CalendarsByMicDaysByDate"]>> {
    return this.request(
      "GET",
      `/api/v1/calendars/${this.pathPart(mic)}/days/${this.pathPart(date)}`,
      undefined,
      undefined,
      options,
      false,
      "getTradingDay",
    );
  }

  listTradingDays(
    mic: string,
    query: Query<CatalogueOperations["getApiV1CalendarsByMicDays"]>,
    options: RequestOptions = {},
  ): Promise<Success<CatalogueOperations["getApiV1CalendarsByMicDays"]>> {
    return this.request(
      "GET",
      `/api/v1/calendars/${this.pathPart(mic)}/days`,
      undefined,
      query,
      options,
      false,
      "listTradingDays",
    );
  }

  getNextSession(
    mic: string,
    query: Query<CatalogueOperations["getApiV1CalendarsByMicNextSession"]>,
    options: RequestOptions = {},
  ): Promise<Success<CatalogueOperations["getApiV1CalendarsByMicNextSession"]>> {
    return this.request(
      "GET",
      `/api/v1/calendars/${this.pathPart(mic)}/next-session`,
      undefined,
      query,
      options,
      false,
      "getNextSession",
    );
  }

  listCalendars(
    options: RequestOptions = {},
  ): Promise<Success<CatalogueOperations["getApiV1Calendars"]>> {
    return this.request(
      "GET",
      `/api/v1/calendars`,
      undefined,
      undefined,
      options,
      false,
      "listCalendars",
    );
  }

  getCalendar(
    mic: string,
    options: RequestOptions = {},
  ): Promise<Success<CatalogueOperations["getApiV1CalendarsByMic"]>> {
    return this.request(
      "GET",
      `/api/v1/calendars/${this.pathPart(mic)}`,
      undefined,
      undefined,
      options,
      false,
      "getCalendar",
    );
  }

  getCategory(
    slug: string,
    options: RequestOptions = {},
  ): Promise<Success<CatalogueOperations["getApiV1CategoriesBySlug"]>> {
    return this.request(
      "GET",
      `/api/v1/categories/${this.pathPart(slug)}`,
      undefined,
      undefined,
      options,
      false,
      "getCategory",
    );
  }

  getEventCanonicalUrl(
    slug: string,
    options: RequestOptions = {},
  ): Promise<Success<CatalogueOperations["getApiV1EventsBySlugCanonicalUrl"]>> {
    return this.request(
      "GET",
      `/api/v1/events/${this.pathPart(slug)}/canonical-url`,
      undefined,
      undefined,
      options,
      false,
      "getEventCanonicalUrl",
    );
  }

  getEvent(
    slug: string,
    options: RequestOptions = {},
  ): Promise<Success<CatalogueOperations["getApiV1EventsBySlug"]>> {
    return this.request(
      "GET",
      `/api/v1/events/${this.pathPart(slug)}`,
      undefined,
      undefined,
      options,
      false,
      "getEvent",
    );
  }

  listEvents(
    query: Query<CatalogueOperations["getApiV1Events"]> = {},
    options: RequestOptions = {},
  ): Promise<Success<CatalogueOperations["getApiV1Events"]>> {
    return this.request("GET", `/api/v1/events`, undefined, query, options, false, "listEvents");
  }

  getMarket(
    id: string,
    options: RequestOptions = {},
  ): Promise<Success<CatalogueOperations["getApiV1MarketsById"]>> {
    return this.request(
      "GET",
      `/api/v1/markets/${this.pathPart(id)}`,
      undefined,
      undefined,
      options,
      false,
      "getMarket",
    );
  }

  listMarkets(
    query: Query<CatalogueOperations["getApiV1Markets"]>,
    options: RequestOptions = {},
  ): Promise<Success<CatalogueOperations["getApiV1Markets"]>> {
    return this.request("GET", `/api/v1/markets`, undefined, query, options, false, "listMarkets");
  }

  getPricePoint(
    query: Query<CatalogueOperations["getApiV1PricesPoint"]>,
    options: RequestOptions = {},
  ): Promise<Success<CatalogueOperations["getApiV1PricesPoint"]>> {
    return this.request(
      "GET",
      `/api/v1/prices/point`,
      undefined,
      query,
      options,
      false,
      "getPricePoint",
    );
  }

  getPriceTicks(
    query: Query<CatalogueOperations["getApiV1PricesTicks"]>,
    options: RequestOptions = {},
  ): Promise<Success<CatalogueOperations["getApiV1PricesTicks"]>> {
    return this.request(
      "GET",
      `/api/v1/prices/ticks`,
      undefined,
      query,
      options,
      false,
      "getPriceTicks",
    );
  }

  getTokenHistory(
    query: Query<CatalogueOperations["getApiV1PricesTokenHistory"]>,
    options: RequestOptions = {},
  ): Promise<Success<CatalogueOperations["getApiV1PricesTokenHistory"]>> {
    return this.request(
      "GET",
      `/api/v1/prices/token-history`,
      undefined,
      query,
      options,
      false,
      "getTokenHistory",
    );
  }

  search(
    query: Query<CatalogueOperations["getApiV1Search"]>,
    options: RequestOptions = {},
  ): Promise<Success<CatalogueOperations["getApiV1Search"]>> {
    return this.request("GET", `/api/v1/search`, undefined, query, options, false, "search");
  }

  listSecurities(
    options: RequestOptions = {},
  ): Promise<Success<CatalogueOperations["getApiV1Securities"]>> {
    return this.request(
      "GET",
      `/api/v1/securities`,
      undefined,
      undefined,
      options,
      false,
      "listSecurities",
    );
  }

  getSecurity(
    symbol: string,
    options: RequestOptions = {},
  ): Promise<Success<CatalogueOperations["getApiV1SecuritiesBySymbol"]>> {
    return this.request(
      "GET",
      `/api/v1/securities/${this.pathPart(symbol)}`,
      undefined,
      undefined,
      options,
      false,
      "getSecurity",
    );
  }
}
export class TraderClient extends PublicClient {
  constructor(options: ClientOptions & { token: string }) {
    super(options);
  }
  placeOrder(
    body: OrderRequest,
    options: RequestOptions = {},
  ): Promise<Success<TradingOperations["create_order"]>> {
    return this.request("POST", `/trade/v1/orders`, body, undefined, options, true, "placeOrder");
  }

  placeSignedOrder(
    body: SignedOrderRequest,
    options: RequestOptions = {},
  ): Promise<Success<TradingOperations["create_signed_order"]>> {
    return this.request(
      "POST",
      `/trade/v1/orders/signed`,
      body,
      undefined,
      options,
      true,
      "placeSignedOrder",
    );
  }

  placeSignedOrders(
    body: SignedOrderBatchRequest,
    options: RequestOptions = {},
  ): Promise<Success<TradingOperations["create_signed_order_batch"]>> {
    return this.request(
      "POST",
      `/trade/v1/orders/signed/batch`,
      body,
      undefined,
      options,
      true,
      "placeSignedOrders",
    );
  }

  listOrders(
    body: RequestBody<TradingOperations["list_orders"]>,
    options: RequestOptions = {},
  ): Promise<Success<TradingOperations["list_orders"]>> {
    return this.request(
      "POST",
      `/trade/v1/orders/list`,
      body,
      undefined,
      options,
      true,
      "listOrders",
    );
  }

  getOrder(
    order_id: string,
    options: RequestOptions = {},
  ): Promise<Success<TradingOperations["get_order"]>> {
    return this.request(
      "GET",
      `/trade/v1/orders/${this.pathPart(order_id)}`,
      undefined,
      undefined,
      options,
      true,
      "getOrder",
    );
  }

  cancelOrder(
    order_id: string,
    options: RequestOptions = {},
  ): Promise<Success<TradingOperations["cancel_order"]>> {
    return this.request(
      "DELETE",
      `/trade/v1/orders/${this.pathPart(order_id)}`,
      undefined,
      undefined,
      options,
      true,
      "cancelOrder",
    );
  }

  getOrderByHash(
    order_hash: string,
    options: RequestOptions = {},
  ): Promise<Success<TradingOperations["get_order_by_hash"]>> {
    return this.request(
      "GET",
      `/trade/v1/orders/by-hash/${this.pathPart(order_hash)}`,
      undefined,
      undefined,
      options,
      true,
      "getOrderByHash",
    );
  }

  getOrderTrades(
    order_id: string,
    options: RequestOptions = {},
  ): Promise<Success<TradingOperations["get_order_trades"]>> {
    return this.request(
      "GET",
      `/trade/v1/orders/${this.pathPart(order_id)}/trades`,
      undefined,
      undefined,
      options,
      true,
      "getOrderTrades",
    );
  }

  cancelAllOrders(
    options: RequestOptions = {},
  ): Promise<Success<TradingOperations["cancel_all_orders"]>> {
    return this.request(
      "POST",
      `/trade/v1/orders/cancel-all`,
      undefined,
      undefined,
      options,
      true,
      "cancelAllOrders",
    );
  }

  submitBatch(
    body: BatchSubmission,
    options: RequestOptions = {},
  ): Promise<Success<TradingOperations["create_batch"]>> {
    return this.request("POST", `/trade/v1/batches`, body, undefined, options, true, "submitBatch");
  }

  getBatch(
    batch_hash: string,
    options: RequestOptions = {},
  ): Promise<Success<TradingOperations["get_batch"]>> {
    return this.request(
      "GET",
      `/trade/v1/batches/${this.pathPart(batch_hash)}`,
      undefined,
      undefined,
      options,
      true,
      "getBatch",
    );
  }

  supersedeBatch(
    batch_hash: string,
    body: BatchSupersedeSubmission,
    options: RequestOptions = {},
  ): Promise<Success<TradingOperations["supersede_batch"]>> {
    return this.request(
      "POST",
      `/trade/v1/batches/${this.pathPart(batch_hash)}/supersede`,
      body,
      undefined,
      options,
      true,
      "supersedeBatch",
    );
  }

  getBatchGroup(
    group_id: string,
    options: RequestOptions = {},
  ): Promise<Success<TradingOperations["get_batch_group"]>> {
    return this.request(
      "GET",
      `/trade/v1/batch-groups/${this.pathPart(group_id)}`,
      undefined,
      undefined,
      options,
      true,
      "getBatchGroup",
    );
  }

  listActivities(
    query: Query<TradingOperations["portfolio_activities"]> = {},
    options: RequestOptions = {},
  ): Promise<Success<TradingOperations["portfolio_activities"]>> {
    return this.request(
      "GET",
      `/trade/v1/portfolio/activities`,
      undefined,
      query,
      options,
      true,
      "listActivities",
    );
  }

  getBridgeDepositAddress(
    options: RequestOptions = {},
  ): Promise<Success<TradingOperations["portfolio_bridge_deposit_address"]>> {
    return this.request(
      "GET",
      `/trade/v1/portfolio/bridge/deposit/address`,
      undefined,
      undefined,
      options,
      true,
      "getBridgeDepositAddress",
    );
  }

  getBridgeDepositAssets(
    options: RequestOptions = {},
  ): Promise<Success<TradingOperations["portfolio_bridge_deposit_supported_assets"]>> {
    return this.request(
      "GET",
      `/trade/v1/portfolio/bridge/deposit/supported-assets`,
      undefined,
      undefined,
      options,
      true,
      "getBridgeDepositAssets",
    );
  }

  quoteBridgeDeposit(
    body: RequestBody<TradingOperations["portfolio_bridge_deposit_quote"]>,
    options: RequestOptions = {},
  ): Promise<Success<TradingOperations["portfolio_bridge_deposit_quote"]>> {
    return this.request(
      "POST",
      `/trade/v1/portfolio/bridge/deposit/quote`,
      body,
      undefined,
      options,
      true,
      "quoteBridgeDeposit",
    );
  }

  getBridgeWithdrawAssets(
    options: RequestOptions = {},
  ): Promise<Success<TradingOperations["portfolio_bridge_withdraw_supported_assets"]>> {
    return this.request(
      "GET",
      `/trade/v1/portfolio/bridge/withdraw/supported-assets`,
      undefined,
      undefined,
      options,
      true,
      "getBridgeWithdrawAssets",
    );
  }

  quoteBridgeWithdrawal(
    body: RequestBody<TradingOperations["portfolio_bridge_withdraw_quote"]>,
    options: RequestOptions = {},
  ): Promise<Success<TradingOperations["portfolio_bridge_withdraw_quote"]>> {
    return this.request(
      "POST",
      `/trade/v1/portfolio/bridge/withdraw/quote`,
      body,
      undefined,
      options,
      true,
      "quoteBridgeWithdrawal",
    );
  }

  listOpenOrders(
    body: RequestBody<TradingOperations["portfolio_open_orders_list"]>,
    options: RequestOptions = {},
  ): Promise<Success<TradingOperations["portfolio_open_orders_list"]>> {
    return this.request(
      "POST",
      `/trade/v1/portfolio/open-orders/list`,
      body,
      undefined,
      options,
      true,
      "listOpenOrders",
    );
  }

  listPositions(
    body: RequestBody<TradingOperations["portfolio_positions_list"]>,
    options: RequestOptions = {},
  ): Promise<Success<TradingOperations["portfolio_positions_list"]>> {
    return this.request(
      "POST",
      `/trade/v1/portfolio/positions/list`,
      body,
      undefined,
      options,
      true,
      "listPositions",
    );
  }

  splitPosition(
    body: RequestBody<TradingOperations["portfolio_positions_split"]>,
    options: RequestOptions = {},
  ): Promise<Success<TradingOperations["portfolio_positions_split"]>> {
    return this.request(
      "POST",
      `/trade/v1/portfolio/positions/split`,
      body,
      undefined,
      options,
      true,
      "splitPosition",
    );
  }

  mergePosition(
    body: RequestBody<TradingOperations["portfolio_positions_merge"]>,
    options: RequestOptions = {},
  ): Promise<Success<TradingOperations["portfolio_positions_merge"]>> {
    return this.request(
      "POST",
      `/trade/v1/portfolio/positions/merge`,
      body,
      undefined,
      options,
      true,
      "mergePosition",
    );
  }

  getPortfolioSummary(
    query: Query<TradingOperations["portfolio_summary"]> = {},
    options: RequestOptions = {},
  ): Promise<Success<TradingOperations["portfolio_summary"]>> {
    return this.request(
      "GET",
      `/trade/v1/portfolio/summary`,
      undefined,
      query,
      options,
      true,
      "getPortfolioSummary",
    );
  }

  listTrades(
    query: Query<TradingOperations["portfolio_trades"]> = {},
    options: RequestOptions = {},
  ): Promise<Success<TradingOperations["portfolio_trades"]>> {
    return this.request(
      "GET",
      `/trade/v1/portfolio/trades`,
      undefined,
      query,
      options,
      true,
      "listTrades",
    );
  }

  getRebates(
    options: RequestOptions = {},
  ): Promise<Success<TradingOperations["portfolio_rebates"]>> {
    return this.request(
      "GET",
      `/trade/v1/portfolio/rebates`,
      undefined,
      undefined,
      options,
      true,
      "getRebates",
    );
  }

  listClosedLpIncentives(
    query: Query<TradingOperations["closed_lp_incentives"]> = {},
    options: RequestOptions = {},
  ): Promise<Success<TradingOperations["closed_lp_incentives"]>> {
    return this.request(
      "GET",
      `/trade/v1/lp-incentives/closed`,
      undefined,
      query,
      options,
      true,
      "listClosedLpIncentives",
    );
  }

  listClosedLpIncentiveCategories(
    options: RequestOptions = {},
  ): Promise<Success<TradingOperations["closed_lp_incentive_categories"]>> {
    return this.request(
      "GET",
      `/trade/v1/lp-incentives/closed/categories`,
      undefined,
      undefined,
      options,
      true,
      "listClosedLpIncentiveCategories",
    );
  }

  getLpIncentiveEarnings(
    options: RequestOptions = {},
  ): Promise<Success<TradingOperations["lp_incentive_earnings"]>> {
    return this.request(
      "GET",
      `/trade/v1/lp-incentives/earnings`,
      undefined,
      undefined,
      options,
      true,
      "getLpIncentiveEarnings",
    );
  }

  getPnl(options: RequestOptions = {}): Promise<Success<TradingOperations["portfolio_pnl"]>> {
    return this.request(
      "GET",
      `/trade/v1/portfolio/pnl`,
      undefined,
      undefined,
      options,
      true,
      "getPnl",
    );
  }

  getPnlHistory(
    query: Query<TradingOperations["portfolio_pnl_history"]>,
    options: RequestOptions = {},
  ): Promise<Success<TradingOperations["portfolio_pnl_history"]>> {
    return this.request(
      "GET",
      `/trade/v1/portfolio/pnl/history`,
      undefined,
      query,
      options,
      true,
      "getPnlHistory",
    );
  }

  getRealizedPnl(
    query: Query<TradingOperations["portfolio_realized_pnl"]>,
    options: RequestOptions = {},
  ): Promise<Success<TradingOperations["portfolio_realized_pnl"]>> {
    return this.request(
      "GET",
      `/trade/v1/portfolio/pnl/realized`,
      undefined,
      query,
      options,
      true,
      "getRealizedPnl",
    );
  }
}

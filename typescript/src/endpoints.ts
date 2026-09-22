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
/**
 * Public discovery and trading-data client; construction does not require credentials.
 *
 * @remarks
 * Configure service URLs, request deadlines and optional credentials through ClientOptions.
 * Methods preserve full response envelopes. HTTP requests are sent once with redirects disabled;
 * server failures, transport failures and malformed success responses have distinct error classes.
 * An optional PAT personalizes eligible trading-data routes such as LP standing.
 */
export class PublicClient extends Transport {
  /**
   * Read public platform market and event counts.
   *
   * @remarks
   * Callable without credentials. These counts do not establish whether any specific market accepts
   * orders.
   *
   * @param options - Per-request abort signal, timeout override and successful-response observer.
   * @returns Current market and event counts.
   * @throws TypeError for invalid local request shapes or text inputs.
   * @throws RangeError for invalid local numeric inputs or timeout overrides.
   * @throws AgaraError for an unsuccessful HTTP response; inspect its status and validated
   * recovery.
   * @throws TransportError for failed or aborted I/O; inspect its cause for the underlying failure.
   * @throws ProtocolError when a successful response is malformed or exceeds the response-byte
   * bound.
   * @throws ResponseObserverError when a callback throws after receiving a valid success response.
   */
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

  /**
   * Read a public outcome-token orderbook snapshot.
   *
   * @remarks
   * Callable without credentials. REST levels use different units from native integer WebSocket
   * books; do not apply stream scales to REST levels.
   *
   * @param token_id - Outcome token identifier encoded as an unsigned decimal 256-bit integer.
   * @param options - Per-request abort signal, timeout override and successful-response observer.
   * @returns Depth levels in whole-unit decimal price/share numbers, with snapshot metadata.
   * @throws TypeError for invalid local request shapes or text inputs.
   * @throws RangeError for invalid local numeric inputs or timeout overrides.
   * @throws AgaraError for an unsuccessful HTTP response; inspect its status and validated
   * recovery.
   * @throws TransportError for failed or aborted I/O; inspect its cause for the underlying failure.
   * @throws ProtocolError when a successful response is malformed or exceeds the response-byte
   * bound.
   * @throws ResponseObserverError when a callback throws after receiving a valid success response.
   */
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

  /**
   * Read active/upcoming LP opportunities and optional personal standing.
   *
   * @remarks
   * Callable without credentials. Anonymous callers can read opportunities. A PAT with
   * portfolio:read adds the caller standing; projections are not guaranteed credited rewards.
   *
   * @param query - Optional category/search and sort field/direction; sort_order requires sort_by
   * and otherwise defaults to descending when sorting.
   * @param options - Per-request abort signal, timeout override and successful-response observer.
   * @returns Current reward epoch, incentive terms and per-market scores/projected payouts.
   * @throws TypeError for invalid local request shapes or text inputs.
   * @throws RangeError for invalid local numeric inputs or timeout overrides.
   * @throws AgaraError for an unsuccessful HTTP response; inspect its status and validated
   * recovery.
   * @throws TransportError for failed or aborted I/O; inspect its cause for the underlying failure.
   * @throws ProtocolError when a successful response is malformed or exceeds the response-byte
   * bound.
   * @throws ResponseObserverError when a callback throws after receiving a valid success response.
   */
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

  /**
   * List root categories containing current LP-incentive opportunities.
   *
   * @remarks
   * Callable without credentials. Use these slugs to filter listLpIncentives; categories may change
   * as opportunities open and close.
   *
   * @param options - Per-request abort signal, timeout override and successful-response observer.
   * @returns Category slugs, labels and matching-market counts.
   * @throws TypeError for invalid local request shapes or text inputs.
   * @throws RangeError for invalid local numeric inputs or timeout overrides.
   * @throws AgaraError for an unsuccessful HTTP response; inspect its status and validated
   * recovery.
   * @throws TransportError for failed or aborted I/O; inspect its cause for the underlying failure.
   * @throws ProtocolError when a successful response is malformed or exceeds the response-byte
   * bound.
   * @throws ResponseObserverError when a callback throws after receiving a valid success response.
   */
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

  /**
   * Read one venue-local calendar day and its trading sessions.
   *
   * @remarks
   * Callable without credentials. A missing covered day is unavailable data, not evidence that the
   * venue is closed.
   *
   * @param mic - Case-insensitive ISO 10383 venue MIC, such as XNAS; discover supported codes with
   * listCalendars.
   * @param date - Real calendar date in YYYY-MM-DD form, interpreted in the venue timezone.
   * @param options - Per-request abort signal, timeout override and successful-response observer.
   * @returns Calendar date, trading/closed state and any session segments.
   * @throws TypeError for invalid local request shapes or text inputs.
   * @throws RangeError for invalid local numeric inputs or timeout overrides.
   * @throws AgaraError for an unsuccessful HTTP response; inspect its status and validated
   * recovery.
   * @throws TransportError for failed or aborted I/O; inspect its cause for the underlying failure.
   * @throws ProtocolError when a successful response is malformed or exceeds the response-byte
   * bound.
   * @throws ResponseObserverError when a callback throws after receiving a valid success response.
   */
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

  /**
   * Read an inclusive range of venue-local trading days.
   *
   * @remarks
   * Callable without credentials. Use real dates in the venue timezone. Missing coverage must not
   * be inferred to mean a closed session.
   *
   * @param mic - Case-insensitive ISO 10383 venue MIC, such as XNAS; discover supported codes with
   * listCalendars.
   * @param query - Inclusive YYYY-MM-DD from/to dates spanning at most 366 calendar days, with to
   * on or after from.
   * @param options - Per-request abort signal, timeout override and successful-response observer.
   * @returns Trading-day records and session segments for the requested covered range.
   * @throws TypeError for invalid local request shapes or text inputs.
   * @throws RangeError for invalid local numeric inputs or timeout overrides.
   * @throws AgaraError for an unsuccessful HTTP response; inspect its status and validated
   * recovery.
   * @throws TransportError for failed or aborted I/O; inspect its cause for the underlying failure.
   * @throws ProtocolError when a successful response is malformed or exceeds the response-byte
   * bound.
   * @throws ResponseObserverError when a callback throws after receiving a valid success response.
   */
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

  /**
   * Find the next covered trading session from a venue-local date.
   *
   * @remarks
   * Callable without credentials. No returned session can mean the requested future range is not
   * covered; it does not establish indefinite closure.
   *
   * @param mic - Case-insensitive ISO 10383 venue MIC, such as XNAS; discover supported codes with
   * listCalendars.
   * @param query - Inclusive venue-local starting date as YYYY-MM-DD.
   * @param options - Per-request abort signal, timeout override and successful-response observer.
   * @returns The next session when available, including its trading day and boundaries.
   * @throws TypeError for invalid local request shapes or text inputs.
   * @throws RangeError for invalid local numeric inputs or timeout overrides.
   * @throws AgaraError for an unsuccessful HTTP response; inspect its status and validated
   * recovery.
   * @throws TransportError for failed or aborted I/O; inspect its cause for the underlying failure.
   * @throws ProtocolError when a successful response is malformed or exceeds the response-byte
   * bound.
   * @throws ResponseObserverError when a callback throws after receiving a valid success response.
   */
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

  /**
   * List registered trading venues and their regular session templates.
   *
   * @remarks
   * Callable without credentials. Use each returned MIC and timezone for date-specific
   * trading-day/session lookups; regular templates do not describe holiday exceptions.
   *
   * @param options - Per-request abort signal, timeout override and successful-response observer.
   * @returns Venue MICs, names, timezones, currencies, active flags and regular session segments.
   * @throws TypeError for invalid local request shapes or text inputs.
   * @throws RangeError for invalid local numeric inputs or timeout overrides.
   * @throws AgaraError for an unsuccessful HTTP response; inspect its status and validated
   * recovery.
   * @throws TransportError for failed or aborted I/O; inspect its cause for the underlying failure.
   * @throws ProtocolError when a successful response is malformed or exceeds the response-byte
   * bound.
   * @throws ResponseObserverError when a callback throws after receiving a valid success response.
   */
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

  /**
   * Read metadata for a supported trading venue.
   *
   * @remarks
   * Callable without credentials. Query the trading-day/session methods for date-specific market
   * hours and holiday exceptions.
   *
   * @param mic - Case-insensitive ISO 10383 venue MIC, such as XNAS; discover supported codes with
   * listCalendars.
   * @param options - Per-request abort signal, timeout override and successful-response observer.
   * @returns Venue identity, timezone, currency, active flag and regular session segments.
   * @throws TypeError for invalid local request shapes or text inputs.
   * @throws RangeError for invalid local numeric inputs or timeout overrides.
   * @throws AgaraError for an unsuccessful HTTP response; inspect its status and validated
   * recovery.
   * @throws TransportError for failed or aborted I/O; inspect its cause for the underlying failure.
   * @throws ProtocolError when a successful response is malformed or exceeds the response-byte
   * bound.
   * @throws ResponseObserverError when a callback throws after receiving a valid success response.
   */
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

  /**
   * Read a catalogue category and its event-family metadata.
   *
   * @remarks
   * Callable without credentials. Use the canonical returned category identity when composing
   * subsequent discovery filters.
   *
   * @param slug - Catalogue category slug.
   * @param options - Per-request abort signal, timeout override and successful-response observer.
   * @returns Category identity and available event-type information.
   * @throws TypeError for invalid local request shapes or text inputs.
   * @throws RangeError for invalid local numeric inputs or timeout overrides.
   * @throws AgaraError for an unsuccessful HTTP response; inspect its status and validated
   * recovery.
   * @throws TransportError for failed or aborted I/O; inspect its cause for the underlying failure.
   * @throws ProtocolError when a successful response is malformed or exceeds the response-byte
   * bound.
   * @throws ResponseObserverError when a callback throws after receiving a valid success response.
   */
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

  /**
   * Resolve an event slug to its canonical catalogue identity and URL.
   *
   * @remarks
   * Callable without credentials. A child event may resolve to a group root. The URL may be null,
   * so preserve the canonical slug independently.
   *
   * @param slug - Event slug, including a child slug that may resolve to its group root.
   * @param options - Per-request abort signal, timeout override and successful-response observer.
   * @returns Canonical event slug and an origin-relative URL when one is available.
   * @throws TypeError for invalid local request shapes or text inputs.
   * @throws RangeError for invalid local numeric inputs or timeout overrides.
   * @throws AgaraError for an unsuccessful HTTP response; inspect its status and validated
   * recovery.
   * @throws TransportError for failed or aborted I/O; inspect its cause for the underlying failure.
   * @throws ProtocolError when a successful response is malformed or exceeds the response-byte
   * bound.
   * @throws ResponseObserverError when a callback throws after receiving a valid success response.
   */
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

  /**
   * Read event details and the associated market groups.
   *
   * @remarks
   * Callable without credentials. A child slug resolves to its group root. Discovery state and
   * configured markets should be checked before trading.
   *
   * @param slug - Event slug, including any child in the requested event group.
   * @param options - Per-request abort signal, timeout override and successful-response observer.
   * @returns The event-type-specific catalogue response, preserving display, lifecycle and market
   * metadata.
   * @throws TypeError for invalid local request shapes or text inputs.
   * @throws RangeError for invalid local numeric inputs or timeout overrides.
   * @throws AgaraError for an unsuccessful HTTP response; inspect its status and validated
   * recovery.
   * @throws TransportError for failed or aborted I/O; inspect its cause for the underlying failure.
   * @throws ProtocolError when a successful response is malformed or exceeds the response-byte
   * bound.
   * @throws ResponseObserverError when a callback throws after receiving a valid success response.
   */
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

  /**
   * Read one filtered catalogue event page.
   *
   * @remarks
   * Callable without credentials. Keep category/root and other filters stable across cursor pages.
   * An upcoming market in discovery may still be PROVISIONED and unable to accept orders.
   *
   * @param query - Category/root, event bucket, source, named filter, lifecycle/sort filters,
   * optional market enrichment, limit (default 20, maximum 100) and opaque cursor.
   * @param options - Per-request abort signal, timeout override and successful-response observer.
   * @returns Event listing items, optional market enrichment and opaque pagination.
   * @throws TypeError for invalid local request shapes or text inputs.
   * @throws RangeError for invalid local numeric inputs or timeout overrides.
   * @throws AgaraError for an unsuccessful HTTP response; inspect its status and validated
   * recovery.
   * @throws TransportError for failed or aborted I/O; inspect its cause for the underlying failure.
   * @throws ProtocolError when a successful response is malformed or exceeds the response-byte
   * bound.
   * @throws ResponseObserverError when a callback throws after receiving a valid success response.
   */
  listEvents(
    query: Query<CatalogueOperations["getApiV1Events"]> = {},
    options: RequestOptions = {},
  ): Promise<Success<CatalogueOperations["getApiV1Events"]>> {
    return this.request("GET", `/api/v1/events`, undefined, query, options, false, "listEvents");
  }

  /**
   * Read a market and its outcome identities and trading configuration.
   *
   * @remarks
   * Callable without credentials. Use outcome token IDs for trading and the returned scales for
   * native stream values; the market UUID and condition identifier serve different purposes.
   *
   * @param id - Catalogue market UUID returned by market/event discovery.
   * @param options - Per-request abort signal, timeout override and successful-response observer.
   * @returns Market lifecycle, outcomes, engine scales/limits and resolution metadata where
   * available.
   * @throws TypeError for invalid local request shapes or text inputs.
   * @throws RangeError for invalid local numeric inputs or timeout overrides.
   * @throws AgaraError for an unsuccessful HTTP response; inspect its status and validated
   * recovery.
   * @throws TransportError for failed or aborted I/O; inspect its cause for the underlying failure.
   * @throws ProtocolError when a successful response is malformed or exceeds the response-byte
   * bound.
   * @throws ResponseObserverError when a callback throws after receiving a valid success response.
   */
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

  /**
   * Read one filtered AGARA market page.
   *
   * @remarks
   * Callable without credentials. Keep filters stable when following next_cursor; one page is not a
   * complete catalogue.
   *
   * @param query - Required AGARA source plus optional event_slug, lifecycle state, cursor and
   * limit (default 64, maximum 256).
   * @param options - Per-request abort signal, timeout override and successful-response observer.
   * @returns Market listing items, event references and opaque pagination.
   * @throws TypeError for invalid local request shapes or text inputs.
   * @throws RangeError for invalid local numeric inputs or timeout overrides.
   * @throws AgaraError for an unsuccessful HTTP response; inspect its status and validated
   * recovery.
   * @throws TransportError for failed or aborted I/O; inspect its cause for the underlying failure.
   * @throws ProtocolError when a successful response is malformed or exceeds the response-byte
   * bound.
   * @throws ResponseObserverError when a callback throws after receiving a valid success response.
   */
  listMarkets(
    query: Query<CatalogueOperations["getApiV1Markets"]>,
    options: RequestOptions = {},
  ): Promise<Success<CatalogueOperations["getApiV1Markets"]>> {
    return this.request("GET", `/api/v1/markets`, undefined, query, options, false, "listMarkets");
  }

  /**
   * Read a provider price pinned to a past Unix second.
   *
   * @remarks
   * Callable without credentials. `symbol` is an Agara security symbol such as `BTC-USD`, listed by
   * `listSecurities`. `provider` is a data-provider slug, lowercase and hyphenated, such as
   * `pyth-pro`; an uppercase value like `PYTH` is rejected with 422. No method here returns the
   * slug, so take it from configuration. Both differ from the provider symbols the price streams
   * take, such as `Crypto.BTC/USD`. The requested second and the actual observation time are
   * distinct.
   *
   * @param query - Agara security symbol (see `listSecurities`), a provider slug matching
   * `^[a-z0-9-]{1,32}$` such as `pyth-pro`, and the requested at timestamp in Unix seconds.
   * @param options - Per-request abort signal, timeout override and successful-response observer.
   * @returns The available price observation and provider timing metadata.
   * @throws TypeError for invalid local request shapes or text inputs.
   * @throws RangeError for invalid local numeric inputs or timeout overrides.
   * @throws AgaraError for an unsuccessful HTTP response; inspect its status and validated
   * recovery.
   * @throws TransportError for failed or aborted I/O; inspect its cause for the underlying failure.
   * @throws ProtocolError when a successful response is malformed or exceeds the response-byte
   * bound.
   * @throws ResponseObserverError when a callback throws after receiving a valid success response.
   */
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

  /**
   * Read provider price ticks over an inclusive historical interval.
   *
   * @remarks
   * Callable without credentials. Takes the same identifiers as `getPricePoint`: an Agara security
   * symbol such as `BTC-USD` and a lowercase provider slug such as `pyth-pro`, not a provider
   * symbol like `Crypto.BTC/USD`. Preserve exact provider price representations; missing
   * observations do not imply a zero price.
   *
   * @param query - Agara security symbol, a provider slug matching `^[a-z0-9-]{1,32}$` such as
   * `pyth-pro`, and inclusive from/to Unix seconds over a window of at most 24 hours.
   * @param options - Per-request abort signal, timeout override and successful-response observer.
   * @returns Price observations and timestamps for the requested history window.
   * @throws TypeError for invalid local request shapes or text inputs.
   * @throws RangeError for invalid local numeric inputs or timeout overrides.
   * @throws AgaraError for an unsuccessful HTTP response; inspect its status and validated
   * recovery.
   * @throws TransportError for failed or aborted I/O; inspect its cause for the underlying failure.
   * @throws ProtocolError when a successful response is malformed or exceeds the response-byte
   * bound.
   * @throws ResponseObserverError when a callback throws after receiving a valid success response.
   */
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

  /**
   * Read sampled price history for a market outcome token.
   *
   * @remarks
   * Callable without credentials. This is outcome-token market history, distinct from the provider
   * security feed used to resolve a market.
   *
   * @param query - Decimal outcome token ID; optional range (1h, 6h, 1d, 7d, 30d or all; default
   * 1d) and points (2 through 1000; default 400).
   * @param options - Per-request abort signal, timeout override and successful-response observer.
   * @returns Outcome price/probability observations with timestamps.
   * @throws TypeError for invalid local request shapes or text inputs.
   * @throws RangeError for invalid local numeric inputs or timeout overrides.
   * @throws AgaraError for an unsuccessful HTTP response; inspect its status and validated
   * recovery.
   * @throws TransportError for failed or aborted I/O; inspect its cause for the underlying failure.
   * @throws ProtocolError when a successful response is malformed or exceeds the response-byte
   * bound.
   * @throws ResponseObserverError when a callback throws after receiving a valid success response.
   */
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

  /**
   * Search catalogue events, markets and categories.
   *
   * @remarks
   * Callable without credentials. Search returns bounded results rather than a cursor-walkable full
   * catalogue.
   *
   * @param query - Search text of 2 through 128 UTF-16 code units, optional AGARA source and
   * per-family limit (default 10, maximum 25).
   * @param options - Per-request abort signal, timeout override and successful-response observer.
   * @returns Matching result families with relevance and canonical identities.
   * @throws TypeError for invalid local request shapes or text inputs.
   * @throws RangeError for invalid local numeric inputs or timeout overrides.
   * @throws AgaraError for an unsuccessful HTTP response; inspect its status and validated
   * recovery.
   * @throws TransportError for failed or aborted I/O; inspect its cause for the underlying failure.
   * @throws ProtocolError when a successful response is malformed or exceeds the response-byte
   * bound.
   * @throws ResponseObserverError when a callback throws after receiving a valid success response.
   */
  search(
    query: Query<CatalogueOperations["getApiV1Search"]>,
    options: RequestOptions = {},
  ): Promise<Success<CatalogueOperations["getApiV1Search"]>> {
    return this.request("GET", `/api/v1/search`, undefined, query, options, false, "search");
  }

  /**
   * List canonical securities and their listing venues.
   *
   * @remarks
   * Callable without credentials. Use these canonical security symbols for price lookup; obtain the
   * configured provider separately from market discovery metadata.
   *
   * @param options - Per-request abort signal, timeout override and successful-response observer.
   * @returns Security symbols, labels, venue MICs, asset classes and active flags.
   * @throws TypeError for invalid local request shapes or text inputs.
   * @throws RangeError for invalid local numeric inputs or timeout overrides.
   * @throws AgaraError for an unsuccessful HTTP response; inspect its status and validated
   * recovery.
   * @throws TransportError for failed or aborted I/O; inspect its cause for the underlying failure.
   * @throws ProtocolError when a successful response is malformed or exceeds the response-byte
   * bound.
   * @throws ResponseObserverError when a callback throws after receiving a valid success response.
   */
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

  /**
   * Read one canonical security and its listing-venue metadata.
   *
   * @remarks
   * Callable without credentials. Use the venue MIC for calendar lookups and the canonical symbol
   * for price requests; obtain the configured price provider separately from market discovery
   * metadata.
   *
   * @param symbol - Case-insensitive canonical security symbol, such as BTC-USD; discover symbols
   * with listSecurities.
   * @param options - Per-request abort signal, timeout override and successful-response observer.
   * @returns The security symbol, label, venue MIC, asset class and active flag.
   * @throws TypeError for invalid local request shapes or text inputs.
   * @throws RangeError for invalid local numeric inputs or timeout overrides.
   * @throws AgaraError for an unsuccessful HTTP response; inspect its status and validated
   * recovery.
   * @throws TransportError for failed or aborted I/O; inspect its cause for the underlying failure.
   * @throws ProtocolError when a successful response is malformed or exceeds the response-byte
   * bound.
   * @throws ResponseObserverError when a callback throws after receiving a valid success response.
   */
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
/**
 * Personal-access-token client for order, portfolio and account-batch operations.
 *
 * @remarks
 * Each method documents its required PAT scope. Acceptance of an asynchronous operation is not
 * proof of a fill or settlement. Reconcile uncertain submissions using their existing identities;
 * this client never automatically replays HTTP mutations. AgaraClient adds polling and pagination.
 */
export class TraderClient extends PublicClient {
  /**
   * Configure an authenticated client without sending a request or validating the token remotely.
   *
   * @param options - Required PAT plus optional URLs, fetch transport, deadlines and response
   * observers.
   * @throws TypeError for invalid URL or token text.
   * @throws RangeError for invalid default timeout or byte limit.
   */
  constructor(
    options: ClientOptions & {
      /** Personal access token with each requested operation's scopes. */ token: string;
    },
  ) {
    super(options);
  }
  /**
   * Submit a LIMIT or MARKET order for asynchronous execution.
   *
   * @remarks
   * Requires a personal access token with scope `orders:place`. Acceptance is not a fill. Inspect
   * the order is_terminal flag and fill history separately. LIMIT orders use shares; MARKET BUY
   * uses collateral and MARKET SELL uses shares. Submission is sent once and ambiguous outcomes
   * require reconciliation.
   *
   * @param body - Order side, time-in-force and exact integer micro amounts; one whole unit is
   * 1000000 micro units.
   * @param options - Per-request abort signal, timeout override and successful-response observer.
   * @returns An acknowledgement containing the internal order UUID and pending operation.
   * @throws TypeError for invalid local request shapes or text inputs.
   * @throws RangeError for invalid local numeric inputs or timeout overrides.
   * @throws AgaraError for an unsuccessful HTTP response; inspect its status and validated
   * recovery.
   * @throws TransportError for failed or aborted I/O; a sent mutation may still complete.
   * @throws ProtocolError when a successful response is malformed or exceeds the response-byte
   * bound.
   * @throws ResponseObserverError when a callback throws after receiving a valid success response.
   */
  placeOrder(
    body: OrderRequest,
    options: RequestOptions = {},
  ): Promise<Success<TradingOperations["create_order"]>> {
    return this.request("POST", `/trade/v1/orders`, body, undefined, options, true, "placeOrder");
  }

  /**
   * Submit one pre-signed AGARA LIMIT order.
   *
   * @remarks
   * Requires a personal access token with scope `orders:place_signed`. The signature and envelope
   * must describe the same economics. A successful submission returns an order_id; read it back
   * with getOrder. An ambiguous submission cannot be reconciled by digest through this client, so
   * do not replay it blindly.
   *
   * @param body - Signed order envelope, exact micro price/shares and matching EIP-712
   * hash/signature fields.
   * @param options - Per-request abort signal, timeout override and successful-response observer.
   * @returns An asynchronous order acknowledgement; no matching or settlement is guaranteed.
   * @throws TypeError for invalid local request shapes or text inputs.
   * @throws RangeError for invalid local numeric inputs or timeout overrides.
   * @throws AgaraError for an unsuccessful HTTP response; inspect its status and validated
   * recovery.
   * @throws TransportError for failed or aborted I/O; a sent mutation may still complete.
   * @throws ProtocolError when a successful response is malformed or exceeds the response-byte
   * bound.
   * @throws ResponseObserverError when a callback throws after receiving a valid success response.
   */
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

  /**
   * Submit a batch of independently signed LIMIT orders.
   *
   * @remarks
   * Requires a personal access token with scope `orders:place_signed`. Each result belongs to its
   * input index; inspect all entries even when the HTTP response is successful. Accepted entries
   * still require order-status and fill reconciliation.
   *
   * @param body - One through 32 signed order envelopes in input order.
   * @param options - Per-request abort signal, timeout override and successful-response observer.
   * @returns Indexed accepted/rejected results, including canonical failure details for rejected
   * entries.
   * @throws TypeError for invalid local request shapes or text inputs.
   * @throws RangeError for invalid local numeric inputs or timeout overrides.
   * @throws AgaraError for an unsuccessful HTTP response; inspect its status and validated
   * recovery.
   * @throws TransportError for failed or aborted I/O; a sent mutation may still complete.
   * @throws ProtocolError when a successful response is malformed or exceeds the response-byte
   * bound.
   * @throws ResponseObserverError when a callback throws after receiving a valid success response.
   */
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

  /**
   * Read one page of your order history.
   *
   * @remarks
   * Requires a personal access token with scope `orders:read`. This is one page, not a complete
   * history. Keep the cursor opaque and use orderPages for a bounded full walk.
   *
   * @param body - Page size and optional cursor returned by the previous response.
   * @param options - Per-request abort signal, timeout override and successful-response observer.
   * @returns Order rows, market metadata, observation time and an opaque next-page cursor.
   * @throws TypeError for invalid local request shapes or text inputs.
   * @throws RangeError for invalid local numeric inputs or timeout overrides.
   * @throws AgaraError for an unsuccessful HTTP response; inspect its status and validated
   * recovery.
   * @throws TransportError for failed or aborted I/O; inspect its cause for the underlying failure.
   * @throws ProtocolError when a successful response is malformed or exceeds the response-byte
   * bound.
   * @throws ResponseObserverError when a callback throws after receiving a valid success response.
   */
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

  /**
   * Read an order by its internal UUID.
   *
   * @remarks
   * Requires a personal access token with scope `orders:read`. Only order.is_terminal establishes
   * order completion; a MATCHED label alone does not prove completion or chain settlement.
   *
   * @param order_id - Internal order UUID returned by order acceptance.
   * @param options - Per-request abort signal, timeout override and successful-response observer.
   * @returns The current order record and market metadata.
   * @throws TypeError for invalid local request shapes or text inputs.
   * @throws RangeError for invalid local numeric inputs or timeout overrides.
   * @throws AgaraError for an unsuccessful HTTP response; inspect its status and validated
   * recovery.
   * @throws TransportError for failed or aborted I/O; inspect its cause for the underlying failure.
   * @throws ProtocolError when a successful response is malformed or exceeds the response-byte
   * bound.
   * @throws ResponseObserverError when a callback throws after receiving a valid success response.
   */
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

  /**
   * Request asynchronous cancellation of one order.
   *
   * @remarks
   * Requires a personal access token with scope `orders:cancel`. The acknowledgement is not proof
   * that all remaining shares were cancelled. Observe the order is_terminal flag and any
   * intervening fills.
   *
   * @param order_id - Internal order UUID to cancel.
   * @param options - Per-request abort signal, timeout override and successful-response observer.
   * @returns A cancellation acknowledgement with the order UUID and pending operation.
   * @throws TypeError for invalid local request shapes or text inputs.
   * @throws RangeError for invalid local numeric inputs or timeout overrides.
   * @throws AgaraError for an unsuccessful HTTP response; inspect its status and validated
   * recovery.
   * @throws TransportError for failed or aborted I/O; a sent mutation may still complete.
   * @throws ProtocolError when a successful response is malformed or exceeds the response-byte
   * bound.
   * @throws ResponseObserverError when a callback throws after receiving a valid success response.
   */
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

  /**
   * Read fill legs associated with one order.
   *
   * @remarks
   * Requires a personal access token with scope `orders:read`. Read fill and settlement state
   * separately from order acceptance and terminality; amounts retain their documented micro units.
   *
   * @param order_id - Internal order UUID whose fills should be read.
   * @param options - Per-request abort signal, timeout override and successful-response observer.
   * @returns Fill records and observation time, retaining execution and transaction metadata.
   * @throws TypeError for invalid local request shapes or text inputs.
   * @throws RangeError for invalid local numeric inputs or timeout overrides.
   * @throws AgaraError for an unsuccessful HTTP response; inspect its status and validated
   * recovery.
   * @throws TransportError for failed or aborted I/O; inspect its cause for the underlying failure.
   * @throws ProtocolError when a successful response is malformed or exceeds the response-byte
   * bound.
   * @throws ResponseObserverError when a callback throws after receiving a valid success response.
   */
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

  /**
   * Schedule cancellation of your current nonterminal orders.
   *
   * @remarks
   * Requires a personal access token with scope `orders:cancel_all`. Matching and fills may race
   * cancellation. Re-read open orders and fills before treating inventory or collateral as
   * released.
   *
   * @param options - Per-request abort signal, timeout override and successful-response observer.
   * @returns An asynchronous cancel-all acknowledgement.
   * @throws TypeError for invalid local request shapes or text inputs.
   * @throws RangeError for invalid local numeric inputs or timeout overrides.
   * @throws AgaraError for an unsuccessful HTTP response; inspect its status and validated
   * recovery.
   * @throws TransportError for failed or aborted I/O; a sent mutation may still complete.
   * @throws ProtocolError when a successful response is malformed or exceeds the response-byte
   * bound.
   * @throws ResponseObserverError when a callback throws after receiving a valid success response.
   */
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

  /**
   * Submit a pre-signed account batch of position or withdrawal operations.
   *
   * @remarks
   * Requires a personal access token with scope `batches:submit`. Acceptance does not establish
   * chain execution. Reconcile the returned digest using getBatch or waitForBatch; do not blindly
   * replay an uncertain mutation.
   *
   * @param body - Signed SPLIT, MERGE or WITHDRAW operations, account sequence, Unix-second
   * deadline and optional healing digest.
   * @param options - Per-request abort signal, timeout override and successful-response observer.
   * @returns The accepted batch digest, which identifies the operation for subsequent status reads.
   * @throws TypeError for invalid local request shapes or text inputs.
   * @throws RangeError for invalid local numeric inputs or timeout overrides.
   * @throws AgaraError for an unsuccessful HTTP response; inspect its status and validated
   * recovery.
   * @throws TransportError for failed or aborted I/O; a sent mutation may still complete.
   * @throws ProtocolError when a successful response is malformed or exceeds the response-byte
   * bound.
   * @throws ResponseObserverError when a callback throws after receiving a valid success response.
   */
  submitBatch(
    body: BatchSubmission,
    options: RequestOptions = {},
  ): Promise<Success<TradingOperations["create_batch"]>> {
    return this.request("POST", `/trade/v1/batches`, body, undefined, options, true, "submitBatch");
  }

  /**
   * Read the lifecycle and reconciliation state of an account batch.
   *
   * @remarks
   * Requires a personal access token with scope `batches:submit`. FAILED with a null unwound_at
   * still requires reconciliation. Inspect failure details for FAILED_DIVERGENT rather than
   * treating a terminal label as success.
   *
   * @param batch_hash - Existing EIP-712 account-batch digest, as 0x-prefixed 32-byte hex.
   * @param options - Per-request abort signal, timeout override and successful-response observer.
   * @returns Batch status, origin, execution/unwind timestamps and public failure evidence.
   * @throws TypeError for invalid local request shapes or text inputs.
   * @throws RangeError for invalid local numeric inputs or timeout overrides.
   * @throws AgaraError for an unsuccessful HTTP response; inspect its status and validated
   * recovery.
   * @throws TransportError for failed or aborted I/O; inspect its cause for the underlying failure.
   * @throws ProtocolError when a successful response is malformed or exceeds the response-byte
   * bound.
   * @throws ResponseObserverError when a callback throws after receiving a valid success response.
   */
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

  /**
   * Request replacement of a still-pending account batch.
   *
   * @remarks
   * Requires a personal access token with scope `batches:submit`. A successful HTTP response may
   * contain a refusal. Branch on the returned result and reconcile the existing digest; this
   * mutation is not automatically retried.
   *
   * @param batch_hash - Digest of the pending batch to replace, as 0x-prefixed 32-byte hex.
   * @param body - Replacement operations, Unix-second deadline and signature bound to the inherited
   * sequence.
   * @param options - Per-request abort signal, timeout override and successful-response observer.
   * @returns Either the accepted successor or a refusal carrying the current batch state.
   * @throws TypeError for invalid local request shapes or text inputs.
   * @throws RangeError for invalid local numeric inputs or timeout overrides.
   * @throws AgaraError for an unsuccessful HTTP response; inspect its status and validated
   * recovery.
   * @throws TransportError for failed or aborted I/O; a sent mutation may still complete.
   * @throws ProtocolError when a successful response is malformed or exceeds the response-byte
   * bound.
   * @throws ResponseObserverError when a callback throws after receiving a valid success response.
   */
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

  /**
   * Read an existing account-batch group and its chunk attempts.
   *
   * @remarks
   * Requires a personal access token with scope `batches:submit`. A group read does not create
   * merge-all work. Retain individual chunk states and failures when deciding how to reconcile.
   *
   * @param group_id - UUID of the existing batch group.
   * @param options - Per-request abort signal, timeout override and successful-response observer.
   * @returns Group metadata, completed_at and per-chunk batch state.
   * @throws TypeError for invalid local request shapes or text inputs.
   * @throws RangeError for invalid local numeric inputs or timeout overrides.
   * @throws AgaraError for an unsuccessful HTTP response; inspect its status and validated
   * recovery.
   * @throws TransportError for failed or aborted I/O; inspect its cause for the underlying failure.
   * @throws ProtocolError when a successful response is malformed or exceeds the response-byte
   * bound.
   * @throws ResponseObserverError when a callback throws after receiving a valid success response.
   */
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

  /**
   * Read one page of account activity.
   *
   * @remarks
   * Requires a personal access token with scope `portfolio:read`. Activity includes orders, splits,
   * merges, redemptions, deposits, withdrawals and LP payouts. Use the realized-PnL report for
   * accounting attribution.
   *
   * @param query - Page size from 1 to 500 (server default 50) and optional opaque cursor.
   * @param options - Per-request abort signal, timeout override and successful-response observer.
   * @returns Activity variants with market, condition and event metadata plus an opaque cursor.
   * @throws TypeError for invalid local request shapes or text inputs.
   * @throws RangeError for invalid local numeric inputs or timeout overrides.
   * @throws AgaraError for an unsuccessful HTTP response; inspect its status and validated
   * recovery.
   * @throws TransportError for failed or aborted I/O; inspect its cause for the underlying failure.
   * @throws ProtocolError when a successful response is malformed or exceeds the response-byte
   * bound.
   * @throws ResponseObserverError when a callback throws after receiving a valid success response.
   */
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

  /**
   * List supported destination chains and assets for bridge withdrawals.
   *
   * @remarks
   * Requires a personal access token with scope `portfolio:read`. Use the provider address family
   * and identifiers when requesting a quote; this method does not withdraw funds.
   *
   * @param options - Per-request abort signal, timeout override and successful-response observer.
   * @returns Destination asset metadata, address families, native decimals and minimum checkout
   * values.
   * @throws TypeError for invalid local request shapes or text inputs.
   * @throws RangeError for invalid local numeric inputs or timeout overrides.
   * @throws AgaraError for an unsuccessful HTTP response; inspect its status and validated
   * recovery.
   * @throws TransportError for failed or aborted I/O; inspect its cause for the underlying failure.
   * @throws ProtocolError when a successful response is malformed or exceeds the response-byte
   * bound.
   * @throws ResponseObserverError when a callback throws after receiving a valid success response.
   */
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

  /**
   * Estimate a cross-chain withdrawal without submitting it.
   *
   * @remarks
   * Requires a personal access token with scope `portfolio:read`. This is a read-only quote. It
   * does not call the browser-authenticated standalone withdrawal route.
   *
   * @param body - Destination chain/token, destination-chain recipient address and source amount as
   * an exact integer base-unit string.
   * @param options - Per-request abort signal, timeout override and successful-response observer.
   * @returns Estimated output quantity, USD amounts, timing, fees and provider quote identity.
   * @throws TypeError for invalid local request shapes or text inputs.
   * @throws RangeError for invalid local numeric inputs or timeout overrides.
   * @throws AgaraError for an unsuccessful HTTP response; inspect its status and validated
   * recovery.
   * @throws TransportError for failed or aborted I/O; inspect its cause for the underlying failure.
   * @throws ProtocolError when a successful response is malformed or exceeds the response-byte
   * bound.
   * @throws ResponseObserverError when a callback throws after receiving a valid success response.
   */
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

  /**
   * Read one page of active orders with portfolio metadata.
   *
   * @remarks
   * Requires a personal access token with scope `portfolio:read`. A single page is not the entire
   * open-order set. Use openOrderPages for a bounded walk and allow for state changes between
   * pages.
   *
   * @param body - Optional token/exchange filters, page size and previous next_cursor.
   * @param options - Per-request abort signal, timeout override and successful-response observer.
   * @returns Nonterminal order rows, market/event sidecars, observation time and opaque pagination.
   * @throws TypeError for invalid local request shapes or text inputs.
   * @throws RangeError for invalid local numeric inputs or timeout overrides.
   * @throws AgaraError for an unsuccessful HTTP response; inspect its status and validated
   * recovery.
   * @throws TransportError for failed or aborted I/O; inspect its cause for the underlying failure.
   * @throws ProtocolError when a successful response is malformed or exceeds the response-byte
   * bound.
   * @throws ResponseObserverError when a callback throws after receiving a valid success response.
   */
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

  /**
   * Read your positions together with exchange-availability diagnostics.
   *
   * @remarks
   * Requires a personal access token with scope `portfolio:read`. An unavailable exchange must not
   * be treated as empty holdings. Call assertComplete before making decisions that require all
   * requested exchanges. At least one condition identifier is required: the server answers an empty
   * array with an empty envelope, which cannot be told apart from holding nothing, so this client
   * rejects it instead.
   *
   * @param body - One or more condition identifiers, and optional exchange filters; omitted filters
   * use server defaults.
   * @param options - Per-request abort signal, timeout override and successful-response observer.
   * @returns The complete response envelope, including positions, market/event sidecars and
   * unavailable_exchanges.
   * @throws TypeError for invalid local request shapes or text inputs.
   * @throws RangeError for invalid local numeric inputs or timeout overrides.
   * @throws AgaraError for an unsuccessful HTTP response; inspect its status and validated
   * recovery.
   * @throws TransportError for failed or aborted I/O; inspect its cause for the underlying failure.
   * @throws ProtocolError when a successful response is malformed or exceeds the response-byte
   * bound.
   * @throws ResponseObserverError when a callback throws after receiving a valid success response.
   */
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

  /**
   * Request conversion of collateral into a complete outcome set.
   *
   * @remarks
   * Requires a personal access token with scope `positions:split`. For a batch_hash response,
   * reconcile getBatch or waitForBatch. Acceptance does not prove minted positions are already
   * spendable.
   *
   * @param body - Condition identifier and positive exact collateral amount in integer micro units.
   * @param options - Per-request abort signal, timeout override and successful-response observer.
   * @returns An accepted AGARA batch or a venue relayer receipt, discriminated by its response
   * fields.
   * @throws TypeError for invalid local request shapes or text inputs.
   * @throws RangeError for invalid local numeric inputs or timeout overrides.
   * @throws AgaraError for an unsuccessful HTTP response; inspect its status and validated
   * recovery.
   * @throws TransportError for failed or aborted I/O; a sent mutation may still complete.
   * @throws ProtocolError when a successful response is malformed or exceeds the response-byte
   * bound.
   * @throws ResponseObserverError when a callback throws after receiving a valid success response.
   */
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

  /**
   * Request conversion of a complete outcome set back into collateral.
   *
   * @remarks
   * Requires a personal access token with scope `positions:merge`. For a batch_hash response,
   * reconcile getBatch or waitForBatch. Acceptance does not prove collateral is already available.
   *
   * @param body - Condition identifier and positive exact quantity per outcome leg in integer micro
   * shares.
   * @param options - Per-request abort signal, timeout override and successful-response observer.
   * @returns An accepted AGARA batch or a venue relayer receipt, discriminated by its response
   * fields.
   * @throws TypeError for invalid local request shapes or text inputs.
   * @throws RangeError for invalid local numeric inputs or timeout overrides.
   * @throws AgaraError for an unsuccessful HTTP response; inspect its status and validated
   * recovery.
   * @throws TransportError for failed or aborted I/O; a sent mutation may still complete.
   * @throws ProtocolError when a successful response is malformed or exceeds the response-byte
   * bound.
   * @throws ResponseObserverError when a callback throws after receiving a valid success response.
   */
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

  /**
   * Read collateral and position valuations for the requested trading wallet.
   *
   * @remarks
   * Requires a personal access token with scope `portfolio:read`. Keep exchange collateral pools
   * separate; summary values do not authorize an order beyond current server balance checks.
   *
   * @param query - Optional AGARA exchange selection; omission uses the server trading-wallet
   * default.
   * @param options - Per-request abort signal, timeout override and successful-response observer.
   * @returns Per-exchange cash balance, free cash and portfolio valuations in documented micro
   * units.
   * @throws TypeError for invalid local request shapes or text inputs.
   * @throws RangeError for invalid local numeric inputs or timeout overrides.
   * @throws AgaraError for an unsuccessful HTTP response; inspect its status and validated
   * recovery.
   * @throws TransportError for failed or aborted I/O; inspect its cause for the underlying failure.
   * @throws ProtocolError when a successful response is malformed or exceeds the response-byte
   * bound.
   * @throws ResponseObserverError when a callback throws after receiving a valid success response.
   */
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

  /**
   * Read one page of your fill history.
   *
   * @remarks
   * Requires a personal access token with scope `portfolio:read`. Use tradePages to walk further
   * pages. Check unavailable_exchanges before treating this history as complete.
   *
   * @param query - Page size from 1 to 500 (server default 500) and optional opaque next_cursor.
   * @param options - Per-request abort signal, timeout override and successful-response observer.
   * @returns Fill legs, market/event sidecars, availability diagnostics and an opaque cursor.
   * @throws TypeError for invalid local request shapes or text inputs.
   * @throws RangeError for invalid local numeric inputs or timeout overrides.
   * @throws AgaraError for an unsuccessful HTTP response; inspect its status and validated
   * recovery.
   * @throws TransportError for failed or aborted I/O; inspect its cause for the underlying failure.
   * @throws ProtocolError when a successful response is malformed or exceeds the response-byte
   * bound.
   * @throws ResponseObserverError when a callback throws after receiving a valid success response.
   */
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

  /**
   * Read pending maker, VIP and LP incentive balances.
   *
   * @remarks
   * Requires a personal access token with scope `portfolio:read`. Pending rebates are not spendable
   * trading collateral until the server reports the corresponding credit.
   *
   * @param options - Per-request abort signal, timeout override and successful-response observer.
   * @returns Pending incentive amounts in integer micro collateral.
   * @throws TypeError for invalid local request shapes or text inputs.
   * @throws RangeError for invalid local numeric inputs or timeout overrides.
   * @throws AgaraError for an unsuccessful HTTP response; inspect its status and validated
   * recovery.
   * @throws TransportError for failed or aborted I/O; inspect its cause for the underlying failure.
   * @throws ProtocolError when a successful response is malformed or exceeds the response-byte
   * bound.
   * @throws ResponseObserverError when a callback throws after receiving a valid success response.
   */
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

  /**
   * Read one page of your historical LP reward cycles.
   *
   * @remarks
   * Requires a personal access token with scope `portfolio:read`. Use 1-based pagination rather
   * than cursor helpers. Potential payout and credited reward are separate amounts in micro
   * collateral.
   *
   * @param query - Optional category/search; page starts at 1, limit is 1 through 100 (default 20),
   * sort defaults to date descending.
   * @param options - Per-request abort signal, timeout override and successful-response observer.
   * @returns Credited-cycle market rows, applied page/limit and total matching-row count.
   * @throws TypeError for invalid local request shapes or text inputs.
   * @throws RangeError for invalid local numeric inputs or timeout overrides.
   * @throws AgaraError for an unsuccessful HTTP response; inspect its status and validated
   * recovery.
   * @throws TransportError for failed or aborted I/O; inspect its cause for the underlying failure.
   * @throws ProtocolError when a successful response is malformed or exceeds the response-byte
   * bound.
   * @throws ResponseObserverError when a callback throws after receiving a valid success response.
   */
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

  /**
   * List categories represented in your historical LP rewards.
   *
   * @remarks
   * Requires a personal access token with scope `portfolio:read`. These are caller-specific
   * historical categories, distinct from current public incentive opportunities.
   *
   * @param options - Per-request abort signal, timeout override and successful-response observer.
   * @returns Root-category slugs and labels available for filtering credited reward cycles.
   * @throws TypeError for invalid local request shapes or text inputs.
   * @throws RangeError for invalid local numeric inputs or timeout overrides.
   * @throws AgaraError for an unsuccessful HTTP response; inspect its status and validated
   * recovery.
   * @throws TransportError for failed or aborted I/O; inspect its cause for the underlying failure.
   * @throws ProtocolError when a successful response is malformed or exceeds the response-byte
   * bound.
   * @throws ResponseObserverError when a callback throws after receiving a valid success response.
   */
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

  /**
   * Read current projected and trailing LP reward totals.
   *
   * @remarks
   * Requires a personal access token with scope `portfolio:read`. The current cycle is projected
   * before minimum payout rules; trailing windows combine that projection with prior credited
   * rewards.
   *
   * @param options - Per-request abort signal, timeout override and successful-response observer.
   * @returns Current-cycle, seven-day, thirty-day and all-time earnings in micro collateral.
   * @throws TypeError for invalid local request shapes or text inputs.
   * @throws RangeError for invalid local numeric inputs or timeout overrides.
   * @throws AgaraError for an unsuccessful HTTP response; inspect its status and validated
   * recovery.
   * @throws TransportError for failed or aborted I/O; inspect its cause for the underlying failure.
   * @throws ProtocolError when a successful response is malformed or exceeds the response-byte
   * bound.
   * @throws ResponseObserverError when a callback throws after receiving a valid success response.
   */
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

  /**
   * Read exact realized-PnL attribution over a UTC calendar window.
   *
   * @remarks
   * Requires a personal access token with scope `portfolio:read`. amountScale describes fractional
   * precision, not a divisor to apply to the returned decimal USDC strings. Use week/all or
   * hour/day with 1d, 7d or 30d.
   *
   * @param query - Required granularity and window: hour/day with 1d, 7d or 30d, or week with all.
   * @param options - Per-request abort signal, timeout override and successful-response observer.
   * @returns Category totals, buckets, running totals and provenance; monetary values are decimal
   * USDC strings.
   * @throws TypeError for invalid local request shapes or text inputs.
   * @throws RangeError for invalid local numeric inputs or timeout overrides.
   * @throws AgaraError for an unsuccessful HTTP response; inspect its status and validated
   * recovery.
   * @throws TransportError for failed or aborted I/O; inspect its cause for the underlying failure.
   * @throws ProtocolError when a successful response is malformed or exceeds the response-byte
   * bound.
   * @throws ResponseObserverError when a callback throws after receiving a valid success response.
   */
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

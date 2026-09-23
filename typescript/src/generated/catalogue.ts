export interface paths {
    "/api/v1/calendars/{mic}/days/{date}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /**
         * Get a trading day
         * @description Whether the venue is open on a date. is_trading_day=false is a definitive closed answer; a 404 means no data for that date (treat as "do not act", not "closed").
         */
        get: operations["getApiV1CalendarsByMicDaysByDate"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/v1/calendars/{mic}/days": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /**
         * List trading days in a range
         * @description Sessions for [from, to] inclusive. Range may not exceed 366 days.
         */
        get: operations["getApiV1CalendarsByMicDays"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/v1/calendars/{mic}/next-session": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /**
         * Next trading session
         * @description The earliest trading day on or after `from`, within entered coverage. session=null when none exists in range.
         */
        get: operations["getApiV1CalendarsByMicNextSession"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/v1/calendars": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /**
         * List trading venues
         * @description Every registered trading venue, keyed by ISO 10383 MIC.
         */
        get: operations["getApiV1Calendars"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/v1/calendars/{mic}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /**
         * Get a venue
         * @description Metadata and entered-coverage range for a single venue.
         */
        get: operations["getApiV1CalendarsByMic"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/v1/categories/{slug}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /**
         * Category resource
         * @description Single-category lookup. Reports which event-type buckets (`game`, `multi_outcome`, `proposition`) have events under this category or any of its descendants.
         */
        get: operations["getApiV1CategoriesBySlug"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/v1/events/{slug}/canonical-url": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /**
         * Resolve event slug to canonical URL
         * @description Returns the category-rooted URL an event slug should redirect to. Child slugs in a group resolve to the group root. Archived events 404.
         */
        get: operations["getApiV1EventsBySlugCanonicalUrl"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/v1/events/{slug}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /**
         * Event detail
         * @description Returns the event + its group markets + curated "see also" relations + leaf category paths. Most event types ship every market in the group; RECURRING_PROPOSITION events serve a window: the live cycle group uncapped, up to 50 newest unsettled cycles, 49 soonest upcoming cycles, and 50 newest settled cycles - whole cycles only. A child-event slug resolves to its group root — clients should compare `slug` on the response and redirect to the canonical URL if it differs. Archived events 404.
         */
        get: operations["getApiV1EventsBySlug"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/v1/events": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /**
         * List events
         * @description `filter=sports_live` returns leaf-category sections (no pagination); any other value (or none) returns a flat keyset-paginated list. Category metadata lives at `GET /categories/:slug`.
         */
        get: operations["getApiV1Events"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/v1/markets/{id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /**
         * Get market by id
         * @description Fetch a single market by its UUID, with its `state` and full outcomes. Same shape as an element of the event-detail `markets[]`. 404 when no non-archived market with that id exists.
         */
        get: operations["getApiV1MarketsById"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/v1/markets": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /**
         * List markets
         * @description Flat keyset-paginated market listing. Markets accepting orders are returned first, then by volume.
         */
        get: operations["getApiV1Markets"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/v1/prices/point": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /**
         * Pinned price at a past second
         * @description The price for an agara security + provider as of a past instant, served from the tiered cache (memory → our tick store → provider API). Every provider fetch is written through to the shared tick store. 422 = no such feed; 503 = the second is not yet available upstream (retry).
         */
        get: operations["getApiV1PricesPoint"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/v1/prices/ticks": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /**
         * Settlement-lineage price history
         * @description Price history for an agara security + provider over [from, to], as TradingView-shim history (`{ s, t, c }`, seconds + close). Returns roughly 30 points spanning the window, served DB-first (our shared tick cache) and filled with the fewest external calls — sampled point reads for a tight window, OHLCV minute bars for a wide one. Long-cached once the range is in the immutable past; no-cache while it overlaps the live edge.
         */
        get: operations["getApiV1PricesTicks"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/v1/prices/token-history": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /**
         * Market outcome price/probability history
         * @description Last-traded-price history for one outcome token over a preset range, as TradingView-shim history (`{ s, t, c }`, seconds + probability 0..1) plus `latestTradeAt`. Step-held onto ~400 uniform points. Long-cached once the window is settled; no-cache while it overlaps the live edge.
         */
        get: operations["getApiV1PricesTokenHistory"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/v1/search": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /**
         * Search events, markets, and categories
         * @description Typo-tolerant search over currently tradable AGARA event titles/slugs and market questions/slugs, plus category labels/slugs. Event and market matches include a comparable 0–1 relevance score; indirect category matches score 0. Each event arrives with its markets and their outcome prices — the same shape the events listing returns — so a result can be rendered without a follow-up request. Markets carry their leading outcomes (label + odds) and order-acceptance status. A category-name query also fans out that category’s highest-volume tradable AGARA events.
         */
        get: operations["getApiV1Search"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/v1/securities": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /**
         * List securities
         * @description Every active security in the catalog, keyed by canonical symbol.
         */
        get: operations["getApiV1Securities"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/v1/securities/{symbol}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /**
         * Get a security
         * @description Catalog metadata for a single security, by canonical symbol.
         */
        get: operations["getApiV1SecuritiesBySymbol"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
}
export type webhooks = Record<string, never>;
export interface components {
    schemas: {
        /** @description Source-discriminated nested sub-bag scaffold for agara-native events. Empty today; lands here when the in-house CMS source ships event- level metadata that doesn't generalize to other sources. */
        AgaraCmsEventDisplay: components["schemas"]["Record_string_never"];
        AgaraCmsMarketDisplay: components["schemas"]["Record_string_never"];
        AgaraCmsOutcomeDisplay: components["schemas"]["Record_string_never"];
        /** @description AGARA matching-engine config for a single market. The engine deals in integer "price units" (`price = micro_dollars × price_scale / 1_000_000`) and integer "size units" (`size = micro_shares × size_scale / 1_000_000`). Used by maker/taker bots to quantize and clamp quotes locally — saves a round-trip per tick. */
        AgaraEngineConfig: {
            /** @description Smallest price increment, in engine price units. */
            tick_size: number;
            /** @description Number of engine price units per $1. e.g. 100 ⇒ 1 unit = $0.01. */
            price_scale: number;
            /** @description Number of engine size units per whole share. e.g. 1_000_000 ⇒ 1 unit = 1e-6 shares. */
            size_scale: number;
            /** @description Lowest accepted price in engine price units. Quotes below this are rejected. Typically equals `tick_size`, but a market can be configured with a wider dust-floor. */
            min_price: number;
            /** @description Highest accepted price in engine price units. Typically equals `price_scale - tick_size`. */
            max_price: number;
        };
        BaseMarketDisplay: {
            /** @description Market card image (distinct from event-level image_url). */
            image_url?: string;
            /** @description Operator-authored portfolio-row title (CMS default: "{event title} - {market title}"). Portfolio-surface consumption is a follow-up; absent = client default. */
            portfolio_title?: string;
            /** @description Free-form market description / rules text. Sync may ship multi-line with `\n` paragraph breaks. */
            description?: string;
            /** @description URL or free-text describing how the market settles. Distinct from the event-level resolution_source on `BaseEventDisplay`. */
            resolution_source?: string;
            /** @description URL describing the market's resolution criteria / methodology; renders as the crypto panel's "View Details" link. Absent when none exists. */
            resolution_criteria?: string;
            /** @description Operator-authored titled resolution links, shown alongside resolution_source. */
            resolution_sources?: components["schemas"]["ResolutionSourceLink"][];
            /** @description Player-prop player name (Steph Curry, Patrick Mahomes, …). */
            player_name?: string;
            /** @description Player-prop stat type (points, assists, passing yards, …). */
            stat_type?: string;
            /** @description Snapshot of liquidity at last sync. Not sorted/filtered on, so it lives in the JSONB bag rather than as a dedicated column. NUMERIC via JSONB — number, not string. */
            liquidity?: number;
            /** @description Current linked-series cadence in seconds. */
            cadence_seconds?: number;
            /** @description RECURRING_PROPOSITION: reference price snapped at the cycle's reference_at (T+0). String to avoid float precision loss; render as a formatted currency value. The bettor compares their direction bet against this number. */
            reference_price?: string;
            /** @description RECURRING_PROPOSITION: ISO 4217 currency of reference_price, stamped at provision from the asset's venue (e.g. INR for NSE, KRW for KRX, USD for crypto). `null` for an index/unitless asset (rendered as a plain number); defaults to USD when absent (legacy rows). */
            currency?: string | null;
            /** @description RECURRING_PROPOSITION: cycle window endpoints as ISO timestamps. reference_at = trading-open (T+0). observe_at = trading-close + propose trigger (T+15). Used to drive the live-cycle countdown timer. */
            reference_at?: string;
            observe_at?: string;
            /** @description price_range band member: 0-based position in the cycle's ladder (low→high), the concrete `[lower, upper)` bounds (null = an open `<`/`>=` tail), and the rendered band label. Stamped at provision; drive the bucket-ladder rows + cycle grouping. */
            band_index?: number;
            band_lower?: string | null;
            band_upper?: string | null;
            band_label?: string;
            /** @description RECURRING_PROPOSITION (crypto cycles): the Pyth Hermes price-feed id, stamped at provision when the spec's primary source is pyth. Lets the event page stream a live price from the same source the market resolves against. Absent for non-pyth (e.g. equity) cycles. */
            pyth_feed_id?: string;
            /** @description RECURRING_PROPOSITION (crypto cycles): the Pyth Benchmarks namespace symbol (e.g. `Crypto.BTC/USD`) the benchmarks TradingView history shim needs to backfill the chart. Distinct from tradingview_symbol, which is the `EXCHANGE:TICKER` for the embedded equity widget. */
            pyth_benchmark_symbol?: string;
            /** @description RECURRING_PROPOSITION (equity cycles): the TradingView `EXCHANGE:TICKER` symbol (e.g. NSE:RELIANCE), stamped at provision when the spec's primary source is tradingview. Drives the event page's embedded TradingView live price. Absent for non-tradingview cycles. */
            tradingview_symbol?: string;
            /** @description RECURRING_PROPOSITION: the Pyth Pro (Lazer) symbol (e.g. Equity.HK.2513/HKD), stamped at provision when the spec's primary source is pyth-pro. Drives the event page's live price via the api SSE relay (the feed is permissioned, so it can't be streamed from the browser directly). Absent otherwise. */
            pyth_pro_symbol?: string;
            /** @description RECURRING_PROPOSITION: the agara security symbol (the spec's asset) keying the settlement-lineage tick series. With `security_provider`, keys `GET /prices/ticks` for the settled chart. Absent when the spec has no asset or the asset has no securities-master row. */
            security_symbol?: string;
            /** @description RECURRING_PROPOSITION: the spec's primary provider slug (e.g. pyth-pro), the other half of the tick-series key. Absent when `security_symbol` is. */
            security_provider?: string;
            /** @description Esports per-game routing — extracted from market.slug via /-game(\d+)/i. null when the slug carries no -gameN suffix. Populated for every market regardless of layout (cheap slug parse); consumed only when the parent event's layout === 'esports'. Hoisted to MarketDetail.game_number — readers prefer top-level. */
            game_number?: number | null;
            /** @description True when sync's classifier tagged this market as THREE_WAY (soccer moneyline with draw leg, etc.). Hoisted to MarketDetail.is_three_way. */
            is_three_way?: boolean;
            /** @description Which leg of a 3-way moneyline this market represents. Computed at sync time by resolveThreeWayRole(display_label, event teams). Hoisted to MarketDetail.three_way_role. */
            three_way_role?: components["schemas"]["ThreeWayRole"];
            /** @description Canonical display label for the row/card title. Sync's recipe (computeMarketLabel) always populates via a five-layer fallback: 3-way leg → tennis → esports tab strip → groupItemTitle for lined/multi-outcome → question. Hoisted to MarketDetail.display_label. */
            display_label?: string;
            /** @description Source-specific extras. Present when row.source === 'AGARA'. */
            agara_cms?: components["schemas"]["AgaraCmsMarketDisplay"];
        };
        BaseOutcomeDisplay: {
            /** @description Canonical raw outcome text — storage home for MarketOutcome.label. Always populated by the sync writer; the wire shape lives in `MarketOutcome.label` after the API serializer hoists. */
            label?: string;
            /** @description Semantic kind. Set by sync's classifier. Readers treat `undefined` the same as `'unknown'` and fall back to label. */
            kind?: components["schemas"]["OutcomeKind"];
            /** @description Short pill-button label (e.g. "LAL", "Over 2.5", "Draw"). */
            short_label?: string;
            /** @description Which event-level team this outcome refers to. Resolves through event.display.team_a / team_b at render time. null for non-team outcomes. */
            team_side?: components["schemas"]["OutcomeTeamSide"] | null;
            /**
             * @description Pill side for binary card layouts: index-0 → 'left', index-1 → 'right'. null for ternary or grid layouts.
             * @enum {string|null}
             */
            side?: "left" | "right" | null;
            /** @description For kind='over'/'under' on player props, the stat threshold (the parent market's `line` value). null otherwise. */
            threshold?: number | null;
            /** @description Pre-rendered bet-slip header sentence — single source of truth. See MarketOutcome.bet_slip_title. */
            bet_slip_title?: string;
            /** @description Resolution-time label. See MarketOutcome.resolution_label. */
            resolution_label?: string;
            /** @description Source-specific extras. Present when parent market.source === 'AGARA'. */
            agara_cms?: components["schemas"]["AgaraCmsOutcomeDisplay"];
        };
        /**
         * @description Render form for one CategorySection. Three values, locked. New rendering forms ship as new enum values plus a frontend dispatcher branch — no other schema change. See spec docs/superpowers/specs/2026-04-25-event-market-tabs-design.md "Render hint catalog" for the table.
         * @enum {string}
         */
        CategoryRenderHint: "card_list" | "lined_single_card" | "multi_outcome_grid";
        CategoryResource: {
            id: string;
            slug: string;
            label: string;
            /** @description Presence of each event_type in the closure-walked subtree. The web client rolls up MULTI_OUTCOME + PROPOSITION → "Props" for display; the API contract reports the schema event_types directly so the rollup decision stays a UI concern. */
            event_types: {
                game: boolean;
                multi_outcome: boolean;
                proposition: boolean;
            };
        };
        /** @description One categorized bucket within a tab — e.g. 'Moneyline', 'Spreads'. */
        CategorySection: {
            /** @description Canonical category key — see schema doc §7 for the full registry. CMS overrides key on (event_id, tab_key, section_key). */
            key: string;
            /** @description Section header text. */
            label: string;
            /** @description Within-tab section order (ascending). */
            order: number;
            render_hint: components["schemas"]["CategoryRenderHint"];
            /** @description One entry per card. */
            groups: components["schemas"]["MarketGroup"][];
            meta?: components["schemas"]["CategorySectionMeta"];
        };
        /** @description Optional per-section extras. Sparse — populated only when a render hint actually needs config (lined / multi-outcome). card_list emits no meta by design. */
        CategorySectionMeta: {
            /**
             * @description lined_single_card only — what the line-selector pill strip displays. Default 'line' for spreads/totals/etc.; 'game_number' for esports map_winners (the pills read "1 / 2 / 3" instead of "1.5 / 2.5").
             * @enum {string}
             */
            selector_axis?: "line" | "game_number";
            /** @description lined_single_card with selector_axis='line' only — the line the frontend should default to on mount. null when no anchor exists; renderer falls back to closest-to-50¢. Ignored when axis is 'game_number' (frontend defaults to the smallest game number). */
            default_line?: number | null;
            /**
             * @description lined_single_card only — which side owns the raw stored line. Replaces the legacy "Team-A-gets-raw" implicit convention.
             * @enum {string}
             */
            line_owner_side?: "first" | "second";
            /**
             * @description multi_outcome_grid only — column-count hint. Frontend can override for viewport.
             * @enum {number}
             */
            grid_cols_hint?: 2 | 3 | 4;
        };
        /** @enum {string} */
        DisplayType: "BINARY" | "MAP_MARKET" | "MULTI_OUTCOME" | "OVER_UNDER" | "PLAYER_PROP" | "SPREAD" | "TERNARY";
        /** EdgeProblem */
        EdgeProblem: {
            type: string;
            title: string;
            status: number;
            code: string;
            detail?: string;
            recovery: components["schemas"]["Recovery"];
        };
        EventCanonicalUrlResponse: {
            /** @description Origin-relative path, e.g. "/sports/tennis/lakers-vs-celtics-2026-04-29". null when the event has no categories — caller falls back to inline render at /event/<canonical_slug>. */
            redirect_url: string | null;
            /** @description Canonical event slug, after the API resolves a child slug to its group root. Web uses this to issue an internal /event/<canonical> redirect when the event has no categories and the request slug is stale. */
            canonical_slug: string;
        };
        /** @description Event detail response. Discriminated on `event_type`. GAME variants carry `game_state` (live score + match lifecycle, populated from the agara_events.game_state column); other types omit it. */
        EventDetail: {
            /** @constant */
            event_type: "GAME";
            display: components["schemas"]["GameEventDisplay"];
            game_state: components["schemas"]["GameState"] | null;
            id: string;
            slug: string;
            title: string;
            is_accepting_orders: boolean;
            is_live: boolean;
            is_group_root: boolean;
            start_time: string | null;
            end_time: string | null;
            /** @description Group-rolled-up volume (root + child events' markets). BIGINT 1e-6 micro-USDC (CC-2/CC-4); ships as a branded string. Liquidity moved to display.liquidity. */
            volume_micro: components["schemas"]["MicroUsd"];
            /** @description UUID of the event's primary leg market. For 2-way events this IS the main market. For 3-way events (soccer moneylines, etc.) this is the lead leg sync's rollup picked deterministically — readers needing the full leg group should use the API's `main_market_group` (assembled at read time via `neg_risk_id`). null when the event has no markets. */
            main_market_id: string | null;
            /**
             * @description Event-level resolution. 'ACTIVE' is the wire spelling for null on the DB.
             * @enum {string}
             */
            resolution_status: "ACTIVE" | "PROPOSED" | "DISPUTED" | "RESOLVED" | "VOID";
            /** @description Market-aggregated event state (rolled up from child market `state`s). Optional on the wire (always set by the api) so clients predating the field stay valid. */
            state?: components["schemas"]["EventLifecycleState"];
            categories: {
                slug: string;
                label: string;
                root_slug: string;
                path: {
                    slug: string;
                    label: string;
                }[];
            }[];
            primary_category: {
                slug: string;
                label: string;
            } | null;
            markets: components["schemas"]["MarketDetail"][];
            /** @description Display-friendly sport label derived from display.sport_slug ("Basketball", "Soccer", "Esports"). Used for breadcrumbs and bet-slip unit inference. */
            sport_group?: string;
            /** @description Curated main market(s) for the event card header. For 3-way sports this carries all 3 legs (home/draw/away); for 2-way moneylines / prop-only events it's a single-market group. See MainMarketGroup. */
            main_market_group?: components["schemas"]["MainMarketGroup"];
            /** @description Up to 2 featured spread markets nearest the event's main spread line. Ordered by distance from `display.spreads_main_line`. Detail only — listing card doesn't render spreads. */
            featured_spread_markets?: components["schemas"]["MarketDetail"][];
            /** @description Single featured totals market, preferring exact match on `display.totals_main_line`. Detail only — listing card doesn't render totals. */
            featured_totals_market?: components["schemas"]["MarketDetail"];
            /** @description Pre-assembled event rendering tree: tabs → sections → groups → market_ids. Frontend hydrates each MarketGroup via market_ids.map(id => marketsById.get(id)). NULL on the DB maps to [] on the wire. See MarketTab. */
            market_tabs?: components["schemas"]["MarketTab"][];
            /** @description Counts for the resolution badges. Computed over the markets served on this payload — for RECURRING_PROPOSITION events that is the windowed subset (live window + capped upcoming/settled cycles), not all cycles the series ever ran. */
            disputed_market_count?: number;
            resolved_market_count?: number;
            /** @description Admin-curated "see also" events surfaced under the detail page. Populated from agara_event_relations. Filtered to non-archived, ordered by position, capped server-side. Empty array when the event has no curated relations. */
            related_events: components["schemas"]["EventListItem"][];
        } | {
            /** @constant */
            event_type: "MULTI_OUTCOME";
            display: components["schemas"]["MultiOutcomeEventDisplay"];
            id: string;
            slug: string;
            title: string;
            is_accepting_orders: boolean;
            is_live: boolean;
            is_group_root: boolean;
            start_time: string | null;
            end_time: string | null;
            /** @description Group-rolled-up volume (root + child events' markets). BIGINT 1e-6 micro-USDC (CC-2/CC-4); ships as a branded string. Liquidity moved to display.liquidity. */
            volume_micro: components["schemas"]["MicroUsd"];
            /** @description UUID of the event's primary leg market. For 2-way events this IS the main market. For 3-way events (soccer moneylines, etc.) this is the lead leg sync's rollup picked deterministically — readers needing the full leg group should use the API's `main_market_group` (assembled at read time via `neg_risk_id`). null when the event has no markets. */
            main_market_id: string | null;
            /**
             * @description Event-level resolution. 'ACTIVE' is the wire spelling for null on the DB.
             * @enum {string}
             */
            resolution_status: "ACTIVE" | "PROPOSED" | "DISPUTED" | "RESOLVED" | "VOID";
            /** @description Market-aggregated event state (rolled up from child market `state`s). Optional on the wire (always set by the api) so clients predating the field stay valid. */
            state?: components["schemas"]["EventLifecycleState"];
            categories: {
                slug: string;
                label: string;
                root_slug: string;
                path: {
                    slug: string;
                    label: string;
                }[];
            }[];
            primary_category: {
                slug: string;
                label: string;
            } | null;
            markets: components["schemas"]["MarketDetail"][];
            /** @description Display-friendly sport label derived from display.sport_slug ("Basketball", "Soccer", "Esports"). Used for breadcrumbs and bet-slip unit inference. */
            sport_group?: string;
            /** @description Curated main market(s) for the event card header. For 3-way sports this carries all 3 legs (home/draw/away); for 2-way moneylines / prop-only events it's a single-market group. See MainMarketGroup. */
            main_market_group?: components["schemas"]["MainMarketGroup"];
            /** @description Up to 2 featured spread markets nearest the event's main spread line. Ordered by distance from `display.spreads_main_line`. Detail only — listing card doesn't render spreads. */
            featured_spread_markets?: components["schemas"]["MarketDetail"][];
            /** @description Single featured totals market, preferring exact match on `display.totals_main_line`. Detail only — listing card doesn't render totals. */
            featured_totals_market?: components["schemas"]["MarketDetail"];
            /** @description Pre-assembled event rendering tree: tabs → sections → groups → market_ids. Frontend hydrates each MarketGroup via market_ids.map(id => marketsById.get(id)). NULL on the DB maps to [] on the wire. See MarketTab. */
            market_tabs?: components["schemas"]["MarketTab"][];
            /** @description Counts for the resolution badges. Computed over the markets served on this payload — for RECURRING_PROPOSITION events that is the windowed subset (live window + capped upcoming/settled cycles), not all cycles the series ever ran. */
            disputed_market_count?: number;
            resolved_market_count?: number;
            /** @description Admin-curated "see also" events surfaced under the detail page. Populated from agara_event_relations. Filtered to non-archived, ordered by position, capped server-side. Empty array when the event has no curated relations. */
            related_events: components["schemas"]["EventListItem"][];
        } | {
            /** @constant */
            event_type: "PROPOSITION";
            display: components["schemas"]["PropositionEventDisplay"];
            id: string;
            slug: string;
            title: string;
            is_accepting_orders: boolean;
            is_live: boolean;
            is_group_root: boolean;
            start_time: string | null;
            end_time: string | null;
            /** @description Group-rolled-up volume (root + child events' markets). BIGINT 1e-6 micro-USDC (CC-2/CC-4); ships as a branded string. Liquidity moved to display.liquidity. */
            volume_micro: components["schemas"]["MicroUsd"];
            /** @description UUID of the event's primary leg market. For 2-way events this IS the main market. For 3-way events (soccer moneylines, etc.) this is the lead leg sync's rollup picked deterministically — readers needing the full leg group should use the API's `main_market_group` (assembled at read time via `neg_risk_id`). null when the event has no markets. */
            main_market_id: string | null;
            /**
             * @description Event-level resolution. 'ACTIVE' is the wire spelling for null on the DB.
             * @enum {string}
             */
            resolution_status: "ACTIVE" | "PROPOSED" | "DISPUTED" | "RESOLVED" | "VOID";
            /** @description Market-aggregated event state (rolled up from child market `state`s). Optional on the wire (always set by the api) so clients predating the field stay valid. */
            state?: components["schemas"]["EventLifecycleState"];
            categories: {
                slug: string;
                label: string;
                root_slug: string;
                path: {
                    slug: string;
                    label: string;
                }[];
            }[];
            primary_category: {
                slug: string;
                label: string;
            } | null;
            markets: components["schemas"]["MarketDetail"][];
            /** @description Display-friendly sport label derived from display.sport_slug ("Basketball", "Soccer", "Esports"). Used for breadcrumbs and bet-slip unit inference. */
            sport_group?: string;
            /** @description Curated main market(s) for the event card header. For 3-way sports this carries all 3 legs (home/draw/away); for 2-way moneylines / prop-only events it's a single-market group. See MainMarketGroup. */
            main_market_group?: components["schemas"]["MainMarketGroup"];
            /** @description Up to 2 featured spread markets nearest the event's main spread line. Ordered by distance from `display.spreads_main_line`. Detail only — listing card doesn't render spreads. */
            featured_spread_markets?: components["schemas"]["MarketDetail"][];
            /** @description Single featured totals market, preferring exact match on `display.totals_main_line`. Detail only — listing card doesn't render totals. */
            featured_totals_market?: components["schemas"]["MarketDetail"];
            /** @description Pre-assembled event rendering tree: tabs → sections → groups → market_ids. Frontend hydrates each MarketGroup via market_ids.map(id => marketsById.get(id)). NULL on the DB maps to [] on the wire. See MarketTab. */
            market_tabs?: components["schemas"]["MarketTab"][];
            /** @description Counts for the resolution badges. Computed over the markets served on this payload — for RECURRING_PROPOSITION events that is the windowed subset (live window + capped upcoming/settled cycles), not all cycles the series ever ran. */
            disputed_market_count?: number;
            resolved_market_count?: number;
            /** @description Admin-curated "see also" events surfaced under the detail page. Populated from agara_event_relations. Filtered to non-archived, ordered by position, capped server-side. Empty array when the event has no curated relations. */
            related_events: components["schemas"]["EventListItem"][];
        } | {
            /** @constant */
            event_type: "RECURRING_PROPOSITION";
            display: components["schemas"]["RecurringPropositionEventDisplay"];
            id: string;
            slug: string;
            title: string;
            is_accepting_orders: boolean;
            is_live: boolean;
            is_group_root: boolean;
            start_time: string | null;
            end_time: string | null;
            /** @description Group-rolled-up volume (root + child events' markets). BIGINT 1e-6 micro-USDC (CC-2/CC-4); ships as a branded string. Liquidity moved to display.liquidity. */
            volume_micro: components["schemas"]["MicroUsd"];
            /** @description UUID of the event's primary leg market. For 2-way events this IS the main market. For 3-way events (soccer moneylines, etc.) this is the lead leg sync's rollup picked deterministically — readers needing the full leg group should use the API's `main_market_group` (assembled at read time via `neg_risk_id`). null when the event has no markets. */
            main_market_id: string | null;
            /**
             * @description Event-level resolution. 'ACTIVE' is the wire spelling for null on the DB.
             * @enum {string}
             */
            resolution_status: "ACTIVE" | "PROPOSED" | "DISPUTED" | "RESOLVED" | "VOID";
            /** @description Market-aggregated event state (rolled up from child market `state`s). Optional on the wire (always set by the api) so clients predating the field stay valid. */
            state?: components["schemas"]["EventLifecycleState"];
            categories: {
                slug: string;
                label: string;
                root_slug: string;
                path: {
                    slug: string;
                    label: string;
                }[];
            }[];
            primary_category: {
                slug: string;
                label: string;
            } | null;
            markets: components["schemas"]["MarketDetail"][];
            /** @description Display-friendly sport label derived from display.sport_slug ("Basketball", "Soccer", "Esports"). Used for breadcrumbs and bet-slip unit inference. */
            sport_group?: string;
            /** @description Curated main market(s) for the event card header. For 3-way sports this carries all 3 legs (home/draw/away); for 2-way moneylines / prop-only events it's a single-market group. See MainMarketGroup. */
            main_market_group?: components["schemas"]["MainMarketGroup"];
            /** @description Up to 2 featured spread markets nearest the event's main spread line. Ordered by distance from `display.spreads_main_line`. Detail only — listing card doesn't render spreads. */
            featured_spread_markets?: components["schemas"]["MarketDetail"][];
            /** @description Single featured totals market, preferring exact match on `display.totals_main_line`. Detail only — listing card doesn't render totals. */
            featured_totals_market?: components["schemas"]["MarketDetail"];
            /** @description Pre-assembled event rendering tree: tabs → sections → groups → market_ids. Frontend hydrates each MarketGroup via market_ids.map(id => marketsById.get(id)). NULL on the DB maps to [] on the wire. See MarketTab. */
            market_tabs?: components["schemas"]["MarketTab"][];
            /** @description Counts for the resolution badges. Computed over the markets served on this payload — for RECURRING_PROPOSITION events that is the windowed subset (live window + capped upcoming/settled cycles), not all cycles the series ever ran. */
            disputed_market_count?: number;
            resolved_market_count?: number;
            /** @description Admin-curated "see also" events surfaced under the detail page. Populated from agara_event_relations. Filtered to non-archived, ordered by position, capped server-side. Empty array when the event has no curated relations. */
            related_events: components["schemas"]["EventListItem"][];
        };
        /**
         * @description Top-level event-shell selector. Set by sync's rollupAgaraEventMainMarket from event_type + display.sport_slug.
         *
         *     - 'regular'       — GAME with non-esports sport (or unknown sport, with a warn-log).
         *     - 'esports'       — GAME with sport in the esports set (lol, cs2, val, ...).
         *     - 'multi_outcome' — MULTI_OUTCOME event_type.
         *     - 'proposition'   — PROPOSITION event_type.
         * @enum {string}
         */
        EventLayout: "regular" | "esports" | "multi_outcome" | "proposition" | "recurring_proposition";
        /** @enum {string} */
        EventLifecycleState: "DRAFT" | "INACTIVE" | "ACTIVE" | "PROPOSED" | "DISPUTED" | "PAUSED" | "RESOLVED" | "VOID" | "ARCHIVED";
        /** @description Card shape for category listings. Discriminated on `event_type`; narrowing `event.event_type === 'GAME'` narrows `event.display` to `GameEventDisplay`. */
        EventListItem: {
            /** @constant */
            event_type: "GAME";
            market_count: number;
            display: components["schemas"]["GameEventDisplay"];
            game_state: components["schemas"]["GameState"] | null;
            id: string;
            slug: string;
            title: string;
            is_accepting_orders: boolean;
            is_live: boolean;
            is_group_root: boolean;
            start_time: string | null;
            end_time: string | null;
            /** @description Group-rolled-up volume (root + child events' markets). The sole listing sort key. BIGINT 1e-6 micro-USDC (CC-2/CC-4); ships as a branded string. Liquidity moved to display.liquidity (read-only, never sorted on). */
            volume_micro: components["schemas"]["MicroUsd"];
            /** @description Operator-assigned primary category (agara_events.primary_category_id), the EventDetail.primary_category twin: null when unset or inactive. */
            primary_category: {
                slug: string;
                label: string;
            } | null;
            /** @description UUID of the event's primary leg market. For 2-way events this IS the main market. For 3-way events (soccer moneylines, etc.) this is the lead leg sync's rollup picked deterministically — readers needing the full leg group should use the API's `main_market_group` (assembled at read time via `neg_risk_id`). null when the event has no markets. */
            main_market_id: string | null;
            /**
             * @description Event-level resolution. 'ACTIVE' is the wire spelling for null on the DB.
             * @enum {string}
             */
            resolution_status: "ACTIVE" | "PROPOSED" | "DISPUTED" | "RESOLVED" | "VOID";
            /** @description Market-aggregated event state (rolled up from child market `state`s). Optional on the wire (always set by the api) so clients predating the field stay valid. */
            state?: components["schemas"]["EventLifecycleState"];
            /** @description Curated main market(s) for the event card header. For 3-way sports this carries all 3 legs (home/draw/away); for 2-way moneylines / prop-only events it's a single-market group. See MainMarketGroup. */
            main_market_group?: components["schemas"]["MainMarketGroup"];
        } | {
            /** @constant */
            event_type: "MULTI_OUTCOME";
            market_count: number;
            display: components["schemas"]["MultiOutcomeEventDisplay"];
            /** @description Top-2 candidate markets by YES price (accepting-orders first). Populated only when the `include_markets` query param is set. */
            listing_markets?: components["schemas"]["ListingMarket"][];
            id: string;
            slug: string;
            title: string;
            is_accepting_orders: boolean;
            is_live: boolean;
            is_group_root: boolean;
            start_time: string | null;
            end_time: string | null;
            /** @description Group-rolled-up volume (root + child events' markets). The sole listing sort key. BIGINT 1e-6 micro-USDC (CC-2/CC-4); ships as a branded string. Liquidity moved to display.liquidity (read-only, never sorted on). */
            volume_micro: components["schemas"]["MicroUsd"];
            /** @description Operator-assigned primary category (agara_events.primary_category_id), the EventDetail.primary_category twin: null when unset or inactive. */
            primary_category: {
                slug: string;
                label: string;
            } | null;
            /** @description UUID of the event's primary leg market. For 2-way events this IS the main market. For 3-way events (soccer moneylines, etc.) this is the lead leg sync's rollup picked deterministically — readers needing the full leg group should use the API's `main_market_group` (assembled at read time via `neg_risk_id`). null when the event has no markets. */
            main_market_id: string | null;
            /**
             * @description Event-level resolution. 'ACTIVE' is the wire spelling for null on the DB.
             * @enum {string}
             */
            resolution_status: "ACTIVE" | "PROPOSED" | "DISPUTED" | "RESOLVED" | "VOID";
            /** @description Market-aggregated event state (rolled up from child market `state`s). Optional on the wire (always set by the api) so clients predating the field stay valid. */
            state?: components["schemas"]["EventLifecycleState"];
            /** @description Curated main market(s) for the event card header. For 3-way sports this carries all 3 legs (home/draw/away); for 2-way moneylines / prop-only events it's a single-market group. See MainMarketGroup. */
            main_market_group?: components["schemas"]["MainMarketGroup"];
        } | {
            /** @constant */
            event_type: "PROPOSITION";
            market_count: number;
            display: components["schemas"]["PropositionEventDisplay"];
            /** @description The event's single main market. Populated only when the `include_markets` query param is set. */
            listing_markets?: components["schemas"]["ListingMarket"][];
            id: string;
            slug: string;
            title: string;
            is_accepting_orders: boolean;
            is_live: boolean;
            is_group_root: boolean;
            start_time: string | null;
            end_time: string | null;
            /** @description Group-rolled-up volume (root + child events' markets). The sole listing sort key. BIGINT 1e-6 micro-USDC (CC-2/CC-4); ships as a branded string. Liquidity moved to display.liquidity (read-only, never sorted on). */
            volume_micro: components["schemas"]["MicroUsd"];
            /** @description Operator-assigned primary category (agara_events.primary_category_id), the EventDetail.primary_category twin: null when unset or inactive. */
            primary_category: {
                slug: string;
                label: string;
            } | null;
            /** @description UUID of the event's primary leg market. For 2-way events this IS the main market. For 3-way events (soccer moneylines, etc.) this is the lead leg sync's rollup picked deterministically — readers needing the full leg group should use the API's `main_market_group` (assembled at read time via `neg_risk_id`). null when the event has no markets. */
            main_market_id: string | null;
            /**
             * @description Event-level resolution. 'ACTIVE' is the wire spelling for null on the DB.
             * @enum {string}
             */
            resolution_status: "ACTIVE" | "PROPOSED" | "DISPUTED" | "RESOLVED" | "VOID";
            /** @description Market-aggregated event state (rolled up from child market `state`s). Optional on the wire (always set by the api) so clients predating the field stay valid. */
            state?: components["schemas"]["EventLifecycleState"];
            /** @description Curated main market(s) for the event card header. For 3-way sports this carries all 3 legs (home/draw/away); for 2-way moneylines / prop-only events it's a single-market group. See MainMarketGroup. */
            main_market_group?: components["schemas"]["MainMarketGroup"];
        } | {
            /** @constant */
            event_type: "RECURRING_PROPOSITION";
            /** @description Never sent: a cycle series gains a market every cadence, so no total is meaningful and an exact count would be O(series lifetime) on the hottest query we have. */
            market_count?: unknown;
            display: components["schemas"]["RecurringPropositionEventDisplay"];
            /** @description The event's single main market (current cycle). Populated only when the `include_markets` query param is set. */
            listing_markets?: components["schemas"]["ListingMarket"][];
            id: string;
            slug: string;
            title: string;
            is_accepting_orders: boolean;
            is_live: boolean;
            is_group_root: boolean;
            start_time: string | null;
            end_time: string | null;
            /** @description Group-rolled-up volume (root + child events' markets). The sole listing sort key. BIGINT 1e-6 micro-USDC (CC-2/CC-4); ships as a branded string. Liquidity moved to display.liquidity (read-only, never sorted on). */
            volume_micro: components["schemas"]["MicroUsd"];
            /** @description Operator-assigned primary category (agara_events.primary_category_id), the EventDetail.primary_category twin: null when unset or inactive. */
            primary_category: {
                slug: string;
                label: string;
            } | null;
            /** @description UUID of the event's primary leg market. For 2-way events this IS the main market. For 3-way events (soccer moneylines, etc.) this is the lead leg sync's rollup picked deterministically — readers needing the full leg group should use the API's `main_market_group` (assembled at read time via `neg_risk_id`). null when the event has no markets. */
            main_market_id: string | null;
            /**
             * @description Event-level resolution. 'ACTIVE' is the wire spelling for null on the DB.
             * @enum {string}
             */
            resolution_status: "ACTIVE" | "PROPOSED" | "DISPUTED" | "RESOLVED" | "VOID";
            /** @description Market-aggregated event state (rolled up from child market `state`s). Optional on the wire (always set by the api) so clients predating the field stay valid. */
            state?: components["schemas"]["EventLifecycleState"];
            /** @description Curated main market(s) for the event card header. For 3-way sports this carries all 3 legs (home/draw/away); for 2-way moneylines / prop-only events it's a single-market group. See MainMarketGroup. */
            main_market_group?: components["schemas"]["MainMarketGroup"];
        };
        EventListSection: {
            /** @description Leaf-category metadata for this section header. */
            category: {
                slug: string;
                label: string;
            };
            /** @description Event ids in render order. Resolve against `events` (single source of truth) — the API never duplicates rows in section vs. flat list. */
            event_ids: string[];
        };
        EventsListResponse: {
            events: components["schemas"]["EventListItem"][];
            pagination: components["schemas"]["Pagination"];
            /** @description Optional section grouping. Present only when the API has chosen a grouped layout for this query (currently: filter=sports_live, where events are bucketed by leaf category, sections sorted by aggregate volume desc). Absent ⇒ caller renders the flat `events` list with whatever local grouping it normally applies. */
            sections?: components["schemas"]["EventListSection"][];
        };
        /** @enum {string} */
        Exchange: "AGARA";
        /** FieldError */
        FieldError: {
            path: (string | number)[];
            code: string;
            message: string;
        };
        /**
         * @description Display bag for `event_type === 'GAME'` — sports games with two teams, sport/league classification.
         *
         *     Live state (score + match lifecycle/period) lives on the top-level `game_state` column on agara_events, NOT in this display bag. WS-tick writes target a dedicated column to avoid fragmenting the display JSONB blob on every live frame. The wire-shape `EventDetail.game_state` surfaces it.
         */
        GameEventDisplay: {
            image_url?: string;
            icon_url?: string;
            description?: string;
            /** @description Free-text shown under the event title on the detail page (may be multi-line). */
            subtext?: string;
            /** @description YYYY-MM-DD bucketable event date (from gameStartTime / startDate). */
            display_date?: string;
            /** @description Group-rolled-up liquidity (root + child events' markets). Set on roots only by rollupAgaraEventMetrics. The corresponding volume rollup lives on the top-level `agara_events.volume` column because listings sort by it; liquidity is read-only and stays in display. */
            liquidity?: number;
            has_three_way_markets?: boolean;
            has_prop_markets?: boolean;
            has_legacy_prop_markets?: boolean;
            market_categories?: string[];
            /** @description Free-form tag list (politics, crypto, culture, etc.). */
            tags?: string[];
            /** @description URL or free-text describing how the event will be settled. Applies to every event type — individual market resolution sources live on each market's display bag. */
            resolution_source?: string;
            /** @description Top-level layout selector. Set by sync. See EventLayout. */
            layout?: components["schemas"]["EventLayout"];
            /** @description Per-game tab label for esports layouts. null for non-esports. */
            game_unit_label?: components["schemas"]["GameUnitLabel"] | null;
            /** @description UUID of the agara_markets row anchoring the spreads section. Set by sync's rollup. null when the event has no eligible spread market. */
            spreads_main_market_id?: string | null;
            /** @description UUID of the agara_markets row anchoring the totals section. null when the event has no eligible over/under market. */
            totals_main_market_id?: string | null;
            agara_cms?: components["schemas"]["AgaraCmsEventDisplay"];
            team_a?: components["schemas"]["SportsTeam"];
            team_b?: components["schemas"]["SportsTeam"];
            sport_slug?: string;
            league_slug?: string;
            /** @description Sync-rendered fixture-context display string — `"Best of 3 maps"`, `"Best of 5 games"`, eventually `"Best of 5 sets"` / `"5 rounds"` / `"T20"`. Persists across the live → final transition so recap and pre-game surfaces both have the format on hand. Esports-only today; other sports require sport-specific lookup work that isn't wired yet. Absent when the format isn't derivable. */
            format_label?: string;
            spreads_main_line?: number;
            totals_main_line?: number;
        };
        /** @description GAME-only live state — the contents of the agara_events.game_state JSONB column. Carries the WS-tick-mutating fields (score + match lifecycle/period) that used to live on display.score / display.match. */
        GameState: {
            id: number;
            /** @description Parsed score. Discriminated on `sport` — callers should narrow via `score.sport === 'tennis'` before accessing per-set detail. Absent when the game hasn't started or the source hasn't shipped a parseable score yet. */
            score?: components["schemas"]["GameStateScore"];
            /** @description Normalized match state — lifecycle (scheduled / live / intermission / final / cancelled / postponed) plus a sync-rendered `live_label` string. Absent when sync hasn't stamped it (forward-compat — treat as `'scheduled'`). */
            match?: components["schemas"]["GameStateMatch"];
        };
        /** @description Catch-all for any sport that fits a simple match-level pair — most of our current coverage (basketball, soccer, baseball, hockey, rugby, esports series, combat sports). */
        GameStateDefaultScore: {
            /** @constant */
            sport: "default";
            match: {
                a: number;
                b: number;
            };
        };
        /** @enum {string} */
        GameStateLifecycle: "scheduled" | "delayed" | "live" | "intermission" | "final" | "cancelled" | "postponed";
        GameStateMatch: {
            lifecycle: components["schemas"]["GameStateLifecycle"];
            /** @description Sync-rendered current-state display string — `"Q3 · 06:44"`, `"TOP 5 · 02:14"`, `"OT"`, `"BREAK"`, `"HT"`, `"S2"`. Sport-aware vocabulary lives in the sync adapter; consumers render this string verbatim. Present only when `lifecycle` is `live` or `intermission` — other lifecycles carry their own UI treatment. */
            live_label?: string;
        };
        /**
         * @description Per-sport score shape. Discriminated on `sport` — TypeScript narrows `score.sport === 'tennis'` to expose the set-level array. `match.a` / `match.b` always line up with team_a / team_b and represent the sport's canonical match-level result (points for basketball, goals for soccer, sets won for tennis, maps won for esports series, etc.).
         *
         *     Today we specialize only tennis (every other sport fits the simple two-number match result). Cricket / golf / motorsports land as peer variants when we have real demand to model them richly — the discriminator is deliberately the sport itself so future additions don't conflict with an orthogonal format tag.
         */
        GameStateScore: components["schemas"]["GameStateTennisScore"] | components["schemas"]["GameStateDefaultScore"];
        GameStateTennisScore: {
            /** @constant */
            sport: "tennis";
            match: {
                a: number;
                b: number;
            };
            sets: {
                a: number;
                b: number;
                tiebreak?: {
                    a: number;
                    b: number;
                };
            }[];
        };
        /**
         * @description Esports per-game tab label. 'Map' for FPS sports (cs2/val/cod/ow/r6siege), 'Game' for MOBA + sc2. null for non-esports events.
         * @enum {string}
         */
        GameUnitLabel: "Game" | "Map";
        /** @description Lean market shape for listing-card enrichment, shipped only when the `include_markets` query param is set. PROPOSITION / RECURRING_PROPOSITION events ship their single main market; MULTI_OUTCOME events ship their top-2 candidate markets by YES price. Subset of `MarketDetail` — just the identity + outcomes the card needs. */
        ListingMarket: {
            id: string;
            /** @description Optional on the wire (always set by the api) so fixtures predating the field stay valid. */
            state?: components["schemas"]["MarketState"];
            slug: string;
            exchange: components["schemas"]["Exchange"];
            question: string;
            display_label: string;
            /** @description Pyth Hermes feed id stamped at provision (crypto cycles) — lets the listing card stream a live price from the same source the market resolves against. Absent for non-pyth markets. */
            pyth_feed_id?: string;
            /** @description Pyth Pro symbol pinned to this market, if configured. */
            pyth_pro_symbol?: string;
            /** @description The agara security symbol (the spec's asset) keying the settlement-lineage tick series. With `security_provider`, the key for `GET /prices/ticks`. Absent when the spec has no asset, or the asset has no securities-master row. */
            security_symbol?: string;
            /** @description The spec's primary provider slug (e.g. pyth-pro), the other half of that key. Absent whenever `security_symbol` is; the reverse is not guaranteed. */
            security_provider?: string;
            /** @description Reference price snapped at the cycle's reference_at (RECURRING_PROPOSITION) — the card's PRICE TO BEAT stat. String to avoid float precision loss. Absent without a stamped reference. */
            reference_price?: string;
            /** @description ISO 4217 currency of `reference_price`; `null` for a unitless/index value, defaults to USD when absent. */
            currency?: string | null;
            /** @description RECURRING_PROPOSITION cycle window, lifted from the market's display: reference_at = trading-open (T+0), observe_at = trading-close + propose trigger (the countdown target). Absent on non-cycle markets. */
            reference_at?: string;
            observe_at?: string;
            outcomes: components["schemas"]["ListingMarketOutcome"][];
        };
        /** @description Lean outcome shape for listing-card enrichment — only the fields a crypto/proposition card renders (label/% + live-price token). Distinct from the full `MarketOutcome` (best_bid/ask, spread, resolution price, display bag) to keep the opt-in listing payload small. */
        ListingMarketOutcome: {
            label: string;
            short_label?: string;
            kind?: components["schemas"]["OutcomeKind"];
            price_micro: components["schemas"]["MicroProbability"] | null;
            token_id: string | null;
        };
        /**
         * @description Curated group of markets the event card header renders as ONE chart. Distinct from MarketTab's MarketGroup (id-only references inside the tabs tree) — this carries embedded MarketDetail rows so the renderer doesn't have to re-resolve through marketsById for the header.
         *
         *     - Singletons (NBA, NFL, tennis moneylines) — `markets` length 1.
         *     - 3-way moneylines (soccer, rugby) — `markets` length 3, sorted   home → draw → away by `three_way_role`.
         *
         *     `neg_risk_id` is the linking key when length > 1 (mirrors MarketDetail.neg_risk_id on every member). null for singletons. The API derives the group at read time from the selected main market and its sibling legs in the loaded markets list — sync still picks one leg as the row's main_market_id; that leg becomes the seed for sibling discovery.
         */
        MainMarketGroup: {
            neg_risk_id: string | null;
            /** @description Markets in display order. Length 1 for singletons; length 3 for 3-way moneylines (sorted home → draw → away). */
            markets: components["schemas"]["MarketDetail"][];
        };
        MarketDetail: {
            id: string;
            /** @description Single lifecycle state (agara_markets.state). is_accepting_orders and resolution_status below are derived from it for wire back-compat. Optional on the wire (always set by the api) so clients/fixtures predating the field stay valid. */
            state?: components["schemas"]["MarketState"];
            /** @description Trading-side backend this market lists on. Mirrors `agara_markets.source` and the router's `agara_domain::Exchange`. Drives per-call routing on the trading side — buy validation reads the matching backend's free cash, order placement targets the matching wallet, etc. Distinct from `source_market_type` (gamma sport/event taxonomy: `'match_result'`, `'spread'`, …). */
            exchange: components["schemas"]["Exchange"];
            slug: string;
            question: string;
            display_type: components["schemas"]["DisplayType"] | null;
            source_market_type: string | null;
            outcomes: components["schemas"]["MarketOutcome"][];
            is_accepting_orders: boolean;
            /** @description Admin-driven operational halt. Set ⇒ trading is stopped and the engine has rejected orders since `halted_at`. Distinct from `is_accepting_orders=false` due to trading-window close. */
            halted_at: string | null;
            halted_reason: string | null;
            /** @description Total trading volume to date. BIGINT 1e-6 micro-USDC (CC-2/CC-4). Wire: branded string. Render via `formatUsd()`. */
            volume_micro: components["schemas"]["MicroUsd"];
            /** @description Snapshot of liquidity at last sync. NUMERIC in display.liquidity (legacy convention, not micro-units yet). */
            liquidity: string;
            /** @description 24h trading volume. BIGINT 1e-6 micro-USDC. */
            volume_24h_micro: components["schemas"]["MicroUsd"];
            /** @description Probability tick size (e.g. 0.001). 1e-6 micro-units. */
            tick_size_micro: components["schemas"]["MicroProbability"] | null;
            /** @description AGARA matching-engine config in engine-native units. Set only on AGARA markets that have been chain-anchored (admin CTF prepare populated the engine_* columns); null on non-AGARA markets and on AGARA markets pre-anchor. */
            engine_config: components["schemas"]["AgaraEngineConfig"] | null;
            /** @description Minimum order size in USDC. 1e-6 micro-units. */
            min_order_size_micro: components["schemas"]["MicroUsd"] | null;
            /** @description Sport-specific number (handicap / total / threshold). Stays NUMERIC, not micro-units (CC-2 case-by-case exception). */
            line: string | null;
            resolution_status: components["schemas"]["ResolutionStatus"] | null;
            /** @description Server-derived "who won?" label — computed at serialization time from the outcome row whose resolution_price = 1.0. NULL when unresolved or VOID (no winner). */
            resolution_outcome: string | null;
            resolved_at: string | null;
            /** @description Settlement-grade resolution values (see `MarketResolution`). Populated only on single-market and event-detail responses; null on list cards and until a canonical attempt row exists. */
            resolution: components["schemas"]["MarketResolution"] | null;
            archived_at: string | null;
            condition_id?: string | null;
            neg_risk_id?: string | null;
            /** @description Which leg of a 3-way moneyline this market represents — set by sync's three-way-role classifier. null for non-3-way markets. */
            three_way_role?: components["schemas"]["ThreeWayRole"] | null;
            /** @description True when sync's classifier tagged this market as a 3-way leg (soccer-style moneyline with draw). Pairs with `neg_risk_id` to separate moneyline-leg groups from MULTI_OUTCOME candidate groups (which share `neg_risk_id` but have `is_three_way=false`). */
            is_three_way?: boolean | null;
            /** @description Esports per-game routing — extracted from market slug. null on non-esports markets / when the slug carries no -gameN suffix. */
            game_number?: number | null;
            /** @description Canonical card title — sync's computeMarketLabel recipe always populates this (3-way leg name → tennis-derived → esports strip → groupItemTitle → question). Renderers trust it directly. */
            display_label: string;
            display: components["schemas"]["MarketDisplay"];
        };
        MarketDisplay: components["schemas"]["BaseMarketDisplay"];
        /**
         * @description One card's worth of markets. Singletons are length 1; 3-way moneyline groups are length 3 (home/draw/away); lined-card groups collect every line of one spread/totals family. The frontend renders the group as one card whose internals depend on the parent section's render_hint.
         *
         *     Markets are referenced by id, not embedded — full MarketDetail rows live in EventDetail.markets[]. The contract is: every id in market_tabs MUST be present in markets[].
         */
        MarketGroup: {
            /** @description Stable id across syncs — neg_risk_id for 3-way / lined groups, market.id for singletons. Frontend uses as React key. */
            key: string;
            /** @description Markets in display order. Resolve via marketsById.get(id). */
            market_ids: string[];
        };
        MarketOutcome: {
            id: string;
            index: number;
            /** @description Canonical raw outcome text ("Celtics", "Yes", "Over 2.5"). Hoisted by the API serializer from `display.label` (storage home). Used in resolution-outcome label derivation and as fallback render text when the classifier didn't produce a short_label. */
            label: string;
            /** @description Last-seen probability midpoint, BIGINT 1e-6 micro-units (CC-2/CC-4). Wire format: branded string, e.g. `"500000"` for 0.5. Render via `formatProbability()` from `@agara/schemas`. */
            price_micro: components["schemas"]["MicroProbability"] | null;
            token_id: string | null;
            /** @description Top-of-book bid for this outcome's token. BIGINT 1e-6 micro-units. Sync seeds via binary-CLOB symmetry; WS book-tick handler overwrites live. */
            best_bid_micro?: components["schemas"]["MicroProbability"] | null;
            /** @description Top-of-book ask. Same write path as best_bid_micro. 1e-6 micro-units. */
            best_ask_micro?: components["schemas"]["MicroProbability"] | null;
            /** @description Persisted spread (best_ask_micro − best_bid_micro). 1e-6 micro-units. */
            spread_micro?: components["schemas"]["MicroProbability"] | null;
            /** @description Semantic kind. Set by sync; absent on rows written before outcome-classification rolled out. Readers should treat `undefined` the same as `'unknown'` and fall back to label-based rendering. */
            kind?: components["schemas"]["OutcomeKind"];
            /** @description Short pill-button label (e.g. "LAL", "Over 2.5", "Draw"). Sync derives at classification time using event team data when relevant. */
            short_label?: string;
            /** @description Which event-level team this outcome refers to. Resolves through event.display.team_a / team_b at render time — see OutcomeTeamSide doc comment. null for non-team outcomes (over/under, draw, no, etc.). */
            team_side?: components["schemas"]["OutcomeTeamSide"] | null;
            /**
             * @description Pill side for binary card layouts: index-0 → 'left', index-1 → 'right'. null for ternary or grid layouts.
             * @enum {string|null}
             */
            side?: "left" | "right" | null;
            /** @description For kind='over'/'under' on player props, the stat threshold (the stored market.line). null otherwise. */
            threshold?: number | null;
            /** @description Per-outcome payout in USDC terms after resolution. BIGINT 1e-6 micro-units: 1_000_000 winner, 0 loser, 500_000 void. NULL while parent market is PROPOSED / DISPUTED / not yet resolved. */
            resolution_price_micro?: components["schemas"]["MicroProbability"] | null;
            /** @description Pre-rendered bet-slip header sentence for THIS outcome — single source of truth for the bet-slip card title. Sync's buildOutcomeBetSlipTitle stamps the full sentence at write time (e.g. "Lakers to win", "Over 2.5 total goals", "Both Teams to Score: Yes"). Client renders verbatim — no composition, no English literal templates, no client-side lookup tables. Hoisted from display.bet_slip_title. */
            bet_slip_title?: string;
            /** @description Label shown when the market resolves (e.g. "BTC Up", "RMA won the match"). Admin-authored in the CMS, or auto-populated for recurring Up/Down series. Hoisted from display.resolution_label. */
            resolution_label?: string;
            display?: components["schemas"]["OutcomeDisplay"];
        };
        /** @description Settlement-grade values from `agara_market_resolution_attempts`, canonical source only (the spec's primary provider, the pointer the resolver writes attempts under; fallback-source rows are excluded). FINAL PRICE and the outcome direction must render from here, never from a chart artifact. Values are decimal strings. */
        MarketResolution: {
            /** @description Canonical source: the resolution_spec primary provider (e.g. `pyth-pro`). */
            source: string;
            /** @description `role='reference'` value (the window's opening price). null when no reference attempt exists or it failed before recording a value. */
            reference_value: string | null;
            /** @description `role='observation'` value (the settlement price pinned to the window boundary). null before observation or when the attempt failed; key "settled" off this, not `observed_at`. */
            observed_value: string | null;
            /** @description `observed_at` of the observation attempt (ISO 8601); set even on a failed attempt. */
            observed_at: string | null;
        };
        /** @enum {string} */
        MarketState: "DRAFT" | "PROVISIONED" | "ACTIVE" | "INACTIVE" | "TRADING_HALT" | "PROPOSAL_PENDING" | "PROPOSED" | "DISPUTED" | "RESOLVED" | "VOID" | "ARCHIVED";
        /** @description One tab in the event's tab strip — e.g. 'Game Lines', 'Goalscorers'. PROPOSITION / MULTI_OUTCOME / esports events emit a single 'all' tab. Frontend hides the pill strip when tabs.length === 1. Named to mirror the wire field `market_tabs` on EventDetail. */
        MarketTab: {
            /** @description Stable id used for URL sync, CMS overrides, frontend tab state. */
            key: string;
            /** @description Human-readable tab label. */
            label: string;
            /** @description Backend-owned tab order (ascending). */
            order: number;
            /** @description Sections in display order — the full tab content. */
            sections: components["schemas"]["CategorySection"][];
        };
        MarketsListEventRef: {
            id: string;
            slug: string;
            title: string;
            active: boolean;
        };
        MarketsListItem: {
            id: string;
            /** @description Single lifecycle state (agara_markets.state). is_accepting_orders and resolution_status below are derived from it for wire back-compat. Optional on the wire (always set by the api) so clients/fixtures predating the field stay valid. */
            state?: components["schemas"]["MarketState"];
            /** @description Trading-side backend this market lists on. Mirrors `agara_markets.source` and the router's `agara_domain::Exchange`. Drives per-call routing on the trading side — buy validation reads the matching backend's free cash, order placement targets the matching wallet, etc. Distinct from `source_market_type` (gamma sport/event taxonomy: `'match_result'`, `'spread'`, …). */
            exchange: components["schemas"]["Exchange"];
            slug: string;
            question: string;
            display_type: components["schemas"]["DisplayType"] | null;
            source_market_type: string | null;
            outcomes: components["schemas"]["MarketOutcome"][];
            is_accepting_orders: boolean;
            /** @description Admin-driven operational halt. Set ⇒ trading is stopped and the engine has rejected orders since `halted_at`. Distinct from `is_accepting_orders=false` due to trading-window close. */
            halted_at: string | null;
            halted_reason: string | null;
            /** @description Total trading volume to date. BIGINT 1e-6 micro-USDC (CC-2/CC-4). Wire: branded string. Render via `formatUsd()`. */
            volume_micro: components["schemas"]["MicroUsd"];
            /** @description Snapshot of liquidity at last sync. NUMERIC in display.liquidity (legacy convention, not micro-units yet). */
            liquidity: string;
            /** @description 24h trading volume. BIGINT 1e-6 micro-USDC. */
            volume_24h_micro: components["schemas"]["MicroUsd"];
            /** @description Probability tick size (e.g. 0.001). 1e-6 micro-units. */
            tick_size_micro: components["schemas"]["MicroProbability"] | null;
            /** @description AGARA matching-engine config in engine-native units. Set only on AGARA markets that have been chain-anchored (admin CTF prepare populated the engine_* columns); null on non-AGARA markets and on AGARA markets pre-anchor. */
            engine_config: components["schemas"]["AgaraEngineConfig"] | null;
            /** @description Minimum order size in USDC. 1e-6 micro-units. */
            min_order_size_micro: components["schemas"]["MicroUsd"] | null;
            /** @description Sport-specific number (handicap / total / threshold). Stays NUMERIC, not micro-units (CC-2 case-by-case exception). */
            line: string | null;
            resolution_status: components["schemas"]["ResolutionStatus"] | null;
            /** @description Server-derived "who won?" label — computed at serialization time from the outcome row whose resolution_price = 1.0. NULL when unresolved or VOID (no winner). */
            resolution_outcome: string | null;
            resolved_at: string | null;
            /** @description Settlement-grade resolution values (see `MarketResolution`). Populated only on single-market and event-detail responses; null on list cards and until a canonical attempt row exists. */
            resolution: components["schemas"]["MarketResolution"] | null;
            archived_at: string | null;
            condition_id?: string | null;
            neg_risk_id?: string | null;
            /** @description Which leg of a 3-way moneyline this market represents — set by sync's three-way-role classifier. null for non-3-way markets. */
            three_way_role?: components["schemas"]["ThreeWayRole"] | null;
            /** @description True when sync's classifier tagged this market as a 3-way leg (soccer-style moneyline with draw). Pairs with `neg_risk_id` to separate moneyline-leg groups from MULTI_OUTCOME candidate groups (which share `neg_risk_id` but have `is_three_way=false`). */
            is_three_way?: boolean | null;
            /** @description Esports per-game routing — extracted from market slug. null on non-esports markets / when the slug carries no -gameN suffix. */
            game_number?: number | null;
            /** @description Canonical card title — sync's computeMarketLabel recipe always populates this (3-way leg name → tennis-derived → esports strip → groupItemTitle → question). Renderers trust it directly. */
            display_label: string;
            display: components["schemas"]["MarketDisplay"];
            event: components["schemas"]["MarketsListEventRef"];
        };
        MarketsListResponse: {
            markets: components["schemas"]["MarketsListItem"][];
            pagination: components["schemas"]["Pagination"];
        };
        MicroProbability: string;
        MicroUsd: string;
        /**
         * @description Display bag for `event_type === 'MULTI_OUTCOME'` — futures / elections / award shows / championship races. Multiple candidates, each a binary Yes/No market linked via `neg_risk_id`.
         *
         *     Summary / candidate fields (candidate_count, top_*, top_candidates, resolved_winner_name, is_future, is_child_event) are computed at response time by the API serializer via `@agara/shared/sports. deriveMultiOutcomeEnrichment`. Sync doesn't currently persist them — a future `rollupAgaraEventMulti` pass can lift the computation to write-time if list-page reads become hot. Read-time keeps prices fresh (post-resolution candidates can't linger in stale snapshots) and avoids write amplification.
         */
        MultiOutcomeEventDisplay: {
            image_url?: string;
            icon_url?: string;
            description?: string;
            /** @description Free-text shown under the event title on the detail page (may be multi-line). */
            subtext?: string;
            /** @description YYYY-MM-DD bucketable event date (from gameStartTime / startDate). */
            display_date?: string;
            /** @description Group-rolled-up liquidity (root + child events' markets). Set on roots only by rollupAgaraEventMetrics. The corresponding volume rollup lives on the top-level `agara_events.volume` column because listings sort by it; liquidity is read-only and stays in display. */
            liquidity?: number;
            has_three_way_markets?: boolean;
            has_prop_markets?: boolean;
            has_legacy_prop_markets?: boolean;
            market_categories?: string[];
            /** @description Free-form tag list (politics, crypto, culture, etc.). */
            tags?: string[];
            /** @description URL or free-text describing how the event will be settled. Applies to every event type — individual market resolution sources live on each market's display bag. */
            resolution_source?: string;
            /** @description Top-level layout selector. Set by sync. See EventLayout. */
            layout?: components["schemas"]["EventLayout"];
            /** @description Per-game tab label for esports layouts. null for non-esports. */
            game_unit_label?: components["schemas"]["GameUnitLabel"] | null;
            /** @description UUID of the agara_markets row anchoring the spreads section. Set by sync's rollup. null when the event has no eligible spread market. */
            spreads_main_market_id?: string | null;
            /** @description UUID of the agara_markets row anchoring the totals section. null when the event has no eligible over/under market. */
            totals_main_market_id?: string | null;
            agara_cms?: components["schemas"]["AgaraCmsEventDisplay"];
            /** @description Topic slug — e.g. `"election"`, `"nba-mvp"`, `"oscars"`, `"tech"`. Higher-level bucket for navigation / breadcrumbs. Analog to GAME's `sport_slug`; the /categories tree already covers coarser navigation. */
            subcategory?: string;
            /** @description Count of candidate markets ever listed on the event — includes eliminated/resolved candidates. */
            candidate_count?: number;
            /** @description Count of candidates still active — excludes archived markets AND markets that have resolved to a losing outcome. Frontend should prefer this for "N remaining" copy over `candidate_count`. */
            active_candidate_count?: number;
            /** @description `display_label` of the highest-volume ACTIVE candidate market. */
            top_candidate_name?: string;
            /** @description Yes-outcome price of the highest-volume active candidate. 1e-6 micro-units (CC-4). Render via `formatProbability`. */
            top_candidate_price_micro?: components["schemas"]["MicroProbability"];
            /** @description Ranked active candidates (default top-3). Structural — card renderers can slice to their N. Each entry carries name + price + volume in 1e-6 micro-units; frontend builds a "Biden 52% · Trump 44% · Harris 12%" pill row directly. */
            top_candidates?: {
                name: string;
                price_micro?: components["schemas"]["MicroProbability"];
                volume_micro: components["schemas"]["MicroUsd"];
            }[];
            /** @description Name of the candidate that resolved to Yes (the contest winner). Absent while no candidate has resolved. Lets callers render "Biden won" / "Spurs won" banners without re-scanning markets. */
            resolved_winner_name?: string;
            /** @description True for real futures (elections, season-long MVPs); false for game-prop child events that happen to be multi-outcome (e.g. first- touchdown scorer in one NFL game). Legacy `is_future` flag. */
            is_future?: boolean;
            /** @description True for multi-outcome groups that belong to a GAME parent (e.g. anytime-goalscorer within one match). Legacy `is_child_event` flag. */
            is_child_event?: boolean;
        };
        /** @description `session` is the earliest trading day on/after the requested date, or null when none exists in the queried range. */
        NextSessionResponse: {
            mic: string;
            session: components["schemas"]["TradingDay"] | null;
        };
        /** OriginProblemDetails */
        OriginProblemDetails: {
            type: string;
            title: string;
            status: number;
            code: string;
            detail?: string;
            /** Format: uuid */
            request_id: string;
            recovery: components["schemas"]["Recovery"];
            field_errors?: components["schemas"]["FieldError"][];
        };
        OutcomeDisplay: components["schemas"]["BaseOutcomeDisplay"];
        /**
         * @description Semantic role of an outcome — set by sync's outcome classifier so the read path never has to string-sniff labels. See scratch/rendering-hints-schema.md §6.1.
         *
         *     - team_home / team_away: outcome IS a team (tennis, NFL spreads).
         *     - draw: outcome IS the draw leg of a 3-way moneyline.
         *     - yes / no: binary proposition or 3-way leg labeled literally Yes/No.
         *     - over / under: O/U totals leg.
         *     - candidate: multi-outcome candidate row (championship, MVP, etc.).
         *     - side_a / side_b: generic two-sided non-team market (cricket toss).
         *     - unknown: classifier couldn't decide — render with raw label fallback.
         * @enum {string}
         */
        OutcomeKind: "team_home" | "team_away" | "draw" | "yes" | "no" | "over" | "under" | "candidate" | "side_a" | "side_b" | "unknown";
        /**
         * @description Reference to the event-level team this outcome is associated with. An indicator only — NO embedded team data — so we don't duplicate the team object that already lives at event.display.team_a / team_b. Renderers resolve via:   outcome.team_side === 'home' → event.display.team_a   outcome.team_side === 'away' → event.display.team_b
         *
         *     Stays valid through any future migration to a first-class agara_teams table — only event-level resolution changes; outcome rows untouched.
         * @enum {string}
         */
        OutcomeTeamSide: "home" | "away";
        Pagination: {
            /** @description Opaque cursor for the next page — null when there are no more results. */
            next_cursor: string | null;
            /** @description Effective page size used on the request. */
            limit: number;
        };
        PricePointBody: {
            value: number;
            mantissa: string;
            expo: number;
            publish_time_micros: number;
        };
        /**
         * @description Display bag for `event_type === 'PROPOSITION'` — single yes/no markets on political / news / culture questions. Legacy stored these as 1-market MULTI_OUTCOME events; sandbox promotes them to their own type so the frontend can render a proposition-specific layout.
         *
         *     Most resolution context (resolution_source, description) lives on the single market, not on the event. Event-level display stays minimal.
         */
        PropositionEventDisplay: {
            image_url?: string;
            icon_url?: string;
            description?: string;
            /** @description Free-text shown under the event title on the detail page (may be multi-line). */
            subtext?: string;
            /** @description YYYY-MM-DD bucketable event date (from gameStartTime / startDate). */
            display_date?: string;
            /** @description Group-rolled-up liquidity (root + child events' markets). Set on roots only by rollupAgaraEventMetrics. The corresponding volume rollup lives on the top-level `agara_events.volume` column because listings sort by it; liquidity is read-only and stays in display. */
            liquidity?: number;
            has_three_way_markets?: boolean;
            has_prop_markets?: boolean;
            has_legacy_prop_markets?: boolean;
            market_categories?: string[];
            /** @description Free-form tag list (politics, crypto, culture, etc.). */
            tags?: string[];
            /** @description URL or free-text describing how the event will be settled. Applies to every event type — individual market resolution sources live on each market's display bag. */
            resolution_source?: string;
            /** @description Top-level layout selector. Set by sync. See EventLayout. */
            layout?: components["schemas"]["EventLayout"];
            /** @description Per-game tab label for esports layouts. null for non-esports. */
            game_unit_label?: components["schemas"]["GameUnitLabel"] | null;
            /** @description UUID of the agara_markets row anchoring the spreads section. Set by sync's rollup. null when the event has no eligible spread market. */
            spreads_main_market_id?: string | null;
            /** @description UUID of the agara_markets row anchoring the totals section. null when the event has no eligible over/under market. */
            totals_main_market_id?: string | null;
            agara_cms?: components["schemas"]["AgaraCmsEventDisplay"];
            /** @description Topic slug — e.g. `"fed-policy"`, `"geopolitics"`. */
            subcategory?: string;
        };
        Record_string_never: {
            [key: string]: unknown;
        };
        /** Recovery */
        Recovery: {
            /** @constant */
            strategy: "none";
        } | {
            /** @constant */
            strategy: "retry";
        } | {
            /** @constant */
            strategy: "retry_after";
            after_seconds: number;
        } | {
            /** @constant */
            strategy: "refresh_identity_token";
        } | {
            /** @constant */
            strategy: "check_status";
            resource: components["schemas"]["RecoveryResource"];
        } | ({
            strategy: string;
        } & {
            [key: string]: unknown;
        });
        /** RecoveryResource */
        RecoveryResource: {
            /** @constant */
            kind: "order";
            /** Format: uuid */
            order_id: string;
        } | {
            /** @constant */
            kind: "batch";
            batch_hash: string;
        } | {
            /** @constant */
            kind: "group";
            /** Format: uuid */
            group_id: string;
        } | {
            /** @constant */
            kind: "wallet_status";
        };
        /** @description Display bag for `event_type === 'RECURRING_PROPOSITION'` — a series of binary cycles (BTC up/down 15-min, etc.). One event represents the whole series; markets[] is a served window, not full history: the live cycle group, then whole-cycle-capped unsettled, upcoming, and settled sides. Wire order: live first, soonest upcoming next, then newest-first (settled and stuck cycles interleave by window). The current/live cycle is at event.main_market_id. Series-level metadata (cadence, source providers) is admin-authored on `agara_market_series` — only the fields the rendering needs land on the wire here. */
        RecurringPropositionEventDisplay: {
            image_url?: string;
            icon_url?: string;
            description?: string;
            /** @description Free-text shown under the event title on the detail page (may be multi-line). */
            subtext?: string;
            /** @description YYYY-MM-DD bucketable event date (from gameStartTime / startDate). */
            display_date?: string;
            /** @description Group-rolled-up liquidity (root + child events' markets). Set on roots only by rollupAgaraEventMetrics. The corresponding volume rollup lives on the top-level `agara_events.volume` column because listings sort by it; liquidity is read-only and stays in display. */
            liquidity?: number;
            has_three_way_markets?: boolean;
            has_prop_markets?: boolean;
            has_legacy_prop_markets?: boolean;
            market_categories?: string[];
            /** @description Free-form tag list (politics, crypto, culture, etc.). */
            tags?: string[];
            /** @description URL or free-text describing how the event will be settled. Applies to every event type — individual market resolution sources live on each market's display bag. */
            resolution_source?: string;
            /** @description Top-level layout selector. Set by sync. See EventLayout. */
            layout?: components["schemas"]["EventLayout"];
            /** @description Per-game tab label for esports layouts. null for non-esports. */
            game_unit_label?: components["schemas"]["GameUnitLabel"] | null;
            /** @description UUID of the agara_markets row anchoring the spreads section. Set by sync's rollup. null when the event has no eligible spread market. */
            spreads_main_market_id?: string | null;
            /** @description UUID of the agara_markets row anchoring the totals section. null when the event has no eligible over/under market. */
            totals_main_market_id?: string | null;
            agara_cms?: components["schemas"]["AgaraCmsEventDisplay"];
            /** @description Topic slug — e.g. `"crypto"`. */
            subcategory?: string;
            /** @description Slug of the parent `agara_market_series` row. */
            series_slug?: string;
            /** @description Cycle period in seconds (e.g. 900 for 15 minutes). Drives the countdown / chart axis. */
            cadence_seconds?: number;
        };
        /** @description A titled resolution-source link, authored as a repeatable list. */
        ResolutionSourceLink: {
            title: string;
            url: string;
        };
        /** @enum {string} */
        ResolutionStatus: "DISPUTED" | "PROPOSED" | "RESOLVED" | "VOID";
        SearchCategoryItem: {
            slug: string;
            label: string;
            icon: string | null;
        };
        SearchEventItem: {
            /** @description Greatest trigram word similarity on a 0–1 scale; indirect category matches carry 0. */
            relevance_score: number;
            /** @constant */
            event_type: "GAME";
            market_count: number;
            display: components["schemas"]["GameEventDisplay"];
            game_state: components["schemas"]["GameState"] | null;
            id: string;
            slug: string;
            title: string;
            is_accepting_orders: boolean;
            is_live: boolean;
            is_group_root: boolean;
            start_time: string | null;
            end_time: string | null;
            /** @description Group-rolled-up volume (root + child events' markets). The sole listing sort key. BIGINT 1e-6 micro-USDC (CC-2/CC-4); ships as a branded string. Liquidity moved to display.liquidity (read-only, never sorted on). */
            volume_micro: components["schemas"]["MicroUsd"];
            /** @description Operator-assigned primary category (agara_events.primary_category_id), the EventDetail.primary_category twin: null when unset or inactive. */
            primary_category: {
                slug: string;
                label: string;
            } | null;
            /** @description UUID of the event's primary leg market. For 2-way events this IS the main market. For 3-way events (soccer moneylines, etc.) this is the lead leg sync's rollup picked deterministically — readers needing the full leg group should use the API's `main_market_group` (assembled at read time via `neg_risk_id`). null when the event has no markets. */
            main_market_id: string | null;
            /**
             * @description Event-level resolution. 'ACTIVE' is the wire spelling for null on the DB.
             * @enum {string}
             */
            resolution_status: "ACTIVE" | "PROPOSED" | "DISPUTED" | "RESOLVED" | "VOID";
            /** @description Market-aggregated event state (rolled up from child market `state`s). Optional on the wire (always set by the api) so clients predating the field stay valid. */
            state?: components["schemas"]["EventLifecycleState"];
            /** @description Curated main market(s) for the event card header. For 3-way sports this carries all 3 legs (home/draw/away); for 2-way moneylines / prop-only events it's a single-market group. See MainMarketGroup. */
            main_market_group?: components["schemas"]["MainMarketGroup"];
        } | {
            /** @description Greatest trigram word similarity on a 0–1 scale; indirect category matches carry 0. */
            relevance_score: number;
            /** @constant */
            event_type: "MULTI_OUTCOME";
            market_count: number;
            display: components["schemas"]["MultiOutcomeEventDisplay"];
            /** @description Top-2 candidate markets by YES price (accepting-orders first). Populated only when the `include_markets` query param is set. */
            listing_markets?: components["schemas"]["ListingMarket"][];
            id: string;
            slug: string;
            title: string;
            is_accepting_orders: boolean;
            is_live: boolean;
            is_group_root: boolean;
            start_time: string | null;
            end_time: string | null;
            /** @description Group-rolled-up volume (root + child events' markets). The sole listing sort key. BIGINT 1e-6 micro-USDC (CC-2/CC-4); ships as a branded string. Liquidity moved to display.liquidity (read-only, never sorted on). */
            volume_micro: components["schemas"]["MicroUsd"];
            /** @description Operator-assigned primary category (agara_events.primary_category_id), the EventDetail.primary_category twin: null when unset or inactive. */
            primary_category: {
                slug: string;
                label: string;
            } | null;
            /** @description UUID of the event's primary leg market. For 2-way events this IS the main market. For 3-way events (soccer moneylines, etc.) this is the lead leg sync's rollup picked deterministically — readers needing the full leg group should use the API's `main_market_group` (assembled at read time via `neg_risk_id`). null when the event has no markets. */
            main_market_id: string | null;
            /**
             * @description Event-level resolution. 'ACTIVE' is the wire spelling for null on the DB.
             * @enum {string}
             */
            resolution_status: "ACTIVE" | "PROPOSED" | "DISPUTED" | "RESOLVED" | "VOID";
            /** @description Market-aggregated event state (rolled up from child market `state`s). Optional on the wire (always set by the api) so clients predating the field stay valid. */
            state?: components["schemas"]["EventLifecycleState"];
            /** @description Curated main market(s) for the event card header. For 3-way sports this carries all 3 legs (home/draw/away); for 2-way moneylines / prop-only events it's a single-market group. See MainMarketGroup. */
            main_market_group?: components["schemas"]["MainMarketGroup"];
        } | {
            /** @description Greatest trigram word similarity on a 0–1 scale; indirect category matches carry 0. */
            relevance_score: number;
            /** @constant */
            event_type: "PROPOSITION";
            market_count: number;
            display: components["schemas"]["PropositionEventDisplay"];
            /** @description The event's single main market. Populated only when the `include_markets` query param is set. */
            listing_markets?: components["schemas"]["ListingMarket"][];
            id: string;
            slug: string;
            title: string;
            is_accepting_orders: boolean;
            is_live: boolean;
            is_group_root: boolean;
            start_time: string | null;
            end_time: string | null;
            /** @description Group-rolled-up volume (root + child events' markets). The sole listing sort key. BIGINT 1e-6 micro-USDC (CC-2/CC-4); ships as a branded string. Liquidity moved to display.liquidity (read-only, never sorted on). */
            volume_micro: components["schemas"]["MicroUsd"];
            /** @description Operator-assigned primary category (agara_events.primary_category_id), the EventDetail.primary_category twin: null when unset or inactive. */
            primary_category: {
                slug: string;
                label: string;
            } | null;
            /** @description UUID of the event's primary leg market. For 2-way events this IS the main market. For 3-way events (soccer moneylines, etc.) this is the lead leg sync's rollup picked deterministically — readers needing the full leg group should use the API's `main_market_group` (assembled at read time via `neg_risk_id`). null when the event has no markets. */
            main_market_id: string | null;
            /**
             * @description Event-level resolution. 'ACTIVE' is the wire spelling for null on the DB.
             * @enum {string}
             */
            resolution_status: "ACTIVE" | "PROPOSED" | "DISPUTED" | "RESOLVED" | "VOID";
            /** @description Market-aggregated event state (rolled up from child market `state`s). Optional on the wire (always set by the api) so clients predating the field stay valid. */
            state?: components["schemas"]["EventLifecycleState"];
            /** @description Curated main market(s) for the event card header. For 3-way sports this carries all 3 legs (home/draw/away); for 2-way moneylines / prop-only events it's a single-market group. See MainMarketGroup. */
            main_market_group?: components["schemas"]["MainMarketGroup"];
        } | {
            /** @description Greatest trigram word similarity on a 0–1 scale; indirect category matches carry 0. */
            relevance_score: number;
            /** @constant */
            event_type: "RECURRING_PROPOSITION";
            /** @description Never sent: a cycle series gains a market every cadence, so no total is meaningful and an exact count would be O(series lifetime) on the hottest query we have. */
            market_count?: unknown;
            display: components["schemas"]["RecurringPropositionEventDisplay"];
            /** @description The event's single main market (current cycle). Populated only when the `include_markets` query param is set. */
            listing_markets?: components["schemas"]["ListingMarket"][];
            id: string;
            slug: string;
            title: string;
            is_accepting_orders: boolean;
            is_live: boolean;
            is_group_root: boolean;
            start_time: string | null;
            end_time: string | null;
            /** @description Group-rolled-up volume (root + child events' markets). The sole listing sort key. BIGINT 1e-6 micro-USDC (CC-2/CC-4); ships as a branded string. Liquidity moved to display.liquidity (read-only, never sorted on). */
            volume_micro: components["schemas"]["MicroUsd"];
            /** @description Operator-assigned primary category (agara_events.primary_category_id), the EventDetail.primary_category twin: null when unset or inactive. */
            primary_category: {
                slug: string;
                label: string;
            } | null;
            /** @description UUID of the event's primary leg market. For 2-way events this IS the main market. For 3-way events (soccer moneylines, etc.) this is the lead leg sync's rollup picked deterministically — readers needing the full leg group should use the API's `main_market_group` (assembled at read time via `neg_risk_id`). null when the event has no markets. */
            main_market_id: string | null;
            /**
             * @description Event-level resolution. 'ACTIVE' is the wire spelling for null on the DB.
             * @enum {string}
             */
            resolution_status: "ACTIVE" | "PROPOSED" | "DISPUTED" | "RESOLVED" | "VOID";
            /** @description Market-aggregated event state (rolled up from child market `state`s). Optional on the wire (always set by the api) so clients predating the field stay valid. */
            state?: components["schemas"]["EventLifecycleState"];
            /** @description Curated main market(s) for the event card header. For 3-way sports this carries all 3 legs (home/draw/away); for 2-way moneylines / prop-only events it's a single-market group. See MainMarketGroup. */
            main_market_group?: components["schemas"]["MainMarketGroup"];
        };
        SearchMarketEventRef: {
            slug: string;
            title: string;
            image_url: string | null;
        };
        SearchMarketItem: {
            /** @description Greatest trigram word similarity on a 0–1 scale; indirect category matches carry 0. */
            relevance_score: number;
            id: string;
            slug: string;
            question: string;
            is_accepting_orders: boolean;
            volume_micro: components["schemas"]["MicroUsd"];
            event: components["schemas"]["SearchMarketEventRef"];
            outcomes: components["schemas"]["SearchOutcome"][];
        };
        SearchOutcome: {
            label: string;
            price_micro: components["schemas"]["MicroProbability"] | null;
        };
        SearchResponse: {
            events: components["schemas"]["SearchEventItem"][];
            markets: components["schemas"]["SearchMarketItem"][];
            categories: components["schemas"]["SearchCategoryItem"][];
        };
        SecuritiesResponse: {
            securities: components["schemas"]["Security"][];
        };
        /** @description A tradeable underlying in the securities master. */
        Security: {
            symbol: string;
            label: string;
            /** @description Listing-venue MIC. Every security has a venue — crypto pairs live on the synthetic XCRYPTO venue — so the venue carries currency + calendar uniformly. */
            mic: string;
            /** @description Instrument asset class; `index` is unitless (no currency), all others quote in the venue currency. One of ASSET_CLASSES. */
            asset_class: string;
            active: boolean;
        };
        /** @description One intraday active interval, wall-clock HH:MM in the venue timezone. Breaks are the gaps between segments: a midday break is two segments, two breaks are three, a half day is a single shorter one, a closed day / outage is no segments at all. */
        SessionSegment: {
            open: string;
            close: string;
        };
        ShimHistory: {
            /** @constant */
            s: "ok";
            t: number[];
            c: number[];
        };
        SportsTeam: {
            name: string;
            abbreviation: string | null;
            logo_url: string | null;
            color: string | null;
            record: string | null;
            sport_slug: string | null;
            league_slug: string | null;
            provider_id: string | null;
            source_team_id: string | null;
        };
        /** @enum {string} */
        ThreeWayRole: "home" | "draw" | "away";
        TokenHistoryResponse: {
            /** @constant */
            s: "ok";
            t: number[];
            c: number[];
            latestTradeAt: number | null;
        };
        /** @description A single (mic, date) session. `is_trading_day = false` (no segments) is a definitive "closed" answer; the absence of a row is "no data" (a 404). */
        TradingDay: {
            mic: string;
            date: string;
            is_trading_day: boolean;
            /** @description This date's active intervals; empty on a closed day or outage. */
            segments: components["schemas"]["SessionSegment"][];
            /** @description Free-text label for an exception day — holiday name, "early close", "outage"; null on a plain regular day. */
            reason: string | null;
        };
        TradingDaysResponse: {
            mic: string;
            days: components["schemas"]["TradingDay"][];
        };
        /** @description A registered trading venue. `default_segments` is the regular-day shape used to seed new session rows; reads of a given date use that day's session row. */
        TradingVenue: {
            mic: string;
            name: string;
            timezone: string;
            country: string;
            currency: string;
            default_segments: components["schemas"]["SessionSegment"][];
            active: boolean;
        };
        TradingVenuesResponse: {
            venues: components["schemas"]["TradingVenue"][];
        };
    };
    responses: never;
    parameters: never;
    requestBodies: never;
    headers: never;
    pathItems: never;
}
export type $defs = Record<string, never>;
export interface operations {
    getApiV1CalendarsByMicDaysByDate: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                /** @description ISO 10383 MIC of the venue, case-insensitive (e.g. `XNAS`). List them with `GET /api/v1/calendars`. */
                mic: string;
                /** @description Calendar date as ISO `YYYY-MM-DD`, in the venue's own timezone. */
                date: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["TradingDay"];
                };
            };
            /** @description The MIC is malformed, or the date is not a valid ISO `YYYY-MM-DD`. */
            400: {
                headers: {
                    /** @description Request correlation identifier for support and diagnostics. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["OriginProblemDetails"];
                };
            };
            /** @description No venue is registered under that MIC, or the venue has no entered coverage for that date. */
            404: {
                headers: {
                    /** @description Request correlation identifier for support and diagnostics. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["OriginProblemDetails"];
                };
            };
            /** @description Rate limited. `Retry-After` carries the seconds to wait. */
            429: {
                headers: {
                    /** @description Request correlation identifier for support and diagnostics. */
                    "X-Request-ID"?: string;
                    /** @description Seconds to wait before retrying the request. */
                    "Retry-After"?: number;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["OriginProblemDetails"];
                    "application/json": components["schemas"]["EdgeProblem"];
                };
            };
            /** @description Unexpected server-side failure. Do not retry automatically; retain the request ID and contact support. */
            500: {
                headers: {
                    /** @description Request correlation identifier for support and diagnostics. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["OriginProblemDetails"];
                };
            };
        };
    };
    getApiV1CalendarsByMicDays: {
        parameters: {
            query: {
                /** @description First date of the range, inclusive, as ISO `YYYY-MM-DD`. */
                from: string;
                /** @description Last date of the range, inclusive, as ISO `YYYY-MM-DD`. Must be on or after `from`, and at most 366 days later. */
                to: string;
            };
            header?: never;
            path: {
                /** @description ISO 10383 MIC of the venue, case-insensitive (e.g. `XNAS`). List them with `GET /api/v1/calendars`. */
                mic: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["TradingDaysResponse"];
                };
            };
            /** @description The MIC is malformed. */
            400: {
                headers: {
                    /** @description Request correlation identifier for support and diagnostics. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["OriginProblemDetails"];
                };
            };
            /** @description No venue is registered under that MIC. */
            404: {
                headers: {
                    /** @description Request correlation identifier for support and diagnostics. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["OriginProblemDetails"];
                };
            };
            /** @description `from`/`to` are not valid ISO dates, `from` is after `to`, or the range exceeds 366 days. */
            422: {
                headers: {
                    /** @description Request correlation identifier for support and diagnostics. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["OriginProblemDetails"];
                };
            };
            /** @description Rate limited. `Retry-After` carries the seconds to wait. */
            429: {
                headers: {
                    /** @description Request correlation identifier for support and diagnostics. */
                    "X-Request-ID"?: string;
                    /** @description Seconds to wait before retrying the request. */
                    "Retry-After"?: number;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["OriginProblemDetails"];
                    "application/json": components["schemas"]["EdgeProblem"];
                };
            };
            /** @description Unexpected server-side failure. Do not retry automatically; retain the request ID and contact support. */
            500: {
                headers: {
                    /** @description Request correlation identifier for support and diagnostics. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["OriginProblemDetails"];
                };
            };
        };
    };
    getApiV1CalendarsByMicNextSession: {
        parameters: {
            query: {
                /** @description Date to search forward from, inclusive, as ISO `YYYY-MM-DD`. */
                from: string;
            };
            header?: never;
            path: {
                /** @description ISO 10383 MIC of the venue, case-insensitive (e.g. `XNAS`). List them with `GET /api/v1/calendars`. */
                mic: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["NextSessionResponse"];
                };
            };
            /** @description The MIC is malformed. */
            400: {
                headers: {
                    /** @description Request correlation identifier for support and diagnostics. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["OriginProblemDetails"];
                };
            };
            /** @description No venue is registered under that MIC. */
            404: {
                headers: {
                    /** @description Request correlation identifier for support and diagnostics. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["OriginProblemDetails"];
                };
            };
            /** @description `from` is not a valid ISO date. */
            422: {
                headers: {
                    /** @description Request correlation identifier for support and diagnostics. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["OriginProblemDetails"];
                };
            };
            /** @description Rate limited. `Retry-After` carries the seconds to wait. */
            429: {
                headers: {
                    /** @description Request correlation identifier for support and diagnostics. */
                    "X-Request-ID"?: string;
                    /** @description Seconds to wait before retrying the request. */
                    "Retry-After"?: number;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["OriginProblemDetails"];
                    "application/json": components["schemas"]["EdgeProblem"];
                };
            };
            /** @description Unexpected server-side failure. Do not retry automatically; retain the request ID and contact support. */
            500: {
                headers: {
                    /** @description Request correlation identifier for support and diagnostics. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["OriginProblemDetails"];
                };
            };
        };
    };
    getApiV1Calendars: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["TradingVenuesResponse"];
                };
            };
            /** @description Rate limited. `Retry-After` carries the seconds to wait. */
            429: {
                headers: {
                    /** @description Request correlation identifier for support and diagnostics. */
                    "X-Request-ID"?: string;
                    /** @description Seconds to wait before retrying the request. */
                    "Retry-After"?: number;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["OriginProblemDetails"];
                    "application/json": components["schemas"]["EdgeProblem"];
                };
            };
            /** @description Unexpected server-side failure. Do not retry automatically; retain the request ID and contact support. */
            500: {
                headers: {
                    /** @description Request correlation identifier for support and diagnostics. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["OriginProblemDetails"];
                };
            };
        };
    };
    getApiV1CalendarsByMic: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                /** @description ISO 10383 MIC of the venue, case-insensitive (e.g. `XNAS`). List them with `GET /api/v1/calendars`. */
                mic: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["TradingVenue"];
                };
            };
            /** @description The MIC is not a well-formed ISO 10383 code. */
            400: {
                headers: {
                    /** @description Request correlation identifier for support and diagnostics. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["OriginProblemDetails"];
                };
            };
            /** @description No venue is registered under that MIC. */
            404: {
                headers: {
                    /** @description Request correlation identifier for support and diagnostics. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["OriginProblemDetails"];
                };
            };
            /** @description Rate limited. `Retry-After` carries the seconds to wait. */
            429: {
                headers: {
                    /** @description Request correlation identifier for support and diagnostics. */
                    "X-Request-ID"?: string;
                    /** @description Seconds to wait before retrying the request. */
                    "Retry-After"?: number;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["OriginProblemDetails"];
                    "application/json": components["schemas"]["EdgeProblem"];
                };
            };
            /** @description Unexpected server-side failure. Do not retry automatically; retain the request ID and contact support. */
            500: {
                headers: {
                    /** @description Request correlation identifier for support and diagnostics. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["OriginProblemDetails"];
                };
            };
        };
    };
    getApiV1CategoriesBySlug: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                /** @description Category slug. */
                slug: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["CategoryResource"];
                };
            };
            /** @description No active category has that slug. */
            404: {
                headers: {
                    /** @description Request correlation identifier for support and diagnostics. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["OriginProblemDetails"];
                };
            };
            /** @description Rate limited. `Retry-After` carries the seconds to wait. */
            429: {
                headers: {
                    /** @description Request correlation identifier for support and diagnostics. */
                    "X-Request-ID"?: string;
                    /** @description Seconds to wait before retrying the request. */
                    "Retry-After"?: number;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["OriginProblemDetails"];
                    "application/json": components["schemas"]["EdgeProblem"];
                };
            };
            /** @description Unexpected server-side failure. Do not retry automatically; retain the request ID and contact support. */
            500: {
                headers: {
                    /** @description Request correlation identifier for support and diagnostics. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["OriginProblemDetails"];
                };
            };
        };
    };
    getApiV1EventsBySlugCanonicalUrl: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                /** @description Event slug. */
                slug: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["EventCanonicalUrlResponse"];
                };
            };
            /** @description No event has that slug. */
            404: {
                headers: {
                    /** @description Request correlation identifier for support and diagnostics. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["OriginProblemDetails"];
                };
            };
            /** @description Rate limited. `Retry-After` carries the seconds to wait. */
            429: {
                headers: {
                    /** @description Request correlation identifier for support and diagnostics. */
                    "X-Request-ID"?: string;
                    /** @description Seconds to wait before retrying the request. */
                    "Retry-After"?: number;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["OriginProblemDetails"];
                    "application/json": components["schemas"]["EdgeProblem"];
                };
            };
            /** @description Unexpected server-side failure. Do not retry automatically; retain the request ID and contact support. */
            500: {
                headers: {
                    /** @description Request correlation identifier for support and diagnostics. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["OriginProblemDetails"];
                };
            };
        };
    };
    getApiV1EventsBySlug: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                /** @description Event slug (any event in the group; resolves to the group root). */
                slug: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["EventDetail"];
                };
            };
            /** @description No event has that slug. */
            404: {
                headers: {
                    /** @description Request correlation identifier for support and diagnostics. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["OriginProblemDetails"];
                };
            };
            /** @description Rate limited. `Retry-After` carries the seconds to wait. */
            429: {
                headers: {
                    /** @description Request correlation identifier for support and diagnostics. */
                    "X-Request-ID"?: string;
                    /** @description Seconds to wait before retrying the request. */
                    "Retry-After"?: number;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["OriginProblemDetails"];
                    "application/json": components["schemas"]["EdgeProblem"];
                };
            };
            /** @description Unexpected server-side failure. Do not retry automatically; retain the request ID and contact support. */
            500: {
                headers: {
                    /** @description Request correlation identifier for support and diagnostics. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["OriginProblemDetails"];
                };
            };
        };
    };
    getApiV1Events: {
        parameters: {
            query?: {
                /** @description Category slug (root or leaf). Descendants are included. Omit for an unscoped global listing. */
                category?: string;
                /** @description Root slug the request originated under, used to disambiguate a category slug that exists in more than one tree. It is part of the cursor fingerprint, so keep it identical while paging. */
                root?: string;
                /** @description Narrow by event-type bucket. */
                event_type_bucket?: "games" | "props";
                /** @description Named filter key (e.g. `sports_live`, `sports_futures`). */
                filter?: string;
                /** @description Filter catalogue results to the AGARA exchange. */
                source?: "AGARA";
                /** @description Drop events whose window has already ended, and events with no market left to trade. A RECURRING_PROPOSITION is retained while its next cycle is still PROVISIONED, which does not accept orders, so check market.state before placing one. Defaults to false. */
                exclude_ended?: "true" | "false";
                /** @description Filter by resolution status. `resolved` covers both RESOLVED and VOID; `active` matches not-yet-resolved events. Omit (or `all`) for no filter. */
                resolution?: "all" | "active" | "proposed" | "disputed" | "resolved";
                /** @description Order within the lifecycle buckets (LIVE, ACTIVE, ENDED, PROPOSED, DISPUTED, CLOSED), all descending: `time`, `volume`, or `markets` (pooled market count). Omit for the default time-based bucketing. */
                sort?: "time" | "volume" | "markets";
                /** @description Opaque keyset cursor returned by a previous page. */
                cursor?: string;
                /** @description Page size. Default 20, max 100. */
                limit?: number;
                /** @description Opt into lean `listing_markets` enrichment on non-GAME items (PROPOSITION / RECURRING_PROPOSITION main market; MULTI_OUTCOME top-2 candidates). Defaults to false. */
                include_markets?: "true" | "false";
            };
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["EventsListResponse"];
                };
            };
            /** @description No active category has the requested `category` slug. */
            404: {
                headers: {
                    /** @description Request correlation identifier for support and diagnostics. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["OriginProblemDetails"];
                };
            };
            /** @description A query parameter failed validation, `filter` is not a known key, or the `cursor` does not match the requested sort and filters — drop it and start from the first page. */
            422: {
                headers: {
                    /** @description Request correlation identifier for support and diagnostics. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["OriginProblemDetails"];
                };
            };
            /** @description Rate limited. `Retry-After` carries the seconds to wait. */
            429: {
                headers: {
                    /** @description Request correlation identifier for support and diagnostics. */
                    "X-Request-ID"?: string;
                    /** @description Seconds to wait before retrying the request. */
                    "Retry-After"?: number;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["OriginProblemDetails"];
                    "application/json": components["schemas"]["EdgeProblem"];
                };
            };
            /** @description Unexpected server-side failure. Do not retry automatically; retain the request ID and contact support. */
            500: {
                headers: {
                    /** @description Request correlation identifier for support and diagnostics. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["OriginProblemDetails"];
                };
            };
        };
    };
    getApiV1MarketsById: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                /** @description Market UUID (agara_markets.id). */
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["MarketDetail"];
                };
            };
            /** @description The market id is not a UUID. */
            400: {
                headers: {
                    /** @description Request correlation identifier for support and diagnostics. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["OriginProblemDetails"];
                };
            };
            /** @description No market has that id. */
            404: {
                headers: {
                    /** @description Request correlation identifier for support and diagnostics. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["OriginProblemDetails"];
                };
            };
            /** @description Rate limited. `Retry-After` carries the seconds to wait. */
            429: {
                headers: {
                    /** @description Request correlation identifier for support and diagnostics. */
                    "X-Request-ID"?: string;
                    /** @description Seconds to wait before retrying the request. */
                    "Retry-After"?: number;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["OriginProblemDetails"];
                    "application/json": components["schemas"]["EdgeProblem"];
                };
            };
            /** @description Unexpected server-side failure. Do not retry automatically; retain the request ID and contact support. */
            500: {
                headers: {
                    /** @description Request correlation identifier for support and diagnostics. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["OriginProblemDetails"];
                };
            };
        };
    };
    getApiV1Markets: {
        parameters: {
            query: {
                /** @description Exchange/source filter. Case-insensitive. */
                source: "agara";
                /** @description Narrow to markets attached to a single event slug. */
                event_slug?: string;
                /** @description Restrict to a single market lifecycle state (e.g. ACTIVE for only markets accepting orders). Case-insensitive. */
                state?: "DRAFT" | "PROVISIONED" | "ACTIVE" | "INACTIVE" | "TRADING_HALT" | "PROPOSAL_PENDING" | "PROPOSED" | "DISPUTED" | "RESOLVED" | "VOID" | "ARCHIVED";
                /** @description Opaque keyset cursor returned by a previous page. */
                cursor?: string;
                /** @description Page size. Default 64, max 256. */
                limit?: number;
            };
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["MarketsListResponse"];
                };
            };
            /** @description A query parameter failed validation, or the `cursor` does not match the requested filters — drop it and start from the first page. */
            422: {
                headers: {
                    /** @description Request correlation identifier for support and diagnostics. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["OriginProblemDetails"];
                };
            };
            /** @description Rate limited. `Retry-After` carries the seconds to wait. */
            429: {
                headers: {
                    /** @description Request correlation identifier for support and diagnostics. */
                    "X-Request-ID"?: string;
                    /** @description Seconds to wait before retrying the request. */
                    "Retry-After"?: number;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["OriginProblemDetails"];
                    "application/json": components["schemas"]["EdgeProblem"];
                };
            };
            /** @description Unexpected server-side failure. Do not retry automatically; retain the request ID and contact support. */
            500: {
                headers: {
                    /** @description Request correlation identifier for support and diagnostics. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["OriginProblemDetails"];
                };
            };
        };
    };
    getApiV1PricesPoint: {
        parameters: {
            query: {
                /** @description agara_securities.symbol, e.g. BTC-USD. */
                symbol: string;
                /** @description agara_data_providers.slug, e.g. pyth-pro. */
                provider: string;
                /** @description Pinned instant, unix seconds. */
                at: number;
            };
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["PricePointBody"];
                };
            };
            /** @description `symbol`, `provider`, or `at` is invalid, or the security and provider pair is not tracked. */
            422: {
                headers: {
                    /** @description Request correlation identifier for support and diagnostics. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["OriginProblemDetails"];
                };
            };
            /** @description Rate limited. `Retry-After` carries the seconds to wait. */
            429: {
                headers: {
                    /** @description Request correlation identifier for support and diagnostics. */
                    "X-Request-ID"?: string;
                    /** @description Seconds to wait before retrying the request. */
                    "Retry-After"?: number;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["OriginProblemDetails"];
                    "application/json": components["schemas"]["EdgeProblem"];
                };
            };
            /** @description Unexpected server-side failure. Do not retry automatically; retain the request ID and contact support. */
            500: {
                headers: {
                    /** @description Request correlation identifier for support and diagnostics. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["OriginProblemDetails"];
                };
            };
            /** @description The provider rejected credentials or returned an invalid response. */
            502: {
                headers: {
                    /** @description Request correlation identifier for support and diagnostics. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["OriginProblemDetails"];
                };
            };
            /** @description That second is not yet available upstream. Retry. */
            503: {
                headers: {
                    /** @description Request correlation identifier for support and diagnostics. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["OriginProblemDetails"];
                };
            };
        };
    };
    getApiV1PricesTicks: {
        parameters: {
            query: {
                /** @description agara_securities.symbol (MarketDetail display.security_symbol). */
                symbol: string;
                /** @description agara_data_providers.slug (MarketDetail display.security_provider). */
                provider: string;
                /** @description Range start, inclusive, unix seconds. */
                from: number;
                /** @description Range end, inclusive, unix seconds. */
                to: number;
            };
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ShimHistory"];
                };
            };
            /** @description `symbol` or `provider` is malformed, `from`/`to` are not unix seconds with `to >= from`, or the range exceeds 86400 seconds (24 hours). */
            422: {
                headers: {
                    /** @description Request correlation identifier for support and diagnostics. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["OriginProblemDetails"];
                };
            };
            /** @description Rate limited. `Retry-After` carries the seconds to wait. */
            429: {
                headers: {
                    /** @description Request correlation identifier for support and diagnostics. */
                    "X-Request-ID"?: string;
                    /** @description Seconds to wait before retrying the request. */
                    "Retry-After"?: number;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["OriginProblemDetails"];
                    "application/json": components["schemas"]["EdgeProblem"];
                };
            };
            /** @description Unexpected server-side failure. Do not retry automatically; retain the request ID and contact support. */
            500: {
                headers: {
                    /** @description Request correlation identifier for support and diagnostics. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["OriginProblemDetails"];
                };
            };
        };
    };
    getApiV1PricesTokenHistory: {
        parameters: {
            query: {
                /** @description Outcome token id (agara_market_outcomes.token_id). */
                token_id: string;
                /** @description Window preset; defaults to 1d. */
                range?: "1h" | "6h" | "1d" | "7d" | "30d" | "all";
                /** @description Number of sampled points; defaults to 400. */
                points?: number;
            };
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["TokenHistoryResponse"];
                };
            };
            /** @description No outcome has that token id, or its market is missing. */
            404: {
                headers: {
                    /** @description Request correlation identifier for support and diagnostics. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["OriginProblemDetails"];
                };
            };
            /** @description `token_id` is missing, or `range` is invalid. */
            422: {
                headers: {
                    /** @description Request correlation identifier for support and diagnostics. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["OriginProblemDetails"];
                };
            };
            /** @description Rate limited. `Retry-After` carries the seconds to wait. */
            429: {
                headers: {
                    /** @description Request correlation identifier for support and diagnostics. */
                    "X-Request-ID"?: string;
                    /** @description Seconds to wait before retrying the request. */
                    "Retry-After"?: number;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["OriginProblemDetails"];
                    "application/json": components["schemas"]["EdgeProblem"];
                };
            };
            /** @description Unexpected server-side failure. Do not retry automatically; retain the request ID and contact support. */
            500: {
                headers: {
                    /** @description Request correlation identifier for support and diagnostics. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["OriginProblemDetails"];
                };
            };
        };
    };
    getApiV1Search: {
        parameters: {
            query: {
                /** @description Search text. Minimum 2 characters. */
                q: string;
                /** @description Max results per list (events, markets). Default 10, max 25. */
                limit?: number;
                /** @description Filter catalogue results to the AGARA exchange. */
                source?: "AGARA";
            };
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["SearchResponse"];
                };
            };
            /** @description `q` is shorter than 2 or longer than 128 characters, or `limit` is outside 1 to 25. */
            422: {
                headers: {
                    /** @description Request correlation identifier for support and diagnostics. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["OriginProblemDetails"];
                };
            };
            /** @description Rate limited. `Retry-After` carries the seconds to wait. */
            429: {
                headers: {
                    /** @description Request correlation identifier for support and diagnostics. */
                    "X-Request-ID"?: string;
                    /** @description Seconds to wait before retrying the request. */
                    "Retry-After"?: number;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["OriginProblemDetails"];
                    "application/json": components["schemas"]["EdgeProblem"];
                };
            };
            /** @description Unexpected server-side failure. Do not retry automatically; retain the request ID and contact support. */
            500: {
                headers: {
                    /** @description Request correlation identifier for support and diagnostics. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["OriginProblemDetails"];
                };
            };
        };
    };
    getApiV1Securities: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["SecuritiesResponse"];
                };
            };
            /** @description Rate limited. `Retry-After` carries the seconds to wait. */
            429: {
                headers: {
                    /** @description Request correlation identifier for support and diagnostics. */
                    "X-Request-ID"?: string;
                    /** @description Seconds to wait before retrying the request. */
                    "Retry-After"?: number;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["OriginProblemDetails"];
                    "application/json": components["schemas"]["EdgeProblem"];
                };
            };
            /** @description Unexpected server-side failure. Do not retry automatically; retain the request ID and contact support. */
            500: {
                headers: {
                    /** @description Request correlation identifier for support and diagnostics. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["OriginProblemDetails"];
                };
            };
        };
    };
    getApiV1SecuritiesBySymbol: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                /** @description Canonical security symbol, case-insensitive (e.g. `BTC-USD`). List them with `GET /api/v1/securities`. */
                symbol: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Security"];
                };
            };
            /** @description The symbol is malformed. */
            400: {
                headers: {
                    /** @description Request correlation identifier for support and diagnostics. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["OriginProblemDetails"];
                };
            };
            /** @description No active security has that symbol. */
            404: {
                headers: {
                    /** @description Request correlation identifier for support and diagnostics. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["OriginProblemDetails"];
                };
            };
            /** @description Rate limited. `Retry-After` carries the seconds to wait. */
            429: {
                headers: {
                    /** @description Request correlation identifier for support and diagnostics. */
                    "X-Request-ID"?: string;
                    /** @description Seconds to wait before retrying the request. */
                    "Retry-After"?: number;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["OriginProblemDetails"];
                    "application/json": components["schemas"]["EdgeProblem"];
                };
            };
            /** @description Unexpected server-side failure. Do not retry automatically; retain the request ID and contact support. */
            500: {
                headers: {
                    /** @description Request correlation identifier for support and diagnostics. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["OriginProblemDetails"];
                };
            };
        };
    };
}

export const endpointDocs = {
  getStatus: {
    summary: "Read public platform market and event counts.",
    returns: "Current market and event counts.",
    remarks: "These counts do not establish whether any specific market accepts orders.",
  },
  getOrderbook: {
    summary: "Read a public outcome-token orderbook snapshot.",
    returns: "Depth levels in whole-unit decimal price/share numbers, with snapshot metadata.",
    remarks:
      "REST levels use different units from native integer WebSocket books; do not apply stream scales to REST levels.",
    params: {
      token_id: "Outcome token identifier encoded as an unsigned decimal 256-bit integer.",
    },
  },
  placeOrder: {
    summary: "Submit a LIMIT or MARKET order for asynchronous execution.",
    returns: "An acknowledgement containing the internal order UUID and pending operation.",
    remarks:
      "Acceptance is not a fill. Inspect the order is_terminal flag and fill history separately. LIMIT orders use shares; MARKET BUY uses collateral and MARKET SELL uses shares. Submission is sent once and ambiguous outcomes require reconciliation.",
    body: "Order side, time-in-force and exact integer micro amounts; one whole unit is 1000000 micro units.",
  },
  placeSignedOrder: {
    summary: "Submit one pre-signed AGARA LIMIT order.",
    returns: "An asynchronous order acknowledgement; no matching or settlement is guaranteed.",
    remarks:
      "The signature and envelope must describe the same economics. After an uncertain submission, reconcile the existing order_hash with getOrderByHash before deciding whether to submit again.",
    body: "Signed order envelope, exact micro price/shares and matching EIP-712 hash/signature fields.",
  },
  placeSignedOrders: {
    summary: "Submit a batch of independently signed LIMIT orders.",
    returns:
      "Indexed accepted/rejected results, including canonical failure details for rejected entries.",
    remarks:
      "Each result belongs to its input index; inspect all entries even when the HTTP response is successful. Accepted entries still require order-status and fill reconciliation.",
    body: "One through 32 signed order envelopes in input order.",
  },
  listOrders: {
    summary: "Read one page of your order history.",
    returns: "Order rows, market metadata, observation time and an opaque next-page cursor.",
    remarks:
      "This is one page, not a complete history. Keep the cursor opaque and use orderPages for a bounded full walk.",
    body: "Page size and optional cursor returned by the previous response.",
  },
  getOrder: {
    summary: "Read an order by its internal UUID.",
    returns: "The current order record and market metadata.",
    remarks:
      "Only order.is_terminal establishes order completion; a MATCHED label alone does not prove completion or chain settlement.",
    params: {
      order_id: "Internal order UUID returned by order acceptance.",
    },
  },
  getOrderByHash: {
    summary: "Reconcile a signed order using its known EIP-712 digest.",
    returns: "The current order record and market metadata for the hash.",
    remarks:
      "Use the original digest after an ambiguous signed submission. A not-found response may reflect persistence lag; it does not by itself prove that resubmission is safe.",
    params: {
      order_hash: "Original 32-byte signed order digest encoded as 0x-prefixed hex.",
    },
  },
  getOrderTrades: {
    summary: "Read fill legs associated with one order.",
    returns: "Fill records and observation time, retaining execution and transaction metadata.",
    remarks:
      "Read fill and settlement state separately from order acceptance and terminality; amounts retain their documented micro units.",
    params: {
      order_id: "Internal order UUID whose fills should be read.",
    },
  },
  cancelOrder: {
    summary: "Request asynchronous cancellation of one order.",
    returns: "A cancellation acknowledgement with the order UUID and pending operation.",
    remarks:
      "The acknowledgement is not proof that all remaining shares were cancelled. Observe the order is_terminal flag and any intervening fills.",
    params: {
      order_id: "Internal order UUID to cancel.",
    },
  },
  cancelAllOrders: {
    summary: "Schedule cancellation of your current nonterminal orders.",
    returns: "An asynchronous cancel-all acknowledgement.",
    remarks:
      "Matching and fills may race cancellation. Re-read open orders and fills before treating inventory or collateral as released.",
  },
  submitBatch: {
    summary: "Submit a pre-signed account batch of position or withdrawal operations.",
    returns:
      "The accepted batch digest, which identifies the operation for subsequent status reads.",
    remarks:
      "Acceptance does not establish chain execution. Reconcile the returned digest using getBatch or waitForBatch; do not blindly replay an uncertain mutation.",
    body: "Signed SPLIT, MERGE or WITHDRAW operations, account sequence, Unix-second deadline and optional healing digest.",
  },
  getBatch: {
    summary: "Read the lifecycle and reconciliation state of an account batch.",
    returns: "Batch status, origin, execution/unwind timestamps and public failure evidence.",
    remarks:
      "FAILED with a null unwound_at still requires reconciliation. Inspect failure details for FAILED_DIVERGENT rather than treating a terminal label as success.",
    params: {
      batch_hash: "Existing EIP-712 account-batch digest, as 0x-prefixed 32-byte hex.",
    },
  },
  supersedeBatch: {
    summary: "Request replacement of a still-pending account batch.",
    returns: "Either the accepted successor or a refusal carrying the current batch state.",
    remarks:
      "A successful HTTP response may contain a refusal. Branch on the returned result and reconcile the existing digest; this mutation is not automatically retried.",
    body: "Replacement operations, Unix-second deadline and signature bound to the inherited sequence.",
    params: {
      batch_hash: "Digest of the pending batch to replace, as 0x-prefixed 32-byte hex.",
    },
  },
  getBatchGroup: {
    summary: "Read an existing account-batch group and its chunk attempts.",
    returns: "Group metadata, completed_at and per-chunk batch state.",
    remarks:
      "A group read does not create merge-all work. Retain individual chunk states and failures when deciding how to reconcile.",
    params: {
      group_id: "UUID of the existing batch group.",
    },
  },
  listActivities: {
    summary: "Read one page of account activity.",
    returns: "Activity variants with market, condition and event metadata plus an opaque cursor.",
    remarks:
      "Activity includes orders, splits, merges, redemptions, deposits, withdrawals and LP payouts. Use the realized-PnL report for accounting attribution.",
    query: "Page size from 1 to 500 (server default 50) and optional opaque cursor.",
  },
  getBridgeWithdrawAssets: {
    summary: "List supported destination chains and assets for bridge withdrawals.",
    returns:
      "Destination asset metadata, address families, native decimals and minimum checkout values.",
    remarks:
      "Use the provider address family and identifiers when requesting a quote; this method does not withdraw funds.",
  },
  quoteBridgeWithdrawal: {
    summary: "Estimate a cross-chain withdrawal without submitting it.",
    returns: "Estimated output quantity, USD amounts, timing, fees and provider quote identity.",
    remarks:
      "This is a read-only quote. It does not call the browser-authenticated standalone withdrawal route.",
    body: "Destination chain/token, destination-chain recipient address and source amount as an exact integer base-unit string.",
  },
  listOpenOrders: {
    summary: "Read one page of active orders with portfolio metadata.",
    returns:
      "Nonterminal order rows, market/event sidecars, observation time and opaque pagination.",
    remarks:
      "A single page is not the entire open-order set. Use openOrderPages for a bounded walk and allow for state changes between pages.",
    body: "Optional token/exchange filters, page size and previous next_cursor.",
  },
  listPositions: {
    summary: "Read your positions together with exchange-availability diagnostics.",
    returns:
      "The complete response envelope, including positions, market/event sidecars and unavailable_exchanges.",
    remarks:
      "An unavailable exchange must not be treated as empty holdings. Call assertComplete before making decisions that require all requested exchanges. At least one condition identifier is required: the server answers an empty array with an empty envelope, which cannot be told apart from holding nothing, so this client rejects it instead.",
    body: "One or more condition identifiers, and optional exchange filters; omitted filters use server defaults.",
  },
  splitPosition: {
    summary: "Request conversion of collateral into a complete outcome set.",
    returns:
      "An accepted AGARA batch or a venue relayer receipt, discriminated by its response fields.",
    remarks:
      "For a batch_hash response, reconcile getBatch or waitForBatch. Acceptance does not prove minted positions are already spendable.",
    body: "Condition identifier and positive exact collateral amount in integer micro units.",
  },
  mergePosition: {
    summary: "Request conversion of a complete outcome set back into collateral.",
    returns:
      "An accepted AGARA batch or a venue relayer receipt, discriminated by its response fields.",
    remarks:
      "For a batch_hash response, reconcile getBatch or waitForBatch. Acceptance does not prove collateral is already available.",
    body: "Condition identifier and positive exact quantity per outcome leg in integer micro shares.",
  },
  getPortfolioSummary: {
    summary: "Read collateral and position valuations for the requested trading wallet.",
    returns:
      "Per-exchange cash balance, free cash and portfolio valuations in documented micro units.",
    remarks:
      "Keep exchange collateral pools separate; summary values do not authorize an order beyond current server balance checks.",
    query: "Optional AGARA exchange selection; omission uses the server trading-wallet default.",
  },
  listTrades: {
    summary: "Read one page of your fill history.",
    returns: "Fill legs, market/event sidecars, availability diagnostics and an opaque cursor.",
    remarks:
      "Use tradePages to walk further pages. Check unavailable_exchanges before treating this history as complete.",
    query: "Page size from 1 to 500 (server default 500) and optional opaque next_cursor.",
  },
  getRebates: {
    summary: "Read pending maker, VIP and LP incentive balances.",
    returns: "Pending incentive amounts in integer micro collateral.",
    remarks:
      "Pending rebates are not spendable trading collateral until the server reports the corresponding credit.",
  },
  listLpIncentives: {
    summary: "Read active/upcoming LP opportunities and optional personal standing.",
    returns: "Current reward epoch, incentive terms and per-market scores/projected payouts.",
    remarks:
      "Anonymous callers can read opportunities. A PAT with portfolio:read adds the caller standing; projections are not guaranteed credited rewards.",
    query:
      "Optional category/search and sort field/direction; sort_order requires sort_by and otherwise defaults to descending when sorting.",
  },
  listLpIncentiveCategories: {
    summary: "List root categories containing current LP-incentive opportunities.",
    returns: "Category slugs, labels and matching-market counts.",
    remarks:
      "Use these slugs to filter listLpIncentives; categories may change as opportunities open and close.",
  },
  listClosedLpIncentives: {
    summary: "Read one page of your historical LP reward cycles.",
    returns: "Credited-cycle market rows, applied page/limit and total matching-row count.",
    remarks:
      "Use 1-based pagination rather than cursor helpers. Potential payout and credited reward are separate amounts in micro collateral.",
    query:
      "Optional category/search; page starts at 1, limit is 1 through 100 (default 20), sort defaults to date descending.",
  },
  listClosedLpIncentiveCategories: {
    summary: "List categories represented in your historical LP rewards.",
    returns: "Root-category slugs and labels available for filtering credited reward cycles.",
    remarks:
      "These are caller-specific historical categories, distinct from current public incentive opportunities.",
  },
  getLpIncentiveEarnings: {
    summary: "Read current projected and trailing LP reward totals.",
    returns: "Current-cycle, seven-day, thirty-day and all-time earnings in micro collateral.",
    remarks:
      "The current cycle is projected before minimum payout rules; trailing windows combine that projection with prior credited rewards.",
  },
  getRealizedPnl: {
    summary: "Read exact realized-PnL attribution over a UTC calendar window.",
    returns:
      "Category totals, buckets, running totals and provenance; monetary values are decimal USDC strings.",
    remarks:
      "amountScale describes fractional precision, not a divisor to apply to the returned decimal USDC strings. Use week/all or hour/day with 1d, 7d or 30d.",
    query: "Required granularity and window: hour/day with 1d, 7d or 30d, or week with all.",
  },
  getTradingDay: {
    summary: "Read one venue-local calendar day and its trading sessions.",
    returns: "Calendar date, trading/closed state and any session segments.",
    remarks: "A missing covered day is unavailable data, not evidence that the venue is closed.",
    params: {
      mic: "Case-insensitive ISO 10383 venue MIC, such as XNAS; discover supported codes with listCalendars.",
      date: "Real calendar date in YYYY-MM-DD form, interpreted in the venue timezone.",
    },
  },
  listTradingDays: {
    summary: "Read an inclusive range of venue-local trading days.",
    returns: "Trading-day records and session segments for the requested covered range.",
    remarks:
      "Use real dates in the venue timezone. Missing coverage must not be inferred to mean a closed session.",
    query:
      "Inclusive YYYY-MM-DD from/to dates spanning at most 366 calendar days, with to on or after from.",
    params: {
      mic: "Case-insensitive ISO 10383 venue MIC, such as XNAS; discover supported codes with listCalendars.",
    },
  },
  getNextSession: {
    summary: "Find the next covered trading session from a venue-local date.",
    returns: "The next session when available, including its trading day and boundaries.",
    remarks:
      "No returned session can mean the requested future range is not covered; it does not establish indefinite closure.",
    query: "Inclusive venue-local starting date as YYYY-MM-DD.",
    params: {
      mic: "Case-insensitive ISO 10383 venue MIC, such as XNAS; discover supported codes with listCalendars.",
    },
  },
  listCalendars: {
    summary: "List registered trading venues and their regular session templates.",
    returns: "Venue MICs, names, timezones, currencies, active flags and regular session segments.",
    remarks:
      "Use each returned MIC and timezone for date-specific trading-day/session lookups; regular templates do not describe holiday exceptions.",
  },
  getCalendar: {
    summary: "Read metadata for a supported trading venue.",
    returns: "Venue identity, timezone, currency, active flag and regular session segments.",
    remarks:
      "Query the trading-day/session methods for date-specific market hours and holiday exceptions.",
    params: {
      mic: "Case-insensitive ISO 10383 venue MIC, such as XNAS; discover supported codes with listCalendars.",
    },
  },
  getCategory: {
    summary: "Read a catalogue category and its event-family metadata.",
    returns: "Category identity and available event-type information.",
    remarks:
      "Use the canonical returned category identity when composing subsequent discovery filters.",
    params: {
      slug: "Catalogue category slug.",
    },
  },
  getEventCanonicalUrl: {
    summary: "Resolve an event slug to its canonical catalogue identity and URL.",
    returns: "Canonical event slug and an origin-relative URL when one is available.",
    remarks:
      "A child event may resolve to a group root. The URL may be null, so preserve the canonical slug independently.",
    params: {
      slug: "Event slug, including a child slug that may resolve to its group root.",
    },
  },
  getEvent: {
    summary: "Read event details and the associated market groups.",
    returns:
      "The event-type-specific catalogue response, preserving display, lifecycle and market metadata.",
    remarks:
      "A child slug resolves to its group root. Discovery state and configured markets should be checked before trading.",
    params: {
      slug: "Event slug, including any child in the requested event group.",
    },
  },
  listEvents: {
    summary: "Read one filtered catalogue event page.",
    returns: "Event listing items, optional market enrichment and opaque pagination.",
    remarks:
      "Keep category/root and other filters stable across cursor pages. An upcoming market in discovery may still be PROVISIONED and unable to accept orders.",
    query:
      "Category/root, event bucket, source, named filter, lifecycle/sort filters, optional market enrichment, limit (default 20, maximum 100) and opaque cursor.",
  },
  getMarket: {
    summary: "Read a market and its outcome identities and trading configuration.",
    returns:
      "Market lifecycle, outcomes, engine scales/limits and resolution metadata where available.",
    remarks:
      "Use outcome token IDs for trading and the returned scales for native stream values; the market UUID and condition identifier serve different purposes.",
    params: {
      id: "Catalogue market UUID returned by market/event discovery.",
    },
  },
  listMarkets: {
    summary: "Read one filtered AGARA market page.",
    returns: "Market listing items, event references and opaque pagination.",
    remarks:
      "Keep filters stable when following next_cursor; one page is not a complete catalogue.",
    query:
      "Required AGARA source plus optional event_slug, lifecycle state, cursor and limit (default 64, maximum 256).",
  },
  getPricePoint: {
    summary: "Read a provider price pinned to a past Unix second.",
    returns: "The available price observation and provider timing metadata.",
    remarks:
      "`symbol` is an Agara security symbol such as `BTC-USD`, listed by `listSecurities`. `provider` is a data-provider slug, lowercase and hyphenated, such as `pyth-pro`; an uppercase value like `PYTH` is rejected with 422. No method here returns the slug, so take it from configuration. Both differ from the provider symbols the price streams take, such as `Crypto.BTC/USD`. The requested second and the actual observation time are distinct.",
    query:
      "Agara security symbol (see `listSecurities`), a provider slug matching `^[a-z0-9-]{1,32}$` such as `pyth-pro`, and the requested at timestamp in Unix seconds.",
  },
  getPriceTicks: {
    summary: "Read provider price ticks over an inclusive historical interval.",
    returns: "Price observations and timestamps for the requested history window.",
    remarks:
      "Takes the same identifiers as `getPricePoint`: an Agara security symbol such as `BTC-USD` and a lowercase provider slug such as `pyth-pro`, not a provider symbol like `Crypto.BTC/USD`. Preserve exact provider price representations; missing observations do not imply a zero price.",
    query:
      "Agara security symbol, a provider slug matching `^[a-z0-9-]{1,32}$` such as `pyth-pro`, and inclusive from/to Unix seconds over a window of at most 24 hours.",
  },
  getTokenHistory: {
    summary: "Read sampled price history for a market outcome token.",
    returns: "Outcome price/probability observations with timestamps.",
    remarks:
      "This is outcome-token market history, distinct from the provider security feed used to resolve a market.",
    query:
      "Decimal outcome token ID; optional range (1h, 6h, 1d, 7d, 30d or all; default 1d) and points (2 through 1000; default 400).",
  },
  search: {
    summary: "Search catalogue events, markets and categories.",
    returns: "Matching result families with relevance and canonical identities.",
    remarks: "Search returns bounded results rather than a cursor-walkable full catalogue.",
    query:
      "Search text of 2 through 128 UTF-16 code units, optional AGARA source and per-family limit (default 10, maximum 25).",
  },
  listSecurities: {
    summary: "List canonical securities and their listing venues.",
    returns: "Security symbols, labels, venue MICs, asset classes and active flags.",
    remarks:
      "Use these canonical security symbols for price lookup; obtain the configured provider separately from market discovery metadata.",
  },
  getSecurity: {
    summary: "Read one canonical security and its listing-venue metadata.",
    returns: "The security symbol, label, venue MIC, asset class and active flag.",
    remarks:
      "Use the venue MIC for calendar lookups and the canonical symbol for price requests; obtain the configured price provider separately from market discovery metadata.",
    params: {
      symbol:
        "Case-insensitive canonical security symbol, such as BTC-USD; discover symbols with listSecurities.",
    },
  },
};

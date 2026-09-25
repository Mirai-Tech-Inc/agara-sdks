import type { PublicFailure } from "./errors.js";

/** Exact JSON integer: safe values decode as numbers and larger values as bigint. */
export type WireInteger = number | bigint;

/** Router WebSocket channels, with subjects selected by {@link ChannelSpec}. */
export type ChannelName =
  | "orderbook"
  | "best_quote"
  | "trades"
  | "market_status"
  | "account_events";

/** A subscription for a market subject or the authenticated account. */
export type ChannelSpec =
  | {
      /** Subscribe to orderbook snapshots and price-level changes. */
      name: "orderbook";
      /** Outcome token whose orderbook is requested. */
      token_id: string;
      /** Optional depth hint; the server may return full depth. */
      depth?: number;
    }
  | {
      /** Subscribe to the best bid and ask. */
      name: "best_quote";
      /** Outcome token whose quotes are requested. */
      token_id: string;
    }
  | {
      /** Subscribe to live fills across a market's outcomes. */
      name: "trades";
      /** Condition identifier of the market. */
      condition_id: string;
    }
  | {
      /** Subscribe to one market's status changes. */
      name: "market_status";
      /** Condition identifier of the market. */
      condition_id: string;
      /** Event scope is mutually exclusive with condition scope. */
      event_id?: never;
    }
  | {
      /** Subscribe to an event's current-market and resolution changes. */
      name: "market_status";
      /** Event identifier whose market lifecycle is requested. */
      event_id: string;
      /** Condition scope is mutually exclusive with event scope. */
      condition_id?: never;
    }
  | {
      /** Subscribe to the authenticated account's live activity. */
      name: "account_events";
      /** Bearer credential sent in the account subscription frame. */
      token: string;
    };

/** JSON commands accepted by the router WebSocket endpoints. */
export type ClientMessage =
  | {
      /** Add or remove the specified channel subscriptions. */
      op: "subscribe" | "unsubscribe";
      /** Subscriptions to add or remove. */
      channels: ChannelSpec[];
    }
  | {
      /** Request a pong or the active subscription list. */
      op: "ping" | "list";
    };

/** Channel identity; account subjects have no token, condition, or event identifier. */
export interface Subject {
  /** Channel carrying the subject. */
  channel: ChannelName;
  /** Outcome identifier for orderbooks and best quotes. */
  token_id?: string;
  /** Market identifier for trades and condition-scoped status. */
  condition_id?: string;
  /** Event identifier for event-scoped lifecycle updates. */
  event_id?: string;
}

/** Divisors for converting wire prices to collateral units and sizes to shares. */
export interface Scales {
  /** Divide an integer price by this value to obtain its collateral price per share. */
  price_scale: number;
  /** Divide an integer size by this value to obtain its share quantity. */
  size_scale: number;
}

/** Integer price and size; delta size zero removes the level. Apply the enclosing scales. */
export type BookLevel = [price: number, size: WireInteger];

/** Replace the book on snapshots; deltas replace or remove the listed price levels. */
export type OrderbookData = Scales &
  (
    | {
        /** Complete book state at the frame's sequence. */
        kind: "snapshot";
        /** Bid levels in scaled price and share units. */
        bids: BookLevel[];
        /** Ask levels in scaled price and share units. */
        asks: BookLevel[];
        /** Minimum price increment in units of `price_scale`. */
        tick_size: number;
      }
    | {
        /** Changes to apply to a previously received snapshot. */
        kind: "delta";
        /** Replacement bid sizes; zero removes the price level. */
        bids: BookLevel[];
        /** Replacement ask sizes; zero removes the price level. */
        asks: BookLevel[];
      }
  );

/** Current best bid and ask, including empty sides. */
export interface BestQuoteData extends Scales {
  /** Best bid, or null when no bids are available. */
  bid: {
    /** Bid price in units of `price_scale`. */
    price: number;
    /** Available shares in units of `size_scale`. */
    size: WireInteger;
  } | null;
  /** Best ask, or null when no asks are available. */
  ask: {
    /** Ask price in units of `price_scale`. */
    price: number;
    /** Available shares in units of `size_scale`. */
    size: WireInteger;
  } | null;
}

/** Buy or sell direction; `UNSPECIFIED` is the wire fallback when no side is supplied. */
export type TradeSide = "BUY" | "SELL" | "UNSPECIFIED";

/** Normal transfer, cross-outcome mint or merge, or an unspecified settlement mode. */
export type SettlementMode = "NORMAL" | "MINT" | "MERGE" | "UNSPECIFIED";

/** A live market fill, expressed from the taker's perspective. */
export interface TradeData extends Scales {
  /** Identifies a market fill. */
  kind: "trade";
  /** Unique fill identifier. */
  fill_id: string;
  /** Outcome token traded by the taker. */
  taker_token_id: string;
  /** Outcome token traded by the maker. */
  maker_token_id: string;
  /** Taker's buy or sell direction. */
  side: TradeSide;
  /** Taker's execution price in units of `price_scale`. */
  price: number;
  /** Filled shares in units of `size_scale`. */
  size: WireInteger;
  /** How the matched outcomes are settled. */
  settlement_mode: SettlementMode;
}

/** Status changes for a condition-scoped market subscription. */
export type MarketStatusData =
  | {
      /** Trading has been halted. */
      kind: "market_halted";
    }
  | {
      /** Trading has resumed. */
      kind: "market_resumed";
    }
  | {
      /** The market has resolved to a winning outcome. */
      kind: "market_resolved";
      /** Token of the winning outcome. */
      winning_token_id: string;
    }
  | {
      /** An outcome has been proposed but is not yet final. */
      kind: "outcome_proposed";
      /** Token of the proposed winning outcome. */
      proposed_token_id: string;
    }
  | ({
      /** The market is available with its initial trading configuration. */
      kind: "market_created";
      /** Number of outcomes in the market. */
      num_outcomes: number;
      /** Minimum price increment in units of `price_scale`. */
      tick_size: number;
      /** Lowest allowed price in units of `price_scale`. */
      min_price: number;
      /** Highest allowed price in units of `price_scale`. */
      max_price: number;
      /** Whether matching can mint or merge complementary outcomes. */
      cross_match_enabled: boolean;
    } & Scales)
  | {
      /** Fee policy changed; fetch the current policy before calculating fees. */
      kind: "fee_policy_updated";
    }
  | {
      /** Cross-outcome matching was enabled or disabled. */
      kind: "cross_match_toggled";
      /** New cross-outcome matching setting. */
      enabled: boolean;
    };

/** Market metadata supplied with event lifecycle updates. */
export interface LifecycleMarket {
  /** Agara market identifier. */
  market_id: string;
  /** Condition identifier used by market subscriptions and trading queries. */
  condition_id: string;
  /** Lifecycle state, such as `ACTIVE` or `PROVISIONED`. */
  state: string;
  /** Decimal reference price, or null when unavailable. */
  reference_price: string | null;
  /** Reference-price currency, or null when unavailable. */
  currency: string | null;
  /** Reference timestamp, or null when unavailable. */
  reference_at: string | null;
  /** Observation timestamp, or null when unavailable. */
  observe_at: string | null;
  /** Outcome tokens and their display metadata. */
  outcomes: {
    /** Outcome token identifier. */
    token_id: string;
    /** Server-provided outcome classification. */
    kind: string;
    /** Full display label. */
    label: string;
    /** Abbreviated display label, or null when unavailable. */
    short_label: string | null;
  }[];
}

/** Resolution details shared by every replacement-state variant. */
type Resolution = {
  /** The market has completed resolution. */
  kind: "market_resolution_completed";
  /** Current main market identifier, or null when none is selected. */
  main_market_id: string | null;
  /** Identifier of the market that resolved. */
  resolved_market_id: string;
  /** Condition identifier of the market that resolved. */
  resolved_condition_id: string;
  /** Token of the winning outcome. */
  winning_token_id: string;
  /** Resolution completion timestamp. */
  resolved_at: string;
};

/** Event-scoped lifecycle updates; these do not carry an engine sequence. */
export type MarketLifecycleData =
  | {
      /** The event's active main market changed. */
      kind: "current_market_changed";
      /** Timestamp of the change. */
      changed_at: string;
      /** Identifier of the newly active main market. */
      main_market_id: string;
      /** Active market whose identifier equals `main_market_id`. */
      market: LifecycleMarket;
    }
  | (Resolution &
      (
        | {
            /** Whether the replacement is active or provisioned for a future cycle. */
            replacement_status: "active" | "upcoming";
            /** Replacement market, distinct from the resolved market. */
            replacement: LifecycleMarket;
          }
        | {
            /** A replacement cycle is expected but its market is not yet available. */
            replacement_status: "expected";
            /** No replacement market has been supplied yet. */
            replacement: null;
            /** Expected reference and observation times for the next cycle. */
            expected_cycle: {
              /** Expected reference timestamp. */
              reference_at: string;
              /** Expected observation timestamp. */
              observe_at: string;
            };
          }
        | {
            /** No replacement market or expected cycle is supplied. */
            replacement_status: "none";
            /** No replacement market is available. */
            replacement: null;
          }
      ));

/** Account-batch origin metadata; hash and operation index are present together or both absent. */
export type BatchProvenance =
  | {
      /** Hash of the originating signed account batch. */
      batch_hash: string;
      /** Zero-based operation index within the batch. */
      batch_index: number;
    }
  | {
      /** Absent when no account-batch origin is supplied. */
      batch_hash?: never;
      /** Absent when no account-batch origin is supplied. */
      batch_index?: never;
    };

/** Order identifiers included in account order and fill notifications. */
interface AccountOrderBase {
  /** Order identifier used by REST order queries. */
  order_id: string;
  /** Signed order hash. */
  order_hash: string;
  /** Outcome token for this order. */
  token_id: string;
}

/** Live account activity; reconcile missed activity through REST after a stream gap. */
export type AccountEventData =
  | (AccountOrderBase &
      Scales & {
        /** The order was accepted by the matching engine. */
        kind: "order_accepted";
        /** Order's buy or sell direction. */
        side: TradeSide;
        /** Limit price in units of `price_scale`. */
        price: number;
        /** Remaining shares in units of `size_scale`. */
        remaining_size: WireInteger;
        /** Original shares in units of `size_scale`. */
        original_size: WireInteger;
        /** Time-in-force reported by the engine. */
        tif: "GTC" | "FAK" | "FOK" | "UNSPECIFIED";
      })
  | (AccountOrderBase &
      Scales & {
        /** The order's unfilled remainder was cancelled. */
        kind: "order_cancelled";
        /** Order's buy or sell direction. */
        side: TradeSide;
        /** Limit price in units of `price_scale`. */
        price: number;
        /** Unfilled shares at cancellation in units of `size_scale`. */
        remaining_size: WireInteger;
        /** Reason the unfilled remainder was cancelled. */
        reason:
          | "USER"
          | "FAK_REMAINDER"
          | "SELF_TRADE_PREVENTION"
          | "MARKET_RESOLVED"
          | "INSUFFICIENT_COLLATERAL"
          | "UNSPECIFIED";
      })
  | {
      /** The submitted order was rejected. */
      kind: "order_rejected";
      /** Identifier of the rejected order. */
      order_id: string;
      /** Signed order hash, or null when unavailable. */
      order_hash: string | null;
      /** Outcome token, or null when unavailable. */
      token_id: string | null;
      /** Server rejection with validated public recovery guidance. */
      failure: PublicFailure;
    }
  | (AccountOrderBase &
      Scales & {
        /** A fill occurred for this account's order. */
        kind: "fill";
        /** Account's role in the match. */
        role: "MAKER" | "TAKER";
        /** Unique fill identifier. */
        fill_id: string;
        /** This account's buy or sell direction. */
        side: TradeSide;
        /** This account's execution price in units of `price_scale`. */
        price: number;
        /** Filled shares in units of `size_scale`. */
        size: WireInteger;
        /** How the matched outcomes are settled. */
        settlement_mode: SettlementMode;
        /** Fee charged to this account as an integer micro-USDC string. */
        fee_micro: string;
      })
  | (BatchProvenance & {
      /** Outcome shares were minted or merged. */
      kind: "tokens_minted" | "tokens_merged";
      /** Condition identifying the affected market. */
      condition_id: string;
      /** Shares minted or burned per outcome in units of `size_scale`. */
      size: WireInteger;
      /** Divisor converting `size` to shares. */
      size_scale: number;
    })
  | (BatchProvenance & {
      /** Winning shares were redeemed and losing shares cleared. */
      kind: "tokens_redeemed";
      /** Condition identifying the resolved market. */
      condition_id: string;
      /** Token redeemed for the payout. */
      winning_token_id: string;
      /** Losing token whose balance was cleared. */
      losing_token_id: string;
      /** Redeemed winning shares as an integer string in units of `size_scale`. */
      winning_shares_redeemed: string;
      /** Cleared losing shares as an integer string in units of `size_scale`. */
      losing_shares_zeroed: string;
      /** Credited payout as an integer micro-USDC string. */
      payout_micro: string;
      /** Divisor converting the integer share strings to shares. */
      size_scale: number;
    })
  | (BatchProvenance & {
      /** Collateral was added to or removed from the account. */
      kind: "collateral_deposited" | "collateral_withdrawn";
      /** Credited or debited amount as an integer micro-USDC string. */
      amount_micro: string;
      /** Resulting cash balance as an integer micro-USDC string. */
      cash_balance_micro: string;
    });

/**
 * A data update for one subscription subject.
 *
 * @remarks
 * Engine sequences need not be consecutive on filtered feeds. Event lifecycle updates have no
 * sequence. The SDK does not infer missing messages from numeric sequence jumps.
 */
export type UpdateFrame =
  | {
      /** Identifies a data update. */
      op: "update";
      /** Identifies orderbook data. */
      channel: "orderbook";
      /** Outcome whose orderbook changed. */
      token_id: string;
      /** Engine sequence associated with the book state. */
      sequence: WireInteger;
      /** Snapshot or price-level replacements. */
      data: OrderbookData;
    }
  | {
      /** Identifies a data update. */
      op: "update";
      /** Identifies best-quote data. */
      channel: "best_quote";
      /** Outcome whose quotes changed. */
      token_id: string;
      /** Engine sequence associated with the quotes. */
      sequence: WireInteger;
      /** Current best bid and ask. */
      data: BestQuoteData;
    }
  | {
      /** Identifies a data update. */
      op: "update";
      /** Identifies a public trade. */
      channel: "trades";
      /** Market in which the fill occurred. */
      condition_id: string;
      /** Engine sequence associated with the fill. */
      sequence: WireInteger;
      /** Fill details from the taker's perspective. */
      data: TradeData;
    }
  | {
      /** Identifies a data update. */
      op: "update";
      /** Identifies a condition-scoped status update. */
      channel: "market_status";
      /** Market whose status changed. */
      condition_id: string;
      /** Engine sequence associated with the status change. */
      sequence: WireInteger;
      /** Market status or configuration change. */
      data: MarketStatusData;
      /** Event scope is absent on condition-scoped updates. */
      event_id?: never;
    }
  | {
      /** Identifies a data update. */
      op: "update";
      /** Identifies an event-scoped lifecycle update. */
      channel: "market_status";
      /** Event whose market lifecycle changed. */
      event_id: string;
      /** Current-market or resolution details. */
      data: MarketLifecycleData;
      /** Event lifecycle updates have no engine sequence. */
      sequence?: never;
      /** Condition scope is absent on event-scoped updates. */
      condition_id?: never;
    }
  | {
      /** Identifies a data update. */
      op: "update";
      /** Identifies private account activity. */
      channel: "account_events";
      /** Engine sequence associated with the account activity. */
      sequence: WireInteger;
      /** Order, fill, token, or collateral activity. */
      data: AccountEventData;
    };

/** Server-reported failure; local transport and decoding errors reject stream iteration instead. */
export interface StreamFailure {
  /** Identifies a server failure frame. */
  op: "error";
  /** Public failure with recovery checked against the SDK's problem registry. */
  failure: PublicFailure;
  /** Automatic action allowed by the registry; unknown or mismatched actions become `none`. */
  action: "none" | "reconnect" | "resubscribe";
  /** Affected channel, when the failure is scoped to one. */
  channel?: ChannelName;
  /** Affected outcome, when supplied. */
  token_id?: string;
  /** Affected market, when supplied. */
  condition_id?: string;
  /** Affected event, when supplied. */
  event_id?: string;
}

/** Decoded router frame, including server controls, failures, and preserved future variants. */
export type ServerFrame =
  | UpdateFrame
  | (Subject & {
      /** Acknowledges a subscription change for this subject. */
      op: "subscribed" | "unsubscribed";
    })
  | (Subject & {
      /** Discard stale subject state and follow the channel's recovery procedure. */
      op: "sequence_reset";
      /** Messages were missed, or the upstream stream restarted. */
      reason: "lagged" | "stream_reset";
    })
  | StreamFailure
  | {
      /** Ping response or unsolicited server heartbeat. */
      op: "pong" | "heartbeat";
      /** Server's RFC 3339 timestamp. */
      server_time: string;
    }
  | {
      /** Response to a subscription-list request. */
      op: "subscription_list";
      /** Subjects currently subscribed on this connection. */
      channels: Subject[];
    }
  | {
      /** A frame variant that this SDK does not recognize. */
      op: "unknown";
      /** Parsed frame preserved for inspection. */
      raw: unknown;
    };

/** One provider price; its decimal value is `price × 10^expo`. */
export interface PriceStreamEntry {
  /** Provider's price-feed identifier. */
  id: string;
  /** Exact price mantissa, exponent, and publication time. */
  price: {
    /** Signed integer mantissa preserved as a decimal string. */
    price: string;
    /** Base-ten exponent applied to the mantissa. */
    expo: number;
    /** Provider publication time in Unix milliseconds. */
    publish_time_ms: number;
  };
}

/** Parsed price data carried by one SSE event. */
export interface PriceStreamFrame {
  /** Provider prices included in this event. */
  parsed: PriceStreamEntry[];
}

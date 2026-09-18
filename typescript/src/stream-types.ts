import type { PublicFailure } from "./errors.js";
export type WireInteger = number | bigint;
export type ChannelName =
  | "orderbook"
  | "best_quote"
  | "trades"
  | "market_status"
  | "account_events";
export type ChannelSpec =
  | { name: "orderbook"; token_id: string; depth?: number }
  | { name: "best_quote"; token_id: string }
  | { name: "trades"; condition_id: string }
  | { name: "market_status"; condition_id: string; event_id?: never }
  | { name: "market_status"; event_id: string; condition_id?: never }
  | { name: "account_events"; token: string };
export type ClientMessage =
  | { op: "subscribe" | "unsubscribe"; channels: ChannelSpec[] }
  | { op: "ping" | "list" };
export interface Subject {
  channel: ChannelName;
  token_id?: string;
  condition_id?: string;
  event_id?: string;
}
export interface Scales {
  price_scale: number;
  size_scale: number;
}
export type BookLevel = [price: number, size: WireInteger];
export type OrderbookData = Scales &
  (
    | { kind: "snapshot"; bids: BookLevel[]; asks: BookLevel[]; tick_size: number }
    | { kind: "delta"; bids: BookLevel[]; asks: BookLevel[] }
  );
export interface BestQuoteData extends Scales {
  bid: { price: number; size: WireInteger } | null;
  ask: { price: number; size: WireInteger } | null;
}
export type TradeSide = "BUY" | "SELL" | "UNSPECIFIED";
export type SettlementMode = "NORMAL" | "MINT" | "MERGE" | "UNSPECIFIED";
export interface TradeData extends Scales {
  kind: "trade";
  fill_id: string;
  taker_token_id: string;
  maker_token_id: string;
  side: TradeSide;
  price: number;
  size: WireInteger;
  settlement_mode: SettlementMode;
}
export type MarketStatusData =
  | { kind: "market_halted" }
  | { kind: "market_resumed" }
  | { kind: "market_resolved"; winning_token_id: string }
  | { kind: "outcome_proposed"; proposed_token_id: string }
  | ({
      kind: "market_created";
      num_outcomes: number;
      tick_size: number;
      min_price: number;
      max_price: number;
      cross_match_enabled: boolean;
    } & Scales)
  | { kind: "fee_policy_updated" }
  | { kind: "cross_match_toggled"; enabled: boolean };
export interface LifecycleMarket {
  market_id: string;
  condition_id: string;
  state: string;
  reference_price: string | null;
  currency: string | null;
  reference_at: string | null;
  observe_at: string | null;
  outcomes: { token_id: string; kind: string; label: string; short_label: string | null }[];
}
type Resolution = {
  kind: "market_resolution_completed";
  main_market_id: string | null;
  resolved_market_id: string;
  resolved_condition_id: string;
  winning_token_id: string;
  resolved_at: string;
};
export type MarketLifecycleData =
  | {
      kind: "current_market_changed";
      changed_at: string;
      main_market_id: string;
      market: LifecycleMarket;
    }
  | (Resolution &
      (
        | { replacement_status: "active" | "upcoming"; replacement: LifecycleMarket }
        | {
            replacement_status: "expected";
            replacement: null;
            expected_cycle: { reference_at: string; observe_at: string };
          }
        | { replacement_status: "none"; replacement: null }
      ));
export type BatchProvenance =
  | { batch_hash: string; batch_index: number }
  | { batch_hash?: never; batch_index?: never };
interface AccountOrderBase {
  order_id: string;
  order_hash: string;
  token_id: string;
}
export type AccountEventData =
  | (AccountOrderBase &
      Scales & {
        kind: "order_accepted";
        side: TradeSide;
        price: number;
        remaining_size: WireInteger;
        original_size: WireInteger;
        tif: "GTC" | "FAK" | "FOK" | "UNSPECIFIED";
      })
  | (AccountOrderBase &
      Scales & {
        kind: "order_cancelled";
        side: TradeSide;
        price: number;
        remaining_size: WireInteger;
        reason:
          | "USER"
          | "FAK_REMAINDER"
          | "SELF_TRADE_PREVENTION"
          | "MARKET_RESOLVED"
          | "INSUFFICIENT_COLLATERAL"
          | "UNSPECIFIED";
      })
  | {
      kind: "order_rejected";
      order_id: string;
      order_hash: string | null;
      token_id: string | null;
      failure: PublicFailure;
    }
  | (AccountOrderBase &
      Scales & {
        kind: "fill";
        role: "MAKER" | "TAKER";
        fill_id: string;
        side: TradeSide;
        price: number;
        size: WireInteger;
        settlement_mode: SettlementMode;
        fee_micro: string;
      })
  | (BatchProvenance & {
      kind: "tokens_minted" | "tokens_merged";
      condition_id: string;
      size: WireInteger;
      size_scale: number;
    })
  | (BatchProvenance & {
      kind: "tokens_redeemed";
      condition_id: string;
      winning_token_id: string;
      losing_token_id: string;
      winning_shares_redeemed: string;
      losing_shares_zeroed: string;
      payout_micro: string;
      size_scale: number;
    })
  | (BatchProvenance & {
      kind: "collateral_deposited" | "collateral_withdrawn";
      amount_micro: string;
      cash_balance_micro: string;
    });
export type UpdateFrame =
  | {
      op: "update";
      channel: "orderbook";
      token_id: string;
      sequence: WireInteger;
      data: OrderbookData;
    }
  | {
      op: "update";
      channel: "best_quote";
      token_id: string;
      sequence: WireInteger;
      data: BestQuoteData;
    }
  | {
      op: "update";
      channel: "trades";
      condition_id: string;
      sequence: WireInteger;
      data: TradeData;
    }
  | {
      op: "update";
      channel: "market_status";
      condition_id: string;
      sequence: WireInteger;
      data: MarketStatusData;
      event_id?: never;
    }
  | {
      op: "update";
      channel: "market_status";
      event_id: string;
      data: MarketLifecycleData;
      sequence?: never;
      condition_id?: never;
    }
  | { op: "update"; channel: "account_events"; sequence: WireInteger; data: AccountEventData };
export interface StreamFailure {
  op: "error";
  failure: PublicFailure;
  action: "none" | "reconnect" | "resubscribe";
  channel?: ChannelName;
  token_id?: string;
  condition_id?: string;
  event_id?: string;
}
export type ServerFrame =
  | UpdateFrame
  | (Subject & { op: "subscribed" | "unsubscribed" })
  | (Subject & { op: "sequence_reset"; reason: "lagged" | "stream_reset" })
  | StreamFailure
  | { op: "pong" | "heartbeat"; server_time: string }
  | { op: "subscription_list"; channels: Subject[] }
  | { op: "unknown"; raw: unknown };
export interface PriceStreamEntry {
  id: string;
  price: { price: string; expo: number; publish_time_ms: number };
}
export interface PriceStreamFrame {
  parsed: PriceStreamEntry[];
}

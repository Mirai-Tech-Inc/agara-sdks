import type {
  components as CatalogueComponents,
  operations as CatalogueOperations,
} from "./generated/catalogue.js";
import type {
  components as TradingComponents,
  operations as TradingOperations,
} from "./generated/trading.js";

/** Generated OpenAPI operation maps for catalogue discovery and trading endpoints. */
export type { CatalogueOperations, TradingOperations };
/** Trading schema map; index by a schema name to obtain its request or response type. */
export type TradingSchemas = TradingComponents["schemas"];
/** Catalogue schema map; index by a schema name to obtain its request or response type. */
export type CatalogueSchemas = CatalogueComponents["schemas"];
/**
 * Extracts an operation's required JSON request body, or `never` when none is defined.
 *
 * @typeParam T - Operation definition from `TradingOperations` or `CatalogueOperations`.
 */
export type RequestBody<T> = T extends { requestBody: { content: { "application/json": infer B } } }
  ? B
  : never;
/**
 * Extracts an operation's query parameters with null and undefined removed.
 *
 * @typeParam T - Operation definition from `TradingOperations` or `CatalogueOperations`.
 */
export type Query<T> = T extends { parameters: { query?: infer Q } } ? NonNullable<Q> : never;
type Json<T> = T extends { content: { "application/json": infer B } } ? B : never;
/**
 * Extracts the union of JSON response bodies declared for HTTP 200, 201, 202, and 206.
 *
 * @typeParam T - Operation definition from `TradingOperations` or `CatalogueOperations`.
 */
export type Success<T> = T extends { responses: infer R }
  ? Json<R[keyof R & (200 | 201 | 202 | 206)]>
  : never;
/** Order snapshot; `is_terminal` determines order completion, which is separate from trade settlement. */
export type Order = TradingSchemas["Order"];
/** Wallet's side of an executed fill, including exact micro amounts and its settlement status. */
export type Fill = TradingSchemas["Fill"];
/** Exchange identifier accepted by trading and portfolio APIs. */
export type Exchange = TradingSchemas["Exchange"];
/** Direction of an order or fill: BUY acquires shares and SELL disposes of shares. */
export type Side = TradingSchemas["Side"];
/**
 * Order duration policy: GTC rests until canceled, GTD rests until expiration, FAK fills available
 * liquidity and cancels the remainder, and FOK requires the full quantity to fill immediately.
 */
export type TimeInForce = TradingSchemas["ClobOrderTimeInForce"];
/** Account-batch state and failure details; reaching a completed state does not imply success. */
export type BatchStatus = TradingSchemas["AccountBatchStatusDto"];
/** SPLIT, MERGE, or WITHDRAW operation supported by presigned account-batch submissions. */
export type BatchOperation = Extract<
  TradingSchemas["BatchOpDto"],
  { kind: "SPLIT" | "MERGE" | "WITHDRAW" }
>;
/** Presigned account-batch submission with a signature bound to its sequence and deadline. */
export type BatchSubmission = Omit<TradingSchemas["AccountBatchSubmission"], "ops"> & {
  /** One through 20 supported operations in the same execution order used for signing. */
  ops: BatchOperation[];
};
/** Request to submit multiple independently signed LIMIT orders. */
export interface SignedOrderBatchRequest {
  /** One through 32 signed order bodies. */
  orders: SignedOrderRequest[];
}
/** Batch acceptance returned by split and merge. */
export type PositionOperationResult = TradingSchemas["PositionOperationAccepted"];
type Resting =
  | {
      /** Rest any unfilled quantity until canceled. */
      time_in_force: "GTC";
      /** Expiration is omitted for GTC orders. */
      expiration_unix_seconds?: never;
      /** Reject the order if it would immediately cross the spread; defaults to false on the server. */
      post_only?: boolean;
    }
  | {
      /** Rest any unfilled quantity until the required expiration. */
      time_in_force: "GTD";
      /** Expiration in Unix seconds; the server requires at least 30 seconds of lead time. */
      expiration_unix_seconds: number;
      /** Reject the order if it would immediately cross the spread; defaults to false on the server. */
      post_only?: boolean;
    };
type Immediate = {
  /** FAK fills available liquidity and cancels the remainder; FOK requires an immediate full fill. */
  time_in_force: "FAK" | "FOK";
  /** Expiration is omitted for immediate orders. */
  expiration_unix_seconds?: never;
  /** Immediate orders cannot be post-only. */
  post_only?: false;
};
/** LIMIT order sized in shares, with a price and a resting or immediate execution policy. */
export type LimitOrderRequest = {
  /** Outcome token identifier. */
  token_id: string;
  /** Selects a LIMIT order with an explicit price. */
  type: "LIMIT";
  /** Buy or sell the requested shares. */
  side: Side;
  /** Integer limit price in millionths of a collateral unit per share, from 1 through 999,999. */
  price_micro: string;
  /** Positive integer share quantity in millionths, within signed 64-bit range. */
  shares_micro: string;
  /** LIMIT orders are sized with shares rather than a collateral budget. */
  collateral_amount_micro?: never;
} & (Resting | Immediate);
/** MARKET order with immediate execution, sized by a BUY collateral budget or a SELL share quantity. */
export type MarketOrderRequest = {
  /** Outcome token identifier. */
  token_id: string;
  /** Selects a MARKET order without an explicit limit price. */
  type: "MARKET";
  /** MARKET orders omit a limit price. */
  price_micro?: never;
} & Immediate &
  (
    | {
        /** Acquire shares using the collateral budget. */
        side: "BUY";
        /** Positive integer collateral budget in millionths, within signed 64-bit range. */
        collateral_amount_micro: string;
        /** MARKET BUY orders specify collateral rather than a share quantity. */
        shares_micro?: never;
      }
    | {
        /** Sell the specified quantity of shares. */
        side: "SELL";
        /** Positive integer share quantity in millionths, within signed 64-bit range. */
        shares_micro: string;
        /** MARKET SELL orders specify shares rather than a collateral budget. */
        collateral_amount_micro?: never;
      }
  );
/** Unsigned LIMIT or MARKET order accepted by the server-signed order endpoint. */
export type OrderRequest = LimitOrderRequest | MarketOrderRequest;
/** LIMIT request combined with the signed EIP-712 envelope, signature, and order hash. */
export type SignedOrderRequest = Omit<
  TradingSchemas["CreateSignedClobOrderRequest"],
  | "type"
  | "side"
  | "price_micro"
  | "shares_micro"
  | "time_in_force"
  | "post_only"
  | "expiration_unix_seconds"
> &
  LimitOrderRequest;

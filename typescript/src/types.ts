import type {
  components as CatalogueComponents,
  operations as CatalogueOperations,
} from "./generated/catalogue.js";
import type {
  components as TradingComponents,
  operations as TradingOperations,
} from "./generated/trading.js";

export type { CatalogueOperations, TradingOperations };
export type TradingSchemas = TradingComponents["schemas"];
export type CatalogueSchemas = CatalogueComponents["schemas"];
export type RequestBody<T> = T extends { requestBody: { content: { "application/json": infer B } } }
  ? B
  : never;
export type Query<T> = T extends { parameters: { query?: infer Q } } ? NonNullable<Q> : never;
type Json<T> = T extends { content: { "application/json": infer B } } ? B : never;
export type Success<T> = T extends { responses: infer R }
  ? Json<R[keyof R & (200 | 201 | 202 | 206)]>
  : never;
export type Order = TradingSchemas["Order"];
export type Fill = TradingSchemas["Fill"];
export type Exchange = TradingSchemas["Exchange"];
export type Side = TradingSchemas["Side"];
export type TimeInForce = TradingSchemas["ClobOrderTimeInForce"];
export type BatchStatus = TradingSchemas["AccountBatchStatusDto"];
export type BatchGroupStatus = TradingSchemas["BatchGroupStatusDto"];
export type BatchOperation = Extract<
  TradingSchemas["BatchOpDto"],
  { kind: "SPLIT" | "MERGE" | "WITHDRAW" }
>;
export type BatchSubmission = Omit<TradingSchemas["AccountBatchSubmission"], "ops"> & {
  ops: BatchOperation[];
};
export type BatchSupersedeSubmission = Omit<
  TradingSchemas["AccountBatchSupersedeSubmission"],
  "ops"
> & { ops: BatchOperation[] };
export interface SignedOrderBatchRequest {
  orders: SignedOrderRequest[];
}
export type PositionOperationResult =
  | TradingSchemas["PositionOperationAccepted"]
  | TradingSchemas["PortfolioPositionOperationResponse"];
type Resting =
  | { time_in_force: "GTC"; expiration_unix_seconds?: never; post_only?: boolean }
  | { time_in_force: "GTD"; expiration_unix_seconds: number; post_only?: boolean };
type Immediate = {
  time_in_force: "FAK" | "FOK";
  expiration_unix_seconds?: never;
  post_only?: false;
};
export type LimitOrderRequest = {
  token_id: string;
  type: "LIMIT";
  side: Side;
  price_micro: string;
  shares_micro: string;
  collateral_amount_micro?: never;
} & (Resting | Immediate);
export type MarketOrderRequest = {
  token_id: string;
  type: "MARKET";
  price_micro?: never;
} & Immediate &
  (
    | { side: "BUY"; collateral_amount_micro: string; shares_micro?: never }
    | { side: "SELL"; shares_micro: string; collateral_amount_micro?: never }
  );
export type OrderRequest = LimitOrderRequest | MarketOrderRequest;
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

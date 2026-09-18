import { expectTypeOf, it } from "vitest";
import type { PublicClient, TraderClient } from "../src/endpoints.js";
import type { UpdateFrame } from "../src/stream-types.js";
import type { BatchOperation, LimitOrderRequest, TradingSchemas } from "../src/types.js";

it("keeps the public, exact-money and presigned operation boundaries in types", () => {
  expectTypeOf<PublicClient>().not.toHaveProperty("placeOrder");
  expectTypeOf<TraderClient>().toHaveProperty("placeOrder");
  expectTypeOf<BatchOperation["kind"]>().toEqualTypeOf<"SPLIT" | "MERGE" | "WITHDRAW">();
  expectTypeOf<LimitOrderRequest["collateral_amount_micro"]>().toEqualTypeOf<undefined>();
  expectTypeOf<TradingSchemas["AccountBatchStatusDto"]["seq"]>().toEqualTypeOf<number | bigint>();
  expectTypeOf<TradingSchemas["PositionOperationAccepted"]["status"]>().toEqualTypeOf<"PENDING">();
  expectTypeOf<Extract<UpdateFrame, { event_id: string }>["sequence"]>().toEqualTypeOf<undefined>();
});

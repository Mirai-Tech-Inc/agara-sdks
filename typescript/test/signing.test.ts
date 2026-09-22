import { hashTypedData, recoverTypedDataAddress } from "viem";
import { privateKeyToAccount } from "viem/accounts";
import { expect, it } from "vitest";
import {
  composeBatchCalls,
  hashBatch,
  ORDER_TYPES,
  orderTypedData,
  type RoutedBatchOperation,
  type SignOrderInput,
  signBatch,
  signOrder,
  ZERO_BYTES32,
} from "../src/signing.js";
import { validateOrder, validateRequest } from "../src/validation.js";
import { contract } from "./helpers.js";

const signer = privateKeyToAccount(`0x${"11".repeat(32)}`);
const orderInput: SignOrderInput = {
  domain: { chainId: 84532, exchangeContract: "0x1b42FF8DdB251074637d3A9872D72f51e3AbB23d" },
  maker: "0x279640887C3806d4FBd424bb0B58F0430CE661C1",
  tokenId: "2",
  side: "BUY",
  priceMicro: 500000n,
  sharesMicro: 2000000n,
  salt: 1n,
};
const context = {
  ctf: "0x2222222222222222222222222222222222222222",
  negRiskAdapter: "0x3333333333333333333333333333333333333333",
  collateral: "0x4444444444444444444444444444444444444444",
} as const;
const domain = {
  chainId: 8453,
  account: "0x1111111111111111111111111111111111111111",
  implementationVersion: 1,
} as const;
const operations: RoutedBatchOperation[] = [
  {
    route: "Ctf",
    op: {
      kind: "SPLIT",
      market_id: "00000000-0000-0000-0000-000000000001",
      condition_id: `0x${"a".repeat(63)}1`,
      shares_micro: 5000000n,
    },
  },
  {
    route: "Ctf",
    op: {
      kind: "MERGE",
      market_id: "00000000-0000-0000-0000-000000000002",
      condition_id: `0x${"a".repeat(63)}2`,
      shares_micro: 2000000n,
    },
  },
  {
    route: "Ctf",
    op: {
      kind: "WITHDRAW",
      destination: "0x5555555555555555555555555555555555555555",
      amount_micro: 1000000n,
    },
  },
];
it("matches the platform nine-field order digest golden", () => {
  expect(
    hashTypedData({
      domain: {
        name: "Agara CTF Exchange",
        version: "1",
        chainId: 84532,
        verifyingContract: orderInput.domain.exchangeContract,
      },
      types: ORDER_TYPES,
      primaryType: "Order",
      message: {
        salt: 1n,
        maker: orderInput.maker,
        tokenId: 2n,
        makerAmount: 100n,
        takerAmount: 100n,
        side: 0,
        timestamp: 0n,
        metadata: ZERO_BYTES32,
        builder: ZERO_BYTES32,
      },
    }),
  ).toBe("0xf5cbbd057896816a0be9a705a90bf1f094b01af05f977658791b3e65c862c961");
});
it.each(["BUY", "SELL"] as const)(
  "binds %s economics and recovers the supplied signer",
  async (side) => {
    const input = { ...orderInput, side };
    const result = await signOrder(input, signer);
    expect(result.maker_amount).toBe(side === "BUY" ? "1000000" : "2000000");
    expect(
      await recoverTypedDataAddress({
        ...orderTypedData(input),
        signature: result.signature as `0x${string}`,
      }),
    ).toBe(signer.address);
    expect(() =>
      validateOrder({ ...result, side: side === "BUY" ? "SELL" : "BUY" }, true),
    ).toThrow();
  },
);
it("rejects invalid signing inputs before any signature", () => {
  expect(() => orderTypedData({ ...orderInput, salt: 0n })).toThrow();
  expect(() => orderTypedData({ ...orderInput, priceMicro: 1000000n })).toThrow();
  expect(() => orderTypedData({ ...orderInput, sharesMicro: 1n, priceMicro: 1n })).toThrow();
});
it("matches both canonical account batch calldata and digest vectors", () => {
  const expected = contract("batch-goldens");
  const calls = composeBatchCalls(operations, context);
  expect(calls.map((c) => c.data)).toEqual([
    expected.SPLIT_CALLDATA,
    expected.MERGE_CALLDATA,
    expected.TRANSFER_CALLDATA,
  ]);
  expect(hashBatch(domain, 4n, 1784022000n, calls.slice(0, 1))).toBe(
    expected.SINGLE_CALL_BATCH_HASH,
  );
  expect(hashBatch(domain, 4n, 1784022000n, calls)).toBe(expected.MULTI_CALL_BATCH_HASH);
  expect(hashBatch({ ...domain, implementationVersion: 2 }, 4n, 1784022000n, calls)).not.toBe(
    expected.MULTI_CALL_BATCH_HASH,
  );
});
it("signs exactly the typed operations subsequently submitted", async () => {
  const result = await signBatch(
    { domain, context, operations, seq: 4n, deadlineUnixSeconds: 1784022000n },
    signer,
  );
  expect(result.batchHash).toBe(contract("batch-goldens").MULTI_CALL_BATCH_HASH);
  expect(result.body.ops).toEqual(operations.map((o) => o.op));
  expect(result.body.seq).toBe(4n);
});
it("routes neg-risk merges and rejects unsupported splits/self withdrawals", async () => {
  const merge = operations[1];
  if (!merge) throw Error();
  expect(
    composeBatchCalls([{ ...merge, route: "NegRisk" }], context)[0]?.target.toLowerCase(),
  ).toBe(context.negRiskAdapter);
  const split = operations[0];
  if (!split) throw Error();
  expect(() => composeBatchCalls([{ ...split, route: "NegRisk" }], context)).toThrow();
  await expect(
    signBatch(
      {
        domain,
        context,
        seq: 4n,
        deadlineUnixSeconds: 1784022000n,
        operations: [
          {
            route: "Ctf",
            op: { kind: "WITHDRAW", destination: domain.account, amount_micro: 1000000n },
          },
        ],
      },
      signer,
    ),
  ).rejects.toThrow();
});
it("carries a heal reference in the submission without changing what was signed", async () => {
  const input = { domain, context, operations, seq: 4n, deadlineUnixSeconds: 1784022000n };
  const plain = await signBatch(input, signer);
  const healing = await signBatch({ ...input, healsBatchHash: `0x${"ab".repeat(32)}` }, signer);

  // The reference rides in the body only. A digest that moved with it would make the healing
  // submission a different batch from the one the signature authorizes.
  expect(healing.batchHash).toBe(contract("batch-goldens").MULTI_CALL_BATCH_HASH);
  expect(healing.batchHash).toBe(plain.batchHash);
  expect(healing.body.signature).toBe(plain.body.signature);
  expect(healing.body.heals_batch_hash).toBe(`0x${"ab".repeat(32)}`);
  expect("heals_batch_hash" in plain.body).toBe(false);
  expect(() => validateRequest("submitBatch", healing.body, undefined)).not.toThrow();
});
it("refuses a heal reference that is not a bytes32 hash", async () => {
  const input = { domain, context, operations, seq: 4n, deadlineUnixSeconds: 1784022000n };
  await expect(
    signBatch({ ...input, healsBatchHash: `0x${"ab".repeat(31)}` }, signer),
  ).rejects.toThrow(TypeError);
});

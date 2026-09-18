import {
  type Address,
  encodeFunctionData,
  getAddress,
  type Hex,
  hashTypedData,
  type LocalAccount,
  parseAbi,
} from "viem";
import { integer, MICRO } from "./amounts.js";
import type {
  BatchSubmission,
  LimitOrderRequest,
  SignedOrderRequest,
  TimeInForce,
  TradingSchemas,
} from "./types.js";
import { validateOrder } from "./validation.js";
export type TypedDataSigner = Pick<LocalAccount, "signTypedData">;
export interface OrderDomain {
  chainId: number | bigint;
  exchangeContract: Address;
}
export const ORDER_TYPES = {
  Order: [
    { name: "salt", type: "uint256" },
    { name: "maker", type: "address" },
    { name: "tokenId", type: "uint256" },
    { name: "makerAmount", type: "uint256" },
    { name: "takerAmount", type: "uint256" },
    { name: "side", type: "uint8" },
    { name: "timestamp", type: "uint256" },
    { name: "metadata", type: "bytes32" },
    { name: "builder", type: "bytes32" },
  ],
} as const;
export const BATCH_TYPES = {
  Call: [
    { name: "target", type: "address" },
    { name: "value", type: "uint256" },
    { name: "data", type: "bytes" },
  ],
  Batch: [
    { name: "wallet", type: "address" },
    { name: "seq", type: "uint256" },
    { name: "deadline", type: "uint256" },
    { name: "calls", type: "Call[]" },
  ],
} as const;
export const ZERO_BYTES32 = `0x${"00".repeat(32)}` as Hex;
const MAX_I64 = (1n << 63n) - 1n;
const ctfAbi = parseAbi([
  "function splitPosition(address collateralToken, bytes32 parentCollectionId, bytes32 conditionId, uint256[] partition, uint256 amount)",
  "function mergePositions(address collateralToken, bytes32 parentCollectionId, bytes32 conditionId, uint256[] partition, uint256 amount)",
]);
const erc20Abi = parseAbi(["function transfer(address to, uint256 amount)"]);
export interface SignOrderInput {
  domain: OrderDomain;
  maker: Address;
  tokenId: string | bigint;
  side: "BUY" | "SELL";
  priceMicro: string | bigint;
  sharesMicro: string | bigint;
  salt?: string | bigint;
  timeInForce?: TimeInForce;
  postOnly?: boolean;
  expirationUnixSeconds?: number;
  timestamp?: string | bigint;
  metadata?: Hex;
  builder?: Hex;
}
export function orderTypedData(input: SignOrderInput) {
  const salt = input.salt === undefined ? randomSalt() : integer(input.salt, "salt", undefined, 1n);
  const shares = integer(input.sharesMicro, "sharesMicro", MAX_I64, 1n),
    price = integer(input.priceMicro, "priceMicro", 999999n, 1n);
  const collateral = (shares * price) / MICRO;
  if (collateral === 0n) throw new RangeError("Order collateral rounds to zero");
  if (input.side !== "BUY" && input.side !== "SELL") throw new TypeError("Invalid order side");
  return {
    domain: {
      name: "Agara CTF Exchange",
      version: "1",
      chainId: integer(input.domain.chainId, "chainId", (1n << 64n) - 1n, 1n),
      verifyingContract: address(input.domain.exchangeContract),
    },
    types: ORDER_TYPES,
    primaryType: "Order" as const,
    message: {
      salt,
      maker: address(input.maker),
      tokenId: integer(input.tokenId, "tokenId"),
      makerAmount: input.side === "BUY" ? collateral : shares,
      takerAmount: input.side === "BUY" ? shares : collateral,
      side: input.side === "BUY" ? 0 : 1,
      timestamp: integer(input.timestamp ?? 0n, "timestamp"),
      metadata: bytes32(input.metadata ?? ZERO_BYTES32),
      builder: bytes32(input.builder ?? ZERO_BYTES32),
    },
  };
}
export function hashOrder(input: SignOrderInput): Hex {
  if (input.salt === undefined)
    throw new TypeError(
      "hashOrder requires an explicit salt; signOrder generates one and returns its hash",
    );
  return hashTypedData(orderTypedData(input));
}
export async function signOrder(
  input: SignOrderInput,
  signer: TypedDataSigner,
): Promise<SignedOrderRequest> {
  const typed = orderTypedData(input);
  const body = {
    token_id: typed.message.tokenId.toString(),
    type: "LIMIT",
    side: input.side,
    price_micro: BigInt(input.priceMicro).toString(),
    shares_micro: BigInt(input.sharesMicro).toString(),
    time_in_force: input.timeInForce ?? "GTC",
    post_only: input.postOnly ?? false,
    ...(input.expirationUnixSeconds === undefined
      ? {}
      : { expiration_unix_seconds: input.expirationUnixSeconds }),
    salt: typed.message.salt.toString(),
    maker: typed.message.maker,
    chain_token_id: typed.message.tokenId.toString(),
    maker_amount: typed.message.makerAmount.toString(),
    taker_amount: typed.message.takerAmount.toString(),
    side_u8: typed.message.side,
    timestamp: typed.message.timestamp.toString(),
    metadata: typed.message.metadata,
    builder: typed.message.builder,
    order_hash: hashTypedData(typed),
    signature: await signer.signTypedData(typed),
  };
  validateOrder(body, true);
  return Object.freeze(body) as SignedOrderRequest;
}
export type PresignedBatchOperation = Extract<
  TradingSchemas["BatchOpDto"],
  { kind: "SPLIT" | "MERGE" | "WITHDRAW" }
>;
export interface RoutedBatchOperation {
  op: PresignedBatchOperation;
  route: "Ctf" | "NegRisk";
}
export interface ComposeContext {
  ctf: Address;
  negRiskAdapter: Address;
  collateral: Address;
}
export interface BatchCall {
  target: Address;
  value: bigint;
  data: Hex;
}
export interface BatchDomain {
  chainId: number | bigint;
  account: Address;
  implementationVersion: number | bigint;
}
export interface SignBatchInput {
  domain: BatchDomain;
  context: ComposeContext;
  operations: readonly RoutedBatchOperation[];
  seq: number | bigint;
  deadlineUnixSeconds: number | bigint;
  healsBatchHash?: Hex;
}
export function composeBatchCalls(
  operations: readonly RoutedBatchOperation[],
  context: ComposeContext,
): BatchCall[] {
  if (operations.length < 1 || operations.length > 20)
    throw new RangeError("Batches need 1–20 operations");
  return operations.map(({ op, route }) => {
    if (op.kind === "WITHDRAW") {
      const amount = integer(op.amount_micro, "amount_micro", MAX_I64, 10000n);
      return {
        target: address(context.collateral),
        value: 0n,
        data: encodeFunctionData({
          abi: erc20Abi,
          functionName: "transfer",
          args: [address(op.destination), amount],
        }),
      };
    }
    if (op.kind !== "SPLIT" && op.kind !== "MERGE")
      throw new TypeError("Operation is not supported for presigned batches");
    if (
      !/^[0-9a-f]{8}(?:-[0-9a-f]{4}){3}-[0-9a-f]{12}$/i.test(op.market_id) ||
      /^0{8}-0{4}-0{4}-0{4}-0{12}$/.test(op.market_id)
    )
      throw new TypeError("Nonzero market UUID required");
    if (route !== "Ctf" && route !== "NegRisk") throw new TypeError("Unknown market route");
    if (op.kind === "SPLIT" && route === "NegRisk")
      throw new TypeError("Neg-risk split is not supported");
    return {
      target: address(route === "NegRisk" ? context.negRiskAdapter : context.ctf),
      value: 0n,
      data: encodeFunctionData({
        abi: ctfAbi,
        functionName: op.kind === "SPLIT" ? "splitPosition" : "mergePositions",
        args: [
          address(context.collateral),
          ZERO_BYTES32,
          bytes32(op.condition_id),
          [1n, 2n],
          integer(op.shares_micro, "shares_micro", MAX_I64, 1n),
        ],
      }),
    };
  });
}
export function batchTypedData(
  domain: BatchDomain,
  seq: number | bigint,
  deadline: number | bigint,
  calls: readonly BatchCall[],
) {
  const account = address(domain.account);
  return {
    domain: {
      name: "AgaraAccount",
      version: integer(
        domain.implementationVersion,
        "implementationVersion",
        (1n << 64n) - 1n,
        1n,
      ).toString(),
      chainId: integer(domain.chainId, "chainId", (1n << 64n) - 1n, 1n),
      verifyingContract: account,
    },
    types: BATCH_TYPES,
    primaryType: "Batch" as const,
    message: {
      wallet: account,
      seq: integer(seq, "seq", MAX_I64),
      deadline: integer(deadline, "deadline", MAX_I64),
      calls: [...calls],
    },
  };
}
export function hashBatch(
  domain: BatchDomain,
  seq: number | bigint,
  deadline: number | bigint,
  calls: readonly BatchCall[],
): Hex {
  return hashTypedData(batchTypedData(domain, seq, deadline, calls));
}
export async function signBatch(
  input: SignBatchInput,
  signer: TypedDataSigner,
): Promise<{ batchHash: Hex; body: BatchSubmission; calls: readonly BatchCall[] }> {
  const account = address(input.domain.account);
  for (const { op } of input.operations)
    if (op.kind === "WITHDRAW" && address(op.destination) === account)
      throw new TypeError("Cannot withdraw to the account itself");
  const calls = composeBatchCalls(input.operations, input.context);
  const typed = batchTypedData(input.domain, input.seq, input.deadlineUnixSeconds, calls);
  const signature = await signer.signTypedData(typed);
  return {
    batchHash: hashTypedData(typed),
    calls,
    body: {
      ops: input.operations.map(({ op }) => ({ ...op })),
      seq: typed.message.seq,
      deadline_unix_seconds: typed.message.deadline,
      signature,
      ...(input.healsBatchHash ? { heals_batch_hash: bytes32(input.healsBatchHash) } : {}),
    },
  };
}
export function limitOrder(
  input: Omit<LimitOrderRequest, "price_micro" | "shares_micro"> & {
    price_micro: string | bigint;
    shares_micro: string | bigint;
  },
): LimitOrderRequest {
  const result = {
    ...input,
    price_micro: input.price_micro.toString(),
    shares_micro: input.shares_micro.toString(),
  };
  validateOrder(result);
  return result as LimitOrderRequest;
}
function address(value: string): Address {
  const result = getAddress(value);
  if (/^0x0{40}$/.test(result)) throw new TypeError("Zero address is not allowed");
  return result;
}
function bytes32(value: string): Hex {
  if (!/^0x[0-9a-fA-F]{64}$/.test(value)) throw new TypeError("Expected bytes32 hex");
  return value as Hex;
}
function randomSalt(): bigint {
  const bytes = crypto.getRandomValues(new Uint8Array(16));
  return BigInt(`0x${Array.from(bytes, (x) => x.toString(16).padStart(2, "0")).join("")}`) || 1n;
}

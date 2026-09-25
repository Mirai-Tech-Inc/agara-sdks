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
/** Signer providing viem-compatible EIP-712 signing; key storage remains with the caller. */
export type TypedDataSigner = Pick<LocalAccount, "signTypedData">;
/** Deployment-specific EIP-712 domain inputs for an Agara CTF Exchange order. */
export interface OrderDomain {
  /** Positive chain ID fitting an unsigned 64-bit integer; number inputs must be safe integers. */
  chainId: number | bigint;
  /** Nonzero address of the exchange contract that verifies the order signature. */
  exchangeContract: Address;
}
/** Nine-field EIP-712 `Order` schema used by the Agara CTF Exchange. */
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
/** EIP-712 `Call` and `Batch` schemas used by AgaraAccount signatures. */
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
/** `0x`-prefixed zero bytes32 value used for optional order fields and the root collection. */
export const ZERO_BYTES32 = `0x${"00".repeat(32)}` as Hex;
const MAX_I64 = (1n << 63n) - 1n;
const ctfAbi = parseAbi([
  "function splitPosition(address collateralToken, bytes32 parentCollectionId, bytes32 conditionId, uint256[] partition, uint256 amount)",
  "function mergePositions(address collateralToken, bytes32 parentCollectionId, bytes32 conditionId, uint256[] partition, uint256 amount)",
]);
const erc20Abi = parseAbi(["function transfer(address to, uint256 amount)"]);
/** Exact order economics, signing domain, and submission policy for a signed LIMIT order. */
export interface SignOrderInput {
  /** Trusted chain ID and exchange contract address for the target deployment. */
  domain: OrderDomain;
  /** Nonzero maker address; the server requires the wallet's deposit-wallet address. */
  maker: Address;
  /** Unsigned 256-bit outcome token ID, supplied as a bigint or decimal/`0x` integer string. */
  tokenId: string | bigint;
  /** Buy shares with collateral or sell shares for collateral. */
  side: "BUY" | "SELL";
  /** Integer collateral price per share in millionths, from 1 through 999,999. */
  priceMicro: string | bigint;
  /** Positive share quantity in millionths, at most the maximum signed 64-bit integer. */
  sharesMicro: string | bigint;
  /** Nonzero unsigned 256-bit salt; omission generates a fresh random 128-bit salt. */
  salt?: string | bigint;
  /** Submission time in force; defaults to `GTC` in `signOrder` and is not part of the digest. */
  timeInForce?: TimeInForce;
  /** Reject crossing orders when true; defaults to false and requires `GTC` or `GTD`. */
  postOnly?: boolean;
  /** Future Unix timestamp in seconds, required only for `GTD`; not part of the signed digest. */
  expirationUnixSeconds?: number;
  /** Unsigned 256-bit order timestamp field; defaults to zero and is not populated from the clock. */
  timestamp?: string | bigint;
  /** `0x`-prefixed 32-byte metadata hash; defaults to {@link ZERO_BYTES32}. */
  metadata?: Hex;
  /** `0x`-prefixed 32-byte builder code; defaults to {@link ZERO_BYTES32}. */
  builder?: Hex;
}
/**
 * Builds the EIP-712 data for an order, deriving maker and taker amounts from its economics.
 *
 * @param input - Domain, maker, exact price and shares, and optional signed fields.
 * @returns Typed data compatible with viem's signing and hashing functions.
 * @throws TypeError - If an integer, side, bytes32 field, or zero address is invalid.
 * @throws RangeError - If a numeric field is out of range or collateral rounds to zero.
 * @remarks
 * Collateral is `sharesMicro * priceMicro / 1_000_000`, truncated to whole micro units.
 * An omitted salt is generated anew on each call; retain the returned data or supply a salt
 * when hashing and signing separately. Submission policy fields are not validated here and
 * are not included in the signed message. Invalid address encodings may also throw viem errors.
 */
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
/**
 * Computes the EIP-712 order digest using an explicit salt.
 *
 * @param input - Order signing input with a nonzero `salt` supplied by the caller.
 * @returns The `0x`-prefixed 32-byte order hash.
 * @throws TypeError - If the salt is omitted; other input errors propagate from {@link orderTypedData}.
 * @remarks
 * To generate a fresh salt and obtain its matching signature and hash together, use {@link signOrder}.
 */
export function hashOrder(input: SignOrderInput): Hex {
  if (input.salt === undefined)
    throw new TypeError(
      "hashOrder requires an explicit salt; signOrder generates one and returns its hash",
    );
  return hashTypedData(orderTypedData(input));
}
/**
 * Signs an order and returns its complete signed submission body, including the order hash.
 *
 * @param input - Exact order economics, deployment domain, and submission policy.
 * @param signer - Caller-owned EIP-712 signer, typically the wallet holder's viem account.
 * @returns A frozen submission body with exact integer strings, the signature, and `order_hash`.
 * @throws TypeError - If signing fields, submission policy, or the returned signature are invalid.
 * @throws RangeError - If amounts exceed supported bounds or collateral rounds to zero.
 * @remarks
 * Omitted policy fields become `GTC` and `post_only: false`. An omitted salt generates a fresh
 * value, so repeated calls produce different orders. Policy and signature validation runs after
 * the signer resolves; signer and address-library errors propagate to the caller. This function
 * neither submits the order nor verifies that the signer is authorized for the maker.
 */
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
/** SPLIT, MERGE, or WITHDRAW intent supported by local account-batch signing. */
export type PresignedBatchOperation = Extract<
  TradingSchemas["BatchOpDto"],
  { kind: "SPLIT" | "MERGE" | "WITHDRAW" }
>;
/** Batch intent paired with the market's contract route, supplied by the caller. */
export interface RoutedBatchOperation {
  /** Operation to encode, with positive integer micro amounts. */
  op: PresignedBatchOperation;
  /** Market route; `NegRisk` permits MERGE only. Ignored for WITHDRAW operations. */
  route: "Ctf" | "NegRisk";
}
/** Trusted deployment contract addresses used when encoding account-batch calls. */
export interface ComposeContext {
  /** Conditional Tokens contract used for ordinary SPLIT and MERGE operations. */
  ctf: Address;
  /** Negative-risk adapter used for MERGE operations routed to `NegRisk`. */
  negRiskAdapter: Address;
  /** Collateral token contract used for transfers and position operations. */
  collateral: Address;
}
/** Contract call included in the account's EIP-712 batch message. */
export interface BatchCall {
  /** Contract receiving the call. */
  target: Address;
  /** Native currency amount in the chain's smallest unit; composed SDK calls use zero. */
  value: bigint;
  /** ABI-encoded call data as `0x`-prefixed hexadecimal bytes. */
  data: Hex;
}
/** Deployment and account information defining an AgaraAccount EIP-712 domain. */
export interface BatchDomain {
  /** Positive chain ID fitting an unsigned 64-bit integer; number inputs must be safe integers. */
  chainId: number | bigint;
  /** Nonzero account contract address, used as both verifying contract and batch wallet. */
  account: Address;
  /** Positive unsigned 64-bit implementation version, encoded as the domain version string. */
  implementationVersion: number | bigint;
}
/** Operations and account state required to sign an account batch locally. */
export interface SignBatchInput {
  /** Trusted account address, chain ID, and current implementation version. */
  domain: BatchDomain;
  /** Trusted contract addresses for the target deployment. */
  context: ComposeContext;
  /** One through 20 supported operations, in execution order. */
  operations: readonly RoutedBatchOperation[];
  /** Nonnegative account sequence within signed 64-bit range, obtained from current account state. */
  seq: number | bigint;
  /** Signature deadline in Unix seconds within nonnegative signed 64-bit range; no clock check is made. */
  deadlineUnixSeconds: number | bigint;
  /** Optional bytes32 hash of a divergent batch this submission heals; not included in the digest. */
  healsBatchHash?: Hex;
}
/**
 * Encodes SPLIT, MERGE, and WITHDRAW operations into ordered account calls.
 *
 * @param operations - One through 20 operations with explicit Ctf/NegRisk routes.
 * @param context - Trusted deployment contract addresses.
 * @returns ABI-encoded calls in input order, each with zero native currency value.
 * @throws TypeError - If an operation, route, market ID, condition ID, or zero address is invalid.
 * @throws RangeError - If the batch length or amounts are outside supported bounds.
 * @remarks
 * SPLIT and MERGE use the root collection and binary partition `[1, 2]`; NegRisk SPLIT is
 * rejected. Share amounts must be positive signed 64-bit integers; withdrawals require at
 * least 10,000 micro collateral units. This helper does not know the wallet address and cannot
 * reject withdrawals to the wallet itself; {@link signBatch} performs that check. Invalid
 * address encodings may also throw viem errors.
 */
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
/**
 * Builds AgaraAccount EIP-712 data for an already-composed list of contract calls.
 *
 * @param domain - Trusted account, chain, and implementation version.
 * @param seq - Nonnegative account sequence within signed 64-bit range.
 * @param deadline - Unix deadline in seconds within nonnegative signed 64-bit range.
 * @param calls - Contract calls in execution order; the array is copied without validating its entries.
 * @returns Typed data binding the account, sequence, deadline, and calls.
 * @throws TypeError - If integer representations or the zero account address are invalid.
 * @throws RangeError - If a domain integer, sequence, or deadline is out of range.
 * @remarks
 * This does not fetch the current sequence, check that the deadline is in the future, or enforce
 * a call-count limit. Invalid account address encodings may also throw viem errors.
 */
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
/**
 * Computes the EIP-712 digest of an account batch.
 *
 * @param domain - Trusted account, chain, and implementation version.
 * @param seq - Account sequence bound by the signature.
 * @param deadline - Signature deadline in Unix seconds.
 * @param calls - Already-composed contract calls in execution order.
 * @returns The `0x`-prefixed 32-byte batch hash used to identify the submission.
 * @remarks
 * Validation errors from {@link batchTypedData} and viem's EIP-712 encoder propagate to the caller.
 */
export function hashBatch(
  domain: BatchDomain,
  seq: number | bigint,
  deadline: number | bigint,
  calls: readonly BatchCall[],
): Hex {
  return hashTypedData(batchTypedData(domain, seq, deadline, calls));
}
/**
 * Composes and signs an account batch, returning its submission body and reconciliation hash.
 *
 * @param input - Operations, deployment context, account sequence, deadline, and optional healing hash.
 * @param signer - Caller-owned signer authorized to sign for the account.
 * @returns The batch hash, encoded calls, and body ready for `submitBatch`.
 * @throws TypeError - If an operation withdraws to the account itself or other signing input is invalid.
 * @throws RangeError - If the operation count or numeric inputs exceed supported bounds.
 * @remarks
 * The caller supplies the live account sequence and deployment context; no network reads or
 * submissions occur here. Signer and encoding errors propagate. The optional healing hash is
 * validated after signing. Retain `batchHash` to reconcile a submission with an uncertain outcome.
 */
export async function signBatch(
  input: SignBatchInput,
  signer: TypedDataSigner,
): Promise<{
  /** EIP-712 digest identifying this batch for submission status and reconciliation. */
  batchHash: Hex;
  /** Submission body with exact bigint sequence and deadline fields. */
  body: BatchSubmission;
  /** Ordered contract calls included in the signed digest. */
  calls: readonly BatchCall[];
}> {
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
/**
 * Builds a LIMIT request from exact micro amounts and validates its order policy.
 *
 * @param input - LIMIT order fields, accepting strings or bigints for price and share quantity.
 * @returns A new request with `price_micro` and `shares_micro` converted to strings.
 * @throws TypeError - If the side, time in force, amount representation, or expiration is invalid.
 * @throws RangeError - If price or share quantities are outside supported bounds.
 * @remarks
 * This checks a future expiration for GTD and restricts post-only to GTC/GTD. Market-specific
 * grids, balance, and permissions remain server checks. No signature or HTTP request is made.
 */
export function limitOrder(
  input: Omit<LimitOrderRequest, "price_micro" | "shares_micro"> & {
    /** Limit price in millionths of a collateral unit per share, from 1 through 999,999. */
    price_micro: string | bigint;
    /** Positive share quantity in millionths, within signed 64-bit range. */
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

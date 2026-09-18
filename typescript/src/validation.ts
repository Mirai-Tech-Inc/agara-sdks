import { integer } from "./amounts.js";
import { definitions, requestContracts } from "./generated/requests.js";
import { assertSafeNumbers, isObject } from "./json.js";

interface Schema {
  const?: unknown;
  additionalProperties?: boolean | Schema;
  $ref?: string;
  type?: string | string[];
  enum?: unknown[];
  oneOf?: Schema[];
  anyOf?: Schema[];
  allOf?: Schema[];
  properties?: Record<string, Schema>;
  required?: string[] | boolean;
  items?: Schema;
  minimum?: number;
  maximum?: number;
  minItems?: number;
  maxItems?: number;
  minLength?: number;
  maxLength?: number;
  pattern?: string;
  format?: string;
}
const schemas = definitions as unknown as Record<string, Schema>;
function check(schema: Schema, value: unknown, path: string): void {
  if (schema.$ref) {
    const target = schemas[schema.$ref.split("/").at(-1) ?? ""];
    if (!target) throw new TypeError(`Unknown schema ${schema.$ref}`);
    check(target, value, path);
    return;
  }
  if (schema.oneOf || schema.anyOf) {
    let matches = 0;
    for (const choice of schema.oneOf ?? schema.anyOf ?? [])
      try {
        check(choice, value, path);
        matches++;
      } catch {}
    if (schema.oneOf ? matches !== 1 : matches < 1)
      throw new TypeError(`${path} does not match exactly one allowed shape`);
    return;
  }
  if ("const" in schema && value !== schema.const)
    throw new TypeError(`${path} differs from its constant`);
  for (const choice of schema.allOf ?? []) check(choice, value, path);
  const types = Array.isArray(schema.type) ? schema.type : [schema.type];
  if (value === null && types.includes("null")) return;
  if (schema.enum && !schema.enum.includes(value))
    throw new TypeError(`${path} has an unsupported value`);
  const type = types.find((t) => t !== "null");
  if (schema.type === "null" && value !== null) throw new TypeError(`${path} must be null`);
  if (type === "object") {
    if (!isObject(value)) throw new TypeError(`${path} must be an object`);
    for (const key of Object.keys(value))
      if (!schema.properties?.[key]) {
        if (schema.additionalProperties === false)
          throw new TypeError(`${path}.${key} is not permitted`);
        if (typeof schema.additionalProperties === "object")
          check(schema.additionalProperties, value[key], `${path}.${key}`);
      }
    for (const key of Array.isArray(schema.required) ? schema.required : [])
      if (value[key] === undefined) throw new TypeError(`${path}.${key} is required`);
    for (const [key, s] of Object.entries(schema.properties ?? {}))
      if (value[key] !== undefined) check(s, value[key], `${path}.${key}`);
  } else if (type === "array") {
    if (!Array.isArray(value)) throw new TypeError(`${path} must be an array`);
    if (value.length < (schema.minItems ?? 0) || value.length > (schema.maxItems ?? Infinity))
      throw new RangeError(`${path} has invalid length`);
    if (schema.items) for (const item of value) check(schema.items, item, path);
  } else if (type === "string") {
    if (typeof value !== "string") throw new TypeError(`${path} must be a string`);
    if (
      schema.format === "int64" &&
      (!/^-?\d+$/.test(value) || BigInt(value) < -(1n << 63n) || BigInt(value) > (1n << 63n) - 1n)
    )
      throw new TypeError(`${path} is not a signed 64-bit decimal string`);
    if (
      value.length < (schema.minLength ?? 0) ||
      value.length > (schema.maxLength ?? Infinity) ||
      (schema.pattern && !new RegExp(schema.pattern).test(value))
    )
      throw new TypeError(`${path} has invalid format`);
  } else if (type === "integer" || type === "number") {
    if (typeof value !== "number" && typeof value !== "bigint")
      throw new TypeError(`${path} must be a number or bigint`);
    if (type === "integer" && typeof value === "number" && !Number.isSafeInteger(value))
      throw new TypeError(`${path} must be an exact integer`);
    if (
      (schema.minimum !== undefined && value < schema.minimum) ||
      (schema.maximum !== undefined && value > schema.maximum)
    )
      throw new RangeError(`${path} is outside its range`);
  } else if (type === "boolean" && typeof value !== "boolean")
    throw new TypeError(`${path} must be boolean`);
}
function micro(value: unknown, name: string, maximum = (1n << 63n) - 1n): bigint {
  if (typeof value !== "string" && typeof value !== "bigint" && typeof value !== "number")
    throw new TypeError(`${name} is required`);
  return integer(value, name, maximum, 1n);
}
export function validateOrder(value: unknown, signed = false): void {
  if (!isObject(value)) throw new TypeError("Order must be an object");
  if (value.side !== "BUY" && value.side !== "SELL")
    throw new TypeError("side must be BUY or SELL");
  if (!["GTC", "GTD", "FAK", "FOK"].includes(String(value.time_in_force)))
    throw new TypeError("Invalid time_in_force");
  if (value.type === "LIMIT") {
    micro(value.price_micro, "price_micro", 999999n);
    micro(value.shares_micro, "shares_micro");
    if (value.collateral_amount_micro != null)
      throw new TypeError("LIMIT orders require shares_micro, not collateral_amount_micro");
  } else if (value.type === "MARKET" && !signed) {
    if (value.price_micro != null || !["FAK", "FOK"].includes(String(value.time_in_force)))
      throw new TypeError("MARKET orders require FAK/FOK and no price");
    micro(
      value.side === "BUY" ? value.collateral_amount_micro : value.shares_micro,
      "market amount",
    );
    if (value.side === "BUY" ? value.shares_micro != null : value.collateral_amount_micro != null)
      throw new TypeError("MARKET BUY needs collateral, SELL needs shares");
  } else throw new TypeError("Unsupported order type");
  if (value.time_in_force === "GTD") {
    if (
      typeof value.expiration_unix_seconds !== "number" ||
      !Number.isSafeInteger(value.expiration_unix_seconds) ||
      value.expiration_unix_seconds <= Date.now() / 1000
    )
      throw new TypeError("GTD needs a future expiration");
  } else if (value.expiration_unix_seconds != null) throw new TypeError("Expiration requires GTD");
  if (value.post_only && !["GTC", "GTD"].includes(String(value.time_in_force)))
    throw new TypeError("post_only requires GTC/GTD");
  if (signed) {
    if (value.side_u8 !== (value.side === "BUY" ? 0 : 1))
      throw new TypeError("Signed side does not match order side");
    const shares = micro(value.shares_micro, "shares_micro"),
      price = micro(value.price_micro, "price_micro", 999999n),
      collateral = (shares * price) / 1000000n;
    if (
      BigInt(String(value.maker_amount)) !== (value.side === "BUY" ? collateral : shares) ||
      BigInt(String(value.taker_amount)) !== (value.side === "BUY" ? shares : collateral)
    )
      throw new TypeError("Signed amounts do not match order economics");
    if (BigInt(String(value.token_id)) !== BigInt(String(value.chain_token_id)))
      throw new TypeError("Signed token does not match order token");
    integer(String(value.salt), "salt", (1n << 256n) - 1n, 1n);
    if (
      !/^0x[0-9a-fA-F]{64}$/.test(String(value.order_hash)) ||
      !/^0x[0-9a-fA-F]{130}$/.test(String(value.signature))
    )
      throw new TypeError("Malformed order hash or signature");
  }
}
export function validateRequest(name: string, body: unknown, query: unknown): void {
  assertSafeNumbers(body);
  assertSafeNumbers(query);
  const contract = (
    requestContracts as unknown as Record<string, { body?: Schema; query: Record<string, Schema> }>
  )[name];
  if (contract?.body) check(contract.body, body, "body");
  if (contract)
    for (const [key, schema] of Object.entries(contract.query)) {
      const value = isObject(query) ? query[key] : undefined;
      if (value === undefined) {
        if (schema.required === true) throw new TypeError(`query.${key} is required`);
      } else check(schema, value, `query.${key}`);
    }
  if (name === "placeOrder" || name === "placeSignedOrder")
    validateOrder(body, name === "placeSignedOrder");
  if (name === "placeSignedOrders") {
    if (
      !isObject(body) ||
      !Array.isArray(body.orders) ||
      body.orders.length < 1 ||
      body.orders.length > 32
    )
      throw new RangeError("Signed order batches need 1–32 entries");
    for (const order of body.orders) validateOrder(order, true);
  }
  if ((name === "splitPosition" || name === "mergePosition") && isObject(body))
    micro(body.collateral_amount_micro ?? body.shares_micro, "position amount");
  if ((name === "submitBatch" || name === "supersedeBatch") && isObject(body)) {
    if (!Array.isArray(body.ops) || body.ops.length < 1 || body.ops.length > 20)
      throw new RangeError("Account batches need 1–20 operations");
    for (const op of body.ops) {
      if (!isObject(op) || !["SPLIT", "MERGE", "WITHDRAW"].includes(String(op.kind)))
        throw new TypeError("Presigned batches support SPLIT, MERGE, and WITHDRAW only");
      micro(op.kind === "WITHDRAW" ? op.amount_micro : op.shares_micro, "batch amount");
    }
    if (!/^0x[0-9a-fA-F]{130}$/.test(String(body.signature)))
      throw new TypeError("Expected a 65-byte batch signature");
    if (name === "submitBatch") integer(String(body.seq), "seq", (1n << 63n) - 1n);
    integer(String(body.deadline_unix_seconds), "deadline", (1n << 63n) - 1n);
  }
}

export function validateResponse(name: string, status: number, value: unknown): void {
  const schema = (
    requestContracts as unknown as Record<string, { responses: Record<string, Schema> }>
  )[name]?.responses[String(status)];
  if (schema) check(schema, value, "response");
}

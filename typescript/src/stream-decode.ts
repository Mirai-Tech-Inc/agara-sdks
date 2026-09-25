import { ProtocolError, parseFailure } from "./errors.js";
import { problemRegistry } from "./generated/problems.js";
import { isObject, parseJson } from "./json.js";
import type { ChannelSpec, PriceStreamFrame, ServerFrame } from "./stream-types.js";

const channels = new Set(["orderbook", "best_quote", "trades", "market_status", "account_events"]);
const kinds: Record<string, readonly string[]> = {
  orderbook: ["snapshot", "delta"],
  trades: ["trade"],
  market_status: [
    "market_created",
    "market_halted",
    "market_resumed",
    "market_resolved",
    "outcome_proposed",
    "fee_policy_updated",
    "cross_match_toggled",
    "current_market_changed",
    "market_resolution_completed",
  ],
  account_events: [
    "order_accepted",
    "order_cancelled",
    "order_rejected",
    "fill",
    "tokens_minted",
    "tokens_merged",
    "tokens_redeemed",
    "collateral_deposited",
    "collateral_withdrawn",
  ],
};
function text(value: unknown, name: string): asserts value is string {
  if (typeof value !== "string") throw new ProtocolError(`${name} must be a string`);
}
function uint(value: unknown, name: string): void {
  if (
    !(
      (typeof value === "bigint" && value >= 0n) ||
      (typeof value === "number" && Number.isSafeInteger(value) && value >= 0)
    )
  )
    throw new ProtocolError(`${name} must be an exact unsigned integer`);
}
function scalar(value: unknown, name: string): void {
  if (typeof value !== "number" || !Number.isSafeInteger(value) || value < 0 || value > 0xffffffff)
    throw new ProtocolError(`${name} must be uint32`);
}
function validateSubject(raw: Record<string, unknown>): void {
  if (!channels.has(String(raw.channel))) throw new ProtocolError("Unknown subject channel", raw);
  const identifiers = ["token_id", "condition_id", "event_id"].filter((k) => raw[k] !== undefined);
  const expected = raw.channel === "account_events" ? 0 : 1;
  if (identifiers.length !== expected) throw new ProtocolError("Invalid stream subject", raw);
  for (const key of identifiers) text(raw[key], key);
  if (["orderbook", "best_quote"].includes(String(raw.channel)) && typeof raw.token_id !== "string")
    throw new ProtocolError("Missing token_id", raw);
  if (raw.channel === "trades" && typeof raw.condition_id !== "string")
    throw new ProtocolError("Missing condition_id", raw);
}
/**
 * Check the local shape of one channel subscription without sending it to the server.
 *
 * @param spec - Channel name, subject, and optional depth or account credential.
 * @throws TypeError - An account subscription has an empty token.
 * @throws ProtocolError - The channel, subject fields, or unsigned 32-bit depth are invalid.
 * @remarks
 * This does not authenticate credentials, resolve identifiers, or check server permissions.
 * Endpoint compatibility and the per-connection subscription count are checked by `AgaraStream`.
 */
export function validateSubscription(spec: ChannelSpec): void {
  const raw = spec as unknown as Record<string, unknown>;
  if (spec.name === "account_events") {
    if (!spec.token) throw new TypeError("Account subscription requires a token");
    return;
  }
  validateSubject({ ...raw, channel: spec.name });
  if (spec.name === "orderbook" && spec.depth !== undefined) scalar(spec.depth, "depth");
}
/**
 * Decode a router WebSocket frame while preserving exact integer values and future frame variants.
 *
 * @param input - JSON text or an already parsed frame; use bigint for large pre-parsed integers.
 * @returns A recognized frame, or an `unknown` frame retaining unrecognized operations, channels,
 * update kinds, or reset reasons. Server failure actions are checked against the problem registry.
 * @throws ProtocolError - A checked field in a recognized frame or its public failure is malformed.
 * @throws SyntaxError - JSON text is invalid.
 * @throws TypeError - JSON text contains a non-finite numeric value.
 * @remarks
 * Native JSON integers outside JavaScript's safe range become bigint. Pre-parsed inputs cannot
 * recover precision already lost to `JSON.parse`. Unknown failure codes retain their details but
 * cannot enable automatic recovery. This function does not enforce sequence continuity or reconcile
 * application state, and is not an exhaustive validator for every server-side domain constraint.
 */
export function decodeFrame(input: string | unknown): ServerFrame {
  const raw = typeof input === "string" ? parseJson(input) : input;
  if (!isObject(raw) || typeof raw.op !== "string")
    throw new ProtocolError("Malformed stream frame", raw);
  if (raw.op === "error") {
    const failure = parseFailure(raw.failure);
    const metadata = problemRegistry[failure.code as keyof typeof problemRegistry];
    const expected = metadata?.websocket?.action;
    const action =
      failure.known &&
      failure.recovery.strategy !== "unknown" &&
      expected === raw.action &&
      ["none", "reconnect", "resubscribe"].includes(String(raw.action))
        ? raw.action
        : "none";
    return { ...raw, failure, action } as ServerFrame;
  }
  if (raw.op === "heartbeat" || raw.op === "pong") {
    text(raw.server_time, "server_time");
    return raw as unknown as ServerFrame;
  }
  if (raw.op === "subscription_list") {
    if (!Array.isArray(raw.channels)) throw new ProtocolError("Invalid subscription list", raw);
    for (const subject of raw.channels) {
      if (!isObject(subject)) throw new ProtocolError("Invalid subject", subject);
      validateSubject(subject);
    }
    return raw as unknown as ServerFrame;
  }
  if (["subscribed", "unsubscribed", "sequence_reset"].includes(raw.op)) {
    if (!channels.has(String(raw.channel))) return { op: "unknown", raw };
    validateSubject(raw);
    if (raw.op === "sequence_reset" && !["lagged", "stream_reset"].includes(String(raw.reason)))
      return { op: "unknown", raw };
    return raw as unknown as ServerFrame;
  }
  if (raw.op !== "update" || !channels.has(String(raw.channel))) return { op: "unknown", raw };
  validateSubject(raw);
  if (!isObject(raw.data)) throw new ProtocolError("Missing update data", raw);
  const data = raw.data,
    channel = String(raw.channel);
  if (channel !== "best_quote" && !kinds[channel]?.includes(String(data.kind)))
    return { op: "unknown", raw };
  const event = channel === "market_status" && typeof raw.event_id === "string";
  if (!event) uint(raw.sequence, "sequence");
  else if (raw.sequence !== undefined)
    throw new ProtocolError("Event lifecycle has no engine sequence", raw);
  if (data.price_scale !== undefined) scalar(data.price_scale, "price_scale");
  if (data.size_scale !== undefined) scalar(data.size_scale, "size_scale");
  if (channel === "orderbook") {
    scalar(data.price_scale, "price_scale");
    scalar(data.size_scale, "size_scale");
    if (data.kind === "snapshot") scalar(data.tick_size, "tick_size");
    for (const name of ["bids", "asks"]) {
      const levels = data[name];
      if (!Array.isArray(levels)) throw new ProtocolError("Missing book levels", raw);
      for (const level of levels) {
        if (!Array.isArray(level) || level.length !== 2)
          throw new ProtocolError("Invalid book level", level);
        scalar(level[0], "price");
        uint(level[1], "size");
      }
    }
  } else if (channel === "best_quote") {
    scalar(data.price_scale, "price_scale");
    scalar(data.size_scale, "size_scale");
    for (const name of ["bid", "ask"]) {
      const quote = data[name];
      if (quote !== null) {
        if (!isObject(quote)) throw new ProtocolError("Invalid quote", raw);
        scalar(quote.price, "price");
        uint(quote.size, "size");
      }
    }
  } else if (channel === "account_events") {
    if ((data.batch_hash === undefined) !== (data.batch_index === undefined))
      throw new ProtocolError("Incomplete batch provenance", raw);
    if (data.batch_hash !== undefined) {
      text(data.batch_hash, "batch_hash");
      scalar(data.batch_index, "batch_index");
    }
    if (data.kind === "order_rejected") {
      text(data.order_id, "order_id");
      for (const name of ["order_hash", "token_id"])
        if (data[name] !== null) text(data[name], name);
      return { ...raw, data: { ...data, failure: parseFailure(data.failure) } } as ServerFrame;
    }
    if (["order_accepted", "order_cancelled", "fill"].includes(String(data.kind))) {
      for (const key of ["order_id", "order_hash", "token_id", "side"]) text(data[key], key);
      scalar(data.price, "price");
      scalar(data.price_scale, "price_scale");
      scalar(data.size_scale, "size_scale");
    }
    if (data.kind === "fill") {
      text(data.fill_id, "fill_id");
      text(data.fee_micro, "fee_micro");
      text(data.role, "role");
      text(data.settlement_mode, "settlement_mode");
      uint(data.size, "size");
    }
    if (data.kind === "order_accepted") {
      text(data.tif, "tif");
      uint(data.original_size, "original_size");
      uint(data.remaining_size, "remaining_size");
    }
    if (data.kind === "order_cancelled") {
      text(data.reason, "reason");
      uint(data.remaining_size, "remaining_size");
    }
    if (data.kind === "tokens_minted" || data.kind === "tokens_merged") {
      text(data.condition_id, "condition_id");
      uint(data.size, "size");
      scalar(data.size_scale, "size_scale");
    }
    if (data.kind === "tokens_redeemed") {
      for (const key of [
        "condition_id",
        "winning_token_id",
        "losing_token_id",
        "winning_shares_redeemed",
        "losing_shares_zeroed",
        "payout_micro",
      ])
        text(data[key], key);
      scalar(data.size_scale, "size_scale");
    }
    if (data.kind === "collateral_deposited" || data.kind === "collateral_withdrawn") {
      text(data.amount_micro, "amount_micro");
      text(data.cash_balance_micro, "cash_balance_micro");
    }
  } else if (channel === "trades") {
    for (const key of ["fill_id", "taker_token_id", "maker_token_id", "side", "settlement_mode"])
      text(data[key], key);
    scalar(data.price, "price");
    uint(data.size, "size");
    scalar(data.price_scale, "price_scale");
    scalar(data.size_scale, "size_scale");
  } else if (event) {
    validateLifecycle(data);
  } else {
    if (["current_market_changed", "market_resolution_completed"].includes(String(data.kind)))
      throw new ProtocolError("Event data requires event_id", raw);
    if (data.kind === "market_created") {
      for (const key of [
        "num_outcomes",
        "tick_size",
        "price_scale",
        "size_scale",
        "min_price",
        "max_price",
      ])
        scalar(data[key], key);
      if (typeof data.cross_match_enabled !== "boolean")
        throw new ProtocolError("Missing cross_match_enabled", raw);
    }
    if (data.kind === "market_resolved") text(data.winning_token_id, "winning_token_id");
    if (data.kind === "outcome_proposed") text(data.proposed_token_id, "proposed_token_id");
    if (data.kind === "cross_match_toggled" && typeof data.enabled !== "boolean")
      throw new ProtocolError("Missing enabled", raw);
  }
  return raw as unknown as ServerFrame;
}
function validateLifecycle(data: Record<string, unknown>): void {
  function market(value: unknown): asserts value is Record<string, unknown> {
    if (!isObject(value)) throw new ProtocolError("Invalid lifecycle market", value);
    for (const key of ["market_id", "condition_id", "state"]) text(value[key], key);
    for (const key of ["reference_price", "currency", "reference_at", "observe_at"])
      if (value[key] !== null) text(value[key], key);
    if (!Array.isArray(value.outcomes) || value.outcomes.length < 2)
      throw new ProtocolError("Invalid lifecycle outcomes", value);
    for (const item of value.outcomes) {
      if (!isObject(item)) throw new ProtocolError("Invalid outcome", item);
      for (const key of ["token_id", "kind", "label"]) text(item[key], key);
      if (item.short_label !== null) text(item.short_label, "short_label");
    }
  }
  if (data.kind === "current_market_changed") {
    text(data.changed_at, "changed_at");
    text(data.main_market_id, "main_market_id");
    market(data.market);
    if (data.market.market_id !== data.main_market_id || data.market.state !== "ACTIVE")
      throw new ProtocolError("Current market must be active main market", data);
  } else if (data.kind === "market_resolution_completed") {
    for (const key of [
      "resolved_market_id",
      "resolved_condition_id",
      "winning_token_id",
      "resolved_at",
    ])
      text(data[key], key);
    if (data.main_market_id !== null) text(data.main_market_id, "main_market_id");
    if (data.replacement_status === "active" || data.replacement_status === "upcoming") {
      market(data.replacement);
      if (
        data.replacement.market_id === data.resolved_market_id ||
        (data.replacement_status === "active" &&
          (data.replacement.state !== "ACTIVE" ||
            data.replacement.market_id !== data.main_market_id)) ||
        (data.replacement_status === "upcoming" && data.replacement.state !== "PROVISIONED")
      )
        throw new ProtocolError("Invalid lifecycle replacement", data);
    } else if (data.replacement_status === "expected") {
      if (data.replacement !== null || !isObject(data.expected_cycle))
        throw new ProtocolError("Invalid expected cycle", data);
      text(data.expected_cycle.reference_at, "reference_at");
      text(data.expected_cycle.observe_at, "observe_at");
    } else if (data.replacement_status !== "none" || data.replacement !== null)
      throw new ProtocolError("Unknown lifecycle replacement", data);
  } else throw new ProtocolError("Invalid event lifecycle kind", data);
}
/**
 * Decode the provider price payload carried in an SSE event's data field.
 *
 * @param input - JSON text containing a `parsed` price-entry array.
 * @returns Entries whose price mantissas remain strings, with integer decimal exponents and Unix
 * publication timestamps in milliseconds.
 * @throws ProtocolError - The entry shape, integer price string, exponent, or timestamp is invalid.
 * @throws SyntaxError - The input is not valid JSON.
 * @throws TypeError - JSON text contains a non-finite numeric value.
 */
export function decodePriceFrame(input: string): PriceStreamFrame {
  const raw = parseJson(input);
  if (!isObject(raw) || !Array.isArray(raw.parsed))
    throw new ProtocolError("Invalid price stream frame", raw);
  for (const entry of raw.parsed) {
    if (!isObject(entry) || !isObject(entry.price))
      throw new ProtocolError("Invalid price entry", entry);
    text(entry.id, "id");
    text(entry.price.price, "price");
    if (
      !/^-?\d+$/.test(entry.price.price) ||
      typeof entry.price.expo !== "number" ||
      !Number.isSafeInteger(entry.price.expo) ||
      typeof entry.price.publish_time_ms !== "number" ||
      !Number.isSafeInteger(entry.price.publish_time_ms)
    )
      throw new ProtocolError("Invalid price value", entry);
  }
  return raw as unknown as PriceStreamFrame;
}

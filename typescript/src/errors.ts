import { problemRegistry } from "./generated/problems.js";
import { isObject } from "./json.js";
export type RecoveryResource =
  | { kind: "order"; order_id: string }
  | { kind: "batch"; batch_hash: string }
  | { kind: "group"; group_id: string }
  | { kind: "wallet_status" };
export type Recovery =
  | { strategy: "none" }
  | { strategy: "retry" }
  | { strategy: "retry_after"; after_seconds: number }
  | { strategy: "refresh_identity_token" }
  | { strategy: "check_status"; resource: RecoveryResource }
  | { strategy: "unknown"; raw: unknown };
export interface PublicFailure {
  code: string;
  title: string;
  detail?: string;
  recovery: Recovery;
  raw: unknown;
  known: boolean;
}
export interface FieldError {
  path: (string | number)[];
  code: string;
  message: string;
}
export interface ProblemDetails extends PublicFailure {
  type: string;
  status: number;
  request_id?: string;
  field_errors?: FieldError[];
}
const uuid =
  /^(?:[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}|00000000-0000-0000-0000-000000000000|ffffffff-ffff-ffff-ffff-ffffffffffff)$/i;
const hash = /^0x[0-9a-f]{64}$/i;
export function parseRecovery(raw: unknown): Recovery {
  if (!isObject(raw) || typeof raw.strategy !== "string") return { strategy: "unknown", raw };
  const keys = Object.keys(raw);
  if (["none", "retry", "refresh_identity_token"].includes(raw.strategy) && keys.length === 1)
    return { strategy: raw.strategy } as Recovery;
  if (
    raw.strategy === "retry_after" &&
    keys.length === 2 &&
    typeof raw.after_seconds === "number" &&
    Number.isInteger(raw.after_seconds) &&
    raw.after_seconds >= 0 &&
    raw.after_seconds <= 86400
  )
    return { strategy: "retry_after", after_seconds: raw.after_seconds };
  if (raw.strategy === "check_status" && keys.length === 2 && isObject(raw.resource)) {
    const r = raw.resource;
    const valid =
      r.kind === "wallet_status"
        ? Object.keys(r).length === 1
        : Object.keys(r).length === 2 &&
          (r.kind === "order"
            ? typeof r.order_id === "string" && uuid.test(r.order_id)
            : r.kind === "batch"
              ? typeof r.batch_hash === "string" && hash.test(r.batch_hash)
              : r.kind === "group" && typeof r.group_id === "string" && uuid.test(r.group_id));
    if (valid) return { strategy: "check_status", resource: r as RecoveryResource };
  }
  return { strategy: "unknown", raw };
}
function bounded(value: unknown, max: number, min = 1): value is string {
  return typeof value === "string" && [...value].length >= min && [...value].length <= max;
}
function identifier(value: unknown): value is string {
  return typeof value === "string" && /^[a-z][a-z0-9_]{0,63}$/.test(value);
}
function validFieldError(value: unknown): value is FieldError {
  return (
    isObject(value) &&
    Object.keys(value).every((k) => ["path", "code", "message"].includes(k)) &&
    Array.isArray(value.path) &&
    value.path.length <= 16 &&
    value.path.every(
      (v) =>
        bounded(v, 64) ||
        (typeof v === "number" && Number.isInteger(v) && v >= 0 && v <= 2147483647),
    ) &&
    identifier(value.code) &&
    bounded(value.message, 256)
  );
}
export function parseFailure(raw: unknown): PublicFailure {
  if (
    !isObject(raw) ||
    !identifier(raw.code) ||
    !bounded(raw.title, 128) ||
    (raw.detail !== undefined && !bounded(raw.detail, 512)) ||
    !isObject(raw.recovery) ||
    !identifier(raw.recovery.strategy) ||
    Object.keys(raw.recovery).length > 8
  )
    throw new ProtocolError("Malformed public failure", raw);
  const metadata = problemRegistry[raw.code as keyof typeof problemRegistry];
  let recovery = parseRecovery(raw.recovery);
  const known = metadata !== undefined;
  if (known) {
    if (
      raw.title !== metadata.title ||
      (raw.detail !== undefined && raw.detail !== metadata.public_detail)
    )
      throw new ProtocolError("Noncanonical public failure text", raw);
    if (
      recovery.strategy === "unknown" ||
      recovery.strategy !== metadata.recovery.strategy ||
      (recovery.strategy === "check_status" &&
        ("resource_kind" in metadata.recovery
          ? metadata.recovery.resource_kind !== recovery.resource.kind
          : true))
    )
      throw new ProtocolError("Invalid recovery for problem code", raw);
  } else recovery = { strategy: "unknown", raw: raw.recovery };
  return {
    code: raw.code,
    title: raw.title,
    ...(typeof raw.detail === "string" ? { detail: raw.detail } : {}),
    recovery,
    raw,
    known,
  };
}
export function parseProblem(
  raw: unknown,
  status: number,
  representation: "origin" | "edge" | "auto" = "auto",
): ProblemDetails | undefined {
  if (
    !isObject(raw) ||
    typeof raw.type !== "string" ||
    raw.status !== status ||
    !Number.isInteger(status) ||
    status < 400 ||
    status > 599
  )
    return undefined;
  if (
    Object.keys(raw).some(
      (k) =>
        ![
          "type",
          "title",
          "status",
          "code",
          "detail",
          "request_id",
          "recovery",
          "field_errors",
        ].includes(k),
    )
  )
    return undefined;
  if (
    (representation === "origin" && typeof raw.request_id !== "string") ||
    (representation === "edge" && (raw.request_id !== undefined || raw.field_errors !== undefined))
  )
    return undefined;
  if (
    raw.request_id !== undefined &&
    (typeof raw.request_id !== "string" || !uuid.test(raw.request_id))
  )
    return undefined;
  if (
    raw.field_errors !== undefined &&
    (!Array.isArray(raw.field_errors) ||
      raw.field_errors.length > 32 ||
      !raw.field_errors.every(validFieldError))
  )
    return undefined;
  try {
    const failure = parseFailure(raw);
    const metadata = problemRegistry[failure.code as keyof typeof problemRegistry];
    if (
      raw.type !== `urn:agara:problem:${failure.code.replaceAll("_", "-")}` ||
      (metadata && metadata.http_status !== status)
    )
      return undefined;
    return {
      ...failure,
      type: raw.type,
      status,
      ...(typeof raw.request_id === "string" ? { request_id: raw.request_id } : {}),
      ...(Array.isArray(raw.field_errors)
        ? { field_errors: raw.field_errors as FieldError[] }
        : {}),
    };
  } catch {
    return undefined;
  }
}
export class ProtocolError extends Error {
  constructor(
    message: string,
    public readonly raw?: unknown,
  ) {
    super(message);
    this.name = "ProtocolError";
  }
}
export class AgaraError extends Error {
  readonly problem: ProblemDetails | undefined;
  readonly retryAfter: number | undefined;
  constructor(
    public readonly status: number,
    public readonly body: unknown,
    public readonly headers: Headers = new Headers(),
  ) {
    const problem = parseProblem(body, status);
    super(problem?.detail ?? problem?.title ?? `Agara HTTP ${status}`);
    this.name = "AgaraError";
    this.problem = problem;
    const hint = headers.get("retry-after");
    const seconds = hint === null ? NaN : Number(hint);
    this.retryAfter =
      Number.isFinite(seconds) && seconds >= 0
        ? seconds
        : problem?.recovery.strategy === "retry_after"
          ? problem.recovery.after_seconds
          : undefined;
  }
  get code() {
    return this.problem?.code;
  }
  get requestId() {
    return this.problem?.request_id ?? this.headers.get("x-request-id") ?? undefined;
  }
  get recovery() {
    return this.problem?.recovery;
  }
  get isRetryable() {
    return this.recovery?.strategy === "retry" || this.recovery?.strategy === "retry_after";
  }
}
export class TransportError extends Error {
  constructor(
    message: string,
    public readonly outcomeUnknown: boolean,
    options?: ErrorOptions,
  ) {
    super(message, options);
    this.name = "TransportError";
  }
}
export class WaitTimeoutError<T = unknown> extends Error {
  constructor(public readonly latest: T | undefined) {
    super("Timed out waiting for completion");
    this.name = "WaitTimeoutError";
  }
}
export class PaginationError extends Error {}
export class PartialAvailabilityError extends Error {
  constructor(public readonly unavailableExchanges: readonly string[]) {
    super(`Unavailable exchanges: ${unavailableExchanges.join(", ")}`);
  }
}

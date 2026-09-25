import { problemRegistry } from "./generated/problems.js";
import { isObject } from "./json.js";
/** Resource to inspect before deciding whether to repeat an operation with an uncertain outcome. */
export type RecoveryResource =
  | {
      /** Inspect the identified order. */
      kind: "order";
      /** Order UUID returned by the server. */
      order_id: string;
    }
  | {
      /** Inspect the identified account batch. */
      kind: "batch";
      /** EIP-712 batch digest as a `0x`-prefixed 32-byte hexadecimal string. */
      batch_hash: string;
    }
  | {
      /** Inspect the identified batch group. */
      kind: "group";
      /** Batch group UUID returned by the server. */
      group_id: string;
    }
  | {
      /** Inspect the authenticated wallet's current status. */
      kind: "wallet_status";
    };
/**
 * Validated recovery guidance attached to a public failure.
 *
 * @remarks
 * Guidance does not itself execute a retry. Unknown guidance must not authorize automatic action.
 */
export type Recovery =
  | {
      /** No automatic recovery is provided. */
      strategy: "none";
    }
  | {
      /** The failure permits retrying the operation. */
      strategy: "retry";
    }
  | {
      /** The failure permits retrying after the indicated delay. */
      strategy: "retry_after";
      /** Minimum delay in whole seconds, from zero through 86,400. */
      after_seconds: number;
    }
  | {
      /** Obtain a fresh identity token before trying again. */
      strategy: "refresh_identity_token";
    }
  | {
      /** Inspect the resource's current state before deciding what to do next. */
      strategy: "check_status";
      /** Resource whose state can resolve the uncertain outcome. */
      resource: RecoveryResource;
    }
  | {
      /** Guidance is unrecognized or cannot be safely validated. */
      strategy: "unknown";
      /** Original guidance retained for inspection. */
      raw: unknown;
    };
/** Public failure decoded from HTTP problem details or a streaming update. */
export interface PublicFailure {
  /** Machine-readable failure identifier. */
  code: string;
  /** Human-readable summary, checked against the registry for known codes. */
  title: string;
  /** Optional explanation, checked against the registry for known codes. */
  detail?: string;
  /** Validated guidance; unknown failure codes always receive the `unknown` strategy. */
  recovery: Recovery;
  /** Original decoded failure object. */
  raw: unknown;
  /** Whether this SDK's checked-in problem registry recognizes the code. */
  known: boolean;
}
/** Validation failure associated with a field in a submitted request. */
export interface FieldError {
  /** Field names and array indexes leading to the invalid value; empty means the root. */
  path: (string | number)[];
  /** Machine-readable validation identifier. */
  code: string;
  /** Human-readable explanation of the field failure. */
  message: string;
}
/** Validated HTTP problem details, including the original public failure and recovery guidance. */
export interface ProblemDetails extends PublicFailure {
  /** Problem URI in the form `urn:agara:problem:<hyphenated-code>`. */
  type: string;
  /** HTTP error status, from 400 through 599, matching the response status. */
  status: number;
  /** Origin request UUID, when supplied. */
  request_id?: string;
  /** Request field failures, when supplied by the origin. */
  field_errors?: FieldError[];
}
const uuid =
  /^(?:[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}|00000000-0000-0000-0000-000000000000|ffffffff-ffff-ffff-ffff-ffffffffffff)$/i;
const hash = /^0x[0-9a-f]{64}$/i;
/**
 * Recognizes a recovery object's supported shape and validates its identifiers and delay.
 *
 * @param raw - Decoded recovery value from a server response.
 * @returns Recognized guidance, or an `unknown` result retaining the original value.
 * @remarks
 * This checks the shape only. Use {@link parseFailure} to also check guidance against a problem code.
 */
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
/**
 * Decodes a public failure and verifies known codes against the SDK's problem registry.
 *
 * @param raw - Decoded failure object, including `code`, `title`, and `recovery`.
 * @returns A failure retaining the raw input; unknown codes receive `unknown` recovery guidance.
 * @throws ProtocolError - If required fields are malformed or known text or recovery is noncanonical.
 */
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
/**
 * Validates an HTTP problem document against its response status and representation.
 *
 * @param raw - Decoded response body.
 * @param status - Actual HTTP response status, expected to be from 400 through 599.
 * @param representation - `origin` requires a request ID; `edge` forbids request IDs and field errors;
 * `auto`, the default, accepts either shape.
 * @returns Validated details, or `undefined` for a malformed or inconsistent problem document.
 */
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
/**
 * Received data violates a supported protocol shape or a configured protocol limit.
 *
 * @remarks
 * This error does not establish whether a submitted mutation completed; reconcile its state
 * before repeating it.
 */
export class ProtocolError extends Error {
  /**
   * Creates a protocol failure with optional diagnostic data.
   *
   * @param message - Explanation of the violated contract or limit.
   * @param raw - Original value or underlying error, when available.
   */
  constructor(
    message: string,
    /** Original value or underlying error retained for inspection. */
    public readonly raw?: unknown,
  ) {
    super(message);
    this.name = "ProtocolError";
  }
}
/** Non-successful HTTP response, with validated problem details when the body supplies them. */
export class AgaraError extends Error {
  /** Validated problem details, or `undefined` if the body is not a valid problem document. */
  readonly problem: ProblemDetails | undefined;
  /**
   * Retry delay in seconds from a nonnegative numeric `Retry-After` header, falling back to
   * validated `retry_after` guidance. HTTP-date headers are not parsed.
   */
  readonly retryAfter: number | undefined;
  /**
   * Creates an HTTP failure and parses its problem document and retry hint.
   *
   * @param status - Actual HTTP status code.
   * @param body - Decoded response body, retained even when problem validation fails.
   * @param headers - Response headers; defaults to an empty collection.
   */
  constructor(
    /** Actual HTTP status code. */
    public readonly status: number,
    /** Decoded response body, whether or not it is a recognized problem document. */
    public readonly body: unknown,
    /** Response headers, including request ID and retry hints when present. */
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
  /** Problem code, or `undefined` when the response lacks valid problem details. */
  get code() {
    return this.problem?.code;
  }
  /** Problem request ID, falling back to `x-request-id`, or `undefined` if neither is present. */
  get requestId() {
    return this.problem?.request_id ?? this.headers.get("x-request-id") ?? undefined;
  }
  /** Validated recovery guidance, or `undefined` when no problem document was recognized. */
  get recovery() {
    return this.problem?.recovery;
  }
  /** Whether validated guidance permits a retry; a retry header alone does not make this true. */
  get isRetryable() {
    return this.recovery?.strategy === "retry" || this.recovery?.strategy === "retry_after";
  }
}
/**
 * Request transport failed or was aborted without a usable result.
 *
 * @remarks
 * When `outcomeUnknown` is true, reconcile the mutation using its order or batch identifier
 * before deciding whether to resubmit. The SDK does not automatically retry HTTP mutations.
 */
export class TransportError extends Error {
  /**
   * Creates a transport failure, optionally retaining the underlying error as its cause.
   *
   * @param message - Description of the transport failure.
   * @param outcomeUnknown - Whether the failed request was a mutation that may still complete.
   * @param options - Standard error options, including an optional cause.
   */
  constructor(
    message: string,
    /** True for failed mutations whose completion cannot be inferred from the transport error. */
    public readonly outcomeUnknown: boolean,
    options?: ErrorOptions,
  ) {
    super(message, options);
    this.name = "TransportError";
  }
}
/**
 * A polling workflow reached its timeout before observing completion.
 *
 * @typeParam T - Response type returned by the polling read.
 * @remarks
 * Timing out does not cancel the underlying operation or imply that it failed.
 */
export class WaitTimeoutError<T = unknown> extends Error {
  /**
   * Creates a timeout retaining the most recent successfully observed state.
   *
   * @param latest - Last polling response, or `undefined` if no read succeeded.
   */
  constructor(
    /** Last successfully observed response, if any; it may describe an unfinished operation. */
    public readonly latest: T | undefined,
  ) {
    super("Timed out waiting for completion");
    this.name = "WaitTimeoutError";
  }
}
/** Pagination encountered an invalid or repeated cursor, or exceeded its configured page limit. */
export class PaginationError extends Error {}
/** A completeness check rejected a snapshot because one or more exchanges were unavailable. */
export class PartialAvailabilityError extends Error {
  /**
   * Creates a failure identifying the missing exchanges.
   *
   * @param unavailableExchanges - Exchanges whose data could not be included in the snapshot.
   */
  constructor(
    /** Exchanges missing from the snapshot; their absence must not be treated as empty holdings. */
    public readonly unavailableExchanges: readonly string[],
  ) {
    super(`Unavailable exchanges: ${unavailableExchanges.join(", ")}`);
  }
}

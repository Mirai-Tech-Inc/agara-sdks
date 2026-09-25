import { AgaraError, ProtocolError, TransportError } from "./errors.js";
import { parseJson, stringifyJson } from "./json.js";
import { validateRequest, validateResponse } from "./validation.js";
/** Per-request cancellation, deadline and successful-response observation. */
export interface RequestOptions {
  /** Cancels this request; aborting a sent mutation does not establish that it failed. */
  signal?: AbortSignal;
  /**
   * Deadline in milliseconds, overriding the client default; zero disables this request's timer.
   * Polling helpers use this field as the overall positive wait deadline instead.
   *
   * @defaultValue The client's timeout, normally 10000 milliseconds.
   */
  timeoutMs?: number;
  /**
   * Runs synchronously after a successful response has been decoded and validated,
   * following the client-level observer. Throwing rejects with `ResponseObserverError`;
   * its retained response does not authorize replaying a completed mutation.
   */
  onResponse?: (metadata: ResponseMetadata) => void;
}
/** HTTP evidence available to successful-response observers. */
export interface ResponseMetadata {
  /** Actual HTTP response status, including asynchronous acceptance statuses. */
  status: number;
  /** Response headers, including any rate-limit or server trace metadata. */
  headers: Headers;
  /** Value of the `x-request-id` response header, or null when absent. */
  requestId: string | null;
}
/** Configuration shared by public and authenticated HTTP clients. */
export interface ClientOptions {
  /**
   * Trading API base URL using HTTP(S), without credentials, query or fragment.
   *
   * @defaultValue `"https://app.sandbox.agara.xyz"`
   */
  baseUrl?: string;
  /**
   * Optional separate origin/base path for catalogue requests under `/api/`.
   * Accepts the same URL shape as `baseUrl`; no bearer token is sent to catalogue routes.
   *
   * @defaultValue `baseUrl`
   */
  catalogueBaseUrl?: string;
  /** Personal access token sent to `/trade/` routes; required for `TraderClient` and `AgaraClient`. */
  token?: string;
  /**
   * Fetch-compatible transport, for example to configure certificates or record requests.
   * It must honor AbortSignal and `redirect: "error"`; requests are never automatically retried.
   *
   * @defaultValue `globalThis.fetch`
   */
  fetch?: typeof fetch;
  /**
   * Positive, finite default request deadline in milliseconds, including response-body reads.
   *
   * @defaultValue 10000
   */
  timeoutMs?: number;
  /**
   * Positive integer maximum response-body bytes; larger bodies reject with `ProtocolError`.
   *
   * @defaultValue 16777216 (16 MiB).
   */
  maxResponseBytes?: number;
  /**
   * Synchronous observer for successfully decoded and validated responses, before any per-request
   * observer. Throwing rejects with `ResponseObserverError` retaining the decoded response.
   */
  onResponse?: (metadata: ResponseMetadata) => void;
}
export interface AbortScope {
  signal: AbortSignal;
  dispose: () => void;
}
export function abortScope(signal: AbortSignal | undefined, timeoutMs: number): AbortScope {
  if (!Number.isFinite(timeoutMs) || timeoutMs < 0)
    throw new RangeError("timeoutMs must be nonnegative and finite");
  const controller = new AbortController();
  const abort = () => controller.abort(signal?.reason);
  if (signal?.aborted) abort();
  else signal?.addEventListener("abort", abort, { once: true });
  const timer = timeoutMs
    ? setTimeout(
        () => controller.abort(new DOMException("Request timed out", "TimeoutError")),
        timeoutMs,
      )
    : undefined;
  return {
    signal: controller.signal,
    dispose: () => {
      if (timer !== undefined) clearTimeout(timer);
      signal?.removeEventListener("abort", abort);
    },
  };
}
export async function readResponse(
  response: Response,
  maxBytes: number,
  signal?: AbortSignal,
): Promise<unknown> {
  if (!response.body) return undefined;
  const reader = response.body.getReader();
  const decoder = new TextDecoder();
  let size = 0,
    text = "";
  try {
    while (true) {
      const { done, value } = await abortable(reader.read(), signal, () => {
        void reader.cancel().catch(() => {});
      });
      if (done) break;
      size += value.byteLength;
      if (size > maxBytes) {
        await reader.cancel();
        throw new ProtocolError("Response exceeds maxResponseBytes");
      }
      text += decoder.decode(value, { stream: true });
    }
    text += decoder.decode();
  } finally {
    reader.releaseLock();
  }
  if (!text) return undefined;
  try {
    return parseJson(text);
  } catch (error) {
    if (response.ok) throw new ProtocolError("Successful response is not valid JSON", error);
    return text;
  }
}
export class Transport {
  protected readonly fetcher: typeof fetch;
  protected readonly baseUrl: string;
  protected readonly catalogueBaseUrl: string;
  protected readonly token: string | undefined;
  protected readonly timeoutMs: number;
  protected readonly maxResponseBytes: number;
  private readonly observer: ClientOptions["onResponse"];
  /**
   * Configure the HTTP transport inherited by the public and authenticated client constructors.
   *
   * @param options - Service URLs, bearer, transport, deadlines and response observers.
   * @throws `TypeError` for invalid service URLs or access-token text.
   * @throws `RangeError` for a nonpositive or nonfinite default timeout or invalid byte limit.
   */
  constructor(options: ClientOptions = {}) {
    this.baseUrl = (options.baseUrl ?? "https://app.sandbox.agara.xyz").replace(/\/$/, "");
    this.catalogueBaseUrl = (options.catalogueBaseUrl ?? this.baseUrl).replace(/\/$/, "");
    for (const value of [this.baseUrl, this.catalogueBaseUrl]) {
      const url = new URL(value);
      if (
        !["https:", "http:"].includes(url.protocol) ||
        url.username ||
        url.password ||
        url.search ||
        url.hash
      )
        throw new TypeError("Base URLs must be HTTP(S) without credentials, query, or fragment");
    }
    if (options.token !== undefined && (!options.token.trim() || /[\r\n]/.test(options.token)))
      throw new TypeError("Invalid access token");
    this.token = options.token;
    this.fetcher = options.fetch ?? globalThis.fetch;
    this.timeoutMs = options.timeoutMs ?? 10000;
    this.maxResponseBytes = options.maxResponseBytes ?? 16 * 1024 * 1024;
    this.observer = options.onResponse;
    if (
      !Number.isFinite(this.timeoutMs) ||
      this.timeoutMs <= 0 ||
      !Number.isSafeInteger(this.maxResponseBytes) ||
      this.maxResponseBytes <= 0
    )
      throw new RangeError("timeoutMs and maxResponseBytes must be positive and finite");
  }
  protected pathPart(value: string): string {
    if (typeof value !== "string" || !value.length || value === "." || value === "..")
      throw new TypeError("Path identifier is required");
    return encodeURIComponent(value);
  }
  protected async request<T>(
    method: string,
    path: string,
    body: unknown,
    query: unknown,
    options: RequestOptions,
    requiresAuth: boolean,
    name: string,
  ): Promise<T> {
    if (requiresAuth && !this.token)
      throw new TypeError(`${name} requires a personal access token`);
    validateRequest(name, body, query);
    const mutation =
      method !== "GET" &&
      method !== "HEAD" &&
      !["listOrders", "listOpenOrders", "listPositions", "quoteBridgeWithdrawal"].includes(name);
    const url = new URL(
      `${path.startsWith("/api/") ? this.catalogueBaseUrl : this.baseUrl}${path}`,
    );
    if (query && typeof query === "object")
      for (const [key, value] of Object.entries(query))
        if (value !== undefined && value !== null)
          url.searchParams.set(key, Array.isArray(value) ? value.join(",") : String(value));
    const scope = abortScope(options.signal, options.timeoutMs ?? this.timeoutMs);
    const headers = new Headers({ Accept: "application/json" });
    if (this.token && path.startsWith("/trade/"))
      headers.set("Authorization", `Bearer ${this.token}`);
    if (body !== undefined) headers.set("Content-Type", "application/json");
    let response: Response;
    try {
      response = await abortable(
        this.fetcher(url, {
          method,
          headers,
          signal: scope.signal,
          redirect: "error",
          ...(body !== undefined ? { body: stringifyJson(body) } : {}),
        }),
        scope.signal,
      );
      const metadata = {
        status: response.status,
        headers: response.headers,
        requestId: response.headers.get("x-request-id"),
      };

      const value = await readResponse(response, this.maxResponseBytes, scope.signal);
      if (!response.ok) throw new AgaraError(response.status, value, response.headers);
      try {
        validateResponse(name, response.status, value);
      } catch (error) {
        throw new ProtocolError("Response does not match its contract", error);
      }
      try {
        this.observer?.(metadata);
        options.onResponse?.(metadata);
      } catch (error) {
        throw new ResponseObserverError(metadata, value, { cause: error });
      }
      return value as T;
    } catch (error) {
      if (
        error instanceof AgaraError ||
        error instanceof ProtocolError ||
        error instanceof ResponseObserverError
      )
        throw error;
      throw new TransportError(
        scope.signal.aborted
          ? "Request aborted; a submitted mutation may still complete"
          : "Request failed",
        mutation,
        { cause: error },
      );
    } finally {
      scope.dispose();
    }
  }
}

/**
 * An observer threw after the server returned a valid success response.
 *
 * @remarks
 * Inspect `responseBody` before deciding what happened; the operation may already have completed.
 * `cause` retains the observer's original exception. The SDK does not replay the request.
 */
export class ResponseObserverError extends Error {
  /**
   * Retain the accepted response alongside the observer's original exception.
   *
   * @param metadata - Status, headers and request ID of the successful response.
   * @param responseBody - Already decoded and validated successful response envelope.
   * @param options - Error options whose cause is the observer's original exception.
   */
  constructor(
    public readonly metadata: ResponseMetadata,
    public readonly responseBody: unknown,
    options: ErrorOptions,
  ) {
    super("Response observer failed after receiving a response", options);
    this.name = "ResponseObserverError";
  }
}
export function abortable<T>(
  work: Promise<T>,
  signal?: AbortSignal,
  onAbort?: () => void,
): Promise<T> {
  if (!signal) return work;
  return new Promise<T>((resolve, reject) => {
    const abort = () => {
      onAbort?.();
      reject(signal.reason ?? new DOMException("Aborted", "AbortError"));
    };
    if (signal.aborted) abort();
    else signal.addEventListener("abort", abort, { once: true });
    work.then(resolve, reject).finally(() => signal.removeEventListener("abort", abort));
  });
}

import { AgaraError, ProtocolError } from "./errors.js";
import { decodePriceFrame } from "./stream-decode.js";
import type { PriceStreamFrame } from "./stream-types.js";
import { abortable, abortScope, readResponse } from "./transport.js";
import { delay } from "./workflows.js";
/** A dispatched SSE event with its latest ID and retry metadata. */
export interface SseEvent {
  /** Data lines joined with newline characters, without the final separator. */
  data: string;
  /** Event type; defaults to `message` for each event. */
  event: string;
  /** Most recently supplied valid ID, retained across events in this connection. */
  id?: string;
  /** Most recently supplied retry hint in milliseconds, retained across events. */
  retry?: number;
}

/** A decoded price payload and its SSE event metadata. */
export interface PriceEvent {
  /** Provider prices with exact mantissas, decimal exponents, and millisecond timestamps. */
  frame: PriceStreamFrame;
  /** SSE event type; defaults to `message`. */
  event: string;
  /** SSE event ID, when supplied; its presence does not guarantee replay support. */
  id?: string;
}

/** HTTP, cancellation, parser limits, and retry settings shared by both price feeds. */
export interface PriceStreamOptions {
  /** HTTP(S) service base URL without credentials; defaults to `https://app.sandbox.agara.xyz`. */
  baseUrl?: string;
  /** Fetch implementation; defaults to `globalThis.fetch`. Redirects are rejected. */
  fetch?: typeof fetch;
  /** Cancels connection attempts, pending reads, and retry waits with the signal's abort reason. */
  signal?: AbortSignal;
  /** Handshake timeout in milliseconds; finite and nonnegative, default 10,000, zero disables. */
  connectTimeoutMs?: number;
  /** Timeout per pending body read in milliseconds; finite and nonnegative, default 45,000, zero disables. */
  idleTimeoutMs?: number;
  /** UTF-8 byte limit per line and accumulated data including separators; defaults to 1,048,576. */
  maxEventBytes?: number;
  /** Reconnect after EOF or eligible errors; defaults to true. Protocol errors remain terminal. */
  reconnect?: boolean;
  /** Lifetime reconnect limit, not reset by receiving prices; defaults to 8. Zero disables retries. */
  maxReconnects?: number;
  /** Initial retry delay in milliseconds; defaults to 1,000 and may be replaced by an SSE retry hint. */
  reconnectDelayMs?: number;
  /**
   * Maximum accepted server SSE retry or HTTP Retry-After hint in milliseconds; defaults to 60,000.
   * This is an acceptance limit; exponential retry waits are separately capped at 60,000 milliseconds.
   */
  maxRetryDelayMs?: number;
  /** Called with the one-based lifetime reconnect count before each retry wait. */
  onReconnect?: (attempt: number) => void;
}

/**
 * Parse UTF-8 byte chunks into SSE events without interpreting their data as JSON.
 *
 * @param chunks - Byte chunks from a single SSE connection.
 * @param maxEventBytes - Byte limit for each line and accumulated data including newline separators;
 * defaults to 1,048,576.
 * @returns Events with data, dispatched at blank lines; incomplete final events are discarded.
 * @throws ProtocolError - A line or accumulated event data exceeds the byte limit.
 * @remarks
 * Handles chunk boundaries, CR/LF delimiters, comments, and multiline data. IDs and retry hints
 * persist within this parser invocation; event names reset to `message` after each blank line.
 */
export async function* parseSse(
  chunks: AsyncIterable<Uint8Array>,
  maxEventBytes = 1048576,
): AsyncGenerator<SseEvent> {
  const decoder = new TextDecoder();
  let buffer = "",
    data: string[] = [],
    event = "message",
    id: string | undefined,
    retry: number | undefined,
    size = 0;
  for await (const chunk of chunks) {
    buffer += decoder.decode(chunk, { stream: true });

    while (true) {
      const end = buffer.search(/[\r\n]/);
      if (end < 0 || (buffer[end] === "\r" && end === buffer.length - 1)) break;
      const line = buffer.slice(0, end);
      if (new TextEncoder().encode(line).byteLength > maxEventBytes)
        throw new ProtocolError("SSE line exceeds maxEventBytes");
      const width = buffer[end] === "\r" && buffer[end + 1] === "\n" ? 2 : 1;
      buffer = buffer.slice(end + width);
      if (line === "") {
        if (data.length)
          yield {
            data: data.join("\n"),
            event,
            ...(id === undefined ? {} : { id }),
            ...(retry === undefined ? {} : { retry }),
          };
        data = [];
        event = "message";
        size = 0;
        continue;
      }
      if (line.startsWith(":")) continue;
      const split = line.indexOf(":"),
        field = split < 0 ? line : line.slice(0, split);
      let value = split < 0 ? "" : line.slice(split + 1);
      if (value.startsWith(" ")) value = value.slice(1);
      if (field === "data") {
        size += new TextEncoder().encode(value).byteLength + 1;
        if (size > maxEventBytes) throw new ProtocolError("SSE event exceeds maxEventBytes");
        data.push(value);
      } else if (field === "event") event = value;
      else if (field === "id" && !value.includes("\0")) id = value;
      else if (field === "retry" && /^\d+$/.test(value) && Number.isSafeInteger(Number(value)))
        retry = Number(value);
    }
    if (new TextEncoder().encode(buffer).byteLength > maxEventBytes)
      throw new ProtocolError("SSE line exceeds maxEventBytes");
  }
  if (buffer === "\r" && data.length)
    yield {
      data: data.join("\n"),
      event,
      ...(id === undefined ? {} : { id }),
      ...(retry === undefined ? {} : { retry }),
    };
}
async function* chunks(
  response: Response,
  options: PriceStreamOptions,
): AsyncGenerator<Uint8Array> {
  if (!response.body) throw new ProtocolError("SSE response has no body");
  const reader = response.body.getReader();
  try {
    while (true) {
      const scope = abortScope(options.signal, options.idleTimeoutMs ?? 45000);
      try {
        const result = await abortable(reader.read(), scope.signal, () => {
          void reader.cancel().catch(() => {});
        });
        if (result.done) return;
        yield result.value;
      } finally {
        scope.dispose();
      }
    }
  } finally {
    await reader.cancel().catch(() => {});
    reader.releaseLock();
  }
}
/**
 * Stream provider prices for one or more symbols over SSE.
 *
 * @param symbols - One to 64 distinct provider symbols, such as `Crypto.BTC/USD`; duplicates are removed.
 * Each symbol must contain 1–64 letters, digits, dots, underscores, slashes, or hyphens.
 * @param options - HTTP, cancellation, event-size, and reconnection settings.
 * @returns A lazy async generator; validation and the first request occur on the first read.
 * @throws TypeError - Symbols or the service URL are invalid.
 * @throws AgaraError - An HTTP failure is not retried, including excessive Retry-After hints.
 * @throws ProtocolError - An SSE size limit or retry-hint limit is exceeded, price fields are invalid,
 * or the reconnect budget is exhausted.
 * @remarks
 * Connection/read failures and EOF may reconnect with exponential delay and a lifetime retry budget.
 * HTTP failures retry only when recognized recovery permits it. The last nonempty SSE ID is sent as
 * `Last-Event-ID` on reconnect; missed-price replay is not guaranteed. Returning from iteration
 * releases the active response reader; use an abort signal to interrupt a pending read. Cancellation
 * and unretried fetch/read failures propagate their original errors.
 */
export async function* priceStream(
  symbols: readonly string[],
  options: PriceStreamOptions = {},
): AsyncGenerator<PriceEvent> {
  yield* stream("/api/v1/prices/stream", "symbols", symbols, options);
}
/**
 * Stream the Pyth Pro price feed for a single provider symbol over SSE.
 *
 * @param symbol - Provider symbol of 1–64 letters, digits, dots, underscores, slashes, or hyphens.
 * @param options - HTTP, cancellation, event-size, and reconnection settings.
 * @returns A lazy async generator with the same retry and cancellation behavior as {@link priceStream}.
 * @throws TypeError - The symbol or service URL is invalid.
 * @throws AgaraError - An HTTP failure is not retried, including excessive Retry-After hints.
 * @throws ProtocolError - An SSE size limit or retry-hint limit is exceeded, price fields are invalid,
 * or the reconnect budget is exhausted.
 * @remarks
 * Requests start on the first read. Reconnection does not guarantee replay of missed prices.
 * Use an abort signal to interrupt pending reads; unretried transport errors propagate unchanged.
 */
export async function* pythProStream(
  symbol: string,
  options: PriceStreamOptions = {},
): AsyncGenerator<PriceEvent> {
  yield* stream("/api/v1/prices/pyth-pro/stream", "symbol", [symbol], options);
}
async function* stream(
  path: string,
  key: string,
  symbols: readonly string[],
  options: PriceStreamOptions,
): AsyncGenerator<PriceEvent> {
  const unique = [...new Set(symbols)];
  if (
    !unique.length ||
    unique.length > 64 ||
    unique.some((s) => !/^[A-Za-z0-9._/-]{1,64}$/.test(s))
  )
    throw new TypeError("Expected 1–64 provider symbols, e.g. Crypto.BTC/USD");
  const fetcher = options.fetch ?? globalThis.fetch;
  let attempts = 0;
  let lastId: string | undefined;
  let reconnectDelay = options.reconnectDelayMs ?? 1000;
  const url = new URL(
    `${(options.baseUrl ?? "https://app.sandbox.agara.xyz").replace(/\/$/, "")}${path}`,
  );
  if (!["http:", "https:"].includes(url.protocol) || url.username || url.password)
    throw new TypeError("SSE URL must be HTTP(S) without credentials");
  url.searchParams.set(key, unique.join(","));
  while (true) {
    options.signal?.throwIfAborted();
    const handshake = abortScope(options.signal, options.connectTimeoutMs ?? 10000);
    let retry = true;
    try {
      const response = await abortable(
        fetcher(url, {
          headers: { Accept: "text/event-stream", ...(lastId ? { "Last-Event-ID": lastId } : {}) },
          signal: handshake.signal,
          redirect: "error",
        }),
        handshake.signal,
      );
      if (!response.ok)
        throw new AgaraError(
          response.status,
          await readResponse(response, 1048576, handshake.signal),
          response.headers,
        );
      handshake.dispose();
      if (!response.headers.get("content-type")?.includes("text/event-stream"))
        throw new ProtocolError("Expected text/event-stream");
      for await (const event of parseSse(
        chunks(response, options),
        options.maxEventBytes ?? 1048576,
      )) {
        if (event.id !== undefined) lastId = event.id;
        if (event.retry !== undefined) {
          if (event.retry > (options.maxRetryDelayMs ?? 60000))
            throw new ProtocolError("SSE retry delay exceeds configured maximum");
          reconnectDelay = event.retry;
        }
        yield {
          frame: decodePriceFrame(event.data),
          event: event.event,
          ...(event.id === undefined ? {} : { id: event.id }),
        };
      }
    } catch (error) {
      options.signal?.throwIfAborted();
      if (error instanceof ProtocolError) throw error;
      if (error instanceof AgaraError) {
        retry = error.isRetryable;
        if ((error.retryAfter ?? 0) * 1000 > (options.maxRetryDelayMs ?? 60000)) throw error;
        reconnectDelay = Math.max(reconnectDelay, (error.retryAfter ?? 0) * 1000);
      }
      if (!retry || options.reconnect === false) throw error;
    } finally {
      handshake.dispose();
    }
    if (options.reconnect === false) return;
    if (++attempts > (options.maxReconnects ?? 8))
      throw new ProtocolError("SSE reconnect limit exceeded");
    options.onReconnect?.(attempts);
    await delay(Math.min(reconnectDelay * 2 ** (attempts - 1), 60000), options.signal);
  }
}

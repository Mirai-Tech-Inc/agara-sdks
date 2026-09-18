import { AgaraError, ProtocolError } from "./errors.js";
import { decodePriceFrame } from "./stream-decode.js";
import type { PriceStreamFrame } from "./stream-types.js";
import { abortable, abortScope, readResponse } from "./transport.js";
import { delay } from "./workflows.js";
export interface SseEvent {
  data: string;
  event: string;
  id?: string;
  retry?: number;
}
export interface PriceEvent {
  frame: PriceStreamFrame;
  event: string;
  id?: string;
}
export interface PriceStreamOptions {
  baseUrl?: string;
  fetch?: typeof fetch;
  signal?: AbortSignal;
  connectTimeoutMs?: number;
  idleTimeoutMs?: number;
  maxEventBytes?: number;
  reconnect?: boolean;
  maxReconnects?: number;
  reconnectDelayMs?: number;
  maxRetryDelayMs?: number;
  onReconnect?: (attempt: number) => void;
}
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
export async function* priceStream(
  symbols: readonly string[],
  options: PriceStreamOptions = {},
): AsyncGenerator<PriceEvent> {
  yield* stream("/api/v1/prices/stream", "symbols", symbols, options);
}
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

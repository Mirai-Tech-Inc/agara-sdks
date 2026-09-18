import { ProtocolError, type PublicFailure } from "./errors.js";
import { stringifyJson } from "./json.js";
import { decodeFrame, validateSubscription } from "./stream-decode.js";
import type { ChannelSpec, ClientMessage, ServerFrame, Subject } from "./stream-types.js";
/** Minimal browser-compatible WebSocket interface accepted by an injected factory. */
export interface WebSocketLike {
  /** Standard WebSocket ready state; `1` means open. */
  readonly readyState: number;
  /**
   * Send a JSON text message on the connection.
   *
   * @param data - Serialized message to send.
   */
  send(data: string): void;
  /**
   * Request connection closure.
   *
   * @param code - Optional WebSocket close code.
   * @param reason - Optional close reason.
   */
  close(code?: number, reason?: string): void;
  /**
   * Register an open, message, close, or error listener.
   *
   * @param type - WebSocket event name.
   * @param listener - Listener receiving browser-compatible events.
   */
  addEventListener(type: string, listener: EventListener): void;
  /**
   * Remove a previously registered listener.
   *
   * @param type - WebSocket event name.
   * @param listener - Listener to remove.
   */
  removeEventListener(type: string, listener: EventListener): void;
}

/** Connection notices, decoded frames, and recovery obligations emitted by {@link AgaraStream}. */
export type StreamEvent =
  | {
      /** A decoded server message. */
      type: "frame";
      /** Data, control, failure, or unrecognized frame received from the server. */
      frame: ServerFrame;
    }
  | {
      /** A connection attempt or successful open. */
      type: "connection";
      /** `reconnecting` means an earlier socket opened; `open` precedes subscription confirmation. */
      state: "connecting" | "open" | "reconnecting";
      /** Lifetime reconnect count; the initial attempt is zero. */
      attempt: number;
    }
  | {
      /** Reconcile this subject before relying on its local state. */
      type: "gap";
      /** Connection loss, server reset, or server-requested resubscription caused the gap. */
      reason: "reconnect" | "sequence_reset" | "resubscribe";
      /** Affected subscription subject, when supplied. */
      subject?: Subject;
      /**
       * Best quotes await fresh state; orderbooks need a REST fence and a fresh snapshot;
       * account, trade, and lifecycle state must be reconciled through REST by the application.
       */
      recovery: "await_snapshot" | "reconcile_rest" | "rest_then_snapshot";
    };

/** WebSocket connection, subscription, queue, and reconnection settings. */
export interface StreamOptions {
  /**
   * Service base URL; defaults to `https://app.sandbox.agara.xyz`.
   * HTTP(S) becomes WS(S), the endpoint path is appended, and query/fragment are removed.
   * Embedded credentials are rejected when iteration starts.
   */
  baseUrl?: string;
  /** Router endpoint; defaults to `market`. Account channels require `account`. */
  endpoint?: "market" | "account";
  /** Initial subscriptions; at most 64 remain after duplicate serialized specs are removed. */
  channels: readonly ChannelSpec[];
  /** Creates a socket for each attempt; defaults to the global `WebSocket` constructor. */
  webSocketFactory?: (url: string) => WebSocketLike;
  /** Stops the stream normally on abort; already queued events remain readable. */
  signal?: AbortSignal;
  /** Maximum unread events of all types; positive safe integer, default 1,024. */
  maxQueueSize?: number;
  /** Lifetime reconnect limit; nonnegative safe integer, default 8. Zero disables retries. */
  maxReconnects?: number;
  /** Initial exponential-backoff ceiling in milliseconds; nonnegative safe integer, default 250. */
  reconnectBaseMs?: number;
  /** Backoff ceiling before jitter in milliseconds; nonnegative safe integer, default 10,000. */
  reconnectMaxMs?: number;
  /** Time to open a socket in milliseconds; nonnegative safe integer, default 10,000, zero disables. */
  connectTimeoutMs?: number;
  /** Time without any message in milliseconds; nonnegative safe integer, default 90,000, zero disables. */
  idleTimeoutMs?: number;
  /** Jitter source returning a value from zero to one; defaults to `Math.random`. */
  random?: () => number;
}

/** The unread event queue overflowed; create a new stream and reconcile state before trading. */
export class StreamBackpressureError extends Error {
  /** Create the terminal local backpressure error. */
  constructor() {
    super("Stream consumer exceeded the bounded queue; reconnect and reconcile before trading");
    this.name = "StreamBackpressureError";
  }
}

/** A socket is unavailable, closed terminally, or exhausted its reconnect budget. */
export class StreamClosedError extends Error {
  /**
   * Create a socket-closure error, optionally retaining a server authentication failure.
   *
   * @param code - Observed close code, or the SDK's synthetic closure code.
   * @param failure - Server failure associated with terminal authentication rejection, if supplied.
   */
  constructor(
    /** WebSocket close code; `1006` also represents sending without an open socket. */
    public readonly code: number,
    /** Server authentication failure, when the SDK received one before stopping. */
    public readonly failure?: PublicFailure,
  ) {
    super(`WebSocket closed with code ${code}`);
    this.name = "StreamClosedError";
  }
}
class Queue<T> implements AsyncIterableIterator<T> {
  private values: T[] = [];
  private waiting: ((result: IteratorResult<T>) => void) | undefined;
  private reject: ((error: unknown) => void) | undefined;
  private done = false;
  private error: unknown;
  constructor(
    private readonly capacity: number,
    private readonly stop: () => void,
  ) {}
  push(value: T) {
    if (this.done) return;
    if (this.waiting) {
      this.waiting({ value, done: false });
      this.waiting = undefined;
      this.reject = undefined;
    } else if (this.values.length >= this.capacity) throw new StreamBackpressureError();
    else this.values.push(value);
  }
  finish(error?: unknown) {
    this.done = true;
    this.error = error;
    if (error) {
      this.values = [];
      this.reject?.(error);
    } else this.waiting?.({ value: undefined, done: true });
    this.waiting = undefined;
    this.reject = undefined;
  }
  next(): Promise<IteratorResult<T>> {
    if (this.error) return Promise.reject(this.error);
    const value = this.values.shift();
    if (value !== undefined) return Promise.resolve({ value, done: false });
    if (this.done) return Promise.resolve({ value: undefined, done: true });
    if (this.waiting) return Promise.reject(new Error("Only one stream consumer is allowed"));
    return new Promise((resolve, reject) => {
      this.waiting = resolve;
      this.reject = reject;
    });
  }
  return(): Promise<IteratorResult<T>> {
    this.stop();
    return Promise.resolve({ value: undefined, done: true });
  }
  [Symbol.asyncIterator]() {
    return this;
  }
}
/**
 * A lazy, single-consumer WebSocket stream with bounded buffering and reconnects.
 *
 * @remarks
 * Obtaining its async iterator starts the connection. Socket opens replay the current subscriptions;
 * successful connections do not reset the reconnect budget. Recognized server recovery actions may
 * reconnect or resubscribe automatically. Authentication failures stop without refreshing credentials.
 * Gap events require application reconciliation; this class does not rebuild books, replay missed
 * fills, or detect gaps from sequence jumps. Terminal errors discard unread events and reject reads.
 *
 * @example
 * ```ts
 * const stream = marketStream({ channels: [{ name: "best_quote", token_id: tokenId }] });
 * try {
 *   for await (const event of stream) {
 *     if (event.type === "frame") console.log(event.frame);
 *   }
 * } finally {
 *   stream.close();
 * }
 * ```
 */
export class AgaraStream implements AsyncIterable<StreamEvent> {
  private readonly queue: Queue<StreamEvent>;
  private socket: WebSocketLike | undefined;
  private readonly factory: (url: string) => WebSocketLike;
  private readonly endpoint: "market" | "account";
  private channels: ChannelSpec[] = [];
  private started = false;
  private stopped = false;
  private attempt = 0;
  private hadConnection = false;
  private reconnectTimer: ReturnType<typeof setTimeout> | undefined;
  private timeout: ReturnType<typeof setTimeout> | undefined;
  private readonly abort = () => this.close();
  private cleanupSocket: (() => void) | undefined;
  /**
   * Validate configuration and store subscriptions without opening a socket.
   *
   * @param options - Connection settings and initial subscriptions.
   * @throws RangeError - Queue, timeout, reconnect, or subscription-count limits are invalid.
   * @throws TypeError - An account token is missing or a channel belongs to the other endpoint.
   * @throws ProtocolError - A subscription subject or depth is malformed.
   */
  constructor(private readonly options: StreamOptions) {
    this.endpoint = options.endpoint ?? "market";
    this.factory = options.webSocketFactory ?? ((url) => new WebSocket(url));
    const capacity = options.maxQueueSize ?? 1024;
    if (!Number.isSafeInteger(capacity) || capacity < 1)
      throw new RangeError("maxQueueSize must be positive");
    for (const [key, value] of Object.entries({
      maxReconnects: options.maxReconnects ?? 8,
      reconnectBaseMs: options.reconnectBaseMs ?? 250,
      reconnectMaxMs: options.reconnectMaxMs ?? 10000,
      connectTimeoutMs: options.connectTimeoutMs ?? 10000,
      idleTimeoutMs: options.idleTimeoutMs ?? 90000,
    }))
      if (!Number.isSafeInteger(value) || value < 0) throw new RangeError(`Invalid ${key}`);
    this.queue = new Queue(capacity, () => this.close());
    this.setSubscriptions(options.channels);
    options.signal?.addEventListener("abort", this.abort, { once: true });
    if (options.signal?.aborted) this.close();
  }
  /**
   * Start the first connection and return the shared event iterator.
   *
   * @returns The single-consumer iterator; do not make concurrent pending `next()` calls.
   * @throws TypeError - The base URL is invalid when the first connection starts.
   * @remarks
   * Reads reject on backpressure, malformed frames, terminal closure, or exhausted retries.
   * Breaking a `for await` loop closes the stream. Reacquiring an iterator does not restart it.
   */
  [Symbol.asyncIterator](): AsyncIterableIterator<StreamEvent> {
    if (!this.started && !this.stopped) {
      this.started = true;
      this.connect();
    }
    return this.queue;
  }
  /**
   * Replace desired subscriptions and reconnect if the socket is currently open.
   *
   * @param channels - Subscriptions for this stream's endpoint; duplicate serialized specs are removed.
   * @throws TypeError - An account token is missing or a channel belongs to the other endpoint.
   * @throws ProtocolError - A subscription subject or depth is malformed.
   * @throws RangeError - More than 64 subscriptions remain after deduplication.
   * @remarks
   * Reconnection consumes the normal retry budget and emits recovery gaps after a prior open.
   * The count limit is checked after storing the new list; after that error, replace it with a valid
   * list or close the stream before it reconnects.
   */
  setSubscriptions(channels: readonly ChannelSpec[]): void {
    const next = channels.map((spec) => {
      validateSubscription(spec);
      if ((spec.name === "account_events") !== (this.endpoint === "account"))
        throw new TypeError("Subscription belongs to another WebSocket endpoint");
      return { ...spec };
    });
    const keys = new Set<string>();
    this.channels = next.filter((spec) => {
      const key = stringifyJson(spec);
      if (keys.has(key)) return false;
      keys.add(key);
      return true;
    });
    if (this.channels.length > 64) throw new RangeError("At most 64 subscriptions per connection");
    if (this.socket?.readyState === 1) {
      this.socket.close(1000, "Subscriptions changed");
    }
  }
  /**
   * Request a pong, delivered as a later frame event.
   *
   * @throws StreamClosedError - No socket is currently open, including before iteration starts.
   */
  ping(): void {
    this.send({ op: "ping" });
  }
  /**
   * Request the server's active subscriptions, delivered as a later `subscription_list` frame.
   *
   * @throws StreamClosedError - No socket is currently open, including before iteration starts.
   */
  listSubscriptions(): void {
    this.send({ op: "list" });
  }
  /**
   * Permanently stop the stream and release its socket, timers, and abort listener.
   *
   * @remarks
   * Repeated calls are harmless. Normal closure, including signal abort, lets iteration drain queued
   * events before finishing; it does not throw the signal's abort reason.
   */
  close(): void {
    this.stop();
  }
  private stop(error?: unknown): void {
    if (this.stopped) return;
    this.stopped = true;
    if (this.reconnectTimer) clearTimeout(this.reconnectTimer);
    if (this.timeout) clearTimeout(this.timeout);
    this.options.signal?.removeEventListener("abort", this.abort);
    this.cleanupSocket?.();
    this.socket?.close(1000, "Client closed");
    this.queue.finish(error);
  }
  private emit(event: StreamEvent) {
    try {
      this.queue.push(event);
    } catch (error) {
      this.fail(error);
    }
  }
  private fail(error: unknown) {
    this.stop(error);
  }
  private send(message: ClientMessage) {
    if (this.socket?.readyState !== 1) throw new StreamClosedError(1006);
    this.socket.send(stringifyJson(message));
  }
  private armTimeout(ms: number) {
    if (this.timeout) clearTimeout(this.timeout);
    if (ms > 0) this.timeout = setTimeout(() => this.socket?.close(4000, "Stream deadline"), ms);
  }
  private connect(): void {
    if (this.stopped) return;
    this.emit({
      type: "connection",
      state: this.hadConnection ? "reconnecting" : "connecting",
      attempt: this.attempt,
    });
    if (this.stopped) return;
    const url = new URL(this.options.baseUrl ?? "https://app.sandbox.agara.xyz");
    if (!["http:", "https:", "ws:", "wss:"].includes(url.protocol) || url.username || url.password)
      throw new TypeError("Invalid stream URL");
    url.protocol = ["http:", "ws:"].includes(url.protocol) ? "ws:" : "wss:";
    url.pathname = `${url.pathname.replace(/\/$/, "")}/trade/v1/${this.endpoint}-stream`;
    url.search = "";
    url.hash = "";
    let socket: WebSocketLike;
    try {
      socket = this.factory(url.toString());
    } catch (error) {
      this.reconnect(error);
      return;
    }
    this.socket = socket;
    const open: EventListener = () => {
      if (this.stopped) return;
      this.hadConnection = true;
      this.emit({ type: "connection", state: "open", attempt: this.attempt });
      if (this.stopped) return;
      this.send({ op: "subscribe", channels: this.channels });
      this.armTimeout(this.options.idleTimeoutMs ?? 90000);
    };
    const message: EventListener = (event) => {
      if (this.stopped) return;
      this.armTimeout(this.options.idleTimeoutMs ?? 90000);
      try {
        const data = (event as MessageEvent<unknown>).data;
        if (typeof data !== "string") throw new ProtocolError("Expected WebSocket text frame");
        const frame = decodeFrame(data);
        this.emit({ type: "frame", frame });
        if (this.stopped) return;
        if (frame.op === "sequence_reset")
          this.emit({
            type: "gap",
            reason: "sequence_reset",
            subject: frame,
            recovery: recoveryFor(frame.channel),
          });
        if (frame.op === "error") {
          if (
            frame.failure.code === "identity_token_expired" ||
            frame.failure.code === "token_expired" ||
            frame.failure.code === "token_revoked" ||
            frame.failure.code === "credentials_missing" ||
            frame.failure.code === "forbidden_scope"
          ) {
            this.fail(new StreamClosedError(4001, frame.failure));
            return;
          }
          if (frame.action === "resubscribe") {
            const affected = this.channels.filter(
              (spec) =>
                frame.channel === undefined ||
                (spec.name === frame.channel &&
                  (!frame.token_id || ("token_id" in spec && spec.token_id === frame.token_id)) &&
                  (!frame.condition_id ||
                    ("condition_id" in spec && spec.condition_id === frame.condition_id)) &&
                  (!frame.event_id || ("event_id" in spec && spec.event_id === frame.event_id))),
            );
            if (affected.length) {
              for (const spec of affected)
                this.emit({
                  type: "gap",
                  reason: "resubscribe",
                  subject: subjectFor(spec),
                  recovery: recoveryFor(spec.name),
                });
              this.send({ op: "unsubscribe", channels: affected });
              this.send({ op: "subscribe", channels: affected });
            }
          } else if (frame.action === "reconnect") socket.close(4000, "Server requested reconnect");
        }
      } catch (error) {
        this.fail(error);
      }
    };
    const close: EventListener = (event) => {
      this.cleanupSocket?.();
      if (this.timeout) clearTimeout(this.timeout);
      if (this.stopped) return;
      const code = (event as CloseEvent).code;
      if ([1002, 1003, 1007, 1008, 1009, 4001].includes(code)) {
        this.fail(new StreamClosedError(code));
        return;
      }
      this.reconnect(new StreamClosedError(code));
    };
    const error: EventListener = () => {
      socket.close();
    };
    for (const [type, listener] of [
      ["open", open],
      ["message", message],
      ["close", close],
      ["error", error],
    ] as const)
      socket.addEventListener(type, listener);
    this.cleanupSocket = () => {
      for (const [type, listener] of [
        ["open", open],
        ["message", message],
        ["close", close],
        ["error", error],
      ] as const)
        socket.removeEventListener(type, listener);
    };
    this.armTimeout(this.options.connectTimeoutMs ?? 10000);
  }
  private reconnect(error: unknown) {
    if (this.stopped) return;
    if (++this.attempt > (this.options.maxReconnects ?? 8)) {
      this.fail(error);
      return;
    }
    if (this.hadConnection)
      for (const spec of this.channels)
        this.emit({
          type: "gap",
          reason: "reconnect",
          subject: subjectFor(spec),
          recovery: recoveryFor(spec.name),
        });
    const base = Math.min(
      (this.options.reconnectBaseMs ?? 250) * 2 ** (this.attempt - 1),
      this.options.reconnectMaxMs ?? 10000,
    );
    const delay = base * (0.5 + (this.options.random ?? Math.random)() / 2);
    this.reconnectTimer = setTimeout(() => this.connect(), delay);
  }
}
/**
 * Create a lazy public stream for market channels.
 *
 * @param options - Connection settings and market subscriptions.
 * @returns A stream that connects when its async iterator is obtained.
 * @throws RangeError - A numeric option or subscription count is invalid.
 * @throws TypeError - An account subscription is supplied.
 * @throws ProtocolError - A market subscription is malformed.
 */
export function marketStream(options: Omit<StreamOptions, "endpoint">): AgaraStream {
  return new AgaraStream({ ...options, endpoint: "market" });
}
/**
 * Create a lazy private stream for the account identified by a bearer credential.
 *
 * @param options - Connection settings and credential sent in the subscription frame.
 * @returns A stream that connects when its async iterator is obtained.
 * @throws RangeError - A numeric option is invalid.
 * @throws TypeError - The token is empty.
 * @remarks
 * Reconnects reuse the token. Terminal authentication failures require the caller to obtain valid
 * credentials and create a new stream, then reconcile account state through REST.
 */
export function accountStream(
  options: Omit<StreamOptions, "endpoint" | "channels"> & {
    /** Bearer credential authorizing the account-events subscription. */
    token: string;
  },
): AgaraStream {
  return new AgaraStream({
    ...options,
    endpoint: "account",
    channels: [{ name: "account_events", token: options.token }],
  });
}

function subjectFor(spec: ChannelSpec): Subject {
  return {
    channel: spec.name,
    ...("token_id" in spec ? { token_id: spec.token_id } : {}),
    ...("condition_id" in spec && spec.condition_id !== undefined
      ? { condition_id: spec.condition_id }
      : {}),
    ...("event_id" in spec && spec.event_id !== undefined ? { event_id: spec.event_id } : {}),
  };
}
function recoveryFor(channel: string): "await_snapshot" | "reconcile_rest" | "rest_then_snapshot" {
  return channel === "orderbook"
    ? "rest_then_snapshot"
    : channel === "best_quote"
      ? "await_snapshot"
      : "reconcile_rest";
}

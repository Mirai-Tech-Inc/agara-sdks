import { ProtocolError, type PublicFailure } from "./errors.js";
import { stringifyJson } from "./json.js";
import { decodeFrame, validateSubscription } from "./stream-decode.js";
import type { ChannelSpec, ClientMessage, ServerFrame, Subject } from "./stream-types.js";
export interface WebSocketLike {
  readonly readyState: number;
  send(data: string): void;
  close(code?: number, reason?: string): void;
  addEventListener(type: string, listener: EventListener): void;
  removeEventListener(type: string, listener: EventListener): void;
}
export type StreamEvent =
  | { type: "frame"; frame: ServerFrame }
  | { type: "connection"; state: "connecting" | "open" | "reconnecting"; attempt: number }
  | {
      type: "gap";
      reason: "reconnect" | "sequence_reset" | "resubscribe";
      subject?: Subject;
      recovery: "await_snapshot" | "reconcile_rest" | "rest_then_snapshot";
    };
export interface StreamOptions {
  baseUrl?: string;
  endpoint?: "market" | "account";
  channels: readonly ChannelSpec[];
  webSocketFactory?: (url: string) => WebSocketLike;
  signal?: AbortSignal;
  maxQueueSize?: number;
  maxReconnects?: number;
  reconnectBaseMs?: number;
  reconnectMaxMs?: number;
  connectTimeoutMs?: number;
  idleTimeoutMs?: number;
  random?: () => number;
}
export class StreamBackpressureError extends Error {
  constructor() {
    super("Stream consumer exceeded the bounded queue; reconnect and reconcile before trading");
    this.name = "StreamBackpressureError";
  }
}
export class StreamClosedError extends Error {
  constructor(
    public readonly code: number,
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
  [Symbol.asyncIterator](): AsyncIterableIterator<StreamEvent> {
    if (!this.started && !this.stopped) {
      this.started = true;
      this.connect();
    }
    return this.queue;
  }
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
  ping(): void {
    this.send({ op: "ping" });
  }
  listSubscriptions(): void {
    this.send({ op: "list" });
  }
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
export function marketStream(options: Omit<StreamOptions, "endpoint">): AgaraStream {
  return new AgaraStream({ ...options, endpoint: "market" });
}
export function accountStream(
  options: Omit<StreamOptions, "endpoint" | "channels"> & { token: string },
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

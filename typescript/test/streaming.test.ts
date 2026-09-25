import { afterEach, expect, it, vi } from "vitest";
import { ProtocolError } from "../src/errors.js";
import {
  AgaraStream,
  accountStream,
  decodeFrame,
  marketStream,
  parseSse,
  priceStream,
  pythProStream,
  StreamBackpressureError,
  StreamClosedError,
  type WebSocketLike,
} from "../src/streaming.js";
import { contract } from "./helpers.js";

const fixtures = contract("ws-fixtures") as unknown as {
  name: string;
  value: Record<string, unknown>;
}[];
afterEach(() => vi.useRealTimers());
for (const fixture of fixtures)
  it(`decodes producer ${fixture.name}`, () => {
    const result = decodeFrame(fixture.value);
    expect(result.op).toBe(fixture.value.op);
    if (fixture.name.startsWith("future websocket") && result.op === "error")
      expect(result.action).toBe("none");
  });
it("preserves a future update and huge sequence without truncation", () => {
  expect(
    decodeFrame(
      '{"op":"update","channel":"trades","condition_id":"x","sequence":9007199254740993,"data":{"kind":"future"}}',
    ),
  ).toMatchObject({ op: "unknown", raw: { sequence: 9007199254740993n } });
  const frame = fixtures.find((f) => f.name === "tokens_minted");
  if (!frame) throw Error();
  expect(decodeFrame({ ...frame.value, sequence: 9007199254740993n })).toMatchObject({
    sequence: 9007199254740993n,
  });
});
it("rejects partial batch provenance and malformed known recovery", () => {
  const frame = fixtures.find((f) => f.name === "tokens_minted");
  if (!frame) throw Error();
  const data = { ...(frame.value.data as object), batch_index: undefined };
  expect(() => decodeFrame({ ...frame.value, data })).toThrow(ProtocolError);
  const error = fixtures.find((f) => f.name === "event-scoped stream unavailable frame");
  if (!error) throw Error();
  expect(() =>
    decodeFrame({
      ...error.value,
      failure: { ...(error.value.failure as object), recovery: { strategy: "retry_after" } },
    }),
  ).toThrow(ProtocolError);
});
class Socket extends EventTarget implements WebSocketLike {
  readyState = 0;
  sent: string[] = [];
  closed = false;
  open() {
    this.readyState = 1;
    this.dispatchEvent(new Event("open"));
  }
  message(value: unknown) {
    this.dispatchEvent(new MessageEvent("message", { data: JSON.stringify(value) }));
  }
  send(data: string) {
    this.sent.push(data);
  }
  close(code = 1000) {
    if (this.closed) return;
    this.closed = true;
    this.readyState = 3;
    this.dispatchEvent(Object.assign(new Event("close"), { code }));
  }
}
function sockets() {
  const created: Socket[] = [];
  return {
    created,
    factory: () => {
      const socket = new Socket();
      created.push(socket);
      return socket;
    },
  };
}
it("subscribes, handles error resubscribe, and closes on consumer cancellation", async () => {
  const { created, factory } = sockets();
  const stream = marketStream({
    channels: [{ name: "market_status", event_id: "10000000-0000-4000-8000-000000000001" }],
    webSocketFactory: factory,
  });
  const iterator = stream[Symbol.asyncIterator]();
  await iterator.next();
  const socket = created[0];
  if (!socket) throw Error();
  socket.open();
  await iterator.next();
  expect(JSON.parse(socket.sent[0] ?? "")).toMatchObject({
    op: "subscribe",
    channels: [{ event_id: "10000000-0000-4000-8000-000000000001" }],
  });
  socket.message(fixtures.find((f) => f.name === "event-scoped stream unavailable frame")?.value);
  await iterator.next();
  expect(socket.sent.slice(1).map((s) => JSON.parse(s).op)).toEqual(["unsubscribe", "subscribe"]);
  await iterator.return?.();
  expect(socket.closed).toBe(true);
});
it("bounds unread events and signals backpressure", async () => {
  const { created, factory } = sockets();
  const stream = marketStream({ channels: [], webSocketFactory: factory, maxQueueSize: 1 });
  const iterator = stream[Symbol.asyncIterator]();
  await iterator.next();
  const socket = created[0];
  if (!socket) throw Error();
  socket.open();
  socket.message({ op: "heartbeat", server_time: "now" });
  await expect(iterator.next()).rejects.toBeInstanceOf(StreamBackpressureError);
  expect(socket.closed).toBe(true);
});
it("emits distinct reconnect recovery for snapshot and live-only channels", async () => {
  vi.useFakeTimers();
  const { created, factory } = sockets();
  const stream = marketStream({
    channels: [
      { name: "orderbook", token_id: "1" },
      { name: "trades", condition_id: "0x1" },
    ],
    webSocketFactory: factory,
    reconnectBaseMs: 1,
    random: () => 0,
  });
  const iterator = stream[Symbol.asyncIterator]();
  await iterator.next();
  const socket = created[0];
  if (!socket) throw Error();
  socket.open();
  await iterator.next();
  socket.close(1006);
  expect((await iterator.next()).value).toMatchObject({
    type: "gap",
    recovery: "rest_then_snapshot",
  });
  expect((await iterator.next()).value).toMatchObject({ type: "gap", recovery: "reconcile_rest" });
  await vi.advanceTimersByTimeAsync(1);
  expect(created).toHaveLength(2);
  stream.close();
});
it("does not loop expired authentication or activate unknown recovery", async () => {
  const { created, factory } = sockets();
  const stream = accountStream({ token: "agt_test", webSocketFactory: factory });
  const iterator = stream[Symbol.asyncIterator]();
  await iterator.next();
  const socket = created[0];
  if (!socket) throw Error();
  socket.open();
  await iterator.next();
  socket.message(
    fixtures.find((f) => f.name === "future websocket code with reconnect action")?.value,
  );
  expect((await iterator.next()).value).toMatchObject({ type: "frame", frame: { action: "none" } });
  expect(socket.closed).toBe(false);
  socket.message(fixtures.find((f) => f.name === "websocket expiry frame")?.value);
  await expect(iterator.next()).rejects.toBeInstanceOf(StreamClosedError);
  expect(created).toHaveLength(1);
});
it("limits subscriptions and keeps credentials on account transport", () => {
  expect(() =>
    marketStream({
      channels: Array.from({ length: 65 }, (_, i) => ({ name: "orderbook", token_id: String(i) })),
    }),
  ).toThrow();
  expect(
    () =>
      new AgaraStream({
        endpoint: "market",
        channels: [{ name: "account_events", token: "secret" }],
      }),
  ).toThrow();
});
it("AbortSignal shuts down an active WebSocket", async () => {
  const { created, factory } = sockets();
  const controller = new AbortController();
  const stream = marketStream({
    channels: [],
    webSocketFactory: factory,
    signal: controller.signal,
  });
  const iterator = stream[Symbol.asyncIterator]();
  await iterator.next();
  controller.abort();
  expect(created[0]?.closed).toBe(true);
  expect((await iterator.next()).done).toBe(true);
});
async function* encodeChunks(chunks: string[]) {
  for (const chunk of chunks) yield new TextEncoder().encode(chunk);
}
it("parses SSE comments, CRLF chunk boundaries, multiline data, and many small events", async () => {
  const events = [];
  for await (const event of parseSse(
    encodeChunks([
      ": keepalive\r",
      '\ndata: {\r\ndata: "a":1}\r\n\r\n',
      ...Array.from({ length: 5 }, () => "data: {}\n\n"),
    ]),
    32,
  ))
    events.push(event);
  expect(events).toHaveLength(6);
  expect(JSON.parse(events[0]?.data ?? "")).toEqual({ a: 1 });
  const together = [];
  for await (const event of parseSse(encodeChunks(["data: {}\n\n".repeat(100)]), 16))
    together.push(event);
  expect(together).toHaveLength(100);
});
it.each(["multi", "single"])(
  "consumes actual %s price-feed wire and releases body on return",
  async (kind) => {
    let url = "",
      cancelled = false;
    const payload = {
      parsed: [
        { id: "BTC/USD", price: { price: "1000001", expo: -2, publish_time_ms: 1700000000123 } },
      ],
    };
    const fetcher: typeof fetch = async (input) => {
      url = String(input);
      return new Response(
        new ReadableStream({
          start(controller) {
            controller.enqueue(new TextEncoder().encode(`data: ${JSON.stringify(payload)}\n\n`));
          },
          cancel() {
            cancelled = true;
          },
        }),
        { headers: { "content-type": "text/event-stream" } },
      );
    };
    const iterator =
      kind === "multi"
        ? priceStream(["BTC/USD"], { fetch: fetcher, reconnect: false })
        : pythProStream("BTC/USD", { fetch: fetcher, reconnect: false });
    expect((await iterator.next()).value?.frame).toEqual(payload);
    await iterator.return(undefined);
    expect(cancelled).toBe(true);
    expect(new URL(url).pathname).toBe(
      kind === "multi" ? "/api/v1/prices/stream" : "/api/v1/prices/pyth-pro/stream",
    );
  },
);
it("does not retry before an oversized Retry-After", async () => {
  const fetcher = vi.fn<typeof fetch>(
    async () =>
      new Response(
        JSON.stringify({
          type: "urn:agara:problem:rate-limited",
          title: "Too many requests",
          code: "rate_limited",
          status: 429,
          recovery: { strategy: "retry_after", after_seconds: 120 },
        }),
        { status: 429, headers: { "retry-after": "120" } },
      ),
  );
  await expect(priceStream(["BTC/USD"], { fetch: fetcher }).next()).rejects.toMatchObject({
    status: 429,
  });
  expect(fetcher).toHaveBeenCalledTimes(1);
});
it("counts reconnects over the stream's lifetime, not since the last price", async () => {
  // Every connection delivers a price and then ends, so a count reset on data would never advance.
  const fetcher = vi.fn<typeof fetch>(
    async () =>
      new Response(`data: ${JSON.stringify({ parsed: [] })}\n\n`, {
        headers: { "content-type": "text/event-stream" },
      }),
  );
  const seen: number[] = [];
  const connectionsMade: number[] = [];
  const stream = priceStream(["BTC/USD"], {
    fetch: fetcher,
    reconnectDelayMs: 0,
    maxReconnects: 2,
    onReconnect: (attempt) => {
      seen.push(attempt);
      connectionsMade.push(fetcher.mock.calls.length);
    },
  });

  await expect(
    (async () => {
      for await (const _event of stream);
    })(),
  ).rejects.toBeInstanceOf(ProtocolError);
  expect(seen).toEqual([1, 2]);
  // Each callback lands before its retry opens the next connection, and the attempt that exceeds the
  // limit is never announced: a caller counting these sees every wait it is actually made to serve.
  expect(connectionsMade).toEqual([1, 2]);
  expect(fetcher).toHaveBeenCalledTimes(3);
});
it("never announces a reconnect when retries are disabled by the limit", async () => {
  const fetcher = vi.fn<typeof fetch>(
    async () =>
      new Response(`data: ${JSON.stringify({ parsed: [] })}\n\n`, {
        headers: { "content-type": "text/event-stream" },
      }),
  );
  const onReconnect = vi.fn();
  const stream = priceStream(["BTC/USD"], { fetch: fetcher, maxReconnects: 0, onReconnect });

  await expect(
    (async () => {
      for await (const _event of stream);
    })(),
  ).rejects.toBeInstanceOf(ProtocolError);
  expect(onReconnect).not.toHaveBeenCalled();
  expect(fetcher).toHaveBeenCalledTimes(1);
});
it("rejects an outstanding next on malformed data rather than ending cleanly", async () => {
  const { created, factory } = sockets();
  const stream = marketStream({ channels: [], webSocketFactory: factory });
  const iterator = stream[Symbol.asyncIterator]();
  await iterator.next();
  const socket = created[0];
  if (!socket) throw Error();
  socket.open();
  await iterator.next();
  const pending = iterator.next();
  socket.message({ op: "update", channel: "orderbook", token_id: "1", data: { kind: "snapshot" } });
  await expect(pending).rejects.toBeInstanceOf(ProtocolError);
});
it("rejects an outstanding next when reconnect attempts are exhausted", async () => {
  const { created, factory } = sockets();
  const stream = marketStream({ channels: [], webSocketFactory: factory, maxReconnects: 0 });
  const iterator = stream[Symbol.asyncIterator]();
  await iterator.next();
  const socket = created[0];
  if (!socket) throw Error();
  socket.open();
  await iterator.next();
  const pending = iterator.next();
  socket.close(1006);
  await expect(pending).rejects.toBeInstanceOf(StreamClosedError);
});
it("bounds peers that acknowledge then immediately disconnect", async () => {
  vi.useFakeTimers();
  const { created, factory } = sockets();
  const stream = marketStream({
    channels: [],
    webSocketFactory: factory,
    maxReconnects: 1,
    reconnectBaseMs: 1,
    random: () => 0,
  });
  const iterator = stream[Symbol.asyncIterator]();
  await iterator.next();
  const first = created[0];
  if (!first) throw Error();
  first.open();
  await iterator.next();
  first.message({ op: "subscribed", channel: "account_events" });
  await iterator.next();
  first.close(1006);
  await vi.advanceTimersByTimeAsync(1);
  const second = created[1];
  if (!second) throw Error();
  await iterator.next();
  second.open();
  await iterator.next();
  second.message({ op: "subscribed", channel: "account_events" });
  await iterator.next();
  const pending = iterator.next();
  second.close(1006);
  await expect(pending).rejects.toBeInstanceOf(StreamClosedError);
  expect(created).toHaveLength(2);
});
it("dispatches a CR-delimited final complete event", async () => {
  const events = [];
  for await (const event of parseSse(encodeChunks(["data: {}\r\r"]))) events.push(event);
  expect(events).toHaveLength(1);
});

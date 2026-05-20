"""Callback-mode demo: subscribe to public + private channels on one
client and dispatch each frame via typed handlers. Auto-reconnects
with backoff.

    export AGARA_BASE_URL="https://d3r180aqvl5ynd.cloudfront.net"
    export AGARA_TOKEN="agt_pat_..."
    pip install 'agara-sdk[streaming]'
    python examples/bot.py <token_id> <condition_id>

Ctrl-C to exit. The client transparently opens two underlying WS
connections — one for public channels, one for `account_events`.
"""

from __future__ import annotations

import asyncio
import os
import sys

from agara_sdk import streaming


async def run(token_id: str, condition_id: str, token: str, base_url: str) -> None:
    client = streaming.AgaraStreamClient(token=token, base_url=base_url)

    @client.on_connect
    async def _(ctx: streaming.ConnectContext) -> None:
        tag = "reconnected" if ctx.is_reconnect else "connected"
        print(f"[{tag}] endpoint={ctx.endpoint} attempt={ctx.attempt}")

    @client.on_orderbook_snapshot
    async def _(snap: streaming.OrderbookSnapshot) -> None:
        print(
            f"[snapshot] token={snap.token_id} seq={snap.sequence} "
            f"bids={len(snap.bids)} asks={len(snap.asks)}"
        )

    @client.on_orderbook_delta
    async def _(d: streaming.OrderbookDelta) -> None:
        print(
            f"[delta] token={d.token_id} seq={d.sequence} "
            f"bids={len(d.bids)} asks={len(d.asks)}"
        )

    @client.on_best_quote
    async def _(q: streaming.BestQuote) -> None:
        bid = f"{q.bid.price}@{q.bid.size}" if q.bid else "—"
        ask = f"{q.ask.price}@{q.ask.size}" if q.ask else "—"
        print(f"[best_quote] {q.token_id} bid={bid} ask={ask}")

    @client.on_trade
    async def _(t: streaming.Trade) -> None:
        print(
            f"[trade] market={t.condition_id} outcome={t.outcome} "
            f"{t.side} {t.size}@{t.price} mode={t.settlement_mode}"
        )

    @client.on_fill
    async def _(fill: streaming.Fill) -> None:
        print(
            f"[fill {fill.role}] id={fill.fill_id} order={fill.order_id} "
            f"market={fill.market_id} outcome={fill.outcome} "
            f"side={fill.side} px={fill.price}/{fill.price_scale} "
            f"sz={fill.size}/{fill.size_scale} mode={fill.settlement_mode} "
            f"fee={fill.fee_micro_usdc}"
        )

    @client.on_order_accepted
    async def _(o: streaming.OrderAccepted) -> None:
        print(
            f"[accepted] order={o.order_id} market={o.market_id} "
            f"outcome={o.outcome} side={o.side} px={o.price}/{o.price_scale} "
            f"remaining={o.remaining_size}/{o.size_scale} tif={o.tif}"
        )

    @client.on_order_cancelled
    async def _(o: streaming.OrderCancelled) -> None:
        print(
            f"[cancelled] order={o.order_id} market={o.market_id} "
            f"outcome={o.outcome} reason={o.reason}"
        )

    @client.on_sequence_reset
    async def _(r: streaming.SequenceReset) -> None:
        print(
            f"[!! sequence_reset] channel={r.channel} "
            f"token_id={r.token_id} condition_id={r.condition_id}"
        )

    @client.on_error
    async def _(err: streaming.StreamError) -> None:
        print(f"[error] {err.code}: {err.message}")

    await client.subscribe([
        streaming.orderbook(token_id),
        streaming.best_quote(token_id),
        streaming.trades(condition_id),
        streaming.account_events(),
    ])
    await client.run()


def main() -> None:
    if len(sys.argv) != 3:
        sys.exit("usage: bot.py <token_id> <condition_id>")
    token = os.environ.get("AGARA_TOKEN")
    if not token:
        sys.exit("set AGARA_TOKEN to a Privy JWT or PAT (agt_pat_...)")
    base_url = os.environ.get("AGARA_BASE_URL", streaming.DEFAULT_BASE_URL)
    try:
        asyncio.run(run(sys.argv[1], sys.argv[2], token, base_url))
    except KeyboardInterrupt:
        pass


if __name__ == "__main__":
    main()

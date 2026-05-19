"""Iterator-mode demo: subscribe to one or more channels on the public
market stream and print every frame as it arrives. Ctrl-C to exit.

    export AGARA_BASE_URL="https://d3r180aqvl5ynd.cloudfront.net"
    pip install 'agara-sdk[streaming]'
    python examples/subscribe.py orderbook 21742…36455
    python examples/subscribe.py trades 0x2174…
    python examples/subscribe.py best_quote 21742…36455 market_status 0x2174…

Endpoint is public — no token. For a callback-style bot that also
streams your private fills, see `examples/bot.py`.
"""

from __future__ import annotations

import asyncio
import os
import sys
from dataclasses import asdict

from agara_sdk import streaming


_FACTORIES = {
    "orderbook": streaming.orderbook,
    "best_quote": streaming.best_quote,
    "market_status": streaming.market_status,
    "trades": streaming.trades,
}


def parse_channels(args: list[str]) -> list[streaming.Channel]:
    if not args or len(args) % 2 != 0:
        sys.exit(
            "usage: subscribe.py <channel> <subject_id> [<channel> <subject_id>]..."
        )
    channels = []
    for name, subject in zip(args[0::2], args[1::2]):
        factory = _FACTORIES.get(name)
        if factory is None:
            sys.exit(f"unknown channel {name!r}; pick one of {list(_FACTORIES)}")
        channels.append(factory(subject))
    return channels


async def run(channels: list[streaming.Channel], base_url: str) -> None:
    client = streaming.AgaraStreamClient(base_url=base_url)
    await client.subscribe(channels)
    async with client as ws:
        async for frame in ws:
            print(type(frame).__name__, asdict(frame))


def main() -> None:
    channels = parse_channels(sys.argv[1:])
    base_url = os.environ.get("AGARA_BASE_URL", streaming.DEFAULT_BASE_URL)
    try:
        asyncio.run(run(channels, base_url))
    except KeyboardInterrupt:
        pass


if __name__ == "__main__":
    main()

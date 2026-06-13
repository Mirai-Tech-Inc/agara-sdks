"""Place an order, wait for it to settle, print fills.

    export AGARA_BASE_URL="https://d3r180aqvl5ynd.cloudfront.net"
    export AGARA_TOKEN="agt_..."
    export AGARA_TOKEN_ID="21742633143463906290569050155826241533067272736897614950488156847949938836455"
    python examples/trading.py
"""

import os

from agara_sdk import AgaraClient, RejectedError, TERMINAL_STATUSES


def main() -> None:
    base_url = os.environ.get("AGARA_BASE_URL", "https://d3r180aqvl5ynd.cloudfront.net")
    token = os.environ["AGARA_TOKEN"]
    token_id = os.environ["AGARA_TOKEN_ID"]

    with AgaraClient(token=token, base_url=base_url) as client:
        # 1. Snapshot the book.
        book = client.get_orderbook(token_id)
        print(f"best bid {book.best_bid} / ask {book.best_ask} / spread {book.spread}")

        # 2. Place a BUY at the current best bid for 1 share. If the
        #    book is empty, fall back to $0.50.
        price = book.best_bid if book.best_bid is not None else 0.50
        try:
            resp = client.place_order(
                token_id=token_id,
                side="BUY",
                price=price,
                shares=1.0,
            )
        except RejectedError as err:
            print(f"order rejected: {err.message}")
            return

        order_id = resp["order_id"]
        print(f"placed {order_id}, status {resp['status']}")

        # 3. Wait up to 30 seconds for it to fill or settle.
        order = client.wait_for_terminal(order_id, timeout=30.0)
        if order["status"] not in TERMINAL_STATUSES:
            print(f"not terminal after 30s ({order['status']}); cancelling")
            client.cancel_order(order_id)
            order = client.wait_for_terminal(order_id, timeout=10.0)
        print(f"final status: {order['status']}")

        # 4. Print the last few fills (first page is newest-first).
        trades = client.list_trades().get("trades", [])[:5]
        for t in trades:
            shares = int(t["shares_micro"]) / 1_000_000
            tprice = int(t["price_micro"]) / 1_000_000
            fee = int(t["fee_micro"]) / 1_000_000
            print(
                f"  {t['executed_at']}  {t['side']}  {shares:.2f}sh "
                f"@ ${tprice:.4f}  fee=${fee:.6f}"
            )


if __name__ == "__main__":
    main()

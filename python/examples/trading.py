"""Place a post-only order and observe order completion separately from trade settlement.

Set AGARA_TOKEN, AGARA_TOKEN_ID and optionally AGARA_BASE_URL before running.
This example submits and cancels real orders on the configured deployment.
"""

import os

from agara_sdk import DEFAULT_BASE_URL, AgaraClient, micro_to_decimal


def main() -> None:
    with AgaraClient(
        os.environ["AGARA_TOKEN"], os.getenv("AGARA_BASE_URL", DEFAULT_BASE_URL)
    ) as client:
        accepted = client.place_order(
            token_id=os.environ["AGARA_TOKEN_ID"],
            side="BUY",
            price="0.50",
            shares="1",
            post_only=True,
        )
        try:
            order = client.wait_for_terminal(accepted["order_id"], timeout=30)
        except TimeoutError:
            client.cancel_order(accepted["order_id"])
            order = client.wait_for_terminal(accepted["order_id"], timeout=30)
        print(order["status"], order["is_terminal"])
        for trade in client.get_order_trades(accepted["order_id"])["trades"]:
            print(
                trade["status"], micro_to_decimal(trade["shares_micro"]), trade["transaction_hash"]
            )


if __name__ == "__main__":
    main()

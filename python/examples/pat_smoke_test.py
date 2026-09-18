"""Read-only personal access token check; needs orders:read and portfolio:read."""

import os

from agara_sdk import DEFAULT_BASE_URL, AgaraClient, micro_to_decimal


def main() -> None:
    with AgaraClient(
        os.environ["AGARA_PAT"], os.getenv("AGARA_BASE_URL", DEFAULT_BASE_URL)
    ) as client:
        for summary in client.get_portfolio_summary()["summaries"]:
            print(summary["exchange"], micro_to_decimal(summary["cash_balance_micro"]))
        positions = client.list_positions()
        print("positions", len(positions["positions"]))
        print("unavailable exchanges", positions["unavailable_exchanges"])
        page = client.list_orders(limit=10)
        for order in page["orders"]:
            print(order["internal_id"], order["status"], order["is_terminal"])


if __name__ == "__main__":
    main()

"""Smoke-test a Personal Access Token against the local router.

Exercises every PAT-allowed surface and confirms the wallet-management
gate (`require_jwt`) rejects PATs.

    # Router is bound to :8100 in main.rs; the FE at :3000 proxies through.
    export AGARA_BASE_URL="http://localhost:8100"
    export AGARA_PAT="agt_..."

    # Any token_id that resolves to a market on the exchange your PAT's
    # wallet is bound to. Pull one from the FE event page or
    # /trade/v1/orderbook docs. Example below is Polymarket.
    export AGARA_TOKEN_ID="21742633143463906290569050155826241533067272736897614950488156847949938836455"

    python examples/pat_smoke_test.py

Expects scopes: portfolio:read, orders:place, orders:cancel.
The order is placed at $0.01 with 1 share so it almost certainly won't
fill — the cancel step then takes it back out.
"""

from __future__ import annotations

import os
import sys

import requests

from agara_sdk import AgaraClient, AuthError, RejectedError, micro_to_float


def main() -> None:
    base_url = os.environ.get("AGARA_BASE_URL", "http://localhost:8100").rstrip("/")
    pat = os.environ["AGARA_PAT"]
    token_id = os.environ["AGARA_TOKEN_ID"]
    if not pat.startswith("agt_"):
        sys.exit("AGARA_PAT must start with 'agt_'")

    headers = {"Authorization": f"Bearer {pat}", "Accept": "application/json"}

    print(f"→ base   = {base_url}")
    print(f"→ pat    = agt_{pat[4:12]}…")
    print(f"→ market = {token_id}\n")

    with AgaraClient(token=pat, base_url=base_url) as client:
        book = client.get_orderbook(token_id)
        print(f"[1] orderbook ok — best_bid={book.best_bid} best_ask={book.best_ask}")

        summaries = client.get_portfolio_summary()
        print(f"[2] portfolio:read ok — {len(summaries)} exchange(s)")
        for s in summaries:
            cash = micro_to_float(s["cash_balance_micro"]) or 0.0
            print(f"      {s['exchange']:<10} cash=${cash:,.2f}")

        try:
            placed = client.place_order(
                token_id=token_id, side="BUY", price=0.01, shares=1.0
            )
        except RejectedError as err:
            sys.exit(f"[3] place rejected: {err.message}")
        order_id = placed["order_id"]
        print(f"[3] orders:place ok — placed {order_id}")

        listing = client.list_orders(limit=10)
        ours = next((o for o in listing["orders"] if o["id"] == order_id), None)
        if ours is None:
            sys.exit(f"[4] orders:read returned {len(listing['orders'])} orders but our id wasn't among them")
        print(f"[4] orders:read ok — status={ours['status']}")

        client.cancel_order(order_id)
        terminal = client.wait_for_terminal(order_id, timeout=10.0)
        print(f"[5] orders:cancel ok — final status={terminal['status']}")

    # PAT-management gate: hitting /auth/tokens with a PAT must 401.
    # Raw requests rather than the SDK because we're asserting on the
    # status code, not on a parsed body.
    resp = requests.get(f"{base_url}/trade/v1/auth/tokens", headers=headers, timeout=10)
    if resp.status_code != 401:
        sys.exit(
            f"[6] FAIL: GET /auth/tokens returned {resp.status_code}, expected 401. "
            f"`require_jwt` is not gating the endpoint correctly."
        )
    print(f"[6] require_jwt ok — /auth/tokens correctly rejected PAT with 401")

    print("\n✓ all checks passed")


if __name__ == "__main__":
    try:
        main()
    except KeyError as err:
        sys.exit(f"missing required env var: {err}")
    except AuthError as err:
        sys.exit(f"auth error: {err.message} (status {err.status_code})")

"""Compose and sign locally; does not submit the result."""

import os

from agara_sdk.batch_signing import BatchContracts, BatchDomain, RoutedOperation, sign_batch


def main() -> None:
    signed = sign_batch(
        private_key=os.environ["AGARA_PRIVATE_KEY"],
        domain=BatchDomain(os.environ["AGARA_ACCOUNT"], int(os.environ["AGARA_CHAIN_ID"])),
        contracts=BatchContracts(
            os.environ["AGARA_CTF"],
            os.environ["AGARA_NEG_RISK_ADAPTER"],
            os.environ["AGARA_COLLATERAL"],
        ),
        operations=[
            RoutedOperation(
                {
                    "kind": "MERGE",
                    "market_id": os.environ["AGARA_MARKET_ID"],
                    "condition_id": os.environ["AGARA_CONDITION_ID"],
                    "shares_micro": 1_000_000,
                }
            )
        ],
        seq=int(os.environ["AGARA_ACCOUNT_SEQ"]),
        deadline_unix_seconds=int(os.environ["AGARA_BATCH_DEADLINE"]),
    )
    print(signed.batch_hash)


if __name__ == "__main__":
    main()

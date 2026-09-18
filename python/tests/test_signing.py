"""EIP-712 order-signing tests.

The order-hash golden is shared cross-repo with `crates/chain-client/src/eip712.rs`
(its `order_hash_matches_independently_computed_golden` test) and was independently
recomputed with a second EIP-712 implementation. A match here proves the SDK produces the exact digest the
on-chain `CTFExchange.hashOrder` / `AgaraAccount.isValidSignature` verify, so a
maker bot's pre-signed order validates on-chain.
"""

from __future__ import annotations

from eth_account import Account
from eth_account.messages import encode_typed_data

from agara_sdk.signing import (
    _ORDER_TYPES,
    DOMAIN_NAME,
    DOMAIN_VERSION,
    EngineDomain,
    sign_limit_order,
)

# Fixed test vector shared with chain-client's eip712 golden.
_ACCOUNT = "0x279640887C3806d4FBd424bb0B58F0430CE661C1"
_EXCHANGE = "0x1b42FF8DdB251074637d3A9872D72f51e3AbB23d"
_CHAIN_ID = 84532
_ORDER_HASH_GOLDEN = "0xf5cbbd057896816a0be9a705a90bf1f094b01af05f977658791b3e65c862c961"
_ZERO_BYTES32 = "0x" + "00" * 32


def test_domain_and_order_fields_match_the_fork():
    # The fork's domain + nine-field ORDER_TYPEHASH (no signer, no signatureType).
    assert DOMAIN_NAME == "Agara CTF Exchange"
    assert DOMAIN_VERSION == "1"
    order_fields = [f["name"] for f in _ORDER_TYPES["Order"]]
    assert order_fields == [
        "salt",
        "maker",
        "tokenId",
        "makerAmount",
        "takerAmount",
        "side",
        "timestamp",
        "metadata",
        "builder",
    ]
    assert "signatureType" not in order_fields
    assert "signer" not in order_fields


def test_order_hash_matches_canonical_golden():
    # Arrange — the fixed order from chain-client's eip712 golden.
    typed_data = {
        "types": _ORDER_TYPES,
        "primaryType": "Order",
        "domain": {
            "name": DOMAIN_NAME,
            "version": DOMAIN_VERSION,
            "chainId": _CHAIN_ID,
            "verifyingContract": _EXCHANGE,
        },
        "message": {
            "salt": 1,
            "maker": _ACCOUNT,
            "tokenId": 2,
            "makerAmount": 100,
            "takerAmount": 100,
            "side": 0,
            "timestamp": 0,
            "metadata": _ZERO_BYTES32,
            "builder": _ZERO_BYTES32,
        },
    }

    # Act — the EIP-712 signing hash is the order hash.
    encoded = encode_typed_data(full_message=typed_data)
    signed = Account.sign_message(encoded, private_key="0x" + "11" * 32)

    # Assert — equals the digest shared with chain-client's golden.
    assert "0x" + signed.message_hash.hex() == _ORDER_HASH_GOLDEN


def test_sign_limit_order_output_shape():
    signed = sign_limit_order(
        private_key="0x" + "11" * 32,
        domain=EngineDomain(chain_id=_CHAIN_ID, exchange_contract=_EXCHANGE),
        deposit_wallet_address=_ACCOUNT,
        token_id=2,
        side="BUY",
        price_micro=500_000,
        shares_micro=2_000_000,
        salt=1,
    )

    assert signed.order_hash.startswith("0x") and len(signed.order_hash) == 66
    assert signed.signature.startswith("0x") and len(signed.signature) == 132
    assert signed.maker == _ACCOUNT


def test_signed_order_economics_are_bound():
    import pytest

    signed = sign_limit_order(
        private_key="0x" + "11" * 32,
        domain=EngineDomain(_CHAIN_ID, _EXCHANGE),
        deposit_wallet_address=_ACCOUNT,
        token_id=2,
        side="BUY",
        price_micro=500000,
        shares_micro=2000000,
        salt=1,
    )
    original = dict(
        token_id_string="2", side_string="BUY", price_micro=500000, shares_micro=2000000
    )
    assert signed.to_request_body(**original)["maker_amount"] == "1000000"
    for change in [
        {"token_id_string": "3"},
        {"side_string": "SELL"},
        {"price_micro": 500001},
        {"shares_micro": 2000001},
    ]:
        with pytest.raises(ValueError):
            signed.to_request_body(**{**original, **change})
    for salt in [0, -1, 2**256, True]:
        with pytest.raises(ValueError):
            sign_limit_order(
                private_key="0x" + "11" * 32,
                domain=EngineDomain(_CHAIN_ID, _EXCHANGE),
                deposit_wallet_address=_ACCOUNT,
                token_id=2,
                side="BUY",
                price_micro=500000,
                shares_micro=2000000,
                salt=salt,
            )


def test_canonical_batch_composition_and_both_published_digests():
    import re
    from pathlib import Path

    from agara_sdk.batch_signing import (
        BatchContracts,
        BatchDomain,
        RoutedOperation,
        batch_digest,
        compose_batch,
        sign_batch,
    )

    golden = (Path(__file__).parent / "fixtures/account-batch-golden-vectors.rs").read_text()
    constants = dict(re.findall(r'const (\w+): &str =\s*"([^"]+)";', golden))
    contracts = BatchContracts("0x" + "22" * 20, "0x" + "33" * 20, "0x" + "44" * 20)
    domain = BatchDomain("0x" + "11" * 20, 8453)
    operations = [
        RoutedOperation(
            {
                "kind": "SPLIT",
                "market_id": "00000000-0000-0000-0000-000000000001",
                "condition_id": constants["CONDITION_1"],
                "shares_micro": 5000000,
            }
        ),
        RoutedOperation(
            {
                "kind": "MERGE",
                "market_id": "00000000-0000-0000-0000-000000000002",
                "condition_id": constants["CONDITION_2"],
                "shares_micro": 2000000,
            }
        ),
        RoutedOperation(
            {"kind": "WITHDRAW", "destination": constants["DESTINATION"], "amount_micro": 1000000}
        ),
    ]
    calls = compose_batch(operations, contracts)
    assert [c.data for c in calls] == [
        constants["SPLIT_CALLDATA"],
        constants["MERGE_CALLDATA"],
        constants["TRANSFER_CALLDATA"],
    ]
    assert batch_digest(domain, 4, 1784022000, calls[:1]) == constants["SINGLE_CALL_BATCH_HASH"]
    assert batch_digest(domain, 4, 1784022000, calls) == constants["MULTI_CALL_BATCH_HASH"]
    signed = sign_batch(
        private_key="0x" + "11" * 32,
        domain=domain,
        contracts=contracts,
        operations=operations,
        seq=4,
        deadline_unix_seconds=1784022000,
    )
    assert signed.batch_hash == constants["MULTI_CALL_BATCH_HASH"]
    wire = signed.to_request_body()
    wire["ops"][0]["shares_micro"] = 1
    assert signed.to_request_body()["ops"][0]["shares_micro"] == 5000000
    assert "seq" not in signed.to_supersede_body()


def test_batch_signer_rejects_unsupported_presigned_operations():
    import pytest

    from agara_sdk.batch_signing import BatchContracts, RoutedOperation, compose_batch

    contracts = BatchContracts("0x" + "22" * 20, "0x" + "33" * 20, "0x" + "44" * 20)
    for kind in ["REDEEM", "ACROSS_SWAP_APPROVE", "WITHDRAW_ACROSS_SWAP_CCTP"]:
        with pytest.raises(ValueError):
            compose_batch([RoutedOperation({"kind": kind})], contracts)

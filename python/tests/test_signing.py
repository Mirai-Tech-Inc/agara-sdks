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
    DOMAIN_NAME,
    DOMAIN_VERSION,
    EngineDomain,
    _ORDER_TYPES,
    sign_limit_order,
)

# Fixed test vector shared with chain-client's eip712 golden.
_ACCOUNT = "0x279640887C3806d4FBd424bb0B58F0430CE661C1"
_EXCHANGE = "0x1b42FF8DdB251074637d3A9872D72f51e3AbB23d"
_CHAIN_ID = 84532
_ORDER_HASH_GOLDEN = (
    "0xf5cbbd057896816a0be9a705a90bf1f094b01af05f977658791b3e65c862c961"
)
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

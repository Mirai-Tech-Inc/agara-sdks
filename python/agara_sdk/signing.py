"""EIP-712 order signing for the agara CLOB exchange.

Mirrors `crates/chain-client/src/eip712.rs`. The on-chain
`CTFExchange.hashOrder` view + alloy's `eip712_signing_hash` produce
the same 32-byte digest as `_hash_typed_data` below; that digest is
what the deposit wallet's EIP-1271 path verifies on-chain via
`ecrecover(hash, sig) == owner`.

LIMIT-only for now. MARKET orders involve orderbook-walking + fee
carve-outs (see `crates/agara-wallet/src/shape.rs`); they continue to
flow through the Privy-signed path until the SDK mirrors that logic.
"""

from __future__ import annotations

import uuid
from dataclasses import dataclass
from typing import Literal

from eth_account import Account
from eth_account.messages import encode_typed_data


DOMAIN_NAME = "Polymarket CTF Exchange"
DOMAIN_VERSION = "2"

SIDE_BUY = 0
SIDE_SELL = 1

# `SignatureKind::Poly1271` in `crates/chain-client/src/types.rs`. The
# deposit wallet is the maker/signer on the order; the user's EOA
# signature is verified via the wallet's `isValidSignature` (EIP-1271).
SIGNATURE_KIND_POLY1271 = 3

MICRO = 1_000_000

_ZERO_BYTES32 = "0x" + "00" * 32

_ORDER_TYPES = {
    "EIP712Domain": [
        {"name": "name", "type": "string"},
        {"name": "version", "type": "string"},
        {"name": "chainId", "type": "uint256"},
        {"name": "verifyingContract", "type": "address"},
    ],
    "Order": [
        {"name": "salt", "type": "uint256"},
        {"name": "maker", "type": "address"},
        {"name": "signer", "type": "address"},
        {"name": "tokenId", "type": "uint256"},
        {"name": "makerAmount", "type": "uint256"},
        {"name": "takerAmount", "type": "uint256"},
        {"name": "side", "type": "uint8"},
        {"name": "signatureType", "type": "uint8"},
        {"name": "timestamp", "type": "uint256"},
        {"name": "metadata", "type": "bytes32"},
        {"name": "builder", "type": "bytes32"},
    ],
}


@dataclass(frozen=True)
class EngineDomain:
    """EIP-712 domain binding the signature to a specific engine
    deployment. `chain_id` is the chain the exchange contract lives
    on; `exchange_contract` is the contract address that will verify
    the signature via EIP-1271."""

    chain_id: int
    exchange_contract: str  # 0x-prefixed hex


@dataclass(frozen=True)
class SignedOrder:
    """Output of `sign_limit_order`. Serialise via `to_request_body`
    when calling `POST /trade/v1/orders/signed`."""

    order_hash: str             # 0x-prefixed keccak256 of the typed-data digest
    signature: str              # 0x-prefixed 65-byte (r||s||v) hex
    salt: int                   # u256 — drawn once at sign time
    maker: str                  # deposit wallet address (== signer for POLY_1271)
    signer: str
    token_id: int               # u256 — outcome token id on the CTF
    maker_amount: int           # μUSDC for BUY, μshares for SELL
    taker_amount: int           # μshares for BUY, μUSDC for SELL
    side: int                   # 0 = BUY, 1 = SELL
    signature_type: int = SIGNATURE_KIND_POLY1271
    timestamp: int = 0
    metadata: str = _ZERO_BYTES32
    builder: str = _ZERO_BYTES32

    def to_request_body(
        self,
        *,
        token_id_string: str,
        side_string: Literal["BUY", "SELL"],
        price_micro: int,
        shares_micro: int,
        time_in_force: str = "GTC",
        post_only: bool = False,
        expiration_unix_seconds: int | None = None,
    ) -> dict:
        """Build the JSON body for `POST /trade/v1/orders/signed`.

        `token_id_string` is the human-facing token id the router
        looks up (matches `agara_market_outcomes.token_id`).
        `token_id` on this dataclass is the same value decoded as
        u256 — the chain envelope uses the integer form. Both are
        sent so the router can validate consistency without
        re-parsing."""
        body = {
            "token_id": token_id_string,
            "side": side_string,
            "type": "LIMIT",
            "time_in_force": time_in_force,
            "price_micro": str(price_micro),
            "shares_micro": str(shares_micro),
            "post_only": post_only,
            "order_hash": self.order_hash,
            "signature": self.signature,
            "salt": str(self.salt),
            "maker": self.maker,
            "signer": self.signer,
            "chain_token_id": str(self.token_id),
            "maker_amount": str(self.maker_amount),
            "taker_amount": str(self.taker_amount),
            "side_u8": self.side,
            "signature_type": self.signature_type,
            "timestamp": str(self.timestamp),
            "metadata": self.metadata,
            "builder": self.builder,
        }
        if expiration_unix_seconds is not None:
            body["expiration_unix_seconds"] = expiration_unix_seconds
        return body


def sign_limit_order(
    *,
    private_key: str,
    domain: EngineDomain,
    deposit_wallet_address: str,
    token_id: int,
    side: Literal["BUY", "SELL"],
    price_micro: int,
    shares_micro: int,
    salt: int | None = None,
) -> SignedOrder:
    """Sign a LIMIT order. `deposit_wallet_address` is both the maker
    and the signer on the chain envelope (POLY_1271 path); the user's
    EOA signature recovers via `ecrecover(hash, sig) == owner` on the
    deposit wallet's `isValidSignature`.

    Args:
        private_key: 0x-prefixed hex of the user's EOA private key
            (exported from Privy's `exportWallet` modal).
        domain: chain id + exchange contract address.
        deposit_wallet_address: 0x-prefixed hex.
        token_id: u256 outcome token id.
        side: "BUY" or "SELL".
        price_micro: μUSDC per share, in (0, 1_000_000).
        shares_micro: μshares.
        salt: u256 uniqueness nonce; defaults to a random u128 so the
            contract's hash-dedup never collides across this client.
    """
    if side not in ("BUY", "SELL"):
        raise ValueError("side must be 'BUY' or 'SELL'")
    if price_micro <= 0 or price_micro >= MICRO:
        raise ValueError("price_micro must be in (0, 1_000_000)")
    if shares_micro <= 0:
        raise ValueError("shares_micro must be > 0")

    collateral_micro = (shares_micro * price_micro) // MICRO
    if collateral_micro <= 0:
        raise ValueError("collateral rounds to zero — order too small")

    side_u8 = SIDE_BUY if side == "BUY" else SIDE_SELL
    # BUY: maker offers USDC, taker is shares; SELL: maker offers shares.
    if side == "BUY":
        maker_amount, taker_amount = collateral_micro, shares_micro
    else:
        maker_amount, taker_amount = shares_micro, collateral_micro

    salt_value = salt if salt is not None else uuid.uuid4().int & ((1 << 128) - 1)

    message = {
        "salt": salt_value,
        "maker": deposit_wallet_address,
        "signer": deposit_wallet_address,
        "tokenId": token_id,
        "makerAmount": maker_amount,
        "takerAmount": taker_amount,
        "side": side_u8,
        "signatureType": SIGNATURE_KIND_POLY1271,
        "timestamp": 0,
        "metadata": _ZERO_BYTES32,
        "builder": _ZERO_BYTES32,
    }
    typed_data = {
        "types": _ORDER_TYPES,
        "primaryType": "Order",
        "domain": {
            "name": DOMAIN_NAME,
            "version": DOMAIN_VERSION,
            "chainId": domain.chain_id,
            "verifyingContract": domain.exchange_contract,
        },
        "message": message,
    }
    encoded = encode_typed_data(full_message=typed_data)
    signed = Account.sign_message(encoded, private_key=private_key)

    return SignedOrder(
        order_hash="0x" + signed.message_hash.hex(),
        signature="0x" + signed.signature.hex(),
        salt=salt_value,
        maker=deposit_wallet_address,
        signer=deposit_wallet_address,
        token_id=token_id,
        maker_amount=maker_amount,
        taker_amount=taker_amount,
        side=side_u8,
    )

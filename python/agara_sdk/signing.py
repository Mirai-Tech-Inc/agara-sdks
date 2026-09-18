"""EIP-712 order signing for the agara CLOB exchange.

Mirrors `crates/chain-client/src/eip712.rs` (domain `("Agara CTF Exchange","1")`,
nine-field Order: no `signer`, no `signatureType`). The on-chain `CTFExchange.hashOrder`
view + alloy's `eip712_signing_hash` produce the same 32-byte digest as the
typed data below; that digest is what the maker account's `AgaraAccount`
`isValidSignature` verifies on-chain via flat `ecrecover(hash, sig) == holder`.

LIMIT only. MARKET orders need the server's live orderbook walk, so
send them through the regular place-order endpoint.
"""

from __future__ import annotations

import uuid
from dataclasses import dataclass
from typing import Literal, cast

from eth_account import Account
from eth_account.messages import encode_typed_data

from . import models
from ._requests import _validate_order
from .amounts import positive_micro

DOMAIN_NAME = "Agara CTF Exchange"
DOMAIN_VERSION = "1"

SIDE_BUY = 0
SIDE_SELL = 1

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
        {"name": "tokenId", "type": "uint256"},
        {"name": "makerAmount", "type": "uint256"},
        {"name": "takerAmount", "type": "uint256"},
        {"name": "side", "type": "uint8"},
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

    order_hash: str  # 0x-prefixed keccak256 of the typed-data digest
    signature: str  # 0x-prefixed 65-byte (r||s||v) hex
    salt: int  # u256 — drawn once at sign time
    maker: str  # deposit-wallet address; the AgaraAccount holds the position
    token_id: int  # u256 — outcome token id on the CTF
    maker_amount: int  # μUSDC for BUY, μshares for SELL
    taker_amount: int  # μshares for BUY, μUSDC for SELL
    side: int  # 0 = BUY, 1 = SELL
    signed_price_micro: int
    signed_shares_micro: int
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
    ) -> models.SignedOrderRequest:
        """Build the JSON body for `POST /trade/v1/orders/signed`.

        `token_id_string` is the human-facing token id the router
        looks up (matches `agara_market_outcomes.token_id`).
        `token_id` on this dataclass is the same value decoded as
        u256 — the chain envelope uses the integer form. Both are
        sent so the router can validate consistency without
        re-parsing."""
        if (
            int(token_id_string, 16) if token_id_string.startswith("0x") else int(token_id_string)
        ) != self.token_id:
            raise ValueError("token_id differs from the signed token")
        if side_string != ("BUY" if self.side == SIDE_BUY else "SELL"):
            raise ValueError("side differs from signed side")
        if price_micro != self.signed_price_micro or shares_micro != self.signed_shares_micro:
            raise ValueError("price or shares differ from signed economics")
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
            "chain_token_id": str(self.token_id),
            "maker_amount": str(self.maker_amount),
            "taker_amount": str(self.taker_amount),
            "side_u8": self.side,
            "timestamp": str(self.timestamp),
            "metadata": self.metadata,
            "builder": self.builder,
        }
        if expiration_unix_seconds is not None:
            body["expiration_unix_seconds"] = expiration_unix_seconds
        _validate_order(body)
        return cast(models.SignedOrderRequest, body)


@dataclass(frozen=True)
class SignedOrderEntry:
    """One order in a `place_signed_orders` batch: a `SignedOrder` plus the
    same per-order arguments `place_signed_order` takes. `to_request_body`
    reuses `SignedOrder.to_request_body`, so a batch element serialises
    identically to a single submission."""

    signed_order: SignedOrder
    token_id: str
    side: Literal["BUY", "SELL"]
    price_micro: int
    shares_micro: int
    time_in_force: str = "GTC"
    post_only: bool = False
    expiration_unix_seconds: int | None = None

    def to_request_body(self) -> models.SignedOrderRequest:
        return self.signed_order.to_request_body(
            token_id_string=self.token_id,
            side_string=self.side,
            price_micro=self.price_micro,
            shares_micro=self.shares_micro,
            time_in_force=self.time_in_force,
            post_only=self.post_only,
            expiration_unix_seconds=self.expiration_unix_seconds,
        )


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
    """Sign a LIMIT order. `deposit_wallet_address` is the maker on the
    chain envelope (it carries the AgaraAccount address); the holder's
    EOA signs the order hash flat, and `AgaraAccount.isValidSignature`
    recovers it via `ecrecover(hash, sig) == holder`.

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
    positive_micro(price_micro)
    positive_micro(shares_micro)
    _uint(token_id, 256, "token_id")
    _uint(domain.chain_id, 64, "chain_id", nonzero=True)
    _address(domain.exchange_contract)
    _address(deposit_wallet_address)
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

    _uint(salt_value, 256, "salt", nonzero=True)

    message = {
        "salt": salt_value,
        "maker": deposit_wallet_address,
        "tokenId": token_id,
        "makerAmount": maker_amount,
        "takerAmount": taker_amount,
        "side": side_u8,
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
        token_id=token_id,
        maker_amount=maker_amount,
        taker_amount=taker_amount,
        side=side_u8,
        signed_price_micro=price_micro,
        signed_shares_micro=shares_micro,
    )


def _uint(value: int, bits: int, name: str, *, nonzero: bool = False) -> int:
    if isinstance(value, bool) or not isinstance(value, int) or not int(nonzero) <= value < 2**bits:
        raise ValueError(f"{name} must fit uint{bits}" + (" and be nonzero" if nonzero else ""))
    return value


def _address(value: str) -> str:
    from eth_utils import is_address

    if not is_address(value) or int(value, 16) == 0:
        raise ValueError("expected a nonzero EVM address")
    return value

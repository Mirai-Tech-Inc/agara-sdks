"""Canonical typed account-batch composition and EIP-712 signing."""

from __future__ import annotations

import json
from dataclasses import dataclass, field
from typing import Any, Literal, cast
from uuid import UUID

from eth_abi import encode
from eth_account import Account
from eth_account.messages import encode_typed_data
from eth_utils import keccak

from . import models
from ._requests import _validate_batch
from .amounts import positive_micro
from .signing import _address, _uint

ZERO_BYTES32 = bytes(32)


@dataclass(frozen=True)
class BatchDomain:
    account: str
    chain_id: int
    implementation_version: int = 1


@dataclass(frozen=True)
class BatchContracts:
    ctf: str
    neg_risk_adapter: str
    collateral: str


@dataclass(frozen=True)
class RoutedOperation:
    operation: models.BatchOp
    route: Literal["Ctf", "NegRisk"] = "Ctf"


@dataclass(frozen=True)
class BatchCall:
    target: str
    data: str
    value: str = "0"


@dataclass(frozen=True)
class SignedBatch:
    batch_hash: str
    signature: str
    seq: int
    deadline_unix_seconds: int
    ops_json: str = field(repr=False)
    calls: tuple[BatchCall, ...]
    heals_batch_hash: str | None = None

    def to_request_body(self) -> models.BatchSubmission:
        return {
            "ops": json.loads(self.ops_json),
            "seq": self.seq,
            "deadline_unix_seconds": self.deadline_unix_seconds,
            "signature": self.signature,
            "heals_batch_hash": self.heals_batch_hash,
        }

    def to_supersede_body(self) -> models.BatchSupersedeSubmission:
        return {
            "ops": json.loads(self.ops_json),
            "deadline_unix_seconds": self.deadline_unix_seconds,
            "signature": self.signature,
        }


def _calldata(function: str, types: list[str], values: list[Any]) -> str:
    return (
        "0x"
        + (keccak(text=function + "(" + ",".join(types) + ")")[:4] + encode(types, values)).hex()
    )


def _bytes32(value: str) -> bytes:
    if not value.startswith("0x") or len(value) != 66:
        raise ValueError("expected a 0x-prefixed bytes32")
    return bytes.fromhex(value[2:])


def compose_batch(
    operations: list[RoutedOperation], contracts: BatchContracts
) -> tuple[BatchCall, ...]:
    """Compose exact backend calls. Deployment addresses and market routes must be verified by caller."""
    _validate_batch({"ops": [entry.operation for entry in operations]})
    _address(contracts.ctf)
    _address(contracts.neg_risk_adapter)
    _address(contracts.collateral)
    calls = []
    for entry in operations:
        op = cast(dict[str, Any], entry.operation)
        kind = op["kind"]
        if entry.route not in {"Ctf", "NegRisk"}:
            raise ValueError("unknown market route")
        if kind in {"SPLIT", "MERGE"}:
            if UUID(op["market_id"]).int == 0:
                raise ValueError("market_id must not be nil")
            condition = _bytes32(op["condition_id"])
            if entry.route == "NegRisk" and kind != "MERGE":
                raise ValueError("neg-risk supports MERGE only")
            target = contracts.neg_risk_adapter if entry.route == "NegRisk" else contracts.ctf
            types = ["address", "bytes32", "bytes32", "uint256[]"]
            values: list[Any] = [contracts.collateral, ZERO_BYTES32, condition, [1, 2]]
            function = {"SPLIT": "splitPosition", "MERGE": "mergePositions"}[kind]
            types.append("uint256")
            values.append(positive_micro(op["shares_micro"]))
            data = _calldata(function, types, values)
        else:
            destination = _address(op["destination"])
            amount = positive_micro(op["amount_micro"])
            if amount < 10_000:
                raise ValueError("withdrawal must be at least 10000 micro units")
            target = contracts.collateral
            data = _calldata("transfer", ["address", "uint256"], [destination, amount])
        calls.append(BatchCall(target=target, data=data))
    return tuple(calls)


def _typed_batch(
    domain: BatchDomain, seq: int, deadline: int, calls: tuple[BatchCall, ...]
) -> dict[str, Any]:
    _address(domain.account)
    _uint(domain.chain_id, 64, "chain_id", nonzero=True)
    _uint(domain.implementation_version, 64, "implementation_version", nonzero=True)
    _uint(seq, 64, "seq")
    _uint(deadline, 64, "deadline", nonzero=True)
    return {
        "types": {
            "EIP712Domain": [
                {"name": "name", "type": "string"},
                {"name": "version", "type": "string"},
                {"name": "chainId", "type": "uint256"},
                {"name": "verifyingContract", "type": "address"},
            ],
            "Call": [
                {"name": "target", "type": "address"},
                {"name": "value", "type": "uint256"},
                {"name": "data", "type": "bytes"},
            ],
            "Batch": [
                {"name": "wallet", "type": "address"},
                {"name": "seq", "type": "uint256"},
                {"name": "deadline", "type": "uint256"},
                {"name": "calls", "type": "Call[]"},
            ],
        },
        "primaryType": "Batch",
        "domain": {
            "name": "AgaraAccount",
            "version": str(domain.implementation_version),
            "chainId": domain.chain_id,
            "verifyingContract": domain.account,
        },
        "message": {
            "wallet": domain.account,
            "seq": seq,
            "deadline": deadline,
            "calls": [{"target": c.target, "value": int(c.value), "data": c.data} for c in calls],
        },
    }


def batch_digest(
    domain: BatchDomain, seq: int, deadline_unix_seconds: int, calls: tuple[BatchCall, ...]
) -> str:
    """Compute a local digest without holding a private key."""
    message = encode_typed_data(
        full_message=_typed_batch(domain, seq, deadline_unix_seconds, calls)
    )
    return "0x" + keccak(b"\x19" + message.version + message.header + message.body).hex()


def sign_batch(
    *,
    private_key: str,
    domain: BatchDomain,
    contracts: BatchContracts,
    operations: list[RoutedOperation],
    seq: int,
    deadline_unix_seconds: int,
    heals_batch_hash: str | None = None,
) -> SignedBatch:
    """Sign typed operations; never signs independently supplied, mismatched calldata."""
    calls = compose_batch(operations, contracts)
    if heals_batch_hash is not None:
        _bytes32(heals_batch_hash)
    message = encode_typed_data(
        full_message=_typed_batch(domain, seq, deadline_unix_seconds, calls)
    )
    signed = Account.sign_message(message, private_key=private_key)
    return SignedBatch(
        "0x" + signed.message_hash.hex(),
        "0x" + signed.signature.hex(),
        seq,
        deadline_unix_seconds,
        json.dumps([entry.operation for entry in operations]),
        calls,
        heals_batch_hash,
    )

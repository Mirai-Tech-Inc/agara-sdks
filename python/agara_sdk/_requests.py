"""Shared wire construction and validation for both HTTP transports."""

from __future__ import annotations

from collections.abc import Mapping
from typing import Any, cast
from urllib.parse import quote

from . import models
from .amounts import MICRO, Amount, positive_micro, to_micro

DEFAULT_BASE_URL = "https://app.sandbox.agara.xyz"


def _compact(**values: Any) -> dict[str, Any]:
    return {
        key: (str(value).lower() if isinstance(value, bool) else value)
        for key, value in values.items()
        if value is not None
    }


def _segment(value: str) -> str:
    if not value or value in {".", ".."}:
        raise ValueError("path identifiers must be nonempty and cannot be dot segments")
    return quote(value, safe="")


def _validate_order(order: Mapping[str, Any]) -> None:
    if order.get("side") not in {"BUY", "SELL"}:
        raise ValueError("side must be BUY or SELL")
    tif = order.get("time_in_force")
    kind = order.get("type")
    if kind == "LIMIT":
        if order.get("price_micro") is None or order.get("shares_micro") is None:
            raise ValueError("LIMIT requires price_micro and shares_micro")
        price = positive_micro(order["price_micro"])
        if price >= MICRO:
            raise ValueError("price must be less than one")
        positive_micro(order["shares_micro"])
        if order.get("collateral_amount_micro") is not None:
            raise ValueError("LIMIT orders require shares; collateral budgets are MARKET BUY only")
        if tif not in {"GTC", "GTD", "FAK", "FOK"}:
            raise ValueError("unsupported time_in_force")
        if order.get("post_only") and tif not in {"GTC", "GTD"}:
            raise ValueError("post_only requires GTC or GTD")
    elif kind == "MARKET":
        if (
            tif not in {"FAK", "FOK"}
            or order.get("post_only")
            or order.get("price_micro") is not None
        ):
            raise ValueError("MARKET orders require FAK/FOK, no price, and no post_only")
        required, forbidden = (
            ("collateral_amount_micro", "shares_micro")
            if order["side"] == "BUY"
            else ("shares_micro", "collateral_amount_micro")
        )
        if order.get(required) is None:
            raise ValueError(f"MARKET {order['side']} requires {required}")
        positive_micro(order[required])
        if order.get(forbidden) is not None:
            raise ValueError(f"MARKET {order['side']} must not include {forbidden}")
    else:
        raise ValueError("type must be LIMIT or MARKET")
    expiration = order.get("expiration_unix_seconds")
    if tif == "GTD":
        if isinstance(expiration, bool) or not isinstance(expiration, int) or expiration <= 0:
            raise ValueError("GTD requires a positive integer expiration_unix_seconds")
    elif expiration is not None:
        raise ValueError("expiration_unix_seconds is only valid for GTD")


def _limit_body(
    *,
    token_id: str,
    side: str,
    price: Amount,
    shares: Amount,
    time_in_force: str,
    post_only: bool,
    expiration_unix_seconds: int | None,
) -> models.LimitOrderRequest:
    body = {
        "token_id": token_id,
        "side": side,
        "type": "LIMIT",
        "time_in_force": time_in_force,
        "price_micro": str(to_micro(price)),
        "shares_micro": str(to_micro(shares)),
        "post_only": post_only,
    }
    if expiration_unix_seconds is not None:
        body["expiration_unix_seconds"] = expiration_unix_seconds
    _validate_order(body)
    return cast(models.LimitOrderRequest, body)


def _market_body(
    *,
    token_id: str,
    side: str,
    shares: Amount | None,
    collateral_amount: Amount | None,
    time_in_force: str,
) -> models.OrderRequest:
    body: dict[str, Any] = {
        "token_id": token_id,
        "side": side,
        "type": "MARKET",
        "time_in_force": time_in_force,
        "post_only": False,
    }
    if shares is not None:
        body["shares_micro"] = str(to_micro(shares))
    if collateral_amount is not None:
        body["collateral_amount_micro"] = str(to_micro(collateral_amount))
    _validate_order(body)
    return cast(models.OrderRequest, body)


def _validate_batch(batch: Mapping[str, Any]) -> None:
    ops = batch.get("ops")
    if not isinstance(ops, list) or not 1 <= len(ops) <= 20:
        raise ValueError("batch requires 1 to 20 operations")
    for op in ops:
        if op.get("kind") not in {"SPLIT", "MERGE", "WITHDRAW"}:
            raise ValueError("presigned trader batches support SPLIT, MERGE and WITHDRAW")
        field = "amount_micro" if op["kind"] == "WITHDRAW" else "shares_micro"
        positive_micro(op[field])
    for key in ["seq", "deadline_unix_seconds"]:
        if key in batch:
            value = batch[key]
            if isinstance(value, bool) or not isinstance(value, int) or not 0 <= value < 2**64:
                raise ValueError(f"{key} must be a uint64")


def batch_is_terminal(batch: models.AccountBatchStatusDto) -> bool:
    """FAILED is still changing until optimistic effects have unwound."""
    return batch["status"] in {"SETTLED", "FAILED_DIVERGENT"} or (
        batch["status"] == "FAILED" and batch["unwound_at"] is not None
    )

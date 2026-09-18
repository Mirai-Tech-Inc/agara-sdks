"""Shared HTTP transport policy."""

from __future__ import annotations

import math
import time
from urllib.parse import urlsplit

from .errors import AuthError

PUBLIC_TRADE_PATHS = frozenset(
    {"/trade/v1/status", "/trade/v1/lp-incentives", "/trade/v1/lp-incentives/categories"}
)


def validate_settings(base_url: str, timeout: float) -> None:
    finite_positive(timeout, "timeout")
    parts = urlsplit(base_url)
    if parts.scheme not in {"http", "https"} or not parts.hostname:
        raise ValueError("base_url must be an absolute HTTP(S) URL")
    if parts.username or parts.password or parts.query or parts.fragment:
        raise ValueError("base_url must not contain credentials, query parameters or a fragment")


def finite_positive(value: float, name: str) -> None:
    if (
        isinstance(value, bool)
        or not isinstance(value, (int, float))
        or not math.isfinite(value)
        or value <= 0
    ):
        raise ValueError(f"{name} must be finite and positive")


def require_auth(path: str, token: str | None) -> None:
    public = (
        path.startswith("/api/v1/")
        or path.startswith("/trade/v1/orderbook/")
        or path in PUBLIC_TRADE_PATHS
    )
    if not public and not token:
        raise AuthError(401, "a token is required for this operation")


def check_deadline(deadline: float | None) -> None:
    if deadline is not None and time.monotonic() >= deadline:
        raise TimeoutError("operation did not complete before timeout")


def request_timeout(timeout: float, deadline: float | None) -> float:
    check_deadline(deadline)
    return timeout if deadline is None else min(timeout, max(0.001, deadline - time.monotonic()))

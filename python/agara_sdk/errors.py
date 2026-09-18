"""HTTP and protocol-neutral failures with conservative recovery."""

from __future__ import annotations

import math
import re
from dataclasses import dataclass, replace
from dataclasses import field as dataclass_field
from typing import Any, Mapping, NoReturn, Optional

from ._problem_registry import METADATA

_KNOWN_RECOVERY_STRATEGIES = frozenset(
    {"none", "retry", "retry_after", "refresh_identity_token", "check_status"}
)


def _is_recovery_uuid(value: Any) -> bool:
    if not isinstance(value, str) or len(value) != 36:
        return False
    for index, character in enumerate(value):
        if index in {8, 13, 18, 23}:
            if character != "-":
                return False
        elif character not in "0123456789abcdefABCDEF":
            return False
    compact = value.replace("-", "")
    if compact in {"0" * 32, "f" * 32, "F" * 32}:
        return True
    return compact[12] in "12345678" and compact[16] in "89abAB"


def _is_recovery_resource(value: Any) -> bool:
    if not isinstance(value, Mapping) or not isinstance(value.get("kind"), str):
        return False
    kind = value["kind"]
    if kind == "order":
        return set(value) == {"kind", "order_id"} and _is_recovery_uuid(value.get("order_id"))
    if kind == "batch":
        batch_hash = value.get("batch_hash")
        return (
            set(value) == {"kind", "batch_hash"}
            and isinstance(batch_hash, str)
            and len(batch_hash) == 66
            and batch_hash.startswith("0x")
            and all(character in "0123456789abcdefABCDEF" for character in batch_hash[2:])
        )
    if kind == "group":
        return set(value) == {"kind", "group_id"} and _is_recovery_uuid(value.get("group_id"))
    return kind == "wallet_status" and set(value) == {"kind"}


@dataclass(frozen=True)
class Recovery:
    """Sanitized recovery guidance. Future or malformed strategies are retained
    for diagnostics but are inert: the SDK never retries or refreshes because of
    a strategy it does not understand."""

    strategy: str
    after_seconds: Optional[float] = None
    resource: Optional[dict[str, Any]] = None
    known: bool = True

    @property
    def is_retryable(self) -> bool:
        return self.known and self.strategy in {"retry", "retry_after"}

    @classmethod
    def from_wire(cls, value: Any) -> "Recovery":
        if not isinstance(value, Mapping) or not isinstance(value.get("strategy"), str):
            return cls(strategy="unknown", known=False)

        strategy = value["strategy"]
        if strategy not in _KNOWN_RECOVERY_STRATEGIES:
            return cls(strategy=strategy, known=False)
        if strategy in {"none", "retry", "refresh_identity_token"}:
            return cls(strategy=strategy, known=set(value) == {"strategy"})
        if strategy == "retry_after":
            after = value.get("after_seconds")
            valid = (
                set(value) == {"strategy", "after_seconds"}
                and isinstance(after, int)
                and not isinstance(after, bool)
                and 0 <= after <= 86_400
            )
            return cls(
                strategy=strategy,
                after_seconds=float(after) if valid and isinstance(after, int) else None,
                known=valid,
            )

        resource = value.get("resource")
        valid = set(value) == {"strategy", "resource"} and _is_recovery_resource(resource)
        return cls(
            strategy=strategy,
            resource=dict(resource) if valid and isinstance(resource, Mapping) else None,
            known=valid,
        )


@dataclass(frozen=True)
class PublicFailure:
    """Protocol-neutral failure used by async order and stream responses."""

    code: str
    title: str
    detail: Optional[str]
    recovery: Recovery
    raw: dict[str, Any] = dataclass_field(default_factory=dict)

    @classmethod
    def from_wire(cls, value: Any) -> "PublicFailure":
        if not isinstance(value, Mapping):
            return _internal_failure()
        code = value.get("code")
        title = value.get("title")
        if not isinstance(code, str) or not isinstance(title, str):
            return _internal_failure()
        detail = value.get("detail")
        recovery = Recovery.from_wire(value.get("recovery"))
        metadata = METADATA.get(code)
        valid = _failure_shape(value) and _registered_failure(value, metadata)
        if not valid or metadata is None:
            recovery = replace(recovery, known=False)
        return cls(
            code=code,
            title=title,
            detail=detail if isinstance(detail, str) else None,
            recovery=recovery,
            raw=dict(value),
        )


@dataclass(frozen=True)
class ProblemDetails:
    """Origin HTTP Problem Details returned for every API failure."""

    code: str
    title: str
    detail: Optional[str]
    recovery: Recovery
    type_uri: str
    status: int
    request_id: Optional[str]
    field_errors: tuple[dict[str, Any], ...] = ()

    @classmethod
    def from_wire(cls, value: Any) -> Optional["ProblemDetails"]:
        if not isinstance(value, Mapping):
            return None
        required = ("type", "title", "status", "code", "recovery")
        if any(key not in value for key in required):
            return None
        if not isinstance(value.get("type"), str) or not isinstance(value.get("status"), int):
            return None
        failure = PublicFailure.from_wire(value)
        metadata = METADATA.get(failure.code)
        recovery = failure.recovery
        valid = _http_shape(value, metadata)
        if metadata is not None:
            valid = (
                valid
                and value["type"] == metadata["urn"]
                and value["status"] == metadata["http_status"]
            )
        if not valid:
            recovery = replace(recovery, known=False)
        request_id = value.get("request_id")
        raw_field_errors = value.get("field_errors")
        field_errors = (
            tuple(item for item in raw_field_errors if isinstance(item, dict))
            if isinstance(raw_field_errors, list)
            else ()
        )
        return cls(
            code=failure.code,
            title=failure.title,
            detail=failure.detail,
            recovery=recovery,
            type_uri=value["type"],
            status=value["status"],
            request_id=request_id if isinstance(request_id, str) else None,
            field_errors=field_errors,
        )


def _internal_failure() -> PublicFailure:
    return PublicFailure(
        code="internal_error",
        title="Internal server error",
        detail=None,
        recovery=Recovery(strategy="none", known=False),
    )


class AgaraError(Exception):
    """Base for every error this SDK raises."""

    def __init__(
        self,
        status_code: int,
        message: str,
        retry_after: Optional[float] = None,
        problem: Optional[ProblemDetails] = None,
    ):
        super().__init__(f"[{status_code}] {message}")
        self.status_code = status_code
        self.message = message
        self.problem = problem
        self.code = problem.code if problem else None
        self.title = problem.title if problem else None
        self.detail = problem.detail if problem else None
        self.request_id = problem.request_id if problem else None
        self.recovery = problem.recovery if problem else None
        self.field_errors = problem.field_errors if problem else ()
        #: Seconds the server asked the caller to wait before retrying, from the
        #: `Retry-After` (or `x-ratelimit-reset`) header. `None` when absent.
        self.retry_after = retry_after
        self.raw_body: Any = None

    @property
    def is_retryable(self) -> bool:
        """Whether the server explicitly permits repeating the same request."""
        if self.problem is not None:
            return self.problem.recovery.is_retryable
        return False


class BadRequestError(AgaraError):
    """400 — malformed request body or invalid parameter values."""


class AuthError(AgaraError):
    """401 — missing / invalid / revoked / expired token."""


class ForbiddenError(AgaraError):
    """403 — token is valid but lacks the required scope. Re-issue
    with broader scopes from the FE; the bearer itself is fine."""


class NotFoundError(AgaraError):
    """404 — order, market, or token id doesn't exist or isn't yours."""


class ConflictError(AgaraError):
    """409 — e.g. cancelling an order already in a terminal state, or
    submitting a signed order whose `order_hash` already exists."""


class MethodNotAllowedError(AgaraError):
    """405 — the route exists but does not support this method."""


class GoneError(AgaraError):
    """410 — the endpoint was retired."""


class PayloadTooLargeError(AgaraError):
    """413 — the request body exceeded the server limit."""


class UnsupportedMediaTypeError(AgaraError):
    """415 — the request was not sent as supported JSON."""


class FailedDependencyError(AgaraError):
    """424 — a prerequisite such as wallet setup is incomplete."""


class TooEarlyError(AgaraError):
    """425 — the requested derived result is not ready yet."""


class UpgradeRequiredError(AgaraError):
    """426 — the route requires a WebSocket upgrade."""


class RejectedError(AgaraError):
    """422 — order rejected (insufficient balance / shares,
    FOK couldn't fill, post-only would cross, market halted)."""


class RateLimitedError(AgaraError):
    """429 — per-tier token bucket exhausted. `retry_after` carries the
    server's `Retry-After` hint (seconds) when present; back off and retry."""


class ServerError(AgaraError):
    """5xx — platform or dependency failure. Retry only when
    ``is_retryable`` is true; status alone is not a retry signal."""


def _parse_retry_after(headers: Optional[Any]) -> Optional[float]:
    """Read the retry hint (seconds) from response headers. Prefers
    `Retry-After`, falls back to `x-ratelimit-reset`; both are emitted as
    integer seconds by the router. Returns None when absent or unparseable."""
    if headers is None:
        return None
    for key in ("retry-after", "x-ratelimit-reset"):
        raw = headers.get(key)
        if raw is None:
            continue
        try:
            value = float(raw)
            return value if math.isfinite(value) and 0 <= value <= 86400 else None
        except (TypeError, ValueError):
            continue
    return None


def _raise_api_error(status_code: int, body: Any, headers: Optional[Any] = None) -> NoReturn:
    """Map an HTTP status to the matching typed exception and raise it.
    Shared by the sync and async clients so the mapping lives once."""
    problem = ProblemDetails.from_wire(body)
    if problem is not None and problem.status != status_code:
        problem = None
    if problem is not None:
        message = problem.detail or problem.title
    elif isinstance(body, Mapping) and isinstance(body.get("error"), str):
        message = body["error"]
    else:
        message = str(body or "")

    retry_after = _parse_retry_after(headers)
    if retry_after is None and problem and problem.recovery.known:
        retry_after = problem.recovery.after_seconds

    error_types = {
        400: BadRequestError,
        401: AuthError,
        403: ForbiddenError,
        404: NotFoundError,
        405: MethodNotAllowedError,
        409: ConflictError,
        410: GoneError,
        413: PayloadTooLargeError,
        415: UnsupportedMediaTypeError,
        422: RejectedError,
        424: FailedDependencyError,
        425: TooEarlyError,
        426: UpgradeRequiredError,
        429: RateLimitedError,
    }
    error_type = error_types.get(status_code)
    if error_type is not None:
        error = error_type(status_code, message, retry_after, problem)
    elif 500 <= status_code < 600:
        error = ServerError(status_code, message, retry_after, problem)
    else:
        error = AgaraError(status_code, message, retry_after, problem)
    error.raw_body = body
    raise error


class TransportError(AgaraError):
    """HTTP transport failed. A mutation may have reached the server; reconcile its hash."""

    def __init__(self, method: str, cause: Exception):
        super().__init__(0, f"{method} transport failed ({type(cause).__name__})")
        self.method = method
        self.mutation_outcome_unknown = method not in {"GET", "HEAD", "OPTIONS"}


class ProtocolError(AgaraError):
    """A successful HTTP response was not a valid JSON object."""

    def __init__(self, message: str):
        super().__init__(0, message)


def _text(value: Any, maximum: int) -> bool:
    return isinstance(value, str) and 1 <= len(value) <= maximum


def _code(value: Any) -> bool:
    return isinstance(value, str) and re.fullmatch(r"[a-z][a-z0-9_]{0,63}", value) is not None


def _failure_shape(value: Mapping[str, Any]) -> bool:
    allowed = {"code", "title", "detail", "recovery"}
    if "status" in value or "type" in value:
        allowed |= {"type", "status", "request_id", "field_errors"}
    if set(value) - allowed:
        return False
    return (
        _code(value.get("code"))
        and _text(value.get("title"), 128)
        and ("detail" not in value or _text(value["detail"], 512))
        and isinstance(value.get("recovery"), Mapping)
    )


def _registered_failure(value: Mapping[str, Any], metadata: dict[str, Any] | None) -> bool:
    if metadata is None:
        return False
    recovery = Recovery.from_wire(value.get("recovery"))
    expected = metadata["recovery"]
    if not recovery.known or recovery.strategy != expected["strategy"]:
        return False
    if recovery.strategy == "check_status" and (recovery.resource or {}).get(
        "kind"
    ) != expected.get("resource_kind"):
        return False
    if value.get("title") != metadata["title"]:
        return False
    if "detail" in value and value["detail"] != metadata["public_detail"]:
        return False
    return True


def _http_shape(value: Mapping[str, Any], metadata: dict[str, Any] | None) -> bool:
    if type(value.get("status")) is not int or not 400 <= value["status"] <= 599:
        return False
    if (
        not isinstance(value.get("type"), str)
        or re.fullmatch(r"urn:agara:problem:[a-z][a-z0-9-]{0,63}", value["type"]) is None
    ):
        return False
    if set(value) - {
        "type",
        "title",
        "status",
        "code",
        "detail",
        "request_id",
        "recovery",
        "field_errors",
    }:
        return False
    if "request_id" in value:
        if not _is_recovery_uuid(value["request_id"]):
            return False
        if metadata is not None and "origin_http" not in metadata["representations"]:
            return False
    elif metadata is not None and "edge_http" not in metadata["representations"]:
        return False
    fields = value.get("field_errors", [])
    if not isinstance(fields, list) or len(fields) > 32:
        return False
    for field in fields:
        if not isinstance(field, Mapping) or set(field) != {"path", "code", "message"}:
            return False
        path = field["path"]
        if (
            not isinstance(path, list)
            or len(path) > 16
            or not _code(field["code"])
            or not _text(field["message"], 256)
        ):
            return False
        if any(
            not (_text(part, 64) or type(part) is int and 0 <= part <= 2147483647) for part in path
        ):
            return False
    return True

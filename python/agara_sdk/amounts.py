"""Exact decimal and micro-unit conversions; float inputs are deliberately rejected."""

from decimal import Decimal, InvalidOperation, localcontext
from typing import TypeAlias

Amount: TypeAlias = Decimal | str | int
MICRO = 1_000_000
MAX_MICRO = (1 << 63) - 1
MIN_MICRO = -(1 << 63)


def to_micro(value: Amount) -> int:
    """Convert whole units without rounding. Reject fractions below one micro unit."""
    if isinstance(value, (float, bool)) or not isinstance(value, (Decimal, str, int)):
        raise TypeError("use Decimal, a decimal string, or an integer; floats are not exact")
    try:
        number = Decimal(value)
    except InvalidOperation as exc:
        raise ValueError("invalid decimal amount") from exc
    if not number.is_finite():
        raise ValueError("amount must be finite")
    with localcontext() as context:
        context.prec = max(
            40, len(number.as_tuple().digits) + abs(int(number.as_tuple().exponent)) + 7
        )
        scaled = number * MICRO
        if scaled != scaled.to_integral_value():
            raise ValueError("amount must be an exact multiple of 0.000001")
        result = int(scaled)
    if not MIN_MICRO <= result <= MAX_MICRO:
        raise ValueError("amount exceeds signed 64-bit micro units")
    return result


def micro_to_decimal(value: str | int | None) -> Decimal | None:
    """Convert integer wire micro units to an exact Decimal, retaining None."""
    if value is None:
        return None
    if isinstance(value, bool) or not isinstance(value, (str, int)):
        raise TypeError("micro units must be an integer or integer string")
    number = int(value)
    if isinstance(value, str) and str(number) != value:
        raise ValueError("micro units must be a canonical integer string")
    with localcontext() as context:
        context.prec = max(40, len(str(abs(number))) + 6)
        return Decimal(number) / MICRO


def positive_micro(value: int | str) -> int:
    if isinstance(value, bool) or not isinstance(value, (str, int)):
        raise TypeError("micro units must be an integer or integer string")
    result = int(value)
    if isinstance(value, str) and str(result) != value:
        raise ValueError("micro units must be a canonical integer string")
    if not 0 < result <= MAX_MICRO:
        raise ValueError("micro units must be positive and fit signed 64 bits")
    return result

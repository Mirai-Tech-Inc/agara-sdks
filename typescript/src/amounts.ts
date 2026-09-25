/** Number of micro units in one collateral unit or one share. */
export const MICRO = 1_000_000n;
/**
 * Converts a decimal amount to an exact integer without rounding.
 *
 * @param value - Decimal string with an optional minus sign; exponents and whitespace are rejected.
 * @param decimals - Fractional precision from 0 through 255; defaults to 6 for micro units.
 * @returns The signed amount scaled by ten to the power of `decimals`.
 * @throws TypeError - If the string is not a supported decimal representation.
 * @throws RangeError - If the precision is invalid or the string has too many fractional digits.
 * @example
 * ```ts
 * parseUnits("1.23"); // 1230000n
 * ```
 */
export function parseUnits(value: string, decimals = 6): bigint {
  if (!Number.isInteger(decimals) || decimals < 0 || decimals > 255)
    throw new RangeError("Invalid decimals");
  if (!/^-?\d+(\.\d+)?$/.test(value))
    throw new TypeError("Expected a decimal string without exponent notation");
  const negative = value.startsWith("-");
  const [whole = "0", fraction = ""] = (negative ? value.slice(1) : value).split(".");
  if (fraction.length > decimals)
    throw new RangeError("Amount has more decimals than the asset supports");
  const result =
    BigInt(whole) * 10n ** BigInt(decimals) + BigInt(fraction.padEnd(decimals, "0") || "0");
  return negative ? -result : result;
}
/**
 * Formats an exact integer as a decimal amount, omitting trailing fractional zeros.
 *
 * @param value - Integer amount as a bigint or a string accepted by `BigInt`.
 * @param decimals - Fractional precision from 0 through 255; defaults to 6 for micro units.
 * @returns A signed decimal string without exponent notation.
 * @throws RangeError - If `decimals` is outside the supported integer range.
 * @throws SyntaxError - If a string cannot be converted to a bigint.
 * @example
 * ```ts
 * formatUnits(1230000n); // "1.23"
 * ```
 */
export function formatUnits(value: bigint | string, decimals = 6): string {
  if (!Number.isInteger(decimals) || decimals < 0 || decimals > 255)
    throw new RangeError("Invalid decimals");
  const amount = BigInt(value);
  const sign = amount < 0n ? "-" : "";
  const digits = (amount < 0n ? -amount : amount).toString().padStart(decimals + 1, "0");
  if (!decimals) return sign + digits;
  const fraction = digits.slice(-decimals).replace(/0+$/, "");
  return sign + digits.slice(0, -decimals) + (fraction ? `.${fraction}` : "");
}
/**
 * Parses an exact integer and enforces inclusive bounds.
 *
 * @param value - Safe integer number, bigint, unsigned decimal string, or `0x` hexadecimal string.
 * @param label - Field name included in validation errors.
 * @param maximum - Inclusive upper bound; defaults to the maximum unsigned 256-bit integer.
 * @param minimum - Inclusive lower bound; defaults to zero.
 * @returns The integer as a bigint.
 * @throws TypeError - If a number is unsafe or a string has an unsupported integer format.
 * @throws RangeError - If the parsed value is outside the supplied bounds.
 * @remarks
 * Signed strings are rejected even when `minimum` is negative; use a number or bigint instead.
 */
export function integer(
  value: string | number | bigint,
  label: string,
  maximum = (1n << 256n) - 1n,
  minimum = 0n,
): bigint {
  if (typeof value === "number" && !Number.isSafeInteger(value))
    throw new TypeError(`${label} must be an exact integer`);
  if (typeof value === "string" && !/^(?:\d+|0x[0-9a-fA-F]+)$/.test(value))
    throw new TypeError(`${label} must be an integer`);
  const result = BigInt(value);
  if (result < minimum || result > maximum)
    throw new RangeError(`${label} is outside its permitted range`);
  return result;
}

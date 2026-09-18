export const MICRO = 1_000_000n;
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

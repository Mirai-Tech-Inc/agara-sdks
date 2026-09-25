import { parse, stringify } from "lossless-json";
export function parseJson(text: string): unknown {
  return parse(text, undefined, (raw) => {
    if (/^-?\d+$/.test(raw)) {
      const value = BigInt(raw);
      return value <= BigInt(Number.MAX_SAFE_INTEGER) && value >= BigInt(Number.MIN_SAFE_INTEGER)
        ? Number(value)
        : value;
    }
    const value = Number(raw);
    if (!Number.isFinite(value)) throw new TypeError("Non-finite JSON number");
    return value;
  });
}
export function stringifyJson(value: unknown): string {
  assertSafeNumbers(value);
  const result = stringify(value);
  if (result === undefined) throw new TypeError("Value is not JSON serializable");
  return result;
}
export function assertSafeNumbers(value: unknown): void {
  if (
    typeof value === "number" &&
    (!Number.isFinite(value) || (Number.isInteger(value) && !Number.isSafeInteger(value)))
  )
    throw new TypeError("Use bigint or a decimal string for exact integers");
  if (Array.isArray(value)) for (const item of value) assertSafeNumbers(item);
  else if (value && typeof value === "object")
    for (const item of Object.values(value)) assertSafeNumbers(item);
}
export function isObject(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

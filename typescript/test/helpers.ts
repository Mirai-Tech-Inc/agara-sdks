import fs from "node:fs";
import { fileURLToPath } from "node:url";
export function contract(name: string): Record<string, unknown> {
  return JSON.parse(
    fs.readFileSync(fileURLToPath(new URL(`../contracts/${name}.json`, import.meta.url)), "utf8"),
  );
}
export type Schema = Record<string, unknown>;
export function sample(schema: Schema, definitions: Record<string, Schema>): unknown {
  if (schema.$ref)
    return sample(definitions[String(schema.$ref).split("/").at(-1) ?? ""] ?? {}, definitions);
  if (Array.isArray(schema.oneOf)) return sample(schema.oneOf[0] as Schema, definitions);
  if (Array.isArray(schema.anyOf)) return sample(schema.anyOf[0] as Schema, definitions);
  if (Array.isArray(schema.allOf))
    return Object.assign({}, ...schema.allOf.map((s) => sample(s as Schema, definitions)));
  if (schema.const !== undefined) return schema.const;
  if (Array.isArray(schema.enum)) return schema.enum[0];
  const type = Array.isArray(schema.type) ? schema.type.find((t) => t !== "null") : schema.type;
  if (type === "object")
    return Object.fromEntries(
      ((schema.required as string[]) ?? []).map((key) => [
        key,
        sample((schema.properties as Record<string, Schema>)[key] ?? {}, definitions),
      ]),
    );
  if (type === "array")
    return Array.from({ length: Math.max(Number(schema.minItems ?? 1), 1) }, () =>
      sample((schema.items as Schema) ?? {}, definitions),
    );
  if (type === "integer" || type === "number") return Math.max(Number(schema.minimum ?? 1), 1);
  if (type === "boolean") return false;
  if (type === "null") return null;
  if (type === "string") {
    if (schema.format === "uuid") return "11111111-1111-4111-8111-111111111111";
    if (schema.format === "date-time") return "2026-09-17T12:00:00Z";
    if (schema.format === "date") return "2026-09-17";
    if (schema.format === "int64") return "1000000";
    if (typeof schema.example === "string") return schema.example;
    if (schema.pattern) {
      const pattern = String(schema.pattern);
      if (pattern.includes("0x")) return `0x${"11".repeat(32)}`;
      if (pattern.includes("[0-9]") || pattern.includes("\\d")) return "1";
    }
    return "x".repeat(Math.max(Number(schema.minLength ?? 1), 1));
  }
  return {};
}
export const traderOptions = { baseUrl: "https://sdk.test", token: "agt_test" };

import { describe, expect, it } from "vitest";
import { AgaraError, parseFailure, parseProblem } from "../src/errors.js";
import { contract } from "./helpers.js";

interface Fixture {
  name: string;
  schema: string;
  contract_valid: boolean;
  value: Record<string, unknown>;
}
const fixtures = contract("problem-fixtures").fixtures as Fixture[];
describe("canonical Problem Details corpus", () => {
  for (const fixture of fixtures.filter(
    (f) =>
      f.schema === "origin-problem-details.schema.json" || f.schema === "edge-problem.schema.json",
  ))
    it(fixture.name, () => {
      const parsed = parseProblem(
        fixture.value,
        Number(fixture.value.status),
        fixture.schema.startsWith("origin") ? "origin" : "edge",
      );
      expect(parsed !== undefined).toBe(fixture.contract_valid);
      if (parsed && !parsed.known)
        expect(new AgaraError(parsed.status, fixture.value).isRetryable).toBe(false);
    });
  for (const fixture of fixtures.filter((f) => f.schema === "public-failure.schema.json"))
    it(fixture.name, () => {
      if (fixture.contract_valid) {
        const parsed = parseFailure(fixture.value);
        expect(parsed.code).toBe(fixture.value.code);
        if (!parsed.known) expect(parsed.recovery.strategy).toBe("unknown");
      } else expect(() => parseFailure(fixture.value)).toThrow();
    });
});
it("retains Retry-After for diagnostic use without authorizing unknown retry", () => {
  const e = new AgaraError(
    503,
    {
      type: "urn:agara:problem:future",
      title: "Future",
      status: 503,
      code: "future",
      recovery: { strategy: "retry" },
    },
    new Headers({ "retry-after": "120" }),
  );
  expect(e.retryAfter).toBe(120);
  expect(e.isRetryable).toBe(false);
});

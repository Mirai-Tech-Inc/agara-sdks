import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { afterEach, describe, expect, it } from "vitest";

const checker = fileURLToPath(new URL("../scripts/check-docs.mjs", import.meta.url));
const temporaryProjects: string[] = [];

function project(
  files: Record<string, string>,
  exports: Record<string, unknown> = { ".": "./dist/index.js" },
) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "agara-docs-"));
  temporaryProjects.push(root);
  const contents = {
    "package.json": JSON.stringify({ name: "docs-fixture", type: "module", exports }),
    "tsconfig.json": JSON.stringify({
      compilerOptions: {
        target: "ES2022",
        module: "NodeNext",
        moduleResolution: "NodeNext",
        rootDir: "src",
        outDir: "dist",
        noLib: true,
        types: [],
        strict: true,
      },
      include: ["src/**/*.ts"],
    }),
    ...files,
  };
  for (const [file, source] of Object.entries(contents)) {
    const destination = path.join(root, file);
    fs.mkdirSync(path.dirname(destination), { recursive: true });
    fs.writeFileSync(destination, source);
  }
  return root;
}

function check(root: string) {
  const result = spawnSync(process.execPath, [checker, root], { encoding: "utf8" });
  if (result.error) throw result.error;
  return { status: result.status, output: result.stdout + result.stderr };
}

afterEach(() => {
  for (const root of temporaryProjects.splice(0)) fs.rmSync(root, { recursive: true, force: true });
});

describe("public TSDoc gate", () => {
  it("uses package subpath source mappings even when emitted paths have different names", () => {
    const result = check(
      project(
        {
          "src/implementation.ts": "export const undocumented = 1;",
          "tsconfig.json": JSON.stringify({
            compilerOptions: {
              noLib: true,
              types: [],
              paths: { "docs-fixture/custom": ["./src/implementation.ts"] },
            },
            include: ["src/**/*.ts"],
          }),
        },
        { "./custom": { types: "./lib/public.d.ts", import: "./lib/public.js" } },
      ),
    );
    expect(result.status).toBe(1);
    expect(result.output).toContain("undocumented: missing public TSDoc summary");
    expect(result.output).toContain("across 1 entrypoints");
  });

  it("finds undocumented exports and client methods through barrels and subpath entrypoints", () => {
    const result = check(
      project(
        {
          "src/index.ts": 'export { Client } from "./client.js";',
          "src/client.ts": `
        /** Reads the service status. */
        export class Client { read(): void {} }
        export function internalHelper() {}
      `,
          "src/signing.ts": "export const domain = 1;",
        },
        { ".": "./dist/index.js", "./signing": "./dist/signing.js" },
      ),
    );
    expect(result.status).toBe(1);
    expect(result.output).toContain("src/client.ts:3:");
    expect(result.output).toContain("Client.read: missing public TSDoc summary");
    expect(result.output).toContain("domain: missing public TSDoc summary");
    expect(result.output).not.toContain("internalHelper:");
    expect(result.output).toContain("across 2 entrypoints");
  });

  it("checks union fields exposed through internal aliases without documenting the helper itself", () => {
    const result = check(
      project({
        "src/index.ts": `
        type Internal = { kind: "ready"; options: { timeout: number } } | { kind: "closed" };
        /** Reports the current connection state. */
        export type Status = Internal;
      `,
      }),
    );
    expect(result.status).toBe(1);
    expect(result.output.match(/Status.kind: missing/g)).toHaveLength(2);
    expect(result.output).toContain("Status.options.timeout: missing public TSDoc summary");
    expect(result.output).not.toContain("Internal: missing");
  });

  it("deduplicates reexports and excludes private members, implementation signatures and internals", () => {
    const result = check(
      project(
        {
          "src/index.ts": 'export { Client as First } from "./client.js";',
          "src/other.ts": 'export { Client as Second } from "./client.js";',
          "src/client.ts": `
        class Internal { missing(): void {} }
        /** Reads values from the service. */
        export class Client {
          private secret = new Internal();
          protected retry(): void {}
          #state = 1;
          /** Creates a client.
           * @param key - Authentication token.
           */
          constructor(public readonly key: string) {}
          /** Reads a numeric identifier.
           * @param id - Numeric resource identifier.
           */
          read(id: number): string;
          /** Reads a textual identifier.
           * @param id - Textual resource identifier.
           */
          read(id: string): string;
          read(id: number | string): string { return String(id); }
        }
      `,
        },
        { ".": "./dist/index.js", "./other": "./dist/other.js" },
      ),
    );
    expect(result.status, result.output).toBe(0);
    expect(result.output).toContain("4 declarations checked across 2 entrypoints");
  });

  it("requires the inherited public constructor while excluding protected base implementation", () => {
    const result = check(
      project({
        "src/index.ts": `
        class Transport {
          constructor(options: string) {}
          protected request(): void {}
        }
        /** Connects to the service. */
        export class Client extends Transport {}
      `,
      }),
    );
    expect(result.status).toBe(1);
    expect(result.output).toContain("Client.constructor: missing public TSDoc summary");
    expect(result.output).not.toContain("Transport: missing");
    expect(result.output).not.toContain("request:");
  });

  it("excludes inherited external declarations and inferred implementation object literals", () => {
    const result = check(
      project({
        "src/index.ts": `
        import { External } from "external";
        export type { External } from "external";
        /** Extends the service connection. */
        export class Client extends External {
          /** Reports the current state. */
          state() { return { implementationDetail: "ready" }; }
        }
      `,
        "node_modules/external/package.json": JSON.stringify({
          name: "external",
          type: "module",
          types: "index.d.ts",
        }),
        "node_modules/external/index.d.ts": `
        /** @invalidTag */
        export class External {
          constructor(url: string);
          externalMethod(argument: string): void;
          externalField: { value: string };
        }
      `,
      }),
    );
    expect(result.status, result.output).toBe(0);
    expect(result.output).toContain("2 declarations checked");
  });

  it("rejects malformed TSDoc and undocumented callable parameters", () => {
    const result = check(
      project({
        "src/index.ts": `
        /** Reads a value from {@link Missing.
         * @param other - Resource identifier.
         */
        export function read(id: string): void {}
        /** TODO add description */
        export const pending = 1;
        /** value */
        export const value = 1;
      `,
      }),
    );
    expect(result.status).toBe(1);
    expect(result.output).toContain("tsdoc-");
    expect(result.output).toContain("@param other does not name a parameter");
    expect(result.output).toContain("missing @param id - description");
    expect(result.output).toContain("pending: write a substantive public TSDoc summary");
    expect(result.output).toContain("value: write a substantive public TSDoc summary");
  });

  it("scopes parameter tags to each callable and accepts callbacks documented by their property", () => {
    const root = project({
      "src/index.ts": `
        /** Selects how responses are observed. */
        export interface Options {
          /** Receives each response status. */
          onResponse?: (status: number) => void;
        }
        /** Requests a response.
         * @param options - Observer configuration.
         */
        export function read(options: Options): void {}
      `,
    });
    expect(check(root).status).toBe(0);
    const file = path.join(root, "src/index.ts");
    fs.writeFileSync(
      file,
      fs
        .readFileSync(file, "utf8")
        .replace(
          "@param options - Observer configuration.",
          "@param status - Incorrectly documents a nested callback argument.",
        ),
    );
    const result = check(root);
    expect(result.status).toBe(1);
    expect(result.output).toContain("read: @param status does not name a parameter");
    expect(result.output).toContain("read: missing @param options");
    expect(result.output).not.toContain("missing @param status");
  });

  it("exempts OpenAPI declarations while checking aliases and generated endpoint wrappers", () => {
    const root = project({
      "src/index.ts": `
        import type { Model } from "./generated/schema.js";
        export type { Model } from "./generated/schema.js";
        /** Represents a model returned by the server. */
        export type Response = Model;
        /** Sends requests to the API. */
        export class Client {
          /** Fetches the current model.
           * @param id - Model identifier.
           */
          getModel(id: string): Model { return { field: id }; }
        }
      `,
      "src/generated/schema.ts": "/** @invalidTag */ export interface Model { field: string; }",
    });
    const passing = check(root);
    expect(passing.status, passing.output).toBe(0);
    expect(passing.output).toContain("3 declarations checked");
    expect(passing.output).toContain("exempt (1 files reached)");
    const file = path.join(root, "src/index.ts");
    fs.writeFileSync(
      file,
      fs
        .readFileSync(file, "utf8")
        .replace(
          "/** Fetches the current model.\n           * @param id - Model identifier.\n           */",
          "// Generated from the endpoint contract.",
        ),
    );
    const result = check(root);
    expect(result.status).toBe(1);
    expect(result.output).toContain("Client.getModel: missing public TSDoc summary");
  });

  it("counts async generator methods and handwritten option and return DTO fields", () => {
    const result = check(
      project({
        "src/index.ts": `
        /** Reads pages from the service. */
        export class Client {
          async *pages(options: { cursor: string }) { yield options.cursor; }
        }
        /** Reports the active cursor. */
        export function current(): { cursor: string } { return { cursor: "next" }; }
      `,
      }),
    );
    expect(result.status).toBe(1);
    expect(result.output).toContain("Client.pages: missing public TSDoc summary");
    expect(result.output).toContain("Client.pages.options.cursor: missing public TSDoc summary");
    expect(result.output).toContain("current result.cursor: missing public TSDoc summary");
  });
});

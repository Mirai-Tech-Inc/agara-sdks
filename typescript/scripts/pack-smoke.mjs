import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import ts from "typescript";

const root = fs.realpathSync(new URL("..", import.meta.url));
const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "agara-ts-pack-"));
try {
  const packed = JSON.parse(
    execFileSync("npm", ["pack", "--json", "--pack-destination", temporary], {
      cwd: root,
      encoding: "utf8",
    }),
  );
  fs.writeFileSync(
    path.join(temporary, "package.json"),
    JSON.stringify({ private: true, type: "module" }),
  );
  execFileSync(
    "npm",
    [
      "install",
      "--prefer-offline",
      "--ignore-scripts",
      "--omit=optional",
      path.join(temporary, packed[0].filename),
    ],
    { cwd: temporary, stdio: "pipe" },
  );
  const declarations = path.join(temporary, "node_modules", "@agara", "sdk", "dist");
  for (const [file, symbol, member] of [
    ["endpoints", "TraderClient", "placeOrder"],
    ["amounts", "parseUnits"],
    ["signing", "hashOrder"],
    ["types", "LimitOrderRequest"],
    ["errors", "TransportError"],
    ["transport", "ClientOptions", "timeoutMs"],
    ["workflows", "AgaraClient", "waitForOrder"],
    ["stream-client", "AgaraStream", "setSubscriptions"],
    ["sse", "PriceStreamOptions", "maxRetryDelayMs"],
  ]) {
    const filename = path.join(declarations, `${file}.d.ts`);
    const source = ts.createSourceFile(
      filename,
      fs.readFileSync(filename, "utf8"),
      ts.ScriptTarget.Latest,
      true,
    );
    const declaration = source.statements.find((node) => node.name?.getText(source) === symbol);
    const target = member
      ? declaration?.members?.find((node) => node.name?.getText(source) === member)
      : declaration;
    if (!target?.jsDoc?.some((doc) => doc.comment?.length))
      throw new Error(
        `Packed declaration lost public documentation: ${file}.${symbol}${member ? `.${member}` : ""}`,
      );
  }
  const run = (script) =>
    execFileSync(process.execPath, ["--input-type=module", "-e", script], {
      cwd: temporary,
      stdio: "pipe",
    });
  run(
    'import {PublicClient,parseUnits} from "@agara/sdk"; import {decodeFrame} from "@agara/sdk/streaming"; import "@agara/sdk/types"; if(parseUnits("1")!==1000000n || !(new PublicClient()))throw Error("bad core export"); if(decodeFrame({op:"future"}).op!=="unknown")throw Error("bad stream export");',
  );
  if (fs.existsSync(path.join(temporary, "node_modules", "viem")))
    throw Error("REST-only install unexpectedly pulled viem");
  execFileSync("npm", ["install", "--prefer-offline", "--ignore-scripts", "viem@2.56.7"], {
    cwd: temporary,
    stdio: "pipe",
  });
  run(
    'import {hashBatch} from "@agara/sdk/signing"; if(typeof hashBatch!=="function")throw Error("bad signing export");',
  );
  console.log(
    "Packed public declaration docs are retained; ESM core/types/streaming work without viem; optional signing imports successfully.",
  );
} finally {
  fs.rmSync(temporary, { recursive: true, force: true });
}

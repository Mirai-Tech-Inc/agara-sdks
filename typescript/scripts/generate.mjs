import fs from "node:fs/promises";
import openapiTS, { astToString } from "openapi-typescript";
import ts from "typescript";

for (const name of ["trading", "catalogue"]) {
  const schema = JSON.parse(
    await fs.readFile(new URL(`../contracts/${name}.json`, import.meta.url), "utf8"),
  );
  const ast = await openapiTS(schema, {
    transform(node) {
      if (node.type === "integer" && ["int64", "uint64"].includes(node.format)) {
        return ts.factory.createUnionTypeNode([
          ts.factory.createKeywordTypeNode(ts.SyntaxKind.NumberKeyword),
          ts.factory.createKeywordTypeNode(ts.SyntaxKind.BigIntKeyword),
        ]);
      }
    },
  });
  await fs.writeFile(new URL(`../src/generated/${name}.ts`, import.meta.url), astToString(ast));
}

const codes = JSON.parse(
  await fs.readFile(new URL("../contracts/problem-codes.json", import.meta.url), "utf8"),
).codes;
const details = JSON.parse(
  await fs.readFile(new URL("../contracts/problem-details.json", import.meta.url), "utf8"),
);
await fs.writeFile(
  new URL("../src/generated/problems.ts", import.meta.url),
  "export const problemRegistry = " +
    JSON.stringify(
      Object.fromEntries(
        codes.map((code) => [code.code, { ...code, public_detail: details[code.code] }]),
      ),
    ) +
    " as const;\n",
);

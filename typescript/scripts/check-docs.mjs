import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { TSDocParser } from "@microsoft/tsdoc";
import ts from "typescript";

const sourceExtensions = [".ts", ".tsx", ".mts", ".cts"];
const parser = new TSDocParser();

function exportTargets(value) {
  if (typeof value === "string") return [value];
  if (!value || typeof value !== "object") return [];
  return Object.values(value).flatMap(exportTargets);
}

function entrypoints(root, manifest, config) {
  const exports = manifest.exports;
  if (!exports) throw new Error("package.json must declare the public exports to check");
  const entries =
    typeof exports === "object" && Object.keys(exports).some((key) => key.startsWith("."))
      ? Object.entries(exports)
      : [[".", exports]];
  const files = new Set();
  for (const [subpath, value] of entries) {
    const targets = exportTargets(value).filter((target) => /\.[cm]?[jt]sx?$/.test(target));
    if (!targets.length) continue;
    if (subpath.includes("*"))
      throw new Error(`Expand wildcard export ${subpath} for docs checking`);
    const name = `${manifest.name}${subpath === "." ? "" : subpath.slice(1)}`;
    const mapped = config.options.paths?.[name] ?? [];
    const candidates = mapped.map((target) => path.resolve(config.options.baseUrl ?? root, target));
    for (const target of targets) {
      const absolute = path.resolve(root, target);
      candidates.push(absolute);
      const relative = path.relative(config.options.outDir ?? path.join(root, "dist"), absolute);
      if (relative.startsWith("..")) continue;
      const stem = relative.replace(/(?:\.d)?\.[cm]?[jt]sx?$/, "");
      for (const extension of sourceExtensions)
        candidates.push(
          path.resolve(config.options.rootDir ?? path.join(root, "src"), stem + extension),
        );
    }
    const source = candidates.find(
      (candidate) => !candidate.endsWith(".d.ts") && config.fileNames.includes(candidate),
    );
    if (!source) throw new Error(`Cannot map package export ${subpath} to a tsconfig source file`);
    files.add(source);
  }
  if (!files.size) throw new Error("No TypeScript public entrypoints were found");
  return [...files];
}

function visibleText(node) {
  if (!node) return "";
  if (node.kind === "PlainText") return node.text;
  if (node.kind === "CodeSpan") return node.code;
  if (node.kind === "EscapedText") return node.decodedText;
  if (node.kind === "LinkTag") return node.linkText ?? "linked reference";
  return node.getChildNodes().map(visibleText).join(" ");
}

function substantive(node) {
  const text = visibleText(node).trim();
  return (text.match(/[\p{L}\p{N}]+/gu)?.length ?? 0) >= 2 && !/^(?:todo|tbd|fixme)\b/i.test(text);
}

function isHidden(node) {
  return (
    (ts.getCombinedModifierFlags(node) &
      (ts.ModifierFlags.Private | ts.ModifierFlags.Protected)) !==
      0 ||
    (node.name && ts.isPrivateIdentifier(node.name))
  );
}

function callable(node) {
  if (ts.isFunctionLike(node)) return node;
  if (node.type && ts.isFunctionTypeNode(node.type)) return node.type;
  if (
    node.initializer &&
    (ts.isArrowFunction(node.initializer) || ts.isFunctionExpression(node.initializer))
  )
    return node.initializer;
  return undefined;
}

function commentRange(node) {
  let owner = node;
  if (ts.isVariableDeclaration(node)) owner = node.parent.parent;
  const docs = owner.jsDoc;
  return docs?.length ? docs[docs.length - 1] : undefined;
}

function nodeName(node) {
  if (ts.isConstructorDeclaration(node)) return "constructor";
  if (ts.isCallSignatureDeclaration(node)) return "call signature";
  if (ts.isConstructSignatureDeclaration(node)) return "construct signature";
  if (ts.isIndexSignatureDeclaration(node)) return "index signature";
  return node.name?.getText() ?? ts.SyntaxKind[node.kind];
}

function checkProject(root) {
  const manifest = JSON.parse(fs.readFileSync(path.join(root, "package.json"), "utf8"));
  const configFile = ts.readConfigFile(path.join(root, "tsconfig.json"), ts.sys.readFile);
  if (configFile.error)
    throw new Error(ts.flattenDiagnosticMessageText(configFile.error.messageText, "\n"));
  const config = ts.parseJsonConfigFileContent(configFile.config, ts.sys, root);
  if (config.errors.length)
    throw new Error(
      config.errors
        .map((error) => ts.flattenDiagnosticMessageText(error.messageText, "\n"))
        .join("\n"),
    );
  const entries = entrypoints(root, manifest, config);
  const program = ts.createProgram(config.fileNames, config.options);
  const checker = program.getTypeChecker();
  const sourceRoot = path.join(root, "src");
  const generatedRoot = path.join(sourceRoot, "generated");
  const diagnostics = [];
  const checked = new Set();
  const visitedTypes = new Set();
  const visitedSymbols = new Set();
  const generatedFiles = new Set();

  function owned(node) {
    const file = node.getSourceFile().fileName;
    if (file.startsWith(`${generatedRoot}${path.sep}`)) {
      generatedFiles.add(file);
      return false;
    }
    return file.startsWith(`${sourceRoot}${path.sep}`) && !file.endsWith(".d.ts");
  }

  function report(node, name, message, position = node.getStart()) {
    const file = node.getSourceFile();
    const { line, character } = file.getLineAndCharacterOfPosition(position);
    diagnostics.push(
      `${path.relative(root, file.fileName)}:${line + 1}:${character + 1}: ${name}: ${message}`,
    );
  }

  function isImplementation(node) {
    if (!node.body) return false;
    const siblings = ts.isConstructorDeclaration(node)
      ? node.parent.members.filter(ts.isConstructorDeclaration)
      : ((node.name && checker.getSymbolAtLocation(node.name)?.declarations) ?? []);
    return siblings.some((sibling) => ts.isFunctionLike(sibling) && !sibling.body);
  }

  function checkDeclaration(node, name) {
    if (!owned(node) || isHidden(node) || checked.has(node) || isImplementation(node)) return;
    checked.add(node);
    const range = commentRange(node);
    if (!range) {
      report(node, name, "missing public TSDoc summary");
      return;
    }
    const context = parser.parseString(node.getSourceFile().text.slice(range.pos, range.end));
    for (const message of context.log.messages)
      report(
        node,
        name,
        `${message.messageId}: ${message.unformattedText}`,
        range.pos + message.textRange.pos,
      );
    const doc = context.docComment;
    if (!substantive(doc.summarySection))
      report(node, name, "write a substantive public TSDoc summary");
    const signature = callable(node);
    const parameters = new Set(
      (signature?.parameters ?? [])
        .filter((parameter) => ts.isIdentifier(parameter.name) && parameter.name.text !== "this")
        .map((parameter) => parameter.name.text),
    );
    const documented = new Set();
    for (const block of doc.params.blocks) {
      if (!parameters.has(block.parameterName))
        report(
          node,
          name,
          `@param ${block.parameterName} does not name a parameter of this declaration`,
        );
      if (documented.has(block.parameterName))
        report(node, name, `duplicate @param ${block.parameterName}`);
      if (!substantive(block.content))
        report(node, name, `@param ${block.parameterName} needs a meaningful description`);
      documented.add(block.parameterName);
    }
    if (signature && !ts.isFunctionTypeNode(signature))
      for (const parameter of parameters)
        if (!documented.has(parameter))
          report(node, name, `missing @param ${parameter} - description`);
    const typeParameters = new Set(
      (node.typeParameters ?? signature?.typeParameters ?? []).map(
        (parameter) => parameter.name.text,
      ),
    );
    for (const block of doc.typeParams.blocks) {
      if (!typeParameters.has(block.parameterName))
        report(
          node,
          name,
          `@typeParam ${block.parameterName} does not name a type parameter of this declaration`,
        );
      if (!substantive(block.content))
        report(node, name, `@typeParam ${block.parameterName} needs a meaningful description`);
    }
  }

  function visitSignature(signature, name) {
    const declaration = signature.getDeclaration();
    if (!declaration || !owned(declaration) || isHidden(declaration)) return;
    for (const parameter of signature.getParameters()) {
      const location = parameter.valueDeclaration ?? parameter.declarations?.[0];
      if (location)
        visitType(
          checker.getTypeOfSymbolAtLocation(parameter, location),
          `${name}.${parameter.name}`,
        );
    }
    visitType(signature.getReturnType(), `${name} result`);
  }

  function visitType(type, name) {
    if (visitedTypes.has(type)) return;
    visitedTypes.add(type);
    if (type.isUnionOrIntersection()) {
      for (const member of type.types) visitType(member, name);
      return;
    }
    if (!(type.flags & ts.TypeFlags.Object)) return;
    if (type.objectFlags & ts.ObjectFlags.Reference)
      for (const argument of checker.getTypeArguments(type)) visitType(argument, name);
    for (const property of type.getProperties()) {
      for (const declaration of property.declarations ?? []) {
        if (!owned(declaration) || isHidden(declaration)) continue;
        if (
          !ts.isPropertySignature(declaration) &&
          !ts.isPropertyDeclaration(declaration) &&
          !ts.isMethodSignature(declaration) &&
          !ts.isMethodDeclaration(declaration) &&
          !ts.isGetAccessorDeclaration(declaration) &&
          !ts.isSetAccessorDeclaration(declaration) &&
          !ts.isParameter(declaration)
        )
          continue;
        const memberName = `${name}.${nodeName(declaration)}`;
        if (ts.isParameter(declaration)) {
          if (ts.isConstructorDeclaration(declaration.parent))
            checkDeclaration(declaration.parent, `${name}.constructor`);
        } else checkDeclaration(declaration, memberName);
        visitType(checker.getTypeOfSymbolAtLocation(property, declaration), memberName);
      }
    }
    for (const kind of [ts.SignatureKind.Call, ts.SignatureKind.Construct]) {
      for (const signature of checker.getSignaturesOfType(type, kind)) {
        const declaration = signature.getDeclaration();
        if (
          declaration &&
          (ts.isCallSignatureDeclaration(declaration) ||
            ts.isConstructSignatureDeclaration(declaration) ||
            ts.isConstructorDeclaration(declaration))
        )
          checkDeclaration(declaration, `${name}.${nodeName(declaration)}`);
        visitSignature(signature, name);
      }
    }
    for (const index of checker.getIndexInfosOfType(type)) {
      if (index.declaration && owned(index.declaration)) {
        checkDeclaration(index.declaration, `${name}.index signature`);
      }
      visitType(index.type, name);
    }
  }

  function visitExport(symbol) {
    if (symbol.flags & ts.SymbolFlags.Alias) symbol = checker.getAliasedSymbol(symbol);
    if (visitedSymbols.has(symbol)) return;
    visitedSymbols.add(symbol);
    const declarations = symbol.declarations ?? [];
    for (const declaration of declarations) {
      if (!owned(declaration)) continue;
      if (ts.isSourceFile(declaration) || ts.isModuleDeclaration(declaration)) {
        for (const member of checker.getExportsOfModule(symbol)) visitExport(member);
        continue;
      }
      checkDeclaration(declaration, symbol.name);
      visitType(checker.getTypeAtLocation(declaration), symbol.name);
      if (ts.isClassDeclaration(declaration)) {
        visitType(checker.getTypeOfSymbolAtLocation(symbol, declaration), symbol.name);
        for (const member of declaration.members) {
          if (isHidden(member) || ts.isClassStaticBlockDeclaration(member)) continue;
          checkDeclaration(member, `${symbol.name}.${nodeName(member)}`);
        }
      }
    }
  }

  for (const entry of entries) {
    const source = program.getSourceFile(entry);
    const symbol = source && checker.getSymbolAtLocation(source);
    if (!symbol) throw new Error(`Cannot resolve exports of ${path.relative(root, entry)}`);
    for (const exported of checker.getExportsOfModule(symbol)) visitExport(exported);
  }
  for (const diagnostic of diagnostics) console.error(diagnostic);
  console.log(
    `Public TSDoc: ${checked.size} declarations checked across ${entries.length} entrypoints; ${diagnostics.length} errors. Generated OpenAPI declarations in src/generated/ are exempt (${generatedFiles.size} files reached).`,
  );
  return diagnostics.length ? 1 : 0;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    if (process.argv.length > 3)
      throw new Error("Usage: node scripts/check-docs.mjs [project-directory]");
    process.exitCode = checkProject(path.resolve(process.argv[2] ?? process.cwd()));
  } catch (error) {
    console.error(`Public TSDoc check failed: ${error.message}`);
    process.exitCode = 1;
  }
}

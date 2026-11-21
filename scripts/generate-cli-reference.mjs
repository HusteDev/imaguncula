// SPDX-License-Identifier: MIT OR Apache-2.0
import { spawnSync } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(fileURLToPath(new URL("..", import.meta.url)));
const destination = path.join(root, "docs", "cli-reference.md");
const check = process.argv.includes("--check");

const sections = [
  ["Top-level help", ["--help"]],
  ["Saved-project export", ["export", "--help"]],
  ["One-shot preset export", ["quick", "--help"]],
];

const rendered = sections
  .map(([heading, args]) => {
    const result = spawnSync(
      "cargo",
      ["run", "--quiet", "--locked", "-p", "imaguncula-cli", "--", ...args],
      { cwd: root, encoding: "utf8" },
    );
    if (result.error) throw result.error;
    if (result.status !== 0) {
      throw new Error(`CLI help generation failed for ${args.join(" ")}: ${result.stderr}`);
    }
    const portableHelp = result.stdout.replaceAll("imaguncula.exe", "imaguncula").trimEnd();
    return `## ${heading}\n\n\`\`\`text\n${portableHelp}\n\`\`\``;
  })
  .join("\n\n");

const document = `# CLI reference

This file is generated from the compiled CLI help. Do not edit it manually. Regenerate it with \`node scripts/generate-cli-reference.mjs\`.

${rendered}
`;

if (check) {
  const current = readFileSync(destination, "utf8");
  if (current !== document) {
    process.stderr.write("docs/cli-reference.md is stale. Regenerate it.\n");
    process.exitCode = 1;
  }
} else {
  writeFileSync(destination, document, "utf8");
}

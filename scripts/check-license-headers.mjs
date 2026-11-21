// SPDX-License-Identifier: MIT OR Apache-2.0
import { readdirSync, readFileSync, statSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(fileURLToPath(new URL("..", import.meta.url)));
const roots = ["crates", "e2e", "fuzz/fuzz_targets", "scripts", "src", "src-tauri"];
const extensions = new Set([".js", ".mjs", ".rs", ".svelte", ".ts"]);
const failures = [];

for (const relativeRoot of roots) visit(path.join(root, relativeRoot));

if (failures.length > 0) {
  process.stderr.write(
    `Missing SPDX-License-Identifier: MIT OR Apache-2.0 header:\n${failures.join("\n")}\n`,
  );
  process.exitCode = 1;
}

function visit(entry) {
  if (!statSync(entry).isDirectory()) {
    if (!extensions.has(path.extname(entry))) return;
    const firstLine = readFileSync(entry, "utf8").split(/\r?\n/, 1)[0];
    if (!firstLine.includes("SPDX-License-Identifier: MIT OR Apache-2.0")) {
      failures.push(path.relative(root, entry).replaceAll("\\", "/"));
    }
    return;
  }

  for (const child of readdirSync(entry, { withFileTypes: true })) {
    if (child.name === "target") continue;
    visit(path.join(entry, child.name));
  }
}

// SPDX-License-Identifier: MIT OR Apache-2.0
import { mkdirSync, writeFileSync } from "node:fs";
import path from "node:path";

const values = Object.fromEntries(
  process.argv.slice(2).map((argument) => {
    const [key, ...rest] = argument.split("=");
    return [key.replace(/^--/, ""), rest.join("=")];
  }),
);

for (const required of ["version", "sha256", "url", "output"]) {
  if (!values[required]) throw new Error(`Missing --${required}=...`);
}
if (!/^\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?$/.test(values.version)) {
  throw new Error("Version is not a supported SemVer value.");
}
if (!/^[0-9a-f]{64}$/.test(values.sha256)) throw new Error("SHA-256 must be lowercase hex.");
if (!values.url.startsWith("https://")) throw new Error("Formula URL must use HTTPS.");

const formula = `class Imaguncula < Formula
  desc "Offline icon asset generator CLI"
  homepage "https://github.com/HusteDev/imaguncula"
  url "${values.url}"
  sha256 "${values.sha256}"
  license any_of: ["MIT", "Apache-2.0"]
  version "${values.version}"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args(path: "crates/imaguncula-cli")
  end

  test do
    assert_match "Imaguncula command-line interface", shell_output("#{bin}/imaguncula --help")
  end
end
`;

mkdirSync(path.dirname(values.output), { recursive: true });
writeFileSync(values.output, formula, "utf8");

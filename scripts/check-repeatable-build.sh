#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Builds the frontend, the CLI, and the desktop executable twice from the same
# source, each time into a fresh Cargo target directory, and fails unless both
# builds are byte-identical. Hash lists are written to target/repeatable-build.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"
out="$root/target/repeatable-build"
rm -rf "$out"
mkdir -p "$out"

exe=""
case "$(uname -s)" in
  MINGW* | MSYS* | CYGWIN*) exe=".exe" ;;
esac

if command -v sha256sum > /dev/null; then
  hash() { sha256sum "$@"; }
else
  hash() { shasum -a 256 "$@"; }
fi

for run in first second; do
  echo "::group::Build ${run}"
  export CARGO_TARGET_DIR="$root/target/repeatable-$run"
  rm -rf "$CARGO_TARGET_DIR" build
  cargo build --locked --release -p imaguncula-cli
  # Runs `pnpm build` for the frontend, then builds the desktop executable.
  pnpm tauri build --no-bundle
  {
    (cd build && find . -type f | LC_ALL=C sort | while read -r file; do hash "$file"; done) |
      sed 's#^#frontend #'
    (cd "$CARGO_TARGET_DIR/release" && hash "imaguncula$exe" "imaguncula-desktop$exe")
  } | tr -d '*' > "$out/$run.sha256"
  echo "::endgroup::"
done

cat "$out/first.sha256"
if diff -u "$out/first.sha256" "$out/second.sha256"; then
  echo "Both builds are byte-identical."
else
  echo "The two builds differ; see the diff above." >&2
  exit 1
fi

#!/usr/bin/env bash
set -u

skip_smoke=false
for argument in "$@"; do
  case "$argument" in
    --skip-smoke) skip_smoke=true ;;
    *) printf 'unknown argument: %s\n' "$argument" >&2; exit 2 ;;
  esac
done

script_dir=$(CDPATH= cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
repo_root=$(CDPATH= cd -- "$script_dir/.." && pwd)
failures=0
warnings=0

fail() {
  failures=$((failures + 1))
  printf 'FAIL: %s\n' "$1" >&2
}

warn() {
  warnings=$((warnings + 1))
  printf 'WARN: %s\n' "$1" >&2
}

require_command() {
  if command -v "$1" >/dev/null 2>&1; then
    return 0
  fi
  fail "$1 is missing. Run scripts/bootstrap.sh."
  return 1
}

run_check() {
  label=$1
  shift
  printf 'check: %s\n' "$label"
  if "$@"; then
    return 0
  else
    status=$?
  fi
  fail "$label failed with exit code $status. Re-run the command above after correcting its first error."
  return 0
}

expected_node=$(tr -d '[:space:]' < "$repo_root/.node-version")
expected_just=$(tr -d '[:space:]' < "$repo_root/.just-version")
expected_pnpm=$(sed -n 's/.*"packageManager"[[:space:]]*:[[:space:]]*"pnpm@\([^"]*\)".*/\1/p' "$repo_root/package.json")
expected_rust=$(sed -n 's/^[[:space:]]*channel[[:space:]]*=[[:space:]]*"\([^"]*\)".*/\1/p' "$repo_root/rust-toolchain.toml")

cd "$repo_root" || exit 1
printf 'Imaguncula doctor\n'
printf 'repository: %s\n' "$repo_root"

has_node=false
has_pnpm=false
has_rustc=false
has_cargo=false
has_rustup=false
has_just=false
require_command node && has_node=true
require_command pnpm && has_pnpm=true
require_command rustc && has_rustc=true
require_command cargo && has_cargo=true
require_command rustup && has_rustup=true
require_command just && has_just=true

if $has_node; then
  actual_node=$(node --version | sed 's/^v//')
  if [ "$actual_node" = "$expected_node" ]; then
    printf 'ok: Node.js %s\n' "$actual_node"
  else
    fail "Node.js $actual_node is active; expected $expected_node from .node-version."
  fi
fi

if $has_pnpm; then
  actual_pnpm=$(pnpm --version)
  if [ "$actual_pnpm" = "$expected_pnpm" ]; then
    printf 'ok: pnpm %s\n' "$actual_pnpm"
  else
    fail "pnpm $actual_pnpm is active; expected $expected_pnpm from package.json."
  fi
fi

if $has_rustc; then
  actual_rust=$(rustc --version | awk '{print $2}')
  if [ "$actual_rust" = "$expected_rust" ]; then
    printf 'ok: Rust %s\n' "$actual_rust"
  else
    fail "Rust $actual_rust is active; expected $expected_rust from rust-toolchain.toml."
  fi
fi

if $has_cargo; then
  printf 'ok: %s\n' "$(cargo --version)"
fi

if $has_rustup; then
  installed_components=$(rustup component list --toolchain "$expected_rust" --installed 2>/dev/null || true)
  for component in clippy rustfmt; do
    if printf '%s\n' "$installed_components" | grep -q "^$component-"; then
      printf 'ok: Rust component %s\n' "$component"
    else
      fail "Rust component $component is missing for toolchain $expected_rust."
    fi
  done
fi

if $has_just; then
  actual_just=$(just --version | awk '{print $NF}')
  if [ "$actual_just" = "$expected_just" ]; then
    printf 'ok: just %s\n' "$actual_just"
  else
    fail "just $actual_just is active; expected $expected_just from .just-version."
  fi
fi

case "$(uname -s)" in
  Darwin)
    if xcode-select -p >/dev/null 2>&1; then
      printf 'ok: Xcode Command Line Tools\n'
    else
      fail 'Xcode Command Line Tools are missing. Run scripts/bootstrap.sh.'
    fi
    ;;
  Linux)
    if require_command pkg-config; then
      for module in webkit2gtk-4.1 ayatana-appindicator3-0.1 librsvg-2.0; do
        if pkg-config --exists "$module"; then
          printf 'ok: native module %s\n' "$module"
        else
          fail "native module $module is missing. Run scripts/bootstrap.sh on a supported Debian/Ubuntu host."
        fi
      done
    fi
    ;;
  *)
    fail 'doctor.sh supports macOS and Linux; use doctor.ps1 on Windows.'
    ;;
esac

if ! $skip_smoke && [ "$failures" -eq 0 ]; then
  run_check 'locked frontend dependency install' pnpm install --frozen-lockfile
  run_check 'Rust workspace check' cargo check --workspace --all-targets
  run_check 'Svelte and TypeScript check' pnpm check
elif $skip_smoke; then
  printf 'skip: dependency and compile smoke checks\n'
fi

if [ "$failures" -gt 0 ]; then
  printf 'doctor failed: %s blocking issue(s), %s warning(s)\n' "$failures" "$warnings" >&2
  exit 1
fi

printf 'doctor passed: %s warning(s)\n' "$warnings"

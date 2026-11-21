set dotenv-load := true
set windows-shell := ["powershell.exe", "-NoLogo", "-NoProfile", "-Command"]

pnpm := if os_family() == "windows" { "pnpm.cmd" } else { "pnpm" }

default:
  @just --list

setup:
  {{pnpm}} install --frozen-lockfile

dev:
  {{pnpm}} tauri dev

run: dev

check:
  {{pnpm}} check
  cargo check --workspace --all-targets

test:
  {{pnpm}} test
  cargo test --workspace

lint:
  {{pnpm}} format:check
  {{pnpm}} lint
  {{pnpm}} check
  node scripts/check-license-headers.mjs
  node scripts/generate-cli-reference.mjs --check
  cargo fmt --all -- --check
  cargo fmt --manifest-path fuzz/Cargo.toml -- --check
  cargo check --manifest-path fuzz/Cargo.toml --all-targets --locked
  cargo clippy --workspace --all-targets -- -D warnings

build:
  {{pnpm}} tauri build --debug --no-bundle

ci: lint test build

[windows]
doctor:
  powershell -NoProfile -ExecutionPolicy Bypass -File scripts/doctor.ps1

[unix]
doctor:
  bash scripts/doctor.sh

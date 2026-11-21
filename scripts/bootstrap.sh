#!/usr/bin/env bash
set -eu

dry_run=false
assume_yes=false
for argument in "$@"; do
  case "$argument" in
    --dry-run) dry_run=true ;;
    --yes) assume_yes=true ;;
    *) printf 'unknown argument: %s\n' "$argument" >&2; exit 2 ;;
  esac
done

script_dir=$(CDPATH= cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
repo_root=$(CDPATH= cd -- "$script_dir/.." && pwd)
expected_node=$(tr -d '[:space:]' < "$repo_root/.node-version")
expected_just=$(tr -d '[:space:]' < "$repo_root/.just-version")
expected_pnpm=$(sed -n 's/.*"packageManager"[[:space:]]*:[[:space:]]*"pnpm@\([^"]*\)".*/\1/p' "$repo_root/package.json")
expected_rust=$(sed -n 's/^[[:space:]]*channel[[:space:]]*=[[:space:]]*"\([^"]*\)".*/\1/p' "$repo_root/rust-toolchain.toml")
nvm_version=0.40.6

run_step() {
  description=$1
  shift
  printf 'step: %s\n' "$description"
  printf '  '
  printf '%q ' "$@"
  printf '\n'
  if $dry_run; then
    return 0
  fi
  if "$@"; then
    return 0
  else
    status=$?
  fi
  printf '%s failed with exit code %s. Correct the reported error, then re-run this script.\n' "$description" "$status" >&2
  exit "$status"
}

as_root() {
  if [ "$(id -u)" -eq 0 ]; then
    "$@"
  elif command -v sudo >/dev/null 2>&1; then
    sudo "$@"
  else
    printf 'sudo is required to install missing Linux system packages.\n' >&2
    return 1
  fi
}

command_version_is() {
  command_name=$1
  expected=$2
  shift 2
  command -v "$command_name" >/dev/null 2>&1 || return 1
  actual=$("$@")
  [ "$actual" = "$expected" ]
}

platform=$(uname -s)
case "$platform" in
  Darwin|Linux) ;;
  *) printf 'bootstrap.sh supports macOS and Linux; use bootstrap.ps1 on Windows.\n' >&2; exit 1 ;;
esac

cd "$repo_root"
printf 'Imaguncula macOS/Linux bootstrap\n'
printf 'repository: %s\n' "$repo_root"
printf 'platform: %s\n' "$platform"
printf 'mode: %s\n' "$(if $dry_run; then printf 'dry run'; elif $assume_yes; then printf 'unattended'; else printf 'interactive'; fi)"
printf 'pinned tools: Node %s; pnpm %s; Rust %s; just %s\n' "$expected_node" "$expected_pnpm" "$expected_rust" "$expected_just"
printf 'system changes may install compiler and Tauri desktop libraries through the host package manager.\n'
printf 'user changes may install nvm/Node, Rust, pnpm, and just under the current user profile.\n'

if $dry_run; then
  printf 'dry-run plan:\n'
  printf '  install missing native prerequisites for %s\n' "$platform"
  printf '  install or select the pinned Node, pnpm, Rust, and just versions\n'
  printf '  run scripts/doctor.sh, including locked install and compile checks\n'
  printf 'dry run complete; no commands were executed\n'
  exit 0
fi

if ! $assume_yes; then
  printf 'Continue with the listed installations? [y/N] '
  read -r answer
  case "$answer" in
    y|Y|yes|YES|Yes) ;;
    *) printf 'bootstrap cancelled\n'; exit 2 ;;
  esac
fi

if [ "$platform" = "Darwin" ]; then
  if xcode-select -p >/dev/null 2>&1; then
    printf 'skip: Xcode Command Line Tools are installed\n'
  else
    run_step 'start the Xcode Command Line Tools installer' xcode-select --install
    printf 'Complete the Apple installer, then re-run this script. The second run resumes safely.\n' >&2
    exit 2
  fi
else
  if ! command -v apt-get >/dev/null 2>&1 || ! command -v dpkg-query >/dev/null 2>&1; then
    printf 'This bootstrap currently supports Debian and Ubuntu. Use the dev container on other Linux distributions.\n' >&2
    exit 1
  fi

  native_packages='build-essential ca-certificates curl wget file libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev patchelf xdg-utils pkg-config libssl-dev'
  missing_packages=''
  for package_name in $native_packages; do
    if ! dpkg-query -W -f='${Status}' "$package_name" 2>/dev/null | grep -q 'ok installed'; then
      missing_packages="$missing_packages $package_name"
    fi
  done
  if [ -n "$missing_packages" ]; then
    run_step 'refresh apt package metadata' as_root apt-get update
    # Word splitting is intentional: the values come from the fixed package list above.
    run_step 'install compiler and Tauri Linux prerequisites' as_root apt-get install --yes $missing_packages
  else
    printf 'skip: Linux native prerequisites are installed\n'
  fi
fi

export NVM_DIR="${NVM_DIR:-"$HOME/.nvm"}"
if ! command_version_is node "$expected_node" sh -c 'node --version | sed "s/^v//"'; then
  if [ ! -s "$NVM_DIR/nvm.sh" ]; then
    nvm_installer=$(mktemp "${TMPDIR:-/tmp}/imaguncula-nvm.XXXXXX")
    run_step "download pinned nvm $nvm_version installer" curl --proto '=https' --tlsv1.2 --fail --silent --show-error --location "https://raw.githubusercontent.com/nvm-sh/nvm/v$nvm_version/install.sh" --output "$nvm_installer"
    run_step "install nvm $nvm_version for the current user" bash "$nvm_installer"
    rm -f -- "$nvm_installer"
  else
    printf 'skip: nvm is installed\n'
  fi
  # shellcheck source=/dev/null
  . "$NVM_DIR/nvm.sh"
  run_step "install Node.js $expected_node" nvm install "$expected_node"
  run_step "select Node.js $expected_node" nvm use "$expected_node"
  run_step "make Node.js $expected_node the nvm default" nvm alias default "$expected_node"
else
  printf 'skip: Node.js %s is already active\n' "$expected_node"
fi

if ! command_version_is pnpm "$expected_pnpm" pnpm --version; then
  run_step "install pnpm $expected_pnpm for the active Node.js installation" npm install --global "pnpm@$expected_pnpm"
else
  printf 'skip: pnpm %s is already active\n' "$expected_pnpm"
fi

if ! command -v rustup >/dev/null 2>&1; then
  rustup_installer=$(mktemp "${TMPDIR:-/tmp}/imaguncula-rustup.XXXXXX")
  run_step 'download the official rustup installer' curl --proto '=https' --tlsv1.2 --fail --silent --show-error --location https://sh.rustup.rs --output "$rustup_installer"
  run_step 'install rustup for the current user' sh "$rustup_installer" -y --profile minimal --default-toolchain none
  rm -f -- "$rustup_installer"
  # shellcheck source=/dev/null
  . "$HOME/.cargo/env"
else
  printf 'skip: rustup is installed\n'
fi
run_step 'install pinned Rust toolchain and components' rustup toolchain install "$expected_rust" --profile minimal --component clippy --component rustfmt

if ! command_version_is just "$expected_just" sh -c 'just --version | awk "{print \$NF}"'; then
  run_step "install just $expected_just in the Cargo user bin directory" cargo install just --locked --version "$expected_just"
else
  printf 'skip: just %s is already active\n' "$expected_just"
fi

run_step 'run environment and compile verification' "$script_dir/doctor.sh"
printf 'ready: open a new shell if this script installed tools, then run just run\n'

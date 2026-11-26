# Contributing to Imaguncula

Thank you for helping. Imaguncula accepts bug reports, feature ideas, documentation fixes, and code contributions from anyone. Everyone taking part follows the [Code of Conduct](CODE_OF_CONDUCT.md).

## Ways to contribute

- **Report a bug:** open an [issue](https://github.com/HusteDev/imaguncula/issues/new/choose) using the bug report form. See [Reporting bugs](#reporting-bugs) below.
- **Report a vulnerability:** never in a public issue. Follow [SECURITY.md](SECURITY.md) to report it privately.
- **Ask a question or propose an idea:** use [Discussions](https://github.com/HusteDev/imaguncula/discussions). Ideas that gain agreement become issues.
- **Contribute code or documentation:** open a pull request as described in [Submitting a pull request](#submitting-a-pull-request). Issues labeled `good first issue` are a good starting point.

## Reporting bugs

Search [existing issues](https://github.com/HusteDev/imaguncula/issues?q=is%3Aissue) first, then open a new one with the bug report form. A useful report includes:

- the Imaguncula version (from the release file you downloaded, or `imaguncula --version` for the CLI) and how you installed it;
- your operating system and architecture;
- the steps that reproduce the problem, and what you expected instead;
- for wrong output, the `report.md` from the exported package, which lists the source hash and every output without exposing local paths;
- a minimal source file that reproduces the problem, if you can share one.

Use synthetic artwork. Do not attach confidential artwork, credentials, or unrelated personal data. The maintainer aims to acknowledge every bug report within seven days.

## Submitting a pull request

1. Fork the repository and create a branch from `main`.
2. Make your change, following the [coding standards](#coding-standards) and the [testing policy](#testing-policy).
3. Run `just ci` locally (see [Development checks](#development-checks)) and fix every failure.
4. Open a pull request against `main` and fill in the pull request template, including the commands you ran and their results.
5. Respond to review feedback. A pull request is merged once the hosted CI checks pass and the maintainer approves it.

For large or behavior-changing work, open an issue or discussion first so the approach can be agreed before you invest time in it.

### Developer Certificate of Origin

Every commit in a pull request must be signed off to certify the [Developer Certificate of Origin 1.1](https://developercertificate.org/): that you wrote the change or otherwise have the right to submit it under the project's MIT OR Apache-2.0 license. Add the sign-off with `git commit -s`, which appends a line such as:

```text
Signed-off-by: Your Name <you@example.com>
```

Use your real name or an established pseudonym. To sign off commits you already made, run `git rebase --signoff main` and force-push your branch.

## Coding standards

- **Rust:** format with `rustfmt` (`cargo fmt`). Code must pass `cargo clippy --workspace --all-targets -- -D warnings` with the workspace's `clippy::pedantic` lints. `unsafe` code is forbidden across the workspace.
- **TypeScript and Svelte:** format with Prettier (`pnpm format`). Code must pass ESLint with zero warnings and `svelte-check` with no errors.
- **License headers:** add `SPDX-License-Identifier: MIT OR Apache-2.0` to every new authored source file. `just lint` checks this.
- **Dependencies:** new Rust dependencies must pass `cargo deny check advisories bans licenses sources --warn unmaintained`, use exact versions, and come from crates.io. Do not add network-dependent product behavior.
- **Architecture:** keep `imaguncula-core` independent of Tauri and frontend types, render every target directly from the original source, and return typed warnings and errors from processing code. See [docs/architecture.md](docs/architecture.md).

## Testing policy

Every change that adds functionality or fixes a bug must include automated tests that exercise it, in the same pull request:

- new or changed behavior gets unit or integration tests in the affected crate or in `src/lib`;
- a bug fix gets a regression test that fails without the fix;
- changes to output dimensions, filenames, package paths, or input validation always need tests;
- parsers and other code that handles untrusted input should also gain property-test or fuzz coverage where practical (see `fuzz/`).

The test suites run with `just test` (`cargo test --workspace` and `pnpm test`), and the desktop end-to-end and accessibility suite runs with `pnpm test:e2e`. CI runs them on Windows, macOS, and Linux for every pull request. Pull requests that add functionality without tests are not merged.

## Development checks

Install the prerequisites listed in [README.md](README.md), then run:

```powershell
just ci
```

This runs the same lint, test, and build steps as CI. Until `just` is installed, use the equivalent commands in the `justfile`.

Changes to Rust dependencies must also pass:

```powershell
cargo deny check advisories bans licenses sources --warn unmaintained
```

Known maintenance-only RustSec notices remain visible as warnings because their current Tauri/resvg dependency paths have no safe upgrade. Vulnerabilities, disallowed licenses, wildcard dependencies, and untrusted sources fail the gate. The hosted security workflow scans the complete Git history with Gitleaks. Do not commit real credentials and do not use an allowlist to conceal a real leak; rotate the credential and remove it from history before publication.

## Plans and decisions

See [GOVERNANCE.md](GOVERNANCE.md) for how decisions are made and who holds which role, and [ROADMAP.md](ROADMAP.md) for what is planned. Read [PLAN.md](PLAN.md) for product requirements, [EXECUTION-PLAN.md](EXECUTION-PLAN.md) for implementation order, and [docs/adr](docs/adr) for accepted decisions. When implementation changes an accepted decision, update its ADR and the execution plan in the same change. Product changes also require an update to `PLAN.md`.

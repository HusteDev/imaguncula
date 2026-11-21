# Contributing to Imaguncula

Imaguncula is a v1 release candidate. Before changing behavior, read:

- [PLAN.md](PLAN.md) for authoritative product requirements;
- [EXECUTION-PLAN.md](EXECUTION-PLAN.md) for implementation order and phase gates;
- [docs/architecture.md](docs/architecture.md) for dependency boundaries;
- [docs/adr](docs/adr) for accepted decisions.

## Development checks

Install the prerequisites listed in [README.md](README.md), then run:

```powershell
just ci
```

Until `just` is installed, use the equivalent commands in `justfile`.

Changes to Rust dependencies must also pass:

```powershell
cargo deny check advisories bans licenses sources --warn unmaintained
```

Known maintenance-only RustSec notices remain visible as warnings because their current Tauri/resvg dependency paths have no safe upgrade. Vulnerabilities, disallowed licenses, wildcard dependencies, and untrusted sources fail the gate. The hosted security workflow scans the complete Git history with Gitleaks. Do not commit real credentials and do not use an allowlist to conceal a real leak; rotate the credential and remove it from history before publication.

## Required boundaries

- Keep `imaguncula-core` independent of Tauri and frontend types.
- Render every target from the original source, not another resized output.
- Return typed warnings and errors from processing code.
- Do not add network-dependent product behavior.
- Add tests for changes to output dimensions, filenames, package paths, or validation.
- Add `SPDX-License-Identifier: MIT OR Apache-2.0` to new authored source files.

## Reports and conduct

- Follow [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) in every project space.
- Follow [SECURITY.md](SECURITY.md) for private vulnerability reporting. Never disclose a vulnerability in a public issue.
- Use synthetic fixtures. Do not submit confidential source artwork, credentials, or unrelated personal data.

## Plans and decisions

When implementation changes an accepted decision, update its ADR and the execution plan in the same change. Product changes also require an update to `PLAN.md`.

# Changelog

All notable changes are documented here. This project uses Semantic Versioning.

## [Unreleased]

### Changed

- CI measures Rust and frontend test coverage on every push and pull request and uploads the reports, and `just coverage` runs the same measurement locally.
- The desktop IPC adapter has its own test suite, which drives every command through Tauri's mock runtime. Windows builds now embed the application manifest through the linker so those tests can run on Windows.
- The contribution guide now documents how to report bugs, how to submit a pull request, the coding standards, and a testing policy that requires automated tests for new functionality and regression tests for bug fixes.
- The security policy now states the response and fix targets for vulnerability reports and where fixed vulnerabilities are disclosed.
- The repository publishes its OpenSSF Best Practices answers in `.bestpractices.json` and shows the badge in the README.
- New governance, roadmap, and security assurance case documents; the security policy describes step by step how vulnerability reports are handled and credited; the user guide explains upgrades; contributions now carry a Developer Certificate of Origin sign-off.

### Security

- Release artifacts, including the SHA-256 sum files, carry signed GitHub artifact attestations (SLSA build provenance) that can be checked with `gh attestation verify`.
- Updated two Tauri dependencies for RustSec advisories: `url` 2.5.8 to 2.5.9 (RUSTSEC-2026-0592, stack overflow in `Url::origin()` on nested `blob:` URLs) and `cookie` 0.18.1 to 0.18.3 (RUSTSEC-2026-0594, private-cookie nonce reuse after `fork`). The webview loads only bundled content and Imaguncula neither forks nor uses private cookies, so neither issue was reachable.

## [1.0.0-beta.1] - 2026-11-21

First public beta. All desktop and CLI artifacts are unsigned and unnotarized; the release notes state the exact trust boundary.

### Added

- Offline Tauri desktop workflow and display-server-free CLI over one pure Rust processing core.
- Bounded SVG/PNG import, exact-size preview, per-output source replacement, and explicit raster-upscale authorization.
- Versioned Windows desktop/MSIX, Web/PWA, Linux hicolor, Apple, Android, and custom-output presets, including the immutable `windows-desktop` v2 preset with a 42×42 ICO compatibility layer for observed Windows taskbar selection at 175% display scaling and an explicit reviewed migration from v1.
- Verified PNG, SVG, ICO, ICNS, metadata, directory-tree, Android resource, Xcode catalog, report, and deterministic ZIP packaging.
- Transactional export with cancellation, overwrite confirmation, staging verification, and atomic commit.
- Schema-v2 portable project files with exact preset pinning and reviewed preset migration fingerprints.
- Hostile-input corpus, property tests, four fuzz targets, stress limits, and staging-escape regression coverage.
- Three checked-in example projects, generated CLI reference, user/security/accessibility/distribution documentation, and release automation.
- Brand artwork: a mosaic app icon for the window, taskbar, and installers, the "I" wordmark in the app header, and a short note explaining the name.

### Changed

- Output-table previews preserve exact-size or integer-multiple display scaling for small raster assets instead of stretching every image to 64×64 CSS pixels. Missing sources and stalled desktop preview requests resolve to explicit error states instead of displaying `Rendering…` indefinitely.
- `report.md` identifies each source by file name and SHA-256 content hash instead of its absolute local path, so shared packages do not disclose the exporting machine's directory layout.

### Security

- Active SVG content and external or embedded resources fail closed before rendering.
- Source bytes, decoded pixels, dimensions, memory, SVG elements/nesting/filter-elements/path-geometry, project size, preview dimensions, and preview cache are bounded, and SVG rendering runs under a 30-second watchdog without holding the shared preview cache lock.
- All package paths use one cross-platform sanitizer and symlinks are rejected during staged verification.
- A confirmed overwrite only replaces a destination that contains a previous package's `report.md`; any other non-empty directory is refused. When an overwrite commit fails and the previous package cannot be restored, the staging directory holding it is preserved and named in the error instead of being deleted.
- Opened project files are treated as untrusted: sources resolving outside the project's own directory are listed and confirmed before anything is read, the CLI refuses them without `--allow-external-sources`, and pre-accepted warnings and upscale authorizations are announced on open.
- Release and distribution workflows pass workflow inputs and secrets through environment variables instead of template expansion, pin WingetCreate to a hash-verified release, pin third-party actions in token-bearing jobs to commit SHAs, and receive Dependabot updates for actions.
- Upgraded the `zip` dependency to 9.8.1 for RUSTSEC-2026-0548; the vulnerable stream-extraction API was never used, and archive verification fails closed on undecodable entry names.

[1.0.0-beta.1]: https://github.com/HusteDev/imaguncula/releases/tag/v1.0.0-beta.1

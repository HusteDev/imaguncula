# Changelog

All notable changes are documented here. This project uses Semantic Versioning.

## [Unreleased]

## [1.0.0-beta.1] - 2026-11-20

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

[1.0.0-beta.1]: https://github.com/HusteDev/imaguncula/releases/tag/v1.0.0-beta.1

# Determinism contract

Status: verified locally for the v1 processing core

Imaguncula treats reproducibility as a processing invariant. Re-exporting the same project with the same source bytes, pinned preset versions, enabled outputs, accepted warnings, and Imaguncula version must produce byte-identical package trees.

The contract covers:

- standalone PNG and sanitized SVG files;
- ICO and ICNS entry order, encoding, dimensions, and bytes;
- generated JSON, XML, HTML, text, and `report.md`;
- deterministic combined ZIP entry order, timestamps, attributes, and contents;
- the complete relative-path-to-byte mapping of the committed export directory.

The core integration suite exports the same project twice, compares both ZIP files, recursively snapshots both directory trees, and compares every relative path and byte. Independent tests cover repeated render/resize/PNG operations and semantic container reopen checks.

## Boundaries

Determinism is scoped to one supported Imaguncula version and identical inputs. The contract does not claim byte stability across:

- a new Imaguncula release or dependency upgrade;
- a preset migration explicitly accepted by the user;
- changed source paths, because reports record source provenance;
- filesystem metadata outside the package bytes, including ownership and host timestamps;
- Tauri MSI, NSIS, AppImage, or DMG bundles produced by external platform toolchains.

Vendor packaging can include toolchain-controlled metadata. Release artifacts therefore carry SHA-256 sums and are tested for clean-runner installation and launch, but are not part of the core byte-determinism contract.

# Security and resource limits

Status: verified locally; bounded fuzz workflow requires hosted nightly execution

All untrusted source and project inputs cross fail-closed limits in `imaguncula-core` before expensive decoding, rendering, or filesystem writes.

| Boundary | v1 limit |
| --- | ---: |
| Project file | 1 MiB |
| SVG or PNG source file | 64 MiB |
| SVG elements | 100,000 |
| SVG nesting depth | 256 |
| SVG filter elements and primitives | 64 |
| SVG path and polygon geometry bytes | 2 MiB |
| SVG render wall-clock time | 30 s |
| PNG dimension per axis | 16,384 px |
| PNG decoded pixels | 64,000,000 |
| PNG decoder allocation | 256 MiB |
| Preview dimension per axis | 4,096 px |
| Preview cache entries | 64 |
| Preview cache encoded bytes | 64 MiB |

The SVG pre-scan rejects scripts, event handlers, external resource references, embedded image data, foreign objects, excessive nesting, excessive elements, excessive filter elements, excessive path geometry, malformed XML, and CSS resource URLs. The renderer has no image resolver and no system-font dependency, runs on a watchdog-bounded worker thread, and never blocks the preview cache while rendering.

Project files are portable and therefore untrusted. Sources reached through absolute paths, drive prefixes, or `..` segments are reported as external references at load time: the desktop interface asks before importing them, and the CLI refuses to export them without `--allow-external-sources`. Pre-accepted warnings and pre-authorized upscales carried by a project file are announced when it opens.

A non-empty export destination is replaced only when it contains a previous package marker (`report.md`); any other non-empty directory is never deleted. When an overwrite commit fails and the previous package cannot be restored, the staging directory holding it is preserved and its path is reported instead of being cleaned up.

Every package-relative path passes the same platform-independent sanitizer. Drive-qualified, UNC, absolute, empty-segment, dot-segment, parent-segment, reserved-name, separator-confused, and NUL-bearing paths fail before staging. Staging is created beside the selected destination, refuses symlinks, verifies exact membership, and commits only after all semantic checks pass. A dedicated traversal test verifies that an escape-shaped destination creates neither a staging directory nor an external file.

`fixtures/hostile/` is the regression corpus. Property tests exercise accepted and rejected path shapes. Four `cargo-fuzz` targets cover SVG preflight, relative paths, ICO encoder inputs, and ICNS encoder inputs. The bounded hosted workflow runs them under nightly Rust; crashes belong in the corpus before a fix is merged.

See [SECURITY.md](../SECURITY.md) for private reporting and [determinism.md](determinism.md) for reproducibility boundaries.

# Security and resource limits

`imaguncula-core` checks every source file and project file against the limits below before it decodes, renders, or writes anything. Input over a limit is rejected, not truncated or repaired. The values are constants in `crates/imaguncula-core/src` (`source.rs`, `project.rs`, `render.rs`, and `preview.rs`).

| Boundary | Limit |
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

The SVG pre-scan rejects scripts, event handlers, external resource references, embedded image data, foreign objects, malformed XML, CSS resource URLs, and files that go over the element, nesting, filter, or path-geometry limits. The renderer has no image resolver and uses no system fonts. It runs on a worker thread that is abandoned after 30 seconds, and the preview cache is not locked while it renders.

Project files can be shared, so they are treated as untrusted. Sources reached through absolute paths, drive prefixes, or `..` segments are reported as external references at load time: the desktop interface asks before importing them, and the CLI refuses to export them without `--allow-external-sources`. Pre-accepted warnings and pre-authorized upscales carried by a project file are announced when it opens.

A non-empty export destination is replaced only when it contains a previous package marker (`report.md`); any other non-empty directory is never deleted. When an overwrite commit fails and the previous package cannot be restored, the staging directory holding it is preserved and its path is reported instead of being cleaned up.

Every package-relative path goes through the same sanitizer on every operating system. Absolute, drive-qualified, and UNC paths, empty, `.`, and `..` segments, reserved names, mixed separators, and NUL bytes are rejected before staging starts. The staging directory is created beside the selected destination, refuses symlinks, must contain exactly the expected files, and is committed only after every content check passes. A traversal test confirms that a destination shaped to escape creates neither a staging directory nor any file outside it.

`fixtures/hostile/` holds the regression corpus of hostile inputs. Property tests cover accepted and rejected path shapes. Four `cargo-fuzz` targets cover SVG preflight, relative paths, ICO encoder input, and ICNS encoder input. The [Fuzz smoke](../.github/workflows/fuzz.yml) workflow runs each target for a fixed number of inputs (2,500 SVG, 10,000 path, 1,000 ICO, and 400 ICNS) on a pinned nightly toolchain every Monday and on pull requests that change `imaguncula-core` or the fuzz targets. An input that causes a crash is added to the corpus before its fix is merged.

See [SECURITY.md](../SECURITY.md) for private reporting and [determinism.md](determinism.md) for what is and is not byte-for-byte reproducible.

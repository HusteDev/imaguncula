# Custom outputs and export

Status: Phase 3 exporter implemented locally; hosted E2E and manual accessibility evidence pending

Imaguncula resolves every custom row into typed review data before it writes any package files. The Svelte table presents the same `OutputSpec` values that the Rust export engine consumes, but Rust validation remains authoritative.

## Custom output rows

A custom output defines:

- a stable row ID;
- exact width and height from 1 through 16,384 pixels;
- PNG format;
- a filename and package-relative destination;
- a source role and usage explanation;
- enabled state through `enabledOutputs`;
- optional per-output source and no-upscale override data through `outputOverrides`.

`resolve_outputs` combines the project row with its override and current source metadata. Each returned row includes the review-table fields, its sanitized package path, source label and hash, and typed issues. Status is `ready`, `warning`, or `error`; color is never the only status signal.

Package paths use forward slashes and pass one cross-platform sanitizer. Absolute paths, drive prefixes, empty components, traversal components, control characters, Windows-reserved names, trailing dots or spaces, and case-insensitive collisions fail before export.

## Source selection and portability

Rows use the `master` project source unless a direct per-output source is present. Direct sources are imported through the same bounded SVG/PNG pipeline as the master. A changed or missing source becomes a row error. A raster source cannot be enlarged unless the exact row stores an explicit `allowUpscale` override.

When a project is saved, source files contained under the project directory are stored as relative paths. Loading resolves those paths against the project file. Moving the project directory and its local source tree together therefore preserves both master and per-output references.

## Transactional directory export

The destination is `<selected-parent>/<base-filename>-icons`. Inspection is read-only and distinguishes a missing path, an empty directory, a non-empty previous package (identified by its `report.md`), an unrelated non-empty directory, and a non-directory. An existing empty directory or previous package requires explicit overwrite confirmation; an unrelated non-empty directory and a non-directory are always rejected, so a confirmed overwrite can never delete data Imaguncula did not write.

Export then:

1. resolves all included rows and rejects any row that is not ready;
2. creates a temporary staging directory beside the final destination;
3. builds a physical artifact plan, renders each logical output independently from its original source, and writes files and containers with create-new semantics;
4. writes deterministic `report.md` content;
5. rejects symlinks and verifies the exact tree, PNG dimensions, SVG parsing, ICO entries, metadata references, and optional ZIP bytes;
6. checks cancellation before commit;
7. atomically renames the verified package into place.

For a confirmed overwrite, the previous directory is first renamed into the staging area. A failed final rename restores it. When that restore also fails, the staging directory is preserved instead of cleaned up and the commit error names the exact path holding the previous package. Cancellation or every other failure removes staging and never exposes an incomplete package under the final name.

The desktop adapter permits one active export job. It emits `export-progress` and `export-finished` events, and cancellation sets a shared atomic token checked by the core. The UI announces progress through an ARIA live region.

## Generation report

The returned and written report records project and base names, physical generated paths, applicable dimensions, format, exact source path and content hash, and accepted or outstanding warnings. Multi-size containers, scalable vectors, and metadata use `multi-size`, `scalable`, and `n/a` rather than fake zero dimensions. It has deterministic ordering and no timestamp so identical inputs remain comparable.

## Verification

Core tests cover multi-row output, exact paths and dimensions, cancellation cleanup, collision detection, missing and changed overrides, protected overwrite, moved project-local sources, path properties, deterministic reporting, and a semantic golden fixture whose decoded exported pixels match same-size preview pixels.

The `Desktop E2E` workflow builds the real application under Linux, launches it through `tauri-driver`, opens the custom-output table, checks its 11 semantic columns, and runs axe against the visible Phase 2 surface. Run it locally with `pnpm test:e2e` after installing `tauri-driver` and the host WebDriver prerequisites documented by Tauri. Automated axe does not replace the pending manual keyboard and screen-reader pass.

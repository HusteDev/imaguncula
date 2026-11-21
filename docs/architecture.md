# Architecture

Status: v1 architecture implemented locally; hosted release gates pending

Imaguncula is an offline icon generator with desktop and headless interfaces over one processing core.

## Dependency direction

```text
Svelte frontend
      |
      v
Tauri IPC adapter -----> imaguncula-core <----- imaguncula CLI
                              |
                              v
                     imaguncula-presets
```

Dependencies point toward the processing core. The core does not depend on Tauri, Svelte, browser APIs, or operating-system UI services. The preset crate owns bundled, versioned preset definitions and their schema support and is consumed by the core through exact preset identities.

## Current crates and applications

### `imaguncula-core`

Pure Rust library for source validation, rendering, output resolution, encoding, metadata, project serialization, reports, and staged exports. It owns the strict schema-v2 project model and migration ladder, explicit preset-diff acceptance, bounded PNG/SVG import, sanitized SVG intermediates, canonical straight-alpha RGBA8 rendering, no-upscale enforcement, final-only opaque flattening, content-addressed preview caching, multi-role preset/custom output resolution, safe-zone alpha analysis, PNG/ICO/ICNS/SVG/metadata packaging, conditional Android XML, Xcode catalog cross-validation, deterministic ZIPs and reports, and verified transactional directory export. Interface adapters do not duplicate those rules.

### `imaguncula-presets`

Rust crate and data bundle for versioned platform presets. Draft 2020-12 JSON Schema validation covers structure; Rust tests enforce identifier uniqueness, valid package/output cross-references, review metadata, metadata token use, and immutable directory identity. Windows desktop/MSIX, Web/PWA, Linux hicolor, Apple, and Android presets are compiled into the application and consumed by exact ID/version. Preset summaries expose only the newest selectable version while older definitions remain available for reproducible projects and reviewed migration.

### `imaguncula-cli`

Display-server-free binary with saved-project and one-shot preset commands. It parses arguments, constructs or loads `ProjectState`, and delegates resolution, rendering, verification, and commit to the same `imaguncula-core::export_project` entry point used by Tauri. Machine-readable output serializes the same typed report.

### `imaguncula-desktop`

Thin Tauri 2 adapter. It owns desktop lifecycle, the native Open/Save dialog plugin, native file-drop events, typed IPC commands, the preview-cache mutex, and one-active-job export coordination. Background export emits typed progress and completion events and forwards cancellation into the core; it contains no project validation, path, rendering, or commit rules.

### Svelte frontend

Svelte 5 and TypeScript application built by SvelteKit in static SPA mode through Vite. It owns presentation and user interaction: file selection/drop state, source metadata and warning display, project dirty state, actual-size and magnified preview controls, a virtualized semantic output table, destination confirmation, progress announcements, and report display. It does not own source-of-truth validation or export authority.

## Security boundary

The Tauri capability exposes core window functionality plus only the dialog plugin's Open and Save commands. No HTTP or general filesystem plugin capability is present. Project file bytes are read and written by typed core commands, which enforce the compound extension, a 1 MiB load limit, strict structure, validation, and atomic replacement.

Source paths enter typed core commands. Source reads are capped at 64 MiB; PNG dimensions, pixels, and decoder memory are bounded; SVG element count and nesting are bounded before normalization. Active SVG content fails closed. The SVG renderer's resource resolvers return no external or embedded images, and system-font/text features are disabled. Preview dimensions and in-memory cache size are also bounded. See [source-import.md](source-import.md).

Untrusted image and path handling belongs in `imaguncula-core`. Every package-relative path passes one sanitizer before any write. Export stages beside the destination, rejects symlinks, verifies exact tree membership, decoded PNG dimensions, ICO and ICNS entries, parsed SVG/JSON/XML, exact `Contents.json` filename sets, metadata references, and ZIP entry bytes, then commits only by rename. Confirmed overwrite retains the previous package until the replacement verifies. Interface checks are explanatory; core checks are authoritative. See [custom-outputs-and-export.md](custom-outputs-and-export.md) and [linux-apple-export.md](linux-apple-export.md).

## Processing invariants

The processing implementation preserves these invariants:

1. Every output is rendered independently from the original SVG tree or canonical decoded raster.
2. Raster upscaling fails unless that exact output carries an explicit override.
3. Every package path passes through one relative-path sanitizer.
4. Export is staged, verified, and atomically moved into place.
5. Warnings and failures are typed data, not display strings.

The Phase 1 cache retains sanitized SVG trees by content hash and encoded previews by content hash plus target dimensions. Source replacement invalidates the previous role hash. Cache hits never bypass raster no-upscale authorization.

## Project and preset compatibility

Project files are human-readable `.imaguncula.json` documents with an explicit schema version and pinned preset versions. Unknown fields and unsupported schema versions fail closed. Saves use a temporary file in the destination directory, sync its contents, and atomically replace the destination. See [project-files.md](project-files.md).

Preset versions are immutable data directories. Logical outputs are separated from physical package definitions so one render can participate in standalone files, containers, directory trees, or generated metadata. See [presets.md](presets.md). Opening an older project must not silently adopt newer preset output requirements.

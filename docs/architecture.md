# Architecture

Imaguncula is an offline icon generator. A desktop application and a headless CLI both call one processing core.

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

Dependencies point toward the processing core. The core does not depend on Tauri, Svelte, browser APIs, or operating-system UI services. The preset crate holds the bundled, versioned preset definitions and their schema, and the core looks presets up by exact ID and version.

## Crates and applications

### `imaguncula-core`

A Rust library that does all of the processing: source validation, rendering, output resolution, encoding, metadata, project files, reports, and staged export. It owns:

- the schema-v2 project model, migration from schema v1, and explicit acceptance of preset changes;
- bounded PNG and SVG import and the sanitized SVG intermediate;
- rendering to straight-alpha RGBA8, the no-upscale rule, and opaque flattening applied only to final outputs;
- the content-addressed preview cache;
- resolving preset and custom outputs for every source role, and safe-zone alpha analysis;
- PNG, ICO, ICNS, SVG, and metadata packaging, Android XML, and Xcode catalog cross-checks;
- deterministic ZIPs and reports, and the verified transactional directory export.

The desktop adapter and the CLI do not reimplement any of these rules.

### `imaguncula-presets`

A Rust crate containing the versioned platform presets. A Draft 2020-12 JSON Schema checks their structure, and Rust tests check identifier uniqueness, package and output cross-references, review metadata, metadata token use, and that released version directories do not change. The Windows desktop and MSIX, Web/PWA, Linux hicolor, Apple, and Android presets are compiled into the application. Preset lists show only the newest version of each preset; older versions stay available so saved projects export the same way and can be migrated after review.

### `imaguncula-cli`

A binary that needs no display server, with commands for exporting a saved project and for a one-shot export from a preset. It parses arguments, builds or loads a `ProjectState`, and calls the same `imaguncula-core::export_project` function the desktop application uses. `--json` output serializes the same typed report.

### `imaguncula-desktop`

A thin Tauri 2 adapter. It handles the application lifecycle, the native Open and Save dialogs, native file-drop events, the typed IPC commands, the preview-cache mutex, and running one export at a time. Exports run in the background, send typed progress and completion events, and pass cancellation through to the core. The adapter has no project validation, path, rendering, or commit logic of its own.

### Svelte frontend

A Svelte 5 and TypeScript application, built by SvelteKit as a static single-page app through Vite. It handles presentation and interaction: choosing or dropping files, showing source metadata and warnings, tracking unsaved changes, actual-size and magnified previews, the virtualized output table, destination confirmation, progress announcements, and the export report. It does not decide what is valid and cannot export on its own.

## Security boundary

The Tauri capability grants `core:default` plus the dialog plugin's Open and Save commands. There is no HTTP, shell, or general filesystem permission. Project files are read and written by typed core commands, which require the `.imaguncula.json` extension, refuse files over 1 MiB, parse strictly, validate, and replace the destination atomically.

Source paths are passed to typed core commands. Source files over 64 MiB are refused; PNG dimensions, pixel count, and decoder memory are bounded; SVG element count and nesting are checked before normalization. Active SVG content is rejected. The SVG renderer's resource resolvers return no external or embedded images, and system fonts and text rendering are disabled. Preview dimensions and the in-memory cache size are also bounded. See [source-import.md](source-import.md).

Untrusted image and path handling lives in `imaguncula-core`. Every package-relative path goes through one sanitizer before anything is written. Export writes a staging directory beside the destination, refuses symlinks, and checks the exact file list, decoded PNG dimensions, ICO and ICNS entries, that SVG, JSON, and XML parse, the exact filenames in each `Contents.json`, metadata references, and ZIP entry bytes. Only then is the package moved into place by rename. When the user confirms an overwrite, the previous package is kept until the new one has been verified. Checks in the interface exist to explain problems early; the core's checks are the ones that decide. See [custom-outputs-and-export.md](custom-outputs-and-export.md) and [linux-apple-export.md](linux-apple-export.md).

## Processing invariants

1. Every output is rendered on its own from the original SVG tree or the decoded source raster.
2. Raster upscaling fails unless that exact output has an explicit override.
3. Every package path goes through one relative-path sanitizer.
4. Export is staged, verified, and moved into place atomically.
5. Warnings and failures are typed data, not display strings.

The preview cache keeps sanitized SVG trees by content hash and encoded previews by content hash and target dimensions. Replacing a source invalidates the previous hash for that role. A cache hit never skips the no-upscale check for rasters.

## Project and preset compatibility

Project files are human-readable `.imaguncula.json` documents with an explicit schema version and pinned preset versions. Unknown fields and unsupported schema versions are rejected. Saving writes a temporary file in the destination directory, syncs it, and atomically replaces the destination. See [project-files.md](project-files.md).

Each preset version is an immutable data directory. Logical outputs are kept separate from physical package definitions, so one render can appear in standalone files, containers, directory trees, or generated metadata. See [presets.md](presets.md). Opening an older project never adopts a newer preset's output requirements without the user accepting them.

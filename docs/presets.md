# Preset data model

Preset definitions live under `crates/imaguncula-presets/presets/<preset-id>/<version>/preset.json`. A version directory is immutable after release. Any requirement change creates a new numeric version directory. Interfaces list only the newest version for new selection; exact older versions remain loadable for pinned projects and diff review.

## Outputs and packages

`outputs` describe logical renders: source role, format, dimensions, encoder settings, and usage text. `packages` describe how those renders appear in an export:

- `files` maps outputs to standalone relative paths;
- `container` creates an ordered ICO or ICNS file with explicit entry encodings;
- `directory` maps outputs into a rooted directory hierarchy;
- `metadata` generates JSON, XML, HTML, or text and resolves declared template tokens to outputs in sibling packages.

This separation prevents platform layout rules from leaking into raster rendering and allows a logical output to be referenced by generated metadata.

An output in a newer preset may declare `renamedFrom`. The comparison model treats only that explicit declaration as a rename; otherwise identities are shown as independent additions and removals. This avoids destructive similarity guesses.

`sourceRoles` declares the independently validated artwork roles a preset consumes. Interfaces expose those roles as source-management controls, but the core resolver blocks an output whose required source is absent or invalid. `exclusiveGroup` marks presets that are alternative modes of one platform output; selecting both iOS catalog modes fails project validation even if a caller bypasses the GUI.

## Validation

`schema/preset.schema.json` is JSON Schema Draft 2020-12. It rejects unknown properties, unsafe package paths, invalid dates and specification URIs, invalid encoder/format combinations, and malformed package variants.

The Rust schema test adds constraints that JSON Schema cannot express cleanly:

- source-role, output, package, group, and metadata-reference IDs are unique;
- every output source role exists;
- every package member names a defined output;
- every metadata reference names an output contained by its declared package;
- every metadata reference token is present in its template;
- real preset identity and version match their immutable directory path.

The schema-risk fixtures cover Windows ICO containers, Web/PWA standalone files and metadata, Linux hicolor directory trees, and an Apple AppIcon asset-catalog directory with `Contents.json`. Production presets add exact catalog-file cross-validation, conventional ICNS decode verification, Icon Composer and Android multi-role packages, and 134-row MSIX resource families. See [preset-review-process.md](preset-review-process.md).

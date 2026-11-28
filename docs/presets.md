# Preset data model

Preset definitions live in `crates/imaguncula-presets/presets/<preset-id>/<version>/preset.json`. A version never changes after release; any change to a preset's requirements goes into a new numbered version directory. New selections always use the newest version, and older versions stay loadable for projects pinned to them and for comparing versions.

## Outputs and packages

`outputs` describe individual renders: source role, format, dimensions, encoder settings, and a usage note. `packages` describe where those renders go in the export:

- `files` places outputs at their own relative paths;
- `container` builds an ICO or ICNS file from outputs in a fixed order, with an encoding for each entry;
- `directory` places outputs inside a directory tree;
- `metadata` generates a JSON, XML, HTML, or text file from a template whose tokens refer to outputs in other packages.

Keeping layout out of the outputs means rendering code never needs to know about platform directory rules, and a metadata file can refer to an output by ID.

An output in a newer preset version can declare `renamedFrom`. Version comparisons treat an output as renamed only when it declares this; otherwise a removed ID and a new ID are shown as a removal and an addition, rather than being guessed as a rename.

`sourceRoles` lists the separate artwork sources a preset uses. The desktop application shows a source picker for each, and the core blocks any output whose required source is missing or invalid. `exclusiveGroup` marks presets that are alternatives for the same platform output; a project that selects both iOS presets fails validation even if it was edited by hand.

## Validation

`schema/preset.schema.json` is a JSON Schema (draft 2020-12). It rejects unknown properties, unsafe package paths, invalid dates and specification URLs, encoders that do not match the output format, and malformed packages.

A Rust test checks rules that JSON Schema cannot express easily:

- source role, output, package, group, and metadata-reference IDs are unique;
- every output's source role is declared;
- every package entry names an existing output;
- every metadata reference names an output in the package it declares;
- every metadata reference's token appears in its template;
- each bundled preset's ID and version match its directory.

The test fixtures cover an ICO container, Web/PWA files and metadata, a Linux hicolor directory tree, and an Apple asset catalog with `Contents.json`. The bundled presets go further, with catalog filename checks, ICNS decoding, multi-source Icon Composer and Android packages, and the 134 outputs of `windows-msix` v2. [preset-review-process.md](preset-review-process.md) describes how presets are reviewed and changed.

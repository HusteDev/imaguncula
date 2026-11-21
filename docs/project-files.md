# Project files

Imaguncula project files use the compound extension `.imaguncula.json`. Schema version 2 is implemented in `imaguncula-core`; the Tauri commands are thin adapters over the same load/save and migration functions used by tests and the CLI.

## Schema version 2

Every project records:

- `schemaVersion`, `projectName`, and `baseFilename`;
- source roles, source paths, and SHA-256 content hashes;
- selected preset IDs with immutable versions;
- enabled outputs, custom outputs, and per-output overrides;
- export settings;
- typed warning acceptances;
- accepted preset migrations with source version, destination version, and reviewed diff fingerprint.

New projects select the current Windows desktop and Web/PWA presets. Each selected preset remains pinned to its exact version until the user reviews and accepts a newer bundled version.

## Load rules

- The filename must end with `.imaguncula.json` case-insensitively.
- Files larger than 1 MiB are rejected before their contents are read.
- JSON must declare schema version 1 or 2. Version 1 migrates through the explicit ladder to version 2; future and unknown versions fail closed.
- Unknown object fields are rejected.
- IDs, hashes, dimensions, filenames, and relative destinations are validated by the core.
- Platform-specific unsafe names and path forms are rejected consistently on every host.

The load result includes `migrationNotices`. The frontend marks migrated documents dirty so the new schema is not presented as persisted until Save succeeds.

## Preset-version migration

Opening a project never silently changes a pinned preset. The core compares the pinned definition with the newest bundled version and returns added, removed, and explicitly renamed output IDs plus a deterministic SHA-256 review fingerprint. Applying an upgrade requires that exact fingerprint. A stale or substituted fingerprint fails closed. The core then updates the pin, preserves renamed output inclusion and overrides, removes deleted rows, enables only newly added defaults, and appends a `presetMigrations` acceptance record.

## Save rules

The core validates before serializing. It writes pretty JSON with a trailing newline to a temporary file in the destination directory, synchronizes the file, then atomically replaces the destination. A validation, serialization, or write failure does not replace the existing project.

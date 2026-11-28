# Project files

Imaguncula project files end in `.imaguncula.json`. The current format is schema version 2. Loading, saving, and migration are implemented in `imaguncula-core`, and the desktop application and the CLI both call the same functions.

## Schema version 2

A project file records:

- `schemaVersion`, `projectName`, and `baseFilename`;
- each source's role, path, and SHA-256 hash;
- the selected preset IDs and their exact versions;
- included outputs, custom outputs, and per-output overrides;
- export settings;
- accepted warnings;
- accepted preset migrations, each with the old version, the new version, and the fingerprint of the reviewed changes.

New projects select the current Windows desktop and Web/PWA presets. A project keeps each preset's exact version until you review and accept a newer bundled version.

## Load rules

- The file name must end with `.imaguncula.json` (in any letter case).
- Files larger than 1 MiB are rejected before they are read.
- The JSON must declare schema version 1 or 2. Version 1 files are migrated to version 2 when loaded; any other version is rejected.
- Unknown fields are rejected.
- The core validates IDs, hashes, dimensions, file names, and relative destinations.
- Unsafe names and path forms are rejected the same way on every operating system.

Loading returns `migrationNotices` for any migration that was applied. The desktop application marks a migrated project as unsaved, because the file on disk is still in the old format until you save.

## Preset-version migration

Opening a project never changes a pinned preset. The core compares the pinned preset with the newest bundled version and returns the output IDs that were added, removed, or explicitly renamed, plus a SHA-256 fingerprint of that comparison. Applying the upgrade requires the same fingerprint, so an upgrade that differs from what was reviewed is refused. The core then updates the pinned version, keeps the inclusion and overrides of renamed outputs, removes deleted outputs, includes new outputs only if they are on by default, and adds a `presetMigrations` record.

## Save rules

The core validates the project before saving. It writes formatted JSON with a trailing newline to a temporary file in the destination folder, flushes it to disk, and then replaces the destination in one step. If validation or writing fails, the existing project file is left unchanged.

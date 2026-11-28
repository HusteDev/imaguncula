# Custom outputs and export

Before writing any files, Imaguncula turns every output row, preset or custom, into an `OutputSpec` with its review data. The Svelte output table displays the same `OutputSpec` values that the Rust exporter uses, and the Rust validation is the one that decides whether a row can be exported.

## Custom output rows

A custom output has:

- a stable row ID;
- a width and height from 1 to 16,384 pixels;
- PNG format;
- a filename and a destination inside the package;
- a source role and a usage note;
- an entry in `enabledOutputs` when it is included;
- optional per-output source and upscale settings in `outputOverrides`.

`resolve_outputs` combines each row with its override and the current source metadata. Each resolved row has the table fields, its sanitized package path, the source label and hash, and any issues. Its status is `ready`, `warning`, or `error`, and the table shows the status as text as well as color.

Package paths use forward slashes and all go through one sanitizer, on every operating system. Absolute paths, drive letters, empty segments, `.` and `..`, control characters, Windows reserved names, names ending in a dot or space, and paths that collide when compared case-insensitively are rejected before export.

## Sources and moving projects

A row uses the project's `master` source unless it has its own source override. Override sources go through the same size-limited SVG and PNG import as the master. If a source changes or goes missing, its rows show an error. A row cannot enlarge a PNG unless that row has `allowUpscale` set.

When a project is saved, sources inside the project's folder are stored as paths relative to the project file, and they are resolved against the project file when it is opened. You can move the project folder together with its sources and both the master and per-output sources still resolve.

## Export steps

The package is written to `<selected-parent>/<base-filename>-icons`. Imaguncula first checks, without changing anything, whether that path is missing, an empty directory, a previous package (a directory with a `report.md`), some other non-empty directory, or a file. An empty directory or a previous package is only replaced after you confirm. Any other non-empty directory or a file is always refused, so a confirmed replacement can never delete files Imaguncula did not write.

Export then:

1. resolves all included rows and stops if any row is not ready;
2. creates a staging directory next to the destination;
3. renders each output separately from its original source and writes every file and container, refusing to overwrite any file that already exists;
4. writes `report.md`;
5. rejects symbolic links and checks the exact file list, PNG dimensions, SVG parsing, ICO entries, metadata references, and the ZIP contents;
6. stops if the export was cancelled;
7. renames the verified package into place.

When replacing a previous package, Imaguncula first moves it into the staging area. If the final rename fails, the previous package is moved back. If moving it back also fails, the staging directory is kept and the error names the path that holds the previous package. After a cancellation or any other failure, the staging directory is removed and nothing appears under the final name.

The desktop application runs one export at a time. It reports progress through `export-progress` and `export-finished` events, and cancelling sets a flag the exporter checks between steps. Progress is announced to screen readers through an ARIA live region.

## Generation report

`report.md` records the project and base names, each generated file's path, dimensions, and format, the file name and SHA-256 hash of its source, and every accepted or outstanding warning. It does not include absolute source paths. Containers with several sizes, SVG files, and metadata files show `multi-size`, `scalable`, or `n/a` instead of a dimension. The report is sorted the same way every time and has no timestamp, so the same inputs always produce the same report. The report the desktop application displays after export also shows each source's full path.

## Tests

Core tests cover exports with many rows, exact paths and dimensions, cleanup after cancellation, path collisions, missing and changed source overrides, protected replacement, moved project folders, path sanitizing, report ordering, and a golden fixture whose exported pixels match the preview pixels at the same size.

The `Desktop E2E` workflow runs on every pull request and push to `main`. It builds the real application on Linux, drives it through `tauri-driver`, opens the custom-output table, checks its 11 columns, and runs axe accessibility checks against the visible interface. Run it locally with `pnpm test:e2e` after installing `tauri-driver` and the WebDriver prerequisites from the Tauri documentation. Automated checks do not replace the manual keyboard and screen-reader pass in [accessibility-audit.md](accessibility-audit.md).

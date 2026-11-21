# Command-line interface

Status: v1 command surface implemented locally

Install the binary from a checkout:

```text
cargo install --path crates/imaguncula-cli --locked
```

Export a saved project into `<output>/<base>-icons`:

```text
imaguncula export project.imaguncula.json --output ./dist
```

Run one bundled preset directly from a source:

```text
imaguncula quick source.svg --preset web-pwa --output ./dist
```

Multi-role presets use repeatable generic role assignments:

```text
imaguncula quick legacy.svg --preset android-launcher --output ./dist \
  --role foreground=foreground.svg \
  --role background=background.svg
```

`ROLE=PATH` splits only at the first equals sign, so Windows drive-qualified paths are preserved. Optional output groups use their preset defaults in quick mode; save a project file when a scripted export needs non-default groups such as Android monochrome resources.

Both commands are non-interactive. An existing final package fails unless `--overwrite` is present, and a confirmed overwrite only ever replaces a directory containing a previous package's `report.md`. `export` refuses a project whose sources resolve outside the project file's own directory unless `--allow-external-sources` acknowledges each listed reference. `--no-zip` disables the combined archive. `--json` prints the typed export result instead of the short human summary.

Stable process exit codes are:

| Code | Meaning |
| ---: | --- |
| 0 | Export completed and verified |
| 2 | Project or preset input is invalid |
| 3 | Source import failed |
| 4 | Resolved outputs or metadata references are not exportable |
| 5 | Rendering, packaging, verification, commit, or report serialization failed |

The CLI has no Tauri dependency and requires no display server. Integration tests compare its complete exported directory tree byte-for-byte with a direct call matching the desktop adapter's core path.

See [cli-reference.md](cli-reference.md) for help text generated from the compiled binary. CI rejects a stale generated reference.

# Command-line interface

The `imaguncula` command exports icon packages without the desktop application. Each [GitHub release](https://github.com/HusteDev/imaguncula/releases) includes an `imaguncula-cli-<version>-<target>.tar.gz` archive for x64 Windows, x64 Linux, and Apple silicon macOS. The CLI is not yet published to crates.io; to build it from a checkout, run:

```text
cargo install --path crates/imaguncula-cli --locked
```

Export a saved project into `<output>/<base>-icons`:

```text
imaguncula export project.imaguncula.json --output ./dist
```

Export one source through one bundled preset, without a project file:

```text
imaguncula quick source.svg --preset web-pwa --output ./dist
```

Presets that need several sources take one `--role` option per extra source:

```text
imaguncula quick legacy.svg --preset android-launcher --output ./dist \
  --role foreground=foreground.svg \
  --role background=background.svg
```

`ROLE=PATH` is split at the first equals sign, so Windows paths such as `C:\art\layer.svg` work. `quick` uses each preset's default optional groups. To export non-default groups, such as Android monochrome resources, save a project file and use `export`.

Neither command prompts. If the final package directory already exists, the export fails unless you pass `--overwrite`, and even then Imaguncula only replaces a directory that contains a previous package's `report.md`. `export` refuses a project whose sources are outside the project file's directory and lists them; pass `--allow-external-sources` once you have reviewed them. `--no-zip` leaves out the combined ZIP. `--json` prints the full export result as JSON instead of the short summary.

Exit codes:

| Code | Meaning |
| ---: | --- |
| 0 | Export completed and verified |
| 2 | The project, preset, or command-line arguments are invalid, or external sources were not allowed |
| 3 | A source could not be imported |
| 4 | The resolved outputs or their metadata references cannot be exported |
| 5 | Rendering, packaging, verification, the final rename, or writing the report failed |

The CLI does not depend on Tauri. Integration tests check that its exported directory tree is byte-for-byte identical to the one produced through the desktop application's code path.

[cli-reference.md](cli-reference.md) contains the full help text, generated from the built binary. CI fails if that file is out of date.

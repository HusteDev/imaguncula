# Example projects

These projects use schema version 2, checked-in artwork, and relative paths.

- `svg-windows-web.imaguncula.json` uses one SVG master with the Windows desktop (version 2) and Web/PWA presets that new projects start with, including the 42-pixel Windows compatibility size.
- `png-windows.imaguncula.json` downsamples a PNG master to every size without upscaling it.
- `android-multi-role.imaguncula.json` assigns the master, foreground, and background roles required for legacy and adaptive Android resources.

Open a file in the desktop application or export it directly:

```text
imaguncula export examples/svg-windows-web.imaguncula.json --output ./dist --allow-external-sources
```

The flag is needed because the examples use artwork stored outside the `examples` directory. The CLI refuses to read sources outside a project's directory unless you pass it.

The CLI integration tests (`crates/imaguncula-cli/tests/example_projects.rs`) export all three projects and fail if any source path, hash, preset version, output, package, report, or combined ZIP stops matching.

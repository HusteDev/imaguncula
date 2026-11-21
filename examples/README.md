# Example projects

These schema-v2 projects use only checked-in artwork and relative paths.

- `svg-windows-web.imaguncula.json` uses one SVG master and the current new-project Windows/Web presets, including the Windows desktop v2 42-pixel compatibility entry.
- `png-windows.imaguncula.json` demonstrates bounded raster downsampling without upscaling.
- `android-multi-role.imaguncula.json` assigns the master, foreground, and background roles required for legacy and adaptive Android resources.

Open a file in the desktop application or export it directly:

```text
imaguncula export examples/svg-windows-web.imaguncula.json --output ./dist --allow-external-sources
```

The flag is required because the example projects reference the repository's checked-in artwork outside their own directory; the CLI refuses such references in any project without this reviewed authorization.

The CLI integration suite exports all three projects through the production path and fails if a relative source, hash, preset pin, output selection, package, report, or combined ZIP becomes invalid.

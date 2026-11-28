# Source import and preview

Imaguncula accepts PNG and SVG sources and identifies the format from the file's contents, not its extension. The desktop application passes the chosen path and source role to `imaguncula-core`, which reads, validates, hashes, and normalizes the file before the project records it.

## Limits

- Source file: at most 64 MiB.
- PNG: at most 16,384 pixels in either dimension, 64 million pixels in total, and 256 MiB of decoder memory.
- SVG: at most 100,000 elements, 256 levels of nesting, 64 filter elements and primitives, and 2 MiB of combined path and polygon data. Rendering an SVG is stopped after 30 seconds.
- Preview: at most 4,096 pixels in either dimension.
- Preview cache: at most 64 previews or 64 MiB, plus at most eight validated sources.

These limits are enforced in Rust. The file picker's filters and the interface describe them, but are not what enforces them.

## SVG rules

Before parsing, Imaguncula rejects SVG files that contain a document type or entity declarations, scripts, `foreignObject`, event-handler attributes, or `javascript:` and `vbscript:` references. The renderer is set up so it cannot load images, whether local, remote, or embedded. External resources, fonts, text, styles, and features that do not apply to a static image are removed or ignored, with a warning where that can be done safely.

An accepted SVG is parsed once into a sanitized `usvg` tree, which is cached by the file's SHA-256 hash. Every preview size is rendered from that tree; the original document is never passed to the renderer.

## PNG and color

Imaguncula decodes the whole PNG, within the limits above, to find its dimensions, whether it has an alpha channel, whether any pixels are actually transparent, and whether it carries an ICC profile. PNG previews use straight-alpha RGBA8 with alpha-aware Lanczos3 resampling. Enlarging a PNG fails unless the request explicitly allows upscaling.

Imaguncula 1.x detects ICC profiles but does not apply them. Pixel values are treated as sRGB, a warning is shown when the source has a profile, and generated preview PNGs are tagged as sRGB. See [ADR 0010](adr/0010-png-color-profile-policy.md).

## Preview caching

Previews are cached by source hash, width, and height. Each request re-reads the source and checks its hash before using a cached tree or preview. Replacing a source with different content removes the cached previews and SVG tree for the old hash.

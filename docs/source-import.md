# Source import and preview

Imaguncula accepts one master source in content-verified PNG or SVG form. A filename extension does not determine the format. The desktop adapter passes the selected path and source role to `imaguncula-core`; the core reads, validates, hashes, and normalizes the source before the UI records it in a project.

## Enforced limits

- Source file: 64 MiB maximum.
- PNG: 16,384 pixels maximum in either dimension, 64 million pixels maximum, and 256 MiB maximum decoder allocation.
- SVG: 100,000 elements maximum and 256 levels of nesting maximum.
- Preview: 4,096 pixels maximum in either dimension.
- Preview cache: 64 encoded previews or 64 MiB, plus at most eight validated source intermediates.

These checks run in Rust. File-picker filters and UI controls explain the policy but do not authorize input.

## SVG policy

The pre-scan rejects document types and entity declarations, scripts, `foreignObject`, event-handler attributes, and script-scheme references. SVG image resolvers cannot load local, remote, or embedded images. External resources, fonts, text, styles, and unsupported static-output features are removed or ignored with structured warnings where safe.

Accepted SVGs become a sanitized `usvg` tree cached by SHA-256 content hash. Every preview size renders directly from that tree. The raw document is never a renderer input.

## PNG and color policy

PNG validation performs a bounded full decode to establish dimensions, alpha-channel presence, actual transparent pixels, and ICC-profile presence. Raster previews use straight-alpha RGBA8 and alpha-aware Lanczos3. Enlarging a raster fails unless the exact request carries an explicit override.

V1 detects but does not apply ICC profiles. It treats decoded channel values as sRGB, warns when a profile exists, and writes an sRGB marker to generated preview PNGs. See [ADR 0010](adr/0010-png-color-profile-policy.md).

## Preview identity and invalidation

Encoded previews use `(source content hash, width, height)` keys. Each request re-reads the bounded source and verifies the hash before using a retained intermediate or encoded preview. Replacing a role with different content removes cached previews and the sanitized tree for the previous hash.

# Windows desktop and Web/PWA export

Status: Phase 3 implemented locally

New projects select the immutable `windows-desktop` v2 and `web-pwa` v1 presets. Preset checkboxes add or remove exact ID/version pairs and their logical output rows. Custom PNG rows remain available beside preset outputs. Projects pinned to `windows-desktop` v1 remain unchanged until the user reviews and accepts the v2 preset migration.

## Windows desktop

The Windows package is `windows-desktop/<base>.ico`. Version 2 contains 15 independently rendered square entries at 16, 20, 24, 30, 32, 36, 40, 42, 48, 60, 64, 72, 80, 96, and 256 pixels. The 42-pixel BMP-with-alpha entry is a compatibility layer for observed Win32 taskbar selection at 175% display scaling; Microsoft does not list 42 pixels in its standard taskbar table, so the other documented layers remain unchanged. Entries through 96 pixels use 32-bit BMP-with-alpha encoding; the 256-pixel entry is PNG-compressed. Verification reopens the ICO, checks entry count and order, checks the declared encoding of every entry, decodes each image, and compares exact dimensions.

## Web/PWA

The `web-pwa/` package contains:

- `favicon.ico` with 16, 32, and 48-pixel BMP entries;
- sanitized `favicon.svg` when the source is SVG;
- `apple-touch-icon.png` at 180 pixels;
- `icon-192.png`, `icon-512.png`, and `icon-maskable-512.png`;
- parsed `site.webmanifest` metadata;
- `icons.html` link markup.

Metadata templates declare every output reference. Export fails before staging if a referenced output is disabled. Verification confirms each reference appears in the generated text and resolves to a staged file.

The maskable row preview draws the 80% safe-zone circle. Core resolution samples sanitized source alpha at 128 pixels and reports `maskable-content-outside-safe-zone` when more than 1% of visible alpha lies outside that circle. This is a review heuristic: disposable background may extend outside the circle, but important artwork should not.

## Combined ZIP

When enabled, `<base>-icons.zip` is written inside the final package directory. It contains every preset-ID directory and `report.md` at the archive root. Entries use sorted relative paths, Deflate compression, fixed ZIP metadata, and stable permissions. Verification reopens the archive, rejects unsafe or duplicate names, compares its exact entry set, and compares every decompressed entry byte-for-byte with staging.

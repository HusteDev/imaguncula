# Windows desktop and Web/PWA export

New projects select the `windows-desktop` v2 and `web-pwa` v1 presets. Turning a preset on or off adds or removes its output rows; custom PNG rows can sit alongside them. A project saved with `windows-desktop` v1 keeps using v1 until you review and accept the migration to v2.

## Windows desktop

The package is `windows-desktop/<base>.ico`. Version 2 contains 15 square entries, each rendered separately from the source: 16, 20, 24, 30, 32, 36, 40, 42, 48, 60, 64, 72, 80, 96, and 256 pixels. Microsoft's taskbar size table does not list 42 pixels, but Win32 taskbars at 175% display scaling were observed choosing that size, so version 2 adds it and leaves the other sizes unchanged. Entries up to 96 pixels are 32-bit BMPs with alpha; the 256-pixel entry is PNG-compressed. Verification reopens the ICO and checks the number, order, and encoding of the entries, then decodes each one and checks its dimensions.

## Web/PWA

The `web-pwa/` package contains:

- `favicon.ico` with 16, 32, and 48-pixel BMP entries;
- a sanitized `favicon.svg` when the source is SVG;
- `apple-touch-icon.png` at 180 pixels;
- `icon-192.png`, `icon-512.png`, and `icon-maskable-512.png`;
- `site.webmanifest`;
- `icons.html` with the `<link>` tags for the page head.

`site.webmanifest` and `icons.html` are generated from templates that name the outputs they reference. If one of those outputs is turned off, export fails before writing anything. Verification checks that every reference appears in the generated text and points to a file in the package.

The maskable icon preview shows the 80% safe-zone circle. Imaguncula samples the source at 128 pixels and warns with `maskable-content-outside-safe-zone` when more than 1% of the source's visible alpha falls outside that circle. This is a guide, not a rule: background can extend past the circle, but anything important should stay inside it.

## Combined ZIP

When **Include combined ZIP** is on, `<base>-icons.zip` is written inside the package directory. The archive holds every preset directory and `report.md` at its root. Entries are sorted by path, Deflate-compressed, and written with fixed timestamps and permissions, so the same export always produces the same ZIP. Verification reopens the archive, rejects unsafe or duplicate names, checks that it contains exactly the expected files, and compares each decompressed file byte-for-byte with the package.

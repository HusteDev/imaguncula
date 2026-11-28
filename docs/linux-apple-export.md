# Linux and Apple export

Imaguncula includes these Linux and Apple presets:

| Preset | Output |
| --- | --- |
| `linux-hicolor` v1 | `hicolor/<size>x<size>/apps/<base>.png` at 16, 24, 32, 48, 64, 128, 256, and 512 pixels, plus `hicolor/scalable/apps/<base>.svg` when the source is SVG |
| `apple-ios-modern` v1 | `AppIcon.appiconset` with one universal 1024-pixel iOS image |
| `apple-ios-compatibility` v1 | `AppIcon.appiconset` with separate iPhone, iPad, iPad Pro, and App Store images for each point size and scale |
| `apple-macos-appicon` v1 | `AppIcon.appiconset` with the ten macOS images from 16 through 1024 pixels |
| `apple-macos-icns` v1 | `<base>.icns` with separately rendered 16, 32, 64, 128, 256, 512, and 1024-pixel entries |
| `apple-icon-composer-guidance` v1 | 1024-pixel foreground and background layers plus instructions for Icon Composer; no `.icon` file |

## Linux hicolor

The PNG sizes follow the freedesktop hicolor directory layout, including the required 48-pixel icon. An SVG source also produces a sanitized copy in `scalable/apps`. With a PNG source the scalable output cannot be produced, so that row shows an error until you assign an SVG source or turn the **Scalable SVG** group off.

Reference: [freedesktop.org Icon Theme Specification](https://specifications.freedesktop.org/icon-theme/latest/).

## Xcode asset catalogs

The two iOS presets are alternatives: selecting one in the desktop application deselects the other, and a hand-edited project that selects both fails validation with `conflicting-selection`.

Before an asset catalog is written, export checks that:

- `Contents.json` is valid JSON;
- the filenames listed in `images[].filename` are exactly the files in the catalog;
- each filename is a safe relative path next to `Contents.json`;
- every listed file exists;
- every PNG's pixel size equals its point size times its scale.

The `apple-assets.yml` workflow runs on every pull request and push to `main`. On macOS it exports all three catalogs with the CLI, checks each `Contents.json`, and compiles each catalog with `xcrun actool`. A separate Linux job validates every preset's schema and package references.

References: [Configuring your app icon](https://developer.apple.com/documentation/xcode/configuring-your-app-icon/) and [archived asset-catalog icon-set format](https://developer.apple.com/library/archive/documentation/Xcode/Reference/xcode_ref-Asset_Catalog_Format/IconSetType.html).

## ICNS

Each ICNS entry is rendered separately from the source. Verification reopens the finished file, checks that it contains exactly the expected entry types, decodes each entry, and checks its dimensions. An integration test also checks that transparency survives in every entry.

## Icon Composer

Apple's layered icons are authored in Icon Composer, so Imaguncula exports the layer artwork rather than generating a `.icon` file it cannot fully describe.

Selecting the Icon Composer preset adds foreground and background source pickers. Both sources are required, must be square, and are validated separately; export stays blocked until both are set. The package contains `IconComposerSources/foreground.png`, `background.png`, and `README.txt`. You then set depth, translucency, appearances, and platform behavior in Icon Composer and check the saved `.icon` in Xcode.

Reference: [Creating your app icon using Icon Composer](https://developer.apple.com/documentation/Xcode/creating-your-app-icon-using-icon-composer).

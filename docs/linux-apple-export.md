# Linux and Apple export

Status: Phase 4 implemented locally; the first hosted `actool` run remains pending

Imaguncula bundles the following immutable Phase 4 presets:

| Preset | Produced artifact |
| --- | --- |
| `linux-hicolor` v1 | `hicolor/<size>x<size>/apps/<base>.png` at 16, 24, 32, 48, 64, 128, 256, and 512 pixels, plus `hicolor/scalable/apps/<base>.svg` for SVG input |
| `apple-ios-modern` v1 | `AppIcon.appiconset` with one universal 1024-pixel iOS image |
| `apple-ios-compatibility` v1 | `AppIcon.appiconset` with explicit iPhone, iPad, iPad Pro, and App Store point-size/scale slots |
| `apple-macos-appicon` v1 | `AppIcon.appiconset` with the ten macOS point-size/scale slots from 16 pixels through 1024 pixels |
| `apple-macos-icns` v1 | Multi-representation `<base>.icns` with independently rendered 16, 32, 64, 128, 256, 512, and 1024-pixel entries |
| `apple-icon-composer-guidance` v1 | Validated 1024-pixel foreground/background layers and a local instruction document; never a fabricated `.icon` |

## Linux hicolor

The fixed-size PNG directory tree follows the freedesktop hicolor fallback layout. The required 48-pixel application icon is included. An SVG source also produces a sanitized scalable copy. A PNG-only source cannot silently claim scalable output; the row fails until an SVG source is assigned or that output is disabled.

Reference: [freedesktop.org Icon Theme Specification](https://specifications.freedesktop.org/icon-theme/latest/).

## Xcode asset catalogs

The iOS modern and compatibility presets are members of the `apple-ios-appicon-mode` exclusive group. Enabling one in the GUI disables the other. A hand-edited project selecting both fails core project validation with `conflicting-selection`.

Every catalog export verifies all of the following before commit:

- `Contents.json` parses as JSON;
- the declared `images[].filename` set exactly equals the metadata package references;
- each referenced filename is a sanitized relative path beside `Contents.json`;
- every declared file exists in staging;
- every PNG decodes to the point-size multiplied by its scale.

The macOS workflow generates all three catalogs through `imaguncula-cli`, validates each JSON property list with `plutil`, and compiles each asset catalog with `xcrun actool`. Schema and semantic package-reference validation also run on Linux. Until that workflow has run in a hosted repository, local tests prove generation and internal verification but do not prove acceptance by the installed Xcode version.

References: [Configuring your app icon](https://developer.apple.com/documentation/xcode/configuring-your-app-icon/) and [archived asset-catalog icon-set format](https://developer.apple.com/library/archive/documentation/Xcode/Reference/xcode_ref-Asset_Catalog_Format/IconSetType.html).

## ICNS

The ICNS encoder receives a fresh render for every representation. Verification reopens the final container, requires exactly the selected entry types, decodes each entry to RGBA, and checks its dimensions and buffer length. The integration test also checks transparency survives every decoded representation.

## Icon Composer

Apple's current layered icon workflow belongs to Icon Composer. Imaguncula therefore exports validated source material instead of flattening it into a misleading `.icon` document.

Selecting the Icon Composer preset exposes foreground and background source-role pickers in the GUI. Both roles are required, square, and independently validated. Export remains blocked until both sources resolve. The result contains `IconComposerSources/foreground.png`, `background.png`, and `README.txt`; the user completes depth, translucency, appearances, and platform behavior in Icon Composer and validates the saved `.icon` in Xcode.

Reference: [Creating your app icon using Icon Composer](https://developer.apple.com/documentation/Xcode/creating-your-app-icon-using-icon-composer).

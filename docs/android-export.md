# Android launcher export

Status: Phase 5 implemented locally; the first hosted AAPT2 compile remains pending

The immutable `android-launcher` v1 preset produces legacy launcher PNGs, adaptive resources, optional round and themed variants, and Play Store artwork. Android sources are independent project roles; Imaguncula never reconstructs layers from a flat master.

## Source roles

| Role | Requirement | Use |
| --- | --- | --- |
| `master` | Required | Legacy launcher, optional round launcher, and Play Store artwork |
| `foreground` | Required for included adaptive outputs | Foreground layer on a 108-dp canvas |
| `background` | Required for included adaptive outputs | Full-bleed adaptive background layer |
| `monochrome` | Optional | API 33 themed-icon layer |

The GUI derives source-role controls from selected preset data. The CLI accepts repeatable `--role ROLE=PATH` assignments. Core output resolution validates each role independently. Default Android export includes adaptive rows and therefore fails before staging if foreground or background is missing. Disabling every adaptive row permits a deliberate legacy-only export from the flat master.

## Resource tree

Legacy `ic_launcher.png` files render at 48, 72, 96, 144, and 192 pixels under `res/mipmap-{mdpi,hdpi,xhdpi,xxhdpi,xxxhdpi}`. Adaptive foreground and background layers render at 108, 162, 216, 324, and 432 pixels in the same density directories. `res/mipmap-anydpi-v26/ic_launcher.xml` references the density resources by Android resource name.

Optional groups are disabled by default and controlled beside the preset picker:

- Round launcher fallbacks add `ic_launcher_round.png` to every density directory.
- Android themed icon adds `ic_launcher_monochrome.png` to every density and an API 33 `ic_launcher.xml` override containing `<monochrome>`.

Conditional metadata requirements are atomic. Enabling only part of a resource group fails with `metadata-condition-incomplete`; it cannot commit a package with a misleading XML/resource mismatch.

## Preview masks and safe zone

The output review toolbar can preview Android rows as full bleed, circle, squircle, or rounded square. Foreground and monochrome rows also show the 66/108 (61.111%) safe-zone circle. Core alpha sampling emits a typed warning when visible artwork extends outside that safe zone; the preview shape and warning do not modify generated pixels.

Reference: [Android adaptive icon guidance](https://developer.android.com/develop/ui/compose/system/icon_design_adaptive).

## Play Store output

`play-store/icon-512.png` is a 512×512 final RGB PNG. Transparent master pixels are composited over white only for this final artifact; the output table and report record `final-alpha-flattened`. Export verifies RGB channel type, exact dimensions, and the 1 MiB maximum. This implements PLAN.md's stricter no-alpha requirement even though the current Play Console documentation permits a 32-bit PNG with an alpha channel.

Reference: [Play Console preview-asset requirements](https://support.google.com/googleplay/android-developer/answer/9866151).

## Independent compiler validation

`.github/workflows/android-assets.yml` generates the default multi-role preset through `imaguncula-cli`, copies the resulting `res` tree into `fixtures/platform/android`, and runs `:app:processDebugResources`. The fixture pins Android Gradle Plugin 9.3.0, Gradle 9.5.0, Java 17, compile SDK 36, and Build Tools 36.0.0. Local Rust tests prove generation, parsing, references, dimensions, channel type, and gating; only the hosted job proves acceptance by AAPT2 in the configured Android toolchain.

References: [Create app icons](https://developer.android.com/studio/write/create-app-icons) and [Android Gradle Plugin 9.3 compatibility](https://developer.android.com/build/releases/agp-9-3-0-release-notes).

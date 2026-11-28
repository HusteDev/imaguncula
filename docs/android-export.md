# Android launcher export

The `android-launcher` v1 preset produces legacy launcher PNGs, adaptive icon layers and XML, optional round and themed variants, and Play Store artwork. Each Android layer is a separate source in the project; Imaguncula never tries to split a flat master into layers.

## Source roles

| Role | Requirement | Use |
| --- | --- | --- |
| `master` | Required | Legacy launcher, optional round launcher, and Play Store artwork |
| `foreground` | Required when adaptive outputs are included | Foreground layer on a 108-dp canvas |
| `background` | Required when adaptive outputs are included | Full-bleed background layer |
| `monochrome` | Optional | Themed-icon layer for API 33 and later |

The desktop application shows a source picker for each role the selected presets use. The CLI takes one `--role ROLE=PATH` option per role. Each role is validated separately. Adaptive outputs are included by default, so a default Android export fails before writing anything if the foreground or background source is missing. To export legacy icons from the master alone, turn off every adaptive output.

## Resource tree

Legacy `ic_launcher.png` files are 48, 72, 96, 144, and 192 pixels in `res/mipmap-mdpi`, `mipmap-hdpi`, `mipmap-xhdpi`, `mipmap-xxhdpi`, and `mipmap-xxxhdpi`. Adaptive foreground and background layers are 108, 162, 216, 324, and 432 pixels in the same directories. `res/mipmap-anydpi-v26/ic_launcher.xml` references the layers by Android resource name.

Two optional groups are off by default and are turned on beside the preset picker:

- **Round launcher fallbacks** add `ic_launcher_round.png` to every density directory.
- **Android themed icon** adds `ic_launcher_monochrome.png` to every density directory and an API 33 `ic_launcher.xml` override with a `<monochrome>` layer.

A group's XML is only written when all of the outputs it references are included. If only some of them are, export fails with `metadata-condition-incomplete` rather than writing XML that points at missing files.

## Preview masks and safe zone

The output table can preview Android rows as full bleed, circle, squircle, or rounded square. Foreground and monochrome rows also show the 66/108 (61.1%) safe-zone circle, and Imaguncula warns when visible artwork extends outside it. The mask preview and the warning never change the generated pixels.

Reference: [Android adaptive icon guidance](https://developer.android.com/develop/ui/compose/system/icon_design_adaptive).

## Play Store output

`play-store/icon-512.png` is a 512×512 RGB PNG with no alpha channel. Transparent pixels in the master are composited over white for this file only, and the output table and report flag it with `final-alpha-flattened`. Export checks that the file is RGB, exactly 512×512, and no larger than 1 MiB. Google Play currently also accepts a 32-bit PNG with alpha, but Imaguncula follows the stricter no-alpha rule in PLAN.md.

Reference: [Play Console preview-asset requirements](https://support.google.com/googleplay/android-developer/answer/9866151).

## Compiler validation

The `android-assets.yml` workflow runs on every pull request and push to `main`. It exports the default Android preset with the CLI, copies the `res` tree into the fixture app in `fixtures/platform/android`, and runs `:app:processDebugResources`, so AAPT2 compiles the generated resources. The fixture uses Android Gradle Plugin 9.3.0, Gradle 9.5.0, Java 17, compile SDK 36, and Build Tools 36.0.0. Rust tests separately check generation, XML references, dimensions, channel types, and the required-source rules.

References: [Create app icons](https://developer.android.com/studio/write/create-app-icons) and [Android Gradle Plugin 9.3 compatibility](https://developer.android.com/build/releases/agp-9-3-0-release-notes).

# Imaguncula Project Plan

Status: implemented locally; public release gates pending  
Last updated: 2026-07-31

## Purpose

Imaguncula is a simple, offline desktop application that converts icon artwork into correctly sized, named, and packaged icon assets for multiple operating systems and application platforms.

The primary workflow should require one high-quality source image. Platform-specific or size-specific replacement images remain available when automatic scaling produces a poor result.

## Product Principles

1. Prefer one master source over a wall of file selectors.
2. Generate filenames, directory structures, metadata, and multi-image containers automatically.
3. Never silently upscale raster artwork.
4. Explain every generated asset: platform, use, dimensions, filename, and packaging destination.
5. Preview the actual output at its actual pixel size before export.
6. Permit per-size and per-role source overrides.
7. Keep all image processing local. No uploads or hosted service are required.
8. Treat platform support as versioned presets because vendor requirements change.
9. Do not describe every platform package as a merged icon file. Some platforms use a container; others require directories, individual images, XML, JSON, or asset catalogs.

## Core User Workflow

1. Start a new icon project.
2. Enter an application/icon name. Imaguncula derives a safe base filename but allows it to be edited.
3. Select or drag in a master source:
   - SVG is preferred.
   - PNG is accepted when it is square and at least as large as the largest requested raster output.
4. Select one or more target presets:
   - Windows desktop
   - Windows packaged application
   - macOS
   - iOS/iPadOS
   - Android
   - Web/PWA
   - Linux desktop
   - Custom outputs
5. Configure platform-specific inputs when required:
   - Android foreground, background, and optional monochrome layers
   - Web/PWA maskable artwork or safe-zone padding
   - Apple layered artwork for workflows that require Icon Composer
6. Review a generated output table.
7. Replace the source for any individual output that needs a manually simplified or optically adjusted rendition.
8. Click **Create**.
9. Select an output directory.
10. Receive:
    - one package per selected target;
    - an optional combined ZIP;
    - a generation report describing every file and any warnings.

## Output Review Table

Each generated or user-added asset row should show:

| Field | Purpose |
| --- | --- |
| Include | Enables or excludes the asset from export |
| Preview | Shows the rendered output |
| Platform | Identifies the target preset |
| Usage | Explains where the operating system or application uses the asset |
| Dimensions | Shows the exact width and height |
| Format | Shows PNG, SVG, ICO, ICNS, JSON, XML, or other output type |
| Filename | Shows the exact generated filename |
| Destination | Shows the path inside the generated package |
| Source | Identifies the master source or a per-output override |
| Status | Shows ready, warning, or error |
| Replace | Selects a custom source for this output |

The table must support:

- adding an arbitrary custom output;
- changing a custom filename and dimensions;
- filtering by platform;
- previewing against light, dark, transparent, and checkerboard backgrounds;
- resetting an override to the master source.

## Image Quality Rules

### Vector input

- Render each output independently from the SVG.
- Preserve transparency.
- Validate that external fonts, linked images, and unsupported SVG features will not disappear during export.
- Warn when the SVG has a non-square view box.

### Raster input

- Read and display the source pixel dimensions.
- Reject or explicitly flag outputs larger than the source.
- Do not enlarge unless the user deliberately enables an override for that export.
- Use a high-quality, deterministic downsampling algorithm.
- Preserve the alpha channel and embedded color profile where supported.
- Avoid repeated resizing. Every output must be generated directly from the original source.

### Small-size legibility

Automatic downscaling cannot preserve detail that does not fit into the target pixel grid. At small sizes such as 12x12, 16x16, and 20x20, the interface must:

- show the image at actual size and at an enlarged nearest-neighbor inspection scale;
- warn about features that collapse below one pixel where detection is practical;
- allow a simplified replacement image;
- retain the replacement as part of a reusable Imaguncula project file.

## Initial Platform Presets

Preset definitions should be data-driven rather than hard-coded into the interface.

### Windows desktop

Generate a multi-image `.ico` file.

Initial required sizes:

- 16x16
- 24x24
- 32x32
- 48x48
- 256x256

Allow additional standard and custom sizes. Store each image at 32-bit color with alpha where supported. Use an exact-size embedded image whenever one exists.

Reference: [Microsoft Windows app icon construction](https://learn.microsoft.com/en-us/windows/apps/design/iconography/app-icon-construction)

### Windows packaged applications

Generate the selected MSIX/UWP/WinUI PNG asset families with their current required filenames, scale variants, target-size variants, and theme variants. Keep this preset separate from the basic desktop `.ico` preset because it produces a directory of assets rather than one icon file.

Reference: [Microsoft Windows app icon construction](https://learn.microsoft.com/en-us/windows/apps/design/iconography/app-icon-construction)

### macOS

Support two practical outputs:

1. conventional `.icns` generated from the required raster sizes;
2. an Xcode-compatible macOS app icon asset catalog.

Do not claim to generate Apple's modern layered Liquid Glass `.icon` format from one flattened source. Instead, provide validated source layers and instructions for importing them into Apple's Icon Composer when that output is selected.

References:

- [Apple app icon asset catalogs](https://developer.apple.com/documentation/xcode/configuring-your-app-icon)
- [Apple Icon Composer](https://developer.apple.com/documentation/Xcode/creating-your-app-icon-using-icon-composer)

### iOS and iPadOS

Generate an Xcode-compatible `AppIcon.appiconset`, including its `Contents.json` metadata and the current App Store source image requirements.

Where current Xcode can derive variants from one 1024x1024 source, avoid manufacturing unnecessary legacy files unless the user selects a compatibility preset.

Reference: [Apple app icon asset catalogs](https://developer.apple.com/documentation/xcode/configuring-your-app-icon)

### Android

Generate:

- legacy launcher icons in `mipmap-*` density directories;
- round launcher variants where selected;
- adaptive-icon resource XML;
- foreground and background resources;
- optional monochrome/themed-icon resources;
- Google Play Store artwork.

Adaptive icon generation requires meaningful foreground and background layers. A flattened image may be used for legacy icons, but Imaguncula must not pretend it can reliably reconstruct adaptive layers from a flat image.

Reference: [Android Image Asset Studio](https://developer.android.com/studio/write/create-app-icons)

### Web and PWA

Generate a configurable subset of:

- `favicon.ico`;
- `favicon.svg`;
- exact-size PNG favicons;
- Apple touch icon;
- 192x192 PWA icon;
- 512x512 PWA icon;
- maskable PWA icon;
- `site.webmanifest` icon entries;
- HTML `<link>` snippets.

Show the PWA maskable safe zone and warn when important artwork falls outside it.

References:

- [W3C Web Application Manifest](https://www.w3.org/TR/appmanifest/)
- [WHATWG HTML icon link requirements](https://html.spec.whatwg.org/multipage/links.html)

### Linux desktop

Generate a Freedesktop-compatible `hicolor` hierarchy containing selected PNG sizes and, when provided, a scalable SVG:

```text
hicolor/
  16x16/apps/
  24x24/apps/
  32x32/apps/
  48x48/apps/
  64x64/apps/
  128x128/apps/
  256x256/apps/
  scalable/apps/
```

Reference: [Freedesktop Icon Theme Specification](https://specifications.freedesktop.org/icon-theme/latest/index.html)

### Custom outputs

Allow any number of custom rows with:

- width;
- height;
- output format;
- filename;
- usage description;
- directory inside the export;
- master-source or custom-source selection.

Custom rows make the application useful for browser extensions, game engines, internal tools, toolbar icons, and platform requirements not yet represented by a built-in preset.

## Project File

Imaguncula should save a human-readable project file, tentatively using the `.imaguncula.json` extension.

It should contain:

- schema version;
- project name and base filename;
- source paths stored relative to the project file when practical;
- source content hashes;
- selected preset identifiers and preset versions;
- enabled outputs;
- custom outputs;
- per-output overrides;
- export settings;
- warnings explicitly accepted by the user.

Do not embed large source images in the project file by default.

## Recommended Architecture

### Application shell

Use Tauri for an offline desktop application with native filesystem access and a small installer footprint.

### Frontend

Use a typed web frontend suitable for:

- drag-and-drop input;
- responsive preview grids;
- a virtualized output table;
- platform selection;
- accessible keyboard operation;
- progress and validation reporting.

Svelte with TypeScript is the initial recommendation, but this remains an implementation decision rather than a product requirement.

### Processing core

Put format validation, rasterization, resizing, package generation, and archive creation behind a platform-independent service boundary.

The core should:

- use one canonical in-memory RGBA representation for raster outputs;
- render each size from the original input;
- isolate encoders for PNG, ICO, and ICNS;
- generate JSON/XML/text metadata from versioned preset definitions;
- write to a temporary staging directory;
- validate staged output before moving it to the chosen destination;
- remove incomplete staging data after a failed or cancelled operation.

### Preset model

Each preset should define:

- stable preset ID;
- display name;
- platform;
- preset version;
- source-role requirements;
- output dimensions;
- filenames and relative destinations;
- encoder and format options;
- generated metadata files;
- validation rules;
- explanatory usage text;
- authoritative specification links;
- last-reviewed date.

Preset changes must not silently alter an existing saved project. Opening a project with an older preset should offer a clear comparison and migration.

## Security and Privacy

- Perform all processing locally.
- Do not execute SVG scripts.
- Do not permit SVGs to read arbitrary local or remote resources.
- Apply input-size, decompression, pixel-count, nesting, and memory limits.
- Treat filenames and archive paths as untrusted input.
- Prevent absolute paths and `..` traversal inside generated packages.
- Write atomically where possible.
- Never overwrite an existing export without explicit confirmation.
- Do not collect analytics in the initial version.

## Accessibility

- All functionality must be keyboard accessible.
- Do not rely on color alone for warning and error states.
- Provide text descriptions for platform usage and validation results.
- Announce generation progress and completion to assistive technology.
- Allow preview backgrounds and zoom levels to be changed without affecting output.

## Implementation Phases

### Phase 0: Repository foundation

- Choose and scaffold the Tauri/frontend stack.
- Add formatting, linting, unit-test, integration-test, and build commands.
- Establish the preset schema.
- Add architecture and contribution documentation.
- Create a minimal CI workflow for supported development platforms.

Exit criteria:

- a clean checkout can install dependencies, run tests, and build the empty application shell;
- the application can open and save a minimal project file;
- preset schema validation is automated.

### Phase 1: Source import and preview

- Import SVG and PNG files.
- Validate dimensions, shape, transparency, and unsafe SVG content.
- Render previews at requested dimensions.
- Add actual-size and magnified inspection modes.
- Add master-source replacement.

Exit criteria:

- every preview is rendered directly from the original source;
- unsupported or unsafe input fails with a specific explanation;
- raster upscaling is visibly blocked or warned according to policy.

### Phase 2: Custom PNG output

- Add the output review table.
- Add, edit, remove, enable, and disable custom rows.
- Support per-row source overrides.
- Generate correctly named PNG files into a staging directory.
- Add overwrite protection and a generation report.

Exit criteria:

- a user can create any number of custom PNG sizes;
- filenames and dimensions match the table exactly;
- cancelled or failed exports do not leave a falsely complete package.

### Phase 3: Windows and Web/PWA

- Add multi-resolution ICO encoding.
- Add the Windows desktop preset.
- Add favicon and PWA presets.
- Generate manifest and HTML snippets.
- Add maskable safe-zone visualization.

Exit criteria:

- generated ICO entries can be enumerated and decoded at every selected size;
- favicon and PWA files match their declared dimensions and MIME types;
- generated manifest and HTML snippets reference files that exist.

### Phase 4: Linux and Apple conventional assets

- Add the Linux `hicolor` package.
- Add Xcode asset catalog generation.
- Add conventional ICNS generation.
- Validate `Contents.json` against the files produced.

Exit criteria:

- generated Linux paths conform to the preset;
- Xcode metadata contains no missing or unreferenced generated icon files;
- ICNS entries decode at all selected sizes.

### Phase 5: Android

- Add legacy density resources.
- Add foreground/background/monochrome role inputs.
- Generate adaptive-icon XML and resource directories.
- Add mask previews for common launcher shapes.

Exit criteria:

- the generated resource tree builds in a minimal Android fixture project;
- adaptive exports cannot proceed without required layer roles;
- legacy-only export remains available from a flat source.

### Phase 6: Windows packaged assets and preset maintenance

- Add current Windows packaged-app asset families.
- Add theme and target-size variants.
- Add preset version comparison and migration.
- Add a documented process for reviewing vendor requirements.

Exit criteria:

- a fixture packaged application accepts the generated assets;
- preset upgrades show their effects before changing a saved project;
- every built-in preset records its specification source and review date.

### Phase 7: Release hardening

- Add cross-platform packaging and signing documentation.
- Test on Windows, macOS, and Linux.
- Add corrupted, oversized, and hostile input fixtures.
- Add deterministic-output tests where metadata permits.
- Produce user documentation and example projects.

Exit criteria:

- supported installer packages pass clean-machine smoke tests;
- hostile inputs cannot escape the staging directory or trigger unbounded processing;
- example outputs pass platform-specific fixture builds or validation tools.

## Testing Strategy

### Unit tests

- preset schema and migration;
- filename sanitization;
- dimension calculations;
- no-upscale rules;
- safe-zone calculations;
- SVG sanitization;
- archive-path validation;
- metadata generation;
- format encoder entry tables.

### Golden-file tests

Use small, intentionally designed source fixtures to compare:

- PNG dimensions and decoded pixels;
- ICO and ICNS entry lists;
- generated directory trees;
- JSON and XML metadata;
- reports and warnings.

Avoid brittle byte-for-byte checks when encoders include nondeterministic metadata. Decode and compare semantic content instead.

### Integration tests

- complete export for every preset;
- combined multi-platform export;
- cancellation;
- output collision;
- missing source override;
- moved project directory;
- opening an older preset version;
- handling a changed or deleted source file.

### Platform fixture tests

Where practical, generate minimal fixture applications and verify that:

- Windows loads the ICO and packaged assets;
- Xcode recognizes the asset catalog;
- Android resource compilation succeeds;
- web manifests reference valid assets;
- Linux files occupy specification-compliant paths.

## Initial Non-Goals

- Editing or drawing icon artwork inside Imaguncula.
- Automatically creating a good logo from a text prompt.
- Reconstructing layered Android or Apple artwork from a flattened image.
- Guaranteeing that one detailed design remains legible at every small size.
- Uploading assets to app stores.
- Modifying a user's application project without an explicit import/export workflow.
- Replacing Apple Icon Composer or other vendor-specific design tools.
- Supporting every historical platform and obsolete icon format in the first release.

## Open Decisions

Resolve these during Phase 0:

1. Exact frontend framework and package manager.
2. Rust image, SVG, ICO, ICNS, and ZIP libraries.
3. Whether SVG remains in memory or is converted into a sanitized intermediate representation.
4. Default platform and size selection for a new project.
5. Combined ZIP naming and directory layout.
6. Whether the first public build targets Windows only or all desktop operating systems.
7. Whether preset definitions can be updated independently of application releases.
8. Whether conventional ICNS generation is included in the first release or deferred until after Windows/Web output.

## Definition of the First Useful Release

The first useful release is complete when a user can:

1. install and run Imaguncula locally;
2. import one SVG or sufficiently large PNG;
3. see source-quality warnings;
4. generate and preview standard Windows desktop and Web/PWA outputs;
5. add arbitrary custom PNG sizes and filenames;
6. replace an individual small-size rendition;
7. export a valid multi-resolution ICO and a correctly structured web asset package;
8. save and reopen the Imaguncula project;
9. receive an accurate report of generated files and unresolved warnings.

## Restart Point

When work resumes:

1. Revalidate the linked vendor specifications because platform requirements can change.
2. Resolve the Phase 0 open decisions.
3. Scaffold the application and test infrastructure.
4. Implement the versioned preset schema before implementing platform-specific generators.
5. Complete Phase 1 and Phase 2 before expanding the number of built-in platform presets.

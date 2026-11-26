# User guide

## Create or open a project

Use **New** for an empty schema-v2 project or **Open** for a `.imaguncula.json` file. New projects start with Windows desktop and Web/PWA presets. Project files store relative source paths when possible, SHA-256 source identities, exact preset versions, output selections, overrides, warning acceptances, and reviewed preset migrations.

Project files are portable, so treat one you did not write as untrusted. When an opened project references sources outside its own folder, the application lists them and asks before reading any of them; declining removes the references so trusted sources can be relinked by hand. Warnings and upscale authorizations pre-accepted inside the file are announced when the project opens.

## Select source artwork

Select or drop one SVG or PNG master. SVG is preferred because every requested size is rendered directly from the original vector tree. PNG sources can be downsampled; an output larger than its raster source remains blocked until that exact output receives an explicit upscale override.

The source panel reports dimensions, transparency, color-profile information, validation warnings, and a content hash. The preview controls render an exact target size from the original source and can magnify pixels without changing output dimensions.

Layered presets expose additional required roles. Android adaptive icons require foreground and background artwork. Selecting only a master does not synthesize those layers. Platform preset cards identify required and optional additional sources before selection. Each additional-source card identifies the selected output types that use that role and whether the role is required or optional for each one.

## Review outputs

Enable platform presets, optional preset groups, or custom outputs. The output table shows each file's platform, purpose, dimensions, format, destination, source, and readiness. A notice above the table summarizes included outputs that still need non-master artwork. Use the platform filter to constrain the table. Android outputs also expose launcher-mask previews.

Per-output replacement is the escape hatch for artwork that does not survive a small target size or platform-specific treatment. The replacement is independently validated and stored by path and content hash.

## Export

Choose **Create**, select a parent directory, and confirm replacement if the final package already exists. Replacement only applies to a directory containing a previous package's `report.md`; any other non-empty directory is refused so unrelated data is never deleted. Imaguncula writes into a sibling staging directory, verifies the complete package, and renames it into place. Cancellation and failure remove staging and do not expose a partial final package; if a confirmed replacement fails and the previous package cannot be moved back, the staging directory holding it is kept and its location is reported.

Each package contains `report.md`. Combined ZIP creation is enabled by default. The report records generated paths, dimensions, source provenance, content hashes, and accepted or outstanding warnings. After export, **Show report** expands a rendered report in the application; the raw Markdown source is not shown.

## Preset updates

Saved projects remain pinned to their exact preset versions. When a newer bundled version exists, the preset-update panel shows added, removed, and explicitly renamed output identities plus a deterministic review fingerprint. Nothing migrates until **Accept and migrate** is used. Acceptance is recorded in the project file and marks it unsaved.

## Upgrading Imaguncula

Install a new release over the old one. A new release opens every project file saved by an earlier 1.x version. Schema version 1 project files migrate to schema version 2 when opened; the project is marked unsaved, and the original file is untouched until you choose **Save**. Pinned preset versions never change during an upgrade; review and accept newer preset versions as described in [Preset updates](#preset-updates). The [changelog](../CHANGELOG.md) lists every change that affects project files, presets, or the CLI, with any steps an upgrade requires. Security fixes go into the current release line, so stay on the latest release.

## Command line

The headless CLI exposes the same core exporter and needs no display server. See [cli.md](cli.md), the generated [CLI reference](cli-reference.md), and the checked-in [example projects](../examples/README.md).

# User guide

## Install

Download the desktop application from [GitHub Releases](https://github.com/HusteDev/imaguncula/releases): the `x64-setup.exe` installer or the portable ZIP for Windows, the DMG for Apple silicon Macs, or the AppImage for x64 Linux. The downloads are not yet code-signed, so Windows SmartScreen may warn before the installer runs and macOS Gatekeeper may block the application until you allow it in System Settings. [distribution.md](distribution.md#verifying-a-release) explains how to check that a download came from this repository.

## Create or open a project

Use **New** to start an empty project or **Open** to load a `.imaguncula.json` file. A new project selects the Windows desktop and Web/PWA presets. The project file stores source paths (relative to the project file when the source is inside its folder), each source's SHA-256 hash, the exact preset versions, which outputs are included, per-output overrides, accepted warnings, and accepted preset migrations.

Treat a project file you did not create as untrusted. If it references sources outside its own folder, Imaguncula lists them and asks before reading any of them. Declining removes those references so you can relink trusted files yourself. If the file already accepts warnings or allows upscaling, Imaguncula tells you when the project opens.

## Select source artwork

Select or drop one SVG or PNG as the master source. SVG is preferred because every size is rendered from the original vector artwork. A PNG source can be scaled down, but an output larger than the PNG stays blocked until you allow upscaling for that specific output.

The source panel shows the dimensions, whether the image has transparency, any color profile, validation warnings, and the content hash. The preview renders the exact target size from the original source and can magnify pixels without changing the output.

Some presets need more than one source. Android adaptive icons need separate foreground and background artwork, and Imaguncula does not split a single master into layers. Each preset card lists the additional sources it needs, and each source card lists which selected outputs use that source and whether it is required or optional for each.

## Review outputs

Turn on platform presets, optional preset groups, or custom outputs. The output table lists each file's platform, purpose, dimensions, format, destination, source, and status. A notice above the table counts included outputs that still need artwork other than the master. Use the platform filter to narrow the table. Android rows can also preview launcher mask shapes.

If your artwork does not work at a small size or needs a platform-specific variant, use **Replace source** on that output to give it its own file. The replacement is validated like any other source and stored by path and content hash.

## Export

Choose **Create**, pick a parent folder, and confirm replacement if a package with the same name already exists there. Imaguncula only replaces a folder that contains a previous package's `report.md`; it refuses any other non-empty folder, so it never deletes files it did not write. The package is built in a staging folder next to the destination, verified, and then renamed into place. If you cancel or the export fails, the staging folder is removed and no partial package appears. If a confirmed replacement fails and the previous package cannot be moved back, the staging folder that holds it is kept and its location is shown.

Every package contains `report.md`, which lists each generated file with its dimensions, its source file name and hash, and any accepted or outstanding warnings. **Include combined ZIP** is on by default and adds a ZIP of the whole package. After an export, **Show report** displays the report in the application.

## Preset updates

A saved project stays on the exact preset versions it was created with. When a newer bundled version exists, the preset-update panel lists the outputs that version adds, removes, or renames, together with a review fingerprint. Nothing changes until you choose **Accept and migrate**. The migration is recorded in the project file and the project is marked unsaved.

## Upgrading Imaguncula

Install the new release over the old one. A new release opens every project file saved by an earlier 1.x version. Schema version 1 project files are migrated to schema version 2 when opened; the project is marked unsaved and the original file is unchanged until you choose **Save**. Upgrading never changes a project's pinned preset versions; review and accept newer presets as described in [Preset updates](#preset-updates). The [changelog](../CHANGELOG.md) lists every change that affects project files, presets, or the CLI, along with any steps an upgrade requires. Security fixes go into the current release line, so stay on the latest release.

## Command line

The `imaguncula` CLI uses the same exporter as the desktop application and does not need a display. See [cli.md](cli.md), the generated [CLI reference](cli-reference.md), and the [example projects](../examples/README.md).

# Imaguncula

Imaguncula is an offline desktop application that turns one high-quality SVG or PNG into correctly sized, named, and packaged icon assets for application platforms.

The v1 implementation is complete locally. The desktop application and headless CLI share one exporter for multi-resolution Windows ICO, Windows MSIX/UWP resource families, Web/PWA assets and metadata, Linux hicolor trees, Xcode AppIcon catalogs, conventional ICNS, Icon Composer source packages, Android legacy/adaptive/themed resources, custom PNG rows, deterministic combined ZIPs, reports, progress, and transactional cancellation-safe commits. Saved projects pin immutable preset versions and expose an explicit diff-and-accept migration path. Hosted release, signing, notarization, package-channel, and manual accessibility gates remain pending until the repository is published.

![Imaguncula production frontend showing project identity, source selection, and output review](docs/images/imaguncula-main.png)

The screenshot is the production frontend in preview mode. The Tauri runtime supplies native dialogs, source processing, presets, previews, and export authority.

## Product invariants

- Render every output size directly from the original source.
- Never silently upscale raster artwork.
- Keep image processing local.
- Explain every output's dimensions, purpose, filename, and package destination.
- Keep platform presets data-driven and versioned.

See [PLAN.md](PLAN.md) for authoritative product requirements and [EXECUTION-PLAN.md](EXECUTION-PLAN.md) for engineering order.

## Use

See the [user guide](docs/user-guide.md) or open one of the three checked-in [example projects](examples/README.md). The headless path is documented in [docs/cli.md](docs/cli.md) and its [generated command reference](docs/cli-reference.md).

## Development

From a clean checkout, run the bootstrap for the host operating system.

Windows PowerShell:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\bootstrap.ps1
```

macOS or Debian/Ubuntu:

```bash
bash ./scripts/bootstrap.sh
```

The scripts announce every installation, install only missing pinned tools and native prerequisites, then run the standalone doctor. Use `--dry-run` to preview changes or `--yes` for unattended execution. Open a new shell after a first-time install so user-level PATH changes are active. Existing environments can run `scripts/doctor.ps1`, `scripts/doctor.sh`, or:

```powershell
just doctor
```

Start the application after the doctor passes:

```powershell
just run
```

All development and CI gates use the commands in `justfile`. Run `just ci` before submitting a change.

The `.devcontainer` definition provides the pinned frontend and Rust toolchains plus Linux Tauri build dependencies for Codespaces and Dev Containers. It runs the doctor after creation. Use it for core, frontend, test, and build work; launch the desktop GUI from a native host checkout when a display server is required.

## Architecture

- `crates/imaguncula-core`: pure processing logic; no Tauri or UI dependencies.
- `crates/imaguncula-presets`: versioned preset data and schema support.
- `crates/imaguncula-cli`: installable headless interface over the core.
- `src-tauri`: thin desktop IPC adapter.
- `src`: Svelte interface.

See [docs/project-files.md](docs/project-files.md), [docs/presets.md](docs/presets.md), [docs/preset-review-process.md](docs/preset-review-process.md), [docs/source-import.md](docs/source-import.md), [docs/custom-outputs-and-export.md](docs/custom-outputs-and-export.md), [docs/windows-web-pwa-export.md](docs/windows-web-pwa-export.md), [docs/windows-msix-export.md](docs/windows-msix-export.md), [docs/linux-apple-export.md](docs/linux-apple-export.md), [docs/android-export.md](docs/android-export.md), [docs/determinism.md](docs/determinism.md), [docs/security-limits.md](docs/security-limits.md), [docs/accessibility-audit.md](docs/accessibility-audit.md), [docs/distribution.md](docs/distribution.md), and [docs/public-launch.md](docs/public-launch.md) for implemented formats, safety boundaries, release gates, and exact repository publication settings.

## License

Licensed under either the Apache License, Version 2.0 or the MIT License, at your option.

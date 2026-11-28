# Windows packaged-app export

The `windows-msix` preset generates the PNG resources an MSIX or UWP manifest references. Version 2 is current. Version 1 is still included so projects saved with it export the same files and can review the changes before migrating.

## Default files

By default the preset writes 57 files to `windows-msix/msix/Assets`:

- 14 `AppList.targetsize-*` images at 16, 20, 24, 30, 32, 36, 40, 48, 60, 64, 72, 80, 96, and 256 pixels;
- an `_altform-unplated` and an `_altform-lightunplated` copy of each, for dark and light surfaces;
- five `AppList.scale-*` images for `Square44x44Logo`;
- five `MedTile.scale-*` images for `Square150x150Logo`;
- five `StoreLogo.scale-*` images for the package logo.

In the manifest, reference `Assets\AppList.png`, `Assets\MedTile.png`, and `Assets\StoreLogo.png`; Windows picks the matching scale or target-size file. Optional groups, all off by default, add Windows 10 tile sizes and their light-theme variants, Windows 10 light-theme app-list images, splash screens and their theme variants, lock-screen badges, and light-theme Store logos. Version 2 also adds an optional group with exact 250% and 300% scale images.

Every row starts with the master source. For theme-specific or non-square artwork, replace the source on the rows that need it. PNG sources are never scaled up without permission, so a PNG master must be at least as large as the largest included output.

## Validation

Rust integration tests decode a sample of the default and intermediate-scale PNGs and check their paths and dimensions. The `windows-msix-assets.yml` workflow runs whenever the CLI, core, presets, or fixture change. It exports the preset with the CLI, puts the assets in a package folder with the fixture manifest from `fixtures/platform/windows-msix`, indexes the resources with `MakePri.exe`, and runs `MakeAppx.exe pack` with semantic validation turned on, which checks that the manifest's references resolve. It does not cover Store certification, signing, installation, or how the icons look on each Windows surface.

References:

- [Windows app icon construction](https://learn.microsoft.com/en-us/windows/apps/design/iconography/app-icon-construction)
- [uap:VisualElements manifest schema](https://learn.microsoft.com/en-us/uwp/schemas/appxpackage/uapmanifestschema/element-uap-visualelements)
- [MakeAppx packaging and semantic validation](https://learn.microsoft.com/en-us/windows/msix/package/create-app-package-with-makeappx-tool)

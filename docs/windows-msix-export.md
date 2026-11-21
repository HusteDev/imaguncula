# Windows packaged-app export

The `windows-msix` preset generates PNG resource families for MSIX/UWP manifests. Version 2 is current. Version 1 remains bundled so saved projects stay reproducible and can review an explicit v1-to-v2 comparison.

## Default asset family

The default set contains 57 files under `windows-msix/msix/Assets`:

- 14 exact `AppList.targetsize-*` sizes from 16 through 256 pixels;
- matching `_altform-unplated` and `_altform-lightunplated` files for dark and light surfaces;
- five `AppList.scale-*` Square44x44Logo resources;
- five `MedTile.scale-*` Square150x150Logo resources;
- five `StoreLogo.scale-*` package resources.

The unqualified manifest references are `Assets\AppList.png`, `Assets\MedTile.png`, and `Assets\StoreLogo.png`. Windows resolves the qualified siblings. Version 2 adds an opt-in group for exact 250% and 300% resources. Other opt-in groups cover Windows 10 tile families, light-theme variants, splash screens, lock-screen badges, and Store light-theme assets.

One master source feeds every row initially. Theme-specific and non-square compositions should use per-output source replacement. Raster sources never upscale silently; the largest enabled output determines the minimum practical master size.

## Validation

Core integration tests decode representative default and intermediate-scale PNGs and assert exact paths and dimensions. The Windows workflow generates through the production CLI, copies the asset directory into `fixtures/platform/windows-msix`, and runs `MakeAppx.exe pack` without `/nv`. This checks manifest references and performs MakeAppx semantic validation. It does not prove Store certification, signing, installation, or visual quality on every Windows surface.

Vendor basis:

- [Windows app icon construction](https://learn.microsoft.com/en-us/windows/apps/design/iconography/app-icon-construction)
- [uap:VisualElements manifest schema](https://learn.microsoft.com/en-us/uwp/schemas/appxpackage/uapmanifestschema/element-uap-visualelements)
- [MakeAppx packaging and semantic validation](https://learn.microsoft.com/en-us/windows/msix/package/create-app-package-with-makeappx-tool)

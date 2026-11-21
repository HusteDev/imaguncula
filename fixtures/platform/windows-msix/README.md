# Windows MSIX asset fixture

The Windows CI workflow copies Imaguncula's generated `msix/Assets` directory into this fixture and adds a known Windows SDK executable. `MakePri.exe` indexes the generated scale, target-size, and alternate-form variants into the package resource index. `MakeAppx.exe` then validates the manifest and builds the fixture package. The manifest intentionally references resource-family base names so the validation fails when a required generated family cannot be resolved.

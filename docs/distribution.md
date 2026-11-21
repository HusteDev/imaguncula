# Packaging, signing, and distribution

Status: automation implemented locally; hosted builds, credentials, signing, notarization, and channel submissions pending

## Release artifacts

Tagging `v<workspace-version>` runs `.github/workflows/release.yml`. The tag must match the Cargo workspace version. Clean hosted runners build and smoke-test:

| Platform | Desktop artifact | CLI artifact | Clean-runner smoke |
| --- | --- | --- | --- |
| Windows x64 | MSI, NSIS setup, portable ZIP | `imaguncula-cli-<version>-x86_64-pc-windows-msvc.tar.gz` | CLI export; portable launch; MSI install, launch, and uninstall; NSIS install and launch |

Pre-release versions skip the Windows MSI (and its smoke steps) because Tauri's MSI target rejects non-numeric SemVer pre-release identifiers; pre-release Windows builds ship NSIS and the portable ZIP only.
| Linux x64 | AppImage | `imaguncula-cli-<version>-x86_64-unknown-linux-gnu.tar.gz` | CLI export and bounded AppImage launch under Xvfb |
| macOS runner architecture | DMG | target-qualified `.tar.gz` | CLI export, DMG mount, application launch |

Every platform artifact set includes SHA-256 sums. The release job uploads only artifacts that passed the clean-runner smoke matrix. The CLI archives match the explicit cargo-binstall metadata in `crates/imaguncula-cli/Cargo.toml`.

## Trust state

The checked-in workflow builds unsigned artifacts until repository environments receive real signing identities. Unsigned Windows downloads can trigger SmartScreen. Unsigned or unnotarized macOS downloads can be blocked by Gatekeeper. Linux AppImage signatures are optional and are not automatically verified by AppImage itself. Never describe an unsigned artifact as signed, trusted, or notarized.

### Windows

Current Tauri supports Windows certificate-store signing, custom sign commands, Azure Key Vault, and Azure Artifact Signing. Azure Artifact Signing is the preferred CI direction. Configure a repository environment with the Azure tenant/client credentials, account endpoint, account name, and certificate profile; provide a `bundle.windows.signCommand` override using `artifact-signing-cli`; timestamp the signature; then verify both the executable and installer with `signtool verify /pa /all /v` before upload.

MSIX packages require a trusted signature whose subject matches the manifest publisher. Imaguncula's current desktop release targets are MSI and NSIS; the MSIX fixture workflow validates generated asset families and manifest packaging separately.

Sources: [Tauri Windows signing](https://v2.tauri.app/distribute/sign/windows/), [Tauri Windows installers](https://v2.tauri.app/distribute/windows-installer/), and [Microsoft MSIX signing](https://learn.microsoft.com/en-us/windows/msix/package/signing-package-overview).

### macOS

Direct DMG distribution requires a paid Apple Developer account, a **Developer ID Application** identity, hardened-runtime signing, and notarization. In CI, import the base64 P12 from `APPLE_CERTIFICATE` with `APPLE_CERTIFICATE_PASSWORD`, select it through `APPLE_SIGNING_IDENTITY`, provide `APPLE_ID`, `APPLE_PASSWORD`, and `APPLE_TEAM_ID` for notarization, submit with `notarytool`, wait for acceptance, and staple the ticket. `altool` is obsolete for notarization. Verify with `codesign --verify --deep --strict --verbose=2` and `spctl --assess --type execute --verbose=4` before publishing.

Sources: [Tauri macOS signing](https://v2.tauri.app/distribute/sign/macos/) and [Apple notarization](https://developer.apple.com/documentation/security/notarizing-macos-software-before-distribution).

### Linux

Build the AppImage on the oldest supported compatible base image; the release workflow uses Ubuntu 22.04. Optional embedded GPG signing uses `SIGN=1`, `SIGN_KEY`, `APPIMAGETOOL_SIGN_PASSPHRASE`, and `APPIMAGETOOL_FORCE_SIGN=1`. Publish SHA-256 sums regardless of embedded signing.

Sources: [Tauri AppImage distribution](https://v2.tauri.app/distribute/appimage/) and [Tauri Linux signing](https://v2.tauri.app/distribute/sign/linux/).

## crates.io and cargo-binstall

The release workflow publishes `imaguncula-presets`, then `imaguncula-core`, then `imaguncula-cli`, because registry dependencies must exist before their dependents. Release preparation verifies the independent preset archive and reviews the package file list for all three crates. On the first release, crates.io cannot package a dependent until its exact upstream version has propagated; each `cargo publish` therefore performs the dependent package validation in sequence. Configure the protected `crates-io` environment with `CRATES_IO_TOKEN`; move to crates.io trusted publishing when the repository/account supports the required OIDC configuration.

After publication:

```text
cargo install imaguncula-cli --locked
cargo binstall imaguncula-cli
```

## Winget and Homebrew

`.github/workflows/distribution.yml` is an explicit post-release workflow. It refuses a version without an existing GitHub release. Its Winget job resolves the published NSIS asset, generates an updated `HusteDev.Imaguncula` manifest with Microsoft's `wingetcreate`, uploads the generated manifest for review, and submits only when the operator selects `submit` and the protected `winget` environment supplies `WINGET_CREATE_GITHUB_TOKEN`.

The first Winget manifest is a one-time reviewed submission; update automation applies after Microsoft merges that package identity. Do not run the update job before the initial manifest exists.

The Homebrew job hashes the immutable GitHub source tag, generates a Rust source-build formula, runs `brew style`, builds and installs it from source, executes its formula test, uploads it for review, and pushes it to `HusteDev/homebrew-tap` only when `submit` is selected and `HOMEBREW_TAP_TOKEN` is present in the protected `homebrew` environment. A desktop cask remains blocked until the macOS DMG is signed and notarized.

Flatpak and AUR are documented community-maintained targets. The project does not promise first-party publication, review cadence, or signing for those channels. Community recipes must download immutable releases, verify checksums, preserve offline runtime behavior, and identify themselves as independently maintained.

Sources: [WingetCreate](https://github.com/microsoft/winget-create), [cargo-binstall support](https://github.com/cargo-bins/cargo-binstall/blob/main/SUPPORT.md), and [Cargo publishing](https://doc.rust-lang.org/cargo/reference/publishing.html).

# Release checklist

Copy this list into a release issue for each version and tick items there, not in this file. Tagging and publishing change public state, so they happen only when the maintainer decides to release; preparing a version locally does not imply either.

## Version and changelog

- [ ] The workspace, CLI, desktop, Tauri, and frontend versions all match the release version, and `Cargo.lock` is updated.
- [ ] `CHANGELOG.md` has a section for the version with its date, and the Unreleased section is empty.
- [ ] The generated CLI reference is current (`just lint` checks this).

## Before tagging

- [ ] `just ci` passes on the release commit with the pinned Node.js, pnpm, Rust, and just versions.
- [ ] The release workflow has been run manually on `main` (Actions, Release, Run workflow) as a dry run. The publish jobs skip on branches, so this exercises the build and smoke jobs without publishing.
- [ ] On the release commit, these hosted checks pass: CI on all three operating systems, coverage, Repeatable build, Desktop E2E (WebDriver smoke and axe), and the Apple `actool`, Android AAPT2, and Windows MakeAppx fixture workflows.
- [ ] The most recent Fuzz smoke run passed. Any crashing input it found is in `fixtures/hostile/` with a fix.
- [ ] For a stable release, the manual keyboard and screen-reader pass in [accessibility-audit.md](accessibility-audit.md) is recorded in the release issue.

## Tag and publish

- [ ] Tag the release commit `v<version>` and push the tag. The release workflow builds every artifact, runs the clean-runner smoke jobs (portable ZIP, NSIS, MSI for stable versions, AppImage, DMG, and CLI), and publishes the GitHub Release. Versions with a pre-release suffix are marked as pre-releases and have no MSI.
- [ ] Every uploaded file has a SHA-256 entry and an attestation that `gh attestation verify` accepts.
- [ ] The release notes say plainly that the Windows and macOS files are not code-signed or notarized, until they are.
- [ ] For a stable release, the three crates published in order and `cargo binstall imaguncula-cli@<version>` installs a working CLI. Pre-releases skip crates.io; see [distribution.md](distribution.md#cratesio-and-cargo-binstall).
- [ ] For a stable release, decide whether to run the Winget and Homebrew jobs in [distribution.md](distribution.md#winget-and-homebrew).

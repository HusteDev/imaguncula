# v1 release checklist

Status: local implementation complete; hosted and external publication gates pending

## Repository and version

- [x] Workspace, CLI, desktop, Tauri, and frontend versions agree on the release version (currently `1.0.0-beta.1`).
- [x] `CHANGELOG.md` contains the v1 feature and security boundary summary.
- [x] Three example projects export through the production CLI integration test.
- [x] Generated CLI reference is current and checked by lint.
- [x] License files, contribution rules, code of conduct, security policy, and issue forms exist.
- [ ] The release commit is reviewed, committed, and tagged with the matching `v<version>` tag.

## Local quality gates

- [x] `just ci` passes from the current reviewed worktree under the pinned Node, pnpm, Rust, and just versions; rerun it from the eventual release commit before tagging.
- [x] `imaguncula-presets` packages and verifies; package file lists for all three publishable crates are reviewed.
- [x] Hostile corpus, property tests, deterministic-tree tests, stress limits, and staging-escape tests pass.
- [x] The four pinned cargo-fuzz targets complete their bounded local campaigns without a crash: 2,500 SVG, 10,000 path, 1,000 ICO, and 400 ICNS inputs.
- [x] Frontend check, lint, format, unit tests, production build, and Tauri adapter checks pass.

## Local platform packaging evidence

- [x] The production Windows MSIX asset family indexes with MakePri and packages with MakeAppx; the fixture contains all 57 required generated PNGs.
- [x] The pinned release toolchain builds the Windows MSI and NSIS bundles, and both archives pass integrity checks.
- [x] The Windows portable ZIP launches, the archived release CLI exports the expected ICO, and all four staged artifact SHA-256 entries reverify.
- [x] The release executable remains running through a launch smoke, and a non-registering MSI administrative extraction contains the expected executable without changing installer registration.
- [ ] Destructive installer smoke remains assigned to a clean runner because this workstation already has Imaguncula 0.1.0 installed.

## Hosted quality gates

- [ ] Three-OS CI matrix passes on the release commit.
- [ ] Nightly fuzz targets complete without a crash; any prior crash is promoted into the regression corpus.
- [ ] Tauri WebDriver smoke and axe audit pass against the built application.
- [ ] Apple `actool`, Android AAPT2, and Windows MakeAppx fixture validation pass.
- [ ] Clean-runner MSI, NSIS, portable ZIP, AppImage, DMG, and CLI smoke jobs pass.
- [ ] Manual keyboard and screen-reader evidence in [accessibility-audit.md](accessibility-audit.md) is recorded.

## Artifact trust and publication

- [ ] Windows artifacts are signed and verified, or the release prominently states they are unsigned.
- [ ] macOS artifact is signed, notarized, stapled, and assessed, or the release prominently states it is unsigned and unnotarized.
- [ ] SHA-256 sum files match every uploaded artifact.
- [ ] GitHub Release is created from the reviewed tag.
- [ ] `imaguncula-presets`, `imaguncula-core`, and `imaguncula-cli` publish in dependency order.
- [ ] `cargo binstall imaguncula-cli@<version>` installs and runs on every published CLI target.
- [ ] Initial Winget manifest is reviewed and submitted; later updates use the distribution workflow.
- [ ] Homebrew formula is reviewed and pushed to the tap.
- [ ] Repository description, homepage, topics, Discussions, and good-first-issue seeds are published.

Tagging and publication are external state changes. They are not implied by local version preparation and require an explicit release operation.

# Public repository launch

Status: local launch materials complete; remote repository settings and seeded issues pending

## Repository metadata

Use these exact public settings so package manifests, documentation, and search results describe the same product:

- Description: `Offline desktop and CLI icon asset generator for Windows, Web/PWA, Linux, Apple, Android, and MSIX.`
- Homepage: the canonical GitHub Release or project site after it exists; leave blank until then.
- Topics: `android`, `favicon`, `icon-generator`, `icns`, `ico`, `linux`, `macos`, `msix`, `offline-first`, `pwa`, `rust`, `svelte`, `tauri`, `windows`.
- Features: enable Issues and Discussions; do not enable the wiki until it has an owned maintenance scope.
- Merge policy: require the repository quality gates; preserve linear history; delete merged branches.

Repository settings are external state. Do not mark them complete from this file alone.

## Launch seeds

Create the five issue specifications in [good-first-issues.md](good-first-issues.md) with the `good first issue` label only after confirming each remains unimplemented on the release commit. Retain their explicit scope exclusions. Do not create placeholder issues for already completed work.

Use the checked-in bug, feature, preset, and security contact paths. Enable the checked-in Ideas Discussion form after Discussions is active. Keep security reports out of public issues.

## Launch evidence

Before changing repository visibility or announcing v1:

1. complete every required item in [release-checklist.md](release-checklist.md);
2. retain hosted workflow links for the exact release commit and tag;
3. publish the checksum and trust state beside the artifacts;
4. verify every README installation command against a clean runner;
5. confirm the screenshot and accessibility record match the shipped interface.

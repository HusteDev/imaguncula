# Repository settings

The repository at github.com/HusteDev/imaguncula was made public for the 1.0.0-beta.1 release on 2026-11-21. This page records the settings it should have, so they can be checked or restored.

## Metadata

These match the package manifests and documentation:

- Description: `Offline desktop and CLI icon asset generator for Windows, Web/PWA, Linux, Apple, Android, and MSIX.`
- Homepage: empty until there is a project site.
- Topics: `android`, `favicon`, `icon-generator`, `icns`, `ico`, `linux`, `macos`, `msix`, `offline-first`, `pwa`, `rust`, `svelte`, `tauri`, `windows`.

## Features and merging

- Issues and Discussions are on. Discussions use the Ideas form in `.github/DISCUSSION_TEMPLATE/ideas.yml`.
- Issue forms: bug report and feature request. Blank issues are off; the issue chooser links to Discussions for questions and to private vulnerability reporting.
- The wiki should stay off until someone is responsible for keeping it current. It is currently on.
- Merge commits are off, so history stays linear; squash and rebase merging are allowed, and merged branches are deleted automatically.
- A ruleset on `main` blocks deletion and force-pushes, requires a pull request, and requires CodeQL and clippy code-scanning results. See [GOVERNANCE.md](../GOVERNANCE.md).

## Good first issues

The five issue specifications in [good-first-issues.md](good-first-issues.md) have not been filed yet. Before filing one, check that it is still unimplemented on `main`, keep its scope exclusions, and add the `good first issue` label.

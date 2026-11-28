# Accessibility audit

Accessibility is checked two ways: an automated audit on every pull request, and a manual pass with a screen reader before a stable release. The manual pass has not been recorded for any release yet; 1.0.0-beta.1 shipped without it.

## Automated gate

The [Desktop E2E](../.github/workflows/e2e.yml) workflow runs on every pull request and push to `main`. It builds the real Tauri application, drives it under WebKitGTK with `tauri-driver`, and runs axe-core against the visible window with the WCAG 2 A, 2 AA, 2.1 AA, and 2.2 AA rules. It also checks:

- one `main`, banner, named navigation, and content-info landmark;
- labelled workspace regions and the semantic output table;
- accessible names for every visible button, input, select, and textarea;
- keyboard reachability and order for **New**, **Open**, and **Save**;
- keyboard activation of **Add custom output** and the resulting table.

Running axe-core against the frontend in an ordinary browser does not count; only results from the built desktop application do.

## Manual release pass

Record the operating system, screen reader and its version, the artifact hash, the tester, and the result in the release issue. Every row must pass on each supported desktop platform before a stable release.

| Sequence | Required result |
| --- | --- |
| Launch and navigate by landmarks/headings | Application name, project actions, workspaces, statuses, and footer are discoverable in a coherent order. |
| Tab and reverse-tab through the complete page | Focus remains visible, follows reading order, reaches every enabled action, and never becomes trapped. |
| Create/open/save with keyboard only | Native dialogs open from the named action and focus returns to the initiating control or logical successor. |
| Select a source and inspect warnings | Source identity, metadata, warning state, and preview alternative text are announced without relying on color. |
| Change presets and optional groups | Checked state, group name, version, and output effect are announced. |
| Review virtualized output rows | Table name, column meaning, row state, controls, errors, and source status remain understandable. |
| Start, monitor, and cancel export | Progress changes are announced without focus theft; completion and failure are reported once. |
| Trigger validation and overwrite confirmation | Error text is associated with the operation; cancellation preserves the previous package. |
| Accept a preset migration | Version change, added/removed/renamed counts, fingerprint, and dirty state are available. |
| High contrast and 200% zoom | Text and controls remain visible and operable without two-dimensional page scrolling at the minimum window size where reflow is expected. |

Use NVDA on Windows, VoiceOver on macOS, and Orca on Linux. A passing axe-core run says nothing about these rows; mark them pending until someone has actually done them.

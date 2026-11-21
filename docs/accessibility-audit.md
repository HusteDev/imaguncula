# Accessibility audit

Status: automated checks implemented; hosted WebDriver and manual assistive-technology evidence pending

## Automated gate

The Tauri WebDriver suite runs axe-core against the visible desktop surface with WCAG 2 A, 2 AA, 2.1 AA, and 2.2 AA tags. It also verifies:

- one `main`, banner, named navigation, and content-info landmark;
- labelled workspace regions and the semantic output table;
- accessible names for every visible button, input, select, and textarea;
- keyboard reachability and order for **New**, **Open**, and **Save**;
- keyboard activation of **Add custom output** and the resulting table.

The hosted job builds the actual Tauri application and drives it under WebKitGTK. A browser-only frontend result is not accepted as desktop evidence.

## Manual release pass

Record the operating system, screen reader, version, artifact hash, tester, and result in the release issue. Complete every row on each supported desktop platform before publication.

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

Recommended combinations are NVDA with Windows, VoiceOver with macOS, and Orca with the supported Linux baseline. Manual results cannot be inferred from axe-core and must remain marked pending until recorded.

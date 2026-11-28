# Good first issues

Five small, self-contained tasks for new contributors. None has been filed as an issue or implemented yet (checked 2026-11-27). Each needs a synthetic test fixture and focused tests. File one with the `good first issue` label and keep its "do not" limits.

## Add a square WebP source rejection fixture

Add a note to [source-import.md](source-import.md) that WebP sources are not supported, add a minimal `fixtures/hostile/unsupported.webp`, and add a source-import test that expects the `UnsupportedFormat` error. Do not add WebP decoding.

## Add a report path-copy action

Add a **Copy path** button, usable from the keyboard, beside a finished export report. Use the Tauri clipboard plugin with only its write-text permission, announce success in the existing status region, and add component and WebDriver tests. Do not give the webview any file-read permission.

## Add a preset-spec link checker

Write a script, run on a schedule, that collects every `specSources` URL from the presets, requests each one with a timeout and a small concurrency limit, and reports pages that fail or redirect. It must not change any `lastReviewed` date. Unit-test the URL collection and de-duplication without network access.

## Add a high-contrast screenshot fixture

Create a fixed frontend state that shows a source warning, a selected preset, an output error, and an export report, and capture a screenshot of it with Windows high-contrast mode on in a CI job. Do not lower the existing color contrast to make the screenshot match.

## Add a Linux desktop-entry semantic check

Extend the release workflow's Linux smoke job to extract the AppImage and check the desktop entry's name, executable, categories, and icon name, and that the icon file is bundled. The check must not need a display server.

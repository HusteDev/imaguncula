# Good first issue seeds

These are bounded issue specifications for repository publication. Each requires a synthetic fixture and focused tests.

## Add a square WebP source rejection fixture

Document that WebP is unsupported in v1, add a minimal `fixtures/hostile/unsupported.webp`, and extend source-import regression coverage to assert the typed unknown-format error. Do not add WebP decoding.

## Add a report path-copy action

Add a keyboard-accessible **Copy path** control beside a completed generation report. Use the narrow Tauri clipboard permission, announce success through the existing status region, and add component and WebDriver coverage. Do not expose arbitrary file reads.

## Add a preset-spec link checker

Create a scheduled script that extracts every `specSources` HTTPS URL, checks status with bounded concurrency and timeouts, and reports drift without rewriting preset review dates. Unit-test parsing and duplicate suppression without network access.

## Add a high-contrast screenshot fixture

Create a deterministic frontend fixture state covering a source warning, preset selection, output error, and export report. Capture it at Windows high-contrast settings in hosted UI evidence. Do not weaken existing color contrast to match the fixture.

## Add a Linux desktop-entry semantic check

Extend release validation to extract the AppImage and assert the desktop entry name, executable, categories, icon name, and bundled icon presence. Keep the check independent of launching a display server.

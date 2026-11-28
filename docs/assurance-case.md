# Security assurance case

This document argues why Imaguncula meets its security requirements. It covers the threat model, the trust boundaries, how secure design principles are applied, and how common implementation weaknesses are countered. The concrete limits are listed in [security-limits.md](security-limits.md); reporting and response are in [SECURITY.md](../SECURITY.md).

## Security requirements

1. **Confidentiality:** artwork, project files, and local paths never leave the user's computer.
2. **Integrity of the user's files:** Imaguncula writes only inside a destination the user selected, never deletes unrelated data, and never leaves a partial package in place of a complete one.
3. **Safe handling of untrusted input:** a crafted SVG, PNG, project file, or preset cannot run code, read or write files outside the user's choices, load external resources, or exhaust resources without bound.
4. **Integrity of releases:** published artifacts are built from the public source by checked-in automation, and users can verify where they came from.

## Threat model

Imaguncula is an offline desktop application and command-line tool. It has no server, no accounts, no network features, and no stored credentials. The relevant attackers are:

| Attacker | Capability | Goal |
| --- | --- | --- |
| Malicious artwork author | Supplies an SVG or PNG that a user opens | Code execution, file disclosure through external references, network beacons, or denial of service |
| Malicious project author | Shares an `.imaguncula.json` project file | Make Imaguncula read files outside the project, write outside the chosen destination, or silently pre-accept warnings |
| Supply-chain attacker | Compromises a dependency, CI action, or release step | Ship malicious code to users |
| Network observer | Watches or alters traffic while a user downloads a release | Substitute a tampered artifact |

Out of scope: an attacker who already controls the user's account or operating system, and physical access to the machine.

## Trust boundaries

1. **Source files to the core.** SVG and PNG bytes are untrusted until they pass the size, structure, and content checks in `imaguncula-core`.
2. **Project files to the core.** Every field of a project file is untrusted, including source paths, output names, destinations, warning acceptances, and preset pins.
3. **Webview to the Rust process.** The Svelte interface runs in a webview and reaches the system only through thirteen Tauri commands. The webview is granted only `core:default`, `dialog:allow-open`, and `dialog:allow-save`; it has no general filesystem, shell, or network permission. A strict content security policy limits it to the application's own content and the IPC channel.
4. **Core to the filesystem.** All writes pass through one path sanitizer and a staged, verified, atomic export.
5. **Source repository to released artifacts.** Releases are built on GitHub-hosted runners from a version tag, the build is repeatable, and release files carry signed provenance (starting with 1.0.0-beta.2).

## Secure design principles applied

- **Economy of mechanism:** one pure Rust processing core (`imaguncula-core`) implements every rule. The desktop IPC adapter and the CLI are thin wrappers, so the rules are enforced once and tested once.
- **Fail-safe defaults:** input that exceeds a limit or contains a disallowed construct is rejected, not repaired. Unknown project fields, unknown schema versions, and unknown preset versions fail closed. Raster upscaling and external sources require explicit approval.
- **Complete mediation:** every package-relative path passes the same platform-independent sanitizer on every host, and every export is verified in staging before it is committed.
- **Least privilege:** the webview holds only the three Tauri permissions above. GitHub workflows default to read-only tokens; write permissions are granted per job only where needed, and third-party actions in token-bearing jobs are pinned to commit SHAs.
- **Least common mechanism:** the application makes no network connections, so there is no shared service whose compromise affects all users.
- **Open design:** the source, limits, threat model, and this argument are public. Security does not depend on secrecy.
- **Separation of privilege:** changes reach `main` only through pull requests that pass code-scanning checks, and release publication runs only from version tags.
- **Psychological acceptability:** risky actions are explained in plain language and confirmed: external project sources, replacing an existing package, and pre-accepted warnings in an opened project.
- **Defense in depth:** SVG content is checked by a pre-scan, then rendered by a renderer with no image resolver and no system fonts, on a worker thread under a 30-second watchdog. Exports are staged, verified, and committed atomically, with a preserved rollback copy if a replacement fails.

## Common weaknesses countered

| Weakness (CWE) | Countermeasure |
| --- | --- |
| Memory-safety errors (CWE-119, CWE-787, CWE-416) | All project code is Rust with `unsafe_code = "forbid"` in every workspace crate. Four fuzz targets exercise SVG preflight, relative paths, ICO encoding, and ICNS encoding. |
| Path traversal (CWE-22) and link following (CWE-59) | One sanitizer rejects absolute, drive-qualified, UNC, dot, parent, empty, reserved, separator-confused, and NUL-bearing paths. Staging refuses symlinks and verifies exact membership. A traversal regression test confirms that an escape-shaped destination creates nothing. |
| Uncontrolled resource consumption (CWE-400, CWE-770) | Limits on file size, decoded pixels, decoder allocation, SVG elements, nesting, filters, path geometry, preview size, cache size, and render time. |
| Code injection through active content (CWE-94, CWE-79) | The SVG pre-scan rejects scripts, event handlers, foreign objects, embedded images, and external references. The webview's content security policy blocks remote content. |
| Server-side request forgery and data exfiltration (CWE-918) | No external resource is ever loaded; CSS resource URLs and external references are rejected before rendering. |
| Improper input validation (CWE-20) | Project files use strict deserialization that rejects unknown fields. IDs, hashes, dimensions, filenames, and destinations are validated by allowlist in the core. |
| Data loss from incomplete writes (CWE-459, CWE-367) | Project saves write a temporary file, sync it, and atomically replace the destination. Exports commit only after staging verification, and never replace a directory that lacks a previous package marker. |
| Information exposure (CWE-200) | Reports identify sources by file name and SHA-256 hash instead of absolute paths. Nothing is sent over the network. |
| CI injection and supply-chain compromise (CWE-829, CWE-1357) | Workflow inputs and secrets are passed through environment variables, not template expansion. Dependencies are locked; `cargo deny`, Dependabot, CodeQL, and clippy code scanning run in CI. Release artifacts carry signed build provenance. |
| Weak cryptography (CWE-327) | The only cryptography is SHA-256 for content hashes and checksums, provided by a maintained library. |

## Evidence

- Limits and hostile-input handling: [security-limits.md](security-limits.md) and the regression corpus in `fixtures/hostile/`.
- Property tests, stress tests, and fuzz targets: the `imaguncula-core` unit and integration tests and `fuzz/fuzz_targets`, run in CI and in the bounded fuzz workflow.
- Byte-for-byte output determinism: [determinism.md](determinism.md).
- Release provenance and verification: [distribution.md](distribution.md#verifying-a-release).
- Repeatable builds, checked on every pull request: [distribution.md](distribution.md#repeatable-builds).
- A pre-beta security review in November 2026 found no critical issues; its six findings were fixed before the 1.0.0-beta.1 release and are listed in the Security section of [CHANGELOG.md](../CHANGELOG.md).

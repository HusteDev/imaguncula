# Roadmap

This roadmap covers what Imaguncula plans to do, and not do, through the end of 2027. It is reviewed at least every six months, and whenever a release changes the plan. Ideas and requests are welcome in [Discussions](https://github.com/HusteDev/imaguncula/discussions); see [GOVERNANCE.md](GOVERNANCE.md) for how decisions are made.

## Now: the 1.0 beta

The first public beta, 1.0.0-beta.1, was released on 2026-11-21, and 1.0.0-beta.2 on 2026-11-27. During the beta the work is fixing what users report and making releases easier to trust:

- Fix bugs reported against the beta, each with a regression test.
- Keep dependencies current and free of known vulnerabilities.

Done in 1.0.0-beta.2: every release file carries a signed GitHub artifact attestation, CI fails when test coverage drops below 80%, and a CI workflow checks that building the same source twice produces byte-identical executables.

## Next: 1.0 stable (first half of 2027)

- Sign the Windows executables and installers with an operating-system code-signing identity, and sign and notarize the macOS DMG.
- Publish the CLI crates to crates.io so that `cargo install` and `cargo binstall` work.
- Publish to Winget and the Homebrew tap through the existing distribution workflow.
- Complete the hosted release gates in [docs/release-checklist.md](docs/release-checklist.md), including the manual keyboard and screen-reader audit.
- Keep the project file format at schema version 2 and the existing preset versions loadable.

## Later in 2027

- Track platform icon specification changes and ship them as new, reviewed preset versions. Existing projects stay pinned until their owners accept a migration.
- Improve small-size legibility warnings and previews.
- Work through the starter tasks listed in [docs/good-first-issues.md](docs/good-first-issues.md).

## Not planned

These are outside the project's scope for at least the next year:

- Editing or drawing icon artwork inside Imaguncula.
- Generating logos or artwork from text prompts or with AI models.
- Reconstructing layered Android or Apple artwork from a flattened image.
- Uploading assets to app stores, or changing a user's application project outside an explicit export.
- Any network feature, including telemetry, accounts, cloud sync, or automatic updates. Imaguncula stays fully offline.
- Replacing Apple Icon Composer or other vendor design tools.
- Supporting obsolete or historical icon formats beyond those already implemented.

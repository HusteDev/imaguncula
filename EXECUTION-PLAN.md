# Imaguncula Execution Plan

Status: Phases 0 through 7 are built. 1.0.0-beta.1 was published as a GitHub pre-release on 2026-11-21. This file is kept as the record of how v1 was built; planned work from here on is in [ROADMAP.md](ROADMAP.md).

Derived from: [PLAN.md](PLAN.md)

Last updated: 2026-11-27

This plan turns the product requirements in `PLAN.md` into engineering work: how to build it, in what order, and with which tools. `PLAN.md` stays authoritative. If the two documents conflict, `PLAN.md` wins and this file is corrected.

### Implementation status

The scaffold step deleted the original plan files on 2026-07-31. `PLAN.md` was restored byte-for-byte; this plan was restored from the task log, and the truncated part of §4.3–§4.7 was rebuilt from the task, milestone, and risk references elsewhere in the document.

"Hosted" below means the check has passed on GitHub Actions. Every workflow named here has run on the public repository at github.com/HusteDev/imaguncula.

| Task | State | Evidence |
| --- | --- | --- |
| 0.1 Repository foundation | Done | Dual MIT/Apache-2.0 license, README, contribution guide, and `justfile` |
| 0.2 Tauri/Svelte scaffold | Done | Imaguncula shell replaced the template demo; frontend and Tauri builds pass in CI |
| 0.3 Cargo workspace | Done | `imaguncula-core`, `imaguncula-presets`, `imaguncula-cli`, and the thin `imaguncula-desktop` adapter build and test as one workspace |
| 0.4 Library spikes | Done | SVG, PNG, Lanczos3, and ICO spike tests (`crates/imaguncula-core/tests/library_spikes.rs`) pass on Windows, macOS, and Linux in CI |
| 0.5 Preset schema | Done | Draft 2020-12 schema plus semantic cross-reference tests for files, containers, directory trees, and generated metadata; checked in CI |
| 0.6 Windows desktop preset | Done | Immutable v1 has Microsoft's minimum ICO sizes; v2 adds the observed 42 px taskbar entry without changing v1 |
| 0.7 Project files | Done | Schema v2 model with a v1 migration ladder, strict load validation, atomic save, Tauri commands, and keyboard-accessible New/Open/Save |
| 0.8 Quality gates | Done | `just ci` runs Prettier, ESLint, svelte-check, Vitest, license headers, fuzz harness compilation, rustfmt, Clippy, Rust tests, and the frontend and Tauri builds |
| 0.9 Cross-platform CI | Done (hosted) | `ci.yml` runs lint, test, and build on Windows, macOS, and Linux through the same `just` recipes, plus a coverage job that fails below 80% |
| 0.10 Architecture and decisions | Done | `docs/architecture.md`, CONTRIBUTING.md, and ADRs 0001–0011 |
| 0.11 Bootstrap scripts | Done (hosted) | `scripts/bootstrap.ps1` and `scripts/bootstrap.sh` reach a working build on Windows, macOS, and pristine Ubuntu 24.04; doctor scripts report the failing subprocess |
| 0.12 Bootstrap CI | Done (hosted) | `bootstrap.yml` runs weekly and when the scripts change, on Windows, macOS, and a pristine Ubuntu container |
| 0.13 Dev container | Built locally; Codespaces not yet tried | Image builds from scratch with checksum-verified Node, and the native-Linux doctor passes with no warnings |
| 0.14 Open-source hygiene | Done (hosted) | `security.yml` runs `cargo deny` and a full-history Gitleaks scan; conduct, security, and reporting templates are in place |
| 1.1 Source import pipeline | Done | Content classification, bounded reads, SHA-256 identity, dimensions, shape, and structured source errors in `source.rs` |
| 1.2 SVG sanitization | Done | Pre-scan limits and active-content rejection run before `usvg`; image resolvers and system fonts are disabled, and text produces a warning |
| 1.3 PNG validation and color policy | Done | Bounded full decode reports alpha and ICC state; ADR 0010 records the sRGB treatment |
| 1.4 Canonical RGBA and rendering | Done | Straight-alpha RGBA8, direct SVG rendering, alpha-aware Lanczos3, and per-request no-upscale enforcement, each with tests |
| 1.5 Preview IPC and cache | Done | Typed IPC returns sRGB-tagged PNG bytes; previews are cached by content and invalidated when a role's source is replaced |
| 1.6 Source-input UI | Done | Native file drop and a filtered picker feed a source card with replace/remove, metadata, warnings, and saved project references |
| 1.7 Preview UI | Done | Device-pixel-ratio-aware actual size, 2×–16× nearest-neighbor zoom, editable render size, and four backgrounds |
| 1.8 Hostile-input tests | Done | Script and DOCTYPE fixtures plus nesting, external-resource, truncated-PNG, dimension and pixel bomb, file-size, and 1×1 transparency tests |
| 2.1 Custom output-row model | Done | Typed custom `OutputSpec` resolution with status and issues; path sanitization is property-tested |
| 2.2 Output review table | Done | Virtualized 11-column semantic table with add, edit, remove, include, filter, and on-demand previews |
| 2.3 Per-row source override | Done | Per-row source with hash checks, reset to master, relative-path portability, and per-row upscale approval, all saved in the project |
| 2.4 Transactional export engine | Done | Same-parent staging, create-new writes, exact tree and decoded-dimension verification, atomic commit, and cleanup |
| 2.5 Overwrite protection | Done | Destination inspection, explicit confirmation, backup before commit, and restoration on failure |
| 2.6 Generation report | Done | Deterministic Markdown report and UI summary listing generated files, source hashes, and how each warning was handled |
| 2.7 Progress and cancellation | Done | One export job at a time, typed progress events, atomic cancellation with cleanup, and an ARIA live status |
| 2.8 Integration tests | Done | Multi-row, cancellation, collision, missing or changed override, overwrite, and moved-project cases |
| 2.9 Golden tests | Done | `fixtures/golden/custom-png` checks exact paths, dimensions, and decoded pixels against previews |
| 2.10 Desktop E2E and a11y #1 | Automated part done (hosted); manual pass open | `e2e.yml` drives the built app with `tauri-driver` on Linux and runs axe WCAG A/AA; the manual keyboard and screen-reader pass has not been done |
| 3.1 ICO encoder integration | Done | Multi-entry ICOs with BMP small entries and a PNG-compressed 256 entry; verification decodes every entry |
| 3.2 Preset consumption | Done | Bundled presets merge with custom rows; preset versions and optional groups are selected in the UI |
| 3.3 Windows desktop preset | Done | Fourteen Windows sizes render independently into one verified `<base>.ico` |
| 3.4 Web/PWA generators | Done | Favicon ICO and SVG, Apple touch icon, PWA any-purpose, and maskable icons |
| 3.5 Manifest and HTML metadata | Done | Templates resolve declared package references; JSON must parse and every link target must exist in staging |
| 3.6 Maskable safe zone | Done | Row preview shows the 80% circle; a sampled alpha check warns when artwork falls outside it |
| 3.7 Combined ZIP | Done | Optional deterministic Deflate archive with sorted preset-ID paths, reopened and verified byte for byte |
| 3.8 Phase 3 golden/integration tests | Done | ICO entry order, encoding, and alpha; Web PNG dimensions; SVG, JSON, and HTML validity; link targets; ZIP determinism |
| 3.9 Headless CLI | Done | `export` and `quick` use the same core entry point as the GUI, with JSON output, stable exit codes, and a GUI-equivalence test (`crates/imaguncula-cli/tests/equivalence.rs`) |
| 3.10 First-useful-release candidate | Done; no separate v0.1 | The Phase 3 exit checks pass. No v0.1 was released; the first public release was 1.0.0-beta.1 |
| 4.1 Linux hicolor preset | Done | Eight PNG sizes plus a sanitized scalable SVG in the freedesktop hicolor tree |
| 4.2 Xcode AppIcon catalogs | Done | Modern single-1024 and compatibility iOS/iPadOS modes (mutually exclusive) plus the ten-slot macOS catalog |
| 4.3 Contents cross-validation | Done | Verification requires the exact declared filenames, safe sibling paths, valid JSON, and decoded dimensions |
| 4.4 ICNS encoder | Done | Seven representations encode, reopen, decode to RGBA, and keep alpha |
| 4.5 Icon Composer path | Done | Required foreground and background roles; export writes validated 1024-pixel layers and instructions and never fakes a `.icon` file |
| 4.6 Apple platform CI | Done (hosted) | `apple-assets.yml` generates catalogs through the CLI, validates them with `plutil`, and compiles them with `actool` on macOS |
| 4.7 Phase 4 golden/integration tests | Done | Linux tree, both iOS modes, macOS catalog, Contents references, ICNS entries, role gates, and the Composer package |
| 5.1 Multi-role source model | Done | Preset-declared roles drive source management in the GUI and repeatable `--role role=path` options in the CLI; each role is imported and validated separately |
| 5.2 Legacy and round launcher assets | Done | Five density outputs from the flat master; the optional round group adds five more |
| 5.3 Adaptive and themed resources | Done | Foreground and background resources and the API 26 XML are required together; optional monochrome resources add the API 33 override; legacy-only export is supported |
| 5.4 Launcher mask previews | Done | Full bleed, circle, squircle, and rounded-square masks; adaptive layers show the 66/108 safe zone and an alpha warning |
| 5.5 Play Store artwork | Done | 512×512 output flattened over white, encoded as RGB, and checked against the 1 MiB limit |
| 5.6 Android resource CI | Done (hosted) | `android-assets.yml` feeds CLI output into a minimal AGP 9.3 fixture and runs AAPT2 `processDebugResources` |
| 5.7 Phase 5 golden/integration tests | Done | Default adaptive, missing-role denial, legacy-only, optional groups, partial-group denial, XML references, densities, and RGB store output |
| 6.1 MSIX/UWP preset | Done | 122 rows in v1 and 134 in v2: 57 required defaults plus optional target-size, unplated, theme, tile, splash, badge, Store, and intermediate-scale groups |
| 6.2 Windows packaged-app fixture | Done (hosted) | `windows-msix-assets.yml` indexes CLI output with MakePri and builds the fixture package with MakeAppx |
| 6.3 Preset comparison and acceptance | Done | Added/removed/renamed diff, deterministic fingerprint, fail-closed application, saved acceptance, and the review UI |
| 6.4 Project migration framework | Done | Schema v2 loads v1 through an explicit ladder, reports notices, keeps exact preset pins, and marks the project unsaved |
| 6.5 Preset review process | Done | `docs/preset-review-process.md`; CI requires review dates, HTTPS sources, immutable identity, and valid cross-references |
| 7.1 Hostile-input campaign | Done (hosted) | Four `cargo-fuzz` targets (SVG inspection, path sanitization, ICO, ICNS) run in the Fuzz smoke workflow every Monday and on pull requests that change the core or the fuzz targets. Its first scheduled run (2026-11-23) failed because `rust-toolchain.toml` overrode the nightly toolchain; the workflow now sets `RUSTUP_TOOLCHAIN` to the pinned nightly |
| 7.2 Determinism tests | Done (hosted) | Repeat exports, reports, containers, directory trees, and ZIPs are compared byte for byte; `repeatable-build.yml` also checks that two builds of the frontend, CLI, and desktop executable are identical on all three platforms |
| 7.3 Resource and staging limits | Done | SVG element stress stops at the limit; size, dimension, pixel, memory, time, traversal, and staging-escape cases fail before anything is written |
| 7.4 Release packaging | Done (hosted) | `release.yml` built 1.0.0-beta.1: Windows NSIS setup and portable ZIP (MSI is skipped for pre-release versions), Linux AppImage, macOS DMG, CLI archives, and SHA-256 sums. Releases after beta.1 also carry Sigstore build-provenance attestations. Artifacts are not code-signed or notarized |
| 7.5 Clean-runner smoke | Done (hosted) | The release matrix installs or mounts and launches each desktop artifact and runs a CLI export on fresh runners before upload |
| 7.6 Accessibility checkpoint | Automated part done (hosted); manual pass open | Landmark, keyboard-order, accessible-name, and WCAG 2.0/2.1/2.2 A/AA axe checks run in the desktop E2E suite; the manual keyboard and screen-reader record in `docs/accessibility-audit.md` is still empty |
| 7.7 Documentation and examples | Done | User, security, determinism, distribution, and accessibility guides; three example projects; generated CLI reference; screenshot |
| 7.8 Distribution channels | Automation written; nothing published | GitHub Releases is in use. crates.io publishing, cargo-binstall metadata, Winget manifests, and the Homebrew formula are defined in `release.yml` and `distribution.yml` but have not been run |
| 7.9 Public launch materials | Done, except starter issues | Repository is public with topics, description, and Discussions; SPDX headers, issue and discussion forms are in place. The five starter tasks in `docs/good-first-issues.md` are not yet filed as issues |
| 7.10 v1 release preparation | Beta published | 1.0.0-beta.1 released 2026-11-21; 1.0.0 stable is planned for the first half of 2027 (see ROADMAP.md) |

### Remaining work

1. Complete and record the manual keyboard and screen-reader pass on each desktop platform.
2. Obtain Windows code signing and Apple Developer ID signing and notarization, or record a decision to ship 1.0 unsigned.
3. Publish `imaguncula-cli` to crates.io and submit the Winget and Homebrew packages.
4. File the starter issues from `docs/good-first-issues.md`.
5. Try the dev container in Codespaces.

---

## 1. Resolutions for the Phase 0 Open Decisions

`PLAN.md` lists eight open decisions. All eight were resolved in Phase 0 and recorded as ADRs 0001–0008 in `docs/adr/`.

| # | Decision | Resolution | Rationale |
| --- | --- | --- | --- |
| 1 | Frontend framework and package manager | Svelte 5 with TypeScript, SvelteKit with the static adapter, Vite, and pnpm | Small runtime and good Tauri support; pnpm gives strict, fast installs. |
| 2 | Rust libraries | See §2.3 | Chosen for maintenance health, deterministic output, and no network access. |
| 3 | SVG handling | Parse once into a sanitized `usvg` tree, keep only that tree in memory, and render every output size from it | Every output still renders from the original input, and sanitization happens in one place. The original bytes stay on disk and are hashed, not re-parsed per output. |
| 4 | Default selection for a new project | Windows desktop and Web/PWA presets, fully enabled, with no custom rows | Matches the Definition of First Useful Release with the smallest useful default. |
| 5 | Combined ZIP layout | `<basefilename>-icons.zip` with one top-level directory per preset (`windows-ico/`, `web/`, `linux-hicolor/`, …) and `report.md` at the root | Predictable and collision-free. Directory names come from preset IDs, not display names. |
| 6 | First public build targets | Build and test on Windows, macOS, and Linux from Phase 0. Publish Windows installer and portable builds and a Linux AppImage; ship macOS builds unsigned with documented Gatekeeper limits until a signing identity exists | Publishing everywhere serves the open-source goal, but Apple signing needs a paid identity. 1.0.0-beta.1 shipped all three, unsigned. |
| 7 | Presets updatable independently of releases | No for v1. Presets are compiled into the binary as versioned data files, and the loader leaves room for a separate update channel later | Avoids designing an update and trust mechanism now, while still building preset versioning in Phase 0. |
| 8 | ICNS in first release | Deferred to Phase 4, after Windows and Web output | Keeps the first useful release small. |

---

## 2. Technology Stack

### 2.1 Application shell

- Tauri 2.
- The frontend is served from the bundled asset directory; release builds do not need a dev server.
- The webview has three permissions: `core:default`, `dialog:allow-open`, and `dialog:allow-save`. There is no HTTP, shell, or general filesystem permission. All file access goes through typed commands in the Rust adapter, and the content security policy blocks remote content.

### 2.2 Frontend

- Svelte 5 with TypeScript in strict mode, built as a static SvelteKit app with Vite.
- pnpm with a committed lockfile.
- The output table uses a small hand-written virtual list, since it can exceed 100 rows with every preset enabled.
- No UI component framework: plain semantic HTML and a small CSS token layer keep the accessibility requirements in PLAN.md under direct control.
- Project state is one serializable object that mirrors the project-file schema, so saving and loading are serialization rather than a separate model.

### 2.3 Rust core libraries

| Concern | Crate | Notes |
| --- | --- | --- |
| Raster decode/encode | `image`, `png` | PNG decode and encode with alpha preserved. |
| High-quality resize | `fast_image_resize` | Deterministic Lanczos3 downsampling. |
| SVG parse/sanitize/render | `usvg` + `resvg` | `usvg` produces a normalized tree that can be inspected for unsupported features, and `resvg` rasterizes it. Active content is rejected by an XML pre-scan before parsing and by the `usvg` options. |
| ICO | `ico` | PNG-compressed 256×256 entries and BMP entries where required. The Phase 3 check confirmed it can express the needed entry order. |
| ICNS | `icns` | Phase 4. |
| ZIP | `zip` | Deflate with sorted entries and fixed timestamps. |
| JSON | `serde` / `serde_json` | Project file, `Contents.json`, web manifest. |
| XML | `quick-xml` | Android adaptive-icon XML, write only. SVG is the only untrusted XML, and `usvg` parses it. |
| Hashing | `sha2` | Source content hashes. |
| Temp/staging | `tempfile` | Staging directories. |
| Schema validation | `jsonschema` | Validates preset files in tests and CI. Project files are validated by strict deserialization. |
| Color profiles | None; see ADR 0010 | V1 detects an ICC profile, warns, treats decoded values as sRGB, and tags generated PNGs as sRGB. It does not copy the source profile or claim color-managed conversion. |

These choices were validated by the Phase 0 spikes and recorded in ADR 0002.

### 2.4 Toolchain and quality gates

- Rust: toolchain pinned in `rust-toolchain.toml`; `rustfmt` and `clippy -D warnings` with the pedantic lints.
- TypeScript: ESLint, Prettier, and `svelte-check` in CI.
- Tests: `cargo test` and `proptest` for Rust; Vitest with Testing Library and jsdom for frontend units and components; `tauri-driver` for the desktop E2E suite. Coverage is measured with `cargo llvm-cov` and Vitest's V8 provider, and CI fails below 80%.
- Commands run through `just` (`setup`, `dev`, `run`, `check`, `test`, `coverage`, `repeatable-build`, `lint`, `build`, `ci`, `doctor`) so CI and contributors run the same thing.

---

## 3. Architecture and Repository Layout

### 3.1 Repository layout

```text
imaguncula/
  PLAN.md  EXECUTION-PLAN.md  ROADMAP.md  GOVERNANCE.md
  README.md  CHANGELOG.md  CONTRIBUTING.md  CODE_OF_CONDUCT.md  SECURITY.md
  LICENSE / LICENSE-MIT / LICENSE-APACHE
  justfile  rust-toolchain.toml  Cargo.toml (workspace)  deny.toml
  scripts/
    bootstrap.ps1 / bootstrap.sh   # fresh machine -> working dev environment
    doctor.ps1 / doctor.sh         # diagnose an existing environment
    check-repeatable-build.sh      # build twice and compare
  .devcontainer/                   # Dockerfile + devcontainer.json
  crates/
    imaguncula-core/               # processing core, no Tauri dependency
      src/  source.rs render.rs preview.rs output.rs preset.rs
            container.rs export.rs report.rs project.rs
      tests/  export_golden.rs library_spikes.rs
    imaguncula-presets/
      schema/preset.schema.json
      presets/<id>/<version>/preset.json
    imaguncula-cli/                # headless CLI over imaguncula-core
  src-tauri/                       # thin IPC adapter (src/lib.rs) and capabilities
  src/                             # Svelte frontend
    lib/  output.ts project.ts source.ts components/OutputTable.svelte
    routes/+page.svelte
    test-support/desktop.ts        # fake Tauri backend for component tests
  e2e/                             # tauri-driver specs
  fuzz/                            # cargo-fuzz targets
  fixtures/  golden/ hostile/ platform/ presets/ sources/
  examples/                        # three example projects
  docs/  architecture.md adr/ and the user and process guides
  .github/workflows/
```

The rule that matters: `imaguncula-core` must compile and test with no Tauri or UI dependency. All processing lives there, and `src-tauri` only adapts IPC. That keeps golden and integration tests cheap and lets `imaguncula-cli` be a thin argument parser instead of a second implementation.

### 3.2 Core data flow

```text
source file ──▶ import: validate, hash, classify (SVG|PNG)
                 │
                 ├─ SVG ▶ sanitize ▶ usvg tree (retained in memory)
                 └─ PNG ▶ decode  ▶ canonical RGBA (retained in memory)
                 │
project state (presets + custom rows + overrides)
                 │
                 ▼
resolve outputs: preset × enabled × overrides ──▶ OutputSpec list
                 │
                 ▼         (each output rendered independently from its source)
render: RGBA at exact target size ──▶ per-output validation (no-upscale, safe-zone)
                 │
                 ▼
encode: png / ico / icns / json / xml / html
                 │
                 ▼
stage: tempdir ▶ verify staged tree matches OutputSpec list ▶ atomic move ▶ report
```

Invariants enforced and unit-tested in `imaguncula-core`:

1. No render takes a previously resized bitmap as input (PLAN.md "avoid repeated resizing").
2. Upscaling raster input is a typed error unless that output carries an explicit user override.
3. Every path written inside a package comes from a sanitizer that rejects absolute paths and `..` segments.
4. An export either completes and moves into place atomically, or leaves nothing at the destination.

### 3.3 IPC command surface

Commands are coarse-grained. Rust types are mirrored by hand in the TypeScript modules under `src/lib`, and the frontend component tests exercise those mirrors against a fake backend.

- `application_name`, `new_project`, `load_project`, `save_project`
- `list_presets`, `resolve_outputs`
- `import_source`, `render_preview`
- `inspect_export_destination`, `start_export`, `cancel_export` (progress and completion arrive as Tauri events so the frontend can announce them)
- `list_preset_upgrades`, `apply_preset_upgrade`

### 3.4 Preset definition format

Presets are JSON data files validated against `preset.schema.json` in CI. Schema version 1 separates rendered outputs from how they are packaged:

```jsonc
{
  "schemaVersion": 1,
  "id": "windows-desktop",
  "version": 1,
  "displayName": "Windows desktop (.ico)",
  "platform": "windows",
  "lastReviewed": "2026-07-31",
  "specSources": ["https://learn.microsoft.com/..."],
  "sourceRoles": [
    {
      "role": "master",
      "required": true,
      "description": "Original artwork used for every output"
    }
  ],
  "outputs": [
    {
      "id": "icon-16",
      "sourceRole": "master",
      "format": "png",
      "width": 16,
      "height": 16,
      "usage": "Context menus and title bars at 100% scale",
      "encoder": { "type": "png", "compression": "best", "filter": "adaptive" }
    }
  ],
  "packages": [
    {
      "kind": "container",
      "id": "desktop-ico",
      "format": "ico",
      "path": "{base}.ico",
      "entries": [{ "output": "icon-16", "encoding": "bmp" }]
    }
  ],
  "validationRules": [
    { "rule": "square-source", "severity": "error", "sourceRole": "master" },
    { "rule": "no-upscale", "severity": "error", "sourceRole": "master" }
  ],
  "optionalOutputGroups": []
}
```

`packages[].kind` is one of:

- `files`: standalone outputs with safe relative paths;
- `container`: ordered ICO or ICNS entries with an explicit encoding per entry;
- `directory`: a package root plus output-to-relative-path mappings;
- `metadata`: JSON, XML, HTML, or text templates whose `{{reference}}` tokens point to an output in another package.

The JSON Schema rejects unknown properties, unsafe paths, invalid dates and URIs, mismatched encoders, and invalid variants. Rust tests add the uniqueness and cross-reference checks JSON Schema cannot express. A new preset version is a new data file (`presets/<id>/<version>/`); old versions are never edited, which is what makes the Phase 6 comparison and migration work.

---

## 4. Cross-Cutting Workstreams

These ran alongside the phases rather than inside one of them.

### 4.1 Security posture

- SVG: reject scripts, external references, and remote images; scan the raw XML for `<script>`, `<foreignObject>`, entity declarations, and external `href` values before parsing; cap byte size and node count.
- Raster: cap decoded pixels, file size, and memory per export job, using the `image` crate's limits plus explicit checks.
- Paths: every writer uses one property-tested sanitizer.
- Archive: the ZIP writer only receives sanitized relative paths and never writes symlinks.
- Overwrite: export will not write into an existing package directory without explicit confirmation from the UI or a CLI flag.
- Hostile fixtures were added to `fixtures/hostile/` as problems were found; Phase 7 organized them and added fuzz targets.

The resulting threat model and argument are in `docs/assurance-case.md`.

### 4.2 Accessibility

- Each UI task's acceptance criteria included its keyboard path, focus order, status shown without relying on color, and live-region announcements.
- Audit checkpoints at the end of Phase 2 and in Phase 7: axe-core in the E2E suite plus a manual keyboard and screen-reader pass. The automated part runs in CI; the manual pass is still open.

### 4.3 Documentation

- `docs/architecture.md` started in Phase 0 and is updated when a boundary changes.
- ADRs record each resolved decision and library choice.
- User documentation and example projects were Phase 7 deliverables; each phase updated the README and architecture notes it affected.
- The CLI reference is generated from the CLI's own help output, and CI fails if it is out of date.
- README setup points to the bootstrap scripts instead of keeping a second list of prerequisites.

### 4.4 Architecture decision records

ADRs live in `docs/adr/`, are numbered in order, and record context, decision, consequences, and status. An accepted ADR is not rewritten; a later ADR supersedes it and links back.

The ADRs so far:

1. frontend framework, static application shape, and package manager;
2. Rust processing libraries, from the Phase 0 spikes;
3. the sanitized SVG intermediate;
4. new-project preset defaults;
5. combined ZIP naming and layout;
6. initial build and published-artifact targets;
7. bundled preset delivery for v1;
8. ICNS phase scope;
9. project licensing;
10. PNG color-profile policy;
11. transactional directory export.

### 4.5 Open-source release path

- Dual-license under MIT or Apache-2.0 and keep package metadata in line with the license files.
- Publish contribution, conduct, security, issue, and pull-request guidance before accepting contributions. Governance and the roadmap are in GOVERNANCE.md and ROADMAP.md.
- Keep dependency locks and release provenance reviewable: builds are repeatable, and releases from 1.0.0-beta.2 on carry signed Sigstore attestations.
- Publish releases from CI with checksums and a changelog, never from a local build.
- Treat code signing and notarization as release work with documented unsigned fallbacks, not as a reason to weaken local security settings.
- Seed starter issues only once the relevant code and test harness exist.

### 4.6 Headless CLI

The CLI is a second interface over `imaguncula-core`. It does not automate the desktop app and does not have its own exporter. It shipped with the Phase 3 work.

```text
imaguncula export <project.imaguncula.json> --output <directory>
imaguncula quick <source.svg|source.png> --preset <preset-id> --output <directory>
```

The CLI:

- resolves and exports through the same core entry point as the GUI;
- uses stable non-zero exit codes for validation, source, collision, cancellation, and export failures;
- can print a machine-readable JSON report;
- never prompts, and needs explicit flags to overwrite;
- produces output equivalent to the GUI for the same project;
- needs no display server or Tauri runtime.

CLI archives ship with each release, and `cargo install --path crates/imaguncula-cli --locked` works from a checkout. Publishing to crates.io is still pending.

### 4.7 Fresh-machine bootstrap

The bootstrap scripts are onboarding documentation that can be run. Windows uses `scripts/bootstrap.ps1`; macOS and Linux use `scripts/bootstrap.sh`. Both hand diagnostics to `scripts/doctor.*`, so an existing environment can be checked without reinstalling.

Requirements:

1. One entry point per shell, with no undocumented commands to run first.
2. Safe to re-run, with pinned versions shown; the repository's toolchain files stay authoritative.
3. Shortest path to a running app: clone, run one script, then `just run`.
4. Print each step before running it; `--dry-run` previews and `--yes` runs unattended. Nothing changes the system silently.
5. Finish by running `doctor` (toolchain versions, native dependencies, a `cargo check` and `pnpm install` smoke test) and print either "ready: run `just run`" or the failing step with a fix.
6. A failure names its step, and re-running picks up where it stopped.
7. Prefer user-scoped installs, and say before anything needs elevation.

CI reuses `doctor` as the acceptance check. `bootstrap.yml` runs the real scripts weekly and whenever they change, on fresh `windows-latest` and `macos-latest` runners with caches disabled and in a pristine Ubuntu container, then checks that `doctor` passes and a debug build completes. Onboarding steps that CI does not run go stale (risk R9).

The dev container covers machines where the bootstrap can't run (locked-down machines, unsupported distributions). It provides the full toolchain for core, CLI, and frontend work; the GUI can still be run natively from the same checkout.

---

## 5. Testing Strategy

PLAN.md's test categories and where they live:

| PLAN.md category | Where it lives | Tooling | Introduced |
| --- | --- | --- | --- |
| Unit tests | `#[cfg(test)]` modules in each `imaguncula-core` source file | `cargo test`, `proptest` for path sanitization and dimension math | Phase 0 |
| Golden-file tests | `fixtures/golden/` and `crates/imaguncula-core/tests/export_golden.rs` | Semantic comparison of decoded output (dimensions, pixel hashes of decoded RGBA, entry lists, parsed JSON/XML), never raw encoded bytes | Phase 2 |
| Frontend tests | `src/**/*.test.ts` | Vitest; component tests render the real Svelte components in jsdom with Testing Library against the fake backend in `src/test-support/desktop.ts` | Phase 2; component tests added in November 2026 |
| Integration tests | `crates/imaguncula-core/src/export.rs` tests and `crates/imaguncula-cli/tests/` | Full export runs into temporary directories per preset; cancellation, collision, moved-project, stale-source, GUI/CLI equivalence, and example-project cases | Phase 2 onward |
| E2E smoke | `e2e/` | `tauri-driver` with WebDriver against the built app, plus axe-core | Phase 2, expanded in Phase 7 |
| Platform fixture tests | `fixtures/platform/` and platform workflows | Android: AAPT2 `processDebugResources` on a minimal fixture project; Xcode: `plutil` and `actool` on macOS; Windows packaged apps: MakePri and MakeAppx; Web: manifest parsing and link-target checks in the export verification step | Each platform phase |

Golden fixtures are tiny (64×64 or smaller where possible) and visually asymmetric so orientation bugs show up. The set includes a square SVG, non-square SVG, SVG with text, large PNG, minimum-size PNG, too-small PNG, PNG with an alpha gradient, and PNG with an ICC profile.

---

## 6. Phase-by-Phase Work Breakdown

The task tables below are the original plan. The status table at the top records what was actually built.

Phases match PLAN.md, and its exit criteria were binding. Effort classes: S is half a day or less, M up to 2 days, L up to a week. Estimates assume one developer; ∥ marks tasks that could run in parallel with their neighbors.

### Phase 0: Repository foundation (~3–4 weeks)

| # | Task | Effort |
| --- | --- | --- |
| 0.1 | Initialize the repository; dual MIT/Apache-2.0 license files and ADR; README stub; `justfile` skeleton | S |
| 0.2 | Scaffold Tauri 2, Svelte 5, TypeScript, and Vite with `create-tauri-app`; check `just dev` and `just build` on Windows | M |
| 0.3 | Create the Cargo workspace with `imaguncula-core` and `imaguncula-presets`; make `src-tauri` depend on core | S |
| 0.4 | Library spikes ∥: `resvg` render of a fixture SVG at three sizes; `fast_image_resize` determinism across machines; `ico` round trip; write ADRs | M |
| 0.5 | Define `preset.schema.json` for all four packaging shapes (§3.4); CI job validating every preset file | L |
| 0.6 | Write the first preset (windows-desktop v1) as a schema proof, not yet used by code | S |
| 0.7 | Project file: Rust type, serde, `schemaVersion`, load-time validation, save/load commands, and minimal New/Open/Save UI | L |
| 0.8 | Lint, format, and test gates for both languages; CI mandatory, pre-commit hook optional | M |
| 0.9 | GitHub Actions CI: {windows, macos, ubuntu} × {lint, test, build} | M |
| 0.10 | First draft of `docs/architecture.md`, CONTRIBUTING stub, and ADRs for the §1 decisions | M |
| 0.11 | Bootstrap scripts per §4.7 and the shared `doctor`; README quick start written against them | L |
| 0.12 | Clean-environment CI workflow for the bootstrap scripts (weekly and on change) | M |
| 0.13 | Dev container with the full toolchain; check it in Codespaces | M |
| 0.14 | Open-source hygiene in CI: `cargo-deny` license and advisory checks, secret scanning; CODE_OF_CONDUCT, SECURITY.md, issue and PR templates | M |

Exit check (from PLAN.md): from a clean checkout, `just setup && just ci` passes on all three operating systems; the app opens and saves a minimal project; preset schema validation is automated. Also: a fresh machine or container with no toolchains reaches a running dev app with one bootstrap script on each OS, checked by the 0.12 workflow.

### Phase 1: Source import and preview (~2–3 weeks)

| # | Task | Effort |
| --- | --- | --- |
| 1.1 | `source.rs` import pipeline: classify, probe size and shape, SHA-256 hash, `SourceInfo` with a structured error enum | M |
| 1.2 | SVG sanitization per §4.1: pre-scan and usvg options; warnings for external fonts, images, and unsupported features; non-square viewBox warning | L |
| 1.3 | PNG validation: dimensions, squareness, alpha, ICC presence; decide and record the v1 color-profile policy (risk R6) | M |
| 1.4 | Canonical RGBA type and render functions for SVG and raster, with the typed no-upscale error and per-output override | M |
| 1.5 | Preview IPC: render the requested size on demand and return PNG bytes; cache by source hash and size, invalidated on replacement | M |
| 1.6 | UI ∥: drag-and-drop and file-picker input; source card with format, dimensions, and warnings; master replacement | L |
| 1.7 | UI ∥: preview pane with actual-size rendering (device-pixel-ratio aware), 2×–16× nearest-neighbor zoom, and light/dark/transparent/checkerboard backgrounds | L |
| 1.8 | Unit tests for every validation rule; hostile fixtures: script SVG, external-entity SVG, decompression-bomb PNG, 1×1 PNG | M |

Exit check: previews render only from the original source (enforced by the cache key and a test); unsafe input fails with a specific typed explanation in the UI; upscaling is visibly blocked or warned.

### Phase 2: Custom PNG output (~3 weeks)

| # | Task | Effort |
| --- | --- | --- |
| 2.1 | `OutputSpec` model and `resolve_outputs` for custom rows; property-tested filename sanitization | M |
| 2.2 | Output review table: virtualized, all PLAN.md columns, add/edit/remove/enable/disable custom rows, platform filter | L |
| 2.3 | Per-row source override: pick a file, validate it against the row's dimensions, reset to master; saved in the project | M |
| 2.4 | Export engine: stage, render and encode each included output, verify the staged tree against the resolved list, move atomically, clean up on cancel or failure | L |
| 2.5 | Overwrite protection: destination inspection and an explicit confirmation path through IPC | S |
| 2.6 | Generation report: model and Markdown (files, dimensions, source used, accepted and outstanding warnings), written into the package and shown in the UI | M |
| 2.7 | Progress and cancellation: job handle, Tauri events, ARIA live announcements | M |
| 2.8 | Integration tests: multi-row export, mid-job cancellation, collision, missing override source, moved project | L |
| 2.9 | Golden test fixtures and decode-and-compare helpers | M |
| 2.10 | Minimal E2E smoke with tauri-driver; accessibility checkpoint 1 | M |

Exit check: custom PNG sizes export with exact names and dimensions; a cancelled or failed export never leaves something that looks like a complete package.

### Phase 3: Windows and Web/PWA (~3 weeks), the First Useful Release candidate

| # | Task | Effort |
| --- | --- | --- |
| 3.1 | ICO encoder integration (or replacement, depending on the 0.4 spike): multiple entries, PNG-compressed 256, alpha, exact-size entries | L |
| 3.2 | Preset consumption: load `imaguncula-presets` data, merge resolved outputs with custom rows in the table, enable and disable presets in the UI | L |
| 3.3 | Windows desktop preset end to end, with extra standard or custom sizes in the ICO | M |
| 3.4 | Web/PWA preset and generators: favicon.ico, favicon.svg (the sanitized SVG), exact-size PNGs, apple-touch-icon, 192/512 PWA icons, maskable icon | L |
| 3.5 | `site.webmanifest` icon entries and HTML `<link>` snippets; link targets checked during export verification | M |
| 3.6 | Maskable safe-zone overlay in preview and a warning when alpha falls outside the safe-zone circle | M |
| 3.7 | Combined ZIP per §1 decision 5 | M |
| 3.8 | Golden and integration tests: ICO entries decoded at every size, manifest and HTML references, MIME and dimension checks | L |
| 3.9 | `imaguncula-cli` (§4.6): `export` for a project file and one-shot `quick`, sharing the core export entry point; GUI/CLI equivalence test; `cargo install` works | L |
| 3.10 | Check the PLAN.md "Definition of the First Useful Release"; cut v0.1 (Windows installer and portable, Linux AppImage, CLI binaries) | M |

Exit check: ICO entries enumerate and decode at every selected size; favicon and PWA files match their declared dimensions and types; manifest and HTML snippets reference existing files; all nine First Useful Release criteria pass.

### Phase 4: Linux and Apple conventional assets (~2–3 weeks)

| # | Task | Effort |
| --- | --- | --- |
| 4.1 | Linux hicolor preset: directory-tree packaging, PNG sizes, scalable SVG when the source is SVG | M |
| 4.2 | Xcode asset catalogs: `AppIcon.appiconset` and `Contents.json` for iOS/iPadOS and macOS; single-1024 modern mode or compatibility mode | L |
| 4.3 | Cross-check `Contents.json` against the produced files during export verification | M |
| 4.4 | ICNS encoder integration, required raster sizes, and entry decode tests | L |
| 4.5 | Icon Composer path: when layered Apple output is selected, produce validated source layers and instructions instead of pretending to build `.icon` (per PLAN.md) | M |
| 4.6 | macOS CI: `actool` compile check of generated catalogs; `plutil` and schema validation | M |
| 4.7 | Golden and integration tests for all three outputs | M |

### Phase 5: Android (~3 weeks)

| # | Task | Effort |
| --- | --- | --- |
| 5.1 | Generalize source roles: foreground, background, and monochrome with per-role validation (building on 1.1's role parameter) | M |
| 5.2 | Legacy mipmap densities and round variants from the flat master | M |
| 5.3 | Adaptive-icon XML and foreground/background/monochrome resources; adaptive export is blocked without the required roles | L |
| 5.4 | Launcher mask previews (circle, squircle, rounded square) | M |
| 5.5 | Play Store artwork (512×512 PNG, no alpha). Flattening is allowed here because it is a final output, not an intermediate | S |
| 5.6 | Android fixture project and a CI resource-compile job | M |
| 5.7 | Golden and integration tests, including missing-role and legacy-only cases | M |

### Phase 6: Windows packaged assets and preset maintenance (~3 weeks)

| # | Task | Effort |
| --- | --- | --- |
| 6.1 | MSIX/UWP preset: scale, target-size, unplated, and theme variants. The largest preset by row count, and the main user of `optionalOutputGroups` | L |
| 6.2 | Packaged fixture app and `makeappx`/manifest validation in Windows CI | M |
| 6.3 | Preset version comparison: `PresetDiff` model and UI showing added, removed, and renamed outputs before a saved project migrates; acceptance is saved | L |
| 6.4 | Generalize project-file migration (`schemaVersion` ladder and preset-version pinning) | M |
| 6.5 | `docs/preset-review-process.md`, and a CI check that every preset has `specSources` and `lastReviewed` | M |

### Phase 7: Release hardening and public launch (~4–5 weeks)

| # | Task | Effort |
| --- | --- | --- |
| 7.1 | Hostile input: organize `fixtures/hostile/` and add `cargo-fuzz` targets for the SVG pre-scan, path sanitizer, and ICO/ICNS encoder inputs | L |
| 7.2 | Determinism tests: byte-identical re-export where the format allows, with documented exceptions | M |
| 7.3 | Memory and time limits under stress fixtures; staging-escape attempts | M |
| 7.4 | Packaging per §1 decision 6: Windows MSI/NSIS and portable ZIP, Linux AppImage, macOS DMG (unsigned, with documented caveats); signing and notarization docs for all three | L |
| 7.5 | Clean-machine smoke tests: install and run every published artifact (installer, portable, AppImage, CLI) | M |
| 7.6 | Accessibility checkpoint 2: full keyboard pass, screen-reader pass, axe-core in E2E | M |
| 7.7 | User documentation and three example projects (SVG master, PNG master, multi-role Android); CLI reference generated from the CLI's help output | L |
| 7.8 | Distribution: GitHub Releases automation; `imaguncula-cli` on crates.io with `cargo-binstall` support; Winget submission; Homebrew tap; Flatpak and AUR listed as community-maintained | L |
| 7.9 | Public launch: README with screenshot, license headers, repository metadata and topics, starter issues, final issue and discussion templates | M |
| 7.10 | v1.0 release checklist, tag, and changelog | S |

The original estimate was about 6–7 months of single-developer work, re-estimated at each phase boundary.

---

## 7. Milestones

| Milestone | After phase | Result | Status |
| --- | --- | --- | --- |
| M0 Walking skeleton | 0 | App opens and saves a project; CI passes on three operating systems; one bootstrap script gets a fresh machine to a running dev app | Reached |
| M1 Trustworthy input | 1 | Import and validated preview of real artwork | Reached |
| M2 First export | 2 | Custom PNG packages with a report; safe cancellation | Reached |
| M3 First Useful Release (v0.1) | 3 | Windows ICO and Web/PWA end to end through the GUI and the CLI | Reached; no v0.1 was published |
| M4 Desktop trifecta | 4 | Linux and macOS/iOS conventional assets | Reached |
| M5 Android complete | 5 | Adaptive, legacy, and store artwork | Reached |
| M6 Preset maturity | 6 | Packaged Windows assets, preset migration UI, preset review guide | Reached |
| M7 v1.0 public launch | 7 | Installers, portable builds, CLI on crates.io, package-manager channels | Beta published 2026-11-21; signing, crates.io, Winget, and Homebrew pending |

---

## 8. Risk Register

| ID | Risk | Likelihood | Impact | Mitigation |
| --- | --- | --- | --- | --- |
| R1 | `ico`/`icns` crates can't express the required entry layouts | Medium | Medium | Phase 0 spike; both formats are simple enough to write by hand behind the encoder interface. Both crates proved sufficient. |
| R2 | Preset schema can't cleanly express all four packaging shapes | Medium | High | Design the schema against all four shapes first (0.5), with windows-desktop, web-pwa, hicolor, and appiconset drafted as schema fixtures before Phase 3 code used them |
| R3 | Vendor requirements change (Apple, Android, MSIX) | High | Medium | Presets are versioned data with `specSources` and `lastReviewed`, re-checked under `docs/preset-review-process.md` |
| R4 | `resvg` gaps (fonts, filters) on real-world SVGs | Medium | Medium | Unsupported features produce warnings instead of silent mis-rendering; a per-output source replacement is the fallback |
| R5 | Tauri WebDriver E2E is flaky in CI | Medium | Low | Keep E2E at smoke depth and put integration coverage in headless core tests. Occasional runner and package-mirror failures have needed reruns |
| R6 | Color-management scope creep | Medium | Medium | v1 policy fixed in ADR 0010: sRGB pipeline, detect and document profiles; revisit after v1 |
| R7 | Detecting detail loss at small sizes (PLAN.md "where detection is practical") becomes a rabbit hole | Medium | Low | v1 uses a heuristic only: a drop in edge density or contrast between the source and a ≤20 px render gives a soft warning |
| R8 | Single-developer bus factor or stall | — | High | Docs, ADRs, and this plan were updated at every phase boundary; GOVERNANCE.md describes continuity, including a designated account successor |
| R9 | Bootstrap scripts and onboarding docs go stale as toolchains move | High | Medium | `bootstrap.yml` runs the real scripts weekly and on change; the README quick start contains nothing CI doesn't run |
| R10 | Unsigned macOS builds make a bad first impression (Gatekeeper) | High | Low | Document how to open unsigned builds; obtain a signing identity before 1.0; add a Homebrew cask once builds are signed |
| R11 | Package-manager channels become too much for one maintainer | Medium | Low | Own only GitHub Releases, crates.io, Winget, and Homebrew; mark Flatpak, AUR, and others as community-maintained |

---

## 9. Working Agreements

1. A phase is done when its PLAN.md exit criteria pass in CI, not when its tasks are merged.
2. No Tauri types in `imaguncula-core`; review rejects them.
3. Every preset row has `usage` text and a spec source.
4. Validation results are typed values carried into the report and project file, never ad-hoc strings.
5. When the code diverges from a plan, update this file (and PLAN.md if the product changes) in the same pull request.

# Imaguncula Execution Plan

Status: active — Phases 0 through 7 implemented and tested locally; hosted, release, package-publication, and manual confirmation pending
Derived from: [PLAN.md](PLAN.md)
Last updated: 2026-08-01

This document turns the product plan in `PLAN.md` into an actionable engineering plan. Where `PLAN.md` defines *what* and *why*, this document defines *how*, *in what order*, and *with what*. Product requirements in `PLAN.md` remain authoritative; if the two documents conflict, `PLAN.md` wins and this document must be updated.

### Recovery and current implementation progress

The official scaffold removed the pre-existing plan files on 2026-07-31. `PLAN.md` was restored byte-for-byte from its originating task. This execution plan was restored from the current task log; the truncated portion of §4.3–§4.7 was reconstructed from this document's task, milestone, and risk references.

| Task | State | Evidence |
| --- | --- | --- |
| 0.1 Repository foundation | Complete | Git initialized on `main`; dual license, README, contribution guide, and `justfile` added |
| 0.2 Tauri/Svelte scaffold | Complete | Imaguncula shell replaces template demo; `pnpm check`, frontend build, and Windows Tauri no-bundle build pass |
| 0.3 Cargo workspace | Complete | `imaguncula-core`, `imaguncula-presets`, and thin `imaguncula-desktop` adapter compile and test as one workspace |
| 0.4 Library spikes | Implemented; hosted confirmation pending | SVG, PNG, Lanczos3, and ICO spikes pass on Windows and clean Linux; the first CI run remains the independent-machine check |
| 0.5 Preset schema | Implemented; CI pending | Draft 2020-12 schema plus semantic cross-reference tests cover files, containers, directory trees, and generated metadata |
| 0.6 Windows desktop preset | Complete | Immutable v1 records Microsoft's minimum ICO sizes; versioned v2 adds the observed 42 px taskbar compatibility entry without changing v1 |
| 0.7 Project files | Complete locally | Typed schema v2 model with a v1 migration ladder, strict load validation, atomic save, Tauri commands, and accessible New/Open/Save UI pass local checks |
| 0.8 Quality gates | Complete locally | The pinned Windows toolchain executes the exact `just ci` wrapper end-to-end; Prettier, ESLint, Svelte/TypeScript checks, Vitest, fuzz harness compilation, rustfmt, Clippy, Rust tests, production frontend build, and native Tauri build pass |
| 0.9 Cross-platform CI | Implemented; first run pending | GitHub Actions defines the required 3 OS × 3 gate matrix and executes the same `just` entry points |
| 0.10 Architecture and decisions | Complete | Architecture, contribution guide, and ADRs 0001–0010 record the implemented decisions; ADR 0002 records local spike evidence |
| 0.11 Bootstrap scripts | Implemented; macOS run pending | Windows dry-run is mutation-free; bare Ubuntu 24.04 installs the pinned toolchains and reaches a successful `just build`; doctor scripts report exact subprocess failures |
| 0.12 Bootstrap CI | Implemented; first run pending | Scheduled/on-change workflow exercises Windows and macOS hosts plus pristine Ubuntu; the Ubuntu path passes locally from the bare image |
| 0.13 Dev container | Complete locally; Codespaces pending | Image builds from scratch with checksum-verified Node; full native-Linux doctor passes with zero warnings |
| 0.14 Open-source hygiene | Complete locally; hosted gate pending | Blocking dependency policy passes, Gitleaks finds no source-worktree leaks, and conduct/security/reporting templates are present |
| 1.1 Source import pipeline | Complete locally | Content classification, bounded reads, SHA-256 identity, dimensions, shape, and structured source failures are implemented in `source.rs` |
| 1.2 SVG sanitization | Complete locally | Pre-scan limits and active-content rejection precede `usvg`; all image resolvers and system-font/text behavior are disabled or warned |
| 1.3 PNG validation and color policy | Complete locally | Bounded full decode reports alpha and ICC state; ADR 0010 records deterministic sRGB treatment |
| 1.4 Canonical RGBA and rendering | Complete locally | Straight-alpha RGBA8, direct SVG rendering, alpha-aware Lanczos3, and per-request no-upscale enforcement have focused tests |
| 1.5 Preview IPC and cache | Complete locally | Typed IPC returns sRGB-tagged PNG bytes; content-addressed previews and sanitized SVG trees invalidate on role replacement |
| 1.6 Source-input UI | Complete locally | Native file drop and filtered picker feed a source card with replacement/removal, metadata, warnings, and persisted project references |
| 1.7 Preview UI | Complete locally | DPR-aware actual-size display, 2×–16× nearest-neighbor inspection, editable render size, and four backgrounds are implemented |
| 1.8 Hostile-input tests | Complete locally | Script/DOCTYPE fixtures plus nesting, external resource, truncated PNG, dimension/pixel bomb, file-size, and 1×1 transparency tests pass |
| 2.1 Custom output-row model | Complete locally | Typed custom `OutputSpec` resolution, status/issues, and property-tested cross-platform path sanitization pass |
| 2.2 Output review table | Complete locally | Virtualized semantic 11-column table supports add/edit/remove/include/filter and on-demand previews |
| 2.3 Per-row source override | Complete locally | Validated direct source, reset-to-master, hash checks, relative project portability, and per-row upscale approval persist |
| 2.4 Transactional export engine | Complete locally | Same-parent staging, create-new writes, exact tree and decoded-dimension verification, atomic commit, and cleanup tests pass |
| 2.5 Overwrite protection | Complete locally | Read-only destination states, explicit confirmation, backup-before-commit, restoration, and sentinel tests pass |
| 2.6 Generation report | Complete locally | Deterministic Markdown and typed UI summary include generated files, source hashes, and warning disposition |
| 2.7 Progress and cancellation | Complete locally | One-job Tauri authority, typed events, atomic cancellation, cleanup test, and ARIA live status are implemented |
| 2.8 Integration tests | Complete locally | Multi-row, cancellation, collision, missing/changed override, overwrite, and moved-project cases pass |
| 2.9 Golden tests | Complete locally | Semantic fixture validates exact paths/dimensions and decoded exported pixels against preview pixels |
| 2.10 Desktop E2E and a11y #1 | Implemented; live evidence pending | Linux `tauri-driver` workflow and axe WCAG A/AA scan are wired; first hosted run and manual keyboard/screen-reader pass remain pending |
| 3.1 ICO encoder integration | Complete locally | Physical artifact planner writes ordered multi-entry ICOs with BMP small entries and PNG-compressed 256; verification enumerates and decodes every entry |
| 3.2 Preset consumption | Complete locally | Typed bundled definitions merge with custom rows; exact preset versions and output enablement are controlled from the UI |
| 3.3 Windows desktop preset | Complete locally | Fourteen current Windows sizes render independently into one verified `<base>.ico` |
| 3.4 Web/PWA generators | Complete locally | Favicon ICO/SVG, Apple touch, PWA any-purpose, and maskable artifacts export through the preset package model |
| 3.5 Manifest and HTML metadata | Complete locally | Templates resolve declared package references; JSON parses and link targets must exist in staging |
| 3.6 Maskable safe zone | Complete locally | Row preview shows the 80% circle; a sampled visible-alpha heuristic emits a typed outside-zone warning |
| 3.7 Combined ZIP | Complete locally | Optional deterministic Deflate archive uses sorted preset-ID paths and is reopened for exact byte verification |
| 3.8 Phase 3 golden/integration tests | Complete locally | ICO entry order/encoding/alpha, Web PNG dimensions, SVG/JSON/HTML validity, link existence, ZIP determinism, and disabled-reference failure pass |
| 3.9 Headless CLI | Complete locally | `export` and `quick` share the core entry point; JSON output, stable exit codes, GUI-equivalence test, isolated `cargo install`, and installed-binary smoke pass |
| 3.10 First-useful-release candidate | Implemented locally; release evidence pending | All code-level Phase 3 exit checks pass; signed/published desktop artifacts, hosted matrices, manual accessibility, and a v0.1 release cut remain external gates |
| 4.1 Linux hicolor preset | Complete locally | Eight fixed PNG sizes and sanitized scalable SVG export in the freedesktop hicolor application directory tree |
| 4.2 Xcode AppIcon catalogs | Complete locally | Mutually exclusive modern single-1024 and compatibility iOS/iPadOS modes plus the ten-slot macOS catalog export exact `AppIcon.appiconset` trees |
| 4.3 Contents cross-validation | Complete locally | Verification requires the exact declared filename set, safe sibling paths, existing files, valid JSON, and decoded dimensions |
| 4.4 ICNS encoder | Complete locally | Seven source-independent representations encode into ICNS, reopen, enumerate, decode to RGBA, and preserve alpha |
| 4.5 Icon Composer path | Complete locally | GUI manages required foreground/background roles; export produces validated 1024-pixel layers and instructions and never fabricates `.icon` |
| 4.6 Apple platform CI | Implemented; hosted confirmation pending | Linux schema job and macOS `plutil` plus `actool` compile jobs generate through the production CLI path |
| 4.7 Phase 4 golden/integration tests | Complete locally | Linux tree, both iOS catalog modes, macOS catalog dimensions, Contents references, ICNS entries, role gates, and Composer package behavior pass |
| 5.1 Multi-role source model | Complete locally | Typed preset summaries drive GUI source management and repeatable CLI role assignments; every role retains independent path, hash, import limits, and validation |
| 5.2 Legacy and round launcher assets | Complete locally | Five Android density outputs render from the flat master; optional round group adds five matching fallbacks |
| 5.3 Adaptive and themed resources | Complete locally | Foreground/background density resources and API 26 XML are required atomically; optional monochrome resources add the API 33 override; legacy-only export remains deliberate and supported |
| 5.4 Launcher mask previews | Complete locally | Android rows switch among full bleed, circle, squircle, and rounded-square masks; adaptive layers show the 66/108 safe-zone overlay and typed alpha warning |
| 5.5 Play Store artwork | Complete locally | Final 512×512 output flattens over declared white, encodes as RGB, and verifies dimensions and 1 MiB maximum without changing intermediates |
| 5.6 Android resource CI | Implemented; hosted confirmation pending | Minimal AGP 9.3 fixture receives production CLI output and runs AAPT2 `processDebugResources` with the pinned compatible toolchain |
| 5.7 Phase 5 golden/integration tests | Complete locally | Default adaptive, missing-role denial, flat legacy-only, complete optional groups, partial-group denial, XML references, all density dimensions, RGB store output, and nondefault CLI selection pass |
| 6.1 MSIX/UWP preset | Complete locally | Versioned preset contains 122 v1 rows and 134 v2 rows; 57 required defaults plus target-size, unplated, theme, tile, splash, badge, Store, and intermediate-scale optional groups |
| 6.2 Windows packaged-app fixture | Complete locally; hosted confirmation pending | Production CLI output feeds a packaged-classic-app fixture; local MakePri indexing resolved all 57 generated PNGs and MakeAppx built the semantic fixture package; Windows CI runs the same SDK sequence |
| 6.3 Preset comparison and acceptance | Complete locally | Added/removed/explicitly-renamed diff model, deterministic fingerprint, fail-closed application, persisted acceptance, Tauri commands, and review UI pass focused checks |
| 6.4 Project migration framework | Complete locally | Schema v2 loads v1 through an explicit ladder, reports notices, preserves exact preset pins, and marks migrated UI state dirty until saved |
| 6.5 Preset review process | Complete locally | Vendor review procedure is documented; CI requires schema-valid review dates, HTTPS sources, immutable identity, and semantic cross-references |
| 7.1 Hostile-input campaign | Complete locally; hosted nightly confirmation pending | Four locked `cargo-fuzz` targets cover SVG inspection, path sanitization, ICO input, and ICNS input; the pinned nightly and cargo-fuzz versions completed 2,500 SVG, 10,000 path, 1,000 ICO, and 400 ICNS local runs without a crash |
| 7.2 Determinism tests | Complete locally | Repeat exports, generated reports, container output, full directory trees, and combined ZIPs are compared byte-for-byte; documented exceptions are explicit |
| 7.3 Resource and staging limits | Complete locally | SVG element stress stops at the configured boundary; size, dimension, pixel, memory, time, traversal, and staging-escape cases fail before publication |
| 7.4 Release packaging | Complete locally for Windows; hosted cross-platform artifacts pending | Pinned local release build produced MSI and NSIS bundles whose archives pass integrity checks; the portable ZIP, target-qualified CLI archive, and four-entry SHA-256 file were staged and reverified; tag workflow defines AppImage, DMG, and unsigned-artifact caveats |
| 7.5 Clean-runner smoke | Implemented; local Windows partial evidence and hosted execution pending | Local release executable and extracted portable launches, archived CLI export, and non-registering MSI administrative extraction pass; installer mutation was not attempted over an existing 0.1.0 installation; release matrix installs and launches every desktop artifact on fresh Windows, Linux, and macOS runners |
| 7.6 Accessibility checkpoint | Automated coverage complete locally; hosted/manual evidence pending | Landmark, keyboard order, accessible-name, and WCAG 2.0/2.1/2.2 A/AA axe assertions extend desktop E2E; manual keyboard and screen-reader records remain open |
| 7.7 Documentation and examples | Complete locally | User, security, determinism, distribution, and accessibility guides; three checked-in projects; generated CLI reference; and a production frontend screenshot are present |
| 7.8 Distribution channels | Implemented locally; publication pending | GitHub Release and dependency-ordered crates.io automation, cargo-binstall metadata, reviewed Winget generation/submission, and Homebrew formula/tap automation are defined |
| 7.9 Public launch materials | Complete locally; remote repository settings pending | README screenshot, SPDX headers, contributor templates, issue/discussion forms, exact repository metadata, and five bounded good-first-issue seeds are ready |
| 7.10 v1 release preparation | Complete locally; tag and release pending | All versions are `1.0.0`; changelog and release checklist separate verified local evidence from hosted and external publication gates |

Next executable task: **review and publish the release commit, then run the hosted workflow and manual evidence gates**. Hosted Windows/macOS/Linux, Codespaces, hosted fuzz confirmation, desktop E2E, Xcode `actool`, Android AAPT2, hosted MakePri/MakeAppx confirmation, clean-runner artifacts, signing/notarization, package publication, repository settings, and manual accessibility evidence remain pending repository publication or an equivalent live environment.

---

## 1. Resolutions for the Phase 0 Open Decisions

`PLAN.md` lists eight open decisions. Proposed resolutions, to be confirmed at the start of Phase 0:

| # | Decision | Proposed resolution | Rationale |
| --- | --- | --- | --- |
| 1 | Frontend framework and package manager | Svelte 5 + TypeScript, Vite, pnpm | Matches the plan's initial recommendation; small runtime, excellent Tauri integration, strong typed-store story for the output table. pnpm for strict, fast, disk-efficient installs. |
| 2 | Rust libraries | See §3.2 library table | Chosen for maintenance health, determinism, and no-network operation. |
| 3 | SVG handling | Parse once into a sanitized `usvg` tree; keep only the sanitized tree in memory; re-render every output size from that tree | Satisfies "render each output from the original input" while making sanitization a single choke point. The original file bytes are retained on disk and hashed, never re-parsed per output. |
| 4 | Default selection for a new project | Presets: Windows desktop + Web/PWA, both fully enabled. No custom rows. | Matches the Definition of First Useful Release; smallest useful default. |
| 5 | Combined ZIP layout | `<basefilename>-icons.zip` containing one top-level directory per preset (`windows-ico/`, `web/`, `linux-hicolor/`, …) plus `report.md` at the root | Predictable, collision-free, and self-describing. Directory names come from preset IDs, not display names. |
| 6 | First public build targets | Build and test on Windows, macOS, and Linux from Phase 0. First release *publishes* Windows (installer + portable) and Linux (AppImage); macOS builds are produced but distributed unsigned with documented caveats until a signing identity exists | The open-source accessibility goal (PLAN.md) pulls toward publishing everywhere; Apple signing/notarization requires a paid identity, so macOS ships best-effort first and first-class once signing is set up. |
| 7 | Presets updatable independently of releases | **No** for v1. Presets ship compiled into the binary as versioned data files. The schema and loader are designed so an out-of-band update channel could be added later without migration. | Avoids designing an update/trust mechanism now; preset *versioning* (the hard part) is still built in Phase 0. |
| 8 | ICNS in first release | Deferred to Phase 4, after Windows/Web output, exactly as the phase ordering already implies | Keeps the first-useful-release surface minimal. |

Each resolution should be recorded as an ADR (see §4.4) when confirmed.

---

## 2. Technology Stack

### 2.1 Application shell

- **Tauri 2.x** (latest stable at scaffold time).
- Frontend served from the bundled asset directory; no dev-server dependency in release builds.
- Tauri capability/permission configuration restricted to: dialog (open/save), filesystem access scoped to user-chosen paths, and shell-open for the "reveal in file manager" affordance. No HTTP client capability — enforces the offline guarantee at the shell level.

### 2.2 Frontend

- **Svelte 5** with TypeScript, strict mode.
- **Vite** for build/dev.
- **pnpm** as package manager, with a committed lockfile.
- Virtualized table: a lightweight virtual-list implementation (e.g. `@tanstack/virtual` or hand-rolled) — the output table can exceed 100 rows with all presets enabled.
- No UI component framework initially; plain semantic HTML + a small design-token CSS layer keeps the accessibility requirements (§ Accessibility in PLAN.md) directly controllable.
- State: Svelte stores wrapping a single serializable `ProjectState` object that mirrors the project-file schema, so save/load is a (de)serialization of app state rather than a separate model.

### 2.3 Rust core libraries

| Concern | Crate | Notes |
| --- | --- | --- |
| Raster decode/encode | `image` | PNG decode/encode; alpha preserved. |
| High-quality resize | `fast_image_resize` | Deterministic Lanczos3 downsampling; SIMD-accelerated. |
| SVG parse/sanitize/render | `usvg` + `resvg` | `usvg` produces a normalized tree we can inspect for unsupported features; `resvg` rasterizes. Script/external-reference stripping happens at `usvg` options level plus an explicit pre-parse XML scan. |
| ICO | `ico` | Encoder producing PNG-compressed entries for 256×256 and BMP entries where required. Verify entry-order behavior in Phase 3; fall back to a hand-rolled encoder if the crate can't express what the preset needs (ICO is a simple format). |
| ICNS | `icns` | Phase 4. Same fallback caveat. |
| ZIP | `zip` | Deflate, deterministic entry ordering, fixed timestamps for reproducibility. |
| JSON | `serde` / `serde_json` | Project file, `Contents.json`, web manifest. |
| XML | `quick-xml` | Android adaptive-icon XML. Writing only; no untrusted XML parsing besides SVG (handled by usvg). |
| Hashing | `sha2` | Source content hashes in project file. |
| Temp/staging | `tempfile` | Staging directories with automatic cleanup on drop. |
| Schema validation | `jsonschema` | Validates preset definition files in tests/CI, and project files on load. |
| Color profiles | Deferred; see ADR 0010 | V1 detects ICC presence, warns, treats decoded channel values as sRGB, and tags generated PNGs as sRGB. It does not copy an unprocessed source profile or claim color-managed conversion. |

Library choices are Phase 0 spike targets, not commitments — each gets a half-day validation spike (§6, Phase 0 tasks) before the ADR is written.

### 2.4 Toolchain and quality gates

- Rust: pinned stable toolchain via `rust-toolchain.toml`; `rustfmt`, `clippy -D warnings`.
- TypeScript: `eslint` + `prettier`, `svelte-check`, `tsc --noEmit` in CI.
- Test runners: `cargo test` / `cargo nextest`; `vitest` for frontend units; Tauri WebDriver (`tauri-driver`) for a small smoke-level E2E suite from Phase 2 onward.
- Task entry points via `just` (a `justfile` with `setup`, `dev`, `test`, `lint`, `build`, `ci` recipes) so CI and contributors run identical commands.

---

## 3. Architecture and Repository Layout

### 3.1 Repository layout

```text
imaguncula/
  PLAN.md
  EXECUTION-PLAN.md
  README.md
  LICENSE / LICENSE-MIT / LICENSE-APACHE
  CONTRIBUTING.md
  CODE_OF_CONDUCT.md
  SECURITY.md
  justfile
  rust-toolchain.toml
  Cargo.toml                     # workspace
  scripts/
    bootstrap.ps1                # fresh Windows machine -> runnable dev env
    bootstrap.sh                 # fresh macOS/Linux machine -> runnable dev env
    doctor.ps1 / doctor.sh       # diagnose an existing env; used by bootstrap verify step
  .devcontainer/
    devcontainer.json            # zero-setup container path (VS Code / Codespaces)
    Dockerfile
  crates/
    imaguncula-core/             # pure processing core, no Tauri deps
      src/
        source.rs                # source import, validation, hashing
        raster.rs                # canonical RGBA type, resize pipeline
        svg.rs                   # sanitize + render
        preset/                  # preset model, loader, schema validation
        encode/                  #   png.rs, ico.rs, icns.rs (feature-gated)
        metadata/                # Contents.json, manifest, XML, HTML snippets
        export/                  # staging, validation, atomic move, zip
        report.rs                # generation report model
        project.rs               # project file (de)serialization + migration
      tests/                     # unit + golden-file tests
    imaguncula-cli/              # headless CLI over imaguncula-core (Phase 3+)
    imaguncula-presets/          # versioned preset data files + schema
      schema/preset.schema.json
      presets/
        windows-desktop/1/preset.json
        web-pwa/1/preset.json
        ...
  src-tauri/                     # Tauri shell: IPC commands only, thin
    src/commands.rs
    capabilities/
  src/                           # Svelte frontend
    lib/
      state/                     # ProjectState stores
      components/                # OutputTable, PreviewPane, SourcePicker, ...
      ipc/                       # typed wrappers over Tauri invoke
    routes/
  fixtures/                      # tiny source images + hostile inputs + goldens
  docs/
    architecture.md
    adr/
    preset-review-process.md     # Phase 6 deliverable
  .github/workflows/ci.yml
```

Governing rule: **`imaguncula-core` must compile and test with no Tauri or UI dependency.** Everything the product plan calls "processing core" lives there; `src-tauri` is a thin IPC adapter. This is what makes golden-file and integration testing cheap, and it is what lets `imaguncula-cli` (the second way to run the product) be a thin argument-parsing wrapper rather than a second implementation.

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

Key invariants, enforced in `imaguncula-core` and unit-tested:

1. No render path ever takes a previously resized bitmap as input (PLAN.md "avoid repeated resizing").
2. Upscale of raster input is a typed error unless the specific output carries an explicit user override flag.
3. Every path written inside a package is produced by a sanitizer that rejects absolute paths and `..` segments.
4. Export either completes and moves atomically, or leaves nothing at the destination.

### 3.3 IPC command surface (initial)

Small, coarse-grained, all typed end to end (Rust types → generated TS via `specta`/`tauri-specta` or hand-maintained mirrors — decide in Phase 0):

- `import_source(path, role) -> SourceInfo | SourceError`
- `render_preview(output_id, background, zoom) -> PngBytes`
- `resolve_outputs(project_state) -> OutputSpec[]` (drives the table)
- `export(project_state, dest_dir, options) -> ExportReport | ExportError` (with progress events)
- `cancel_export(job_id)`
- `save_project(path, project_state)` / `load_project(path) -> ProjectState + migration notices`
- `list_preset_upgrades(project_state) -> PresetUpgrade[]` and `apply_preset_upgrade(...) -> ProjectState` (Phase 6)

Progress and completion are emitted as Tauri events so the frontend can announce them to assistive technology (accessibility requirement).

### 3.4 Preset definition format

Presets are JSON data files validated against `preset.schema.json` in CI (PLAN.md requires data-driven, versioned presets). Schema version 1 separates logical rendered outputs from their physical packaging:

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

`packages[].kind` is a discriminated union:

- `files`: standalone outputs with safe relative paths;
- `container`: ordered ICO/ICNS entries with explicit per-entry encoding;
- `directory`: a package root plus output-to-relative-path mappings;
- `metadata`: JSON/XML/HTML/text templates whose declared `{{reference}}` tokens point to an output inside another package.

JSON Schema rejects unknown properties, unsafe paths, invalid dates/URIs, mismatched encoders, and invalid structural variants. Rust tests add semantic validation for uniqueness and cross-references that JSON Schema cannot express. Preset version bumps create a *new* data file (`presets/<id>/<version>/`), never mutate an old one; that is what makes the Phase 6 comparison/migration feature tractable.

---

## 4. Cross-Cutting Workstreams

These run alongside the phases rather than inside any single one.

### 4.1 Security posture (from Phase 0, hardened in Phase 7)

- SVG: disable scripts, external references, and remote images at parse options; additionally scan raw XML for `<script>`, `<foreignObject>`, external entity declarations, and `href` values outside the document before parsing. Enforce byte-size and node-count caps.
- Raster: cap decoded pixel count (e.g. 64 MP), file size, and total memory per export job. `image` crate limits API + explicit checks.
- Paths: single `sanitize_relative_path()` used by every writer; property-tested.
- Archive: ZIP writer only ever receives sanitized relative paths; no symlinks.
- Overwrite: export refuses to write into a non-empty destination package directory without an explicit confirmation flag from the UI.
- Hostile fixtures collected continuously in `fixtures/hostile/` as issues are found, not saved up for Phase 7 — Phase 7 only *systematizes* them.

### 4.2 Accessibility

- Component-level requirements written into each UI task's acceptance criteria (keyboard path, focus order, non-color status signaling, ARIA live region for progress).
- One a11y audit checkpoint at the end of Phase 2 (first real UI surface) and again in Phase 7, using axe-core in the E2E suite plus a manual keyboard/screen-reader pass.

### 4.3 Documentation

- `docs/architecture.md` started in Phase 0, updated when a phase changes a boundary.
- ADRs (below) for every resolved open decision and library selection.
- User documentation and example projects are Phase 7 deliverables, but every completed phase updates the README and architecture notes needed to use and maintain that phase.
- CLI reference documentation is generated from the CLI's own help output so documented flags cannot drift from implemented flags.
- README setup instructions point to the tested bootstrap scripts; prerequisites are not maintained as a second, manual source of truth.

### 4.4 Architecture decision records

ADRs live in `docs/adr/`, use monotonically numbered filenames, and record context, decision, consequences, and status. Accepted decisions are not silently rewritten; later changes supersede an earlier ADR and link back to it.

The initial ADR queue covers:

1. frontend framework, static application shape, and package manager;
2. Rust processing-library choices after the Phase 0 spikes;
3. the sanitized SVG intermediate;
4. new-project preset defaults;
5. combined ZIP naming and layout;
6. initial build and published-artifact targets;
7. bundled preset delivery for v1;
8. ICNS phase scope;
9. project licensing.

### 4.5 Open-source release path

- Dual-license the project under MIT or Apache-2.0 and keep package metadata aligned with the repository license files.
- Add contribution, conduct, security, issue, and pull-request guidance before accepting public contributions.
- Keep generated artifacts, dependency locks, and release provenance reproducible and reviewable.
- Publish releases from CI, with checksums and a changelog, rather than from an unrecorded local build.
- Treat code signing and notarization as platform release work with documented unsigned-development fallbacks, not as reasons to weaken local security settings.
- Seed issue labels and narrowly scoped starter issues only after the relevant architecture and test harness exist.

### 4.6 Headless CLI

The CLI is the second interface over `imaguncula-core`, not a Tauri automation wrapper and not a separate exporter. It arrives with the Phase 3 first-useful-release work.

Initial command shapes:

```text
imaguncula export <project.imaguncula.json> --output <directory>
imaguncula quick <source.svg|source.png> --preset <preset-id> --output <directory>
```

The CLI must:

- resolve and export through the same core entry point as the GUI;
- use stable non-zero exit codes for validation, source, collision, cancellation, and export failures;
- support a machine-readable report option in addition to human-readable output;
- remain non-interactive by default, requiring explicit overwrite flags;
- produce output semantically equivalent to the GUI for the same project state;
- require no display server or Tauri runtime.

Release binaries accompany desktop artifacts, and `cargo install` is supported once the CLI package is ready to publish.

### 4.7 Fresh-machine bootstrap

Bootstrap scripts are executable onboarding documentation. Windows uses `scripts/bootstrap.ps1`; macOS and Linux use `scripts/bootstrap.sh`. Both delegate diagnostics to `scripts/doctor.*` so an existing environment can be checked without reinstalling anything.

Bootstrap requirements:

1. **One entry point per shell** — no undocumented prerequisite commands before the script.
2. **Idempotent and pinned** — re-running is safe, installed versions are visible, and repository toolchain files remain authoritative.
3. **Shortest path to ready**: clone (or download zip) → run one script → `just run`.
4. **Explain everything it does** — print each step before doing it, support `--dry-run` to preview and `--yes` for unattended runs; no silent system mutation.
5. **End with verification** — run `doctor` (toolchain versions, native deps, a `cargo check` + `pnpm install` smoke) and print an unambiguous "ready: run `just run`" or a specific failure with a remediation hint.
6. **Fail loudly and resumably** — any step failure names the step, and re-running resumes where it left off (consequence of idempotency).
7. **Respect the machine** — user-scoped installs where possible; anything needing elevation is called out explicitly before it happens.

`doctor` is factored out (`scripts/doctor.*`) so existing-env contributors can diagnose problems without re-running installs, and CI reuses it as the bootstrap acceptance check.

**Continuous verification (anti-rot)**: a scheduled weekly + on-script-change CI workflow runs the real bootstrap from truly clean environments — fresh `windows-latest` and `macos-latest` runners with caches disabled and a pristine Ubuntu container — then asserts `doctor` passes and a debug build completes. Onboarding instructions that aren't executed by CI are instructions that rot (risk R9).

The dev container is the escape hatch when local bootstrap isn't possible (locked-down machine, unsupported distro): `devcontainer.json` + Dockerfile provide the full toolchain for core/CLI/frontend work and building, with the GUI runnable natively from the same checkout when desired.

---

## 5. Testing Strategy — Concrete Mapping

PLAN.md's testing strategy mapped to infrastructure:

| PLAN.md category | Where it lives | Tooling | Introduced |
| --- | --- | --- | --- |
| Unit tests | `crates/imaguncula-core` inline + `tests/` | `cargo test`, `proptest` for path sanitization and dimension math | Phase 0 |
| Golden-file tests | `fixtures/goldens/` + `crates/imaguncula-core/tests/golden.rs` | Decode-and-compare helpers (semantic comparison: dimensions, pixel hashes of decoded RGBA, entry lists, parsed JSON/XML) — never raw encoded bytes | Phase 2 |
| Frontend units | `src/**/*.test.ts` | `vitest` — table row derivation, filename derivation preview, store logic | Phase 2 |
| Integration tests | `crates/imaguncula-core/tests/integration/` | Full export runs into tempdirs per preset; cancellation via job handle; collision, moved-project, stale-source scenarios | Phase 2 onward, grows per preset phase |
| E2E smoke | `e2e/` | `tauri-driver` + WebDriver: launch app, import fixture, export, assert report | Phase 2 (minimal), expanded Phase 7 |
| Platform fixture tests | `fixtures/platform/` + CI jobs | Windows: PowerShell script loading ICO via .NET; Android: Gradle `:app:processDebugResources` on a minimal fixture project (Phase 5, Linux CI runner with Android SDK); Xcode: `Contents.json` schema validation everywhere + `actool` compile on macOS runners (Phase 4); web: manifest JSON-schema check + link-target existence check | Per platform phase |

Golden fixture design rule: fixtures are tiny (≤64×64 sources where possible), visually asymmetric (so orientation bugs fail loudly), and include: square SVG, non-square SVG, SVG with text, large PNG, exactly-minimum PNG, too-small PNG, PNG with alpha gradient, PNG with ICC profile.

---

## 6. Phase-by-Phase Work Breakdown

Phases match PLAN.md exactly; exit criteria there are binding. This section adds task-level breakdown, sequencing, and effort classes. Effort classes: **S** ≤ ½ day, **M** ≤ 2 days, **L** ≤ 1 week. Estimates assume one developer; tasks marked ∥ can proceed in parallel with their neighbors.

### Phase 0 — Repository foundation (~3–4 weeks)

| # | Task | Effort |
| --- | --- | --- |
| 0.1 | Initialize git repo; dual MIT/Apache-2.0 license files + ADR; README stub; `justfile` skeleton | S |
| 0.2 | Scaffold Tauri 2 + Svelte 5 + TS + Vite via `create-tauri-app`; verify `just dev` and `just build` on Windows | M |
| 0.3 | Create Cargo workspace with `imaguncula-core` and `imaguncula-presets`; wire `src-tauri` to depend on core | S |
| 0.4 | Library spikes ∥: `resvg` render of a fixture SVG at 3 sizes; `fast_image_resize` determinism check across two machines/CI; `ico` crate round-trip; write ADRs | M |
| 0.5 | Define `preset.schema.json` covering all four packaging shapes (§3.4); CI job validating all preset files | L |
| 0.6 | Author the first preset data file (windows-desktop v1) as schema proof, not yet consumed by code | S |
| 0.7 | Project file: `ProjectState` Rust type + serde + `schemaVersion` + load-time validation + save/load IPC commands + minimal UI (New/Open/Save) | L |
| 0.8 | Lint/format/test gates for both languages; pre-commit hook optional, CI mandatory | M |
| 0.9 | GitHub Actions CI: matrix {windows, macos, ubuntu} × {lint, test, build} | M |
| 0.10 | `docs/architecture.md` first draft + CONTRIBUTING stub + resolve §1 decisions as ADRs | M |
| 0.11 | Bootstrap scripts per §4.7: `bootstrap.ps1`, `bootstrap.sh`, shared `doctor` verification; README quickstart written against them | L |
| 0.12 | Clean-environment CI workflow exercising the bootstrap scripts (scheduled weekly + on change) | M |
| 0.13 | Dev container: `.devcontainer/` with full toolchain; verified in Codespaces | M |
| 0.14 | Open-source hygiene gates in CI: `cargo-deny` license/advisory audit, secret scanning; CODE_OF_CONDUCT, SECURITY.md, issue/PR templates | M |

**Exit check** (from PLAN.md): clean checkout → `just setup && just ci` green on all three OSes; app opens/saves a minimal project file; preset schema validation automated. **Additionally**: a fresh VM/container with no toolchains reaches a running dev app via one bootstrap script on each OS, verified by the 0.12 workflow.

### Phase 1 — Source import and preview (~2–3 weeks)

| # | Task | Effort |
| --- | --- | --- |
| 1.1 | `source.rs`: import pipeline — classify, size/shape probe, SHA-256 hash, `SourceInfo` result type with structured error enum (not strings) | M |
| 1.2 | SVG sanitize pipeline per §4.1: pre-scan + usvg options; detection of external fonts/images/unsupported features → warnings list; non-square viewBox warning | L |
| 1.3 | PNG validation: dimensions, squareness, alpha detection, ICC presence; decide and ADR the v1 color-profile policy (risk R6) | M |
| 1.4 | Canonical RGBA type + render functions: `render_svg(tree, w, h)`, `resize_raster(src, w, h)` with the no-upscale typed error and per-output override flag | M |
| 1.5 | Preview IPC: render requested size on demand, return PNG bytes; cache keyed by (source hash, size) with invalidation on source replacement | M |
| 1.6 | UI ∥: drag-and-drop + file-picker source input; source card showing format, dimensions, warnings; master-source replacement | L |
| 1.7 | UI ∥: preview pane — actual-size rendering (device-pixel-ratio aware) and nearest-neighbor magnified inspection (2×–16×); background switcher (light/dark/transparent/checkerboard) | L |
| 1.8 | Unit tests: every validation rule; hostile fixtures: script SVG, external-entity SVG, decompression-bomb PNG, 1×1 PNG | M |

**Exit check**: previews render from original source only (assert via cache-key design + test); unsafe input fails with specific typed explanation surfaced in UI; upscaling visibly blocked/warned.

### Phase 2 — Custom PNG output (~3 weeks)

| # | Task | Effort |
| --- | --- | --- |
| 2.1 | `OutputSpec` model + `resolve_outputs`: custom rows only at this phase; filename sanitization (property-tested) | M |
| 2.2 | Output review table UI: virtualized, all PLAN.md columns, add/edit/remove/enable/disable custom rows, platform filter (trivial until Phase 3, built now) | L |
| 2.3 | Per-row source override: pick file, validate against row dimensions, reset-to-master; persisted in project file | M |
| 2.4 | Export engine: staging tempdir → render+encode each included output → verify staged tree exactly matches resolved `OutputSpec` list → atomic move → cleanup on cancel/failure | L |
| 2.5 | Overwrite protection: destination inspection + explicit confirm path through IPC | S |
| 2.6 | Generation report: model + Markdown rendering (file list, dimensions, source used, warnings accepted/outstanding); written into package and shown in UI | M |
| 2.7 | Progress + cancellation: job handle, Tauri events, ARIA live region announcements | M |
| 2.8 | Integration tests: multi-row export, cancellation mid-job, collision, missing override source, moved project dir | L |
| 2.9 | Golden tests bootstrap: fixture set + decode-compare helpers | M |
| 2.10 | Minimal E2E smoke via tauri-driver; a11y audit checkpoint #1 | M |

**Exit check**: arbitrary custom PNG sizes export with exact names/dimensions; cancelled/failed exports leave no falsely complete package.

### Phase 3 — Windows and Web/PWA (~3 weeks) → **First Useful Release candidate**

| # | Task | Effort |
| --- | --- | --- |
| 3.1 | ICO encoder integration (or replacement per 0.4 spike): multi-entry, PNG-compressed 256, alpha, exact-size embedding priority | L |
| 3.2 | Preset *consumption*: loader from `imaguncula-presets` data → resolved outputs merged with custom rows in the table; preset enable/disable UI | L |
| 3.3 | Windows desktop preset wired end to end; additional standard/custom sizes into the ICO | M |
| 3.4 | Web/PWA preset data + generators: favicon.ico, favicon.svg (pass-through of sanitized SVG), exact-size PNGs, apple-touch-icon, 192/512 PWA, maskable variant | L |
| 3.5 | `site.webmanifest` icon entries + HTML `<link>` snippet generation; link-target existence validation in export verify step | M |
| 3.6 | Maskable safe-zone visualization in preview + out-of-safe-zone warning heuristic (alpha coverage outside the safe-zone circle) | M |
| 3.7 | Combined ZIP output per §1 decision 5 | M |
| 3.8 | Golden + integration tests: ICO entry enumeration/decode at every size, manifest/HTML reference validity, MIME/dimension assertions | L |
| 3.9 | `imaguncula-cli` MVP (§4.6): `export` on a project file + `quick` one-shot mode; shares the core `export` entry point; GUI/CLI output-equivalence integration test; `cargo install` works | L |
| 3.10 | Release-candidate pass against PLAN.md "Definition of the First Useful Release" checklist; cut v0.1 (Windows installer + portable, Linux AppImage, CLI binaries) | M |

**Exit check**: ICO entries enumerate and decode at every selected size; favicon/PWA files match declared dimensions and types; manifest/HTML snippets reference existing files. Plus: all nine First-Useful-Release criteria demonstrably pass.

### Phase 4 — Linux and Apple conventional assets (~2–3 weeks)

| # | Task | Effort |
| --- | --- | --- |
| 4.1 | Linux hicolor preset: directory-tree packaging shape, PNG sizes, scalable SVG pass-through when source is SVG | M |
| 4.2 | Xcode asset catalog generator: `AppIcon.appiconset` + `Contents.json` for iOS/iPadOS and macOS catalogs; single-1024 modern mode vs. compatibility mode toggle | L |
| 4.3 | `Contents.json` ↔ produced-files cross-validation in export verify step | M |
| 4.4 | ICNS encoder integration; required raster sizes; entry decode tests | L |
| 4.5 | Icon Composer guidance path: when the layered-Apple output is selected, produce validated source layers + instructions document instead of pretending to build `.icon` (per PLAN.md) | M |
| 4.6 | macOS CI runner: `actool` compile check of generated catalogs; `plutil`/schema validation elsewhere | M |
| 4.7 | Golden + integration tests for all three outputs | M |

### Phase 5 — Android (~3 weeks)

| # | Task | Effort |
| --- | --- | --- |
| 5.1 | Source-role model generalization: foreground / background / monochrome roles with per-role validation (built on 1.1's role parameter) | M |
| 5.2 | Legacy mipmap density outputs + round variants from flat master | M |
| 5.3 | Adaptive-icon XML + foreground/background/monochrome resource generation; hard requirement gate: adaptive export blocked without required roles | L |
| 5.4 | Launcher-shape mask previews (circle, squircle, rounded square) in preview pane | M |
| 5.5 | Play Store artwork output (512×512 PNG, no alpha — note the flatten step is allowed here as it is a *final* output, not an intermediate) | S |
| 5.6 | Android fixture project + CI resource-compile job | M |
| 5.7 | Golden + integration tests; role-missing and legacy-only-from-flat scenarios | M |

### Phase 6 — Windows packaged assets and preset maintenance (~3 weeks)

| # | Task | Effort |
| --- | --- | --- |
| 6.1 | MSIX/UWP asset-family preset data: scale variants, target-size variants, unplated/theme variants — largest preset by row count; exercises `optionalOutputGroups` | L |
| 6.2 | Fixture packaged app + `makeappx`/manifest validation in Windows CI | M |
| 6.3 | Preset version comparison: `PresetDiff` model + UI showing added/removed/renamed outputs before migrating a saved project; explicit user acceptance persisted | L |
| 6.4 | Project-file migration framework generalization (schemaVersion ladder + preset-version pinning) | M |
| 6.5 | `docs/preset-review-process.md`: documented vendor-spec review procedure; verify every preset file carries specSources + lastReviewed (CI check) | M |

### Phase 7 — Release hardening and public launch (~4–5 weeks)

| # | Task | Effort |
| --- | --- | --- |
| 7.1 | Hostile-input campaign: systematize `fixtures/hostile/`, add fuzz targets (`cargo-fuzz`) for SVG pre-scan, path sanitizer, ICO/ICNS encoders' input paths | L |
| 7.2 | Determinism tests: byte-identical re-export where format permits; documented exceptions where not | M |
| 7.3 | Memory/time limits verified under stress fixtures; staging-escape attempts tested | M |
| 7.4 | Packaging per §1 decision 6: Windows MSI/NSIS + portable zip, Linux AppImage, macOS dmg (unsigned, documented caveats); signing/notarization docs for all three | L |
| 7.5 | Clean-machine smoke tests: install and run every published artifact (installer, portable, AppImage, CLI) on fresh VMs | M |
| 7.6 | A11y audit checkpoint #2: full keyboard pass, screen-reader pass, axe-core in E2E | M |
| 7.7 | User documentation + 3 example projects (SVG master; PNG master; multi-role Android); CLI reference doc generated from the CLI's own help output | L |
| 7.8 | Distribution channels: GitHub Releases automation for all artifacts; publish `imaguncula-cli` to crates.io with `cargo-binstall` support; winget manifest submission; Homebrew tap; Flatpak/AUR documented as community-maintained targets | L |
| 7.9 | Public launch checklist (§4.5): README with screenshots/demo GIF, license headers, repo metadata/topics, `good first issue` seed set, discussion/issue templates final | M |
| 7.10 | v1.0 release checklist, tagging, changelog | S |

Cumulative estimate: **roughly 6–7 months** of single-developer effort. Treat as ordering guidance, not commitment; re-estimate at each phase boundary.

---

## 7. Milestones

| Milestone | After phase | Externally visible result |
| --- | --- | --- |
| M0 Walking skeleton | 0 | App opens/saves a project; CI green on 3 OSes; a fresh machine reaches a running dev app via one bootstrap script |
| M1 Trustworthy input | 1 | Import + validated preview of real artwork |
| M2 First export | 2 | Custom PNG packages with report; safe cancel |
| **M3 First Useful Release (v0.1)** | 3 | Windows ICO + Web/PWA end to end, via GUI *and* CLI — PLAN.md's definition satisfied |
| M4 Desktop trifecta | 4 | Linux + macOS/iOS conventional assets |
| M5 Android complete | 5 | Adaptive + legacy + store artwork |
| M6 Preset maturity | 6 | Packaged-Windows assets; versioned-preset migration UX; preset contribution guide |
| M7 v1.0 public launch | 7 | Hardened, documented release: installers, portable builds, CLI on crates.io, package-manager channels, open-source launch complete |

---

## 8. Risk Register

| ID | Risk | Likelihood | Impact | Mitigation |
| --- | --- | --- | --- | --- |
| R1 | `ico`/`icns` crates can't express required entry layouts | Medium | Medium | Phase 0 spike; both formats are simple enough to hand-roll behind the existing encoder trait if needed |
| R2 | Preset schema can't cleanly express all four packaging shapes, causing churn | Medium | High | Design schema against all four shapes up front (0.5); windows-desktop + web-pwa + hicolor + appiconset drafted as schema test fixtures before Phase 3 code consumes them |
| R3 | Vendor requirements change mid-build (Apple/Android/MSIX) | High | Medium | Presets are data + versioned + carry specSources/lastReviewed; re-validate links at each phase start (PLAN.md Restart Point step 1) |
| R4 | `resvg` feature gaps (fonts, filters) on real-world SVGs | Medium | Medium | 1.2 detects and *warns* on unsupported features instead of silently mis-rendering; per-output replacement is the designed escape hatch |
| R5 | Tauri WebDriver E2E flaky on CI | Medium | Low | Keep E2E to smoke depth; integration coverage lives in headless core tests |
| R6 | Color-management scope creep | Medium | Medium | Explicit v1 policy decided/ADR'd in 1.3: sRGB pipeline, preserve-and-document; revisit post-v1 |
| R7 | Small-size feature-collapse detection (PLAN.md "where detection is practical") rabbit hole | Medium | Low | v1 heuristic only: edge-density/contrast drop between source and ≤20px render triggers a soft warning; anything smarter is post-v1 |
| R8 | Single-developer bus factor / stall | — | High | Restart Point discipline: docs, ADRs, and this plan updated at every phase boundary so work can resume cold |
| R9 | Bootstrap scripts and onboarding docs rot as toolchains move | High | Medium | §4.7 clean-environment CI workflow runs the real scripts weekly and on change; README quickstart contains nothing CI doesn't execute |
| R10 | Unsigned macOS builds create bad first impressions (Gatekeeper friction) | High | Low | Document the right-click-open path prominently; prioritize obtaining a signing identity after v0.1; Homebrew cask once signed |
| R11 | Package-manager channel maintenance outgrows one maintainer | Medium | Low | Own only GitHub Releases, crates.io, and winget; explicitly mark Flatpak/AUR/others as community-maintained |

---

## 9. Working Agreements

1. **Phase gates are real.** A phase is done when its PLAN.md exit criteria pass in CI, not when its tasks are merged.
2. **Core stays pure.** No Tauri types in `imaguncula-core`; violations fail review.
3. **Every preset row is explainable.** No output ships without `usage` text and a spec source.
4. **Warnings are data.** Validation results are typed enums carried through to the report and project file, never ad-hoc strings.
5. **Update the plans.** When reality diverges, amend this file (and PLAN.md if the product itself changes) in the same PR as the diverging change.

## 10. Immediate Next Actions

1. Review the complete untracked release tree, commit it intentionally, and publish the repository without rewriting local implementation evidence.
2. Run the three-OS CI, bootstrap, platform-fixture, desktop E2E/axe, fuzz, release-build, and clean-runner smoke workflows from the reviewed commit.
3. Remediate only evidence-backed hosted failures; record platform-specific results in this recovery table and the release checklist.
4. Configure protected signing, notarization, crates.io, Winget, and Homebrew environments; complete the manual keyboard and screen-reader audit.
5. Tag `v1.0.0` only after every required v1 checklist item is satisfied or the release notes explicitly disclose an accepted unsigned/unnotarized boundary.

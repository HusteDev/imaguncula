# ADR 0002: Rust image-processing libraries

Status: accepted; independent hosted CI verification pending

Date: 2026-07-31

## Context

Imaguncula needs a pure-Rust pipeline for PNG decoding and encoding, deterministic high-quality raster downsampling, sanitized SVG parsing and rendering, and multi-entry ICO encoding. These APIs establish the core image representation and encoder boundaries.

The Phase 0 spike used an asymmetric 64×64 SVG and generated RGBA patterns. Tests live in `crates/imaguncula-core/tests/library_spikes.rs`. The PNG, resize, and SVG families became production dependencies in Phase 1; ICO remains development-only until its production encoder is implemented.

## Decision

Adopt these crate families and initial versions:

| Concern | Selection | Spike evidence |
| --- | --- | --- |
| PNG decode/encode and RGBA interoperability | `image` 0.25.10 | Encoded every `resvg` output to PNG, decoded it, and recovered identical dimensions and RGBA bytes |
| Raster downsampling | `fast_image_resize` 6.1.0 | Lanczos3 with alpha handling produced repeatable 64×64 → 19×13 RGBA bytes; FNV-1a-64 `c47d9be52f69febc` is fixed as the CI comparison value |
| SVG normalization and rendering | `usvg` + `resvg` 0.47.0 | One parsed tree rendered independently at 16, 32, and 64 pixels; repeated renders were identical |
| Windows ICO | `ico` 0.5.0 | One ICO preserved ordered 16px and 32px BMP entries plus a 256px PNG entry, 32-bit depth, alpha, and decoded RGBA bytes |

Use explicit versions in spike tests. Production dependency ranges may use compatible releases after the owning module has regression coverage and the lockfile records the resolved version.

Use `fast_image_resize::ResizeAlg::Convolution(FilterType::Lanczos3)` with alpha multiplication/division enabled for the initial raster policy. The core must still reject upscaling before calling the resizer.

Use explicit ICO encoding selection: BMP for conventional small entries and PNG for 256×256. Do not rely on the crate's automatic encoding heuristic when a preset requires a specific entry layout.

Production `usvg` and `resvg` disable default features. Text and system-font loading are excluded, SVG-linked data and string image resolvers return no resources, and unsupported resource-dependent constructs produce warnings. System-font discovery cannot become an unrecorded source of cross-machine output differences.

ICNS remains a Phase 4 decision under ADR 0008. ZIP, schema-validation, metadata, hashing, and staging libraries will be accepted when their owning Phase 0 tasks exercise them.

## Consequences

- The core can share RGBA bytes between rendering, resizing, PNG encoding, and ICO encoding without a second image model.
- The spike suite will fail on a future CI runner if Lanczos3 output differs from the recorded hash.
- The recorded Lanczos3 hash and the native library tests pass on Windows x64 and a clean Linux x86_64 container. A hosted Windows/macOS/Linux CI run remains the independent-machine confirmation.
- Phase 1 establishes SVG resource denial, deterministic font behavior, source limits, and hostile-input rejection in production code and tests.

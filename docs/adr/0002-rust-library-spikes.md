# ADR 0002: Rust image-processing libraries

Status: accepted

Date: 2026-07-31

## Context

Imaguncula needs pure-Rust libraries for PNG decoding and encoding, deterministic raster downsampling, sanitized SVG parsing and rendering, and multi-entry ICO encoding. The choice fixes the core image representation and the encoder boundaries.

The Phase 0 spike used an asymmetric 64×64 SVG and generated RGBA patterns. Tests live in `crates/imaguncula-core/tests/library_spikes.rs`. The PNG, resize, and SVG crates became production dependencies in Phase 1, and `ico` became one when the Windows ICO encoder was implemented. All four are pinned to exact versions in `crates/imaguncula-core/Cargo.toml`.

## Decision

Adopt these crate families and initial versions:

| Concern | Selection | Spike evidence |
| --- | --- | --- |
| PNG decode/encode and RGBA interoperability | `image` 0.25.10 | Encoded every `resvg` output to PNG, decoded it, and recovered identical dimensions and RGBA bytes |
| Raster downsampling | `fast_image_resize` 6.1.0 | Lanczos3 with alpha handling produced repeatable 64×64 → 19×13 RGBA bytes; FNV-1a-64 `c47d9be52f69febc` is fixed as the CI comparison value |
| SVG normalization and rendering | `usvg` + `resvg` 0.47.0 | One parsed tree rendered independently at 16, 32, and 64 pixels; repeated renders were identical |
| Windows ICO | `ico` 0.5.0 | One ICO preserved ordered 16px and 32px BMP entries plus a 256px PNG entry, 32-bit depth, alpha, and decoded RGBA bytes |

Pin exact versions. A version change lands with the regression tests of the module that uses the crate.

Use `fast_image_resize::ResizeAlg::Convolution(FilterType::Lanczos3)` with alpha multiplication/division enabled for the initial raster policy. The core must still reject upscaling before calling the resizer.

Use explicit ICO encoding selection: BMP for conventional small entries and PNG for 256×256. Do not rely on the crate's automatic encoding heuristic when a preset requires a specific entry layout.

Production `usvg` and `resvg` disable default features. Text and system-font loading are excluded, SVG-linked data and string image resolvers return no resources, and unsupported resource-dependent constructs produce warnings. System-font discovery cannot become an unrecorded source of cross-machine output differences.

ICNS was decided separately in ADR 0008. ZIP, schema-validation, metadata, hashing, and staging libraries were chosen when the tasks that needed them were implemented.

## Consequences

- The core can share RGBA bytes between rendering, resizing, PNG encoding, and ICO encoding without a second image model.
- The spike suite will fail on a future CI runner if Lanczos3 output differs from the recorded hash.
- The spike tests, including the recorded Lanczos3 hash, run in hosted CI on Windows, macOS, and Linux for every pull request.
- Phase 1 establishes SVG resource denial, deterministic font behavior, source limits, and hostile-input rejection in production code and tests.

# ADR 0010: PNG color-profile policy

Status: accepted

Date: 2026-07-31

## Context

PNG sources can carry ICC profiles. The v1 decoder converts channel layout and bit depth to straight-alpha RGBA8 but does not perform ICC color conversion. Copying an input profile to resized output would falsely imply that the decoded channel values had been transformed under that profile.

## Decision

Detect ICC profile presence during bounded PNG validation and surface `icc-profile-treated-as-srgb` as a non-fatal warning. Treat decoded RGBA channel values as sRGB for v1 processing. Encode generated preview PNGs with an explicit sRGB chunk and do not copy the source ICC profile.

Preserve alpha independently of this color policy. Transparent pixels remain straight-alpha RGBA8 throughout the canonical pipeline, and alpha-aware Lanczos3 resizing prevents hidden color from bleeding across transparent edges.

Do not claim color-managed conversion. A future Little CMS or equivalent pipeline must replace this ADR before interpreting and transforming embedded profiles.

## Consequences

- Sources with an ICC profile remain usable but visibly disclose the v1 limitation.
- Generated previews have one declared color space instead of inheriting an unprocessed profile.
- Exact appearance can differ from a color-managed editor for non-sRGB source profiles.
- The policy is deterministic and does not depend on host-installed color-management services.

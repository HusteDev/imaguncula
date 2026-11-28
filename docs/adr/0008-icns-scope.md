# ADR 0008: ICNS scope

Status: accepted

Date: 2026-07-31

## Context

Windows desktop and Web/PWA outputs define the first useful release. Implementing and validating Apple container and asset-catalog behavior first would have delayed that release.

## Decision

Implement conventional ICNS generation in Phase 4 after custom PNG, Windows desktop, and Web/PWA output are complete.

## Consequences

The first useful release did not promise ICNS. ICNS export shipped in Phase 4 and is included in 1.0.0-beta.1.

## Phase 4 implementation evidence

The `icns` 0.4.0 crate is pinned with only PNG I/O enabled. Each required representation is rendered from the original source and inserted independently. Export verification reopens the completed family, requires the exact expected icon-type set, decodes each entry to RGBA, and checks dimensions and buffer length. Integration tests cover 16 through 1024 pixels and preserved transparency.

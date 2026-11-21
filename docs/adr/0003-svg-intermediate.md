# ADR 0003: Sanitized SVG intermediate

Status: accepted

Date: 2026-07-31

## Context

SVG input must be sanitized once, must not access local or remote resources, and must render every requested output from the original vector information.

## Decision

Parse accepted SVG input into one sanitized `usvg` tree and retain that tree in the bounded preview cache under the source content hash. Re-read the original file through the byte limit to verify its hash on each request, but do not parse unchanged SVG bytes again for each preview size. Render every target directly from the retained tree.

The cache retains at most eight validated source intermediates. Replacing a role invalidates the old hash's previews and tree. Eviction is safe: the next request performs the complete safety scan and normalization again before rendering.

## Consequences

Sanitization becomes one auditable choke point. Renderer behavior is deterministic for a given sanitized tree and target specification. Raw document bytes establish content identity and can create a new sanitized tree after cache eviction, but they never go directly to the renderer.

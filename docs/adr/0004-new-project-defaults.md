# ADR 0004: New-project defaults

Status: accepted

Date: 2026-07-31

## Context

A new project needs a useful default without presenting the full platform matrix before the user imports artwork.

## Decision

Enable the Windows desktop and Web/PWA presets and all their required outputs by default. Create no custom output rows.

## Consequences

The default matches the first useful release. Other platform presets remain opt-in. Users can disable individual outputs after output resolution.

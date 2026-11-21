# ADR 0005: Combined ZIP layout

Status: accepted

Date: 2026-07-31

## Context

Different platforms require incompatible package shapes. A combined archive must avoid filename collisions and remain understandable without the application.

## Decision

Name the archive `<basefilename>-icons.zip`. Place each preset package under a top-level directory named by its stable preset ID, and write `report.md` at the archive root.

## Consequences

Display-name changes do not change paths. Presets cannot collide at the archive root. The report can describe every directory and file from one stable location.

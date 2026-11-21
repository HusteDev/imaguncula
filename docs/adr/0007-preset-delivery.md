# ADR 0007: Preset delivery

Status: accepted

Date: 2026-07-31

## Context

Independently updated presets would require an update protocol, integrity checks, trust policy, and rollback behavior before the core preset model has stabilized.

## Decision

Bundle versioned preset data with application releases for v1. Design loaders and project files around explicit preset versions so an independent signed update channel can be added later without changing stored project semantics.

## Consequences

Preset corrections require an application release in v1. Existing projects stay pinned and cannot silently change when the bundled preset set grows.

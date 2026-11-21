# ADR 0006: Initial release platforms

Status: accepted

Date: 2026-07-31

## Context

The project should remain portable from its foundation, but signed Apple distribution requires credentials that are not currently part of the project.

## Decision

Build and test on Windows, macOS, and Linux from Phase 0. The first public release publishes Windows installer and portable artifacts plus a Linux AppImage. Produce macOS builds but distribute them unsigned with explicit Gatekeeper limitations until signing and notarization are available.

## Consequences

Cross-platform failures remain visible early. Release documentation must distinguish built artifacts from first-class signed distribution.

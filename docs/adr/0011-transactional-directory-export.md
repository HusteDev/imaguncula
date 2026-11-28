# ADR 0011: Transactional directory export

Status: accepted

Date: 2026-07-31

## Context

An icon package can contain many files. A cancellation, encoder failure, source change, path collision, or process error must not leave a destination that appears complete. Replacing an existing package also must not destroy the last valid package before the replacement verifies.

## Decision

Directory exports are built in a unique temporary directory under the final destination's parent. Every file uses create-new semantics. The staged tree is rejected if it contains a symlink or differs from the resolved output list plus `report.md`; every staged PNG is decoded and checked for exact dimensions.

Only a completely verified staging directory can be renamed to the final destination. Replacing an existing directory requires explicit confirmation, and a non-empty directory is replaced only if it contains `report.md` from a previous package. A confirmed existing destination is renamed to `previous` inside the staging directory immediately before commit and renamed back if the final rename fails. If that restore also fails, the staging directory is kept and the error names the path of the preserved copy. Cancellation is checked during rendering and immediately before commit.

The core owns these rules. The Tauri adapter exposes destination inspection, one active background job, progress events, and cancellation; the frontend cannot bypass overwrite confirmation or staged verification.

## Consequences

- Failed and cancelled jobs leave no falsely complete package.
- A valid existing package survives replacement failure.
- Final commit relies on same-parent filesystem rename behavior.
- Export requires temporary free space for both a new package and, during overwrite, the previous package.
- The combined ZIP is written inside the staging directory from the same resolved paths and verified with the rest of the package before commit.

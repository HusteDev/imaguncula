# Preset review process

Presets decide exactly which files Imaguncula generates, so review a preset change as carefully as a code change. Work from the platform vendor's own documentation, not from secondary articles or other generators.

## When to review

Review a preset when:

1. a vendor changes its asset-generation documentation, manifest schema, store submission rules, or platform tooling;
2. a CI fixture stops compiling or packaging;
3. a user reports a generated asset rejected by a current vendor tool;
4. 12 months have elapsed since `lastReviewed`.

Open every URL in `specSources`, confirm it still loads, and record the date you reviewed it as `YYYY-MM-DD`. Replace retired pages with the vendor's replacement. Only update `lastReviewed` if you actually read the sources.

## Change procedure

1. Compare current vendor requirements with the newest immutable preset version.
2. Record each required filename, dimensions, format, role, optionality, package path, and validation rule in a review worksheet or issue.
3. If requirements are unchanged, update only `lastReviewed` in the current unreleased version. Released version directories are immutable.
4. If requirements changed, copy the newest version to the next integer directory. Never modify or delete a released version.
5. Give renamed logical outputs a `renamedFrom` value. Do not infer renames from similar names or dimensions.
6. Put non-required families in an `optionalOutputGroup`; set `defaultEnabled` from the vendor requirement, not preference.
7. Update or add a fixture that consumes production CLI output through the vendor tool where one is available.
8. Run schema, semantic-reference, core export, determinism, CLI-equivalence, and platform fixture checks.
9. Open an older saved project and confirm the UI lists added, removed, and renamed outputs before acceptance. Confirm acceptance writes the diff fingerprint to `presetMigrations`.
10. Update format documentation and `EXECUTION-PLAN.md` evidence in the same change.

## CI enforcement

The `imaguncula-presets` tests compile the Draft 2020-12 schema and validate every checked-in preset against it. The review-metadata test also requires at least one HTTPS `specSources` entry and a valid `lastReviewed` date. Other tests reject duplicate IDs and references to roles, outputs, packages, metadata, or optional groups that do not exist. Separate workflows run the vendors' own tools on generated Apple catalogs, Android resources, and MSIX packages.

CI can confirm the metadata and the structure of what gets generated. It cannot tell whether a vendor page was read correctly, whether a store will accept the result, or whether the artwork looks right; the reviewer has to judge those.

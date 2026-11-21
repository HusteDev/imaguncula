# Preset review process

Preset data is vendor-derived executable configuration. Review it as code, not as a list copied from a secondary article.

## Scheduled and change-triggered review

Review a preset when any of these occurs:

1. a vendor changes its asset-generation documentation, manifest schema, store submission rules, or platform tooling;
2. a CI fixture stops compiling or packaging;
3. a user reports a generated asset rejected by a current vendor tool;
4. 12 months have elapsed since `lastReviewed`.

Use primary vendor documentation. Open every URL in `specSources`, confirm it still resolves, and record the actual review date in ISO `YYYY-MM-DD` form. Replace retired sources with their official successors. Do not update `lastReviewed` when the sources were not inspected.

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

`imaguncula-presets` tests compile the Draft 2020-12 schema and validate every checked-in preset. The review-metadata test additionally requires at least one HTTPS `specSources` entry and a schema-validated `lastReviewed` date. Semantic tests reject duplicate identities and broken role, output, package, metadata, and optional-group references. Vendor-tool workflows provide independent structural checks for Apple catalogs, Android resources, and MSIX packages.

CI confirms checked-in metadata and generated structure. It cannot prove that a vendor page was interpreted correctly, that a store will accept an artifact, or that visual artwork is suitable. Those remain review responsibilities.

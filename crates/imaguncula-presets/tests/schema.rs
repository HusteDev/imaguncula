// SPDX-License-Identifier: MIT OR Apache-2.0
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

use serde_json::Value;

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn load_json(path: &Path) -> Value {
    let contents = fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));

    serde_json::from_str(&contents)
        .unwrap_or_else(|error| panic!("invalid JSON in {}: {error}", path.display()))
}

fn collect_json_files(root: &Path, paths: &mut Vec<PathBuf>) {
    let mut entries = fs::read_dir(root)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", root.display()))
        .collect::<Result<Vec<_>, _>>()
        .unwrap_or_else(|error| panic!("failed to enumerate {}: {error}", root.display()));
    entries.sort_by_key(std::fs::DirEntry::path);

    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            collect_json_files(&path, paths);
        } else if path
            .extension()
            .is_some_and(|extension| extension == "json")
        {
            paths.push(path);
        }
    }
}

fn preset_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    collect_json_files(&manifest_dir().join("presets"), &mut paths);
    collect_json_files(&manifest_dir().join("../../fixtures/presets"), &mut paths);
    paths
}

fn schema() -> Value {
    load_json(&manifest_dir().join("schema/preset.schema.json"))
}

fn validate_schema_instance(validator: &jsonschema::Validator, preset: &Value, path: &Path) {
    let errors = validator
        .iter_errors(preset)
        .map(|error| format!("{}: {error}", error.instance_path()))
        .collect::<Vec<_>>();

    assert!(
        errors.is_empty(),
        "{} failed schema validation:\n{}",
        path.display(),
        errors.join("\n")
    );
}

fn object_array<'a>(preset: &'a Value, property: &str, path: &Path) -> &'a [Value] {
    preset[property]
        .as_array()
        .unwrap_or_else(|| panic!("{}: {property} must be an array", path.display()))
}

fn string_property<'a>(value: &'a Value, property: &str, path: &Path) -> &'a str {
    value[property]
        .as_str()
        .unwrap_or_else(|| panic!("{}: {property} must be a string", path.display()))
}

fn assert_unique<'a>(values: impl IntoIterator<Item = &'a str>, label: &str, path: &Path) {
    let mut seen = HashSet::new();
    for value in values {
        assert!(
            seen.insert(value),
            "{}: duplicate {label} `{value}`",
            path.display()
        );
    }
}

fn validate_references(preset: &Value, path: &Path) {
    let source_roles = object_array(preset, "sourceRoles", path);
    let outputs = object_array(preset, "outputs", path);
    let packages = object_array(preset, "packages", path);
    let optional_groups = object_array(preset, "optionalOutputGroups", path);

    let source_role_ids = collect_unique_ids(source_roles, "role", "source role", path);
    let output_ids = collect_unique_ids(outputs, "id", "output ID", path);
    for output in outputs {
        let role = string_property(output, "sourceRole", path);
        assert!(
            source_role_ids.contains(role),
            "{}: output references unknown source role `{role}`",
            path.display()
        );
    }

    let package_ids = collect_unique_ids(packages, "id", "package ID", path);
    let package_outputs = validate_package_outputs(packages, &output_ids, path);
    validate_package_conditions(packages, &output_ids, path);
    validate_metadata_references(packages, &package_outputs, path);
    validate_optional_groups(optional_groups, &output_ids, &package_ids, path);
}

fn validate_package_conditions(packages: &[Value], output_ids: &HashSet<&str>, path: &Path) {
    for package in packages {
        let required = package["requiresOutputs"].as_array();
        let excluded = package["excludesOutputs"].as_array();
        for (property, values) in [("requiresOutputs", required), ("excludesOutputs", excluded)] {
            for output in values.into_iter().flatten() {
                let output_id = output.as_str().unwrap_or_else(|| {
                    panic!("{}: {property} entries must be strings", path.display())
                });
                assert!(
                    output_ids.contains(output_id),
                    "{}: {property} references unknown output `{output_id}`",
                    path.display()
                );
            }
        }
        if let (Some(required), Some(excluded)) = (required, excluded) {
            let excluded = excluded.iter().collect::<HashSet<_>>();
            assert!(
                required.iter().all(|output| !excluded.contains(output)),
                "{}: one output occurs in both metadata conditions",
                path.display()
            );
        }
    }
}

fn collect_unique_ids<'a>(
    values: &'a [Value],
    property: &str,
    label: &str,
    path: &Path,
) -> HashSet<&'a str> {
    assert_unique(
        values
            .iter()
            .map(|value| string_property(value, property, path)),
        label,
        path,
    );
    values
        .iter()
        .map(|value| string_property(value, property, path))
        .collect()
}

fn validate_package_outputs<'a>(
    packages: &'a [Value],
    output_ids: &HashSet<&str>,
    path: &Path,
) -> HashMap<&'a str, HashSet<&'a str>> {
    let mut package_outputs = HashMap::new();
    for package in packages {
        let package_id = string_property(package, "id", path);
        let package_kind = string_property(package, "kind", path);
        let members = match package_kind {
            "files" | "directory" => object_array(package, "files", path),
            "container" => object_array(package, "entries", path),
            "metadata" => &[],
            unknown => panic!("{}: unknown package kind `{unknown}`", path.display()),
        };

        let mut member_ids = HashSet::new();
        for member in members {
            let output_id = string_property(member, "output", path);
            assert!(
                output_ids.contains(output_id),
                "{}: package `{package_id}` references unknown output `{output_id}`",
                path.display()
            );
            assert!(
                member_ids.insert(output_id),
                "{}: package `{package_id}` contains output `{output_id}` twice",
                path.display()
            );
        }
        package_outputs.insert(package_id, member_ids);
    }

    package_outputs
}

fn validate_metadata_references(
    packages: &[Value],
    package_outputs: &HashMap<&str, HashSet<&str>>,
    path: &Path,
) {
    for package in packages {
        if string_property(package, "kind", path) != "metadata" {
            continue;
        }

        let template = string_property(package, "template", path);
        let references = object_array(package, "references", path);
        assert_unique(
            references
                .iter()
                .map(|reference| string_property(reference, "name", path)),
            "metadata reference name",
            path,
        );

        for reference in references {
            let name = string_property(reference, "name", path);
            let referenced_package = string_property(reference, "package", path);
            let referenced_output = string_property(reference, "output", path);
            let members = package_outputs.get(referenced_package).unwrap_or_else(|| {
                panic!(
                    "{}: metadata reference `{name}` names unknown package `{referenced_package}`",
                    path.display()
                )
            });
            assert!(
                members.contains(referenced_output),
                "{}: output `{referenced_output}` is not in package `{referenced_package}`",
                path.display()
            );
            assert!(
                template.contains(&format!("{{{{{name}}}}}")),
                "{}: metadata template does not use reference `{name}`",
                path.display()
            );
        }
    }
}

fn validate_optional_groups(
    optional_groups: &[Value],
    output_ids: &HashSet<&str>,
    package_ids: &HashSet<&str>,
    path: &Path,
) {
    assert_unique(
        optional_groups
            .iter()
            .map(|group| string_property(group, "id", path)),
        "optional output group ID",
        path,
    );
    for group in optional_groups {
        if let Some(group_outputs) = group["outputs"].as_array() {
            for output in group_outputs {
                let output_id = output
                    .as_str()
                    .unwrap_or_else(|| panic!("{}: group output must be a string", path.display()));
                assert!(
                    output_ids.contains(output_id),
                    "{}: optional group references unknown output `{output_id}`",
                    path.display()
                );
            }
        }
        if let Some(group_packages) = group["packages"].as_array() {
            for package in group_packages {
                let package_id = package.as_str().unwrap_or_else(|| {
                    panic!("{}: group package must be a string", path.display())
                });
                assert!(
                    package_ids.contains(package_id),
                    "{}: optional group references unknown package `{package_id}`",
                    path.display()
                );
            }
        }
    }
}

#[test]
fn preset_schema_is_valid_draft_2020_12() {
    let schema = schema();
    jsonschema::meta::validate(&schema)
        .unwrap_or_else(|error| panic!("preset.schema.json is invalid: {error}"));
}

#[test]
fn all_checked_in_presets_validate() {
    let schema = schema();
    let validator = jsonschema::options()
        .should_validate_formats(true)
        .build(&schema)
        .expect("preset schema should compile");
    let paths = preset_paths();

    assert!(!paths.is_empty(), "at least one preset must be checked in");
    for path in paths {
        let preset = load_json(&path);
        validate_schema_instance(&validator, &preset, &path);
        validate_references(&preset, &path);
    }
}

#[test]
fn fixtures_cover_every_packaging_shape() {
    let mut package_kinds = HashSet::new();
    for path in preset_paths() {
        let preset = load_json(&path);
        for package in object_array(&preset, "packages", &path) {
            package_kinds.insert(string_property(package, "kind", &path).to_owned());
        }
    }

    assert_eq!(
        package_kinds,
        HashSet::from([
            "container".to_owned(),
            "directory".to_owned(),
            "files".to_owned(),
            "metadata".to_owned(),
        ])
    );
}

#[test]
fn schema_rejects_traversal_and_unknown_fields() {
    let schema = schema();
    let validator = jsonschema::validator_for(&schema).expect("preset schema should compile");
    let path = manifest_dir().join("presets/windows-desktop/1/preset.json");
    let preset = load_json(&path);

    let mut traversal = preset.clone();
    traversal["packages"][0]["path"] = Value::String("../escape.ico".to_owned());
    assert!(!validator.is_valid(&traversal));

    let mut absolute = preset.clone();
    absolute["packages"][0]["path"] = Value::String("C:/escape.ico".to_owned());
    assert!(!validator.is_valid(&absolute));

    let mut unknown = preset;
    unknown["surprise"] = Value::Bool(true);
    assert!(!validator.is_valid(&unknown));
}

#[test]
fn immutable_preset_directory_matches_identity() {
    let presets_root = manifest_dir().join("presets");
    let mut paths = Vec::new();
    collect_json_files(&presets_root, &mut paths);

    for path in paths {
        let preset = load_json(&path);
        let relative = path
            .strip_prefix(&presets_root)
            .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        let parts = relative.iter().collect::<Vec<_>>();
        assert_eq!(parts.len(), 3, "unexpected preset path: {}", path.display());
        assert_eq!(
            parts[0].to_string_lossy(),
            string_property(&preset, "id", &path)
        );
        assert_eq!(parts[1].to_string_lossy(), preset["version"].to_string());
        assert_eq!(parts[2].to_string_lossy(), "preset.json");
    }
}

#[test]
fn preset_review_metadata_is_present_and_auditable() {
    let presets_root = manifest_dir().join("presets");
    let mut paths = Vec::new();
    collect_json_files(&presets_root, &mut paths);

    for path in paths {
        let preset = load_json(&path);
        let reviewed = string_property(&preset, "lastReviewed", &path);
        assert_eq!(
            reviewed.len(),
            10,
            "{}: lastReviewed must be an ISO 8601 calendar date",
            path.display()
        );
        let sources = object_array(&preset, "specSources", &path);
        assert!(
            !sources.is_empty(),
            "{}: at least one vendor specification source is required",
            path.display()
        );
        for source in sources {
            let source = source.as_str().unwrap_or_else(|| {
                panic!("{}: specSources entries must be strings", path.display())
            });
            assert!(
                source.starts_with("https://"),
                "{}: specification source must use HTTPS: {source}",
                path.display()
            );
        }
    }
}

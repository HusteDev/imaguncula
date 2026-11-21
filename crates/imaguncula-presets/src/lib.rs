// SPDX-License-Identifier: MIT OR Apache-2.0
#![doc = "Typed, versioned platform-preset data bundled with Imaguncula."]

use std::collections::{BTreeSet, HashSet};

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Initial preset data schema version.
pub const PRESET_SCHEMA_VERSION: u32 = 1;

/// Presets selected for a new project under ADR 0004.
pub const DEFAULT_PRESET_IDENTITIES: &[(&str, u32)] = &[("windows-desktop", 2), ("web-pwa", 1)];

const WINDOWS_DESKTOP_V1: &str = include_str!("../presets/windows-desktop/1/preset.json");
const WINDOWS_DESKTOP_V2: &str = include_str!("../presets/windows-desktop/2/preset.json");
const WEB_PWA_V1: &str = include_str!("../presets/web-pwa/1/preset.json");
const LINUX_HICOLOR_V1: &str = include_str!("../presets/linux-hicolor/1/preset.json");
const APPLE_IOS_MODERN_V1: &str = include_str!("../presets/apple-ios-modern/1/preset.json");
const APPLE_IOS_COMPATIBILITY_V1: &str =
    include_str!("../presets/apple-ios-compatibility/1/preset.json");
const APPLE_MACOS_APPICON_V1: &str = include_str!("../presets/apple-macos-appicon/1/preset.json");
const APPLE_MACOS_ICNS_V1: &str = include_str!("../presets/apple-macos-icns/1/preset.json");
const APPLE_ICON_COMPOSER_GUIDANCE_V1: &str =
    include_str!("../presets/apple-icon-composer-guidance/1/preset.json");
const ANDROID_LAUNCHER_V1: &str = include_str!("../presets/android-launcher/1/preset.json");
const WINDOWS_MSIX_V1: &str = include_str!("../presets/windows-msix/1/preset.json");
const WINDOWS_MSIX_V2: &str = include_str!("../presets/windows-msix/2/preset.json");

const BUNDLED_PRESETS: &[(&str, u32, &str)] = &[
    ("windows-desktop", 1, WINDOWS_DESKTOP_V1),
    ("windows-desktop", 2, WINDOWS_DESKTOP_V2),
    ("web-pwa", 1, WEB_PWA_V1),
    ("linux-hicolor", 1, LINUX_HICOLOR_V1),
    ("apple-ios-modern", 1, APPLE_IOS_MODERN_V1),
    ("apple-ios-compatibility", 1, APPLE_IOS_COMPATIBILITY_V1),
    ("apple-macos-appicon", 1, APPLE_MACOS_APPICON_V1),
    ("apple-macos-icns", 1, APPLE_MACOS_ICNS_V1),
    (
        "apple-icon-composer-guidance",
        1,
        APPLE_ICON_COMPOSER_GUIDANCE_V1,
    ),
    ("android-launcher", 1, ANDROID_LAUNCHER_V1),
    ("windows-msix", 1, WINDOWS_MSIX_V1),
    ("windows-msix", 2, WINDOWS_MSIX_V2),
];

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PresetDefinition {
    pub schema_version: u32,
    pub id: String,
    pub version: u32,
    pub display_name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub exclusive_group: Option<String>,
    pub platform: Platform,
    pub last_reviewed: String,
    pub spec_sources: Vec<String>,
    pub source_roles: Vec<SourceRole>,
    pub outputs: Vec<PresetOutput>,
    pub packages: Vec<PresetPackage>,
    pub validation_rules: Vec<ValidationRule>,
    pub optional_output_groups: Vec<OptionalOutputGroup>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    Windows,
    Web,
    Linux,
    Macos,
    Ios,
    Ipados,
    Android,
    Custom,
}

impl Platform {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Windows => "windows",
            Self::Web => "web",
            Self::Linux => "linux",
            Self::Macos => "macos",
            Self::Ios => "ios",
            Self::Ipados => "ipados",
            Self::Android => "android",
            Self::Custom => "custom",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SourceRole {
    pub role: String,
    pub required: bool,
    pub description: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PresetOutput {
    pub id: String,
    #[serde(default)]
    pub renamed_from: Option<String>,
    pub source_role: String,
    pub format: OutputFormat,
    pub width: Option<u32>,
    pub height: Option<u32>,
    #[serde(default)]
    pub maximum_bytes: Option<u64>,
    pub usage: String,
    pub encoder: OutputEncoder,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    Png,
    Svg,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase", deny_unknown_fields)]
pub enum OutputEncoder {
    Png {
        compression: PngCompression,
        filter: PngFilter,
        #[serde(default, rename = "flattenColor")]
        flatten_color: Option<RgbColor>,
    },
    Svg {
        mode: SvgMode,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RgbColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PngCompression {
    Fast,
    Default,
    Best,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PngFilter {
    Adaptive,
    None,
    Sub,
    Up,
    Average,
    Paeth,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SvgMode {
    SanitizedSource,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
pub enum PresetPackage {
    Files {
        id: String,
        files: Vec<PackagedFile>,
    },
    Container {
        id: String,
        format: ContainerFormat,
        path: String,
        entries: Vec<ContainerEntry>,
    },
    Directory {
        id: String,
        root: String,
        files: Vec<PackagedFile>,
    },
    Metadata {
        id: String,
        format: MetadataFormat,
        path: String,
        template: String,
        references: Vec<MetadataReference>,
        #[serde(default, rename = "requiresOutputs")]
        requires_outputs: Vec<String>,
        #[serde(default, rename = "excludesOutputs")]
        excludes_outputs: Vec<String>,
    },
}

impl PresetPackage {
    #[must_use]
    pub fn id(&self) -> &str {
        match self {
            Self::Files { id, .. }
            | Self::Container { id, .. }
            | Self::Directory { id, .. }
            | Self::Metadata { id, .. } => id,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackagedFile {
    pub output: String,
    pub path: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContainerFormat {
    Ico,
    Icns,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContainerEntry {
    pub output: String,
    pub encoding: ContainerEncoding,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContainerEncoding {
    Bmp,
    Png,
    Argb,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MetadataFormat {
    Json,
    Xml,
    Html,
    Text,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MetadataReference {
    pub name: String,
    pub package: String,
    pub output: String,
    #[serde(default)]
    pub value: MetadataReferenceValue,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MetadataReferenceValue {
    #[default]
    RelativePath,
    ResourceName,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RuleSeverity {
    Error,
    Warning,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "rule",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum ValidationRule {
    SquareSource {
        severity: RuleSeverity,
        source_role: String,
    },
    NoUpscale {
        severity: RuleSeverity,
        source_role: String,
    },
    MinimumSourceSize {
        severity: RuleSeverity,
        source_role: String,
        width: u32,
        height: u32,
    },
    MaskableSafeZone {
        severity: RuleSeverity,
        outputs: Vec<String>,
        minimum_safe_zone_percent: f64,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OptionalOutputGroup {
    pub id: String,
    pub display_name: String,
    pub description: String,
    pub default_enabled: bool,
    pub outputs: Option<Vec<String>>,
    pub packages: Option<Vec<String>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PresetSummary {
    pub id: String,
    pub version: u32,
    pub display_name: String,
    pub exclusive_group: Option<String>,
    pub platform: Platform,
    pub source_roles: Vec<SourceRole>,
    pub output_ids: Vec<String>,
    pub default_output_ids: Vec<String>,
    pub optional_output_groups: Vec<OptionalOutputGroup>,
}

/// One output identity changed explicitly by a newer immutable preset version.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PresetOutputRename {
    pub from: String,
    pub to: String,
}

/// User-reviewable output identity changes between two versions of one preset.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PresetDiff {
    pub preset_id: String,
    pub from_version: u32,
    pub to_version: u32,
    pub added_outputs: Vec<String>,
    pub removed_outputs: Vec<String>,
    pub renamed_outputs: Vec<PresetOutputRename>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PresetDiffError {
    #[error("cannot compare different preset identities `{from}` and `{to}`")]
    IdentityMismatch { from: String, to: String },
    #[error("preset versions must be different")]
    SameVersion,
}

#[derive(Debug, Error)]
pub enum PresetLoadError {
    #[error("bundled preset `{id}` version {version} does not exist")]
    NotFound { id: String, version: u32 },
    #[error("bundled preset `{id}` version {version} is invalid: {message}")]
    Invalid {
        id: String,
        version: u32,
        message: String,
    },
}

/// Loads one immutable bundled preset by exact identity.
///
/// # Errors
///
/// Returns [`PresetLoadError::NotFound`] for unknown identities and
/// [`PresetLoadError::Invalid`] if checked-in data cannot deserialize.
pub fn load_bundled_preset(id: &str, version: u32) -> Result<PresetDefinition, PresetLoadError> {
    let (_, _, contents) = BUNDLED_PRESETS
        .iter()
        .find(|(candidate_id, candidate_version, _)| {
            *candidate_id == id && *candidate_version == version
        })
        .ok_or_else(|| PresetLoadError::NotFound {
            id: id.to_owned(),
            version,
        })?;
    serde_json::from_str(contents).map_err(|error| PresetLoadError::Invalid {
        id: id.to_owned(),
        version,
        message: error.to_string(),
    })
}

/// Returns every immutable preset compiled into this application version.
///
/// # Errors
///
/// Returns [`PresetLoadError::Invalid`] if checked-in data cannot deserialize.
pub fn bundled_presets() -> Result<Vec<PresetDefinition>, PresetLoadError> {
    BUNDLED_PRESETS
        .iter()
        .map(|(id, version, _)| load_bundled_preset(id, *version))
        .collect()
}

/// Returns the newest compiled version for one preset identity.
#[must_use]
pub fn latest_bundled_preset_version(id: &str) -> Option<u32> {
    BUNDLED_PRESETS
        .iter()
        .filter(|(candidate_id, _, _)| *candidate_id == id)
        .map(|(_, version, _)| *version)
        .max()
}

/// Compares output identities between two immutable versions of one preset.
///
/// Explicit `renamedFrom` declarations on destination outputs prevent a rename from being
/// presented as an unrelated removal and addition.
///
/// # Errors
///
/// Returns an error when the definitions have different identities or the same version.
pub fn compare_preset_definitions(
    from: &PresetDefinition,
    to: &PresetDefinition,
) -> Result<PresetDiff, PresetDiffError> {
    if from.id != to.id {
        return Err(PresetDiffError::IdentityMismatch {
            from: from.id.clone(),
            to: to.id.clone(),
        });
    }
    if from.version == to.version {
        return Err(PresetDiffError::SameVersion);
    }

    let from_ids = from
        .outputs
        .iter()
        .map(|output| output.id.clone())
        .collect::<BTreeSet<_>>();
    let to_ids = to
        .outputs
        .iter()
        .map(|output| output.id.clone())
        .collect::<BTreeSet<_>>();
    let mut renamed_outputs = to
        .outputs
        .iter()
        .filter_map(|output| {
            output
                .renamed_from
                .as_ref()
                .filter(|old_id| from_ids.contains(*old_id) && !to_ids.contains(*old_id))
                .map(|old_id| PresetOutputRename {
                    from: old_id.clone(),
                    to: output.id.clone(),
                })
        })
        .collect::<Vec<_>>();
    renamed_outputs.sort_by(|left, right| left.from.cmp(&right.from).then(left.to.cmp(&right.to)));
    let renamed_from = renamed_outputs
        .iter()
        .map(|rename| &rename.from)
        .collect::<HashSet<_>>();
    let renamed_to = renamed_outputs
        .iter()
        .map(|rename| &rename.to)
        .collect::<HashSet<_>>();

    Ok(PresetDiff {
        preset_id: from.id.clone(),
        from_version: from.version,
        to_version: to.version,
        added_outputs: to_ids
            .difference(&from_ids)
            .filter(|id| !renamed_to.contains(id))
            .cloned()
            .collect(),
        removed_outputs: from_ids
            .difference(&to_ids)
            .filter(|id| !renamed_from.contains(id))
            .cloned()
            .collect(),
        renamed_outputs,
    })
}

/// Returns presentation metadata for every compiled preset.
///
/// # Errors
///
/// Returns [`PresetLoadError::Invalid`] if checked-in data cannot deserialize.
pub fn bundled_preset_summaries() -> Result<Vec<PresetSummary>, PresetLoadError> {
    bundled_presets().map(|presets| {
        presets
            .into_iter()
            .filter(|preset| latest_bundled_preset_version(&preset.id) == Some(preset.version))
            .map(|preset| {
                let output_ids = preset
                    .outputs
                    .iter()
                    .map(|output| output.id.clone())
                    .collect::<Vec<_>>();
                let disabled_by_default = preset
                    .optional_output_groups
                    .iter()
                    .filter(|group| !group.default_enabled)
                    .flat_map(|group| group.outputs.iter().flatten())
                    .collect::<HashSet<_>>();
                let default_output_ids = output_ids
                    .iter()
                    .filter(|id| !disabled_by_default.contains(id))
                    .cloned()
                    .collect();
                PresetSummary {
                    output_ids,
                    default_output_ids,
                    id: preset.id,
                    version: preset.version,
                    display_name: preset.display_name,
                    exclusive_group: preset.exclusive_group,
                    platform: preset.platform,
                    source_roles: preset.source_roles,
                    optional_output_groups: preset.optional_output_groups,
                }
            })
            .collect()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preset_schema_starts_at_version_one() {
        assert_eq!(PRESET_SCHEMA_VERSION, 1);
    }

    #[test]
    fn bundled_presets_are_typed_and_exactly_addressable() {
        let presets = bundled_presets().expect("bundled presets deserialize");
        assert_eq!(presets.len(), 12);
        for id in [
            "windows-desktop",
            "web-pwa",
            "linux-hicolor",
            "apple-ios-modern",
            "apple-ios-compatibility",
            "apple-macos-appicon",
            "apple-macos-icns",
            "apple-icon-composer-guidance",
            "android-launcher",
            "windows-msix",
        ] {
            assert!(presets.iter().any(|preset| preset.id == id));
        }
        assert!(matches!(
            load_bundled_preset("missing", 1),
            Err(PresetLoadError::NotFound { .. })
        ));
    }

    #[test]
    fn bundled_output_ids_are_globally_unique() {
        let presets = bundled_presets().expect("bundled presets deserialize");
        let mut output_ids = HashSet::new();
        for preset in presets
            .iter()
            .filter(|preset| latest_bundled_preset_version(&preset.id) == Some(preset.version))
        {
            for output in &preset.outputs {
                assert!(
                    output_ids.insert(output.id.as_str()),
                    "bundled output ID `{}` occurs in more than one current preset",
                    output.id
                );
            }
        }
    }

    #[test]
    fn summaries_expose_only_the_latest_version() {
        let summaries = bundled_preset_summaries().expect("summaries deserialize");
        assert_eq!(summaries.len(), 10);
        let msix = summaries
            .iter()
            .find(|preset| preset.id == "windows-msix")
            .expect("MSIX summary exists");
        assert_eq!(msix.version, 2);
        assert_eq!(msix.output_ids.len(), 134);
        assert_eq!(msix.default_output_ids.len(), 57);
        let windows_desktop = summaries
            .iter()
            .find(|preset| preset.id == "windows-desktop")
            .expect("Windows desktop summary exists");
        assert_eq!(windows_desktop.version, 2);
        assert_eq!(windows_desktop.output_ids.len(), 15);
        assert!(windows_desktop.output_ids.contains(&"icon-42".to_owned()));
    }

    #[test]
    fn preset_diff_separates_added_removed_and_explicitly_renamed_outputs() {
        let mut from = load_bundled_preset("windows-desktop", 1).expect("source preset");
        let mut to = from.clone();
        to.version = 2;
        let removed = to.outputs.remove(0);
        let mut renamed = to.outputs.remove(0);
        let renamed_from = renamed.id.clone();
        renamed.id = "renamed-output".to_owned();
        renamed.renamed_from = Some(renamed_from.clone());
        to.outputs.push(renamed);
        let mut added = to.outputs[0].clone();
        added.id = "added-output".to_owned();
        to.outputs.push(added);

        let diff = compare_preset_definitions(&from, &to).expect("same identity compares");
        assert_eq!(diff.added_outputs, ["added-output"]);
        assert_eq!(diff.removed_outputs, [removed.id]);
        assert_eq!(
            diff.renamed_outputs,
            [PresetOutputRename {
                from: renamed_from,
                to: "renamed-output".to_owned(),
            }]
        );

        from.id = "different".to_owned();
        assert!(matches!(
            compare_preset_definitions(&from, &to),
            Err(PresetDiffError::IdentityMismatch { .. })
        ));
    }
}

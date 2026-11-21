// SPDX-License-Identifier: MIT OR Apache-2.0
//! Custom output resolution and cross-platform package-path safety.

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use imaguncula_presets::{
    OutputEncoder, PresetDefinition, PresetLoadError, PresetOutput, PresetPackage, RgbColor,
    RuleSeverity, ValidationRule, load_bundled_preset,
};

use crate::{
    CanonicalRgba, CustomOutput, OutputFormat, OutputOverride, ProjectSource, ProjectState,
    ProjectValidationIssue, SourceFormat, SourceInfo, decode_png_rgba, import_source,
    render_svg_bounded, resize_raster, source::load_source_data,
};

type SourceProbeCache = HashMap<(String, String), Result<SourceInfo, String>>;

struct ResolvedSource {
    role: String,
    label: String,
    path: Option<String>,
    content_hash: Option<String>,
    is_override: bool,
}

/// Resolved output readiness shown in the review table.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputStatus {
    Ready,
    Warning,
    Error,
}

/// Severity for one machine-readable output issue.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputIssueSeverity {
    Warning,
    Error,
}

/// Physical package shape receiving a logical render output.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OutputPackageKind {
    File,
    ContainerEntry,
    DirectoryFile,
}

/// One output warning or error carried into the table and generation report.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OutputIssue {
    pub code: String,
    pub severity: OutputIssueSeverity,
    pub message: String,
}

/// One fully resolved row consumed by previews, exporters, and reports.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OutputSpec {
    pub id: String,
    pub preset_id: Option<String>,
    pub preset_version: Option<u32>,
    pub included: bool,
    pub platform: String,
    pub usage: String,
    pub width: u32,
    pub height: u32,
    pub maximum_bytes: Option<u64>,
    pub format: OutputFormat,
    pub filename: String,
    pub destination: String,
    pub package_path: String,
    pub package_id: String,
    pub package_kind: OutputPackageKind,
    pub source_role: String,
    pub source_label: String,
    pub source_path: Option<String>,
    pub source_content_hash: Option<String>,
    pub source_override: bool,
    pub allow_upscale: bool,
    pub flatten_color: Option<RgbColor>,
    pub safe_zone_percent: Option<f64>,
    pub status: OutputStatus,
    pub issues: Vec<OutputIssue>,
}

/// Output-resolution failures that prevent producing a trustworthy table.
#[derive(Debug, Error, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum ResolveOutputsError {
    #[error("project validation failed")]
    ProjectValidation { issues: Vec<ProjectValidationIssue> },
    #[error("failed to load selected preset `{id}` version {version}: {message}")]
    Preset {
        id: String,
        version: u32,
        message: String,
    },
}

/// Reasons a package-relative path is rejected on every supported host.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Error, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RelativePathError {
    #[error("path is empty")]
    Empty,
    #[error("path is absolute or drive-qualified")]
    Absolute,
    #[error("backslashes are not accepted in package paths")]
    Backslash,
    #[error("path contains an empty, dot, or parent segment")]
    Traversal,
    #[error("path contains a control or reserved character")]
    ReservedCharacter,
    #[error("path contains a Windows-reserved segment")]
    ReservedName,
    #[error("path segment ends in a dot or space")]
    ReservedEnding,
}

/// Validates and normalizes one forward-slash package-relative path.
///
/// # Errors
///
/// Returns a stable [`RelativePathError`] for absolute, traversal, reserved,
/// ambiguous, or platform-dependent paths.
pub fn sanitize_relative_path(value: &str) -> Result<String, RelativePathError> {
    if value.is_empty() {
        return Err(RelativePathError::Empty);
    }
    if value.starts_with('/') || value.as_bytes().get(1).is_some_and(|byte| *byte == b':') {
        return Err(RelativePathError::Absolute);
    }
    if value.contains('\\') {
        return Err(RelativePathError::Backslash);
    }

    for segment in value.split('/') {
        if segment.is_empty() || segment == "." || segment == ".." {
            return Err(RelativePathError::Traversal);
        }
        if segment.ends_with(['.', ' ']) {
            return Err(RelativePathError::ReservedEnding);
        }
        if segment.chars().any(|character| {
            character.is_control() || matches!(character, '<' | '>' | ':' | '"' | '|' | '?' | '*')
        }) {
            return Err(RelativePathError::ReservedCharacter);
        }
        if is_windows_reserved_name(segment) {
            return Err(RelativePathError::ReservedName);
        }
    }

    Ok(value.to_owned())
}

/// Resolves selected preset outputs and custom rows into one review-table model.
///
/// Source failures become per-row typed issues so incomplete projects remain
/// editable. Structural project failures prevent resolution.
///
/// # Errors
///
/// Returns [`ResolveOutputsError::ProjectValidation`] when persisted project
/// values are structurally invalid.
pub fn resolve_outputs(project: &ProjectState) -> Result<Vec<OutputSpec>, ResolveOutputsError> {
    let validation_issues = project.validation_issues();
    if !validation_issues.is_empty() {
        return Err(ResolveOutputsError::ProjectValidation {
            issues: validation_issues,
        });
    }

    let sources_by_role: HashMap<&str, _> = project
        .sources
        .iter()
        .map(|source| (source.role.as_str(), source))
        .collect();
    let mut source_probes = SourceProbeCache::new();
    let mut specs = Vec::new();
    for selected in &project.selected_presets {
        let preset = load_bundled_preset(&selected.id, selected.version)
            .map_err(|error| preset_resolution_error(&selected.id, selected.version, &error))?;
        specs.extend(resolve_preset_outputs(
            &preset,
            project,
            &sources_by_role,
            &mut source_probes,
        ));
    }
    specs.extend(project.custom_outputs.iter().map(|output| {
        resolve_custom_output(output, project, &sources_by_role, &mut source_probes)
    }));

    mark_collisions(&mut specs);
    Ok(specs)
}

fn preset_resolution_error(id: &str, version: u32, error: &PresetLoadError) -> ResolveOutputsError {
    ResolveOutputsError::Preset {
        id: id.to_owned(),
        version,
        message: error.to_string(),
    }
}

struct ResolvedPackaging {
    id: String,
    kind: OutputPackageKind,
    path: String,
}

fn resolve_preset_outputs(
    preset: &PresetDefinition,
    project: &ProjectState,
    sources_by_role: &HashMap<&str, &ProjectSource>,
    source_probes: &mut SourceProbeCache,
) -> Vec<OutputSpec> {
    preset
        .outputs
        .iter()
        .map(|output| {
            resolve_preset_output(output, preset, project, sources_by_role, source_probes)
        })
        .collect()
}

fn resolve_preset_output(
    output: &PresetOutput,
    preset: &PresetDefinition,
    project: &ProjectState,
    sources_by_role: &HashMap<&str, &ProjectSource>,
    source_probes: &mut SourceProbeCache,
) -> OutputSpec {
    let output_override = project.output_overrides.get(&output.id);
    let width = output_override
        .and_then(|value| value.width)
        .or(output.width)
        .unwrap_or(0);
    let height = output_override
        .and_then(|value| value.height)
        .or(output.height)
        .unwrap_or(0);
    let allow_upscale = output_override.is_some_and(|value| value.allow_upscale);
    let source = resolve_source(&output.source_role, output_override, sources_by_role);
    let mut issues = Vec::new();
    let packaging = resolve_preset_packaging(preset, output, project, &mut issues);
    append_source_issues(
        &source,
        width,
        height,
        allow_upscale,
        source_probes,
        &mut issues,
    );
    append_preset_validation_issues(preset, output, &source, source_probes, &mut issues);

    let format = match output.format {
        imaguncula_presets::OutputFormat::Png => OutputFormat::Png,
        imaguncula_presets::OutputFormat::Svg => OutputFormat::Svg,
    };
    let flatten_color = match output.encoder {
        OutputEncoder::Png { flatten_color, .. } => flatten_color,
        OutputEncoder::Svg { .. } => None,
    };
    if let Some(color) = flatten_color {
        push_issue(
            &mut issues,
            "final-alpha-flattened",
            OutputIssueSeverity::Warning,
            format!(
                "transparent pixels are flattened into the final RGB output over #{:02X}{:02X}{:02X}",
                color.red, color.green, color.blue
            ),
        );
    }
    let safe_zone_percent = preset.validation_rules.iter().find_map(|rule| match rule {
        ValidationRule::MaskableSafeZone {
            outputs,
            minimum_safe_zone_percent,
            ..
        } if outputs.contains(&output.id) => Some(*minimum_safe_zone_percent),
        _ => None,
    });
    let filename = packaging
        .path
        .rsplit('/')
        .next()
        .unwrap_or(&packaging.path)
        .to_owned();
    let destination = packaging
        .path
        .rsplit_once('/')
        .map_or_else(String::new, |(parent, _)| parent.to_owned());

    OutputSpec {
        id: output.id.clone(),
        preset_id: Some(preset.id.clone()),
        preset_version: Some(preset.version),
        included: project.enabled_outputs.contains(&output.id),
        platform: preset.platform.as_str().to_owned(),
        usage: output.usage.clone(),
        width,
        height,
        maximum_bytes: output.maximum_bytes,
        format,
        filename,
        destination,
        package_path: packaging.path,
        package_id: packaging.id,
        package_kind: packaging.kind,
        source_role: source.role,
        source_label: source.label,
        source_path: source.path,
        source_content_hash: source.content_hash,
        source_override: source.is_override,
        allow_upscale,
        flatten_color,
        safe_zone_percent,
        status: status_for(&issues),
        issues,
    }
}

fn resolve_preset_packaging(
    preset: &PresetDefinition,
    output: &PresetOutput,
    project: &ProjectState,
    issues: &mut Vec<OutputIssue>,
) -> ResolvedPackaging {
    let packaging = preset.packages.iter().find_map(|package| match package {
        PresetPackage::Files { id, files } => files
            .iter()
            .find(|file| file.output == output.id)
            .map(|file| (id, OutputPackageKind::File, file.path.clone())),
        PresetPackage::Container {
            id, path, entries, ..
        } => entries
            .iter()
            .any(|entry| entry.output == output.id)
            .then(|| (id, OutputPackageKind::ContainerEntry, path.clone())),
        PresetPackage::Directory { id, root, files } => files
            .iter()
            .find(|file| file.output == output.id)
            .map(|file| {
                (
                    id,
                    OutputPackageKind::DirectoryFile,
                    format!("{root}/{}", file.path),
                )
            }),
        PresetPackage::Metadata { .. } => None,
    });

    let Some((package_id, package_kind, raw_path)) = packaging else {
        push_issue(
            issues,
            "preset-output-unpackaged",
            OutputIssueSeverity::Error,
            "preset output is not assigned to a physical package",
        );
        return ResolvedPackaging {
            id: "unpackaged".to_owned(),
            kind: OutputPackageKind::File,
            path: format!("{}/unpackaged/{}", preset.id, output.id),
        };
    };

    let expanded = raw_path.replace("{base}", &project.base_filename);
    let prefixed = format!("{}/{expanded}", preset.id);
    let package_path = sanitize_relative_path(&prefixed).unwrap_or_else(|error| {
        push_issue(
            issues,
            "invalid-package-path",
            OutputIssueSeverity::Error,
            error.to_string(),
        );
        prefixed
    });
    ResolvedPackaging {
        id: package_id.clone(),
        kind: package_kind,
        path: package_path,
    }
}

fn append_preset_validation_issues(
    preset: &PresetDefinition,
    output: &PresetOutput,
    source: &ResolvedSource,
    source_probes: &SourceProbeCache,
    issues: &mut Vec<OutputIssue>,
) {
    let Some((path, expected_hash)) = source.path.as_ref().zip(source.content_hash.as_ref()) else {
        return;
    };
    let Some(Ok(info)) = source_probes.get(&(path.clone(), expected_hash.clone())) else {
        return;
    };
    if output.format == imaguncula_presets::OutputFormat::Svg && info.format != SourceFormat::Svg {
        push_issue(
            issues,
            "svg-source-required",
            OutputIssueSeverity::Error,
            "this scalable output requires an SVG source",
        );
    }

    for rule in &preset.validation_rules {
        match rule {
            ValidationRule::SquareSource {
                severity,
                source_role,
            } if source_role == &source.role
                && (info.dimensions.width - info.dimensions.height).abs() > f64::EPSILON =>
            {
                push_rule_issue(
                    issues,
                    "source-not-square",
                    *severity,
                    "preset requires a square source",
                );
            }
            ValidationRule::MinimumSourceSize {
                severity,
                source_role,
                width,
                height,
            } if source_role == &source.role
                && info.format == SourceFormat::Png
                && (info.dimensions.width < f64::from(*width)
                    || info.dimensions.height < f64::from(*height)) =>
            {
                push_rule_issue(
                    issues,
                    "source-below-recommended-size",
                    *severity,
                    format!("preset recommends a source of at least {width}x{height}"),
                );
            }
            ValidationRule::MaskableSafeZone {
                severity,
                outputs,
                minimum_safe_zone_percent,
            } if outputs.contains(&output.id) => {
                if let Ok(coverage) =
                    maskable_outside_alpha_percent(path, expected_hash, *minimum_safe_zone_percent)
                    && coverage > 1.0
                {
                    push_rule_issue(
                        issues,
                        "maskable-content-outside-safe-zone",
                        *severity,
                        format!(
                            "{coverage:.1}% of visible alpha lies outside the {minimum_safe_zone_percent:.0}% maskable safe-zone circle; keep only disposable background outside the circle"
                        ),
                    );
                }
            }
            ValidationRule::NoUpscale { .. }
            | ValidationRule::MaskableSafeZone { .. }
            | ValidationRule::SquareSource { .. }
            | ValidationRule::MinimumSourceSize { .. } => {}
        }
    }
}

fn maskable_outside_alpha_percent(
    path: &str,
    expected_hash: &str,
    safe_zone_percent: f64,
) -> Result<f64, String> {
    const SAMPLE_SIZE: u32 = 128;
    let loaded = load_source_data(path, "maskable-probe").map_err(|error| error.to_string())?;
    if loaded.info.content_hash != expected_hash {
        return Err("source content changed during maskable safe-zone analysis".to_owned());
    }
    let rendered = match loaded.info.format {
        SourceFormat::Png => {
            let source = decode_png_rgba(&loaded.bytes).map_err(|error| error.to_string())?;
            resize_raster(&source, SAMPLE_SIZE, SAMPLE_SIZE, true)
                .map_err(|error| error.to_string())?
        }
        SourceFormat::Svg => render_svg_bounded(
            loaded
                .svg_tree
                .as_ref()
                .ok_or_else(|| "sanitized SVG tree is unavailable".to_owned())?,
            SAMPLE_SIZE,
            SAMPLE_SIZE,
        )
        .map_err(|error| error.to_string())?,
    };
    Ok(outside_alpha_percent(&rendered, safe_zone_percent))
}

fn outside_alpha_percent(image: &CanonicalRgba, safe_zone_percent: f64) -> f64 {
    let center_x = f64::from(image.width()) / 2.0;
    let center_y = f64::from(image.height()) / 2.0;
    let radius = f64::from(image.width().min(image.height())) * safe_zone_percent / 200.0;
    let radius_squared = radius * radius;
    let mut total_alpha = 0.0;
    let mut outside_alpha = 0.0;
    for y in 0..image.height() {
        for x in 0..image.width() {
            let pixel_index = usize::try_from((y * image.width() + x) * 4)
                .expect("bounded image index fits usize");
            let alpha = f64::from(image.pixels()[pixel_index + 3]);
            total_alpha += alpha;
            let dx = f64::from(x) + 0.5 - center_x;
            let dy = f64::from(y) + 0.5 - center_y;
            if dx * dx + dy * dy > radius_squared {
                outside_alpha += alpha;
            }
        }
    }
    if total_alpha == 0.0 {
        0.0
    } else {
        outside_alpha * 100.0 / total_alpha
    }
}

fn push_rule_issue(
    issues: &mut Vec<OutputIssue>,
    code: &str,
    severity: RuleSeverity,
    message: impl Into<String>,
) {
    push_issue(
        issues,
        code,
        match severity {
            RuleSeverity::Error => OutputIssueSeverity::Error,
            RuleSeverity::Warning => OutputIssueSeverity::Warning,
        },
        message,
    );
}

fn resolve_custom_output(
    output: &CustomOutput,
    project: &ProjectState,
    sources_by_role: &HashMap<&str, &ProjectSource>,
    source_probes: &mut SourceProbeCache,
) -> OutputSpec {
    let output_override = project.output_overrides.get(&output.id);
    let width = output_override
        .and_then(|value| value.width)
        .unwrap_or(output.width);
    let height = output_override
        .and_then(|value| value.height)
        .unwrap_or(output.height);
    let allow_upscale = output_override.is_some_and(|value| value.allow_upscale);
    let source = resolve_source(&output.source_role, output_override, sources_by_role);
    let mut issues = Vec::new();
    let package_path = resolve_package_path(output, &mut issues);
    append_format_issues(output, &mut issues);
    append_source_issues(
        &source,
        width,
        height,
        allow_upscale,
        source_probes,
        &mut issues,
    );

    OutputSpec {
        id: output.id.clone(),
        preset_id: None,
        preset_version: None,
        included: project.enabled_outputs.contains(&output.id),
        platform: "custom".to_owned(),
        usage: output.usage.clone(),
        width,
        height,
        maximum_bytes: None,
        format: output.format,
        filename: output.filename.clone(),
        destination: output.destination.clone(),
        package_path,
        package_id: "custom-files".to_owned(),
        package_kind: OutputPackageKind::File,
        source_role: source.role,
        source_label: source.label,
        source_path: source.path,
        source_content_hash: source.content_hash,
        source_override: source.is_override,
        allow_upscale,
        flatten_color: None,
        safe_zone_percent: None,
        status: status_for(&issues),
        issues,
    }
}

fn resolve_source(
    default_role: &str,
    output_override: Option<&OutputOverride>,
    sources_by_role: &HashMap<&str, &ProjectSource>,
) -> ResolvedSource {
    if let Some((path, content_hash)) = output_override.and_then(|value| {
        value
            .source_path
            .as_ref()
            .zip(value.source_content_hash.as_ref())
    }) {
        return ResolvedSource {
            role: "override".to_owned(),
            label: "Custom override".to_owned(),
            path: Some(path.clone()),
            content_hash: Some(content_hash.clone()),
            is_override: true,
        };
    }

    let role = output_override
        .and_then(|value| value.source_role.as_deref())
        .unwrap_or(default_role);
    let source = sources_by_role.get(role).copied();
    ResolvedSource {
        role: role.to_owned(),
        label: source.map_or_else(|| role.to_owned(), |value| value.role.clone()),
        path: source.map(|value| value.path.clone()),
        content_hash: source.map(|value| value.content_hash.clone()),
        is_override: output_override
            .and_then(|value| value.source_role.as_ref())
            .is_some(),
    }
}

fn resolve_package_path(output: &CustomOutput, issues: &mut Vec<OutputIssue>) -> String {
    let raw = if output.destination.is_empty() {
        output.filename.clone()
    } else {
        format!("{}/{}", output.destination, output.filename)
    };
    sanitize_relative_path(&raw).unwrap_or_else(|error| {
        push_issue(
            issues,
            "invalid-package-path",
            OutputIssueSeverity::Error,
            error.to_string(),
        );
        raw
    })
}

fn append_format_issues(output: &CustomOutput, issues: &mut Vec<OutputIssue>) {
    if output.format != OutputFormat::Png {
        push_issue(
            issues,
            "unsupported-phase-two-format",
            OutputIssueSeverity::Error,
            "Phase 2 custom exports support PNG only",
        );
    } else if !output.filename.to_ascii_lowercase().ends_with(".png") {
        push_issue(
            issues,
            "png-extension-required",
            OutputIssueSeverity::Error,
            "PNG output filenames must end with .png",
        );
    }
}

fn append_source_issues(
    source: &ResolvedSource,
    width: u32,
    height: u32,
    allow_upscale: bool,
    source_probes: &mut SourceProbeCache,
    issues: &mut Vec<OutputIssue>,
) {
    let (Some(path), Some(expected_hash)) = (&source.path, &source.content_hash) else {
        push_issue(
            issues,
            "source-role-missing",
            OutputIssueSeverity::Error,
            format!("project source role `{}` is not available", source.role),
        );
        return;
    };
    let probe = source_probes
        .entry((path.clone(), expected_hash.clone()))
        .or_insert_with(|| import_source(path, "resolve").map_err(|error| error.to_string()));
    let info = match probe {
        Ok(info) => info,
        Err(message) => {
            push_issue(
                issues,
                "source-invalid-or-missing",
                OutputIssueSeverity::Error,
                message.clone(),
            );
            return;
        }
    };
    if info.content_hash != *expected_hash {
        push_issue(
            issues,
            "source-content-changed",
            OutputIssueSeverity::Error,
            "source contents changed since the project reference was recorded",
        );
    }
    for warning in &info.warnings {
        push_issue(
            issues,
            warning.code(),
            OutputIssueSeverity::Warning,
            warning.to_string(),
        );
    }
    if info.format == SourceFormat::Png
        && (f64::from(width) > info.dimensions.width || f64::from(height) > info.dimensions.height)
    {
        push_issue(
            issues,
            if allow_upscale {
                "raster-upscale-allowed"
            } else {
                "raster-upscale-forbidden"
            },
            if allow_upscale {
                OutputIssueSeverity::Warning
            } else {
                OutputIssueSeverity::Error
            },
            if allow_upscale {
                "raster source will be enlarged by explicit per-output override"
            } else {
                "output exceeds the raster source; enable the per-output override to enlarge"
            },
        );
    }
}

fn mark_collisions(specs: &mut [OutputSpec]) {
    let mut paths: HashMap<String, Vec<usize>> = HashMap::new();
    for (index, spec) in specs.iter().enumerate().filter(|(_, spec)| spec.included) {
        paths
            .entry(spec.package_path.to_ascii_lowercase())
            .or_default()
            .push(index);
    }
    let colliding: HashSet<usize> = paths
        .values()
        .filter(|indices| indices.len() > 1)
        .filter(|indices| !shared_container(specs, indices))
        .flatten()
        .copied()
        .collect();
    for index in colliding {
        let spec = &mut specs[index];
        let path = spec.package_path.clone();
        push_issue(
            &mut spec.issues,
            "output-path-collision",
            OutputIssueSeverity::Error,
            format!("multiple included outputs resolve to `{path}`"),
        );
        spec.status = status_for(&spec.issues);
    }
}

fn shared_container(specs: &[OutputSpec], indices: &[usize]) -> bool {
    let Some(first) = indices.first().map(|index| &specs[*index]) else {
        return false;
    };
    first.package_kind == OutputPackageKind::ContainerEntry
        && indices.iter().all(|index| {
            let candidate = &specs[*index];
            candidate.package_kind == OutputPackageKind::ContainerEntry
                && candidate.preset_id == first.preset_id
                && candidate.package_id == first.package_id
        })
}

fn push_issue(
    issues: &mut Vec<OutputIssue>,
    code: impl Into<String>,
    severity: OutputIssueSeverity,
    message: impl Into<String>,
) {
    issues.push(OutputIssue {
        code: code.into(),
        severity,
        message: message.into(),
    });
}

fn status_for(issues: &[OutputIssue]) -> OutputStatus {
    if issues
        .iter()
        .any(|issue| issue.severity == OutputIssueSeverity::Error)
    {
        OutputStatus::Error
    } else if issues.is_empty() {
        OutputStatus::Ready
    } else {
        OutputStatus::Warning
    }
}

fn is_windows_reserved_name(value: &str) -> bool {
    let stem = value.split('.').next().unwrap_or_default();
    matches!(
        stem.to_ascii_uppercase().as_str(),
        "CON"
            | "PRN"
            | "AUX"
            | "NUL"
            | "COM1"
            | "COM2"
            | "COM3"
            | "COM4"
            | "COM5"
            | "COM6"
            | "COM7"
            | "COM8"
            | "COM9"
            | "LPT1"
            | "LPT2"
            | "LPT3"
            | "LPT4"
            | "LPT5"
            | "LPT6"
            | "LPT7"
            | "LPT8"
            | "LPT9"
    )
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, fs};

    use image::{ImageFormat, Rgba, RgbaImage};
    use proptest::prelude::*;
    use tempfile::tempdir;

    use super::*;
    use crate::{CustomOutput, OutputOverride, ProjectSource};

    fn project_with_source() -> (tempfile::TempDir, ProjectState) {
        let directory = tempdir().expect("temporary directory");
        let source_path = directory.path().join("master.png");
        RgbaImage::from_pixel(64, 64, Rgba([1, 2, 3, 255]))
            .save_with_format(&source_path, ImageFormat::Png)
            .expect("write source");
        let source = import_source(&source_path, "master").expect("import source");
        let mut project = ProjectState::new("Output tests");
        project.selected_presets.clear();
        project.enabled_outputs.clear();
        project.sources.push(ProjectSource {
            role: "master".to_owned(),
            path: source.path,
            content_hash: source.content_hash,
        });
        (directory, project)
    }

    fn custom_output(id: &str, filename: &str, width: u32) -> CustomOutput {
        CustomOutput {
            id: id.to_owned(),
            width,
            height: width,
            format: OutputFormat::Png,
            filename: filename.to_owned(),
            destination: "custom".to_owned(),
            source_role: "master".to_owned(),
            usage: "Test output".to_owned(),
        }
    }

    #[test]
    fn selected_presets_resolve_into_shared_container_and_file_rows() {
        let directory = tempdir().expect("temporary directory");
        let source_path = directory.path().join("master.svg");
        fs::write(
            &source_path,
            r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512"><rect width="512" height="512" rx="80" fill="#336699"/></svg>"##,
        )
        .expect("write SVG source");
        let source = import_source(&source_path, "master").expect("import source");
        let mut project = ProjectState::new("Preset outputs");
        project.sources.push(ProjectSource {
            role: "master".to_owned(),
            path: source.path,
            content_hash: source.content_hash,
        });

        let specs = resolve_outputs(&project).expect("resolve preset outputs");

        assert_eq!(specs.len(), 23);
        assert_eq!(
            specs
                .iter()
                .filter(|spec| spec.preset_id.as_deref() == Some("windows-desktop"))
                .count(),
            15
        );
        assert!(specs.iter().all(|spec| spec.included));
        assert!(specs.iter().all(|spec| spec.status != OutputStatus::Error));
        assert!(specs.iter().any(|spec| {
            spec.id == "pwa-maskable-512"
                && spec
                    .issues
                    .iter()
                    .any(|issue| issue.code == "maskable-content-outside-safe-zone")
        }));
        let windows_paths: HashSet<_> = specs
            .iter()
            .filter(|spec| spec.preset_id.as_deref() == Some("windows-desktop"))
            .map(|spec| spec.package_path.as_str())
            .collect();
        assert_eq!(
            windows_paths,
            HashSet::from(["windows-desktop/preset-outputs.ico"])
        );
        assert!(specs.iter().any(|spec| {
            spec.id == "favicon-svg"
                && spec.package_path == "web-pwa/favicon.svg"
                && spec.width == 0
                && spec.height == 0
        }));
    }

    #[test]
    fn resolves_custom_rows_and_blocks_unapproved_upscale() {
        let (_directory, mut project) = project_with_source();
        project
            .custom_outputs
            .push(custom_output("small", "small.png", 32));
        project
            .custom_outputs
            .push(custom_output("large", "large.png", 128));
        project
            .enabled_outputs
            .extend(["small".to_owned(), "large".to_owned()]);

        let specs = resolve_outputs(&project).expect("resolve outputs");

        assert_eq!(specs[0].package_path, "custom/small.png");
        assert_eq!(specs[0].status, OutputStatus::Ready);
        assert_eq!(specs[1].status, OutputStatus::Error);
        assert!(
            specs[1]
                .issues
                .iter()
                .any(|issue| issue.code == "raster-upscale-forbidden")
        );
    }

    #[test]
    fn maskable_probe_measures_visible_alpha_outside_safe_circle() {
        let opaque = CanonicalRgba::new(10, 10, vec![255; 10 * 10 * 4]).expect("opaque image");
        assert!(outside_alpha_percent(&opaque, 80.0) > 40.0);

        let mut centered = vec![0; 10 * 10 * 4];
        for y in 3..7 {
            for x in 3..7 {
                centered[(y * 10 + x) * 4 + 3] = 255;
            }
        }
        let centered = CanonicalRgba::new(10, 10, centered).expect("centered image");
        assert!(outside_alpha_percent(&centered, 80.0).abs() < f64::EPSILON);
    }

    #[test]
    fn direct_override_and_collision_are_visible_in_table_state() {
        let (directory, mut project) = project_with_source();
        let override_path = directory.path().join("override.png");
        RgbaImage::from_pixel(32, 32, Rgba([4, 5, 6, 255]))
            .save_with_format(&override_path, ImageFormat::Png)
            .expect("write override");
        let override_info = import_source(&override_path, "override").expect("import override");
        project
            .custom_outputs
            .push(custom_output("first", "same.png", 32));
        project
            .custom_outputs
            .push(custom_output("second", "same.png", 32));
        project
            .enabled_outputs
            .extend(["first".to_owned(), "second".to_owned()]);
        project.output_overrides = BTreeMap::from([(
            "first".to_owned(),
            OutputOverride {
                source_path: Some(override_info.path),
                source_content_hash: Some(override_info.content_hash),
                ..OutputOverride::default()
            },
        )]);

        let specs = resolve_outputs(&project).expect("resolve outputs");

        assert!(specs[0].source_override);
        assert!(specs.iter().all(|spec| spec.status == OutputStatus::Error));
        assert!(specs.iter().all(|spec| {
            spec.issues
                .iter()
                .any(|issue| issue.code == "output-path-collision")
        }));
    }

    #[test]
    fn missing_override_source_is_a_row_error_not_a_resolution_failure() {
        let (_directory, mut project) = project_with_source();
        project
            .custom_outputs
            .push(custom_output("missing", "missing.png", 32));
        project.enabled_outputs.insert("missing".to_owned());
        project.output_overrides.insert(
            "missing".to_owned(),
            OutputOverride {
                source_path: Some("missing.png".to_owned()),
                source_content_hash: Some("0".repeat(64)),
                ..OutputOverride::default()
            },
        );

        let specs = resolve_outputs(&project).expect("resolve outputs");

        assert_eq!(specs[0].status, OutputStatus::Error);
        assert!(
            specs[0]
                .issues
                .iter()
                .any(|issue| issue.code == "source-invalid-or-missing")
        );
    }

    #[test]
    fn package_paths_reject_cross_platform_escape_shapes() {
        for hostile in [
            "../escape.png",
            "/absolute.png",
            "C:/drive.png",
            "folder\\escape.png",
            "folder//empty.png",
            "folder/CON.png",
            "folder/trailing. ",
        ] {
            assert!(
                sanitize_relative_path(hostile).is_err(),
                "accepted {hostile}"
            );
        }
        assert_eq!(
            sanitize_relative_path("icons/custom-32.png"),
            Ok("icons/custom-32.png".to_owned())
        );
    }

    #[test]
    fn hostile_path_corpus_is_rejected() {
        for path in include_str!("../../../fixtures/hostile/traversal-paths.txt")
            .lines()
            .filter(|line| !line.is_empty())
        {
            assert!(
                sanitize_relative_path(path).is_err(),
                "hostile fixture unexpectedly accepted: {path}"
            );
        }
    }

    proptest! {
        #[test]
        fn accepted_package_paths_never_escape_or_change(value in ".{0,256}") {
            if let Ok(sanitized) = sanitize_relative_path(&value) {
                prop_assert_eq!(&sanitized, &value);
                prop_assert!(!sanitized.starts_with('/'));
                prop_assert!(!sanitized.contains('\\'));
                prop_assert!(sanitized.split('/').all(|part| !part.is_empty() && part != "." && part != ".."));
            }
        }
    }

    #[test]
    fn changed_source_hash_is_reported() {
        let (directory, mut project) = project_with_source();
        project
            .custom_outputs
            .push(custom_output("changed", "changed.png", 32));
        project.enabled_outputs.insert("changed".to_owned());
        fs::write(directory.path().join("master.png"), b"changed").expect("replace source");

        let specs = resolve_outputs(&project).expect("resolve outputs");

        assert_eq!(specs[0].status, OutputStatus::Error);
    }
}

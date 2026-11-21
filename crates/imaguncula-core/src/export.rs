// SPDX-License-Identifier: MIT OR Apache-2.0
//! Transactional preset and custom-output export with verification and cancellation.

use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    fs::{self, File, OpenOptions},
    io::{Cursor, Read, Write},
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use serde::{Deserialize, Serialize};
use tempfile::Builder;
use thiserror::Error;
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};

use imaguncula_presets::{
    ContainerEncoding, ContainerFormat, MetadataFormat, MetadataReference, MetadataReferenceValue,
    PresetDefinition, PresetPackage, load_bundled_preset,
};

use crate::{
    CanonicalRgba, GeneratedFile, GenerationReport, IconContainerEntry, IconContainerFormat,
    IconEntryEncoding, OutputFormat, OutputSpec, OutputStatus, ProjectState, RenderError,
    ResolveOutputsError, SourceError, SourceFormat, decode_png_rgba, encode_icon_container,
    encode_png_srgb, encode_png_srgb_flattened, render_svg_bounded, resize_raster,
    source::{LoadedSource, load_source_data},
};

/// Cooperative cancellation handle shared by interface adapters and the core.
#[derive(Clone, Default)]
pub struct ExportCancellation {
    cancelled: Arc<AtomicBool>,
}

impl ExportCancellation {
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }

    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }
}

/// Progress update emitted after each verified output write.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExportProgress {
    pub completed: usize,
    pub total: usize,
    pub current_output: Option<String>,
}

/// Read-only destination state used by explicit overwrite confirmation UI.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DestinationState {
    Missing,
    EmptyDirectory,
    /// Non-empty directory containing a previous package marker (`report.md`).
    NonEmptyPackageDirectory,
    /// Non-empty directory without a package marker; never replaced.
    NonEmptyDirectory,
    NonDirectory,
}

/// Read-only destination inspection used by explicit overwrite confirmation UI.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DestinationInspection {
    pub path: String,
    pub state: DestinationState,
    pub requires_confirmation: bool,
}

/// Successful export result returned to GUI and CLI interfaces.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExportReport {
    pub destination: String,
    pub report: GenerationReport,
    pub report_markdown: String,
}

/// Structured export failures. No failure variant represents a complete package.
#[derive(Debug, Error, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum ExportError {
    #[error("output resolution failed: {error}")]
    Resolve { error: ResolveOutputsError },
    #[error("no custom outputs are included")]
    NoIncludedOutputs,
    #[error("included outputs are not ready")]
    OutputsNotReady { output_ids: Vec<String> },
    #[error("export was cancelled")]
    Cancelled,
    #[error("export destination `{path}` has no usable parent directory")]
    InvalidDestination { path: String },
    #[error("export destination `{path}` already exists and requires confirmation")]
    DestinationExists { path: String },
    #[error(
        "export destination `{path}` is not empty and does not contain a previous Imaguncula package (`report.md` is missing); it is never replaced"
    )]
    DestinationNotPackage { path: String },
    #[error("export destination `{path}` exists but is not a directory")]
    DestinationNotDirectory { path: String },
    #[error("failed to inspect export destination `{path}`: {message}")]
    Inspect { path: String, message: String },
    #[error("source import failed for output {output_id}: {error}")]
    Source {
        output_id: String,
        error: SourceError,
    },
    #[error("source contents changed for output {output_id}: `{path}`")]
    SourceChanged { output_id: String, path: String },
    #[error("rendering failed for output {output_id}: {error}")]
    Render {
        output_id: String,
        error: RenderError,
    },
    #[error("preset `{preset_id}` package `{package_id}` uses an unsupported container format")]
    UnsupportedContainer {
        preset_id: String,
        package_id: String,
    },
    #[error("metadata package `{package_id}` references disabled output `{output_id}`")]
    MetadataReferenceMissing {
        package_id: String,
        output_id: String,
    },
    #[error("metadata package `{package_id}` has a partially enabled output requirement")]
    MetadataConditionIncomplete {
        package_id: String,
        missing_output_ids: Vec<String>,
    },
    #[error("metadata generation failed for package `{package_id}`: {message}")]
    Metadata { package_id: String, message: String },
    #[error("container encoding failed for package `{package_id}`: {message}")]
    Container { package_id: String, message: String },
    #[error("combined ZIP generation failed for `{path}`: {message}")]
    Zip { path: String, message: String },
    #[error("failed to write staged export data `{path}`: {message}")]
    Write { path: String, message: String },
    #[error("staged export did not match the resolved output list")]
    Verification {
        missing: Vec<String>,
        unexpected: Vec<String>,
    },
    #[error("failed to commit staged export to `{path}`: {message}")]
    Commit { path: String, message: String },
    #[error(
        "failed to commit staged export to `{path}`: {message}; the previous package was preserved at `{backup_path}`"
    )]
    CommitRestoreFailed {
        path: String,
        backup_path: String,
        message: String,
    },
}

enum ExportSource {
    Png(CanonicalRgba),
    Svg(Arc<usvg::Tree>),
}

struct ArtifactPlan {
    path: String,
    output_indices: Vec<usize>,
    kind: ArtifactKind,
}

enum ArtifactKind {
    Png {
        output_index: usize,
    },
    Svg {
        output_index: usize,
    },
    Ico {
        entries: Vec<IcoEntryPlan>,
    },
    Icns {
        entries: Vec<IcoEntryPlan>,
    },
    Metadata {
        package_id: String,
        format: MetadataFormat,
        contents: String,
        references: Vec<RenderedMetadataReference>,
    },
}

struct RenderedMetadataReference {
    path: String,
    value: String,
}

struct IcoEntryPlan {
    output_index: usize,
    encoding: ContainerEncoding,
}

/// Returns the conventional Phase 2 package directory below a chosen parent.
#[must_use]
pub fn default_export_path(parent: impl AsRef<Path>, base_filename: &str) -> PathBuf {
    parent.as_ref().join(format!("{base_filename}-icons"))
}

/// Inspects a final package path without modifying it.
///
/// # Errors
///
/// Returns [`ExportError::Inspect`] when metadata or directory enumeration fails.
pub fn inspect_export_destination(
    destination: impl AsRef<Path>,
) -> Result<DestinationInspection, ExportError> {
    let destination = destination.as_ref();
    let path = display_path(destination);
    match fs::symlink_metadata(destination) {
        Ok(metadata) => {
            let is_directory = metadata.is_dir() && !metadata.file_type().is_symlink();
            let is_empty = if is_directory {
                fs::read_dir(destination)
                    .map_err(|error| ExportError::Inspect {
                        path: path.clone(),
                        message: error.to_string(),
                    })?
                    .next()
                    .is_none()
            } else {
                false
            };
            Ok(DestinationInspection {
                path,
                state: if !is_directory {
                    DestinationState::NonDirectory
                } else if is_empty {
                    DestinationState::EmptyDirectory
                } else if destination.join("report.md").is_file() {
                    DestinationState::NonEmptyPackageDirectory
                } else {
                    DestinationState::NonEmptyDirectory
                },
                requires_confirmation: true,
            })
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(DestinationInspection {
            path,
            state: DestinationState::Missing,
            requires_confirmation: false,
        }),
        Err(error) => Err(ExportError::Inspect {
            path,
            message: error.to_string(),
        }),
    }
}

/// Renders, packages, verifies, and atomically commits every included output.
///
/// The destination is the final package directory, not its parent. Existing
/// paths require overwrite confirmation, and a non-empty destination is only
/// replaced when it contains a previous package marker (`report.md`).
/// Cancellation and failures drop the staging directory before the final
/// move, except after a failed restore, where staging is preserved because it
/// holds the only copy of the previous package.
///
/// # Errors
///
/// Returns a typed [`ExportError`] for project/output readiness, source changes,
/// cancellation, rendering, staging, verification, or commit failures.
pub fn export_project<F>(
    project: &ProjectState,
    destination: impl AsRef<Path>,
    overwrite: bool,
    cancellation: &ExportCancellation,
    mut on_progress: F,
) -> Result<ExportReport, ExportError>
where
    F: FnMut(ExportProgress),
{
    let specs = crate::resolve_outputs(project).map_err(|error| ExportError::Resolve { error })?;
    let included = ready_output_indices(&specs)?;
    let artifacts = build_artifact_plan(project, &specs, &included)?;
    check_cancelled(cancellation)?;

    let destination = destination.as_ref();
    let (parent, destination_exists) = validate_destination(destination, overwrite)?;
    let (staging, package) = create_staging_package(parent)?;
    stage_artifacts(&specs, &artifacts, &package, cancellation, &mut on_progress)?;

    let report = GenerationReport::from_generated_files(
        project,
        &specs,
        generated_files(&specs, &artifacts),
    );
    let report_markdown = report.to_markdown();
    write_new_file(&package.join("report.md"), report_markdown.as_bytes())?;
    let zip_path = project
        .export_settings
        .create_combined_zip
        .then(|| format!("{}-icons.zip", project.base_filename));
    if let Some(zip_path) = &zip_path {
        write_combined_zip(&package, zip_path)?;
    }
    verify_staged_package(&package, &specs, &artifacts, zip_path.as_deref())?;
    check_cancelled(cancellation)?;
    commit_package(staging, &package, destination, destination_exists)?;

    Ok(ExportReport {
        destination: display_path(destination),
        report,
        report_markdown,
    })
}

fn ready_output_indices(specs: &[OutputSpec]) -> Result<Vec<usize>, ExportError> {
    let included: Vec<_> = specs
        .iter()
        .enumerate()
        .filter(|(_, spec)| spec.included)
        .map(|(index, _)| index)
        .collect();
    if included.is_empty() {
        return Err(ExportError::NoIncludedOutputs);
    }
    let not_ready: Vec<_> = included
        .iter()
        .map(|index| &specs[*index])
        .filter(|spec| spec.status == OutputStatus::Error)
        .map(|spec| spec.id.clone())
        .collect();
    if not_ready.is_empty() {
        Ok(included)
    } else {
        Err(ExportError::OutputsNotReady {
            output_ids: not_ready,
        })
    }
}

fn validate_destination(destination: &Path, overwrite: bool) -> Result<(&Path, bool), ExportError> {
    let parent = destination
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .ok_or_else(|| ExportError::InvalidDestination {
            path: display_path(destination),
        })?;
    if !parent.is_dir() {
        return Err(ExportError::InvalidDestination {
            path: display_path(destination),
        });
    }
    let inspection = inspect_export_destination(destination)?;
    if inspection.state == DestinationState::NonDirectory {
        return Err(ExportError::DestinationNotDirectory {
            path: inspection.path,
        });
    }
    let exists = inspection.state != DestinationState::Missing;
    if exists && !overwrite {
        return Err(ExportError::DestinationExists {
            path: inspection.path,
        });
    }
    if inspection.state == DestinationState::NonEmptyDirectory {
        return Err(ExportError::DestinationNotPackage {
            path: inspection.path,
        });
    }
    Ok((parent, exists))
}

fn create_staging_package(parent: &Path) -> Result<(tempfile::TempDir, PathBuf), ExportError> {
    let staging = Builder::new()
        .prefix(".imaguncula-stage-")
        .tempdir_in(parent)
        .map_err(|error| ExportError::Write {
            path: display_path(parent),
            message: error.to_string(),
        })?;
    let package = staging.path().join("package");
    fs::create_dir(&package).map_err(|error| ExportError::Write {
        path: display_path(&package),
        message: error.to_string(),
    })?;
    Ok((staging, package))
}

fn build_artifact_plan(
    project: &ProjectState,
    specs: &[OutputSpec],
    included: &[usize],
) -> Result<Vec<ArtifactPlan>, ExportError> {
    let included: BTreeSet<_> = included.iter().copied().collect();
    let indices_by_id: HashMap<_, _> = specs
        .iter()
        .enumerate()
        .map(|(index, spec)| (spec.id.as_str(), index))
        .collect();
    let mut artifacts = Vec::new();
    let mut packaged = BTreeSet::new();

    for index in included
        .iter()
        .copied()
        .filter(|index| specs[*index].preset_id.is_none())
    {
        artifacts.push(file_artifact(index, &specs[index]));
        packaged.insert(index);
    }

    for selected in &project.selected_presets {
        let preset = load_bundled_preset(&selected.id, selected.version).map_err(|error| {
            ExportError::Resolve {
                error: ResolveOutputsError::Preset {
                    id: selected.id.clone(),
                    version: selected.version,
                    message: error.to_string(),
                },
            }
        })?;
        append_preset_artifacts(
            project,
            &preset,
            specs,
            &indices_by_id,
            &included,
            &mut packaged,
            &mut artifacts,
        )?;
    }

    let missing: Vec<_> = included
        .difference(&packaged)
        .map(|index| specs[*index].id.clone())
        .collect();
    if !missing.is_empty() {
        return Err(ExportError::OutputsNotReady {
            output_ids: missing,
        });
    }

    let mut paths = BTreeSet::new();
    for artifact in &artifacts {
        if !paths.insert(artifact.path.to_ascii_lowercase()) {
            return Err(ExportError::Verification {
                missing: Vec::new(),
                unexpected: vec![format!("duplicate artifact path: {}", artifact.path)],
            });
        }
    }
    Ok(artifacts)
}

fn append_preset_artifacts(
    project: &ProjectState,
    preset: &PresetDefinition,
    specs: &[OutputSpec],
    indices_by_id: &HashMap<&str, usize>,
    included: &BTreeSet<usize>,
    packaged: &mut BTreeSet<usize>,
    artifacts: &mut Vec<ArtifactPlan>,
) -> Result<(), ExportError> {
    for package in &preset.packages {
        match package {
            PresetPackage::Files { files, .. } | PresetPackage::Directory { files, .. } => {
                for file in files {
                    let Some(index) = indices_by_id.get(file.output.as_str()).copied() else {
                        continue;
                    };
                    if included.contains(&index) {
                        artifacts.push(file_artifact(index, &specs[index]));
                        packaged.insert(index);
                    }
                }
            }
            PresetPackage::Container {
                format, entries, ..
            } => {
                let entry_plans: Vec<_> = entries
                    .iter()
                    .filter_map(|entry| {
                        let index = indices_by_id.get(entry.output.as_str()).copied()?;
                        included.contains(&index).then_some(IcoEntryPlan {
                            output_index: index,
                            encoding: entry.encoding,
                        })
                    })
                    .collect();
                if entry_plans.is_empty() {
                    continue;
                }
                let output_indices = entry_plans
                    .iter()
                    .map(|entry| entry.output_index)
                    .collect::<Vec<_>>();
                let path = specs[output_indices[0]].package_path.clone();
                packaged.extend(output_indices.iter().copied());
                artifacts.push(ArtifactPlan {
                    path,
                    output_indices,
                    kind: match format {
                        ContainerFormat::Ico => ArtifactKind::Ico {
                            entries: entry_plans,
                        },
                        ContainerFormat::Icns => ArtifactKind::Icns {
                            entries: entry_plans,
                        },
                    },
                });
            }
            PresetPackage::Metadata {
                id,
                format,
                path,
                template,
                references,
                requires_outputs,
                excludes_outputs,
            } => {
                let missing_required = requires_outputs
                    .iter()
                    .filter(|output_id| {
                        !indices_by_id
                            .get(output_id.as_str())
                            .is_some_and(|index| included.contains(index))
                    })
                    .cloned()
                    .collect::<Vec<_>>();
                let enabled_required = requires_outputs.len() - missing_required.len();
                if enabled_required > 0 && !missing_required.is_empty() {
                    return Err(ExportError::MetadataConditionIncomplete {
                        package_id: id.clone(),
                        missing_output_ids: missing_required,
                    });
                }
                let required_enabled = requires_outputs.is_empty() || missing_required.is_empty();
                let activated = requires_outputs.is_empty() || enabled_required > 0;
                let excluded_enabled = excludes_outputs.iter().any(|output_id| {
                    indices_by_id
                        .get(output_id.as_str())
                        .is_some_and(|index| included.contains(index))
                });
                if activated && required_enabled && !excluded_enabled {
                    artifacts.push(metadata_artifact(
                        project,
                        preset,
                        id,
                        *format,
                        path,
                        template,
                        references,
                        specs,
                        indices_by_id,
                        included,
                    )?);
                }
            }
        }
    }
    Ok(())
}

fn file_artifact(output_index: usize, spec: &OutputSpec) -> ArtifactPlan {
    ArtifactPlan {
        path: spec.package_path.clone(),
        output_indices: vec![output_index],
        kind: match spec.format {
            OutputFormat::Png => ArtifactKind::Png { output_index },
            OutputFormat::Svg => ArtifactKind::Svg { output_index },
        },
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "metadata templates are resolved against one exact preset package"
)]
fn metadata_artifact(
    project: &ProjectState,
    preset: &PresetDefinition,
    package_id: &str,
    format: MetadataFormat,
    raw_path: &str,
    template: &str,
    references: &[MetadataReference],
    specs: &[OutputSpec],
    indices_by_id: &HashMap<&str, usize>,
    included: &BTreeSet<usize>,
) -> Result<ArtifactPlan, ExportError> {
    let path = crate::sanitize_relative_path(&format!(
        "{}/{}",
        preset.id,
        raw_path.replace("{base}", &project.base_filename)
    ))
    .map_err(|error| ExportError::Metadata {
        package_id: package_id.to_owned(),
        message: error.to_string(),
    })?;
    let mut contents = template.replace("{base}", &project.base_filename);
    let mut rendered_references = Vec::new();
    for reference in references {
        let index = indices_by_id
            .get(reference.output.as_str())
            .copied()
            .filter(|index| included.contains(index))
            .ok_or_else(|| ExportError::MetadataReferenceMissing {
                package_id: package_id.to_owned(),
                output_id: reference.output.clone(),
            })?;
        let spec = &specs[index];
        if spec.package_id != reference.package || spec.preset_id.as_deref() != Some(&preset.id) {
            return Err(ExportError::Metadata {
                package_id: package_id.to_owned(),
                message: format!(
                    "reference `{}` does not resolve to package `{}`",
                    reference.name, reference.package
                ),
            });
        }
        let value = match reference.value {
            MetadataReferenceValue::RelativePath => relative_reference(&path, &spec.package_path),
            MetadataReferenceValue::ResourceName => Path::new(&spec.filename)
                .file_stem()
                .and_then(|stem| stem.to_str())
                .filter(|stem| !stem.is_empty())
                .ok_or_else(|| ExportError::Metadata {
                    package_id: package_id.to_owned(),
                    message: format!("output `{}` has no resource-name stem", spec.id),
                })?
                .to_owned(),
        };
        contents = contents.replace(&format!("{{{{{}}}}}", reference.name), &value);
        rendered_references.push(RenderedMetadataReference {
            path: spec.package_path.clone(),
            value,
        });
    }
    if contents.contains("{{") || contents.contains("}}") {
        return Err(ExportError::Metadata {
            package_id: package_id.to_owned(),
            message: "template contains an unresolved reference token".to_owned(),
        });
    }
    if format == MetadataFormat::Json {
        serde_json::from_str::<serde_json::Value>(&contents).map_err(|error| {
            ExportError::Metadata {
                package_id: package_id.to_owned(),
                message: error.to_string(),
            }
        })?;
    }
    Ok(ArtifactPlan {
        path,
        output_indices: Vec::new(),
        kind: ArtifactKind::Metadata {
            package_id: package_id.to_owned(),
            format,
            contents,
            references: rendered_references,
        },
    })
}

fn relative_reference(from_file: &str, to_file: &str) -> String {
    let mut from = from_file.split('/').collect::<Vec<_>>();
    from.pop();
    let to = to_file.split('/').collect::<Vec<_>>();
    let common = from
        .iter()
        .zip(&to)
        .take_while(|(left, right)| left.eq_ignore_ascii_case(right))
        .count();
    let mut parts = vec![".."; from.len().saturating_sub(common)];
    parts.extend_from_slice(&to[common..]);
    parts.join("/")
}

fn stage_artifacts<F>(
    specs: &[OutputSpec],
    artifacts: &[ArtifactPlan],
    package: &Path,
    cancellation: &ExportCancellation,
    on_progress: &mut F,
) -> Result<(), ExportError>
where
    F: FnMut(ExportProgress),
{
    let total = artifacts
        .iter()
        .map(|artifact| artifact.output_indices.len())
        .sum();
    let mut completed = 0;
    on_progress(ExportProgress {
        completed,
        total,
        current_output: None,
    });
    let mut source_cache = HashMap::new();
    for artifact in artifacts {
        check_cancelled(cancellation)?;
        write_artifact(package, artifact, specs, &mut source_cache)?;
        for index in &artifact.output_indices {
            completed += 1;
            on_progress(ExportProgress {
                completed,
                total,
                current_output: Some(specs[*index].id.clone()),
            });
        }
    }
    check_cancelled(cancellation)
}

fn source_key(spec: &OutputSpec) -> Result<(String, String), ExportError> {
    Ok((
        spec.source_path
            .clone()
            .ok_or_else(|| ExportError::OutputsNotReady {
                output_ids: vec![spec.id.clone()],
            })?,
        spec.source_content_hash
            .clone()
            .ok_or_else(|| ExportError::OutputsNotReady {
                output_ids: vec![spec.id.clone()],
            })?,
    ))
}

fn write_artifact(
    package: &Path,
    artifact: &ArtifactPlan,
    specs: &[OutputSpec],
    source_cache: &mut HashMap<(String, String), ExportSource>,
) -> Result<(), ExportError> {
    let bytes = match &artifact.kind {
        ArtifactKind::Png { output_index } => {
            let spec = &specs[*output_index];
            let rendered = render_output(spec, source_cache)?;
            let encoded = spec.flatten_color.map_or_else(
                || encode_png_srgb(&rendered),
                |color| encode_png_srgb_flattened(&rendered, [color.red, color.green, color.blue]),
            );
            encoded.map_err(|error| ExportError::Render {
                output_id: spec.id.clone(),
                error,
            })?
        }
        ArtifactKind::Svg { output_index } => {
            let spec = &specs[*output_index];
            let source = source_for(spec, source_cache)?;
            let ExportSource::Svg(tree) = source else {
                return Err(ExportError::OutputsNotReady {
                    output_ids: vec![spec.id.clone()],
                });
            };
            tree.to_string(&usvg::WriteOptions::default()).into_bytes()
        }
        ArtifactKind::Ico { entries } => encode_ico(entries, specs, source_cache)?,
        ArtifactKind::Icns { entries } => encode_icns(entries, specs, source_cache)?,
        ArtifactKind::Metadata { contents, .. } => contents.as_bytes().to_vec(),
    };
    let output_path = package.join(relative_to_path(&artifact.path));
    if let Some(output_parent) = output_path.parent() {
        fs::create_dir_all(output_parent).map_err(|error| ExportError::Write {
            path: display_path(output_parent),
            message: error.to_string(),
        })?;
    }
    write_new_file(&output_path, &bytes)
}

fn source_for<'a>(
    spec: &OutputSpec,
    source_cache: &'a mut HashMap<(String, String), ExportSource>,
) -> Result<&'a ExportSource, ExportError> {
    let key = source_key(spec)?;
    if !source_cache.contains_key(&key) {
        let loaded = load_source_data(&key.0, "export").map_err(|error| ExportError::Source {
            output_id: spec.id.clone(),
            error,
        })?;
        if loaded.info.content_hash != key.1 {
            return Err(ExportError::SourceChanged {
                output_id: spec.id.clone(),
                path: key.0,
            });
        }
        source_cache.insert(key.clone(), export_source(loaded, spec)?);
    }
    source_cache
        .get(&key)
        .ok_or_else(|| ExportError::OutputsNotReady {
            output_ids: vec![spec.id.clone()],
        })
}

fn render_output(
    spec: &OutputSpec,
    source_cache: &mut HashMap<(String, String), ExportSource>,
) -> Result<CanonicalRgba, ExportError> {
    let source = source_for(spec, source_cache)?;
    match source {
        ExportSource::Png(source) => {
            resize_raster(source, spec.width, spec.height, spec.allow_upscale)
        }
        ExportSource::Svg(tree) => render_svg_bounded(tree, spec.width, spec.height),
    }
    .map_err(|error| ExportError::Render {
        output_id: spec.id.clone(),
        error,
    })
}

fn encode_ico(
    entries: &[IcoEntryPlan],
    specs: &[OutputSpec],
    source_cache: &mut HashMap<(String, String), ExportSource>,
) -> Result<Vec<u8>, ExportError> {
    let package_id = entries.first().map_or_else(
        || "ico".to_owned(),
        |entry| specs[entry.output_index].package_id.clone(),
    );
    let mut rendered_entries = Vec::with_capacity(entries.len());
    for entry in entries {
        let rendered = render_output(&specs[entry.output_index], source_cache)?;
        rendered_entries.push(IconContainerEntry {
            image: rendered,
            encoding: icon_entry_encoding(entry.encoding),
        });
    }
    encode_icon_container(IconContainerFormat::Ico, rendered_entries).map_err(|error| {
        ExportError::Container {
            package_id,
            message: error.to_string(),
        }
    })
}

fn encode_icns(
    entries: &[IcoEntryPlan],
    specs: &[OutputSpec],
    source_cache: &mut HashMap<(String, String), ExportSource>,
) -> Result<Vec<u8>, ExportError> {
    let package_id = entries.first().map_or_else(
        || "icns".to_owned(),
        |entry| specs[entry.output_index].package_id.clone(),
    );
    let mut rendered_entries = Vec::with_capacity(entries.len());
    for entry in entries {
        let rendered = render_output(&specs[entry.output_index], source_cache)?;
        rendered_entries.push(IconContainerEntry {
            image: rendered,
            encoding: icon_entry_encoding(entry.encoding),
        });
    }
    encode_icon_container(IconContainerFormat::Icns, rendered_entries).map_err(|error| {
        ExportError::Container {
            package_id,
            message: error.to_string(),
        }
    })
}

const fn icon_entry_encoding(encoding: ContainerEncoding) -> IconEntryEncoding {
    match encoding {
        ContainerEncoding::Bmp => IconEntryEncoding::Bmp,
        ContainerEncoding::Png => IconEntryEncoding::Png,
        ContainerEncoding::Argb => IconEntryEncoding::Argb,
    }
}

fn export_source(loaded: LoadedSource, spec: &OutputSpec) -> Result<ExportSource, ExportError> {
    match loaded.info.format {
        SourceFormat::Png => decode_png_rgba(&loaded.bytes)
            .map(ExportSource::Png)
            .map_err(|error| ExportError::Render {
                output_id: spec.id.clone(),
                error,
            }),
        SourceFormat::Svg => {
            loaded
                .svg_tree
                .map(ExportSource::Svg)
                .ok_or_else(|| ExportError::OutputsNotReady {
                    output_ids: vec![spec.id.clone()],
                })
        }
    }
}

fn verify_staged_package(
    package: &Path,
    specs: &[OutputSpec],
    artifacts: &[ArtifactPlan],
    zip_path: Option<&str>,
) -> Result<(), ExportError> {
    let expected: BTreeSet<_> = artifacts
        .iter()
        .map(|artifact| artifact.path.clone())
        .chain(std::iter::once("report.md".to_owned()))
        .chain(zip_path.map(str::to_owned))
        .collect();
    let mut actual = BTreeSet::new();
    collect_staged_files(package, package, &mut actual)?;

    let missing: Vec<_> = expected.difference(&actual).cloned().collect();
    let unexpected: Vec<_> = actual.difference(&expected).cloned().collect();
    if !missing.is_empty() || !unexpected.is_empty() {
        return Err(ExportError::Verification {
            missing,
            unexpected,
        });
    }

    for artifact in artifacts {
        verify_artifact(package, artifact, specs)?;
    }
    if let Some(zip_path) = zip_path {
        verify_combined_zip(package, zip_path, &expected)?;
    }
    Ok(())
}

fn generated_files(specs: &[OutputSpec], artifacts: &[ArtifactPlan]) -> Vec<GeneratedFile> {
    artifacts
        .iter()
        .map(|artifact| {
            let first = artifact.output_indices.first().map(|index| &specs[*index]);
            let output_id = if artifact.output_indices.is_empty() {
                match &artifact.kind {
                    ArtifactKind::Metadata { package_id, .. } => package_id.clone(),
                    _ => "generated".to_owned(),
                }
            } else {
                artifact
                    .output_indices
                    .iter()
                    .map(|index| specs[*index].id.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            let (width, height) = if artifact.output_indices.len() == 1
                && matches!(&artifact.kind, ArtifactKind::Png { .. })
            {
                first.map_or((None, None), |spec| (Some(spec.width), Some(spec.height)))
            } else {
                (None, None)
            };
            let format = match &artifact.kind {
                ArtifactKind::Png { .. } => "png",
                ArtifactKind::Svg { .. } => "svg",
                ArtifactKind::Ico { .. } => "ico",
                ArtifactKind::Icns { .. } => "icns",
                ArtifactKind::Metadata { format, .. } => match format {
                    MetadataFormat::Json => "json",
                    MetadataFormat::Xml => "xml",
                    MetadataFormat::Html => "html",
                    MetadataFormat::Text => "text",
                },
            };
            GeneratedFile {
                output_id,
                path: artifact.path.clone(),
                width,
                height,
                format: format.to_owned(),
                source_label: first
                    .map_or_else(|| "generated".to_owned(), |spec| spec.source_label.clone()),
                source_path: first
                    .and_then(|spec| spec.source_path.clone())
                    .unwrap_or_default(),
                source_content_hash: first
                    .and_then(|spec| spec.source_content_hash.clone())
                    .unwrap_or_default(),
            }
        })
        .collect()
}

fn verify_artifact(
    package: &Path,
    artifact: &ArtifactPlan,
    specs: &[OutputSpec],
) -> Result<(), ExportError> {
    let bytes = fs::read(package.join(relative_to_path(&artifact.path))).map_err(|error| {
        ExportError::Write {
            path: artifact.path.clone(),
            message: error.to_string(),
        }
    })?;
    match &artifact.kind {
        ArtifactKind::Png { output_index } => {
            let spec = &specs[*output_index];
            if spec.maximum_bytes.is_some_and(|maximum| {
                u64::try_from(bytes.len()).is_ok_and(|actual| actual > maximum)
            }) {
                return Err(ExportError::Verification {
                    missing: vec![format!(
                        "{} at no more than {} bytes",
                        artifact.path,
                        spec.maximum_bytes.unwrap_or_default()
                    )],
                    unexpected: vec![format!("{} bytes", bytes.len())],
                });
            }
            let png_header = png::Decoder::new(Cursor::new(&bytes));
            let reader = png_header
                .read_info()
                .map_err(|error| ExportError::Verification {
                    missing: vec![format!("valid PNG at {}", artifact.path)],
                    unexpected: vec![error.to_string()],
                })?;
            let expected_color_type = if spec.flatten_color.is_some() {
                png::ColorType::Rgb
            } else {
                png::ColorType::Rgba
            };
            if reader.info().color_type != expected_color_type {
                return Err(ExportError::Verification {
                    missing: vec![format!("{expected_color_type:?} PNG at {}", artifact.path)],
                    unexpected: vec![format!("{:?} PNG", reader.info().color_type)],
                });
            }
            let decoded_image = decode_png_rgba(&bytes).map_err(|error| ExportError::Render {
                output_id: spec.id.clone(),
                error,
            })?;
            verify_dimensions(
                &artifact.path,
                spec,
                decoded_image.width(),
                decoded_image.height(),
            )
        }
        ArtifactKind::Svg { output_index } => {
            let spec = &specs[*output_index];
            usvg::Tree::from_data(&bytes, &usvg::Options::default()).map_err(|error| {
                ExportError::Verification {
                    missing: vec![format!("valid SVG at {}", artifact.path)],
                    unexpected: vec![error.to_string()],
                }
            })?;
            if spec.format != OutputFormat::Svg {
                return Err(ExportError::Verification {
                    missing: vec![format!("SVG declaration for {}", artifact.path)],
                    unexpected: vec![format!("{:?}", spec.format).to_ascii_lowercase()],
                });
            }
            Ok(())
        }
        ArtifactKind::Ico { entries } => verify_ico(&bytes, &artifact.path, entries, specs),
        ArtifactKind::Icns { entries } => verify_icns(&bytes, &artifact.path, entries, specs),
        ArtifactKind::Metadata {
            package_id,
            format,
            contents,
            references,
        } => verify_metadata(
            package,
            &artifact.path,
            package_id,
            *format,
            contents,
            references,
            &bytes,
        ),
    }
}

fn verify_dimensions(
    path: &str,
    spec: &OutputSpec,
    actual_width: u32,
    actual_height: u32,
) -> Result<(), ExportError> {
    if (actual_width, actual_height) == (spec.width, spec.height) {
        Ok(())
    } else {
        Err(ExportError::Verification {
            missing: vec![format!("{path} at {}x{}", spec.width, spec.height)],
            unexpected: vec![format!("{path} at {actual_width}x{actual_height}")],
        })
    }
}

fn verify_ico(
    bytes: &[u8],
    path: &str,
    expected_entries: &[IcoEntryPlan],
    specs: &[OutputSpec],
) -> Result<(), ExportError> {
    let directory =
        ico::IconDir::read(Cursor::new(bytes)).map_err(|error| ExportError::Verification {
            missing: vec![format!("valid ICO at {path}")],
            unexpected: vec![error.to_string()],
        })?;
    if directory.entries().len() != expected_entries.len() {
        return Err(ExportError::Verification {
            missing: vec![format!("{} ICO entries at {path}", expected_entries.len())],
            unexpected: vec![format!("{} ICO entries", directory.entries().len())],
        });
    }
    for (actual, expected) in directory.entries().iter().zip(expected_entries) {
        let spec = &specs[expected.output_index];
        verify_dimensions(path, spec, actual.width(), actual.height())?;
        let expected_png = expected.encoding == ContainerEncoding::Png;
        if actual.is_png() != expected_png {
            return Err(ExportError::Verification {
                missing: vec![format!(
                    "{}-encoded ICO entry {}",
                    if expected_png { "PNG" } else { "BMP" },
                    spec.id
                )],
                unexpected: vec![format!(
                    "{}-encoded ICO entry {}",
                    if actual.is_png() { "PNG" } else { "BMP" },
                    spec.id
                )],
            });
        }
        let decoded = actual.decode().map_err(|error| ExportError::Verification {
            missing: vec![format!("decodable ICO entry {}", spec.id)],
            unexpected: vec![error.to_string()],
        })?;
        verify_dimensions(path, spec, decoded.width(), decoded.height())?;
    }
    Ok(())
}

fn verify_icns(
    bytes: &[u8],
    path: &str,
    expected_entries: &[IcoEntryPlan],
    specs: &[OutputSpec],
) -> Result<(), ExportError> {
    let family =
        icns::IconFamily::read(Cursor::new(bytes)).map_err(|error| ExportError::Verification {
            missing: vec![format!("valid ICNS at {path}")],
            unexpected: vec![error.to_string()],
        })?;
    let available = family.available_icons();
    if available.len() != expected_entries.len() {
        return Err(ExportError::Verification {
            missing: vec![format!("{} ICNS entries at {path}", expected_entries.len())],
            unexpected: vec![format!("{} ICNS entries", available.len())],
        });
    }
    for expected in expected_entries {
        let spec = &specs[expected.output_index];
        let icon_type =
            icns::IconType::from_pixel_size(spec.width, spec.height).ok_or_else(|| {
                ExportError::Verification {
                    missing: vec![format!("supported ICNS type for {}", spec.id)],
                    unexpected: Vec::new(),
                }
            })?;
        if !available.contains(&icon_type) {
            return Err(ExportError::Verification {
                missing: vec![format!("ICNS entry {}x{}", spec.width, spec.height)],
                unexpected: Vec::new(),
            });
        }
        let decoded = family
            .get_icon_with_type(icon_type)
            .map_err(|error| ExportError::Verification {
                missing: vec![format!("decodable ICNS entry {}", spec.id)],
                unexpected: vec![error.to_string()],
            })?
            .convert_to(icns::PixelFormat::RGBA);
        verify_dimensions(path, spec, decoded.width(), decoded.height())?;
        if decoded.data().len()
            != usize::try_from(u64::from(spec.width) * u64::from(spec.height) * 4)
                .expect("bounded ICNS buffer length fits usize")
        {
            return Err(ExportError::Verification {
                missing: vec![format!("RGBA ICNS entry {}", spec.id)],
                unexpected: vec!["decoded ICNS buffer length differs".to_owned()],
            });
        }
    }
    Ok(())
}

fn verify_metadata(
    package: &Path,
    path: &str,
    package_id: &str,
    format: MetadataFormat,
    contents: &str,
    references: &[RenderedMetadataReference],
    bytes: &[u8],
) -> Result<(), ExportError> {
    if bytes != contents.as_bytes() {
        return Err(ExportError::Verification {
            missing: vec![format!("planned metadata bytes at {path}")],
            unexpected: vec!["staged metadata bytes differ".to_owned()],
        });
    }
    match format {
        MetadataFormat::Json => {
            let value = serde_json::from_slice::<serde_json::Value>(bytes).map_err(|error| {
                ExportError::Metadata {
                    package_id: package_id.to_owned(),
                    message: error.to_string(),
                }
            })?;
            if path.ends_with("/Contents.json") {
                verify_contents_json(package, path, references, &value)?;
            }
        }
        MetadataFormat::Xml => {
            let mut reader = quick_xml::Reader::from_reader(bytes);
            loop {
                match reader.read_event() {
                    Ok(quick_xml::events::Event::Eof) => break,
                    Ok(_) => {}
                    Err(error) => {
                        return Err(ExportError::Metadata {
                            package_id: package_id.to_owned(),
                            message: error.to_string(),
                        });
                    }
                }
            }
        }
        MetadataFormat::Html | MetadataFormat::Text => {}
    }
    for reference in references {
        if !package.join(relative_to_path(&reference.path)).is_file() {
            return Err(ExportError::Verification {
                missing: vec![reference.path.clone()],
                unexpected: Vec::new(),
            });
        }
        if !contents.contains(&reference.value) {
            return Err(ExportError::Verification {
                missing: vec![format!("reference `{}` in {path}", reference.value)],
                unexpected: Vec::new(),
            });
        }
    }
    Ok(())
}

fn verify_contents_json(
    package: &Path,
    path: &str,
    references: &[RenderedMetadataReference],
    value: &serde_json::Value,
) -> Result<(), ExportError> {
    let filenames: BTreeSet<_> = value
        .get("images")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|image| image.get("filename").and_then(serde_json::Value::as_str))
        .map(str::to_owned)
        .collect();
    let expected: BTreeSet<_> = references
        .iter()
        .map(|reference| relative_reference(path, &reference.path))
        .collect();
    let missing: Vec<_> = expected.difference(&filenames).cloned().collect();
    let unexpected: Vec<_> = filenames.difference(&expected).cloned().collect();
    if !missing.is_empty() || !unexpected.is_empty() {
        return Err(ExportError::Verification {
            missing: missing
                .into_iter()
                .map(|filename| format!("Contents.json image `{filename}`"))
                .collect(),
            unexpected,
        });
    }
    let parent = path.rsplit_once('/').map_or("", |(parent, _)| parent);
    for filename in filenames {
        let staged_path = if parent.is_empty() {
            filename
        } else {
            format!("{parent}/{filename}")
        };
        let staged_path = crate::sanitize_relative_path(&staged_path).map_err(|error| {
            ExportError::Verification {
                missing: Vec::new(),
                unexpected: vec![error.to_string()],
            }
        })?;
        if !package.join(relative_to_path(&staged_path)).is_file() {
            return Err(ExportError::Verification {
                missing: vec![staged_path],
                unexpected: Vec::new(),
            });
        }
    }
    Ok(())
}

fn write_combined_zip(package: &Path, zip_path: &str) -> Result<(), ExportError> {
    let mut paths = BTreeSet::new();
    collect_staged_files(package, package, &mut paths)?;
    let destination = package.join(relative_to_path(zip_path));
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&destination)
        .map_err(|error| ExportError::Zip {
            path: zip_path.to_owned(),
            message: error.to_string(),
        })?;
    let mut archive = ZipWriter::new(file);
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o644);
    for path in paths {
        let bytes =
            fs::read(package.join(relative_to_path(&path))).map_err(|error| ExportError::Zip {
                path: path.clone(),
                message: error.to_string(),
            })?;
        archive
            .start_file(&path, options)
            .and_then(|()| archive.write_all(&bytes).map_err(Into::into))
            .map_err(|error| ExportError::Zip {
                path: path.clone(),
                message: error.to_string(),
            })?;
    }
    let file = archive.finish().map_err(|error| ExportError::Zip {
        path: zip_path.to_owned(),
        message: error.to_string(),
    })?;
    file.sync_all().map_err(|error| ExportError::Zip {
        path: zip_path.to_owned(),
        message: error.to_string(),
    })
}

fn verify_combined_zip(
    package: &Path,
    zip_path: &str,
    expected_package_paths: &BTreeSet<String>,
) -> Result<(), ExportError> {
    let file =
        File::open(package.join(relative_to_path(zip_path))).map_err(|error| ExportError::Zip {
            path: zip_path.to_owned(),
            message: error.to_string(),
        })?;
    let mut archive = ZipArchive::new(file).map_err(|error| ExportError::Zip {
        path: zip_path.to_owned(),
        message: error.to_string(),
    })?;
    let expected: BTreeSet<_> = expected_package_paths
        .iter()
        .filter(|path| path.as_str() != zip_path)
        .cloned()
        .collect();
    let mut actual = BTreeMap::new();
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).map_err(|error| ExportError::Zip {
            path: zip_path.to_owned(),
            message: error.to_string(),
        })?;
        if !entry.is_file() || entry.enclosed_name().is_none() {
            return Err(ExportError::Verification {
                missing: Vec::new(),
                unexpected: vec![entry.name().to_owned()],
            });
        }
        let name = entry.name().to_owned();
        let mut bytes = Vec::new();
        entry
            .read_to_end(&mut bytes)
            .map_err(|error| ExportError::Zip {
                path: name.clone(),
                message: error.to_string(),
            })?;
        if actual.insert(name.clone(), bytes).is_some() {
            return Err(ExportError::Verification {
                missing: Vec::new(),
                unexpected: vec![format!("duplicate ZIP entry: {name}")],
            });
        }
    }
    let actual_paths: BTreeSet<_> = actual.keys().cloned().collect();
    let missing: Vec<_> = expected.difference(&actual_paths).cloned().collect();
    let unexpected: Vec<_> = actual_paths.difference(&expected).cloned().collect();
    if !missing.is_empty() || !unexpected.is_empty() {
        return Err(ExportError::Verification {
            missing,
            unexpected,
        });
    }
    for (path, archived) in actual {
        let staged =
            fs::read(package.join(relative_to_path(&path))).map_err(|error| ExportError::Zip {
                path: path.clone(),
                message: error.to_string(),
            })?;
        if archived != staged {
            return Err(ExportError::Verification {
                missing: vec![format!("byte-identical ZIP entry {path}")],
                unexpected: vec!["archive contents differ from staged file".to_owned()],
            });
        }
    }
    Ok(())
}

fn collect_staged_files(
    root: &Path,
    directory: &Path,
    files: &mut BTreeSet<String>,
) -> Result<(), ExportError> {
    for entry in fs::read_dir(directory).map_err(|error| ExportError::Write {
        path: display_path(directory),
        message: error.to_string(),
    })? {
        let entry = entry.map_err(|error| ExportError::Write {
            path: display_path(directory),
            message: error.to_string(),
        })?;
        let metadata = entry.metadata().map_err(|error| ExportError::Write {
            path: display_path(&entry.path()),
            message: error.to_string(),
        })?;
        if metadata.is_dir() {
            collect_staged_files(root, &entry.path(), files)?;
        } else if metadata.is_file() && !entry.file_type().is_ok_and(|kind| kind.is_symlink()) {
            let entry_path = entry.path();
            let relative = entry_path
                .strip_prefix(root)
                .map_err(|error| ExportError::Write {
                    path: display_path(&entry_path),
                    message: error.to_string(),
                })?;
            files.insert(relative.to_string_lossy().replace('\\', "/"));
        } else {
            return Err(ExportError::Verification {
                missing: Vec::new(),
                unexpected: vec![display_path(&entry.path())],
            });
        }
    }
    Ok(())
}

fn write_new_file(path: &Path, bytes: &[u8]) -> Result<(), ExportError> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| ExportError::Write {
            path: display_path(path),
            message: error.to_string(),
        })?;
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(|error| ExportError::Write {
            path: display_path(path),
            message: error.to_string(),
        })
}

fn commit_package(
    staging: tempfile::TempDir,
    package: &Path,
    destination: &Path,
    destination_exists: bool,
) -> Result<(), ExportError> {
    commit_package_with(
        staging,
        package,
        destination,
        destination_exists,
        |from, to| fs::rename(from, to),
    )
}

fn commit_package_with<F>(
    staging: tempfile::TempDir,
    package: &Path,
    destination: &Path,
    destination_exists: bool,
    rename: F,
) -> Result<(), ExportError>
where
    F: Fn(&Path, &Path) -> std::io::Result<()>,
{
    if destination_exists {
        let staging_root = package.parent().ok_or_else(|| ExportError::Commit {
            path: display_path(destination),
            message: "staging package has no parent".to_owned(),
        })?;
        let previous = staging_root.join("previous");
        rename(destination, &previous).map_err(|error| ExportError::Commit {
            path: display_path(destination),
            message: error.to_string(),
        })?;
        if let Err(error) = rename(package, destination) {
            if let Err(restore_error) = rename(&previous, destination) {
                // Dropping the staging directory here would delete the user's
                // only copy of the previous package; keep it and say where.
                let preserved = staging.keep();
                return Err(ExportError::CommitRestoreFailed {
                    path: display_path(destination),
                    backup_path: display_path(&preserved.join("previous")),
                    message: format!(
                        "{error}; restoring the previous package also failed: {restore_error}"
                    ),
                });
            }
            return Err(ExportError::Commit {
                path: display_path(destination),
                message: error.to_string(),
            });
        }
    } else {
        rename(package, destination).map_err(|error| ExportError::Commit {
            path: display_path(destination),
            message: error.to_string(),
        })?;
    }
    sync_directory(destination.parent().unwrap_or_else(|| Path::new(".")));
    Ok(())
}

fn sync_directory(path: &Path) {
    if let Ok(directory) = File::open(path) {
        let _ = directory.sync_all();
    }
}

fn check_cancelled(cancellation: &ExportCancellation) -> Result<(), ExportError> {
    if cancellation.is_cancelled() {
        Err(ExportError::Cancelled)
    } else {
        Ok(())
    }
}

fn relative_to_path(path: &str) -> PathBuf {
    path.split('/').collect()
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

#[cfg(test)]
mod tests {
    use std::{
        collections::BTreeMap,
        fs,
        io::Cursor,
        path::{Path, PathBuf},
    };

    use image::{ImageFormat, Rgba, RgbaImage};
    use tempfile::tempdir;

    use super::*;
    use crate::{CustomOutput, OutputFormat, ProjectSource, SelectedPreset, import_source};

    fn export_project_fixture() -> (tempfile::TempDir, ProjectState) {
        let directory = tempdir().expect("temporary directory");
        let source_path = directory.path().join("master.png");
        RgbaImage::from_pixel(64, 64, Rgba([10, 20, 30, 128]))
            .save_with_format(&source_path, ImageFormat::Png)
            .expect("write source");
        let source = import_source(&source_path, "master").expect("import source");
        let mut project = ProjectState::new("Export fixture");
        project.selected_presets.clear();
        project.enabled_outputs.clear();
        project.sources.push(ProjectSource {
            role: "master".to_owned(),
            path: source.path,
            content_hash: source.content_hash,
        });
        for (id, size) in [("small", 16), ("large", 32)] {
            project.custom_outputs.push(CustomOutput {
                id: id.to_owned(),
                width: size,
                height: size,
                format: OutputFormat::Png,
                filename: format!("{id}.png"),
                destination: "custom".to_owned(),
                source_role: "master".to_owned(),
                usage: format!("{size}px test output"),
            });
            project.enabled_outputs.insert(id.to_owned());
        }
        (directory, project)
    }

    fn preset_project_fixture() -> ProjectState {
        let source_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("fixtures/sources/asymmetric.svg");
        let source = import_source(&source_path, "master").expect("import preset source");
        let mut project = ProjectState::new("Preset export");
        project.sources.push(ProjectSource {
            role: "master".to_owned(),
            path: source.path,
            content_hash: source.content_hash,
        });
        project
    }

    fn select_only_preset(project: &mut ProjectState, id: &str) {
        let preset = imaguncula_presets::load_bundled_preset(id, 1)
            .unwrap_or_else(|error| panic!("load preset {id}: {error}"));
        project.selected_presets = vec![SelectedPreset {
            id: preset.id,
            version: preset.version,
        }];
        project.enabled_outputs = preset.outputs.into_iter().map(|output| output.id).collect();
        project.export_settings.create_combined_zip = false;
    }

    fn select_default_preset(project: &mut ProjectState, id: &str) {
        let summary = imaguncula_presets::bundled_preset_summaries()
            .expect("load preset summaries")
            .into_iter()
            .find(|preset| preset.id == id)
            .unwrap_or_else(|| panic!("missing preset {id}"));
        project.selected_presets = vec![SelectedPreset {
            id: summary.id,
            version: summary.version,
        }];
        project.enabled_outputs = summary.default_output_ids.into_iter().collect();
        project.export_settings.create_combined_zip = false;
    }

    fn add_role_from_master(project: &mut ProjectState, role: &str) {
        let source = project
            .sources
            .iter()
            .find(|source| source.role == "master")
            .expect("master source")
            .clone();
        project.sources.push(ProjectSource {
            role: role.to_owned(),
            path: source.path,
            content_hash: source.content_hash,
        });
    }

    fn tree_snapshot(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
        fn visit(root: &Path, current: &Path, snapshot: &mut BTreeMap<PathBuf, Vec<u8>>) {
            let mut entries = fs::read_dir(current)
                .expect("read deterministic tree")
                .collect::<Result<Vec<_>, _>>()
                .expect("enumerate deterministic tree");
            entries.sort_by_key(std::fs::DirEntry::path);
            for entry in entries {
                let path = entry.path();
                if path.is_dir() {
                    visit(root, &path, snapshot);
                } else {
                    snapshot.insert(
                        path.strip_prefix(root)
                            .expect("path is below root")
                            .to_owned(),
                        fs::read(&path).expect("read deterministic file"),
                    );
                }
            }
        }

        let mut snapshot = BTreeMap::new();
        visit(root, root, &mut snapshot);
        snapshot
    }

    #[test]
    fn multi_row_export_matches_names_dimensions_and_report() {
        let (directory, project) = export_project_fixture();
        let destination = directory.path().join("result");
        let mut progress = Vec::new();

        let report = export_project(
            &project,
            &destination,
            false,
            &ExportCancellation::default(),
            |event| progress.push(event),
        )
        .expect("export succeeds");

        assert_eq!(progress.first().expect("initial progress").completed, 0);
        assert_eq!(progress.last().expect("final progress").completed, 2);
        assert_eq!(report.report.files.len(), 2);
        for (name, size) in [("small.png", 16), ("large.png", 32)] {
            let bytes = fs::read(destination.join("custom").join(name)).expect("read output");
            let image = decode_png_rgba(&bytes).expect("decode output");
            assert_eq!((image.width(), image.height()), (size, size));
        }
        assert!(destination.join("report.md").is_file());
    }

    #[test]
    fn cancellation_removes_staging_and_never_exposes_destination() {
        let (directory, project) = export_project_fixture();
        let destination = directory.path().join("cancelled");
        let cancellation = ExportCancellation::default();
        let callback_cancellation = cancellation.clone();

        let error = export_project(&project, &destination, false, &cancellation, move |event| {
            if event.completed == 1 {
                callback_cancellation.cancel();
            }
        })
        .expect_err("export is cancelled");

        assert!(matches!(error, ExportError::Cancelled));
        assert!(!destination.exists());
        assert!(
            fs::read_dir(directory.path())
                .expect("read fixture directory")
                .all(|entry| !entry
                    .expect("directory entry")
                    .file_name()
                    .to_string_lossy()
                    .starts_with(".imaguncula-stage-"))
        );
    }

    #[test]
    fn traversal_destination_fails_before_staging_or_external_write() {
        let (directory, mut project) = export_project_fixture();
        project.custom_outputs[0].destination = "../outside".to_owned();
        let destination = directory.path().join("result");
        let error = export_project(
            &project,
            &destination,
            false,
            &ExportCancellation::default(),
            |_| {},
        )
        .expect_err("traversal destination is rejected");
        assert!(matches!(error, ExportError::Resolve { .. }));
        assert!(!destination.exists());
        assert!(!directory.path().join("outside").exists());
        assert!(
            fs::read_dir(directory.path())
                .expect("read fixture parent")
                .filter_map(Result::ok)
                .all(|entry| !entry.file_name().to_string_lossy().contains("stage"))
        );
    }

    #[test]
    fn overwrite_requires_confirmation_and_replaces_only_after_verification() {
        let (directory, project) = export_project_fixture();
        let destination = directory.path().join("existing");
        fs::create_dir(&destination).expect("create existing package");
        fs::write(destination.join("sentinel.txt"), b"old").expect("write sentinel");

        assert!(matches!(
            export_project(
                &project,
                &destination,
                false,
                &ExportCancellation::default(),
                |_| {}
            ),
            Err(ExportError::DestinationExists { .. })
        ));
        assert!(destination.join("sentinel.txt").is_file());

        assert!(matches!(
            export_project(
                &project,
                &destination,
                true,
                &ExportCancellation::default(),
                |_| {}
            ),
            Err(ExportError::DestinationNotPackage { .. })
        ));
        assert!(destination.join("sentinel.txt").is_file());

        fs::write(destination.join("report.md"), b"# previous package").expect("write marker");
        export_project(
            &project,
            &destination,
            true,
            &ExportCancellation::default(),
            |_| {},
        )
        .expect("confirmed replacement succeeds");
        assert!(!destination.join("sentinel.txt").exists());
        assert!(destination.join("custom/small.png").is_file());
    }

    #[test]
    fn inspection_distinguishes_previous_packages_from_unrelated_directories() {
        let directory = tempdir().expect("temporary directory");
        let unrelated = directory.path().join("unrelated");
        fs::create_dir(&unrelated).expect("create unrelated directory");
        fs::write(unrelated.join("photo.jpg"), b"data").expect("write unrelated file");
        assert_eq!(
            inspect_export_destination(&unrelated)
                .expect("inspect unrelated")
                .state,
            DestinationState::NonEmptyDirectory
        );

        let package = directory.path().join("package");
        fs::create_dir(&package).expect("create package directory");
        fs::write(package.join("report.md"), b"# report").expect("write marker");
        assert_eq!(
            inspect_export_destination(&package)
                .expect("inspect package")
                .state,
            DestinationState::NonEmptyPackageDirectory
        );
    }

    #[test]
    fn failed_restore_preserves_the_previous_package_and_names_its_location() {
        let parent = tempdir().expect("temporary directory");
        let destination = parent.path().join("existing");
        fs::create_dir(&destination).expect("create destination");
        fs::write(destination.join("report.md"), b"previous data").expect("write previous data");
        let (staging, package) = create_staging_package(parent.path()).expect("create staging");
        fs::write(package.join("new.txt"), b"new").expect("stage new file");
        let staging_path = staging.path().to_path_buf();

        let error = commit_package_with(staging, &package, &destination, true, |from, to| {
            if from == destination.as_path() {
                fs::rename(from, to)
            } else {
                Err(std::io::Error::other("simulated rename failure"))
            }
        })
        .expect_err("commit and restore failures surface");

        match error {
            ExportError::CommitRestoreFailed { backup_path, .. } => {
                assert_eq!(
                    std::path::Path::new(&backup_path),
                    staging_path.join("previous")
                );
            }
            other => panic!("unexpected error variant: {other:?}"),
        }
        assert!(staging_path.is_dir(), "staging must survive the failure");
        assert_eq!(
            fs::read(staging_path.join("previous").join("report.md"))
                .expect("backup file remains readable"),
            b"previous data"
        );
    }

    #[test]
    fn source_change_fails_before_touching_existing_destination() {
        let (directory, project) = export_project_fixture();
        let destination = directory.path().join("existing");
        fs::create_dir(&destination).expect("create existing package");
        fs::write(destination.join("sentinel.txt"), b"old").expect("write sentinel");
        fs::write(directory.path().join("master.png"), b"changed").expect("replace source");

        assert!(matches!(
            export_project(
                &project,
                &destination,
                true,
                &ExportCancellation::default(),
                |_| {}
            ),
            Err(ExportError::OutputsNotReady { .. })
        ));
        assert_eq!(
            fs::read(destination.join("sentinel.txt")).expect("read sentinel"),
            b"old"
        );
    }

    #[test]
    fn default_presets_export_verified_ico_web_metadata_and_zip() {
        let directory = tempdir().expect("temporary directory");
        let project = preset_project_fixture();
        let destination = directory.path().join("preset-package");
        let mut progress = Vec::new();

        let report = export_project(
            &project,
            &destination,
            false,
            &ExportCancellation::default(),
            |event| progress.push(event),
        )
        .expect("preset export succeeds");

        assert_eq!(progress.first().expect("initial progress").total, 23);
        assert_eq!(progress.last().expect("final progress").completed, 23);
        assert_eq!(report.report.files.len(), 9);

        let windows_bytes = fs::read(destination.join("windows-desktop/preset-export.ico"))
            .expect("read Windows ICO");
        let windows = ico::IconDir::read(Cursor::new(windows_bytes)).expect("decode Windows ICO");
        let expected_sizes = [16, 20, 24, 30, 32, 36, 40, 42, 48, 60, 64, 72, 80, 96, 256];
        assert_eq!(windows.entries().len(), expected_sizes.len());
        for (index, (entry, size)) in windows.entries().iter().zip(expected_sizes).enumerate() {
            assert_eq!((entry.width(), entry.height()), (size, size));
            assert_eq!(entry.is_png(), index == expected_sizes.len() - 1);
            let decoded = entry.decode().expect("decode Windows ICO entry");
            assert!(
                decoded
                    .rgba_data()
                    .chunks_exact(4)
                    .any(|pixel| pixel[3] < 255)
            );
        }

        let favicon_bytes =
            fs::read(destination.join("web-pwa/favicon.ico")).expect("read favicon ICO");
        let favicon = ico::IconDir::read(Cursor::new(favicon_bytes)).expect("decode favicon ICO");
        assert_eq!(
            favicon
                .entries()
                .iter()
                .map(ico::IconDirEntry::width)
                .collect::<Vec<_>>(),
            [16, 32, 48]
        );
        assert!(favicon.entries().iter().all(|entry| !entry.is_png()));

        for (path, size) in [
            ("web-pwa/apple-touch-icon.png", 180),
            ("web-pwa/icon-192.png", 192),
            ("web-pwa/icon-512.png", 512),
            ("web-pwa/icon-maskable-512.png", 512),
        ] {
            let decoded = decode_png_rgba(&fs::read(destination.join(path)).expect("read web PNG"))
                .expect("decode web PNG");
            assert_eq!((decoded.width(), decoded.height()), (size, size));
        }

        let manifest: serde_json::Value = serde_json::from_slice(
            &fs::read(destination.join("web-pwa/site.webmanifest")).expect("read manifest"),
        )
        .expect("parse manifest");
        assert_eq!(
            manifest["icons"].as_array().expect("manifest icons").len(),
            3
        );
        let html =
            fs::read_to_string(destination.join("web-pwa/icons.html")).expect("read HTML links");
        for reference in ["favicon.svg", "favicon.ico", "apple-touch-icon.png"] {
            assert!(html.contains(reference));
        }
        let sanitized_svg =
            fs::read(destination.join("web-pwa/favicon.svg")).expect("read sanitized SVG");
        usvg::Tree::from_data(&sanitized_svg, &usvg::Options::default())
            .expect("parse sanitized SVG");

        let zip_bytes =
            fs::read(destination.join("preset-export-icons.zip")).expect("read combined ZIP");
        let archive = ZipArchive::new(Cursor::new(zip_bytes)).expect("open combined ZIP");
        assert_eq!(archive.len(), 10);
    }

    #[test]
    fn metadata_reference_to_disabled_output_fails_before_staging() {
        let directory = tempdir().expect("temporary directory");
        let mut project = preset_project_fixture();
        project.enabled_outputs.remove("pwa-maskable-512");
        let destination = directory.path().join("invalid-metadata");

        let error = export_project(
            &project,
            &destination,
            false,
            &ExportCancellation::default(),
            |_| {},
        )
        .expect_err("disabled metadata target fails");

        assert!(matches!(
            error,
            ExportError::MetadataReferenceMissing {
                ref output_id,
                ..
            } if output_id == "pwa-maskable-512"
        ));
        assert!(!destination.exists());
    }

    #[test]
    fn combined_zip_is_byte_deterministic() {
        let directory = tempdir().expect("temporary directory");
        let project = preset_project_fixture();
        let first = directory.path().join("first");
        let second = directory.path().join("second");
        for destination in [&first, &second] {
            export_project(
                &project,
                destination,
                false,
                &ExportCancellation::default(),
                |_| {},
            )
            .expect("deterministic export succeeds");
        }
        assert_eq!(
            fs::read(first.join("preset-export-icons.zip")).expect("read first ZIP"),
            fs::read(second.join("preset-export-icons.zip")).expect("read second ZIP")
        );
        assert_eq!(tree_snapshot(&first), tree_snapshot(&second));
    }

    #[test]
    fn linux_hicolor_exports_fixed_and_scalable_directory_tree() {
        let directory = tempdir().expect("temporary directory");
        let mut project = preset_project_fixture();
        let preset =
            imaguncula_presets::load_bundled_preset("linux-hicolor", 1).expect("load Linux preset");
        project.selected_presets = vec![SelectedPreset {
            id: preset.id,
            version: preset.version,
        }];
        project.enabled_outputs = preset.outputs.into_iter().map(|output| output.id).collect();
        project.export_settings.create_combined_zip = false;
        let destination = directory.path().join("linux-package");

        let report = export_project(
            &project,
            &destination,
            false,
            &ExportCancellation::default(),
            |_| {},
        )
        .expect("Linux export succeeds");

        assert_eq!(report.report.files.len(), 9);
        for size in [16, 24, 32, 48, 64, 128, 256, 512] {
            let path = format!("linux-hicolor/hicolor/{size}x{size}/apps/preset-export.png");
            let image = decode_png_rgba(&fs::read(destination.join(path)).expect("read Linux PNG"))
                .expect("decode Linux PNG");
            assert_eq!((image.width(), image.height()), (size, size));
        }
        let scalable =
            fs::read(destination.join("linux-hicolor/hicolor/scalable/apps/preset-export.svg"))
                .expect("read Linux SVG");
        usvg::Tree::from_data(&scalable, &usvg::Options::default())
            .expect("parse Linux scalable SVG");
    }

    #[test]
    fn apple_appicon_catalog_modes_export_exact_contents_and_dimensions() {
        let directory = tempdir().expect("temporary directory");
        for (preset_id, catalog_path, expected_count) in [
            ("apple-ios-modern", "apple-ios-modern/AppIcon.appiconset", 1),
            (
                "apple-ios-compatibility",
                "apple-ios-compatibility/AppIcon.appiconset",
                18,
            ),
            (
                "apple-macos-appicon",
                "apple-macos-appicon/AppIcon.appiconset",
                10,
            ),
        ] {
            let mut project = preset_project_fixture();
            select_only_preset(&mut project, preset_id);
            let destination = directory.path().join(preset_id);
            export_project(
                &project,
                &destination,
                false,
                &ExportCancellation::default(),
                |_| {},
            )
            .unwrap_or_else(|error| panic!("export {preset_id}: {error}"));

            let catalog = destination.join(catalog_path);
            let contents: serde_json::Value = serde_json::from_slice(
                &fs::read(catalog.join("Contents.json")).expect("read Contents.json"),
            )
            .expect("parse Contents.json");
            let images = contents["images"].as_array().expect("images array");
            assert_eq!(images.len(), expected_count, "{preset_id}");
            for entry in images {
                let filename = entry["filename"].as_str().expect("catalog filename");
                let points = entry["size"]
                    .as_str()
                    .expect("point size")
                    .split('x')
                    .next()
                    .expect("width component");
                let half_points = points.split_once('.').map_or_else(
                    || points.parse::<u32>().expect("whole point size") * 2,
                    |(whole, fraction)| {
                        assert_eq!(fraction, "5", "only half-point catalog sizes are supported");
                        whole.parse::<u32>().expect("half-point whole component") * 2 + 1
                    },
                );
                let scale = entry
                    .get("scale")
                    .and_then(serde_json::Value::as_str)
                    .map_or(1, |scale| {
                        scale
                            .trim_end_matches('x')
                            .parse::<u32>()
                            .expect("numeric scale")
                    });
                let scaled_half_points = half_points * scale;
                assert_eq!(
                    scaled_half_points % 2,
                    0,
                    "catalog size resolves to whole pixels"
                );
                let pixels = scaled_half_points / 2;
                let decoded =
                    decode_png_rgba(&fs::read(catalog.join(filename)).expect("read catalog PNG"))
                        .expect("decode catalog PNG");
                assert_eq!((decoded.width(), decoded.height()), (pixels, pixels));
            }
        }
    }

    #[test]
    fn macos_icns_exports_every_decodable_rgba_entry() {
        let directory = tempdir().expect("temporary directory");
        let mut project = preset_project_fixture();
        select_only_preset(&mut project, "apple-macos-icns");
        let destination = directory.path().join("icns");
        export_project(
            &project,
            &destination,
            false,
            &ExportCancellation::default(),
            |_| {},
        )
        .expect("ICNS export succeeds");

        let bytes =
            fs::read(destination.join("apple-macos-icns/preset-export.icns")).expect("read ICNS");
        let family = icns::IconFamily::read(Cursor::new(bytes)).expect("decode ICNS");
        assert_eq!(family.available_icons().len(), 7);
        for size in [16, 32, 64, 128, 256, 512, 1024] {
            let icon_type =
                icns::IconType::from_pixel_size(size, size).expect("supported ICNS size");
            let decoded = family
                .get_icon_with_type(icon_type)
                .expect("decode ICNS entry")
                .convert_to(icns::PixelFormat::RGBA);
            assert_eq!((decoded.width(), decoded.height()), (size, size));
            assert!(decoded.data().chunks_exact(4).any(|pixel| pixel[3] < 255));
        }
    }

    #[test]
    fn icon_composer_path_requires_roles_and_exports_layers_not_a_fake_icon() {
        let directory = tempdir().expect("temporary directory");
        let mut incomplete = preset_project_fixture();
        select_only_preset(&mut incomplete, "apple-icon-composer-guidance");
        let incomplete_destination = directory.path().join("incomplete");
        assert!(matches!(
            export_project(
                &incomplete,
                &incomplete_destination,
                false,
                &ExportCancellation::default(),
                |_| {}
            ),
            Err(ExportError::OutputsNotReady { .. })
        ));
        assert!(!incomplete_destination.exists());

        let source = incomplete.sources[0].clone();
        let mut project = incomplete;
        project.sources.extend([
            ProjectSource {
                role: "foreground".to_owned(),
                path: source.path.clone(),
                content_hash: source.content_hash.clone(),
            },
            ProjectSource {
                role: "background".to_owned(),
                path: source.path,
                content_hash: source.content_hash,
            },
        ]);
        let destination = directory.path().join("complete");
        export_project(
            &project,
            &destination,
            false,
            &ExportCancellation::default(),
            |_| {},
        )
        .expect("Icon Composer source export succeeds");

        let root = destination.join("apple-icon-composer-guidance/IconComposerSources");
        for filename in ["foreground.png", "background.png"] {
            let image = decode_png_rgba(&fs::read(root.join(filename)).expect("read layer"))
                .expect("decode layer");
            assert_eq!((image.width(), image.height()), (1024, 1024));
        }
        let instructions = fs::read_to_string(root.join("README.txt")).expect("read instructions");
        assert!(instructions.contains("background.png"));
        assert!(instructions.contains("foreground.png"));
        assert!(
            !fs::read_dir(root)
                .expect("read source package")
                .filter_map(Result::ok)
                .any(|entry| entry
                    .path()
                    .extension()
                    .is_some_and(|extension| extension == "icon"))
        );
    }

    #[test]
    fn android_default_export_builds_legacy_adaptive_and_opaque_store_assets() {
        let directory = tempdir().expect("temporary directory");
        let mut project = preset_project_fixture();
        select_default_preset(&mut project, "android-launcher");
        add_role_from_master(&mut project, "foreground");
        add_role_from_master(&mut project, "background");
        assert!(!project
            .enabled_outputs
            .iter()
            .any(|id| id.starts_with("android-round-") || id.starts_with("android-monochrome-")));
        let destination = directory.path().join("android");
        export_project(
            &project,
            &destination,
            false,
            &ExportCancellation::default(),
            |_| {},
        )
        .expect("Android export succeeds");

        let root = destination.join("android-launcher");
        for (density, legacy, adaptive) in [
            ("mdpi", 48, 108),
            ("hdpi", 72, 162),
            ("xhdpi", 96, 216),
            ("xxhdpi", 144, 324),
            ("xxxhdpi", 192, 432),
        ] {
            for (name, size) in [
                ("ic_launcher.png", legacy),
                ("ic_launcher_foreground.png", adaptive),
                ("ic_launcher_background.png", adaptive),
            ] {
                let image = decode_png_rgba(
                    &fs::read(root.join(format!("res/mipmap-{density}/{name}")))
                        .expect("read Android PNG"),
                )
                .expect("decode Android PNG");
                assert_eq!((image.width(), image.height()), (size, size));
            }
        }
        let adaptive = fs::read_to_string(root.join("res/mipmap-anydpi-v26/ic_launcher.xml"))
            .expect("read adaptive XML");
        assert!(adaptive.contains("@mipmap/ic_launcher_background"));
        assert!(adaptive.contains("@mipmap/ic_launcher_foreground"));
        assert!(!adaptive.contains("monochrome"));

        let store_bytes =
            fs::read(root.join("play-store/icon-512.png")).expect("read Play Store PNG");
        let reader = png::Decoder::new(Cursor::new(&store_bytes))
            .read_info()
            .expect("read Play Store PNG header");
        assert_eq!(reader.info().color_type, png::ColorType::Rgb);
        let store = decode_png_rgba(&store_bytes).expect("decode Play Store PNG");
        assert_eq!((store.width(), store.height()), (512, 512));
        assert!(store.pixels().chunks_exact(4).all(|pixel| pixel[3] == 255));
        assert!(store_bytes.len() <= 1_048_576);
    }

    #[test]
    fn android_adaptive_roles_fail_closed_but_flat_legacy_only_succeeds() {
        let directory = tempdir().expect("temporary directory");
        let mut incomplete = preset_project_fixture();
        select_default_preset(&mut incomplete, "android-launcher");
        let incomplete_destination = directory.path().join("incomplete");
        assert!(matches!(
            export_project(
                &incomplete,
                &incomplete_destination,
                false,
                &ExportCancellation::default(),
                |_| {}
            ),
            Err(ExportError::OutputsNotReady { .. })
        ));
        assert!(!incomplete_destination.exists());

        incomplete.enabled_outputs.retain(|output| {
            output.starts_with("android-legacy-") || output == "android-play-store-512"
        });
        let legacy_destination = directory.path().join("legacy");
        export_project(
            &incomplete,
            &legacy_destination,
            false,
            &ExportCancellation::default(),
            |_| {},
        )
        .expect("flat legacy-only export succeeds");
        let root = legacy_destination.join("android-launcher");
        assert!(root.join("res/mipmap-mdpi/ic_launcher.png").is_file());
        assert!(!root.join("res/mipmap-anydpi-v26/ic_launcher.xml").exists());
        assert!(
            !root
                .join("res/mipmap-mdpi/ic_launcher_foreground.png")
                .exists()
        );
    }

    #[test]
    fn android_optional_round_and_monochrome_groups_are_atomic() {
        let directory = tempdir().expect("temporary directory");
        let mut project = preset_project_fixture();
        select_default_preset(&mut project, "android-launcher");
        for role in ["foreground", "background", "monochrome"] {
            add_role_from_master(&mut project, role);
        }
        let summary = imaguncula_presets::bundled_preset_summaries()
            .expect("load summaries")
            .into_iter()
            .find(|preset| preset.id == "android-launcher")
            .expect("Android summary");
        for group in &summary.optional_output_groups {
            project.enabled_outputs.extend(
                group
                    .outputs
                    .as_ref()
                    .expect("Android groups contain outputs")
                    .iter()
                    .cloned(),
            );
        }
        let destination = directory.path().join("complete");
        export_project(
            &project,
            &destination,
            false,
            &ExportCancellation::default(),
            |_| {},
        )
        .expect("complete optional Android export succeeds");
        let root = destination.join("android-launcher/res");
        assert!(root.join("mipmap-mdpi/ic_launcher_round.png").is_file());
        assert!(
            root.join("mipmap-mdpi/ic_launcher_monochrome.png")
                .is_file()
        );
        let themed = fs::read_to_string(root.join("mipmap-anydpi-v33/ic_launcher.xml"))
            .expect("read themed XML");
        assert!(themed.contains("@mipmap/ic_launcher_monochrome"));

        let mut partial = project;
        partial
            .enabled_outputs
            .retain(|id| !id.starts_with("android-monochrome-"));
        partial
            .enabled_outputs
            .insert("android-monochrome-mdpi".to_owned());
        let error = export_project(
            &partial,
            directory.path().join("partial"),
            false,
            &ExportCancellation::default(),
            |_| {},
        )
        .expect_err("partial metadata group fails");
        assert!(matches!(
            error,
            ExportError::MetadataConditionIncomplete {
                ref package_id,
                ..
            } if package_id == "android-adaptive-themed"
        ));
    }

    #[test]
    fn windows_msix_default_export_matches_required_asset_families() {
        let directory = tempdir().expect("temporary directory");
        let mut project = preset_project_fixture();
        select_default_preset(&mut project, "windows-msix");
        assert_eq!(project.enabled_outputs.len(), 57);
        let destination = directory.path().join("msix");
        let report = export_project(
            &project,
            &destination,
            false,
            &ExportCancellation::default(),
            |_| {},
        )
        .expect("default MSIX export succeeds");
        assert_eq!(report.report.files.len(), 57);

        let root = destination.join("windows-msix/msix/Assets");
        for (filename, width, height) in [
            ("AppList.targetsize-16.png", 16, 16),
            ("AppList.targetsize-256_altform-unplated.png", 256, 256),
            ("AppList.targetsize-256_altform-lightunplated.png", 256, 256),
            ("AppList.scale-400.png", 176, 176),
            ("MedTile.scale-400.png", 600, 600),
            ("StoreLogo.scale-400.png", 200, 200),
        ] {
            let image = decode_png_rgba(&fs::read(root.join(filename)).expect("read MSIX PNG"))
                .expect("decode MSIX PNG");
            assert_eq!((image.width(), image.height()), (width, height));
        }
        assert!(!root.join("SmallTile.scale-100.png").exists());
        assert!(!root.join("AppList.scale-250.png").exists());
    }

    #[test]
    fn windows_msix_intermediate_scale_group_exports_atomically() {
        let directory = tempdir().expect("temporary directory");
        let mut project = preset_project_fixture();
        select_default_preset(&mut project, "windows-msix");
        let summary = imaguncula_presets::bundled_preset_summaries()
            .expect("load summaries")
            .into_iter()
            .find(|preset| preset.id == "windows-msix")
            .expect("MSIX summary");
        let group = summary
            .optional_output_groups
            .iter()
            .find(|group| group.id == "intermediate-scales")
            .expect("intermediate scale group");
        project.enabled_outputs.extend(
            group
                .outputs
                .as_ref()
                .expect("group contains outputs")
                .iter()
                .cloned(),
        );
        let destination = directory.path().join("msix-intermediate");
        let report = export_project(
            &project,
            &destination,
            false,
            &ExportCancellation::default(),
            |_| {},
        )
        .expect("intermediate MSIX export succeeds");
        assert_eq!(report.report.files.len(), 69);
        let root = destination.join("windows-msix/msix/Assets");
        for filename in [
            "AppList.scale-250.png",
            "MedTile.scale-300.png",
            "WideTile.scale-250.png",
            "LargeTile.scale-300.png",
            "StoreLogo.scale-300.png",
        ] {
            assert!(root.join(filename).is_file(), "missing {filename}");
        }
    }
}

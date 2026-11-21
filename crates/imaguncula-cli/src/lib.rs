// SPDX-License-Identifier: MIT OR Apache-2.0
//! Headless command handling over the shared Imaguncula processing core.

use std::{collections::BTreeSet, fs, path::PathBuf};

use clap::{Args, Parser, Subcommand};
use imaguncula_core::{
    ExportCancellation, ExportError, ExportReport, ProjectFileError, ProjectSource, ProjectState,
    SelectedPreset, SourceError, default_export_path, export_project, import_source, list_presets,
    load_project,
};
use thiserror::Error;

/// Imaguncula command-line interface.
#[derive(Clone, Debug, Parser)]
#[command(name = "imaguncula", version, about, arg_required_else_help = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

/// Headless export modes.
#[derive(Clone, Debug, Subcommand)]
pub enum Command {
    /// Export an existing .imaguncula.json project.
    Export(ExportArgs),
    /// Export one source through one bundled preset without saving a project.
    Quick(QuickArgs),
}

/// Arguments shared by saved-project and one-shot exports.
#[derive(Clone, Debug, Args)]
pub struct CommonExportArgs {
    /// Parent directory that will receive <base>-icons; created when missing.
    #[arg(short, long, value_name = "DIRECTORY")]
    pub output: PathBuf,
    /// Replace an existing final package directory.
    #[arg(long)]
    pub overwrite: bool,
    /// Do not include the deterministic combined ZIP.
    #[arg(long)]
    pub no_zip: bool,
    /// Print the complete export result as JSON.
    #[arg(long)]
    pub json: bool,
}

/// Export an existing project file.
#[derive(Clone, Debug, Args)]
pub struct ExportArgs {
    /// Project file ending in .imaguncula.json.
    #[arg(value_name = "PROJECT")]
    pub project: PathBuf,
    /// Read sources the project references outside its own directory.
    ///
    /// Project files are portable, so sources reached through absolute paths
    /// or .. segments are refused until this reviewed authorization is given.
    #[arg(long)]
    pub allow_external_sources: bool,
    #[command(flatten)]
    pub common: CommonExportArgs,
}

/// One-shot preset export.
#[derive(Clone, Debug, Args)]
pub struct QuickArgs {
    /// SVG or PNG master source.
    #[arg(value_name = "SOURCE")]
    pub source: PathBuf,
    /// Exact bundled preset identifier, such as windows-desktop or web-pwa.
    #[arg(short, long, value_name = "PRESET_ID")]
    pub preset: String,
    /// Project/display name; defaults to the source filename stem.
    #[arg(long, value_name = "NAME")]
    pub name: Option<String>,
    /// Assign an additional preset source role, for example --role foreground=layer.svg.
    #[arg(long = "role", value_name = "ROLE=PATH")]
    pub role_sources: Vec<RoleSourceArg>,
    #[command(flatten)]
    pub common: CommonExportArgs,
}

/// One generic preset source-role assignment accepted by `quick` mode.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoleSourceArg {
    pub role: String,
    pub path: PathBuf,
}

impl std::str::FromStr for RoleSourceArg {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let (role, path) = value
            .split_once('=')
            .ok_or_else(|| "role sources must use ROLE=PATH".to_owned())?;
        if role.is_empty() || path.is_empty() {
            return Err("role sources must contain a non-empty role and path".to_owned());
        }
        Ok(Self {
            role: role.to_owned(),
            path: PathBuf::from(path),
        })
    }
}

/// Stable CLI failure classes.
#[derive(Debug, Error)]
pub enum CliError {
    #[error(
        "the project references sources outside its own directory; review them and re-run with --allow-external-sources: {}",
        references.join(", ")
    )]
    ExternalSources { references: Vec<String> },
    #[error("project load failed: {0}")]
    Project(#[from] ProjectFileError),
    #[error("source import failed: {0}")]
    Source(#[from] SourceError),
    #[error("bundled preset `{id}` does not exist")]
    Preset { id: String },
    #[error("export failed: {0}")]
    Export(#[from] ExportError),
    #[error("failed to prepare output parent `{path}`: {message}")]
    OutputDirectory { path: String, message: String },
    #[error("failed to serialize the machine-readable report: {message}")]
    ReportSerialization { message: String },
}

impl CliError {
    /// Process exit code kept stable for scripts.
    #[must_use]
    pub const fn exit_code(&self) -> u8 {
        match self {
            Self::Project(_) | Self::Preset { .. } | Self::ExternalSources { .. } => 2,
            Self::Source(_) => 3,
            Self::Export(
                ExportError::Resolve { .. }
                | ExportError::NoIncludedOutputs
                | ExportError::OutputsNotReady { .. }
                | ExportError::MetadataReferenceMissing { .. }
                | ExportError::MetadataConditionIncomplete { .. },
            ) => 4,
            Self::Export(_) | Self::OutputDirectory { .. } | Self::ReportSerialization { .. } => 5,
        }
    }
}

/// Successful command output.
#[derive(Debug)]
pub struct CliSuccess {
    pub report: ExportReport,
    pub json: bool,
}

impl CliSuccess {
    /// Renders the stable stdout representation.
    ///
    /// # Errors
    ///
    /// Returns an error only if JSON serialization unexpectedly fails.
    pub fn stdout(&self) -> Result<String, CliError> {
        if self.json {
            serde_json::to_string_pretty(&self.report).map_err(|error| {
                CliError::ReportSerialization {
                    message: error.to_string(),
                }
            })
        } else {
            Ok(format!(
                "Created {} generated files in {}",
                self.report.report.files.len(),
                self.report.destination
            ))
        }
    }
}

/// Executes parsed CLI arguments without any display-server dependency.
///
/// # Errors
///
/// Returns a stable typed failure for project loading, source import, preset lookup, or export.
pub fn execute(cli: Cli) -> Result<CliSuccess, CliError> {
    match cli.command {
        Command::Export(args) => export_saved_project(&args),
        Command::Quick(args) => export_quick(args),
    }
}

/// Exports a saved project through the shared core entry point.
///
/// # Errors
///
/// Returns project-load or core-export failures, and refuses sources outside
/// the project directory without `--allow-external-sources`.
pub fn export_saved_project(args: &ExportArgs) -> Result<CliSuccess, CliError> {
    let loaded = load_project(&args.project)?;
    if !args.allow_external_sources && !loaded.external_sources.is_empty() {
        return Err(CliError::ExternalSources {
            references: loaded
                .external_sources
                .into_iter()
                .map(|external| format!("{} -> {}", external.reference, external.path))
                .collect(),
        });
    }
    let mut project = loaded.project;
    project.export_settings.create_combined_zip = !args.common.no_zip;
    export_state(&project, &args.common)
}

/// Creates an in-memory one-preset project and exports it through the shared core entry point.
///
/// # Errors
///
/// Returns source, preset, or core-export failures.
pub fn export_quick(args: QuickArgs) -> Result<CliSuccess, CliError> {
    let preset = list_presets()
        .map_err(|error| CliError::Preset {
            id: error.to_string(),
        })?
        .into_iter()
        .find(|preset| preset.id == args.preset)
        .ok_or_else(|| CliError::Preset {
            id: args.preset.clone(),
        })?;
    let source = import_source(&args.source, "master")?;
    let name = args.name.unwrap_or_else(|| {
        args.source
            .file_stem()
            .and_then(|value| value.to_str())
            .filter(|value| !value.is_empty())
            .unwrap_or("Untitled")
            .to_owned()
    });
    let mut project = ProjectState::new(name);
    project.selected_presets = vec![SelectedPreset {
        id: preset.id,
        version: preset.version,
    }];
    project.enabled_outputs = preset
        .default_output_ids
        .into_iter()
        .collect::<BTreeSet<_>>();
    project.sources.push(ProjectSource {
        role: "master".to_owned(),
        path: source.path,
        content_hash: source.content_hash,
    });
    for role_source in args.role_sources {
        let source = import_source(&role_source.path, &role_source.role)?;
        project.sources.push(ProjectSource {
            role: role_source.role,
            path: source.path,
            content_hash: source.content_hash,
        });
    }
    project.export_settings.create_combined_zip = !args.common.no_zip;
    export_state(&project, &args.common)
}

fn export_state(project: &ProjectState, common: &CommonExportArgs) -> Result<CliSuccess, CliError> {
    fs::create_dir_all(&common.output).map_err(|error| CliError::OutputDirectory {
        path: common.output.to_string_lossy().into_owned(),
        message: error.to_string(),
    })?;
    let destination = default_export_path(&common.output, &project.base_filename);
    let report = export_project(
        project,
        destination,
        common.overwrite,
        &ExportCancellation::default(),
        |_| {},
    )?;
    Ok(CliSuccess {
        report,
        json: common.json,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_exit_codes_separate_input_and_export_failures() {
        assert_eq!(
            CliError::Preset {
                id: "missing".to_owned()
            }
            .exit_code(),
            2
        );
        assert_eq!(
            CliError::Export(ExportError::NoIncludedOutputs).exit_code(),
            4
        );
        assert_eq!(
            CliError::ReportSerialization {
                message: "failed".to_owned()
            }
            .exit_code(),
            5
        );
    }

    #[test]
    fn role_source_argument_preserves_windows_paths_after_first_separator() {
        let parsed = "foreground=C:\\art\\layer.svg"
            .parse::<RoleSourceArg>()
            .expect("parse role source");
        assert_eq!(parsed.role, "foreground");
        assert_eq!(parsed.path, PathBuf::from("C:\\art\\layer.svg"));
        assert!("foreground".parse::<RoleSourceArg>().is_err());
        assert!("=layer.svg".parse::<RoleSourceArg>().is_err());
    }
}

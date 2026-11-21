// SPDX-License-Identifier: MIT OR Apache-2.0
//! Human-readable project state and atomic project-file persistence.

use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    fs,
    io::Write,
    path::Path,
};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tempfile::NamedTempFile;
use thiserror::Error;

use imaguncula_presets::{DEFAULT_PRESET_IDENTITIES, load_bundled_preset};

/// Current `.imaguncula.json` schema version.
pub const PROJECT_SCHEMA_VERSION: u32 = 2;

/// Maximum accepted project-file size.
pub const PROJECT_FILE_MAX_BYTES: u64 = 1_048_576;

/// Complete persisted state for an Imaguncula project.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProjectState {
    pub schema_version: u32,
    pub project_name: String,
    pub base_filename: String,
    pub sources: Vec<ProjectSource>,
    pub selected_presets: Vec<SelectedPreset>,
    pub enabled_outputs: BTreeSet<String>,
    pub custom_outputs: Vec<CustomOutput>,
    pub output_overrides: BTreeMap<String, OutputOverride>,
    pub export_settings: ExportSettings,
    pub accepted_warnings: Vec<AcceptedWarning>,
    pub preset_migrations: Vec<PresetMigrationAcceptance>,
}

impl ProjectState {
    /// Creates an empty project using the current schema.
    #[must_use]
    pub fn new(project_name: impl Into<String>) -> Self {
        let project_name = project_name.into();
        let base_filename = default_base_filename(&project_name);

        let selected_presets = DEFAULT_PRESET_IDENTITIES
            .iter()
            .map(|(id, version)| SelectedPreset {
                id: (*id).to_owned(),
                version: *version,
            })
            .collect::<Vec<_>>();
        let enabled_outputs = DEFAULT_PRESET_IDENTITIES
            .iter()
            .filter_map(|(id, version)| load_bundled_preset(id, *version).ok())
            .flat_map(|preset| preset.outputs.into_iter().map(|output| output.id))
            .collect();

        Self {
            schema_version: PROJECT_SCHEMA_VERSION,
            project_name,
            base_filename,
            sources: Vec::new(),
            selected_presets,
            enabled_outputs,
            custom_outputs: Vec::new(),
            output_overrides: BTreeMap::new(),
            export_settings: ExportSettings::default(),
            accepted_warnings: Vec::new(),
            preset_migrations: Vec::new(),
        }
    }

    /// Returns every validation issue without mutating the project.
    #[must_use]
    pub fn validation_issues(&self) -> Vec<ProjectValidationIssue> {
        let mut issues = Vec::new();

        if self.schema_version != PROJECT_SCHEMA_VERSION {
            issues.push(ProjectValidationIssue::new(
                ProjectValidationCode::UnsupportedSchemaVersion,
                "schemaVersion",
                format!(
                    "schema version {} is not supported; expected {PROJECT_SCHEMA_VERSION}",
                    self.schema_version
                ),
            ));
        }
        validate_required_text(&self.project_name, "projectName", 200, &mut issues);
        validate_base_filename(&self.base_filename, &mut issues);

        validate_sources(&self.sources, &mut issues);
        validate_presets(&self.selected_presets, &mut issues);
        validate_custom_outputs(&self.custom_outputs, &self.selected_presets, &mut issues);
        validate_output_overrides(&self.output_overrides, &mut issues);
        validate_accepted_warnings(&self.accepted_warnings, &mut issues);
        validate_preset_migrations(&self.preset_migrations, &mut issues);

        issues
    }
}

/// A source path and its content identity.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProjectSource {
    pub role: String,
    pub path: String,
    pub content_hash: String,
}

/// A selected immutable preset version.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SelectedPreset {
    pub id: String,
    pub version: u32,
}

/// Format for a project-defined custom output.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    Png,
    Svg,
}

/// A user-defined standalone output.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CustomOutput {
    pub id: String,
    pub width: u32,
    pub height: u32,
    pub format: OutputFormat,
    pub filename: String,
    pub destination: String,
    pub source_role: String,
    pub usage: String,
}

/// User changes applied to one preset output.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OutputOverride {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub source_role: Option<String>,
    pub source_path: Option<String>,
    pub source_content_hash: Option<String>,
    pub allow_upscale: bool,
}

/// Project-level export behavior.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExportSettings {
    pub create_combined_zip: bool,
    pub overwrite_confirmed: bool,
}

impl Default for ExportSettings {
    fn default() -> Self {
        Self {
            create_combined_zip: true,
            overwrite_confirmed: false,
        }
    }
}

/// A typed warning acceptance persisted for reproducibility.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AcceptedWarning {
    pub code: String,
    pub output_id: Option<String>,
}

/// One explicit user acceptance of a reviewed immutable-preset version change.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PresetMigrationAcceptance {
    pub preset_id: String,
    pub from_version: u32,
    pub to_version: u32,
    pub diff_fingerprint: String,
}

/// One source reference that resolves outside the project file's directory.
///
/// Project files are portable and may come from someone else, so interfaces
/// must confirm these references before reading the files they name.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExternalSourceReference {
    /// Human-readable reference: a source role or an overridden output id.
    pub reference: String,
    /// The resolved path outside the project directory.
    pub path: String,
}

/// Result of loading a project, including future migration notices.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProjectLoad {
    pub project: ProjectState,
    pub migration_notices: Vec<String>,
    /// Source references that leave the project directory (absolute paths,
    /// drive prefixes, or `..` segments); none have been read yet at load time.
    pub external_sources: Vec<ExternalSourceReference>,
}

/// Stable machine-readable categories for project validation failures.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProjectValidationCode {
    ConflictingSelection,
    DuplicateIdentifier,
    InvalidBaseFilename,
    InvalidContentHash,
    InvalidIdentifier,
    InvalidRelativePath,
    MissingValue,
    OutOfRange,
    UnsupportedSchemaVersion,
    ValueTooLong,
}

/// One structured validation failure.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProjectValidationIssue {
    pub code: ProjectValidationCode,
    pub path: String,
    pub message: String,
}

impl ProjectValidationIssue {
    fn new(
        code: ProjectValidationCode,
        path: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            path: path.into(),
            message: message.into(),
        }
    }
}

/// Structured project-file load/save failures exposed through interface adapters.
#[derive(Debug, Error, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum ProjectFileError {
    #[error("project path must end with .imaguncula.json")]
    InvalidFileName { path: String },
    #[error("project file exceeds the {maximum_bytes}-byte limit")]
    FileTooLarge {
        path: String,
        maximum_bytes: u64,
        actual_bytes: u64,
    },
    #[error("failed to read project file")]
    Read { path: String, message: String },
    #[error("project file is not valid JSON")]
    InvalidJson { path: String, message: String },
    #[error("project file does not declare a supported schema version")]
    UnsupportedSchemaVersion { found: Option<u64>, supported: u32 },
    #[error("project file has an invalid structure")]
    InvalidStructure { path: String, message: String },
    #[error("project validation failed")]
    Validation { issues: Vec<ProjectValidationIssue> },
    #[error("failed to serialize project file")]
    Serialize { message: String },
    #[error("failed to write project file")]
    Write { path: String, message: String },
}

/// Loads and validates one `.imaguncula.json` file.
///
/// # Errors
///
/// Returns a structured error for invalid paths, I/O failures, unsupported schemas, malformed
/// JSON, or invalid project data.
pub fn load_project(path: impl AsRef<Path>) -> Result<ProjectLoad, ProjectFileError> {
    let path = path.as_ref();
    validate_project_path(path)?;

    let metadata = fs::metadata(path).map_err(|error| ProjectFileError::Read {
        path: display_path(path),
        message: error.to_string(),
    })?;
    if metadata.len() > PROJECT_FILE_MAX_BYTES {
        return Err(ProjectFileError::FileTooLarge {
            path: display_path(path),
            maximum_bytes: PROJECT_FILE_MAX_BYTES,
            actual_bytes: metadata.len(),
        });
    }

    let contents = fs::read(path).map_err(|error| ProjectFileError::Read {
        path: display_path(path),
        message: error.to_string(),
    })?;
    let value: Value =
        serde_json::from_slice(&contents).map_err(|error| ProjectFileError::InvalidJson {
            path: display_path(path),
            message: error.to_string(),
        })?;

    let declared_version = value.get("schemaVersion").and_then(Value::as_u64);
    let Some(declared_version) = declared_version else {
        return Err(ProjectFileError::UnsupportedSchemaVersion {
            found: declared_version,
            supported: PROJECT_SCHEMA_VERSION,
        });
    };
    if declared_version == 0 || declared_version > u64::from(PROJECT_SCHEMA_VERSION) {
        return Err(ProjectFileError::UnsupportedSchemaVersion {
            found: Some(declared_version),
            supported: PROJECT_SCHEMA_VERSION,
        });
    }

    let (value, migration_notices) = migrate_project_value(value, declared_version, path)?;

    let mut project = serde_json::from_value::<ProjectState>(value).map_err(|error| {
        ProjectFileError::InvalidStructure {
            path: display_path(path),
            message: error.to_string(),
        }
    })?;
    let external_sources = resolve_project_source_paths(&mut project, path);
    validate_project(&project)?;

    Ok(ProjectLoad {
        project,
        migration_notices,
        external_sources,
    })
}

fn migrate_project_value(
    mut value: Value,
    mut version: u64,
    path: &Path,
) -> Result<(Value, Vec<String>), ProjectFileError> {
    let mut notices = Vec::new();
    while version < u64::from(PROJECT_SCHEMA_VERSION) {
        match version {
            1 => {
                let object =
                    value
                        .as_object_mut()
                        .ok_or_else(|| ProjectFileError::InvalidStructure {
                            path: display_path(path),
                            message: "project root must be a JSON object".to_owned(),
                        })?;
                object.insert("schemaVersion".to_owned(), Value::from(2));
                object.insert("presetMigrations".to_owned(), Value::Array(Vec::new()));
                notices.push(
                    "Migrated project schema from version 1 to 2 and initialized preset migration history."
                        .to_owned(),
                );
                version = 2;
            }
            _ => {
                return Err(ProjectFileError::UnsupportedSchemaVersion {
                    found: Some(version),
                    supported: PROJECT_SCHEMA_VERSION,
                });
            }
        }
    }
    Ok((value, notices))
}

/// Validates and atomically saves one `.imaguncula.json` file.
///
/// # Errors
///
/// Returns a structured error for invalid paths, invalid project data, serialization failures, or
/// write failures. A failed write does not replace the destination.
pub fn save_project(
    path: impl AsRef<Path>,
    project: &ProjectState,
) -> Result<(), ProjectFileError> {
    let path = path.as_ref();
    validate_project_path(path)?;
    validate_project(project)?;

    let mut serializable = project.clone();
    relativize_project_source_paths(&mut serializable, path);

    let mut serialized =
        serde_json::to_vec_pretty(&serializable).map_err(|error| ProjectFileError::Serialize {
            message: error.to_string(),
        })?;
    serialized.push(b'\n');

    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty());
    let parent = parent.unwrap_or_else(|| Path::new("."));
    let mut temporary = NamedTempFile::new_in(parent).map_err(|error| ProjectFileError::Write {
        path: display_path(path),
        message: error.to_string(),
    })?;
    temporary
        .write_all(&serialized)
        .and_then(|()| temporary.as_file().sync_all())
        .map_err(|error| ProjectFileError::Write {
            path: display_path(path),
            message: error.to_string(),
        })?;
    temporary
        .persist(path)
        .map_err(|error| ProjectFileError::Write {
            path: display_path(path),
            message: error.error.to_string(),
        })?;

    Ok(())
}

fn validate_project(project: &ProjectState) -> Result<(), ProjectFileError> {
    let issues = project.validation_issues();
    if issues.is_empty() {
        Ok(())
    } else {
        Err(ProjectFileError::Validation { issues })
    }
}

fn validate_project_path(path: &Path) -> Result<(), ProjectFileError> {
    let valid = path
        .file_name()
        .and_then(|filename| filename.to_str())
        .is_some_and(|filename| filename.to_ascii_lowercase().ends_with(".imaguncula.json"));

    if valid {
        Ok(())
    } else {
        Err(ProjectFileError::InvalidFileName {
            path: display_path(path),
        })
    }
}

fn validate_sources(sources: &[ProjectSource], issues: &mut Vec<ProjectValidationIssue>) {
    let mut source_roles = HashSet::new();
    for (index, source) in sources.iter().enumerate() {
        validate_id(&source.role, &format!("sources[{index}].role"), issues);
        if !source_roles.insert(source.role.as_str()) {
            issues.push(ProjectValidationIssue::new(
                ProjectValidationCode::DuplicateIdentifier,
                format!("sources[{index}].role"),
                format!("source role `{}` occurs more than once", source.role),
            ));
        }
        validate_required_text(
            &source.path,
            &format!("sources[{index}].path"),
            4096,
            issues,
        );
        if !is_sha256(&source.content_hash) {
            issues.push(ProjectValidationIssue::new(
                ProjectValidationCode::InvalidContentHash,
                format!("sources[{index}].contentHash"),
                "content hash must be 64 lowercase hexadecimal SHA-256 characters",
            ));
        }
    }
}

fn validate_presets(presets: &[SelectedPreset], issues: &mut Vec<ProjectValidationIssue>) {
    let mut preset_ids = HashSet::new();
    let mut exclusive_groups = HashSet::new();
    for (index, preset) in presets.iter().enumerate() {
        validate_id(&preset.id, &format!("selectedPresets[{index}].id"), issues);
        if preset.version == 0 {
            issues.push(ProjectValidationIssue::new(
                ProjectValidationCode::OutOfRange,
                format!("selectedPresets[{index}].version"),
                "preset version must be at least 1",
            ));
        }
        if !preset_ids.insert(preset.id.as_str()) {
            issues.push(ProjectValidationIssue::new(
                ProjectValidationCode::DuplicateIdentifier,
                format!("selectedPresets[{index}].id"),
                format!("preset `{}` is selected more than once", preset.id),
            ));
        }
        if let Ok(definition) = load_bundled_preset(&preset.id, preset.version)
            && let Some(group) = definition.exclusive_group
            && !exclusive_groups.insert(group.clone())
        {
            issues.push(ProjectValidationIssue::new(
                ProjectValidationCode::ConflictingSelection,
                format!("selectedPresets[{index}].id"),
                format!(
                    "preset `{}` conflicts with another `{group}` selection",
                    preset.id
                ),
            ));
        }
    }
}

fn validate_custom_outputs(
    outputs: &[CustomOutput],
    presets: &[SelectedPreset],
    issues: &mut Vec<ProjectValidationIssue>,
) {
    let preset_output_ids: HashSet<_> = presets
        .iter()
        .filter_map(|preset| load_bundled_preset(&preset.id, preset.version).ok())
        .flat_map(|preset| preset.outputs.into_iter().map(|output| output.id))
        .collect();
    let mut custom_ids = HashSet::new();
    for (index, output) in outputs.iter().enumerate() {
        let prefix = format!("customOutputs[{index}]");
        validate_id(&output.id, &format!("{prefix}.id"), issues);
        if !custom_ids.insert(output.id.as_str()) {
            issues.push(ProjectValidationIssue::new(
                ProjectValidationCode::DuplicateIdentifier,
                format!("{prefix}.id"),
                format!("custom output `{}` occurs more than once", output.id),
            ));
        }
        if preset_output_ids.contains(&output.id) {
            issues.push(ProjectValidationIssue::new(
                ProjectValidationCode::DuplicateIdentifier,
                format!("{prefix}.id"),
                format!(
                    "custom output `{}` conflicts with a selected preset output",
                    output.id
                ),
            ));
        }
        validate_dimension(output.width, &format!("{prefix}.width"), issues);
        validate_dimension(output.height, &format!("{prefix}.height"), issues);
        validate_leaf_filename(&output.filename, &format!("{prefix}.filename"), issues);
        if !output.destination.is_empty() && !is_safe_relative_path(&output.destination) {
            issues.push(ProjectValidationIssue::new(
                ProjectValidationCode::InvalidRelativePath,
                format!("{prefix}.destination"),
                "destination must be a safe relative path",
            ));
        }
        validate_id(&output.source_role, &format!("{prefix}.sourceRole"), issues);
        validate_required_text(&output.usage, &format!("{prefix}.usage"), 500, issues);
    }
}

fn validate_output_overrides(
    output_overrides: &BTreeMap<String, OutputOverride>,
    issues: &mut Vec<ProjectValidationIssue>,
) {
    for (output_id, output_override) in output_overrides {
        validate_id(output_id, "outputOverrides.<key>", issues);
        if let Some(width) = output_override.width {
            validate_dimension(width, "outputOverrides.<value>.width", issues);
        }
        if let Some(height) = output_override.height {
            validate_dimension(height, "outputOverrides.<value>.height", issues);
        }
        if let Some(source_role) = &output_override.source_role {
            validate_id(source_role, "outputOverrides.<value>.sourceRole", issues);
        }
        match (
            &output_override.source_path,
            &output_override.source_content_hash,
        ) {
            (Some(path), Some(content_hash)) => {
                validate_required_text(path, "outputOverrides.<value>.sourcePath", 4096, issues);
                if !is_sha256(content_hash) {
                    issues.push(ProjectValidationIssue::new(
                        ProjectValidationCode::InvalidContentHash,
                        "outputOverrides.<value>.sourceContentHash",
                        "content hash must be 64 lowercase hexadecimal SHA-256 characters",
                    ));
                }
                if output_override.source_role.is_some() {
                    issues.push(ProjectValidationIssue::new(
                        ProjectValidationCode::InvalidRelativePath,
                        "outputOverrides.<value>",
                        "a direct source override cannot also select a project source role",
                    ));
                }
            }
            (None, None) => {}
            _ => issues.push(ProjectValidationIssue::new(
                ProjectValidationCode::MissingValue,
                "outputOverrides.<value>",
                "sourcePath and sourceContentHash must be set together",
            )),
        }
    }
}

fn resolve_project_source_paths(
    project: &mut ProjectState,
    project_path: &Path,
) -> Vec<ExternalSourceReference> {
    let parent = project_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let mut external = Vec::new();
    for source in &mut project.sources {
        let is_external = is_external_stored_path(&source.path);
        source.path = resolve_stored_path(parent, &source.path);
        if is_external {
            external.push(ExternalSourceReference {
                reference: format!("source role `{}`", source.role),
                path: source.path.clone(),
            });
        }
    }
    for (output_id, output_override) in &mut project.output_overrides {
        if let Some(source_path) = &mut output_override.source_path {
            let is_external = is_external_stored_path(source_path);
            *source_path = resolve_stored_path(parent, source_path);
            if is_external {
                external.push(ExternalSourceReference {
                    reference: format!("override for output `{output_id}`"),
                    path: source_path.clone(),
                });
            }
        }
    }
    external
}

/// Reports whether a stored source path can leave the project directory.
///
/// Checked on the stored text with both separators so classification does not
/// change with the operating system that opens the file.
fn is_external_stored_path(stored: &str) -> bool {
    stored.starts_with('/')
        || stored.starts_with('\\')
        || stored.as_bytes().get(1) == Some(&b':')
        || stored.split(['/', '\\']).any(|segment| segment == "..")
}

fn resolve_stored_path(parent: &Path, stored: &str) -> String {
    let stored_path = Path::new(stored);
    if stored_path.is_absolute() {
        stored.to_owned()
    } else {
        display_path(&parent.join(stored_path))
    }
}

fn relativize_project_source_paths(project: &mut ProjectState, project_path: &Path) {
    let parent = project_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    for source in &mut project.sources {
        source.path = relativize_stored_path(parent, &source.path);
    }
    for output_override in project.output_overrides.values_mut() {
        if let Some(source_path) = &mut output_override.source_path {
            *source_path = relativize_stored_path(parent, source_path);
        }
    }
}

fn relativize_stored_path(parent: &Path, stored: &str) -> String {
    Path::new(stored)
        .strip_prefix(parent)
        .ok()
        .and_then(Path::to_str)
        .map_or_else(|| stored.to_owned(), |relative| relative.replace('\\', "/"))
}

fn validate_accepted_warnings(
    warnings: &[AcceptedWarning],
    issues: &mut Vec<ProjectValidationIssue>,
) {
    for (index, warning) in warnings.iter().enumerate() {
        validate_id(
            &warning.code,
            &format!("acceptedWarnings[{index}].code"),
            issues,
        );
        if let Some(output_id) = &warning.output_id {
            validate_id(
                output_id,
                &format!("acceptedWarnings[{index}].outputId"),
                issues,
            );
        }
    }
}

fn validate_preset_migrations(
    migrations: &[PresetMigrationAcceptance],
    issues: &mut Vec<ProjectValidationIssue>,
) {
    let mut identities = HashSet::new();
    for (index, migration) in migrations.iter().enumerate() {
        let prefix = format!("presetMigrations[{index}]");
        validate_id(&migration.preset_id, &format!("{prefix}.presetId"), issues);
        if migration.from_version == 0 || migration.to_version <= migration.from_version {
            issues.push(ProjectValidationIssue::new(
                ProjectValidationCode::OutOfRange,
                format!("{prefix}.toVersion"),
                "preset migration versions must increase from a version of at least 1",
            ));
        }
        if !is_sha256(&migration.diff_fingerprint) {
            issues.push(ProjectValidationIssue::new(
                ProjectValidationCode::InvalidContentHash,
                format!("{prefix}.diffFingerprint"),
                "diff fingerprint must be 64 lowercase hexadecimal SHA-256 characters",
            ));
        }
        if !identities.insert((
            migration.preset_id.as_str(),
            migration.from_version,
            migration.to_version,
        )) {
            issues.push(ProjectValidationIssue::new(
                ProjectValidationCode::DuplicateIdentifier,
                prefix,
                "preset migration acceptance occurs more than once",
            ));
        }
    }
}

fn validate_required_text(
    value: &str,
    path: &str,
    maximum_length: usize,
    issues: &mut Vec<ProjectValidationIssue>,
) {
    if value.trim().is_empty() {
        issues.push(ProjectValidationIssue::new(
            ProjectValidationCode::MissingValue,
            path,
            "value must not be empty",
        ));
    } else if value.chars().count() > maximum_length {
        issues.push(ProjectValidationIssue::new(
            ProjectValidationCode::ValueTooLong,
            path,
            format!("value must contain at most {maximum_length} characters"),
        ));
    }
}

fn validate_base_filename(value: &str, issues: &mut Vec<ProjectValidationIssue>) {
    validate_required_text(value, "baseFilename", 120, issues);
    let invalid = value == "."
        || value == ".."
        || value.ends_with(['.', ' '])
        || value.chars().any(char::is_control)
        || is_windows_reserved_name(value)
        || value.chars().any(|character| {
            matches!(
                character,
                '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*'
            )
        });
    if invalid {
        issues.push(ProjectValidationIssue::new(
            ProjectValidationCode::InvalidBaseFilename,
            "baseFilename",
            "base filename contains a reserved path character or ending",
        ));
    }
}

fn validate_id(value: &str, path: &str, issues: &mut Vec<ProjectValidationIssue>) {
    if !is_valid_identifier(value) {
        issues.push(ProjectValidationIssue::new(
            ProjectValidationCode::InvalidIdentifier,
            path,
            "identifier must use lowercase ASCII letters, digits, and single hyphens",
        ));
    }
}

pub(crate) fn is_valid_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 80
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        && !value.starts_with('-')
        && !value.ends_with('-')
        && !value.contains("--")
}

fn validate_dimension(value: u32, path: &str, issues: &mut Vec<ProjectValidationIssue>) {
    if !(1..=16_384).contains(&value) {
        issues.push(ProjectValidationIssue::new(
            ProjectValidationCode::OutOfRange,
            path,
            "dimension must be between 1 and 16384 pixels",
        ));
    }
}

fn validate_leaf_filename(value: &str, path: &str, issues: &mut Vec<ProjectValidationIssue>) {
    let invalid = value.trim().is_empty()
        || value == "."
        || value == ".."
        || value.ends_with(['.', ' '])
        || value.chars().any(char::is_control)
        || is_windows_reserved_name(value)
        || value.chars().any(|character| {
            matches!(
                character,
                '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*'
            )
        });
    if invalid {
        issues.push(ProjectValidationIssue::new(
            ProjectValidationCode::InvalidRelativePath,
            path,
            "filename must be one safe path segment",
        ));
    }
}

fn is_safe_relative_path(value: &str) -> bool {
    let has_drive_prefix = value.as_bytes().get(1).is_some_and(|byte| *byte == b':');

    !value.is_empty()
        && !value.starts_with(['/', '\\'])
        && !value.contains('\\')
        && !has_drive_prefix
        && value
            .split('/')
            .all(|segment| !segment.is_empty() && segment != "." && segment != "..")
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

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn default_base_filename(project_name: &str) -> String {
    let mut result = String::new();
    let mut previous_was_separator = false;

    for character in project_name.chars() {
        if character.is_ascii_alphanumeric() {
            result.push(character.to_ascii_lowercase());
            previous_was_separator = false;
        } else if !result.is_empty() && !previous_was_separator {
            result.push('-');
            previous_was_separator = true;
        }
    }

    while result.ends_with('-') {
        result.pop();
    }
    if result.is_empty() {
        "untitled".to_owned()
    } else {
        result
    }
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::{
        OutputOverride, PROJECT_SCHEMA_VERSION, ProjectFileError, ProjectSource, ProjectState,
        ProjectValidationCode, SelectedPreset, default_base_filename, display_path,
        is_external_stored_path, is_safe_relative_path, load_project, save_project,
    };

    #[test]
    fn stored_paths_leaving_the_project_directory_are_classified_on_any_platform() {
        assert!(is_external_stored_path("C:/art/logo.svg"));
        assert!(is_external_stored_path("c:\\art\\logo.svg"));
        assert!(is_external_stored_path("\\\\server\\share\\logo.svg"));
        assert!(is_external_stored_path("/home/user/logo.svg"));
        assert!(is_external_stored_path("../fixtures/logo.svg"));
        assert!(is_external_stored_path("..\\fixtures\\logo.svg"));
        assert!(is_external_stored_path("nested/../../logo.svg"));

        assert!(!is_external_stored_path("logo.svg"));
        assert!(!is_external_stored_path("artwork/logo.svg"));
        assert!(!is_external_stored_path("artwork/..hidden/logo.svg"));
    }

    #[test]
    fn new_project_has_stable_defaults() {
        let project = ProjectState::new("Small Café Icon");

        assert_eq!(project.schema_version, PROJECT_SCHEMA_VERSION);
        assert_eq!(project.project_name, "Small Café Icon");
        assert_eq!(project.base_filename, "small-caf-icon");
        assert_eq!(project.selected_presets.len(), 2);
        assert!(
            project
                .selected_presets
                .iter()
                .any(|preset| preset.id == "windows-desktop" && preset.version == 2)
        );
        assert!(project.enabled_outputs.contains("icon-16"));
        assert!(project.enabled_outputs.contains("icon-42"));
        assert!(project.enabled_outputs.contains("pwa-512"));
        assert!(project.preset_migrations.is_empty());
        assert!(project.validation_issues().is_empty());
    }

    #[test]
    fn empty_name_uses_safe_base_name_but_remains_invalid() {
        let project = ProjectState::new("   ");

        assert_eq!(project.base_filename, "untitled");
        assert!(project.validation_issues().iter().any(|issue| {
            issue.code == ProjectValidationCode::MissingValue && issue.path == "projectName"
        }));
    }

    #[test]
    fn mutually_exclusive_preset_modes_fail_closed() {
        let mut project = ProjectState::new("Apple modes");
        project.selected_presets.extend([
            SelectedPreset {
                id: "apple-ios-modern".to_owned(),
                version: 1,
            },
            SelectedPreset {
                id: "apple-ios-compatibility".to_owned(),
                version: 1,
            },
        ]);

        assert!(project.validation_issues().iter().any(|issue| {
            issue.code == ProjectValidationCode::ConflictingSelection
                && issue.path == "selectedPresets[3].id"
        }));
    }

    #[test]
    fn project_round_trip_is_human_readable_and_replaces_atomically() {
        let directory = tempfile::tempdir().expect("temporary directory should be created");
        let path = directory.path().join("sample.imaguncula.json");
        let first = ProjectState::new("First project");
        save_project(&path, &first).expect("first project should save");

        let text = fs::read_to_string(&path).expect("saved project should be readable");
        assert!(text.contains("\n  \"schemaVersion\": 2,"));
        assert!(text.ends_with('\n'));
        assert_eq!(
            load_project(&path).expect("project should load").project,
            first
        );

        let second = ProjectState::new("Replacement project");
        save_project(&path, &second).expect("existing project should be replaced");
        assert_eq!(
            load_project(&path)
                .expect("replacement should load")
                .project,
            second
        );
    }

    #[test]
    fn load_rejects_unknown_or_future_schema_before_deserializing() {
        let directory = tempfile::tempdir().expect("temporary directory should be created");
        let path = directory.path().join("future.imaguncula.json");
        fs::write(&path, br#"{"schemaVersion":99,"unknown":true}"#)
            .expect("fixture should be written");

        assert!(matches!(
            load_project(&path),
            Err(ProjectFileError::UnsupportedSchemaVersion {
                found: Some(99),
                supported: PROJECT_SCHEMA_VERSION,
            })
        ));
    }

    #[test]
    fn load_migrates_schema_one_through_the_version_ladder() {
        let directory = tempfile::tempdir().expect("temporary directory should be created");
        let path = directory.path().join("legacy.imaguncula.json");
        let mut value =
            serde_json::to_value(ProjectState::new("Legacy project")).expect("serialize project");
        value["schemaVersion"] = serde_json::Value::from(1);
        value
            .as_object_mut()
            .expect("project is an object")
            .remove("presetMigrations");
        fs::write(
            &path,
            serde_json::to_vec_pretty(&value).expect("serialize legacy fixture"),
        )
        .expect("write legacy fixture");

        let loaded = load_project(&path).expect("schema one migrates");
        assert_eq!(loaded.project.schema_version, PROJECT_SCHEMA_VERSION);
        assert!(loaded.project.preset_migrations.is_empty());
        assert_eq!(loaded.migration_notices.len(), 1);
        assert!(loaded.migration_notices[0].contains("version 1 to 2"));
    }

    #[test]
    fn load_rejects_unknown_fields_in_current_schema() {
        let directory = tempfile::tempdir().expect("temporary directory should be created");
        let path = directory.path().join("unknown.imaguncula.json");
        let mut value =
            serde_json::to_value(ProjectState::new("Known project")).expect("serialize project");
        value["unknown"] = serde_json::Value::Bool(true);
        fs::write(
            &path,
            serde_json::to_vec(&value).expect("serialize fixture"),
        )
        .expect("fixture should be written");

        assert!(matches!(
            load_project(&path),
            Err(ProjectFileError::InvalidStructure { .. })
        ));
    }

    #[test]
    fn save_rejects_wrong_extension_and_invalid_state() {
        let directory = tempfile::tempdir().expect("temporary directory should be created");
        let project = ProjectState::new("Valid project");
        assert!(matches!(
            save_project(directory.path().join("project.json"), &project),
            Err(ProjectFileError::InvalidFileName { .. })
        ));

        let mut invalid = project;
        invalid.base_filename = "../escape".to_owned();
        assert!(matches!(
            save_project(directory.path().join("project.imaguncula.json"), &invalid),
            Err(ProjectFileError::Validation { .. })
        ));
    }

    #[test]
    fn base_filename_derivation_is_deterministic() {
        assert_eq!(default_base_filename("  Hello---WORLD  "), "hello-world");
        assert_eq!(default_base_filename("日本語"), "untitled");
    }

    #[test]
    fn validation_rejects_windows_reserved_names_on_every_host() {
        let mut project = ProjectState::new("Portable project");
        project.base_filename = "CON.png".to_owned();

        assert!(
            project
                .validation_issues()
                .iter()
                .any(|issue| issue.code == ProjectValidationCode::InvalidBaseFilename)
        );
    }

    #[test]
    fn relative_path_check_is_platform_independent() {
        assert!(is_safe_relative_path("icons/16x16"));
        assert!(!is_safe_relative_path("../escape"));
        assert!(!is_safe_relative_path("C:/escape"));
        assert!(!is_safe_relative_path("\\\\server\\share"));
    }

    #[test]
    fn project_local_sources_survive_moving_the_project_directory() {
        let root = tempfile::tempdir().expect("temporary directory");
        let original = root.path().join("original");
        let moved = root.path().join("moved");
        fs::create_dir(&original).expect("create original directory");
        let source = original.join("master.png");
        let override_source = original.join("small.png");
        fs::write(&source, b"master fixture").expect("write master fixture");
        fs::write(&override_source, b"override fixture").expect("write override fixture");
        let project_path = original.join("portable.imaguncula.json");
        let mut project = ProjectState::new("Portable project");
        project.sources.push(ProjectSource {
            role: "master".to_owned(),
            path: display_path(&source),
            content_hash: "0".repeat(64),
        });
        project.output_overrides.insert(
            "small".to_owned(),
            OutputOverride {
                source_path: Some(display_path(&override_source)),
                source_content_hash: Some("1".repeat(64)),
                ..OutputOverride::default()
            },
        );

        save_project(&project_path, &project).expect("save portable project");
        let serialized = fs::read_to_string(&project_path).expect("read saved project");
        assert!(serialized.contains("\"path\": \"master.png\""));
        assert!(serialized.contains("\"sourcePath\": \"small.png\""));

        fs::rename(&original, &moved).expect("move project directory");
        let loaded = load_project(moved.join("portable.imaguncula.json"))
            .expect("load moved project")
            .project;

        assert_eq!(
            loaded.sources[0].path,
            display_path(&moved.join("master.png"))
        );
        assert_eq!(
            loaded.output_overrides["small"].source_path,
            Some(display_path(&moved.join("small.png")))
        );
    }
}

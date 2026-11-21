// SPDX-License-Identifier: MIT OR Apache-2.0
//! Deterministic generation-report models and Markdown rendering.

use std::fmt::Write as _;

use serde::{Deserialize, Serialize};

use crate::{OutputIssueSeverity, OutputSpec, OutputStatus, ProjectState};

/// One generated file recorded in the export report.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GeneratedFile {
    pub output_id: String,
    pub path: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub format: String,
    pub source_label: String,
    pub source_path: String,
    pub source_content_hash: String,
}

/// One warning and whether the project explicitly accepted it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReportWarning {
    pub output_id: String,
    pub code: String,
    pub message: String,
    pub accepted: bool,
}

/// Complete deterministic report returned to interfaces and written as Markdown.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GenerationReport {
    pub project_name: String,
    pub base_filename: String,
    pub files: Vec<GeneratedFile>,
    pub warnings: Vec<ReportWarning>,
}

impl GenerationReport {
    #[must_use]
    pub fn from_specs(project: &ProjectState, specs: &[OutputSpec]) -> Self {
        let files = specs
            .iter()
            .filter(|spec| spec.included && spec.status != OutputStatus::Error)
            .filter_map(|spec| {
                Some(GeneratedFile {
                    output_id: spec.id.clone(),
                    path: spec.package_path.clone(),
                    width: Some(spec.width),
                    height: Some(spec.height),
                    format: format!("{:?}", spec.format).to_ascii_lowercase(),
                    source_label: spec.source_label.clone(),
                    source_path: spec.source_path.clone()?,
                    source_content_hash: spec.source_content_hash.clone()?,
                })
            })
            .collect();
        Self::from_generated_files(project, specs, files)
    }

    pub(crate) fn from_generated_files(
        project: &ProjectState,
        specs: &[OutputSpec],
        files: Vec<GeneratedFile>,
    ) -> Self {
        let warnings = specs
            .iter()
            .filter(|spec| spec.included)
            .flat_map(|spec| {
                spec.issues
                    .iter()
                    .filter(|issue| issue.severity == OutputIssueSeverity::Warning)
                    .map(|issue| ReportWarning {
                        output_id: spec.id.clone(),
                        code: issue.code.clone(),
                        message: issue.message.clone(),
                        accepted: issue.code == "raster-upscale-allowed"
                            || project.accepted_warnings.iter().any(|accepted| {
                                accepted.code == issue.code
                                    && accepted
                                        .output_id
                                        .as_ref()
                                        .is_none_or(|output_id| output_id == &spec.id)
                            }),
                    })
            })
            .collect();

        Self {
            project_name: project.project_name.clone(),
            base_filename: project.base_filename.clone(),
            files,
            warnings,
        }
    }

    #[must_use]
    pub fn to_markdown(&self) -> String {
        let mut markdown = format!(
            "# Imaguncula generation report\n\nProject: **{}**  \nBase filename: `{}`\n\n## Generated files\n\n| Output | Path | Dimensions | Format | Source |\n| --- | --- | ---: | --- | --- |\n",
            escape_markdown(&self.project_name),
            escape_code(&self.base_filename)
        );
        for file in &self.files {
            let dimensions = match (file.width, file.height, file.format.as_str()) {
                (Some(width), Some(height), _) => format!("{width}x{height}"),
                (_, _, "ico" | "icns") => "multi-size".to_owned(),
                (_, _, "svg") => "scalable".to_owned(),
                _ => "n/a".to_owned(),
            };
            let _ = writeln!(
                markdown,
                "| {} | `{}` | {} | {} | {} (`{}`, sha256 `{}`) |",
                escape_markdown(&file.output_id),
                escape_code(&file.path),
                dimensions,
                escape_markdown(&file.format.to_ascii_uppercase()),
                escape_markdown(&file.source_label),
                escape_code(source_file_name(&file.source_path)),
                escape_code(&file.source_content_hash)
            );
        }
        if self.files.is_empty() {
            markdown.push_str("| None | — | — | — | — |\n");
        }

        markdown.push_str("\n## Warnings\n\n");
        if self.warnings.is_empty() {
            markdown.push_str("No warnings.\n");
        } else {
            for warning in &self.warnings {
                let _ = writeln!(
                    markdown,
                    "- [{}] **{}** `{}`: {}",
                    if warning.accepted {
                        "accepted"
                    } else {
                        "outstanding"
                    },
                    escape_markdown(&warning.output_id),
                    escape_code(&warning.code),
                    escape_markdown(&warning.message)
                );
            }
        }
        markdown
    }
}

/// The report ships inside every exported package, so it must identify the
/// source without disclosing the local directory layout.
fn source_file_name(path: &str) -> &str {
    path.rsplit(['/', '\\']).next().unwrap_or(path)
}

fn escape_markdown(value: &str) -> String {
    value.replace(['\r', '\n'], " ").replace('|', "\\|")
}

fn escape_code(value: &str) -> String {
    value
        .replace(['\r', '\n'], " ")
        .replace('`', "'")
        .replace('|', "\\|")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markdown_report_is_deterministic_and_escapes_project_text() {
        let report = GenerationReport {
            project_name: "One | Two".to_owned(),
            base_filename: "one-two".to_owned(),
            files: vec![GeneratedFile {
                output_id: "custom-16".to_owned(),
                path: "icons/custom.png".to_owned(),
                width: Some(16),
                height: Some(16),
                format: "png".to_owned(),
                source_label: "master".to_owned(),
                source_path: "C:\\Users\\someone\\artwork/source.png".to_owned(),
                source_content_hash: "0".repeat(64),
            }],
            warnings: Vec::new(),
        };

        let first = report.to_markdown();
        assert_eq!(first, report.to_markdown());
        assert!(first.contains("One \\| Two"));
        assert!(first.contains("16x16"));
        assert!(first.contains("`source.png`"));
        assert!(first.contains(&"0".repeat(64)));
        assert!(
            !first.contains("someone"),
            "report markdown must not disclose local directory layout"
        );
        assert!(first.ends_with("No warnings.\n"));
    }
}

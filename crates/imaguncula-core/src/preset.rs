// SPDX-License-Identifier: MIT OR Apache-2.0
//! Typed access to immutable presets compiled into the application.

use std::collections::HashSet;

use imaguncula_presets::{
    PresetDiff, PresetSummary, bundled_preset_summaries, compare_preset_definitions,
    latest_bundled_preset_version, load_bundled_preset,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::project::{PresetMigrationAcceptance, ProjectState};

/// Failure to enumerate compile-time preset data.
#[derive(Debug, Error, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum PresetQueryError {
    #[error("bundled preset data is invalid")]
    InvalidBundledData { message: String },
}

/// A reviewable update from the version pinned by a project to the newest bundled version.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PresetUpgrade {
    pub diff: PresetDiff,
    pub diff_fingerprint: String,
}

/// Stable preset-upgrade failures exposed through interface adapters.
#[derive(Debug, Error, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum PresetUpgradeError {
    #[error("preset `{preset_id}` is not selected by the project")]
    PresetNotSelected { preset_id: String },
    #[error("preset `{preset_id}` version {target_version} is not the newest bundled version")]
    TargetIsNotLatest {
        preset_id: String,
        target_version: u32,
        latest_version: Option<u32>,
    },
    #[error("preset `{preset_id}` is already at version {version}")]
    NoVersionChange { preset_id: String, version: u32 },
    #[error("the reviewed preset comparison is stale; review the current diff again")]
    StaleComparison,
    #[error("bundled preset data is invalid")]
    InvalidBundledData { message: String },
}

/// Returns summaries for the newest version of every preset compiled into this application.
///
/// # Errors
///
/// Returns [`PresetQueryError::InvalidBundledData`] if checked-in preset data
/// cannot be deserialized by its typed model.
pub fn list_presets() -> Result<Vec<PresetSummary>, PresetQueryError> {
    bundled_preset_summaries().map_err(|error| PresetQueryError::InvalidBundledData {
        message: error.to_string(),
    })
}

/// Returns every newer bundled preset version requiring explicit project migration acceptance.
///
/// # Errors
///
/// Returns [`PresetQueryError::InvalidBundledData`] if an exact bundled version cannot be loaded
/// or compared.
pub fn list_preset_upgrades(
    project: &ProjectState,
) -> Result<Vec<PresetUpgrade>, PresetQueryError> {
    let mut upgrades = Vec::new();
    for selected in &project.selected_presets {
        let Some(latest_version) = latest_bundled_preset_version(&selected.id) else {
            continue;
        };
        if latest_version <= selected.version {
            continue;
        }
        let from = load_bundled_preset(&selected.id, selected.version).map_err(query_error)?;
        let to = load_bundled_preset(&selected.id, latest_version).map_err(query_error)?;
        let diff = compare_preset_definitions(&from, &to).map_err(query_error)?;
        upgrades.push(PresetUpgrade {
            diff_fingerprint: preset_diff_fingerprint(&diff),
            diff,
        });
    }
    Ok(upgrades)
}

/// Applies one reviewed preset upgrade and records the exact diff acceptance in project state.
///
/// Newly added outputs are enabled only when the destination preset enables them by default.
/// Removed outputs and overrides are deleted; explicitly renamed outputs preserve inclusion and
/// per-output overrides.
///
/// # Errors
///
/// Returns a typed failure for missing selections, stale review fingerprints, non-latest targets,
/// or invalid bundled data.
pub fn apply_preset_upgrade(
    project: &ProjectState,
    preset_id: &str,
    target_version: u32,
    accepted_diff_fingerprint: &str,
) -> Result<ProjectState, PresetUpgradeError> {
    let selected = project
        .selected_presets
        .iter()
        .find(|preset| preset.id == preset_id)
        .ok_or_else(|| PresetUpgradeError::PresetNotSelected {
            preset_id: preset_id.to_owned(),
        })?;
    if selected.version == target_version {
        return Err(PresetUpgradeError::NoVersionChange {
            preset_id: preset_id.to_owned(),
            version: target_version,
        });
    }
    let latest_version = latest_bundled_preset_version(preset_id);
    if latest_version != Some(target_version) {
        return Err(PresetUpgradeError::TargetIsNotLatest {
            preset_id: preset_id.to_owned(),
            target_version,
            latest_version,
        });
    }

    let from = load_bundled_preset(preset_id, selected.version).map_err(upgrade_error)?;
    let to = load_bundled_preset(preset_id, target_version).map_err(upgrade_error)?;
    let diff = compare_preset_definitions(&from, &to).map_err(upgrade_error)?;
    let diff_fingerprint = preset_diff_fingerprint(&diff);
    if diff_fingerprint != accepted_diff_fingerprint {
        return Err(PresetUpgradeError::StaleComparison);
    }

    let summaries = bundled_preset_summaries().map_err(upgrade_error)?;
    let default_outputs = summaries
        .iter()
        .find(|summary| summary.id == preset_id && summary.version == target_version)
        .map(|summary| summary.default_output_ids.iter().collect::<HashSet<_>>())
        .ok_or_else(|| PresetUpgradeError::InvalidBundledData {
            message: format!("missing summary for `{preset_id}` version {target_version}"),
        })?;

    let mut migrated = project.clone();
    let migrated_selection = migrated
        .selected_presets
        .iter_mut()
        .find(|preset| preset.id == preset_id)
        .ok_or_else(|| PresetUpgradeError::PresetNotSelected {
            preset_id: preset_id.to_owned(),
        })?;
    migrated_selection.version = target_version;

    for output_id in &diff.removed_outputs {
        migrated.enabled_outputs.remove(output_id);
        migrated.output_overrides.remove(output_id);
    }
    for rename in &diff.renamed_outputs {
        if migrated.enabled_outputs.remove(&rename.from) {
            migrated.enabled_outputs.insert(rename.to.clone());
        }
        if let Some(output_override) = migrated.output_overrides.remove(&rename.from) {
            migrated
                .output_overrides
                .entry(rename.to.clone())
                .or_insert(output_override);
        }
    }
    for output_id in &diff.added_outputs {
        if default_outputs.contains(output_id) {
            migrated.enabled_outputs.insert(output_id.clone());
        }
    }
    migrated.preset_migrations.push(PresetMigrationAcceptance {
        preset_id: preset_id.to_owned(),
        from_version: selected.version,
        to_version: target_version,
        diff_fingerprint,
    });

    Ok(migrated)
}

fn preset_diff_fingerprint(diff: &PresetDiff) -> String {
    let mut hasher = Sha256::new();
    hasher.update(diff.preset_id.as_bytes());
    hasher.update([0]);
    hasher.update(diff.from_version.to_le_bytes());
    hasher.update(diff.to_version.to_le_bytes());
    for (kind, ids) in [
        (b"added".as_slice(), diff.added_outputs.as_slice()),
        (b"removed".as_slice(), diff.removed_outputs.as_slice()),
    ] {
        hasher.update(kind);
        hasher.update([0]);
        for id in ids {
            hasher.update(id.as_bytes());
            hasher.update([0]);
        }
    }
    hasher.update(b"renamed");
    hasher.update([0]);
    for rename in &diff.renamed_outputs {
        hasher.update(rename.from.as_bytes());
        hasher.update([0]);
        hasher.update(rename.to.as_bytes());
        hasher.update([0]);
    }
    format!("{:x}", hasher.finalize())
}

fn query_error(error: impl std::fmt::Display) -> PresetQueryError {
    PresetQueryError::InvalidBundledData {
        message: error.to_string(),
    }
}

fn upgrade_error(error: impl std::fmt::Display) -> PresetUpgradeError {
    PresetUpgradeError::InvalidBundledData {
        message: error.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::SelectedPreset;

    #[test]
    fn lists_latest_bundled_presets_with_output_ids() {
        let presets = list_presets().expect("list bundled presets");
        assert_eq!(presets.len(), 10);
        for (id, output_id) in [
            ("windows-desktop", "icon-256"),
            ("web-pwa", "pwa-512"),
            ("linux-hicolor", "linux-scalable"),
            ("apple-ios-modern", "ios-modern-1024"),
            ("apple-macos-icns", "mac-icns-1024"),
            ("apple-icon-composer-guidance", "icon-composer-foreground"),
            ("android-launcher", "android-play-store-512"),
            ("windows-msix", "msix-app-list-target-256"),
        ] {
            let preset = presets
                .iter()
                .find(|preset| preset.id == id)
                .expect("expected bundled preset");
            assert!(preset.output_ids.contains(&output_id.to_owned()));
        }
    }

    #[test]
    fn msix_upgrade_requires_the_reviewed_fingerprint_and_persists_acceptance() {
        let mut project = ProjectState::new("Migration fixture");
        project.selected_presets.push(SelectedPreset {
            id: "windows-msix".to_owned(),
            version: 1,
        });
        project
            .enabled_outputs
            .insert("msix-app-list-target-16".to_owned());

        let upgrades = list_preset_upgrades(&project).expect("upgrade comparison succeeds");
        assert_eq!(upgrades.len(), 1);
        let upgrade = &upgrades[0];
        assert_eq!(upgrade.diff.preset_id, "windows-msix");
        assert_eq!(upgrade.diff.from_version, 1);
        assert_eq!(upgrade.diff.to_version, 2);
        assert_eq!(upgrade.diff.added_outputs.len(), 12);
        assert!(upgrade.diff.removed_outputs.is_empty());
        assert!(matches!(
            apply_preset_upgrade(&project, "windows-msix", 2, &"0".repeat(64)),
            Err(PresetUpgradeError::StaleComparison)
        ));

        let migrated = apply_preset_upgrade(&project, "windows-msix", 2, &upgrade.diff_fingerprint)
            .expect("reviewed comparison applies");
        assert_eq!(
            migrated
                .selected_presets
                .iter()
                .find(|preset| preset.id == "windows-msix")
                .expect("MSIX remains selected")
                .version,
            2
        );
        assert_eq!(migrated.preset_migrations.len(), 1);
        assert_eq!(
            migrated.preset_migrations[0].diff_fingerprint,
            upgrade.diff_fingerprint
        );
        assert!(
            upgrade
                .diff
                .added_outputs
                .iter()
                .all(|id| !migrated.enabled_outputs.contains(id))
        );
        assert!(migrated.validation_issues().is_empty());
    }

    #[test]
    fn windows_desktop_upgrade_adds_and_enables_the_42_pixel_entry() {
        let mut project = ProjectState::new("Windows desktop migration fixture");
        let selected = project
            .selected_presets
            .iter_mut()
            .find(|preset| preset.id == "windows-desktop")
            .expect("Windows desktop is selected by default");
        selected.version = 1;
        project.enabled_outputs.remove("icon-42");

        let upgrades = list_preset_upgrades(&project).expect("upgrade comparison succeeds");
        assert_eq!(upgrades.len(), 1);
        let upgrade = &upgrades[0];
        assert_eq!(upgrade.diff.preset_id, "windows-desktop");
        assert_eq!(upgrade.diff.from_version, 1);
        assert_eq!(upgrade.diff.to_version, 2);
        assert_eq!(upgrade.diff.added_outputs, ["icon-42".to_owned()]);
        assert!(upgrade.diff.removed_outputs.is_empty());
        assert!(upgrade.diff.renamed_outputs.is_empty());

        let migrated =
            apply_preset_upgrade(&project, "windows-desktop", 2, &upgrade.diff_fingerprint)
                .expect("reviewed Windows desktop upgrade applies");
        assert!(migrated.enabled_outputs.contains("icon-42"));
        assert_eq!(
            migrated
                .selected_presets
                .iter()
                .find(|preset| preset.id == "windows-desktop")
                .expect("Windows desktop remains selected")
                .version,
            2
        );
        assert_eq!(migrated.preset_migrations.len(), 1);
        assert!(migrated.validation_issues().is_empty());
    }
}

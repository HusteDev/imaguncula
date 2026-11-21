// SPDX-License-Identifier: MIT OR Apache-2.0
#[tauri::command]
fn application_name() -> &'static str {
    imaguncula_core::APPLICATION_NAME
}

#[tauri::command]
fn new_project(project_name: String) -> imaguncula_core::ProjectState {
    imaguncula_core::ProjectState::new(project_name)
}

#[tauri::command]
fn list_presets() -> Result<Vec<imaguncula_core::PresetSummary>, imaguncula_core::PresetQueryError>
{
    imaguncula_core::list_presets()
}

#[tauri::command]
#[expect(
    clippy::needless_pass_by_value,
    reason = "Tauri command payloads are deserialized into owned values"
)]
fn list_preset_upgrades(
    project: imaguncula_core::ProjectState,
) -> Result<Vec<imaguncula_core::PresetUpgrade>, imaguncula_core::PresetQueryError> {
    imaguncula_core::list_preset_upgrades(&project)
}

#[tauri::command]
#[expect(
    clippy::needless_pass_by_value,
    reason = "Tauri command payloads are deserialized into owned values"
)]
fn apply_preset_upgrade(
    project: imaguncula_core::ProjectState,
    preset_id: String,
    target_version: u32,
    accepted_diff_fingerprint: String,
) -> Result<imaguncula_core::ProjectState, imaguncula_core::PresetUpgradeError> {
    imaguncula_core::apply_preset_upgrade(
        &project,
        &preset_id,
        target_version,
        &accepted_diff_fingerprint,
    )
}

#[tauri::command]
fn load_project(
    path: String,
) -> Result<imaguncula_core::ProjectLoad, imaguncula_core::ProjectFileError> {
    imaguncula_core::load_project(path)
}

#[tauri::command]
#[expect(
    clippy::needless_pass_by_value,
    reason = "Tauri command payloads are deserialized into owned values"
)]
fn save_project(
    path: String,
    project: imaguncula_core::ProjectState,
) -> Result<(), imaguncula_core::ProjectFileError> {
    imaguncula_core::save_project(path, &project)
}

#[tauri::command]
#[expect(
    clippy::needless_pass_by_value,
    reason = "Tauri command payloads are deserialized into owned values"
)]
fn resolve_outputs(
    project: imaguncula_core::ProjectState,
) -> Result<Vec<imaguncula_core::OutputSpec>, imaguncula_core::ResolveOutputsError> {
    imaguncula_core::resolve_outputs(&project)
}

#[tauri::command]
#[expect(
    clippy::needless_pass_by_value,
    reason = "Tauri command payloads are deserialized into owned values"
)]
fn inspect_export_destination(
    parent: String,
    base_filename: String,
) -> Result<imaguncula_core::DestinationInspection, imaguncula_core::ExportError> {
    let destination = imaguncula_core::default_export_path(parent, &base_filename);
    imaguncula_core::inspect_export_destination(destination)
}

#[tauri::command]
#[expect(
    clippy::needless_pass_by_value,
    reason = "Tauri injects app/state and deserializes command payloads by value"
)]
fn start_export(
    app: tauri::AppHandle,
    state: tauri::State<'_, ExportJobs>,
    project: imaguncula_core::ProjectState,
    parent: String,
    overwrite: bool,
) -> Result<String, ExportCommandError> {
    let sequence = state.next_id.fetch_add(1, Ordering::Relaxed) + 1;
    let job_id = format!("export-{sequence}");
    let cancellation = imaguncula_core::ExportCancellation::default();
    {
        let mut active = state
            .active
            .lock()
            .map_err(|_| ExportCommandError::JobsUnavailable)?;
        if !active.is_empty() {
            return Err(ExportCommandError::JobAlreadyRunning);
        }
        active.insert(job_id.clone(), cancellation.clone());
    }

    let destination = imaguncula_core::default_export_path(parent, &project.base_filename);
    let active = Arc::clone(&state.active);
    let thread_job_id = job_id.clone();
    let thread_result = std::thread::Builder::new()
        .name(job_id.clone())
        .spawn(move || {
            let progress_app = app.clone();
            let progress_job_id = thread_job_id.clone();
            let result = imaguncula_core::export_project(
                &project,
                destination,
                overwrite,
                &cancellation,
                move |progress| {
                    let _ = progress_app.emit(
                        "export-progress",
                        ExportProgressEvent {
                            job_id: progress_job_id.clone(),
                            completed: progress.completed,
                            total: progress.total,
                            current_output: progress.current_output,
                        },
                    );
                },
            );
            let finished = match result {
                Ok(report) => ExportFinishedEvent {
                    job_id: thread_job_id.clone(),
                    report: Some(report),
                    error: None,
                },
                Err(error) => ExportFinishedEvent {
                    job_id: thread_job_id.clone(),
                    report: None,
                    error: Some(serde_json::to_value(&error).unwrap_or_else(
                        |serialization_error| {
                            serde_json::json!({
                                "kind": "serialization-failed",
                                "message": serialization_error.to_string()
                            })
                        },
                    )),
                },
            };
            let _ = app.emit("export-finished", finished);
            if let Ok(mut jobs) = active.lock() {
                jobs.remove(&thread_job_id);
            }
        });

    if let Err(error) = thread_result {
        state
            .active
            .lock()
            .map_err(|_| ExportCommandError::JobsUnavailable)?
            .remove(&job_id);
        return Err(ExportCommandError::Spawn {
            message: error.to_string(),
        });
    }
    Ok(job_id)
}

#[tauri::command]
#[expect(
    clippy::needless_pass_by_value,
    reason = "Tauri injects command state and deserializes command payloads by value"
)]
fn cancel_export(
    state: tauri::State<'_, ExportJobs>,
    job_id: String,
) -> Result<bool, ExportCommandError> {
    let active = state
        .active
        .lock()
        .map_err(|_| ExportCommandError::JobsUnavailable)?;
    if let Some(cancellation) = active.get(&job_id) {
        cancellation.cancel();
        Ok(true)
    } else {
        Ok(false)
    }
}

#[tauri::command]
#[expect(
    clippy::needless_pass_by_value,
    reason = "Tauri injects command state and deserializes command payloads by value"
)]
fn import_source(
    state: tauri::State<'_, Mutex<imaguncula_core::PreviewCache>>,
    path: String,
    role: String,
) -> Result<imaguncula_core::SourceInfo, imaguncula_core::PreviewError> {
    state
        .lock()
        .map_err(|_| imaguncula_core::PreviewError::CacheUnavailable)?
        .import(path, role)
        .map_err(|error| imaguncula_core::PreviewError::Source { error })
}

#[tauri::command]
#[expect(
    clippy::needless_pass_by_value,
    reason = "Tauri injects command state and deserializes command payloads by value"
)]
fn render_preview(
    state: tauri::State<'_, Mutex<imaguncula_core::PreviewCache>>,
    path: String,
    role: String,
    width: u32,
    height: u32,
    allow_upscale: bool,
) -> Result<imaguncula_core::PreviewImage, imaguncula_core::PreviewError> {
    // Hold the shared cache lock only for validation and cache bookkeeping;
    // rendering runs unlocked so one slow source cannot block every other
    // preview and import command.
    let prepared = state
        .lock()
        .map_err(|_| imaguncula_core::PreviewError::CacheUnavailable)?
        .prepare_render(path, role, width, height, allow_upscale)?;
    match prepared {
        imaguncula_core::PreparedPreview::Cached(image) => Ok(image),
        imaguncula_core::PreparedPreview::Pending(pending) => {
            let image = pending.compute()?;
            if let Ok(mut cache) = state.lock() {
                cache.store_preview(&image);
            }
            Ok(image)
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
/// Starts the Imaguncula desktop application.
///
/// # Panics
///
/// Panics when Tauri cannot initialize or run the desktop application.
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(Mutex::new(imaguncula_core::PreviewCache::default()))
        .manage(ExportJobs::default())
        .invoke_handler(tauri::generate_handler![
            apply_preset_upgrade,
            application_name,
            cancel_export,
            import_source,
            inspect_export_destination,
            list_presets,
            list_preset_upgrades,
            load_project,
            new_project,
            render_preview,
            resolve_outputs,
            start_export,
            save_project
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
};

use serde::Serialize;
use tauri::Emitter;

#[derive(Default)]
struct ExportJobs {
    next_id: AtomicU64,
    active: Arc<Mutex<HashMap<String, imaguncula_core::ExportCancellation>>>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
enum ExportCommandError {
    JobsUnavailable,
    JobAlreadyRunning,
    Spawn { message: String },
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ExportProgressEvent {
    job_id: String,
    completed: usize,
    total: usize,
    current_output: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ExportFinishedEvent {
    job_id: String,
    report: Option<imaguncula_core::ExportReport>,
    error: Option<serde_json::Value>,
}

// SPDX-License-Identifier: MIT OR Apache-2.0
//! Tests for the desktop IPC adapter. Commands are called directly; commands
//! that need Tauri state or an app handle run against Tauri's mock runtime.

use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    path::{Path, PathBuf},
    sync::{Mutex, mpsc},
    time::{Duration, Instant},
};

use tauri::{
    Listener, Manager,
    test::{MockRuntime, mock_builder, mock_context, noop_assets},
};

use super::{
    ExportCommandError, ExportJobs, application_name, apply_preset_upgrade, cancel_export,
    import_source, inspect_export_destination, list_preset_upgrades, list_presets, load_project,
    new_project, render_preview, resolve_outputs, save_project, start_export,
};

const ROLE: &str = "master";

fn fixture_source() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../fixtures/sources/asymmetric.svg")
}

fn path_string(path: &Path) -> String {
    path.to_str().expect("UTF-8 temporary path").to_owned()
}

fn project_with_source() -> imaguncula_core::ProjectState {
    let source = imaguncula_core::import_source(fixture_source(), ROLE).expect("import fixture");
    let mut project = imaguncula_core::ProjectState::new("Adapter test");
    project.sources.push(imaguncula_core::ProjectSource {
        role: ROLE.to_owned(),
        path: source.path,
        content_hash: source.content_hash,
    });
    project
}

fn mock_app() -> tauri::App<MockRuntime> {
    mock_builder()
        .manage(Mutex::new(imaguncula_core::PreviewCache::default()))
        .manage(ExportJobs::default())
        .build(mock_context(noop_assets()))
        .expect("build mock application")
}

fn poison<T>(mutex: &Mutex<T>) {
    let _ = catch_unwind(AssertUnwindSafe(|| {
        let _guard = mutex.lock().expect("lock before poisoning");
        panic!("poison the lock for an error-path test");
    }));
    assert!(mutex.is_poisoned());
}

fn start(
    app: &tauri::App<MockRuntime>,
    project: imaguncula_core::ProjectState,
    parent: String,
) -> Result<String, ExportCommandError> {
    start_export(
        app.handle().clone(),
        app.state::<ExportJobs>(),
        project,
        parent,
        false,
    )
}

/// Starts an export and returns the `export-finished` event payload once the
/// worker has finished and released its job.
fn run_export(
    app: &tauri::App<MockRuntime>,
    project: imaguncula_core::ProjectState,
    parent: String,
) -> serde_json::Value {
    let (sender, receiver) = mpsc::channel();
    app.listen("export-finished", move |event| {
        let _ = sender.send(event.payload().to_owned());
    });
    let job_id = start(app, project, parent).expect("start export");
    assert_eq!(job_id, "export-1");

    let payload = receiver
        .recv_timeout(Duration::from_secs(120))
        .expect("export-finished event");
    let finished: serde_json::Value =
        serde_json::from_str(&payload).expect("finished payload is JSON");
    assert_eq!(finished["jobId"], "export-1");

    // The worker removes its job right after emitting the event.
    let deadline = Instant::now() + Duration::from_secs(10);
    while !app
        .state::<ExportJobs>()
        .active
        .lock()
        .expect("jobs lock")
        .is_empty()
    {
        assert!(Instant::now() < deadline, "finished job was not removed");
        std::thread::sleep(Duration::from_millis(10));
    }
    finished
}

#[test]
fn simple_commands_delegate_to_the_core() {
    assert_eq!(application_name(), imaguncula_core::APPLICATION_NAME);
    assert_eq!(new_project("Docs".to_owned()).project_name, "Docs");

    let presets = list_presets().expect("list presets");
    assert!(presets.iter().any(|preset| preset.id == "windows-desktop"));

    let project = project_with_source();
    list_preset_upgrades(project.clone()).expect("list preset upgrades");
    assert!(
        !resolve_outputs(project)
            .expect("resolve outputs")
            .is_empty()
    );
}

#[test]
fn apply_preset_upgrade_reports_core_errors() {
    let project = new_project("Upgrade".to_owned());
    let not_selected = apply_preset_upgrade(
        project.clone(),
        "no-such-preset".to_owned(),
        1,
        "fingerprint".to_owned(),
    );
    assert!(matches!(
        not_selected,
        Err(imaguncula_core::PresetUpgradeError::PresetNotSelected { .. })
    ));

    let selected = project.selected_presets[0].id.clone();
    let not_latest = apply_preset_upgrade(project, selected, u32::MAX, "fingerprint".to_owned());
    assert!(matches!(
        not_latest,
        Err(imaguncula_core::PresetUpgradeError::TargetIsNotLatest { .. })
    ));
}

#[test]
fn project_files_round_trip_through_the_commands() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let path = path_string(&directory.path().join("round.imaguncula.json"));
    let project = new_project("Round trip".to_owned());

    save_project(path.clone(), project.clone()).expect("save project");
    assert_eq!(load_project(path).expect("load project").project, project);

    let wrong_name = load_project(path_string(&directory.path().join("round.json")));
    assert!(matches!(
        wrong_name,
        Err(imaguncula_core::ProjectFileError::InvalidFileName { .. })
    ));
}

#[test]
fn inspect_export_destination_reports_a_missing_package() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let inspection = inspect_export_destination(path_string(directory.path()), "icons".to_owned())
        .expect("inspect destination");
    assert_eq!(inspection.state, imaguncula_core::DestinationState::Missing);
    assert!(!inspection.requires_confirmation);
}

#[test]
fn import_and_preview_use_the_shared_cache() {
    let app = mock_app();
    let cache = app.state::<Mutex<imaguncula_core::PreviewCache>>();
    let source = path_string(&fixture_source());

    let info = import_source(cache.clone(), source.clone(), ROLE.to_owned()).expect("import");
    assert_eq!(info.role, ROLE);

    let first = render_preview(
        cache.clone(),
        source.clone(),
        ROLE.to_owned(),
        32,
        32,
        false,
    )
    .expect("render preview");
    assert_eq!((first.width, first.height), (32, 32));
    assert!(!first.cache_hit);
    let second = render_preview(
        cache.clone(),
        source.clone(),
        ROLE.to_owned(),
        32,
        32,
        false,
    )
    .expect("cached preview");
    assert!(second.cache_hit);
    assert_eq!(first.png_bytes, second.png_bytes);

    let missing = import_source(cache.clone(), format!("{source}.missing"), ROLE.to_owned());
    assert!(matches!(
        missing,
        Err(imaguncula_core::PreviewError::Source { .. })
    ));
    assert!(render_preview(cache, source, ROLE.to_owned(), 0, 32, false).is_err());
}

#[test]
fn preview_commands_report_an_unavailable_cache() {
    let app = mock_app();
    let cache = app.state::<Mutex<imaguncula_core::PreviewCache>>();
    poison(&cache);
    let source = path_string(&fixture_source());

    assert!(matches!(
        import_source(cache.clone(), source.clone(), ROLE.to_owned()),
        Err(imaguncula_core::PreviewError::CacheUnavailable)
    ));
    assert!(matches!(
        render_preview(cache, source, ROLE.to_owned(), 32, 32, false),
        Err(imaguncula_core::PreviewError::CacheUnavailable)
    ));
}

#[test]
fn start_export_emits_a_report_for_a_successful_export() {
    let app = mock_app();
    let directory = tempfile::tempdir().expect("temporary directory");
    let project = project_with_source();
    let destination =
        imaguncula_core::default_export_path(directory.path(), &project.base_filename);

    let finished = run_export(&app, project, path_string(directory.path()));
    assert!(finished["error"].is_null(), "unexpected error: {finished}");
    assert!(finished["report"]["destination"].is_string());
    assert!(destination.join("report.md").is_file());
}

#[test]
fn start_export_emits_a_structured_error_for_a_failed_export() {
    let app = mock_app();
    let directory = tempfile::tempdir().expect("temporary directory");

    let finished = run_export(
        &app,
        new_project("No sources".to_owned()),
        path_string(directory.path()),
    );
    assert!(finished["report"].is_null());
    assert!(
        finished["error"]["kind"].is_string(),
        "unexpected payload: {finished}"
    );
}

#[test]
fn only_one_export_runs_at_a_time_and_jobs_can_be_cancelled() {
    let app = mock_app();
    app.state::<ExportJobs>()
        .active
        .lock()
        .expect("jobs lock")
        .insert(
            "export-running".to_owned(),
            imaguncula_core::ExportCancellation::default(),
        );

    let second = start(&app, new_project("Second".to_owned()), String::new());
    assert!(matches!(second, Err(ExportCommandError::JobAlreadyRunning)));

    let jobs = || app.state::<ExportJobs>();
    assert!(cancel_export(jobs(), "export-running".to_owned()).expect("cancel running"));
    assert!(!cancel_export(jobs(), "export-unknown".to_owned()).expect("cancel unknown"));
}

#[test]
fn export_commands_report_unavailable_job_state() {
    let app = mock_app();
    poison(&app.state::<ExportJobs>().active);

    let started = start(&app, new_project("Poisoned".to_owned()), String::new());
    assert!(matches!(started, Err(ExportCommandError::JobsUnavailable)));
    assert!(matches!(
        cancel_export(app.state::<ExportJobs>(), "export-1".to_owned()),
        Err(ExportCommandError::JobsUnavailable)
    ));
}

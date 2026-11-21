// SPDX-License-Identifier: MIT OR Apache-2.0
use std::{collections::BTreeMap, fs, path::Path};

use imaguncula_cli::{
    CommonExportArgs, ExportArgs, QuickArgs, RoleSourceArg, export_quick, export_saved_project,
};
use imaguncula_core::{
    ExportCancellation, ProjectSource, ProjectState, default_export_path, export_project,
    import_source, save_project,
};
use tempfile::tempdir;

#[test]
fn cli_and_gui_core_path_produce_byte_equivalent_packages() {
    let directory = tempdir().expect("temporary directory");
    let source_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("fixtures/sources/asymmetric.svg");
    let source = import_source(&source_path, "master").expect("import source");
    let mut project = ProjectState::new("Equivalent package");
    project.sources.push(ProjectSource {
        role: "master".to_owned(),
        path: source.path,
        content_hash: source.content_hash,
    });
    let project_path = directory.path().join("equivalent.imaguncula.json");
    save_project(&project_path, &project).expect("save project");

    let cli_parent = directory.path().join("cli");
    let gui_parent = directory.path().join("gui");
    fs::create_dir(&gui_parent).expect("create GUI parent");
    let cli = export_saved_project(&ExportArgs {
        project: project_path,
        allow_external_sources: true,
        common: CommonExportArgs {
            output: cli_parent.clone(),
            overwrite: false,
            no_zip: false,
            json: true,
        },
    })
    .expect("CLI export");
    assert!(
        cli_parent.is_dir(),
        "CLI should create a missing output parent"
    );
    let gui_destination = default_export_path(&gui_parent, &project.base_filename);
    let gui = export_project(
        &project,
        &gui_destination,
        false,
        &ExportCancellation::default(),
        |_| {},
    )
    .expect("GUI-equivalent core export");

    assert_eq!(cli.report.report, gui.report);
    assert_eq!(collect_files(&cli_parent), collect_files(&gui_parent));
    assert!(
        cli.stdout()
            .expect("JSON output")
            .contains("\"reportMarkdown\"")
    );
}

#[test]
fn quick_mode_selects_nondefault_multi_role_preset() {
    let directory = tempdir().expect("temporary directory");
    let source_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("fixtures/sources/asymmetric.svg");
    let success = export_quick(QuickArgs {
        source: source_path.clone(),
        preset: "android-launcher".to_owned(),
        name: Some("Android quick".to_owned()),
        role_sources: vec![
            RoleSourceArg {
                role: "foreground".to_owned(),
                path: source_path.clone(),
            },
            RoleSourceArg {
                role: "background".to_owned(),
                path: source_path,
            },
        ],
        common: CommonExportArgs {
            output: directory.path().to_path_buf(),
            overwrite: false,
            no_zip: true,
            json: false,
        },
    })
    .expect("nondefault quick export");

    assert!(
        Path::new(&success.report.destination)
            .join("android-launcher/res/mipmap-anydpi-v26/ic_launcher.xml")
            .is_file()
    );
}

fn collect_files(root: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut files = BTreeMap::new();
    collect_directory(root, root, &mut files);
    files
}

fn collect_directory(root: &Path, directory: &Path, files: &mut BTreeMap<String, Vec<u8>>) {
    for entry in fs::read_dir(directory).expect("read export directory") {
        let entry = entry.expect("directory entry");
        if entry.file_type().expect("entry type").is_dir() {
            collect_directory(root, &entry.path(), files);
        } else {
            let path = entry.path();
            let relative = path
                .strip_prefix(root)
                .expect("relative export path")
                .to_string_lossy()
                .replace('\\', "/");
            files.insert(relative, fs::read(path).expect("read exported file"));
        }
    }
}

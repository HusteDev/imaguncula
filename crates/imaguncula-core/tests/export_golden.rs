// SPDX-License-Identifier: MIT OR Apache-2.0
use std::{collections::BTreeSet, fs, path::Path};

use imaguncula_core::{
    CustomOutput, ExportCancellation, OutputFormat, PreviewCache, ProjectSource, ProjectState,
    decode_png_rgba, export_project, import_source,
};
use serde::Deserialize;
use tempfile::tempdir;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExpectedGolden {
    files: Vec<ExpectedFile>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExpectedFile {
    output_id: String,
    path: String,
    width: u32,
    height: u32,
}

#[test]
fn custom_png_export_matches_semantic_golden_and_preview_pixels() {
    let source_path = fixture("sources/asymmetric.svg");
    let expected: ExpectedGolden = serde_json::from_slice(
        &fs::read(fixture("golden/custom-png/expected.json")).expect("read golden fixture"),
    )
    .expect("parse golden fixture");
    let source = import_source(&source_path, "master").expect("import golden source");
    let mut project = ProjectState::new("Custom PNG golden");
    project.selected_presets.clear();
    project.enabled_outputs.clear();
    project.sources.push(ProjectSource {
        role: "master".to_owned(),
        path: source.path,
        content_hash: source.content_hash,
    });
    for file in &expected.files {
        let filename = Path::new(&file.path)
            .file_name()
            .and_then(|name| name.to_str())
            .expect("golden filename");
        let destination = Path::new(&file.path)
            .parent()
            .and_then(Path::to_str)
            .unwrap_or_default();
        project.custom_outputs.push(CustomOutput {
            id: file.output_id.clone(),
            width: file.width,
            height: file.height,
            format: OutputFormat::Png,
            filename: filename.to_owned(),
            destination: destination.to_owned(),
            source_role: "master".to_owned(),
            usage: "Semantic golden output".to_owned(),
        });
        project.enabled_outputs.insert(file.output_id.clone());
    }

    let directory = tempdir().expect("temporary directory");
    let destination = directory.path().join("golden-export");
    let report = export_project(
        &project,
        &destination,
        false,
        &ExportCancellation::default(),
        |_| {},
    )
    .expect("export golden package");

    let actual_paths: BTreeSet<_> = report
        .report
        .files
        .iter()
        .map(|file| file.path.as_str())
        .collect();
    let expected_paths: BTreeSet<_> = expected
        .files
        .iter()
        .map(|file| file.path.as_str())
        .collect();
    assert_eq!(actual_paths, expected_paths);

    let mut previews = PreviewCache::default();
    for file in &expected.files {
        let exported = fs::read(destination.join(&file.path)).expect("read exported PNG");
        let decoded = decode_png_rgba(&exported).expect("decode exported PNG");
        let preview = previews
            .render(&source_path, "master", file.width, file.height, false)
            .expect("render matching preview");
        let preview_decoded = decode_png_rgba(&preview.png_bytes).expect("decode preview PNG");

        assert_eq!(
            (decoded.width(), decoded.height()),
            (file.width, file.height)
        );
        assert_eq!(decoded.pixels(), preview_decoded.pixels());
    }
}

fn fixture(relative: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("fixtures")
        .join(relative)
}

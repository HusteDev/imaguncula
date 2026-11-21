// SPDX-License-Identifier: MIT OR Apache-2.0
use std::path::PathBuf;

use imaguncula_cli::{Cli, CliError, Command, CommonExportArgs, ExportArgs, execute};

#[test]
fn checked_in_example_projects_export_through_the_cli_path() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let output = tempfile::tempdir().expect("temporary export parent");

    for filename in [
        "svg-windows-web.imaguncula.json",
        "png-windows.imaguncula.json",
        "android-multi-role.imaguncula.json",
    ] {
        let refused = execute(Cli {
            command: Command::Export(ExportArgs {
                project: root.join("examples").join(filename),
                allow_external_sources: false,
                common: CommonExportArgs {
                    output: output.path().to_path_buf(),
                    overwrite: false,
                    no_zip: false,
                    json: true,
                },
            }),
        })
        .expect_err("examples reference checked-in artwork outside their directory");
        assert!(matches!(refused, CliError::ExternalSources { .. }));

        let success = execute(Cli {
            command: Command::Export(ExportArgs {
                project: root.join("examples").join(filename),
                allow_external_sources: true,
                common: CommonExportArgs {
                    output: output.path().to_path_buf(),
                    overwrite: false,
                    no_zip: false,
                    json: true,
                },
            }),
        })
        .unwrap_or_else(|error| panic!("{filename} must export: {error}"));

        let destination = PathBuf::from(&success.report.destination);
        assert!(destination.join("report.md").is_file());
        assert!(
            destination
                .join(format!("{}-icons.zip", success.report.report.base_filename))
                .is_file()
        );
        assert!(!success.report.report.files.is_empty());
    }
}

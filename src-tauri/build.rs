// SPDX-License-Identifier: MIT OR Apache-2.0
use std::{env, path::PathBuf};

fn main() {
    let mut attributes = tauri_build::Attributes::new();
    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows")
        && env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc")
    {
        // Tauri needs the Common Controls v6 manifest on Windows, and
        // tauri-build embeds it only in the application executable. Without
        // it, test binaries that load Tauri fail at startup with
        // STATUS_ENTRYPOINT_NOT_FOUND. Embedding the same manifest through
        // the linker applies it to the executable and the test binaries alike.
        attributes = attributes
            .windows_attributes(tauri_build::WindowsAttributes::new_without_app_manifest());
        let manifest =
            PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("Cargo sets CARGO_MANIFEST_DIR"))
                .join("windows-app-manifest.xml");
        println!("cargo:rerun-if-changed={}", manifest.display());
        println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
        println!("cargo:rustc-link-arg=/MANIFESTINPUT:{}", manifest.display());
        println!("cargo:rustc-link-arg=/WX");
    }
    tauri_build::try_build(attributes).expect("tauri-build failed");
}

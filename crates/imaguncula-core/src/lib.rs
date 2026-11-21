// SPDX-License-Identifier: MIT OR Apache-2.0
#![doc = "Pure processing primitives for Imaguncula."]

pub mod container;
pub mod export;
pub mod output;
pub mod preset;
pub mod preview;
pub mod project;
pub mod render;
pub mod report;
pub mod source;

pub use container::{
    ContainerEncodeError, IconContainerEntry, IconContainerFormat, IconEntryEncoding,
    encode_icon_container,
};
pub use export::{
    DestinationInspection, DestinationState, ExportCancellation, ExportError, ExportProgress,
    ExportReport, default_export_path, export_project, inspect_export_destination,
};
pub use imaguncula_presets::{
    Platform as PresetPlatform, PresetDiff, PresetOutputRename, PresetSummary,
};
pub use output::{
    OutputIssue, OutputIssueSeverity, OutputPackageKind, OutputSpec, OutputStatus,
    RelativePathError, ResolveOutputsError, resolve_outputs, sanitize_relative_path,
};
pub use preset::{
    PresetQueryError, PresetUpgrade, PresetUpgradeError, apply_preset_upgrade,
    list_preset_upgrades, list_presets,
};
pub use preview::{
    PREVIEW_CACHE_MAX_BYTES, PREVIEW_CACHE_MAX_ENTRIES, PREVIEW_DIMENSION_MAX, PendingPreview,
    PreparedPreview, PreviewCache, PreviewError, PreviewImage,
};
pub use project::{
    AcceptedWarning, CustomOutput, ExportSettings, ExternalSourceReference, OutputFormat,
    OutputOverride, PresetMigrationAcceptance, ProjectFileError, ProjectLoad, ProjectSource,
    ProjectState, ProjectValidationCode, ProjectValidationIssue, SelectedPreset, load_project,
    save_project,
};
pub use render::{
    CanonicalRgba, RenderError, SVG_RENDER_TIMEOUT_SECONDS, decode_png_rgba, encode_png_srgb,
    encode_png_srgb_flattened, render_svg, render_svg_bounded, resize_raster,
};
pub use report::{GeneratedFile, GenerationReport, ReportWarning};
pub use source::{
    PNG_DECODE_MEMORY_MAX_BYTES, PNG_DIMENSION_MAX, PNG_PIXEL_MAX, PngSourceInfo,
    SOURCE_FILE_MAX_BYTES, SVG_ELEMENT_MAX, SVG_FILTER_ELEMENT_MAX, SVG_GEOMETRY_BYTES_MAX,
    SVG_NESTING_MAX, SourceDimensions, SourceError, SourceFormat, SourceInfo, SourceShape,
    SourceWarning, SvgInvalidReason, SvgSafetyViolation, SvgUnsupportedFeature, import_source,
    inspect_svg_bytes,
};

/// Human-readable application name shared by every interface over the core.
pub const APPLICATION_NAME: &str = "Imaguncula";

#[cfg(test)]
mod tests {
    use super::APPLICATION_NAME;

    #[test]
    fn application_name_is_stable() {
        assert_eq!(APPLICATION_NAME, "Imaguncula");
    }
}

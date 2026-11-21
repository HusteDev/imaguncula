// SPDX-License-Identifier: MIT OR Apache-2.0
//! Bounded source-file import and SVG safety inspection.

use std::{
    fmt,
    fs::File,
    io::{BufReader, Cursor, Read},
    path::Path,
    sync::Arc,
};

use quick_xml::{Reader, events::Event};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

use image::{ImageDecoder, ImageFormat, ImageReader, Limits};

use crate::project::is_valid_identifier;

/// Maximum compressed/source-file bytes accepted by the import boundary.
pub const SOURCE_FILE_MAX_BYTES: u64 = 64 * 1024 * 1024;

/// Maximum XML elements accepted before `usvg` parsing.
pub const SVG_ELEMENT_MAX: usize = 100_000;

/// Maximum XML nesting accepted before `usvg` parsing.
pub const SVG_NESTING_MAX: usize = 256;

/// Maximum filter elements and filter primitives accepted before `usvg` parsing.
///
/// Filter primitives multiply render cost per covered pixel, so their count is
/// bounded like element count and nesting rather than left to render time.
pub const SVG_FILTER_ELEMENT_MAX: usize = 64;

/// Maximum combined path and polygon geometry bytes accepted before `usvg` parsing.
///
/// Bounds the total `d` and `points` attribute data so one file cannot carry
/// hundreds of megabytes of segments into tessellation.
pub const SVG_GEOMETRY_BYTES_MAX: usize = 2 * 1024 * 1024;

/// Maximum decoded PNG pixels accepted by the import boundary.
pub const PNG_PIXEL_MAX: u64 = 64_000_000;

/// Maximum width or height accepted from a PNG source.
pub const PNG_DIMENSION_MAX: u32 = 16_384;

/// Maximum bytes allocated for one PNG decoder output buffer.
pub const PNG_DECODE_MEMORY_MAX_BYTES: u64 = 256 * 1024 * 1024;

const PNG_SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n";

/// Source formats supported by the processing pipeline.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SourceFormat {
    Png,
    Svg,
}

/// Geometric classification used by validation and UI summaries.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SourceShape {
    Landscape,
    Portrait,
    Square,
}

/// Intrinsic source dimensions.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SourceDimensions {
    pub width: f64,
    pub height: f64,
}

/// PNG-specific metadata established by a limit-enforced full decode.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PngSourceInfo {
    pub pixel_count: u64,
    pub bits_per_channel: u8,
    pub has_alpha_channel: bool,
    pub has_transparent_pixels: bool,
    pub has_icc_profile: bool,
}

/// Known static SVG constructs that the v1 renderer does not preserve.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SvgUnsupportedFeature {
    Animation,
    Audio,
    Canvas,
    Iframe,
    Link,
    Video,
}

/// Non-fatal source conditions that must be surfaced before export.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "code", rename_all = "kebab-case")]
pub enum SourceWarning {
    EmbeddedImageIgnored,
    ExternalImageIgnored,
    ExternalReferenceIgnored,
    ExternalStyleIgnored,
    FontDefinitionIgnored,
    IccProfileTreatedAsSrgb,
    NonSquareRaster { width: u32, height: u32 },
    NonSquareViewBox { width: f64, height: f64 },
    TextIgnored,
    UnsupportedSvgFeatureIgnored { feature: SvgUnsupportedFeature },
}

impl SourceWarning {
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::EmbeddedImageIgnored => "embedded-image-ignored",
            Self::ExternalImageIgnored => "external-image-ignored",
            Self::ExternalReferenceIgnored => "external-reference-ignored",
            Self::ExternalStyleIgnored => "external-style-ignored",
            Self::FontDefinitionIgnored => "font-definition-ignored",
            Self::IccProfileTreatedAsSrgb => "icc-profile-treated-as-srgb",
            Self::NonSquareRaster { .. } => "non-square-raster",
            Self::NonSquareViewBox { .. } => "non-square-view-box",
            Self::TextIgnored => "text-ignored",
            Self::UnsupportedSvgFeatureIgnored { .. } => "unsupported-svg-feature-ignored",
        }
    }
}

impl fmt::Display for SourceWarning {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmbeddedImageIgnored => formatter.write_str("embedded image ignored"),
            Self::ExternalImageIgnored => formatter.write_str("external image ignored"),
            Self::ExternalReferenceIgnored => formatter.write_str("external reference ignored"),
            Self::ExternalStyleIgnored => formatter.write_str("external stylesheet ignored"),
            Self::FontDefinitionIgnored => formatter.write_str("font definition ignored"),
            Self::IccProfileTreatedAsSrgb => {
                formatter.write_str("ICC profile detected; channel values treated as sRGB")
            }
            Self::NonSquareRaster { width, height } => {
                write!(formatter, "non-square raster source ({width}x{height})")
            }
            Self::NonSquareViewBox { width, height } => {
                write!(formatter, "non-square SVG view box ({width}x{height})")
            }
            Self::TextIgnored => formatter.write_str("SVG text ignored"),
            Self::UnsupportedSvgFeatureIgnored { feature } => {
                write!(formatter, "unsupported SVG {feature:?} feature ignored")
            }
        }
    }
}

/// Metadata returned after a source passes the import boundary.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SourceInfo {
    pub role: String,
    pub path: String,
    pub format: SourceFormat,
    pub byte_length: u64,
    pub content_hash: String,
    pub dimensions: SourceDimensions,
    pub shape: SourceShape,
    pub warnings: Vec<SourceWarning>,
    pub png: Option<PngSourceInfo>,
}

/// Stable reasons a PNG header cannot be used for dimension probing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PngInvalidReason {
    BadSignature,
    MissingHeader,
    InvalidHeaderChunk,
    ZeroDimension,
}

impl fmt::Display for PngInvalidReason {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::BadSignature => "bad PNG signature",
            Self::MissingHeader => "missing PNG header",
            Self::InvalidHeaderChunk => "invalid PNG IHDR chunk",
            Self::ZeroDimension => "zero PNG dimension",
        })
    }
}

/// Stable reasons syntactically SVG-like input cannot be normalized.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SvgInvalidReason {
    InvalidDimensions,
    MalformedXml,
    MissingSvgRoot,
    NotUtf8,
    RendererRejected,
}

impl fmt::Display for SvgInvalidReason {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidDimensions => "invalid SVG dimensions",
            Self::MalformedXml => "malformed SVG XML",
            Self::MissingSvgRoot => "missing SVG root element",
            Self::NotUtf8 => "SVG is not UTF-8",
            Self::RendererRejected => "SVG normalization failed",
        })
    }
}

/// Active or resource-amplifying SVG constructs rejected before normalization.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "code", rename_all = "kebab-case")]
pub enum SvgSafetyViolation {
    DocumentType,
    EventHandlerAttribute { attribute: String },
    ForeignObjectElement,
    MaximumElementCount { maximum: usize },
    MaximumFilterElements { maximum: usize },
    MaximumGeometryBytes { maximum: usize },
    MaximumNestingDepth { maximum: usize },
    ScriptElement,
    ScriptReference,
}

impl fmt::Display for SvgSafetyViolation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DocumentType => formatter.write_str("document type declarations are forbidden"),
            Self::EventHandlerAttribute { attribute } => {
                write!(
                    formatter,
                    "event handler attribute {attribute} is forbidden"
                )
            }
            Self::ForeignObjectElement => {
                formatter.write_str("foreignObject elements are forbidden")
            }
            Self::MaximumElementCount { maximum } => {
                write!(formatter, "SVG exceeds the {maximum}-element limit")
            }
            Self::MaximumFilterElements { maximum } => {
                write!(formatter, "SVG exceeds the {maximum}-filter-element limit")
            }
            Self::MaximumGeometryBytes { maximum } => {
                write!(
                    formatter,
                    "SVG exceeds the {maximum}-byte path geometry limit"
                )
            }
            Self::MaximumNestingDepth { maximum } => {
                write!(formatter, "SVG exceeds the {maximum}-level nesting limit")
            }
            Self::ScriptElement => formatter.write_str("script elements are forbidden"),
            Self::ScriptReference => formatter.write_str("script references are forbidden"),
        }
    }
}

/// Structured source-import failures exposed through interface adapters.
#[derive(Debug, Error, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum SourceError {
    #[error("source role is not a valid identifier")]
    InvalidRole { role: String },
    #[error("failed to open source file")]
    Open { path: String, message: String },
    #[error("failed to inspect source file")]
    Metadata { path: String, message: String },
    #[error("source path is not a regular file")]
    NotRegularFile { path: String },
    #[error("source file is empty")]
    EmptyFile { path: String },
    #[error("source file exceeds the {maximum_bytes}-byte limit")]
    FileTooLarge {
        path: String,
        maximum_bytes: u64,
        actual_bytes: u64,
    },
    #[error("failed to read source file")]
    Read { path: String, message: String },
    #[error("source format is not supported")]
    UnsupportedFormat { path: String },
    #[error("source file is not valid PNG: {reason}")]
    InvalidPng {
        path: String,
        reason: PngInvalidReason,
    },
    #[error("PNG exceeds the {maximum_pixels}-pixel limit")]
    PngPixelLimit {
        path: String,
        maximum_pixels: u64,
        actual_pixels: u64,
    },
    #[error("PNG dimension exceeds the {maximum_dimension}-pixel limit")]
    PngDimensionLimit {
        path: String,
        maximum_dimension: u32,
        width: u32,
        height: u32,
    },
    #[error("decoded PNG exceeds the {maximum_bytes}-byte memory limit")]
    PngDecodedMemoryLimit {
        path: String,
        maximum_bytes: u64,
        actual_bytes: u64,
    },
    #[error("failed to allocate the bounded PNG decode buffer")]
    PngAllocation { path: String, requested_bytes: u64 },
    #[error("PNG decoding failed")]
    PngDecode { path: String, message: String },
    #[error("source file contains unsafe SVG: {violation}")]
    UnsafeSvg {
        path: String,
        violation: SvgSafetyViolation,
    },
    #[error("source file is not valid SVG: {reason}")]
    InvalidSvg {
        path: String,
        reason: SvgInvalidReason,
        message: Option<String>,
    },
}

/// Imports one source without decoding raster pixels or accessing SVG-linked files.
///
/// Format classification is content-based. The complete file is read through a
/// hard byte limit, then hashed so every later cache and project reference can
/// use content identity rather than path identity.
///
/// # Errors
///
/// Returns a structured [`SourceError`] when the role or path is invalid, the
/// byte limit is exceeded, format probing fails, or SVG safety inspection finds
/// an active or resource-amplifying construct.
pub fn import_source(
    path: impl AsRef<Path>,
    role: impl Into<String>,
) -> Result<SourceInfo, SourceError> {
    Ok(load_source_data(path, role)?.info)
}

pub(crate) struct LoadedSource {
    pub info: SourceInfo,
    pub bytes: Vec<u8>,
    pub svg_tree: Option<Arc<usvg::Tree>>,
}

pub(crate) fn load_source_data(
    path: impl AsRef<Path>,
    role: impl Into<String>,
) -> Result<LoadedSource, SourceError> {
    inspect_source_data(read_source_data(path, role)?)
}

pub(crate) struct SourceData {
    pub role: String,
    pub path: String,
    pub bytes: Vec<u8>,
    pub byte_length: u64,
    pub content_hash: String,
}

pub(crate) fn read_source_data(
    path: impl AsRef<Path>,
    role: impl Into<String>,
) -> Result<SourceData, SourceError> {
    let path = path.as_ref();
    let path_text = path.to_string_lossy().into_owned();
    let role = role.into();

    if !is_valid_identifier(&role) {
        return Err(SourceError::InvalidRole { role });
    }

    let metadata = path.metadata().map_err(|error| SourceError::Metadata {
        path: path_text.clone(),
        message: error.to_string(),
    })?;

    if !metadata.is_file() {
        return Err(SourceError::NotRegularFile { path: path_text });
    }
    if metadata.len() == 0 {
        return Err(SourceError::EmptyFile { path: path_text });
    }
    if metadata.len() > SOURCE_FILE_MAX_BYTES {
        return Err(SourceError::FileTooLarge {
            path: path_text,
            maximum_bytes: SOURCE_FILE_MAX_BYTES,
            actual_bytes: metadata.len(),
        });
    }

    let file = File::open(path).map_err(|error| SourceError::Open {
        path: path_text.clone(),
        message: error.to_string(),
    })?;

    let mut bytes = Vec::with_capacity(usize::try_from(metadata.len()).unwrap_or_default());
    BufReader::new(file)
        .take(SOURCE_FILE_MAX_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| SourceError::Read {
            path: path_text.clone(),
            message: error.to_string(),
        })?;

    let byte_length = bytes.len() as u64;
    if byte_length == 0 {
        return Err(SourceError::EmptyFile { path: path_text });
    }
    if byte_length > SOURCE_FILE_MAX_BYTES {
        return Err(SourceError::FileTooLarge {
            path: path_text,
            maximum_bytes: SOURCE_FILE_MAX_BYTES,
            actual_bytes: byte_length,
        });
    }

    Ok(SourceData {
        role,
        path: path_text,
        byte_length,
        content_hash: sha256_hex(&bytes),
        bytes,
    })
}

pub(crate) fn inspect_source_data(source: SourceData) -> Result<LoadedSource, SourceError> {
    let SourceData {
        role,
        path,
        bytes,
        byte_length,
        content_hash,
    } = source;
    let (format, dimensions, warnings, png, svg_tree) = if bytes.starts_with(PNG_SIGNATURE) {
        let (dimensions, png, warnings) = probe_png(&bytes, &path)?;
        (SourceFormat::Png, dimensions, warnings, Some(png), None)
    } else if bytes.starts_with(b"\x89PNG") {
        return Err(SourceError::InvalidPng {
            path,
            reason: PngInvalidReason::BadSignature,
        });
    } else {
        let (tree, warnings) = sanitized_svg_tree(&bytes, &path)?;
        let size = tree.size();
        let dimensions = SourceDimensions {
            width: f64::from(size.width()),
            height: f64::from(size.height()),
        };
        (
            SourceFormat::Svg,
            dimensions,
            warnings,
            None,
            Some(Arc::new(tree)),
        )
    };

    Ok(LoadedSource {
        info: SourceInfo {
            role,
            path,
            format,
            byte_length,
            content_hash,
            dimensions,
            shape: classify_shape(dimensions),
            warnings,
            png,
        },
        bytes,
        svg_tree,
    })
}

fn probe_png(
    bytes: &[u8],
    path: &str,
) -> Result<(SourceDimensions, PngSourceInfo, Vec<SourceWarning>), SourceError> {
    let (width, height) = probe_png_header(bytes, path)?;
    let png = decode_png_metadata(bytes, path, width, height)?;
    let mut warnings = Vec::new();
    if width != height {
        warnings.push(SourceWarning::NonSquareRaster { width, height });
    }
    if png.has_icc_profile {
        warnings.push(SourceWarning::IccProfileTreatedAsSrgb);
    }

    Ok((
        SourceDimensions {
            width: f64::from(width),
            height: f64::from(height),
        },
        png,
        warnings,
    ))
}

pub(crate) fn probe_png_header(bytes: &[u8], path: &str) -> Result<(u32, u32), SourceError> {
    if bytes.len() < 33 {
        return Err(SourceError::InvalidPng {
            path: path.to_owned(),
            reason: PngInvalidReason::MissingHeader,
        });
    }
    if &bytes[8..12] != 13_u32.to_be_bytes().as_slice() || &bytes[12..16] != b"IHDR" {
        return Err(SourceError::InvalidPng {
            path: path.to_owned(),
            reason: PngInvalidReason::InvalidHeaderChunk,
        });
    }

    let width = u32::from_be_bytes(bytes[16..20].try_into().expect("fixed-width PNG field"));
    let height = u32::from_be_bytes(bytes[20..24].try_into().expect("fixed-width PNG field"));
    if width == 0 || height == 0 {
        return Err(SourceError::InvalidPng {
            path: path.to_owned(),
            reason: PngInvalidReason::ZeroDimension,
        });
    }

    if width > PNG_DIMENSION_MAX || height > PNG_DIMENSION_MAX {
        return Err(SourceError::PngDimensionLimit {
            path: path.to_owned(),
            maximum_dimension: PNG_DIMENSION_MAX,
            width,
            height,
        });
    }
    let pixel_count = u64::from(width) * u64::from(height);
    if pixel_count > PNG_PIXEL_MAX {
        return Err(SourceError::PngPixelLimit {
            path: path.to_owned(),
            maximum_pixels: PNG_PIXEL_MAX,
            actual_pixels: pixel_count,
        });
    }

    Ok((width, height))
}

fn decode_png_metadata(
    bytes: &[u8],
    path: &str,
    width: u32,
    height: u32,
) -> Result<PngSourceInfo, SourceError> {
    let mut limits = Limits::default();
    limits.max_image_width = Some(PNG_DIMENSION_MAX);
    limits.max_image_height = Some(PNG_DIMENSION_MAX);
    limits.max_alloc = Some(PNG_DECODE_MEMORY_MAX_BYTES);
    let mut reader = ImageReader::with_format(Cursor::new(bytes), ImageFormat::Png);
    reader.limits(limits);
    let mut decoder = reader
        .into_decoder()
        .map_err(|error| png_decode_error(path, &error))?;
    let (decoded_width, decoded_height) = decoder.dimensions();
    if (decoded_width, decoded_height) != (width, height) {
        return Err(SourceError::PngDecode {
            path: path.to_owned(),
            message: "decoded dimensions do not match the PNG header".to_owned(),
        });
    }

    let color_type = decoder.color_type();
    let has_icc_profile = decoder
        .icc_profile()
        .map_err(|error| png_decode_error(path, &error))?
        .is_some();
    let decoded_bytes = decoder.total_bytes();
    if decoded_bytes > PNG_DECODE_MEMORY_MAX_BYTES {
        return Err(SourceError::PngDecodedMemoryLimit {
            path: path.to_owned(),
            maximum_bytes: PNG_DECODE_MEMORY_MAX_BYTES,
            actual_bytes: decoded_bytes,
        });
    }
    let decoded_length =
        usize::try_from(decoded_bytes).map_err(|_| SourceError::PngAllocation {
            path: path.to_owned(),
            requested_bytes: decoded_bytes,
        })?;
    let mut pixel_bytes = Vec::new();
    pixel_bytes
        .try_reserve_exact(decoded_length)
        .map_err(|_| SourceError::PngAllocation {
            path: path.to_owned(),
            requested_bytes: decoded_bytes,
        })?;
    pixel_bytes.resize(decoded_length, 0);
    decoder
        .read_image(&mut pixel_bytes)
        .map_err(|error| png_decode_error(path, &error))?;

    let has_alpha_channel = color_type.has_alpha();
    let has_transparent_pixels =
        has_alpha_channel && contains_transparent_pixel(&pixel_bytes, color_type);
    let bits_per_channel =
        u8::try_from(color_type.bits_per_pixel() / u16::from(color_type.channel_count()))
            .expect("PNG channel depth fits in u8");
    Ok(PngSourceInfo {
        pixel_count: u64::from(width) * u64::from(height),
        bits_per_channel,
        has_alpha_channel,
        has_transparent_pixels,
        has_icc_profile,
    })
}

fn png_decode_error(path: &str, error: &image::ImageError) -> SourceError {
    SourceError::PngDecode {
        path: path.to_owned(),
        message: error.to_string(),
    }
}

fn contains_transparent_pixel(bytes: &[u8], color_type: image::ColorType) -> bool {
    match color_type {
        image::ColorType::La8 => bytes.chunks_exact(2).any(|pixel| pixel[1] < u8::MAX),
        image::ColorType::Rgba8 => bytes.chunks_exact(4).any(|pixel| pixel[3] < u8::MAX),
        image::ColorType::La16 => bytes
            .chunks_exact(4)
            .any(|pixel| u16::from_ne_bytes([pixel[2], pixel[3]]) < u16::MAX),
        image::ColorType::Rgba16 => bytes
            .chunks_exact(8)
            .any(|pixel| u16::from_ne_bytes([pixel[6], pixel[7]]) < u16::MAX),
        _ => false,
    }
}

/// Runs the bounded SVG safety scan and normalization pipeline directly on in-memory bytes.
///
/// This is the file-independent boundary used by fuzzing. Production file import applies the same
/// function after the regular-file and byte-length checks.
///
/// # Errors
///
/// Returns the same typed SVG and source-limit failures as file import.
pub fn inspect_svg_bytes(bytes: &[u8]) -> Result<Vec<SourceWarning>, SourceError> {
    let actual_bytes = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
    if actual_bytes > SOURCE_FILE_MAX_BYTES {
        return Err(SourceError::FileTooLarge {
            path: "<memory>".to_owned(),
            maximum_bytes: SOURCE_FILE_MAX_BYTES,
            actual_bytes,
        });
    }
    sanitized_svg_tree(bytes, "<memory>").map(|(_, warnings)| warnings)
}

pub(crate) fn sanitized_svg_tree(
    bytes: &[u8],
    path: &str,
) -> Result<(usvg::Tree, Vec<SourceWarning>), SourceError> {
    let text = std::str::from_utf8(bytes).map_err(|_| SourceError::InvalidSvg {
        path: path.to_owned(),
        reason: SvgInvalidReason::NotUtf8,
        message: None,
    })?;
    if !text
        .trim_start_matches(|character: char| character.is_whitespace() || character == '\u{feff}')
        .starts_with('<')
    {
        return Err(SourceError::UnsupportedFormat {
            path: path.to_owned(),
        });
    }

    let scan = scan_svg(text, path)?;
    let options = usvg::Options {
        image_href_resolver: usvg::ImageHrefResolver {
            resolve_data: Box::new(|_, _, _| None),
            resolve_string: Box::new(|_, _| None),
        },
        ..usvg::Options::default()
    };
    let tree = usvg::Tree::from_data(bytes, &options).map_err(|error| {
        let reason = match error {
            usvg::Error::NotAnUtf8Str => SvgInvalidReason::NotUtf8,
            usvg::Error::InvalidSize => SvgInvalidReason::InvalidDimensions,
            _ => SvgInvalidReason::RendererRejected,
        };
        SourceError::InvalidSvg {
            path: path.to_owned(),
            reason,
            message: Some(error.to_string()),
        }
    })?;
    Ok((tree, scan.warnings))
}

#[derive(Default)]
struct SvgScan {
    warnings: Vec<SourceWarning>,
    filter_elements: usize,
    geometry_bytes: usize,
}

fn scan_svg(text: &str, path: &str) -> Result<SvgScan, SourceError> {
    let mut reader = Reader::from_str(text);
    let mut scan = SvgScan::default();
    let mut root_seen = false;
    let mut element_count = 0_usize;
    let mut depth = 0_usize;
    let mut style_depth = None;

    loop {
        let event = reader
            .read_event()
            .map_err(|error| SourceError::InvalidSvg {
                path: path.to_owned(),
                reason: SvgInvalidReason::MalformedXml,
                message: Some(error.to_string()),
            })?;
        match event {
            Event::Start(element) => {
                element_count += 1;
                enforce_element_limit(element_count, path)?;
                depth += 1;
                enforce_nesting_limit(depth, path)?;
                let name = element_local_name(&element);
                inspect_root(&name, &mut root_seen, path)?;
                inspect_svg_element(&element, &name, &reader, &mut scan, path)?;
                if name == "style" {
                    style_depth = Some(depth);
                }
            }
            Event::Empty(element) => {
                element_count += 1;
                enforce_element_limit(element_count, path)?;
                let name = element_local_name(&element);
                inspect_root(&name, &mut root_seen, path)?;
                inspect_svg_element(&element, &name, &reader, &mut scan, path)?;
            }
            Event::End(element) => {
                if depth == 0 {
                    return invalid_svg(path, SvgInvalidReason::MalformedXml, None);
                }
                let name = lowercase_local_name(element.name().as_ref());
                if name == "style" && style_depth == Some(depth) {
                    style_depth = None;
                }
                depth -= 1;
            }
            Event::Text(text) if style_depth.is_some() => {
                let css = text.decode().map_err(|error| SourceError::InvalidSvg {
                    path: path.to_owned(),
                    reason: SvgInvalidReason::MalformedXml,
                    message: Some(error.to_string()),
                })?;
                inspect_css(&css, &mut scan.warnings, path)?;
            }
            Event::CData(text) if style_depth.is_some() => {
                let css = text.decode().map_err(|error| SourceError::InvalidSvg {
                    path: path.to_owned(),
                    reason: SvgInvalidReason::MalformedXml,
                    message: Some(error.to_string()),
                })?;
                inspect_css(&css, &mut scan.warnings, path)?;
            }
            Event::DocType(_) => {
                return Err(SourceError::UnsafeSvg {
                    path: path.to_owned(),
                    violation: SvgSafetyViolation::DocumentType,
                });
            }
            Event::PI(instruction) => {
                let value = String::from_utf8_lossy(instruction.as_ref()).to_ascii_lowercase();
                if value.trim_start().starts_with("xml-stylesheet") {
                    push_warning(&mut scan.warnings, SourceWarning::ExternalStyleIgnored);
                }
            }
            Event::Eof => break,
            Event::Text(_)
            | Event::CData(_)
            | Event::Comment(_)
            | Event::Decl(_)
            | Event::GeneralRef(_) => {}
        }
    }

    if !root_seen {
        return invalid_svg(path, SvgInvalidReason::MissingSvgRoot, None);
    }
    if depth != 0 {
        return invalid_svg(path, SvgInvalidReason::MalformedXml, None);
    }

    Ok(scan)
}

fn inspect_root(name: &str, root_seen: &mut bool, path: &str) -> Result<(), SourceError> {
    if *root_seen {
        return Ok(());
    }
    if name != "svg" {
        return Err(SourceError::UnsupportedFormat {
            path: path.to_owned(),
        });
    }
    *root_seen = true;
    Ok(())
}

fn inspect_svg_element(
    element: &quick_xml::events::BytesStart<'_>,
    name: &str,
    reader: &Reader<&[u8]>,
    scan: &mut SvgScan,
    path: &str,
) -> Result<(), SourceError> {
    match name {
        "script" => {
            return Err(SourceError::UnsafeSvg {
                path: path.to_owned(),
                violation: SvgSafetyViolation::ScriptElement,
            });
        }
        "foreignobject" => {
            return Err(SourceError::UnsafeSvg {
                path: path.to_owned(),
                violation: SvgSafetyViolation::ForeignObjectElement,
            });
        }
        "text" | "tspan" | "textpath" => {
            push_warning(&mut scan.warnings, SourceWarning::TextIgnored);
        }
        _ => {
            if let Some(feature) = unsupported_feature(name) {
                push_warning(
                    &mut scan.warnings,
                    SourceWarning::UnsupportedSvgFeatureIgnored { feature },
                );
            }
        }
    }

    if name == "filter" || name.starts_with("fe") {
        scan.filter_elements += 1;
        if scan.filter_elements > SVG_FILTER_ELEMENT_MAX {
            return Err(SourceError::UnsafeSvg {
                path: path.to_owned(),
                violation: SvgSafetyViolation::MaximumFilterElements {
                    maximum: SVG_FILTER_ELEMENT_MAX,
                },
            });
        }
    }

    for attribute in element.attributes().with_checks(true) {
        let attribute = attribute.map_err(|error| SourceError::InvalidSvg {
            path: path.to_owned(),
            reason: SvgInvalidReason::MalformedXml,
            message: Some(error.to_string()),
        })?;
        let attribute_name = lowercase_local_name(attribute.key.as_ref());
        if is_event_handler_attribute(&attribute_name) {
            return Err(SourceError::UnsafeSvg {
                path: path.to_owned(),
                violation: SvgSafetyViolation::EventHandlerAttribute {
                    attribute: attribute_name,
                },
            });
        }

        let value = attribute
            .decoded_and_normalized_value(quick_xml::XmlVersion::Implicit1_0, reader.decoder())
            .map_err(|error| SourceError::InvalidSvg {
                path: path.to_owned(),
                reason: SvgInvalidReason::MalformedXml,
                message: Some(error.to_string()),
            })?;
        if matches!(attribute_name.as_str(), "d" | "points") {
            scan.geometry_bytes = scan.geometry_bytes.saturating_add(value.len());
            if scan.geometry_bytes > SVG_GEOMETRY_BYTES_MAX {
                return Err(SourceError::UnsafeSvg {
                    path: path.to_owned(),
                    violation: SvgSafetyViolation::MaximumGeometryBytes {
                        maximum: SVG_GEOMETRY_BYTES_MAX,
                    },
                });
            }
        }
        if matches!(attribute_name.as_str(), "href" | "src") {
            inspect_reference(name, &value, &mut scan.warnings, path)?;
        }
        if attribute_name == "style" {
            inspect_css(&value, &mut scan.warnings, path)?;
        }
        if name == "svg" && attribute_name == "viewbox" {
            inspect_view_box(&value, &mut scan.warnings);
        }
    }

    Ok(())
}

fn inspect_reference(
    element_name: &str,
    reference: &str,
    warnings: &mut Vec<SourceWarning>,
    path: &str,
) -> Result<(), SourceError> {
    let reference = reference.trim();
    if is_script_reference(reference) {
        return Err(SourceError::UnsafeSvg {
            path: path.to_owned(),
            violation: SvgSafetyViolation::ScriptReference,
        });
    }
    if element_name == "image" && reference.to_ascii_lowercase().starts_with("data:") {
        push_warning(warnings, SourceWarning::EmbeddedImageIgnored);
    } else if is_external_reference(reference) {
        let warning = if element_name == "image" {
            SourceWarning::ExternalImageIgnored
        } else if element_name == "style" || element_name == "link" {
            SourceWarning::ExternalStyleIgnored
        } else {
            SourceWarning::ExternalReferenceIgnored
        };
        push_warning(warnings, warning);
    }
    Ok(())
}

fn inspect_css(
    css: &str,
    warnings: &mut Vec<SourceWarning>,
    path: &str,
) -> Result<(), SourceError> {
    let lowercase = css.to_ascii_lowercase();
    if lowercase.contains("@font-face") {
        push_warning(warnings, SourceWarning::FontDefinitionIgnored);
    }
    if lowercase.contains("@import") {
        push_warning(warnings, SourceWarning::ExternalStyleIgnored);
    }

    let mut remainder = lowercase.as_str();
    while let Some(start) = remainder.find("url(") {
        remainder = &remainder[start + 4..];
        let Some(end) = remainder.find(')') else {
            break;
        };
        let reference = remainder[..end]
            .trim()
            .trim_matches(|character| character == '\'' || character == '"');
        if is_script_reference(reference) {
            return Err(SourceError::UnsafeSvg {
                path: path.to_owned(),
                violation: SvgSafetyViolation::ScriptReference,
            });
        }
        if is_external_reference(reference) {
            push_warning(warnings, SourceWarning::ExternalReferenceIgnored);
        }
        remainder = &remainder[end + 1..];
    }
    Ok(())
}

fn inspect_view_box(value: &str, warnings: &mut Vec<SourceWarning>) {
    let values = value
        .split(|character: char| character.is_ascii_whitespace() || character == ',')
        .filter(|part| !part.is_empty())
        .filter_map(|part| part.parse::<f64>().ok())
        .collect::<Vec<_>>();
    if let [_, _, width, height] = values.as_slice()
        && width.is_finite()
        && height.is_finite()
        && *width > 0.0
        && *height > 0.0
        && !dimensions_are_square(*width, *height)
    {
        push_warning(
            warnings,
            SourceWarning::NonSquareViewBox {
                width: *width,
                height: *height,
            },
        );
    }
}

fn unsupported_feature(name: &str) -> Option<SvgUnsupportedFeature> {
    match name {
        "animate" | "animatemotion" | "animatetransform" | "set" => {
            Some(SvgUnsupportedFeature::Animation)
        }
        "audio" => Some(SvgUnsupportedFeature::Audio),
        "canvas" => Some(SvgUnsupportedFeature::Canvas),
        "iframe" => Some(SvgUnsupportedFeature::Iframe),
        "link" => Some(SvgUnsupportedFeature::Link),
        "video" => Some(SvgUnsupportedFeature::Video),
        _ => None,
    }
}

fn enforce_element_limit(count: usize, path: &str) -> Result<(), SourceError> {
    if count > SVG_ELEMENT_MAX {
        return Err(SourceError::UnsafeSvg {
            path: path.to_owned(),
            violation: SvgSafetyViolation::MaximumElementCount {
                maximum: SVG_ELEMENT_MAX,
            },
        });
    }
    Ok(())
}

fn enforce_nesting_limit(depth: usize, path: &str) -> Result<(), SourceError> {
    if depth > SVG_NESTING_MAX {
        return Err(SourceError::UnsafeSvg {
            path: path.to_owned(),
            violation: SvgSafetyViolation::MaximumNestingDepth {
                maximum: SVG_NESTING_MAX,
            },
        });
    }
    Ok(())
}

fn invalid_svg<T>(
    path: &str,
    reason: SvgInvalidReason,
    message: Option<String>,
) -> Result<T, SourceError> {
    Err(SourceError::InvalidSvg {
        path: path.to_owned(),
        reason,
        message,
    })
}

fn element_local_name(element: &quick_xml::events::BytesStart<'_>) -> String {
    lowercase_local_name(element.name().as_ref())
}

fn lowercase_local_name(name: &[u8]) -> String {
    let local = name.rsplit(|byte| *byte == b':').next().unwrap_or(name);
    String::from_utf8_lossy(local).to_ascii_lowercase()
}

fn is_event_handler_attribute(name: &str) -> bool {
    name.strip_prefix("on").is_some_and(|event_name| {
        !event_name.is_empty() && event_name.bytes().all(|byte| byte.is_ascii_alphanumeric())
    })
}

fn is_script_reference(reference: &str) -> bool {
    let lowercase = reference.trim_start().to_ascii_lowercase();
    lowercase.starts_with("javascript:") || lowercase.starts_with("vbscript:")
}

fn is_external_reference(reference: &str) -> bool {
    let reference = reference.trim();
    !reference.is_empty()
        && !reference.starts_with('#')
        && !reference.to_ascii_lowercase().starts_with("data:")
}

fn push_warning(warnings: &mut Vec<SourceWarning>, warning: SourceWarning) {
    if !warnings.contains(&warning) {
        warnings.push(warning);
    }
}

fn classify_shape(dimensions: SourceDimensions) -> SourceShape {
    if dimensions_are_square(dimensions.width, dimensions.height) {
        SourceShape::Square
    } else if dimensions.width > dimensions.height {
        SourceShape::Landscape
    } else {
        SourceShape::Portrait
    }
}

fn dimensions_are_square(width: f64, height: f64) -> bool {
    let scale = width.abs().max(height.abs()).max(1.0);
    (width - height).abs() <= scale * 1.0e-9
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        fs::OpenOptions,
        time::{Duration, Instant},
    };

    use image::{ImageFormat, Rgba, RgbaImage};
    use tempfile::tempdir;

    use super::*;

    fn write_svg(contents: &str) -> (tempfile::TempDir, std::path::PathBuf) {
        let directory = tempdir().expect("temporary directory");
        let path = directory.path().join("source.svg");
        fs::write(&path, contents).expect("write SVG fixture");
        (directory, path)
    }

    #[test]
    fn imports_png_by_content_and_reports_dimensions() {
        let directory = tempdir().expect("temporary directory");
        let path = directory.path().join("not-an-svg.svg");
        let image = RgbaImage::from_pixel(7, 5, Rgba([1, 2, 3, 4]));
        image
            .save_with_format(&path, ImageFormat::Png)
            .expect("write PNG fixture");

        let info = import_source(&path, "master").expect("import PNG");

        assert_eq!(info.format, SourceFormat::Png);
        assert_eq!(
            info.dimensions,
            SourceDimensions {
                width: 7.0,
                height: 5.0
            }
        );
        assert_eq!(info.shape, SourceShape::Landscape);
        assert_eq!(info.content_hash.len(), 64);
        assert_eq!(
            info.warnings,
            vec![SourceWarning::NonSquareRaster {
                width: 7,
                height: 5
            }]
        );
        assert_eq!(
            info.png,
            Some(PngSourceInfo {
                pixel_count: 35,
                bits_per_channel: 8,
                has_alpha_channel: true,
                has_transparent_pixels: true,
                has_icc_profile: false,
            })
        );
    }

    #[test]
    fn imports_one_pixel_png_with_transparency_intact() {
        let directory = tempdir().expect("temporary directory");
        let path = directory.path().join("one-pixel.png");
        RgbaImage::from_pixel(1, 1, Rgba([7, 11, 13, 17]))
            .save_with_format(&path, ImageFormat::Png)
            .expect("write one-pixel PNG fixture");

        let info = import_source(&path, "master").expect("import one-pixel PNG");

        assert!((info.dimensions.width - 1.0).abs() < f64::EPSILON);
        assert!((info.dimensions.height - 1.0).abs() < f64::EPSILON);
        assert_eq!(info.shape, SourceShape::Square);
        assert!(info.png.expect("PNG metadata").has_transparent_pixels);
    }

    #[test]
    fn imports_svg_with_content_hash_and_non_square_view_box_warning() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="48" height="32" viewBox="0 0 24 16"><rect width="24" height="16"/></svg>"#;
        let (_directory, path) = write_svg(svg);

        let info = import_source(&path, "master").expect("import SVG");

        assert_eq!(info.format, SourceFormat::Svg);
        assert_eq!(info.shape, SourceShape::Landscape);
        assert_eq!(info.content_hash, sha256_hex(svg.as_bytes()));
        assert!(info.warnings.contains(&SourceWarning::NonSquareViewBox {
            width: 24.0,
            height: 16.0,
        }));
    }

    #[test]
    fn ignores_external_images_without_opening_them() {
        let (_directory, path) = write_svg(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32"><image href="missing-secret.png" width="32" height="32"/></svg>"#,
        );

        let info = import_source(&path, "master").expect("import sanitized SVG");

        assert_eq!(info.warnings, vec![SourceWarning::ExternalImageIgnored]);
    }

    #[test]
    fn ignores_embedded_images_before_any_raster_decode() {
        let (_directory, path) = write_svg(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32"><image href="data:image/png;base64,iVBORw0KGgo=" width="32" height="32"/></svg>"#,
        );

        let info = import_source(&path, "master").expect("import sanitized SVG");

        assert_eq!(info.warnings, vec![SourceWarning::EmbeddedImageIgnored]);
    }

    #[test]
    fn reports_deterministic_text_and_unsupported_feature_warnings() {
        let (_directory, path) = write_svg(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32"><text x="1" y="12">A</text><animate attributeName="x"/></svg>"#,
        );

        let info = import_source(&path, "master").expect("import static subset");

        assert!(info.warnings.contains(&SourceWarning::TextIgnored));
        assert!(
            info.warnings
                .contains(&SourceWarning::UnsupportedSvgFeatureIgnored {
                    feature: SvgUnsupportedFeature::Animation,
                })
        );
    }

    #[test]
    fn permits_metadata_attributes_that_only_begin_with_on() {
        let (_directory, path) = write_svg(
            r##"<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" only_selected="true"><rect width="32" height="32" fill="#123456"/></svg>"##,
        );

        let info = import_source(&path, "master").expect("import SVG metadata attribute");

        assert_eq!(info.format, SourceFormat::Svg);
    }

    #[test]
    fn event_handler_detection_remains_fail_closed_for_identifier_names() {
        for attribute in ["onclick", "onload", "onpointerdown", "oncustom", "ONLOAD"] {
            assert!(is_event_handler_attribute(&attribute.to_ascii_lowercase()));
        }
        for attribute in ["on", "only_selected", "on-custom", "on:custom", "fill"] {
            assert!(!is_event_handler_attribute(attribute));
        }
    }

    #[test]
    fn rejects_active_svg_content_with_specific_violations() {
        let cases = [
            (
                include_str!("../../../fixtures/hostile/script.svg"),
                SvgSafetyViolation::ScriptElement,
            ),
            (
                include_str!("../../../fixtures/hostile/event-handler.svg"),
                SvgSafetyViolation::EventHandlerAttribute {
                    attribute: "onload".to_owned(),
                },
            ),
            (
                include_str!("../../../fixtures/hostile/foreign-object.svg"),
                SvgSafetyViolation::ForeignObjectElement,
            ),
            (
                include_str!("../../../fixtures/hostile/external-entity.svg"),
                SvgSafetyViolation::DocumentType,
            ),
            (
                r#"<svg xmlns="http://www.w3.org/2000/svg"><a href="javascript:run()"/></svg>"#,
                SvgSafetyViolation::ScriptReference,
            ),
        ];

        for (svg, expected) in cases {
            let (_directory, path) = write_svg(svg);
            let error = import_source(&path, "master").expect_err("reject active content");
            assert!(matches!(
                error,
                SourceError::UnsafeSvg { violation, .. } if violation == expected
            ));
        }
    }

    #[test]
    fn hostile_corpus_covers_malformed_and_resource_bearing_css() {
        assert!(matches!(
            inspect_svg_bytes(include_bytes!("../../../fixtures/hostile/malformed.svg")),
            Err(SourceError::InvalidSvg { .. })
        ));
        let warnings = inspect_svg_bytes(include_bytes!("../../../fixtures/hostile/css-url.svg"))
            .expect("external CSS reference is stripped without a fetch");
        assert!(warnings.contains(&SourceWarning::ExternalReferenceIgnored));
    }

    #[test]
    fn rejects_excessive_svg_nesting_before_normalization() {
        let mut svg =
            String::from(r#"<svg xmlns="http://www.w3.org/2000/svg" width="1" height="1">"#);
        svg.push_str(&"<g>".repeat(SVG_NESTING_MAX));
        svg.push_str(&"</g>".repeat(SVG_NESTING_MAX));
        svg.push_str("</svg>");
        let (_directory, path) = write_svg(&svg);

        let error = import_source(&path, "master").expect_err("reject deep SVG");

        assert!(matches!(
            error,
            SourceError::UnsafeSvg {
                violation: SvgSafetyViolation::MaximumNestingDepth { maximum },
                ..
            } if maximum == SVG_NESTING_MAX
        ));
    }

    #[test]
    fn element_count_stress_stops_at_the_configured_boundary() {
        let mut svg =
            String::from(r#"<svg xmlns="http://www.w3.org/2000/svg" width="1" height="1">"#);
        svg.push_str(&"<path/>".repeat(SVG_ELEMENT_MAX));
        svg.push_str("</svg>");
        let started = Instant::now();
        let error =
            inspect_svg_bytes(svg.as_bytes()).expect_err("reject element-count stress input");
        assert!(matches!(
            error,
            SourceError::UnsafeSvg {
                violation: SvgSafetyViolation::MaximumElementCount { maximum },
                ..
            } if maximum == SVG_ELEMENT_MAX
        ));
        assert!(
            started.elapsed() < Duration::from_secs(15),
            "bounded pre-scan exceeded the stress-test ceiling"
        );
    }

    #[test]
    fn rejects_filter_element_bombs_before_normalization() {
        let mut svg =
            String::from(r#"<svg xmlns="http://www.w3.org/2000/svg" width="1" height="1">"#);
        svg.push_str("<filter id=\"f\">");
        svg.push_str(&"<feGaussianBlur stdDeviation=\"8\"/>".repeat(SVG_FILTER_ELEMENT_MAX));
        svg.push_str("</filter></svg>");

        let error = inspect_svg_bytes(svg.as_bytes()).expect_err("reject filter bomb");

        assert!(matches!(
            error,
            SourceError::UnsafeSvg {
                violation: SvgSafetyViolation::MaximumFilterElements { maximum },
                ..
            } if maximum == SVG_FILTER_ELEMENT_MAX
        ));
    }

    #[test]
    fn rejects_path_geometry_bombs_before_normalization() {
        let mut svg =
            String::from(r#"<svg xmlns="http://www.w3.org/2000/svg" width="1" height="1">"#);
        svg.push_str("<path d=\"M0 0");
        svg.push_str(&"L1 1L0 0".repeat(SVG_GEOMETRY_BYTES_MAX / 8));
        svg.push_str("Z\"/></svg>");

        let error = inspect_svg_bytes(svg.as_bytes()).expect_err("reject geometry bomb");

        assert!(matches!(
            error,
            SourceError::UnsafeSvg {
                violation: SvgSafetyViolation::MaximumGeometryBytes { maximum },
                ..
            } if maximum == SVG_GEOMETRY_BYTES_MAX
        ));
    }

    #[test]
    fn rejects_invalid_role_unknown_format_and_directory() {
        let directory = tempdir().expect("temporary directory");
        let path = directory.path().join("source.bin");
        fs::write(&path, b"plain text").expect("write fixture");

        assert!(matches!(
            import_source(&path, "Master Source"),
            Err(SourceError::InvalidRole { .. })
        ));
        assert!(matches!(
            import_source(&path, "master"),
            Err(SourceError::UnsupportedFormat { .. })
        ));
        assert!(matches!(
            import_source(directory.path(), "master"),
            Err(SourceError::NotRegularFile { .. })
        ));
    }

    #[test]
    fn rejects_files_over_the_byte_limit_without_reading_them() {
        let directory = tempdir().expect("temporary directory");
        let path = directory.path().join("oversized.png");
        let file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)
            .expect("create sparse fixture");
        file.set_len(SOURCE_FILE_MAX_BYTES + 1)
            .expect("extend sparse fixture");

        let error = import_source(&path, "master").expect_err("reject oversized source");

        assert!(matches!(
            error,
            SourceError::FileTooLarge {
                maximum_bytes: SOURCE_FILE_MAX_BYTES,
                actual_bytes,
                ..
            } if actual_bytes == SOURCE_FILE_MAX_BYTES + 1
        ));
    }

    #[test]
    fn rejects_truncated_png_and_hashes_known_bytes() {
        let directory = tempdir().expect("temporary directory");
        let path = directory.path().join("truncated.png");
        fs::write(&path, PNG_SIGNATURE).expect("write fixture");

        assert!(matches!(
            import_source(&path, "master"),
            Err(SourceError::InvalidPng {
                reason: PngInvalidReason::MissingHeader,
                ..
            })
        ));
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn rejects_png_dimensions_and_pixel_counts_before_decompression() {
        let directory = tempdir().expect("temporary directory");
        let path = directory.path().join("bomb.png");
        let image = RgbaImage::from_pixel(1, 1, Rgba([1, 2, 3, 255]));
        image
            .save_with_format(&path, ImageFormat::Png)
            .expect("write PNG fixture");
        let mut bytes = fs::read(&path).expect("read PNG fixture");

        bytes[16..20].copy_from_slice(&16_000_u32.to_be_bytes());
        bytes[20..24].copy_from_slice(&16_000_u32.to_be_bytes());
        fs::write(&path, &bytes).expect("write pixel bomb fixture");
        assert!(matches!(
            import_source(&path, "master"),
            Err(SourceError::PngPixelLimit {
                maximum_pixels: PNG_PIXEL_MAX,
                actual_pixels: 256_000_000,
                ..
            })
        ));

        bytes[16..20].copy_from_slice(&20_000_u32.to_be_bytes());
        bytes[20..24].copy_from_slice(&1_u32.to_be_bytes());
        fs::write(&path, &bytes).expect("write dimension bomb fixture");
        assert!(matches!(
            import_source(&path, "master"),
            Err(SourceError::PngDimensionLimit {
                maximum_dimension: PNG_DIMENSION_MAX,
                width: 20_000,
                height: 1,
                ..
            })
        ));
    }

    #[test]
    fn detects_icc_profile_and_reports_v1_color_policy() {
        let directory = tempdir().expect("temporary directory");
        let path = directory.path().join("profiled.png");
        let mut info = png::Info::with_size(1, 1);
        info.color_type = png::ColorType::Rgba;
        info.bit_depth = png::BitDepth::Eight;
        info.icc_profile = Some(vec![0x42; 128].into());
        let file = fs::File::create(&path).expect("create PNG fixture");
        let mut writer = png::Encoder::with_info(file, info)
            .expect("configure PNG encoder")
            .write_header()
            .expect("write PNG header");
        writer
            .write_image_data(&[1, 2, 3, 255])
            .expect("write PNG pixels");
        drop(writer);

        let source = import_source(&path, "master").expect("import profiled PNG");

        assert!(source.png.expect("PNG metadata").has_icc_profile);
        assert_eq!(
            source.warnings,
            vec![SourceWarning::IccProfileTreatedAsSrgb]
        );
    }
}

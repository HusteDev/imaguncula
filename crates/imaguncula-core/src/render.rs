// SPDX-License-Identifier: MIT OR Apache-2.0
//! Canonical straight-alpha RGBA rendering and resizing.

use std::io::Cursor;

use fast_image_resize::{
    FilterType, PixelType, ResizeAlg, ResizeOptions, Resizer,
    images::{Image as ResizeImage, ImageRef},
};
use image::{ImageFormat, ImageReader, Limits};
use resvg::tiny_skia::{Pixmap, Transform};
use serde::Serialize;
use thiserror::Error;

use crate::source::{
    PNG_DECODE_MEMORY_MAX_BYTES, PNG_DIMENSION_MAX, PNG_PIXEL_MAX, probe_png_header,
};

/// Straight-alpha, 8-bit RGBA pixels shared by every renderer and encoder.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanonicalRgba {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
}

impl CanonicalRgba {
    /// Validates dimensions and exact RGBA buffer length.
    ///
    /// # Errors
    ///
    /// Returns [`RenderError`] for zero or excessive dimensions, excessive
    /// pixel count, or a buffer whose length is not exactly `width * height * 4`.
    pub fn new(width: u32, height: u32, pixels: Vec<u8>) -> Result<Self, RenderError> {
        let expected = validate_dimensions(width, height)?;
        if pixels.len() != expected {
            return Err(RenderError::InvalidBufferLength {
                expected_bytes: expected,
                actual_bytes: pixels.len(),
            });
        }
        Ok(Self {
            width,
            height,
            pixels,
        })
    }

    #[must_use]
    pub const fn width(&self) -> u32 {
        self.width
    }

    #[must_use]
    pub const fn height(&self) -> u32 {
        self.height
    }

    #[must_use]
    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    #[must_use]
    pub fn into_pixels(self) -> Vec<u8> {
        self.pixels
    }
}

/// Typed rendering and canonical-image failures.
#[derive(Debug, Error, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum RenderError {
    #[error("render dimensions must be non-zero")]
    InvalidDimensions { width: u32, height: u32 },
    #[error("render dimension exceeds the {maximum_dimension}-pixel limit")]
    DimensionLimit {
        maximum_dimension: u32,
        width: u32,
        height: u32,
    },
    #[error("render exceeds the {maximum_pixels}-pixel limit")]
    PixelLimit {
        maximum_pixels: u64,
        actual_pixels: u64,
    },
    #[error("RGBA buffer length does not match its dimensions")]
    InvalidBufferLength {
        expected_bytes: usize,
        actual_bytes: usize,
    },
    #[error("failed to allocate a bounded RGBA buffer")]
    Allocation { requested_bytes: u64 },
    #[error("SVG pixmap allocation failed")]
    SvgPixmapAllocation { width: u32, height: u32 },
    #[error("PNG decoding failed")]
    PngDecode { message: String },
    #[error("PNG encoding failed")]
    PngEncode { message: String },
    #[error("raster upscaling requires an explicit override")]
    UpscaleForbidden {
        source_width: u32,
        source_height: u32,
        target_width: u32,
        target_height: u32,
    },
    #[error("raster resize failed")]
    ResizeFailed { message: String },
    #[error("SVG rendering did not finish within the {seconds}-second limit")]
    Timeout { seconds: u64 },
}

/// Decodes a bounded PNG into canonical straight-alpha RGBA8 pixels.
///
/// # Errors
///
/// Returns [`RenderError`] if the PNG is malformed, violates dimension,
/// pixel, or memory limits, or cannot be converted to RGBA8.
pub fn decode_png_rgba(bytes: &[u8]) -> Result<CanonicalRgba, RenderError> {
    let (width, height) =
        probe_png_header(bytes, "<memory>").map_err(|error| RenderError::PngDecode {
            message: error.to_string(),
        })?;
    validate_dimensions(width, height)?;

    let mut limits = Limits::default();
    limits.max_image_width = Some(PNG_DIMENSION_MAX);
    limits.max_image_height = Some(PNG_DIMENSION_MAX);
    limits.max_alloc = Some(PNG_DECODE_MEMORY_MAX_BYTES);
    let mut reader = ImageReader::with_format(Cursor::new(bytes), ImageFormat::Png);
    reader.limits(limits);
    let decoded = reader.decode().map_err(|error| RenderError::PngDecode {
        message: error.to_string(),
    })?;
    let rgba = decoded.into_rgba8();
    if rgba.dimensions() != (width, height) {
        return Err(RenderError::PngDecode {
            message: "decoded dimensions do not match the PNG header".to_owned(),
        });
    }

    CanonicalRgba::new(width, height, rgba.into_raw())
}

/// Renders one normalized SVG tree directly at the requested output size.
///
/// The result is demultiplied before entering the canonical buffer; tiny-skia's
/// internal premultiplied bytes must not leak into PNG encoders or resizers.
///
/// # Errors
///
/// Returns [`RenderError`] if target dimensions violate limits or the pixmap
/// allocation fails.
pub fn render_svg(
    tree: &usvg::Tree,
    width: u32,
    height: u32,
) -> Result<CanonicalRgba, RenderError> {
    validate_dimensions(width, height)?;
    let mut pixmap =
        Pixmap::new(width, height).ok_or(RenderError::SvgPixmapAllocation { width, height })?;
    let source_size = tree.size();
    #[expect(
        clippy::cast_precision_loss,
        reason = "target dimensions are capped at 16384 and therefore exact in f32"
    )]
    let transform = Transform::from_scale(
        width as f32 / source_size.width(),
        height as f32 / source_size.height(),
    );
    resvg::render(tree, transform, &mut pixmap.as_mut());
    CanonicalRgba::new(width, height, pixmap.take_demultiplied())
}

/// Maximum wall-clock seconds one SVG render may take before it is abandoned.
pub const SVG_RENDER_TIMEOUT_SECONDS: u64 = 30;

/// Renders one normalized SVG tree with a hard wall-clock bound.
///
/// The render runs on a dedicated worker thread. When the deadline passes, the
/// caller receives [`RenderError::Timeout`] and the worker is abandoned to
/// finish and discard its result, so one pathological file cannot wedge the
/// calling interface indefinitely.
///
/// # Errors
///
/// Returns every [`render_svg`] failure, plus [`RenderError::Timeout`] when
/// the bound is exceeded or the worker dies before reporting.
pub fn render_svg_bounded(
    tree: &std::sync::Arc<usvg::Tree>,
    width: u32,
    height: u32,
) -> Result<CanonicalRgba, RenderError> {
    validate_dimensions(width, height)?;
    let worker_tree = std::sync::Arc::clone(tree);
    let (sender, receiver) = std::sync::mpsc::channel();
    if std::thread::Builder::new()
        .name("imaguncula-svg-render".to_owned())
        .spawn(move || {
            let _ = sender.send(render_svg(&worker_tree, width, height));
        })
        .is_err()
    {
        // Thread exhaustion is an environment failure, not hostile input;
        // render inline rather than failing the request.
        return render_svg(tree, width, height);
    }
    receiver
        .recv_timeout(std::time::Duration::from_secs(SVG_RENDER_TIMEOUT_SECONDS))
        .unwrap_or(Err(RenderError::Timeout {
            seconds: SVG_RENDER_TIMEOUT_SECONDS,
        }))
}

/// Resizes directly from canonical source pixels using alpha-aware Lanczos3.
///
/// # Errors
///
/// Returns [`RenderError::UpscaleForbidden`] when either target dimension is
/// larger than the source without `allow_upscale`, plus structured dimension,
/// allocation, and resizer failures.
pub fn resize_raster(
    source: &CanonicalRgba,
    width: u32,
    height: u32,
    allow_upscale: bool,
) -> Result<CanonicalRgba, RenderError> {
    let destination_bytes = validate_dimensions(width, height)?;
    if !allow_upscale && (width > source.width || height > source.height) {
        return Err(RenderError::UpscaleForbidden {
            source_width: source.width,
            source_height: source.height,
            target_width: width,
            target_height: height,
        });
    }
    if (width, height) == (source.width, source.height) {
        return Ok(source.clone());
    }

    let source_view = ImageRef::new(source.width, source.height, &source.pixels, PixelType::U8x4)
        .map_err(|error| RenderError::ResizeFailed {
        message: error.to_string(),
    })?;
    let mut destination_pixels = allocate_zeroed(destination_bytes)?;
    let mut destination =
        ResizeImage::from_slice_u8(width, height, &mut destination_pixels, PixelType::U8x4)
            .map_err(|error| RenderError::ResizeFailed {
                message: error.to_string(),
            })?;
    let options = ResizeOptions::new()
        .resize_alg(ResizeAlg::Convolution(FilterType::Lanczos3))
        .use_alpha(true);
    Resizer::new()
        .resize(&source_view, &mut destination, &options)
        .map_err(|error| RenderError::ResizeFailed {
            message: error.to_string(),
        })?;
    drop(destination);

    CanonicalRgba::new(width, height, destination_pixels)
}

/// Encodes canonical RGBA pixels as a deterministic PNG tagged as sRGB.
///
/// # Errors
///
/// Returns [`RenderError::PngEncode`] if the PNG header or image data cannot
/// be written.
pub fn encode_png_srgb(source: &CanonicalRgba) -> Result<Vec<u8>, RenderError> {
    let mut output = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut output, source.width, source.height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.set_source_srgb(png::SrgbRenderingIntent::Perceptual);
        let mut writer = encoder
            .write_header()
            .map_err(|error| RenderError::PngEncode {
                message: error.to_string(),
            })?;
        writer
            .write_image_data(&source.pixels)
            .map_err(|error| RenderError::PngEncode {
                message: error.to_string(),
            })?;
    }
    Ok(output)
}

/// Flattens canonical RGBA pixels over one opaque sRGB color and encodes an RGB PNG.
///
/// # Errors
///
/// Returns [`RenderError::PngEncode`] if the PNG header or image data cannot be written.
pub fn encode_png_srgb_flattened(
    source: &CanonicalRgba,
    background: [u8; 3],
) -> Result<Vec<u8>, RenderError> {
    let pixel_count = source.pixels.len() / 4;
    let rgb_length = pixel_count.checked_mul(3).ok_or(RenderError::Allocation {
        requested_bytes: u64::MAX,
    })?;
    let mut flattened = Vec::new();
    flattened
        .try_reserve_exact(rgb_length)
        .map_err(|_| RenderError::Allocation {
            requested_bytes: u64::try_from(rgb_length).unwrap_or(u64::MAX),
        })?;
    for pixel in source.pixels.chunks_exact(4) {
        let alpha = u16::from(pixel[3]);
        let inverse = u16::from(u8::MAX) - alpha;
        for (channel, matte) in pixel[..3].iter().zip(background) {
            let blended = (u16::from(*channel) * alpha + u16::from(matte) * inverse + 127) / 255;
            flattened.push(u8::try_from(blended).unwrap_or(u8::MAX));
        }
    }

    let mut output = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut output, source.width, source.height);
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.set_source_srgb(png::SrgbRenderingIntent::Perceptual);
        let mut writer = encoder
            .write_header()
            .map_err(|error| RenderError::PngEncode {
                message: error.to_string(),
            })?;
        writer
            .write_image_data(&flattened)
            .map_err(|error| RenderError::PngEncode {
                message: error.to_string(),
            })?;
    }
    Ok(output)
}

fn validate_dimensions(width: u32, height: u32) -> Result<usize, RenderError> {
    if width == 0 || height == 0 {
        return Err(RenderError::InvalidDimensions { width, height });
    }
    if width > PNG_DIMENSION_MAX || height > PNG_DIMENSION_MAX {
        return Err(RenderError::DimensionLimit {
            maximum_dimension: PNG_DIMENSION_MAX,
            width,
            height,
        });
    }
    let pixels = u64::from(width) * u64::from(height);
    if pixels > PNG_PIXEL_MAX {
        return Err(RenderError::PixelLimit {
            maximum_pixels: PNG_PIXEL_MAX,
            actual_pixels: pixels,
        });
    }
    let bytes = pixels * 4;
    usize::try_from(bytes).map_err(|_| RenderError::Allocation {
        requested_bytes: bytes,
    })
}

fn allocate_zeroed(length: usize) -> Result<Vec<u8>, RenderError> {
    let mut buffer = Vec::new();
    buffer
        .try_reserve_exact(length)
        .map_err(|_| RenderError::Allocation {
            requested_bytes: length as u64,
        })?;
    buffer.resize(length, 0);
    Ok(buffer)
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use image::{DynamicImage, ImageFormat, Rgba, RgbaImage};

    use super::*;

    const ASYMMETRIC_SVG: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/sources/asymmetric.svg"
    ));

    #[test]
    fn canonical_rgba_rejects_invalid_shape_and_buffer_length() {
        assert!(matches!(
            CanonicalRgba::new(0, 1, Vec::new()),
            Err(RenderError::InvalidDimensions { .. })
        ));
        assert!(matches!(
            CanonicalRgba::new(2, 2, vec![0; 15]),
            Err(RenderError::InvalidBufferLength {
                expected_bytes: 16,
                actual_bytes: 15
            })
        ));
    }

    #[test]
    fn svg_render_is_direct_repeatable_and_straight_alpha() {
        let tree = usvg::Tree::from_data(ASYMMETRIC_SVG, &usvg::Options::default())
            .expect("parse SVG fixture");
        let first = render_svg(&tree, 19, 13).expect("render SVG");
        let second = render_svg(&tree, 19, 13).expect("render SVG again");

        assert_eq!(first, second);
        assert_eq!((first.width(), first.height()), (19, 13));
        assert!(first.pixels().chunks_exact(4).any(|pixel| pixel[3] > 0));

        let alpha_svg = br##"<svg xmlns="http://www.w3.org/2000/svg" width="4" height="4"><rect width="4" height="4" fill="#ff0000" fill-opacity="0.5"/></svg>"##;
        let alpha_tree =
            usvg::Tree::from_data(alpha_svg, &usvg::Options::default()).expect("parse alpha SVG");
        let alpha = render_svg(&alpha_tree, 4, 4).expect("render alpha SVG");
        let center = &alpha.pixels()[20..24];
        assert!(center[0] > 250, "red must be demultiplied");
        assert!((126..=129).contains(&center[3]));
    }

    #[test]
    fn bounded_svg_render_matches_the_direct_render() {
        let tree = std::sync::Arc::new(
            usvg::Tree::from_data(ASYMMETRIC_SVG, &usvg::Options::default())
                .expect("parse SVG fixture"),
        );

        let direct = render_svg(&tree, 19, 13).expect("direct render");
        let bounded = render_svg_bounded(&tree, 19, 13).expect("bounded render");

        assert_eq!(direct, bounded);
    }

    #[test]
    fn raster_resize_blocks_upscale_without_explicit_override() {
        let source = CanonicalRgba::new(2, 2, vec![255; 16]).expect("canonical source");

        assert!(matches!(
            resize_raster(&source, 3, 2, false),
            Err(RenderError::UpscaleForbidden {
                source_width: 2,
                source_height: 2,
                target_width: 3,
                target_height: 2
            })
        ));
        let allowed = resize_raster(&source, 3, 2, true).expect("explicit upscale");
        assert_eq!((allowed.width(), allowed.height()), (3, 2));
    }

    #[test]
    fn raster_downsample_is_repeatable_and_preserves_source() {
        let pixels = (0_u8..64)
            .flat_map(|value| [value, value.wrapping_mul(3), 127, 128])
            .collect::<Vec<_>>();
        let source = CanonicalRgba::new(8, 8, pixels).expect("canonical source");
        let original = source.clone();

        let first = resize_raster(&source, 3, 5, false).expect("downsample");
        let second = resize_raster(&source, 3, 5, false).expect("repeat downsample");

        assert_eq!(first, second);
        assert_eq!(source, original);
        assert_eq!(first.pixels().len(), 3 * 5 * 4);
    }

    #[test]
    fn png_decode_produces_canonical_straight_alpha() {
        let image = RgbaImage::from_pixel(2, 1, Rgba([200, 100, 50, 64]));
        let mut encoded = Cursor::new(Vec::new());
        DynamicImage::ImageRgba8(image)
            .write_to(&mut encoded, ImageFormat::Png)
            .expect("encode PNG fixture");

        let decoded = decode_png_rgba(encoded.get_ref()).expect("decode PNG");

        assert_eq!((decoded.width(), decoded.height()), (2, 1));
        assert_eq!(decoded.pixels(), &[200, 100, 50, 64, 200, 100, 50, 64]);
    }

    #[test]
    fn png_encoding_is_repeatable_and_tagged_srgb() {
        let source = CanonicalRgba::new(1, 1, vec![7, 11, 13, 17]).expect("canonical RGBA");

        let first = encode_png_srgb(&source).expect("encode PNG");
        let second = encode_png_srgb(&source).expect("repeat PNG encoding");

        assert_eq!(first, second);
        assert!(first.windows(4).any(|chunk| chunk == b"sRGB"));
    }

    #[test]
    fn flattened_png_is_rgb_and_composites_transparency() {
        let source = CanonicalRgba::new(2, 1, vec![200, 100, 0, 255, 0, 0, 0, 0])
            .expect("canonical fixture");
        let encoded =
            encode_png_srgb_flattened(&source, [10, 20, 30]).expect("encode flattened PNG");
        let decoder = png::Decoder::new(Cursor::new(encoded));
        let mut reader = decoder.read_info().expect("read PNG info");
        assert_eq!(reader.info().color_type, png::ColorType::Rgb);
        let mut pixels = vec![0; reader.output_buffer_size().expect("bounded output size")];
        let info = reader.next_frame(&mut pixels).expect("decode PNG frame");
        assert_eq!(&pixels[..info.buffer_size()], &[200, 100, 0, 10, 20, 30]);
    }
}

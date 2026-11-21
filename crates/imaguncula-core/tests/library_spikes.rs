// SPDX-License-Identifier: MIT OR Apache-2.0
use std::io::Cursor;

use fast_image_resize::images::Image as ResizeImage;
use fast_image_resize::{FilterType, PixelType, ResizeAlg, ResizeOptions, Resizer};
use ico::{IconDir, IconDirEntry, IconImage, ResourceType};
use image::{DynamicImage, ImageFormat, RgbaImage};
use resvg::tiny_skia::{Pixmap, Transform};

const ASYMMETRIC_SVG: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../fixtures/sources/asymmetric.svg"
));
const EXPECTED_LANCZOS3_FNV1A64: u64 = 0xc47d_9be5_2f69_febc;

#[test]
fn resvg_renders_one_sanitized_tree_at_three_sizes() {
    let tree = usvg::Tree::from_data(ASYMMETRIC_SVG, &usvg::Options::default())
        .expect("fixture SVG should parse into a usvg tree");

    for (size, scale) in [(16, 0.25), (32, 0.5), (64, 1.0)] {
        let first = render_svg(&tree, size, scale);
        let second = render_svg(&tree, size, scale);

        assert_eq!(first, second, "rendering must be repeatable at {size}px");
        assert_eq!(first.len(), usize::try_from(size * size * 4).unwrap());
        assert!(
            first.chunks_exact(4).any(|pixel| pixel[3] != 0),
            "render must contain visible pixels at {size}px"
        );

        let rgba = RgbaImage::from_raw(size, size, first.clone())
            .expect("rendered RGBA buffer should have exact dimensions");
        let mut png = Cursor::new(Vec::new());
        DynamicImage::ImageRgba8(rgba)
            .write_to(&mut png, ImageFormat::Png)
            .expect("image crate should encode rendered RGBA as PNG");

        let decoded = image::load_from_memory(png.get_ref())
            .expect("image crate should decode its PNG output")
            .to_rgba8();
        assert_eq!(decoded.dimensions(), (size, size));
        assert_eq!(decoded.into_raw(), first);
    }
}

#[test]
fn lanczos3_rgba_resize_is_repeatable() {
    let first = resize_asymmetric_rgba();
    let second = resize_asymmetric_rgba();

    assert_eq!(first, second);
    assert_eq!(first.len(), 19 * 13 * 4);
    assert!(first.chunks_exact(4).any(|pixel| pixel[3] < u8::MAX));
    assert_eq!(fnv1a64(&first), EXPECTED_LANCZOS3_FNV1A64);
}

#[test]
fn ico_round_trip_preserves_entry_order_dimensions_encoding_and_pixels() {
    let mut directory = IconDir::new(ResourceType::Icon);
    let mut originals = Vec::new();

    for size in [16, 32, 256] {
        let rgba = icon_rgba(size);
        let image = IconImage::from_rgba_data(size, size, rgba.clone());
        let entry = if size == 256 {
            IconDirEntry::encode_as_png(&image).expect("256px entry should encode as PNG")
        } else {
            IconDirEntry::encode_as_bmp(&image).expect("small entry should encode as BMP")
        };

        originals.push(rgba);
        directory.add_entry(entry);
    }

    let mut encoded = Cursor::new(Vec::new());
    directory
        .write(&mut encoded)
        .expect("multi-entry ICO should write to memory");
    encoded.set_position(0);

    let decoded = IconDir::read(encoded).expect("written ICO should parse");
    assert_eq!(decoded.resource_type(), ResourceType::Icon);
    assert_eq!(decoded.entries().len(), 3);

    for (index, ((entry, size), original)) in decoded
        .entries()
        .iter()
        .zip([16, 32, 256])
        .zip(originals)
        .enumerate()
    {
        assert_eq!(entry.width(), size, "entry {index} width changed");
        assert_eq!(entry.height(), size, "entry {index} height changed");
        assert_eq!(entry.bits_per_pixel(), 32, "entry {index} lost RGBA depth");
        assert_eq!(
            entry.is_png(),
            size == 256,
            "entry {index} encoding changed"
        );

        let image = entry.decode().expect("ICO entry should decode");
        assert_eq!(image.rgba_data(), original);
    }
}

fn render_svg(tree: &usvg::Tree, size: u32, scale: f32) -> Vec<u8> {
    let mut pixmap = Pixmap::new(size, size).expect("target pixmap dimensions should be valid");
    resvg::render(
        tree,
        Transform::from_scale(scale, scale),
        &mut pixmap.as_mut(),
    );
    pixmap.data().to_vec()
}

fn resize_asymmetric_rgba() -> Vec<u8> {
    let source = ResizeImage::from_vec_u8(64, 64, source_rgba(), PixelType::U8x4)
        .expect("source RGBA buffer should be valid");
    let mut destination = ResizeImage::new(19, 13, PixelType::U8x4);
    let options = ResizeOptions::new()
        .resize_alg(ResizeAlg::Convolution(FilterType::Lanczos3))
        .use_alpha(true);

    Resizer::new()
        .resize(&source, &mut destination, &options)
        .expect("Lanczos3 resize should succeed");
    destination.into_vec()
}

fn source_rgba() -> Vec<u8> {
    let mut pixels = Vec::with_capacity(64 * 64 * 4);
    for y in 0_u8..64 {
        for x in 0_u8..64 {
            pixels.extend_from_slice(&[
                x.wrapping_mul(3).wrapping_add(y),
                y.wrapping_mul(5).wrapping_add(17),
                x.wrapping_add(y.wrapping_mul(2)),
                96_u8.saturating_add(x).saturating_add(y),
            ]);
        }
    }
    pixels
}

fn icon_rgba(size: u32) -> Vec<u8> {
    let dimension = usize::try_from(size).expect("supported icon size fits usize");
    let mut pixels = Vec::with_capacity(dimension * dimension * 4);

    for y in 0..size {
        for x in 0..size {
            let x8 = x.to_le_bytes()[0];
            let y8 = y.to_le_bytes()[0];
            let on_edge = x == 0 || y == 0 || x + 1 == size || y + 1 == size;
            pixels.extend_from_slice(&[
                x8.wrapping_mul(7).wrapping_add(11),
                y8.wrapping_mul(5).wrapping_add(23),
                x8.wrapping_add(y8).wrapping_mul(3),
                if on_edge { 0x80 } else { u8::MAX },
            ]);
        }
    }

    pixels
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

// SPDX-License-Identifier: MIT OR Apache-2.0
#![no_main]

use imaguncula_core::{
    CanonicalRgba, IconContainerEntry, IconContainerFormat, IconEntryEncoding,
    encode_icon_container,
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let sizes = [16_u32, 20, 24, 32, 48, 64, 128, 256];
    let size = sizes[data.first().copied().unwrap_or(0) as usize % sizes.len()];
    let pixels = fuzz_pixels(size, data.get(1..).unwrap_or_default());
    if let Ok(image) = CanonicalRgba::new(size, size, pixels) {
        let encoding = if data.get(1).copied().unwrap_or(0) & 1 == 0 {
            IconEntryEncoding::Bmp
        } else {
            IconEntryEncoding::Png
        };
        let _ = encode_icon_container(
            IconContainerFormat::Ico,
            vec![IconContainerEntry { image, encoding }],
        );
    }
});

fn fuzz_pixels(size: u32, seed: &[u8]) -> Vec<u8> {
    let length = size as usize * size as usize * 4;
    if seed.is_empty() {
        return vec![0; length];
    }
    (0..length).map(|index| seed[index % seed.len()]).collect()
}

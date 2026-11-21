// SPDX-License-Identifier: MIT OR Apache-2.0
#![no_main]

use imaguncula_core::{
    CanonicalRgba, IconContainerEntry, IconContainerFormat, IconEntryEncoding,
    encode_icon_container,
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let sizes = [16_u32, 32, 64, 128, 256, 512, 1024];
    let size = sizes[data.first().copied().unwrap_or(0) as usize % sizes.len()];
    let pixels = fuzz_pixels(size, data.get(1..).unwrap_or_default());
    if let Ok(image) = CanonicalRgba::new(size, size, pixels) {
        let _ = encode_icon_container(
            IconContainerFormat::Icns,
            vec![IconContainerEntry {
                image,
                encoding: IconEntryEncoding::Argb,
            }],
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

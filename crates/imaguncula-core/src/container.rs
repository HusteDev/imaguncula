// SPDX-License-Identifier: MIT OR Apache-2.0
//! Bounded, pure ICO and ICNS encoder inputs shared by export and fuzz harnesses.

use std::io::Cursor;

use serde::Serialize;
use thiserror::Error;

use crate::CanonicalRgba;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IconContainerFormat {
    Ico,
    Icns,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IconEntryEncoding {
    Bmp,
    Png,
    Argb,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IconContainerEntry {
    pub image: CanonicalRgba,
    pub encoding: IconEntryEncoding,
}

#[derive(Debug, Error, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum ContainerEncodeError {
    #[error("an icon container must contain at least one entry")]
    Empty,
    #[error("entry encoding is not supported by this container")]
    UnsupportedEncoding,
    #[error("ICO encoding failed")]
    Ico { message: String },
    #[error("ICNS encoding failed")]
    Icns { message: String },
}

/// Encodes already-bounded canonical images into one ICO or ICNS container.
///
/// # Errors
///
/// Returns a typed error for an empty list, an invalid format/encoding combination, unsupported
/// ICNS dimensions, or an encoder failure.
pub fn encode_icon_container(
    format: IconContainerFormat,
    entries: Vec<IconContainerEntry>,
) -> Result<Vec<u8>, ContainerEncodeError> {
    if entries.is_empty() {
        return Err(ContainerEncodeError::Empty);
    }
    match format {
        IconContainerFormat::Ico => encode_ico(entries),
        IconContainerFormat::Icns => encode_icns(entries),
    }
}

fn encode_ico(entries: Vec<IconContainerEntry>) -> Result<Vec<u8>, ContainerEncodeError> {
    let mut directory = ico::IconDir::new(ico::ResourceType::Icon);
    for entry in entries {
        let image = ico::IconImage::from_rgba_data(
            entry.image.width(),
            entry.image.height(),
            entry.image.into_pixels(),
        );
        let encoded = match entry.encoding {
            IconEntryEncoding::Bmp => ico::IconDirEntry::encode_as_bmp(&image),
            IconEntryEncoding::Png => ico::IconDirEntry::encode_as_png(&image),
            IconEntryEncoding::Argb => return Err(ContainerEncodeError::UnsupportedEncoding),
        }
        .map_err(|error| ContainerEncodeError::Ico {
            message: error.to_string(),
        })?;
        directory.add_entry(encoded);
    }
    let mut bytes = Vec::new();
    directory
        .write(&mut bytes)
        .map_err(|error| ContainerEncodeError::Ico {
            message: error.to_string(),
        })?;
    Ok(bytes)
}

fn encode_icns(entries: Vec<IconContainerEntry>) -> Result<Vec<u8>, ContainerEncodeError> {
    let mut family = icns::IconFamily::new();
    for entry in entries {
        if entry.encoding == IconEntryEncoding::Bmp {
            return Err(ContainerEncodeError::UnsupportedEncoding);
        }
        let image = icns::Image::from_data(
            icns::PixelFormat::RGBA,
            entry.image.width(),
            entry.image.height(),
            entry.image.into_pixels(),
        )
        .map_err(|error| ContainerEncodeError::Icns {
            message: error.to_string(),
        })?;
        family
            .add_icon(&image)
            .map_err(|error| ContainerEncodeError::Icns {
                message: error.to_string(),
            })?;
    }
    let mut bytes = Vec::new();
    family
        .write(Cursor::new(&mut bytes))
        .map_err(|error| ContainerEncodeError::Icns {
            message: error.to_string(),
        })?;
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn image(size: u32) -> CanonicalRgba {
        CanonicalRgba::new(size, size, vec![127; size as usize * size as usize * 4])
            .expect("bounded test image")
    }

    #[test]
    fn ico_and_icns_inputs_encode_and_reopen() {
        let ico = encode_icon_container(
            IconContainerFormat::Ico,
            vec![IconContainerEntry {
                image: image(16),
                encoding: IconEntryEncoding::Bmp,
            }],
        )
        .expect("ICO encodes");
        let decoded = ico::IconDir::read(Cursor::new(ico)).expect("ICO reopens");
        assert_eq!(decoded.entries().len(), 1);

        let icns = encode_icon_container(
            IconContainerFormat::Icns,
            vec![IconContainerEntry {
                image: image(16),
                encoding: IconEntryEncoding::Argb,
            }],
        )
        .expect("ICNS encodes");
        let decoded = icns::IconFamily::read(Cursor::new(icns)).expect("ICNS reopens");
        assert!(decoded.has_icon_with_type(icns::IconType::from_pixel_size(16, 16).unwrap()));
    }

    #[test]
    fn invalid_container_inputs_fail_without_partial_bytes() {
        assert!(matches!(
            encode_icon_container(IconContainerFormat::Ico, Vec::new()),
            Err(ContainerEncodeError::Empty)
        ));
        assert!(matches!(
            encode_icon_container(
                IconContainerFormat::Ico,
                vec![IconContainerEntry {
                    image: image(16),
                    encoding: IconEntryEncoding::Argb,
                }]
            ),
            Err(ContainerEncodeError::UnsupportedEncoding)
        ));
    }
}

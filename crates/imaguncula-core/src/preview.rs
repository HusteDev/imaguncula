// SPDX-License-Identifier: MIT OR Apache-2.0
//! On-demand source previews with content-addressed caching.

use std::{collections::HashMap, sync::Arc};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    RenderError, SourceError, SourceFormat, SourceInfo, decode_png_rgba, encode_png_srgb,
    render_svg_bounded, resize_raster,
    source::{
        LoadedSource, inspect_source_data, load_source_data, probe_png_header, read_source_data,
    },
};

/// Maximum preview width or height accepted over IPC.
pub const PREVIEW_DIMENSION_MAX: u32 = 4_096;

/// Maximum number of encoded previews retained in memory.
pub const PREVIEW_CACHE_MAX_ENTRIES: usize = 64;

/// Maximum encoded preview bytes retained in memory.
pub const PREVIEW_CACHE_MAX_BYTES: usize = 64 * 1024 * 1024;

const PREVIEW_ROLE_MAX_ENTRIES: usize = 64;
const VALIDATED_SOURCE_MAX_ENTRIES: usize = 8;

/// PNG preview response returned through interface adapters.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PreviewImage {
    pub content_hash: String,
    pub width: u32,
    pub height: u32,
    pub png_bytes: Vec<u8>,
    pub cache_hit: bool,
}

/// Structured preview failures.
#[derive(Debug, Error, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum PreviewError {
    #[error("source import failed: {error}")]
    Source { error: SourceError },
    #[error("preview rendering failed: {error}")]
    Render { error: RenderError },
    #[error("preview cache is unavailable")]
    CacheUnavailable,
    #[error("preview cache lost a validated source intermediate")]
    CacheInvariant,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct PreviewCacheKey {
    content_hash: String,
    width: u32,
    height: u32,
}

#[derive(Default)]
pub struct PreviewCache {
    entries: HashMap<PreviewCacheKey, Vec<u8>>,
    role_hashes: HashMap<String, String>,
    validated_sources: HashMap<String, ValidatedSource>,
    encoded_bytes: usize,
}

struct ValidatedSource {
    info: SourceInfo,
    svg_tree: Option<Arc<usvg::Tree>>,
}

/// Outcome of the lock-held preparation phase of one preview request.
pub enum PreparedPreview {
    /// The preview was already cached; no rendering is needed.
    Cached(PreviewImage),
    /// Validated rendering work to run without holding the cache lock.
    Pending(PendingPreview),
}

/// One validated, un-rendered preview request that owns everything rendering
/// needs, so a shared cache lock can be released while it computes.
pub struct PendingPreview {
    content_hash: String,
    width: u32,
    height: u32,
    allow_upscale: bool,
    work: PreviewWork,
}

enum PreviewWork {
    Png { bytes: Vec<u8> },
    Svg { tree: Arc<usvg::Tree> },
}

impl PendingPreview {
    /// Renders and encodes the prepared preview without touching the cache.
    ///
    /// SVG work runs through the wall-clock-bounded renderer, so one
    /// pathological source cannot stall the caller indefinitely.
    ///
    /// # Errors
    ///
    /// Returns [`PreviewError::Render`] for decode, policy, render, timeout,
    /// or PNG encoding failures.
    pub fn compute(self) -> Result<PreviewImage, PreviewError> {
        let rgba = match &self.work {
            PreviewWork::Png { bytes } => {
                let source =
                    decode_png_rgba(bytes).map_err(|error| PreviewError::Render { error })?;
                resize_raster(&source, self.width, self.height, self.allow_upscale)
                    .map_err(|error| PreviewError::Render { error })?
            }
            PreviewWork::Svg { tree } => render_svg_bounded(tree, self.width, self.height)
                .map_err(|error| PreviewError::Render { error })?,
        };
        let png_bytes = encode_png_srgb(&rgba).map_err(|error| PreviewError::Render { error })?;
        Ok(PreviewImage {
            content_hash: self.content_hash,
            width: self.width,
            height: self.height,
            png_bytes,
            cache_hit: false,
        })
    }
}

impl PreviewCache {
    /// Validates and retains one source's sanitized intermediate for previews.
    ///
    /// # Errors
    ///
    /// Returns [`SourceError`] if the source fails the bounded import policy.
    pub fn import(
        &mut self,
        path: impl AsRef<std::path::Path>,
        role: impl Into<String>,
    ) -> Result<SourceInfo, SourceError> {
        let loaded = load_source_data(path, role)?;
        self.register_role_hash(&loaded.info.role, &loaded.info.content_hash);
        let info = loaded.info.clone();
        drop(self.insert_validated_source(loaded));
        Ok(info)
    }

    /// Validates one preview request and returns either the cached PNG or the
    /// owned rendering work, without performing any rendering itself.
    ///
    /// Replacing a role's source hash removes every cached size for the previous
    /// hash. Raster upscaling authority is checked before cache lookup, so a
    /// preview generated with an override cannot bypass a later denied request.
    ///
    /// Interface adapters that guard the cache with a lock should release it
    /// between this call and [`PendingPreview::compute`], then re-acquire it
    /// for [`PreviewCache::store_preview`], so slow renders never block other
    /// cache users.
    ///
    /// # Errors
    ///
    /// Returns [`PreviewError`] for source import or policy failures.
    pub fn prepare_render(
        &mut self,
        path: impl AsRef<std::path::Path>,
        role: impl Into<String>,
        width: u32,
        height: u32,
        allow_upscale: bool,
    ) -> Result<PreparedPreview, PreviewError> {
        validate_preview_dimensions(width, height)?;
        let role = role.into();
        let source =
            read_source_data(path, role.clone()).map_err(|error| PreviewError::Source { error })?;
        let content_hash = source.content_hash.clone();
        self.register_role_hash(&role, &content_hash);

        let bytes = if self.validated_sources.contains_key(&content_hash) {
            source.bytes
        } else {
            let loaded =
                inspect_source_data(source).map_err(|error| PreviewError::Source { error })?;
            self.insert_validated_source(loaded)
        };
        let validated = self
            .validated_sources
            .get(&content_hash)
            .ok_or(PreviewError::CacheInvariant)?;

        if validated.info.format == SourceFormat::Png {
            let (source_width, source_height) = probe_png_header(&bytes, &validated.info.path)
                .map_err(|error| PreviewError::Source { error })?;
            if !allow_upscale && (width > source_width || height > source_height) {
                return Err(PreviewError::Render {
                    error: RenderError::UpscaleForbidden {
                        source_width,
                        source_height,
                        target_width: width,
                        target_height: height,
                    },
                });
            }
        }

        let key = PreviewCacheKey {
            content_hash,
            width,
            height,
        };
        if let Some(png_bytes) = self.entries.get(&key) {
            return Ok(PreparedPreview::Cached(PreviewImage {
                content_hash: key.content_hash,
                width,
                height,
                png_bytes: png_bytes.clone(),
                cache_hit: true,
            }));
        }

        let work = match validated.info.format {
            SourceFormat::Png => PreviewWork::Png { bytes },
            SourceFormat::Svg => PreviewWork::Svg {
                tree: Arc::clone(
                    validated
                        .svg_tree
                        .as_ref()
                        .ok_or(PreviewError::CacheInvariant)?,
                ),
            },
        };
        Ok(PreparedPreview::Pending(PendingPreview {
            content_hash: key.content_hash,
            width,
            height,
            allow_upscale,
            work,
        }))
    }

    /// Caches one computed preview after rendering outside the lock.
    pub fn store_preview(&mut self, image: &PreviewImage) {
        self.insert(
            PreviewCacheKey {
                content_hash: image.content_hash.clone(),
                width: image.width,
                height: image.height,
            },
            image.png_bytes.clone(),
        );
    }

    /// Renders a source preview or returns the content-addressed cached PNG.
    ///
    /// Single-call convenience over [`PreviewCache::prepare_render`],
    /// [`PendingPreview::compute`], and [`PreviewCache::store_preview`].
    ///
    /// # Errors
    ///
    /// Returns [`PreviewError`] for source import, rendering, policy, or PNG
    /// encoding failures.
    pub fn render(
        &mut self,
        path: impl AsRef<std::path::Path>,
        role: impl Into<String>,
        width: u32,
        height: u32,
        allow_upscale: bool,
    ) -> Result<PreviewImage, PreviewError> {
        match self.prepare_render(path, role, width, height, allow_upscale)? {
            PreparedPreview::Cached(image) => Ok(image),
            PreparedPreview::Pending(pending) => {
                let image = pending.compute()?;
                self.store_preview(&image);
                Ok(image)
            }
        }
    }

    fn register_role_hash(&mut self, role: &str, content_hash: &str) {
        if let Some(previous) = self.role_hashes.get(role) {
            if previous == content_hash {
                return;
            }
            let previous = previous.clone();
            self.entries.retain(|key, _| key.content_hash != previous);
            self.validated_sources.remove(&previous);
            self.recount_bytes();
        } else if self.role_hashes.len() >= PREVIEW_ROLE_MAX_ENTRIES {
            self.clear();
        }
        self.role_hashes
            .insert(role.to_owned(), content_hash.to_owned());
    }

    fn insert_validated_source(&mut self, loaded: LoadedSource) -> Vec<u8> {
        let LoadedSource {
            info,
            bytes,
            svg_tree,
        } = loaded;
        let content_hash = info.content_hash.clone();
        if self.validated_sources.len() >= VALIDATED_SOURCE_MAX_ENTRIES
            && !self.validated_sources.contains_key(&content_hash)
        {
            self.validated_sources.clear();
        }
        self.validated_sources
            .insert(content_hash, ValidatedSource { info, svg_tree });
        bytes
    }

    fn insert(&mut self, key: PreviewCacheKey, bytes: Vec<u8>) {
        if bytes.len() > PREVIEW_CACHE_MAX_BYTES {
            return;
        }
        if self.entries.len() >= PREVIEW_CACHE_MAX_ENTRIES
            || self.encoded_bytes.saturating_add(bytes.len()) > PREVIEW_CACHE_MAX_BYTES
        {
            self.entries.clear();
            self.encoded_bytes = 0;
        }
        self.encoded_bytes += bytes.len();
        self.entries.insert(key, bytes);
    }

    fn recount_bytes(&mut self) {
        self.encoded_bytes = self.entries.values().map(Vec::len).sum();
    }

    fn clear(&mut self) {
        self.entries.clear();
        self.role_hashes.clear();
        self.validated_sources.clear();
        self.encoded_bytes = 0;
    }
}

fn validate_preview_dimensions(width: u32, height: u32) -> Result<(), PreviewError> {
    if width == 0 || height == 0 {
        return Err(PreviewError::Render {
            error: RenderError::InvalidDimensions { width, height },
        });
    }
    if width > PREVIEW_DIMENSION_MAX || height > PREVIEW_DIMENSION_MAX {
        return Err(PreviewError::Render {
            error: RenderError::DimensionLimit {
                maximum_dimension: PREVIEW_DIMENSION_MAX,
                width,
                height,
            },
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{fs, io::Cursor};

    use image::{DynamicImage, ImageFormat, Rgba, RgbaImage};
    use tempfile::tempdir;

    use super::*;

    fn write_png(path: &std::path::Path, color: [u8; 4]) {
        let image = RgbaImage::from_pixel(8, 8, Rgba(color));
        let mut encoded = Cursor::new(Vec::new());
        DynamicImage::ImageRgba8(image)
            .write_to(&mut encoded, ImageFormat::Png)
            .expect("encode PNG fixture");
        fs::write(path, encoded.into_inner()).expect("write PNG fixture");
    }

    #[test]
    fn preview_cache_hits_by_hash_and_size_and_invalidates_on_role_replacement() {
        let directory = tempdir().expect("temporary directory");
        let first_path = directory.path().join("first.png");
        let second_path = directory.path().join("second.png");
        write_png(&first_path, [255, 0, 0, 255]);
        write_png(&second_path, [0, 255, 0, 255]);
        let mut cache = PreviewCache::default();

        let first = cache
            .render(&first_path, "master", 4, 4, false)
            .expect("first preview");
        let repeated = cache
            .render(&first_path, "master", 4, 4, false)
            .expect("cached preview");
        let replacement = cache
            .render(&second_path, "master", 4, 4, false)
            .expect("replacement preview");
        let old_again = cache
            .render(&first_path, "master", 4, 4, false)
            .expect("old source after replacement");

        assert!(!first.cache_hit);
        assert!(repeated.cache_hit);
        assert!(!replacement.cache_hit);
        assert!(!old_again.cache_hit);
        assert_eq!(first.png_bytes, repeated.png_bytes);
        assert_ne!(first.content_hash, replacement.content_hash);
    }

    #[test]
    fn cached_upscale_cannot_bypass_a_later_denied_request() {
        let directory = tempdir().expect("temporary directory");
        let path = directory.path().join("source.png");
        write_png(&path, [1, 2, 3, 255]);
        let mut cache = PreviewCache::default();

        cache
            .render(&path, "master", 16, 16, true)
            .expect("authorized upscale");
        let denied = cache
            .render(&path, "master", 16, 16, false)
            .expect_err("deny unauthorized cached upscale");

        assert!(matches!(
            denied,
            PreviewError::Render {
                error: RenderError::UpscaleForbidden { .. }
            }
        ));
    }

    #[test]
    fn svg_preview_png_is_tagged_srgb() {
        let directory = tempdir().expect("temporary directory");
        let path = directory.path().join("source.svg");
        fs::write(
            &path,
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="8" height="8"><rect width="8" height="8" fill="red"/></svg>"#,
        )
        .expect("write SVG fixture");
        let mut cache = PreviewCache::default();

        let source = cache.import(&path, "master").expect("import SVG source");
        assert!(
            cache
                .validated_sources
                .get(&source.content_hash)
                .and_then(|source| source.svg_tree.as_ref())
                .is_some()
        );

        let preview = cache
            .render(&path, "master", 8, 8, false)
            .expect("SVG preview");
        let second_size = cache
            .render(&path, "master", 4, 4, false)
            .expect("second SVG preview size");

        assert!(preview.png_bytes.windows(4).any(|chunk| chunk == b"sRGB"));
        assert!(!second_size.cache_hit);
        assert_eq!(cache.validated_sources.len(), 1);
    }

    #[test]
    fn preview_dimensions_are_bounded_before_source_io() {
        let mut cache = PreviewCache::default();

        assert!(matches!(
            cache.render("missing.png", "master", PREVIEW_DIMENSION_MAX + 1, 1, false),
            Err(PreviewError::Render {
                error: RenderError::DimensionLimit { .. }
            })
        ));
    }
}

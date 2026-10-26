//! Precompiled, non-generic wrappers for toolbox crates whose generic API would otherwise be
//! compiled again in every new script. Scripts use them as `tb::img::open(..)`.
//!
//! Measured first build of a new script (2026-10-26): `image::DynamicImage::save` 13 s and
//! `image::open` 4 s, because every encoder/decoder is instantiated for the script's writer
//! or reader type. All other toolbox crates build in 1-3.5 s and need no wrapper. Add a
//! wrapper only for a call measured to be expensive; each one makes this library bigger.

pub mod img {
    //! Load and save images without compiling the codecs of `image` in each script.
    use image::{DynamicImage, ExtendedColorType, GrayImage, ImageFormat, ImageResult, RgbImage, RgbaImage};
    use std::path::Path;

    /// Like `image::open`: decode a file, format from its extension.
    pub fn open(path: impl AsRef<Path>) -> ImageResult<DynamicImage> {
        open_path(path.as_ref())
    }
    fn open_path(path: &Path) -> ImageResult<DynamicImage> {
        image::open(path)
    }

    /// Like `image::load_from_memory`: decode bytes, format guessed from the content.
    pub fn load(bytes: &[u8]) -> ImageResult<DynamicImage> {
        image::load_from_memory(bytes)
    }

    /// Like `DynamicImage::save`: encode to a file, format from its extension.
    pub fn save(img: &DynamicImage, path: impl AsRef<Path>) -> ImageResult<()> {
        save_path(img, path.as_ref())
    }
    fn save_path(img: &DynamicImage, path: &Path) -> ImageResult<()> {
        img.save(path)
    }

    /// Like `DynamicImage::save_with_format`.
    pub fn save_as(img: &DynamicImage, path: impl AsRef<Path>, format: ImageFormat) -> ImageResult<()> {
        save_as_path(img, path.as_ref(), format)
    }
    fn save_as_path(img: &DynamicImage, path: &Path, format: ImageFormat) -> ImageResult<()> {
        img.save_with_format(path, format)
    }

    /// Encode into memory, e.g. for an HTTP upload or a data URL.
    pub fn encode(img: &DynamicImage, format: ImageFormat) -> ImageResult<Vec<u8>> {
        let mut out = std::io::Cursor::new(Vec::new());
        img.write_to(&mut out, format)?;
        Ok(out.into_inner())
    }

    /// `RgbImage::save` without converting to `DynamicImage` first.
    pub fn save_rgb(img: &RgbImage, path: impl AsRef<Path>) -> ImageResult<()> {
        save_raw(path.as_ref(), img.as_raw(), img.width(), img.height(), ExtendedColorType::Rgb8)
    }

    /// `RgbaImage::save` without converting to `DynamicImage` first.
    pub fn save_rgba(img: &RgbaImage, path: impl AsRef<Path>) -> ImageResult<()> {
        save_raw(path.as_ref(), img.as_raw(), img.width(), img.height(), ExtendedColorType::Rgba8)
    }

    /// `GrayImage::save` without converting to `DynamicImage` first.
    pub fn save_gray(img: &GrayImage, path: impl AsRef<Path>) -> ImageResult<()> {
        save_raw(path.as_ref(), img.as_raw(), img.width(), img.height(), ExtendedColorType::L8)
    }

    fn save_raw(path: &Path, buf: &[u8], w: u32, h: u32, color: ExtendedColorType) -> ImageResult<()> {
        image::save_buffer(path, buf, w, h, color)
    }
}

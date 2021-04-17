use crate::font::locator::FontDataHandle;
use crate::utils::PixelLength;
use failure::Fallible;

pub mod freetype;

pub struct RasterizedGlyph {
    pub data: Vec<u8>,
    pub height: usize,
    pub width: usize,
    pub top: PixelLength,
    pub left: PixelLength,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct FontMetrics {
    pub cell_width: PixelLength,
    pub cell_height: PixelLength,
    pub descender: PixelLength,
    pub underline_thickness: PixelLength,
    pub underline_position: PixelLength,
}

pub trait FontRasterizer {
    fn rasterize_glyph(&self, glyph_pos: u32) -> Fallible<RasterizedGlyph>;
}

pub fn new_rasterizer(
    handle: &FontDataHandle,
    size: f64,
    dpi: u32,
) -> Fallible<Box<dyn FontRasterizer>> {
    Ok(Box::new(freetype::FreeTypeRasterizer::new(handle, size, dpi)?))
}

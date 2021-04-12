use crate::font::locator::FontDataHandle;
use crate::utils::PixelLength;
use failure::Fallible;

pub mod allsorts;

#[derive(Clone, Debug)]
pub struct GlyphInfo {
    #[cfg(debug_assertions)]
    pub text: String,
    pub cluster: u32,
    pub num_cells: u8,
    pub glyph_pos: u32,
    pub x_advance: PixelLength,
    pub y_advance: PixelLength,
    pub x_offset: PixelLength,
    pub y_offset: PixelLength,
}

pub type FallbackIdx = usize;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct FontMetrics {
    pub cell_width: PixelLength,
    pub cell_height: PixelLength,
    pub descender: PixelLength,
    pub underline_thickness: PixelLength,
    pub underline_position: PixelLength,
}

pub trait FontShaper {
    fn shape(&self, text: &str, size: f64, dpi: u32) -> Fallible<Vec<GlyphInfo>>;
    fn metrics(&self, size: f64, dpi: u32) -> Fallible<FontMetrics>;
}

#[allow(dead_code)]
pub enum FontShaperSelection {
    Allsorts,
}

pub fn new_shaper(
    shaper: FontShaperSelection,
    handle: &FontDataHandle,
) -> Fallible<Box<dyn FontShaper>> {
    match shaper {
        FontShaperSelection::Allsorts => Ok(Box::new(allsorts::AllsortsShaper::new(handle)?)),
    }
}

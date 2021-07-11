use crate::font::ftwrap;
use crate::utils::PixelLength;
use anyhow::Result;

pub mod harfbuzz;

#[derive(Clone, Debug)]
pub struct GlyphInfo {
    pub glyph_pos: u32,
    pub x_advance: PixelLength,
    pub y_advance: PixelLength,
    pub x_offset: PixelLength,
    pub y_offset: PixelLength,
}

pub trait FontShaper {
    fn shape(&self, text: &str) -> Result<Vec<GlyphInfo>>;
}

pub fn new_shaper(face: &ftwrap::Face) -> Result<Box<dyn FontShaper>> {
    Ok(Box::new(harfbuzz::HarfbuzzShaper::new(face)?))
}

use crate::font::ftwrap;
use crate::utils::PixelLength;
use failure::Fallible;

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
    fn shape(
        &self,
        text: &str,
        hb_script: u32,
        hb_direction: u32,
        hb_lang: &str,
    ) -> Fallible<Vec<GlyphInfo>>;
}

pub fn new_shaper(face: &ftwrap::Face) -> Fallible<Box<dyn FontShaper>> {
    Ok(Box::new(harfbuzz::HarfbuzzShaper::new(face)?))
}

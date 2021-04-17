use crate::font::locator::FontDataHandle;
use crate::font::rasterizer::FontRasterizer;
use crate::font::{ftwrap, RasterizedGlyph};
use crate::utils::PixelLength;
use failure::Fallible;
use freetype::freetype::FT_GlyphSlotRec_;
use std::cell::RefCell;
use std::slice;

pub struct FreeTypeRasterizer {
    face: RefCell<ftwrap::Face>,
    _lib: ftwrap::Library,
}

impl FontRasterizer for FreeTypeRasterizer {
    fn rasterize_glyph(&self, glyph_pos: u32) -> Fallible<RasterizedGlyph> {
        let (load_flags, render_mode) = ftwrap::compute_load_flags();

        let mut face = self.face.borrow_mut();
        let ft_glyph = face.load_and_render_glyph(glyph_pos, load_flags, render_mode)?;

        let pitch = ft_glyph.bitmap.pitch.abs() as usize;
        let data = unsafe {
            slice::from_raw_parts_mut(ft_glyph.bitmap.buffer, ft_glyph.bitmap.rows as usize * pitch)
        };

        let glyph = self.rasterize(pitch, ft_glyph, data);
        Ok(glyph)
    }
}

impl FreeTypeRasterizer {
    fn rasterize(&self, pitch: usize, ft_glyph: &FT_GlyphSlotRec_, data: &[u8]) -> RasterizedGlyph {
        let width = ft_glyph.bitmap.width as usize;
        let height = ft_glyph.bitmap.rows as usize;

        let mut packed = Vec::with_capacity(height * width);
        for i in 0..height {
            let start = (i as usize) * pitch;
            let stop = start + width;
            packed.extend_from_slice(&data[start..stop]);
        }
        RasterizedGlyph {
            data: packed,
            height,
            width: width / 3,
            left: PixelLength::new(ft_glyph.bitmap_left as f64),
            top: PixelLength::new(ft_glyph.bitmap_top as f64),
        }
    }

    pub fn new(handle: &FontDataHandle, size: f64, dpi: u32) -> Fallible<Self> {
        let lib = ftwrap::Library::new()?;
        let mut face = lib.face_from_locator(handle)?;
        face.set_font_size(size, dpi)?;
        Ok(Self { _lib: lib, face: RefCell::new(face) })
    }
}

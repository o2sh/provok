use crate::font::loader::FontDataHandle;
use crate::font::rasterizer::FontRasterizer;
use crate::font::{ftwrap, RasterizedGlyph};
use crate::utils::PixelLength;
use failure::Fallible;
use freetype::freetype::FT_GlyphSlotRec_;
use std::cell::RefCell;
use std::slice;

pub struct FreeTypeRasterizer {
    _lib: ftwrap::Library,
    face: RefCell<ftwrap::Face>,
}

impl FontRasterizer for FreeTypeRasterizer {
    fn rasterize(&self, glyph_pos: u32) -> Fallible<RasterizedGlyph> {
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
        let width = ft_glyph.bitmap.width as usize / 3;
        let height = ft_glyph.bitmap.rows as usize;
        let size = (width * height * 4) as usize;
        let mut rgba = vec![0u8; size];
        for y in 0..height {
            let src_offset = y * pitch as usize;
            let dest_offset = y * width * 4;
            for x in 0..width {
                let red = data[src_offset + (x * 3)];
                let green = data[src_offset + (x * 3) + 1];
                let blue = data[src_offset + (x * 3) + 2];
                let alpha = red.min(green).min(blue);
                rgba[dest_offset + (x * 4)] = red;
                rgba[dest_offset + (x * 4) + 1] = green;
                rgba[dest_offset + (x * 4) + 2] = blue;
                rgba[dest_offset + (x * 4) + 3] = alpha;
            }
        }
        RasterizedGlyph {
            data: rgba,
            height,
            width,
            left: PixelLength::new(ft_glyph.bitmap_left as f64),
            top: PixelLength::new(ft_glyph.bitmap_top as f64),
        }
    }

    pub fn new(font_data_handle: &FontDataHandle, font_size: f64, dpi: u32) -> Fallible<Self> {
        let lib = ftwrap::Library::new()?;
        let mut face = lib.new_face(&font_data_handle)?;
        face.set_font_size(font_size, dpi)?;
        Ok(Self { _lib: lib, face: RefCell::new(face) })
    }
}

use crate::font::locator::FontDataHandle;
use crate::font::rasterizer::FontRasterizer;
use crate::font::{ftwrap, RasterizedGlyph};
use crate::utils::PixelLength;
use failure::Fallible;
use freetype::freetype::FT_GlyphSlotRec_;
use std::cell::RefCell;
use std::slice;

pub struct FreeTypeRasterizer {
    has_color: bool,
    face: RefCell<ftwrap::Face>,
    _lib: ftwrap::Library,
}
impl FontRasterizer for FreeTypeRasterizer {
    fn rasterize_glyph(&self, glyph_pos: u32, size: f64, dpi: u32) -> Fallible<RasterizedGlyph> {
        self.face.borrow_mut().set_font_size(size, dpi)?;

        let (load_flags, render_mode) = ftwrap::compute_load_flags();

        let mut face = self.face.borrow_mut();
        let ft_glyph = face.load_and_render_glyph(glyph_pos, load_flags, render_mode)?;

        let pitch = ft_glyph.bitmap.pitch.abs() as usize;
        let data = unsafe {
            slice::from_raw_parts_mut(ft_glyph.bitmap.buffer, ft_glyph.bitmap.rows as usize * pitch)
        };

        let glyph = self.rasterize_lcd(pitch, ft_glyph, data);
        Ok(glyph)
    }
}

impl FreeTypeRasterizer {
    fn rasterize_lcd(
        &self,
        pitch: usize,
        ft_glyph: &FT_GlyphSlotRec_,
        data: &[u8],
    ) -> RasterizedGlyph {
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
            bearing_x: PixelLength::new(ft_glyph.bitmap_left as f64),
            bearing_y: PixelLength::new(ft_glyph.bitmap_top as f64),
            has_color: self.has_color,
        }
    }

    pub fn from_locator(handle: &FontDataHandle) -> Fallible<Self> {
        let lib = ftwrap::Library::new()?;
        let face = lib.face_from_locator(handle)?;
        let has_color = unsafe {
            (((*face.face).face_flags as u32) & (ftwrap::FT_FACE_FLAG_COLOR as u32)) != 0
        };
        Ok(Self { _lib: lib, face: RefCell::new(face), has_color })
    }
}

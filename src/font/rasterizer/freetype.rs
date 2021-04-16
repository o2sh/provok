use crate::font::locator::FontDataHandle;
use crate::font::rasterizer::{FontMetrics, FontRasterizer};
use crate::font::{ftwrap, RasterizedGlyph};
use crate::utils::PixelLength;
use failure::Fallible;
use freetype::freetype::FT_GlyphSlotRec_;
use std::cell::RefCell;
use std::slice;

pub struct FreeTypeRasterizer {
    face: RefCell<ftwrap::Face>,
    _lib: ftwrap::Library,
    metrics: RefCell<Option<FontMetrics>>,
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

        let glyph = self.rasterize(pitch, ft_glyph, data);
        Ok(glyph)
    }

    fn metrics(&self, size: f64, dpi: u32) -> Fallible<FontMetrics> {
        let mut face = self.face.borrow_mut();
        let mut font_metrics = self.metrics.borrow_mut();
        if let Some(metrics) = *font_metrics {
            return Ok(metrics.clone());
        }

        let (cell_width, cell_height) = face.set_font_size(size, dpi)?;
        let y_scale = unsafe { (*(*face.face).size).metrics.y_scale as f64 / 65536.0 };
        let metrics = FontMetrics {
            cell_height: PixelLength::new(cell_height),
            cell_width: PixelLength::new(cell_width),
            descender: PixelLength::new(
                unsafe { (*(*face.face).size).metrics.descender as f64 } / 64.0,
            ),
            underline_thickness: PixelLength::new(
                unsafe { (*face.face).underline_thickness as f64 } * y_scale / 64.,
            ),
            underline_position: PixelLength::new(
                unsafe { (*face.face).underline_position as f64 } * y_scale / 64.,
            ),
        };

        *font_metrics = Some(metrics.clone());

        log::warn!("metrics: {:?}", metrics);

        Ok(metrics)
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

    pub fn new(handle: &FontDataHandle) -> Fallible<Self> {
        let lib = ftwrap::Library::new()?;
        let face = lib.face_from_locator(handle)?;
        Ok(Self { _lib: lib, face: RefCell::new(face), metrics: RefCell::new(None) })
    }
}

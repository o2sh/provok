use crate::bitmaps::atlas::{Atlas, Sprite};
use crate::bitmaps::{Image, Texture2d};
use crate::font::{GlyphInfo, RasterizedGlyph};
use crate::utils::PixelLength;
use anyhow::Result;
use glium::texture::SrgbTexture2d;
use glium::Display;
use std::rc::Rc;

pub struct GlyphTexture<T: Texture2d> {
    pub x_offset: PixelLength,
    pub y_offset: PixelLength,
    pub bearing_x: PixelLength,
    pub bearing_y: PixelLength,
    pub texture: Sprite<T>,
}

pub struct GlyphAtlas<T: Texture2d> {
    pub atlas: Atlas<T>,
}

impl GlyphAtlas<SrgbTexture2d> {
    pub fn new(backend: &Display, size: usize) -> Result<Self> {
        let surface = Rc::new(SrgbTexture2d::empty_with_format(
            backend,
            glium::texture::SrgbFormat::U8U8U8U8,
            glium::texture::MipmapsOption::NoMipmap,
            size as u32,
            size as u32,
        )?);
        let atlas = Atlas::new(&surface).expect("failed to create new texture atlas");

        Ok(Self { atlas })
    }
}

impl<T: Texture2d> GlyphAtlas<T> {
    pub fn load_glyph(
        &mut self,
        glyph: RasterizedGlyph,
        info: &GlyphInfo,
    ) -> Result<GlyphTexture<T>> {
        let raw_im = Image::with_rgba32(
            glyph.width as usize,
            glyph.height as usize,
            4 * glyph.width as usize,
            &glyph.data,
        );

        let bearing_x = glyph.left;
        let bearing_y = glyph.top;
        let x_offset = info.x_offset;
        let y_offset = info.y_offset;

        let texture = self.atlas.allocate(&raw_im)?;

        let glyph = GlyphTexture { texture, x_offset, y_offset, bearing_x, bearing_y };

        Ok(glyph)
    }
}

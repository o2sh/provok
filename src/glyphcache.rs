use crate::bitmaps::atlas::{Atlas, Sprite};
use crate::bitmaps::{Image, Texture2d};
use crate::font::{GlyphInfo, LoadedFont};
use crate::input::TextStyle;
use crate::utils::PixelLength;
use failure::Fallible;
use glium::texture::SrgbTexture2d;
use glium::Display;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GlyphKey {
    pub glyph_pos: u32,
    pub style: TextStyle,
}

pub struct CachedGlyph<T: Texture2d> {
    pub x_offset: PixelLength,
    pub y_offset: PixelLength,
    pub bearing_x: PixelLength,
    pub bearing_y: PixelLength,
    pub texture: Option<Sprite<T>>,
}

pub struct GlyphCache<T: Texture2d> {
    glyph_cache: HashMap<GlyphKey, Rc<CachedGlyph<T>>>,
    pub atlas: Atlas<T>,
}

impl GlyphCache<SrgbTexture2d> {
    pub fn new(backend: &Display, size: usize) -> Fallible<Self> {
        let surface = Rc::new(SrgbTexture2d::empty_with_format(
            backend,
            glium::texture::SrgbFormat::U8U8U8U8,
            glium::texture::MipmapsOption::NoMipmap,
            size as u32,
            size as u32,
        )?);
        let atlas = Atlas::new(&surface).expect("failed to create new texture atlas");

        Ok(Self { glyph_cache: HashMap::new(), atlas })
    }
}

impl<T: Texture2d> GlyphCache<T> {
    pub fn get_glyph(
        &mut self,
        font: &Rc<LoadedFont>,
        info: &GlyphInfo,
        style: &TextStyle,
    ) -> Fallible<Rc<CachedGlyph<T>>> {
        let key = GlyphKey { glyph_pos: info.glyph_pos, style: style.clone() };

        if let Some(entry) = self.glyph_cache.get(&key) {
            return Ok(Rc::clone(entry));
        }

        let glyph = self.load_glyph(info, font)?;
        self.glyph_cache.insert(key, Rc::clone(&glyph));
        Ok(glyph)
    }

    fn load_glyph(
        &mut self,
        info: &GlyphInfo,
        font: &Rc<LoadedFont>,
    ) -> Fallible<Rc<CachedGlyph<T>>> {
        let glyph = font.rasterize_glyph(info.glyph_pos)?;

        let raw_im = Image::with(
            glyph.width as usize,
            glyph.height as usize,
            3 * glyph.width as usize,
            &glyph.data,
        );

        let bearing_x = glyph.left;
        let bearing_y = glyph.top;
        let x_offset = info.x_offset;
        let y_offset = info.y_offset;

        let tex = self.atlas.allocate(&raw_im)?;

        let glyph = CachedGlyph { texture: Some(tex), x_offset, y_offset, bearing_x, bearing_y };

        Ok(Rc::new(glyph))
    }
}

use failure::Fallible;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub mod ftwrap;
pub mod hbwrap;
pub mod loader;
pub mod rasterizer;
pub mod shaper;

use crate::font::loader::parser::load_built_in_font;
use crate::font::rasterizer::FontRasterizer;
pub use crate::font::rasterizer::{FontMetrics, RasterizedGlyph};
use crate::font::shaper::FontShaper;
pub use crate::font::shaper::GlyphInfo;
use crate::input::TextStyle;

pub struct LoadedFont {
    rasterizer: Box<dyn FontRasterizer>,
    shaper: Box<dyn FontShaper>,
}

impl LoadedFont {
    pub fn shape(&self, text: &str) -> Fallible<Vec<GlyphInfo>> {
        self.shaper.shape(text)
    }

    pub fn rasterize(&self, glyph_pos: u32) -> Fallible<RasterizedGlyph> {
        self.rasterizer.rasterize(glyph_pos)
    }
}

pub struct FontConfiguration {
    fonts: RefCell<HashMap<TextStyle, Rc<LoadedFont>>>,
    font_size: f64,
    dpi: u32,
    lib: ftwrap::Library,
    faces: RefCell<Vec<ftwrap::Face>>,
}

impl FontConfiguration {
    pub fn new(font_size: f64, dpi: u32) -> Fallible<Self> {
        let lib = ftwrap::Library::new()?;
        Ok(Self {
            fonts: RefCell::new(HashMap::new()),
            font_size,
            dpi,
            lib,
            faces: RefCell::new(vec![]),
        })
    }

    pub fn get_font(&self, style: &TextStyle) -> Fallible<Rc<LoadedFont>> {
        let mut fonts = self.fonts.borrow_mut();
        let mut faces = self.faces.borrow_mut();
        if let Some(entry) = fonts.get(style) {
            return Ok(Rc::clone(entry));
        }
        let font_data_handle = load_built_in_font(&style.font_attributes)?;
        let mut face = self.lib.new_face(&font_data_handle)?;
        face.set_font_size(self.font_size, self.dpi)?;
        let shaper = shaper::new_shaper(&face)?;
        let rasterizer = rasterizer::new_rasterizer(&face)?;
        let loaded = Rc::new(LoadedFont { rasterizer, shaper });

        faces.push(face);
        fonts.insert(style.clone(), Rc::clone(&loaded));

        Ok(loaded)
    }
}

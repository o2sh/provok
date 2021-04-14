use failure::{Error, Fallible};

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub mod ftwrap;
pub mod hbwrap;
pub mod locator;
pub mod rasterizer;
pub mod shaper;

use crate::font::locator::allsorts::load_built_in_font;
use crate::font::rasterizer::FontRasterizer;
pub use crate::font::rasterizer::RasterizedGlyph;
pub use crate::font::shaper::{FontMetrics, GlyphInfo};
use crate::font::shaper::{FontShaper, FontShaperSelection};

use crate::input::{Config, TextStyle};

pub struct LoadedFont {
    rasterizer: Box<dyn FontRasterizer>,
    shaper: Box<dyn FontShaper>,
    metrics: FontMetrics,
    font_size: f64,
    dpi: u32,
}

impl LoadedFont {
    pub fn metrics(&self) -> FontMetrics {
        self.metrics
    }

    pub fn shape(&self, text: &str) -> Fallible<Vec<GlyphInfo>> {
        self.shaper.shape(text, self.font_size, self.dpi)
    }

    pub fn rasterize_glyph(&self, glyph_pos: u32) -> Fallible<RasterizedGlyph> {
        self.rasterizer.rasterize_glyph(glyph_pos, self.font_size, self.dpi)
    }
}

pub struct FontConfiguration {
    fonts: RefCell<HashMap<TextStyle, Rc<LoadedFont>>>,
    dpi_scale: RefCell<f64>,
    font_scale: RefCell<f64>,
    config: Rc<Config>,
}

impl FontConfiguration {
    pub fn new(config: Rc<Config>) -> Self {
        Self {
            fonts: RefCell::new(HashMap::new()),
            font_scale: RefCell::new(1.0),
            dpi_scale: RefCell::new(1.0),
            config,
        }
    }

    pub fn resolve_font(&self, style: &TextStyle) -> Fallible<Rc<LoadedFont>> {
        let mut fonts = self.fonts.borrow_mut();

        if let Some(entry) = fonts.get(style) {
            return Ok(Rc::clone(entry));
        }
        let font_data_handle = load_built_in_font(&style.font_attributes)?;
        let shaper = shaper::new_shaper(FontShaperSelection::Harfbuzz, &font_data_handle)?;
        let rasterizer = rasterizer::new_rasterizer(&font_data_handle)?;
        let font_size = self.config.font_size * *self.font_scale.borrow();
        let dpi = *self.dpi_scale.borrow() as u32 * self.config.dpi as u32;
        let metrics = shaper.metrics(font_size, dpi)?;

        let loaded = Rc::new(LoadedFont { rasterizer, shaper, metrics, font_size, dpi });

        fonts.insert(style.clone(), Rc::clone(&loaded));

        Ok(loaded)
    }

    pub fn font_metrics(&self, style: &TextStyle) -> Result<FontMetrics, Error> {
        let font = self.resolve_font(style)?;
        let metrics = font.metrics();
        Ok(metrics)
    }
}

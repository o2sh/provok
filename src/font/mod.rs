use crate::cell::{CellAttributes, Intensity};
use failure::{format_err, Error, Fallible};
mod hbwrap;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub mod ftwrap;
pub mod locator;
pub mod rasterizer;
pub mod shaper;

#[cfg(all(unix, not(target_os = "macos")))]
pub mod fcwrap;

use crate::font::locator::{FontLocator, FontLocatorSelection};
pub use crate::font::rasterizer::RasterizedGlyph;
use crate::font::rasterizer::{FontRasterizer, FontRasterizerSelection};
pub use crate::font::shaper::{FallbackIdx, FontMetrics, GlyphInfo};
use crate::font::shaper::{FontShaper, FontShaperSelection};

use crate::color::RgbColor;
use crate::input::{FontAttributes, Input, TextStyle};

pub struct LoadedFont {
    rasterizers: Vec<Box<dyn FontRasterizer>>,
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

    pub fn rasterize_glyph(
        &self,
        glyph_pos: u32,
        fallback: FallbackIdx,
    ) -> Fallible<RasterizedGlyph> {
        let rasterizer = self
            .rasterizers
            .get(fallback)
            .ok_or_else(|| format_err!("no such fallback index: {}", fallback))?;
        rasterizer.rasterize_glyph(glyph_pos, self.font_size, self.dpi)
    }
}

pub struct FontConfiguration {
    fonts: RefCell<HashMap<TextStyle, Rc<LoadedFont>>>,
    metrics: RefCell<Option<FontMetrics>>,
    dpi_scale: RefCell<f64>,
    font_scale: RefCell<f64>,
    input: Rc<Input>,
    locator: Box<dyn FontLocator>,
}

impl FontConfiguration {
    pub fn new(input: Rc<Input>) -> Self {
        let locator = FontLocatorSelection::get_default().new_locator();
        Self {
            fonts: RefCell::new(HashMap::new()),
            locator,
            metrics: RefCell::new(None),
            font_scale: RefCell::new(1.0),
            dpi_scale: RefCell::new(1.0),
            input,
        }
    }

    pub fn resolve_font(&self, style: &TextStyle) -> Fallible<Rc<LoadedFont>> {
        let mut fonts = self.fonts.borrow_mut();

        if let Some(entry) = fonts.get(style) {
            return Ok(Rc::clone(entry));
        }

        let attributes = style.font_with_fallback();
        let handles = self.locator.load_font(&attributes)?;
        let mut rasterizers = vec![];
        for handle in &handles {
            rasterizers.push(FontRasterizerSelection::get_default().new_rasterizer(&handle)?);
        }
        let shaper = FontShaperSelection::get_default().new_shaper(&handles)?;

        let font_size = self.input.config.font_size * *self.font_scale.borrow();
        let dpi = *self.dpi_scale.borrow() as u32 * self.input.config.dpi as u32;
        let metrics = shaper.metrics(font_size, dpi)?;

        let loaded = Rc::new(LoadedFont { rasterizers, shaper, metrics, font_size, dpi });

        fonts.insert(style.clone(), Rc::clone(&loaded));

        Ok(loaded)
    }

    pub fn default_font(&self) -> Fallible<Rc<LoadedFont>> {
        self.resolve_font(&self.input.words[0].style)
    }

    pub fn default_font_metrics(&self) -> Result<FontMetrics, Error> {
        {
            let metrics = self.metrics.borrow();
            if let Some(metrics) = metrics.as_ref() {
                return Ok(*metrics);
            }
        }

        let font = self.default_font()?;
        let metrics = font.metrics();

        *self.metrics.borrow_mut() = Some(metrics);

        Ok(metrics)
    }

    pub fn get_style(&self, attrs: &CellAttributes) -> TextStyle {
        let fonts = vec![FontAttributes::default()];
        let mut text_style = TextStyle {
            fg_color: RgbColor::default(),
            bg_color: None,
            underline: false,
            strikethrough: false,
            fonts,
        };
        if attrs.italic() {
            text_style.make_italic();
        }
        if attrs.intensity() == Intensity::Bold {
            text_style.make_bold();
        }
        text_style
    }
}

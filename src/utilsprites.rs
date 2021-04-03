use crate::bitmaps::atlas::{OutOfTextureSpace, Sprite};
use crate::bitmaps::{BitmapImage, Image, Texture2d};
use crate::cell::Underline;
use crate::color::Color;
use crate::glyphcache::GlyphCache;
use crate::renderstate::RenderMetrics;
use crate::utils::*;

pub struct UtilSprites<T: Texture2d> {
    pub white_space: Sprite<T>,
    pub single_underline: Sprite<T>,
    pub double_underline: Sprite<T>,
    pub strike_through: Sprite<T>,
    pub single_and_strike: Sprite<T>,
    pub double_and_strike: Sprite<T>,
}

impl<T: Texture2d> UtilSprites<T> {
    pub fn new(
        glyph_cache: &mut GlyphCache<T>,
        metrics: &RenderMetrics,
    ) -> Result<Self, OutOfTextureSpace> {
        let mut buffer =
            Image::new(metrics.cell_size.width as usize, metrics.cell_size.height as usize);

        let black = Color::rgba(0, 0, 0, 0);
        let white = Color::rgb(0xff, 0xff, 0xff);

        let cell_rect = Rect::new(Point::new(0, 0), metrics.cell_size);

        buffer.clear_rect(cell_rect, black);
        let white_space = glyph_cache.atlas.allocate(&buffer)?;

        let draw_single = |buffer: &mut Image| {
            for row in 0..metrics.underline_height {
                buffer.draw_line(
                    Point::new(
                        cell_rect.origin.x,
                        cell_rect.origin.y + metrics.descender_row + row,
                    ),
                    Point::new(
                        cell_rect.origin.x + metrics.cell_size.width,
                        cell_rect.origin.y + metrics.descender_row + row,
                    ),
                    white,
                    Operator::Source,
                );
            }
        };

        let draw_double = |buffer: &mut Image| {
            for row in 0..metrics.underline_height {
                buffer.draw_line(
                    Point::new(
                        cell_rect.origin.x,
                        cell_rect.origin.y + metrics.descender_row + row,
                    ),
                    Point::new(
                        cell_rect.origin.x + metrics.cell_size.width,
                        cell_rect.origin.y + metrics.descender_row + row,
                    ),
                    white,
                    Operator::Source,
                );
                buffer.draw_line(
                    Point::new(
                        cell_rect.origin.x,
                        cell_rect.origin.y + metrics.descender_plus_two + row,
                    ),
                    Point::new(
                        cell_rect.origin.x + metrics.cell_size.width,
                        cell_rect.origin.y + metrics.descender_plus_two + row,
                    ),
                    white,
                    Operator::Source,
                );
            }
        };

        let draw_strike = |buffer: &mut Image| {
            for row in 0..metrics.underline_height {
                buffer.draw_line(
                    Point::new(cell_rect.origin.x, cell_rect.origin.y + metrics.strike_row + row),
                    Point::new(
                        cell_rect.origin.x + metrics.cell_size.width,
                        cell_rect.origin.y + metrics.strike_row + row,
                    ),
                    white,
                    Operator::Source,
                );
            }
        };

        buffer.clear_rect(cell_rect, black);
        draw_single(&mut buffer);
        let single_underline = glyph_cache.atlas.allocate(&buffer)?;

        buffer.clear_rect(cell_rect, black);
        draw_double(&mut buffer);
        let double_underline = glyph_cache.atlas.allocate(&buffer)?;

        buffer.clear_rect(cell_rect, black);
        draw_strike(&mut buffer);
        let strike_through = glyph_cache.atlas.allocate(&buffer)?;

        buffer.clear_rect(cell_rect, black);
        draw_single(&mut buffer);
        draw_strike(&mut buffer);
        let single_and_strike = glyph_cache.atlas.allocate(&buffer)?;

        buffer.clear_rect(cell_rect, black);
        draw_double(&mut buffer);
        draw_strike(&mut buffer);
        let double_and_strike = glyph_cache.atlas.allocate(&buffer)?;

        Ok(Self {
            white_space,
            single_underline,
            double_underline,
            strike_through,
            single_and_strike,
            double_and_strike,
        })
    }

    pub fn select_sprite(&self, is_strike_through: bool, underline: Underline) -> &Sprite<T> {
        match (is_strike_through, underline) {
            (false, Underline::None) => &self.white_space,
            (false, Underline::Single) => &self.single_underline,
            (false, Underline::Double) => &self.double_underline,
            (true, Underline::None) => &self.strike_through,
            (true, Underline::Single) => &self.single_and_strike,
            (true, Underline::Double) => &self.double_and_strike,
        }
    }
}

use crate::font::ftwrap;
use crate::font::hbwrap as harfbuzz;
use crate::font::locator::FontDataHandle;
use crate::font::shaper::{FontMetrics, FontShaper, GlyphInfo};
use crate::utils::PixelLength;
use failure::Fallible;
use std::cell::{RefCell, RefMut};

#[derive(Clone)]
struct Info<'a> {
    cluster: usize,
    codepoint: harfbuzz::hb_codepoint_t,
    pos: &'a harfbuzz::hb_glyph_position_t,
}

fn make_glyphinfo(info: &Info) -> GlyphInfo {
    GlyphInfo {
        glyph_pos: info.codepoint,
        cluster: info.cluster as u32,
        x_advance: PixelLength::new(f64::from(info.pos.x_advance) / 64.0),
        y_advance: PixelLength::new(f64::from(info.pos.y_advance) / 64.0),
        x_offset: PixelLength::new(f64::from(info.pos.x_offset) / 64.0),
        y_offset: PixelLength::new(f64::from(info.pos.y_offset) / 64.0),
    }
}

struct FontPair {
    face: ftwrap::Face,
    font: harfbuzz::Font,
}

pub struct HarfbuzzShaper {
    handle: FontDataHandle,
    font: RefCell<Option<FontPair>>,
    lib: ftwrap::Library,
    metrics: RefCell<Option<FontMetrics>>,
}

impl HarfbuzzShaper {
    pub fn new(handle: &FontDataHandle) -> Fallible<Self> {
        let lib = ftwrap::Library::new()?;
        Ok(Self {
            font: RefCell::new(None),
            handle: handle.clone(),
            lib,
            metrics: RefCell::new(None),
        })
    }

    fn load_font_pair(&self) -> Fallible<RefMut<FontPair>> {
        let mut opt_pair = self.font.borrow_mut();
        if opt_pair.is_none() {
            let face = self.lib.face_from_locator(&self.handle)?;
            let mut font = harfbuzz::Font::new(face.face);
            let (load_flags, _) = ftwrap::compute_load_flags();
            font.set_load_flags(load_flags);
            *opt_pair = Some(FontPair { face, font });
        }

        Ok(RefMut::map(opt_pair, |opt_pair| opt_pair.as_mut().unwrap()))
    }

    fn do_shape(
        &self,
        s: &str,
        hb_script: u32,
        hb_direction: u32,
        hb_lang: &str,
    ) -> Fallible<Vec<GlyphInfo>> {
        let features = vec![
            harfbuzz::feature_from_string("kern")?,
            harfbuzz::feature_from_string("liga")?,
            harfbuzz::feature_from_string("clig")?,
        ];
        let mut buf = harfbuzz::Buffer::new()?;
        buf.set_script(hb_script);
        buf.set_direction(hb_direction);
        buf.set_language(harfbuzz::language_from_string(hb_lang)?);
        buf.add_str(s);

        let mut pair = self.load_font_pair()?;
        pair.font.shape(&mut buf, features.as_slice());

        let hb_infos = buf.glyph_infos();
        let positions = buf.glyph_positions();

        let mut cluster = Vec::new();

        for (i, info) in hb_infos.iter().enumerate() {
            let info = Info {
                cluster: info.cluster as usize,
                codepoint: info.codepoint,
                pos: &positions[i],
            };
            let glyph = make_glyphinfo(&info);
            cluster.push(glyph);
        }

        Ok(cluster)
    }
}

impl FontShaper for HarfbuzzShaper {
    fn shape(
        &self,
        text: &str,
        hb_script: u32,
        hb_direction: u32,
        hb_lang: &str,
    ) -> Fallible<Vec<GlyphInfo>> {
        let result = self.do_shape(text, hb_script, hb_direction, hb_lang);
        result
    }

    fn metrics(&self, size: f64, dpi: u32) -> Fallible<FontMetrics> {
        let mut pair = self.load_font_pair()?;
        let mut font_metrics = self.metrics.borrow_mut();
        if let Some(metrics) = *font_metrics {
            return Ok(metrics.clone());
        }

        let (cell_width, cell_height) = pair.face.set_font_size(size, dpi)?;
        let y_scale = unsafe { (*(*pair.face.face).size).metrics.y_scale as f64 / 65536.0 };
        let metrics = FontMetrics {
            cell_height: PixelLength::new(cell_height),
            cell_width: PixelLength::new(cell_width),
            descender: PixelLength::new(
                unsafe { (*(*pair.face.face).size).metrics.descender as f64 } / 64.0,
            ),
            underline_thickness: PixelLength::new(
                unsafe { (*pair.face.face).underline_thickness as f64 } * y_scale / 64.,
            ),
            underline_position: PixelLength::new(
                unsafe { (*pair.face.face).underline_position as f64 } * y_scale / 64.,
            ),
        };

        *font_metrics = Some(metrics.clone());

        log::warn!("metrics: {:?}", metrics);

        Ok(metrics)
    }
}

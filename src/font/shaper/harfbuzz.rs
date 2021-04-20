use crate::font::ftwrap;
use crate::font::hbwrap as harfbuzz;
use crate::font::loader::FontDataHandle;
use crate::font::shaper::{FontShaper, GlyphInfo};
use crate::utils::PixelLength;
use failure::Fallible;
use std::cell::RefCell;

#[derive(Clone)]
struct Info<'a> {
    codepoint: harfbuzz::hb_codepoint_t,
    pos: &'a harfbuzz::hb_glyph_position_t,
}

fn make_glyphinfo(info: &Info) -> GlyphInfo {
    GlyphInfo {
        glyph_pos: info.codepoint,
        x_advance: PixelLength::new(f64::from(info.pos.x_advance) / 64.0),
        y_advance: PixelLength::new(f64::from(info.pos.y_advance) / 64.0),
        x_offset: PixelLength::new(f64::from(info.pos.x_offset) / 64.0),
        y_offset: PixelLength::new(f64::from(info.pos.y_offset) / 64.0),
    }
}

pub struct HarfbuzzShaper {
    font: RefCell<harfbuzz::Font>,
    _lib: ftwrap::Library,
}

impl FontShaper for HarfbuzzShaper {
    fn shape(&self, text: &str) -> Fallible<Vec<GlyphInfo>> {
        let features = vec![
            harfbuzz::feature_from_string("kern")?,
            harfbuzz::feature_from_string("liga")?,
            harfbuzz::feature_from_string("clig")?,
        ];
        let mut buf = harfbuzz::Buffer::new()?;
        buf.add_str(text);

        buf.guess_segment_properties();
        let mut font = self.font.borrow_mut();
        font.shape(&mut buf, features.as_slice());

        let hb_infos = buf.glyph_infos();
        let positions = buf.glyph_positions();

        let mut cluster = Vec::new();

        for (i, info) in hb_infos.iter().enumerate() {
            let info = Info { codepoint: info.codepoint, pos: &positions[i] };
            let glyph = make_glyphinfo(&info);
            cluster.push(glyph);
        }

        Ok(cluster)
    }
}

impl HarfbuzzShaper {
    pub fn new(font_data_handle: &FontDataHandle, font_size: f64, dpi: u32) -> Fallible<Self> {
        let lib = ftwrap::Library::new()?;
        let mut face = lib.face_from_locator(&font_data_handle)?;
        face.set_font_size(font_size, dpi)?;
        let mut font = harfbuzz::Font::new(face.face);
        let (load_flags, _) = ftwrap::compute_load_flags();
        font.set_load_flags(load_flags);
        Ok(Self { _lib: lib, font: RefCell::new(font) })
    }
}

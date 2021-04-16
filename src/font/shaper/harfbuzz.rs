use crate::font::ftwrap;
use crate::font::hbwrap as harfbuzz;
use crate::font::locator::FontDataHandle;
use crate::font::shaper::{FontShaper, GlyphInfo};
use crate::utils::PixelLength;
use failure::Fallible;
use std::cell::{RefCell, RefMut};

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
    handle: FontDataHandle,
    font: RefCell<Option<harfbuzz::Font>>,
    lib: ftwrap::Library,
}

impl HarfbuzzShaper {
    pub fn new(handle: &FontDataHandle) -> Fallible<Self> {
        let lib = ftwrap::Library::new()?;
        Ok(Self { font: RefCell::new(None), handle: handle.clone(), lib })
    }

    fn load_font(&self) -> Fallible<RefMut<harfbuzz::Font>> {
        let mut opt_pair = self.font.borrow_mut();
        if opt_pair.is_none() {
            let face = self.lib.face_from_locator(&self.handle)?;
            let mut font = harfbuzz::Font::new(face.face);
            let (load_flags, _) = ftwrap::compute_load_flags();
            font.set_load_flags(load_flags);
            *opt_pair = Some(font);
        }

        Ok(RefMut::map(opt_pair, |opt_pair| opt_pair.as_mut().unwrap()))
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
        let features = vec![
            harfbuzz::feature_from_string("kern")?,
            harfbuzz::feature_from_string("liga")?,
            harfbuzz::feature_from_string("clig")?,
        ];
        let mut buf = harfbuzz::Buffer::new()?;
        buf.set_script(hb_script);
        buf.set_direction(hb_direction);
        buf.set_language(harfbuzz::language_from_string(hb_lang)?);
        buf.add_str(text);

        let mut font = self.load_font()?;
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

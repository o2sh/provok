use crate::font::ftwrap;
use crate::font::hbwrap as harfbuzz;
use crate::font::locator::unicode_column_width;
use crate::font::locator::FontDataHandle;
use crate::font::shaper::{FontMetrics, FontShaper, GlyphInfo};
use crate::utils::PixelLength;
use failure::Fallible;
use std::cell::{RefCell, RefMut};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Clone)]
struct Info<'a> {
    cluster: usize,
    len: usize,
    codepoint: harfbuzz::hb_codepoint_t,
    pos: &'a harfbuzz::hb_glyph_position_t,
}

impl<'a> std::fmt::Debug for Info<'a> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        fmt.debug_struct("Info")
            .field("cluster", &self.cluster)
            .field("len", &self.len)
            .field("codepoint", &self.codepoint)
            .finish()
    }
}

fn make_glyphinfo(text: &str, info: &Info) -> GlyphInfo {
    let num_cells = unicode_column_width(text) as u8;
    GlyphInfo {
        #[cfg(debug_assertions)]
        text: text.into(),
        num_cells,
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

#[derive(Debug)]
struct NoMoreFallbacksError {
    text: String,
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

    fn do_shape(&self, s: &str, font_size: f64, dpi: u32) -> Fallible<Vec<GlyphInfo>> {
        let features = vec![
            harfbuzz::feature_from_string("kern")?,
            harfbuzz::feature_from_string("liga")?,
            harfbuzz::feature_from_string("clig")?,
        ];
        let mut buf = harfbuzz::Buffer::new()?;
        buf.set_script(harfbuzz::HB_SCRIPT_LATIN);
        buf.set_direction(harfbuzz::HB_DIRECTION_LTR);
        buf.set_language(harfbuzz::language_from_string("en")?);
        buf.add_str(s);
        buf.set_cluster_level(harfbuzz::HB_BUFFER_CLUSTER_LEVEL_MONOTONE_GRAPHEMES);

        let mut pair = self.load_font_pair()?;
        let (width, _height) = pair.face.set_font_size(font_size, dpi)?;
        let cell_width = width;
        pair.font.shape(&mut buf, Some(features.as_slice()));

        let hb_infos = buf.glyph_infos();
        let positions = buf.glyph_positions();

        let mut cluster = Vec::new();

        let mut info_clusters: Vec<Vec<Info>> = vec![];
        let mut info_iter = hb_infos.iter().enumerate().peekable();
        while let Some((i, info)) = info_iter.next() {
            let next_pos =
                info_iter.peek().map(|(_, info)| info.cluster as usize).unwrap_or(s.len());

            let info = Info {
                cluster: info.cluster as usize,
                len: next_pos - info.cluster as usize,
                codepoint: info.codepoint,
                pos: &positions[i],
            };

            if let Some(ref mut cluster) = info_clusters.last_mut() {
                if cluster.last().unwrap().cluster == info.cluster {
                    cluster.push(info);
                    continue;
                }
                if info.codepoint == 0 {
                    let prior = cluster.last_mut().unwrap();
                    if prior.codepoint == 0 {
                        prior.len = next_pos - prior.cluster;
                        continue;
                    }
                }
            }
            info_clusters.push(vec![info]);
        }

        for infos in &info_clusters {
            let cluster_len: usize = infos.iter().map(|info| info.len).sum();
            let cluster_start = infos.first().unwrap().cluster;
            let substr = &s[cluster_start..cluster_start + cluster_len];

            let mut next_idx = 0;
            for info in infos.iter() {
                if info.pos.x_advance == 0 {
                    continue;
                }

                let nom_width =
                    ((f64::from(info.pos.x_advance) / 64.0) / cell_width).ceil() as usize;

                let len;
                if nom_width == 0 || !substr.is_char_boundary(next_idx + nom_width) {
                    let remainder = &substr[next_idx..];
                    if let Some(g) = remainder.graphemes(true).next() {
                        len = g.len();
                    } else {
                        len = remainder.len();
                    }
                } else {
                    len = nom_width;
                }

                let glyph = if len > 0 {
                    let text = &substr[next_idx..next_idx + len];
                    make_glyphinfo(text, info)
                } else {
                    make_glyphinfo("__", info)
                };

                if glyph.x_advance != PixelLength::new(0.0) {
                    cluster.push(glyph);
                }

                next_idx += len;
            }
        }

        Ok(cluster)
    }
}

impl FontShaper for HarfbuzzShaper {
    fn shape(&self, text: &str, size: f64, dpi: u32) -> Fallible<Vec<GlyphInfo>> {
        let result = self.do_shape(text, size, dpi);
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

        println!("metrics: {:?}", metrics);

        Ok(metrics)
    }
}

use crate::cell::unicode_column_width;
use crate::font::ftwrap;
use crate::font::hbwrap as harfbuzz;
use crate::font::locator::FontDataHandle;
use crate::font::shaper::{FallbackIdx, FontMetrics, FontShaper, GlyphInfo};
use crate::utils::PixelLength;
use failure::{bail, Fallible};
use log::error;
use ordered_float::NotNan;
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
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

fn make_glyphinfo(text: &str, font_idx: usize, info: &Info) -> GlyphInfo {
    let num_cells = unicode_column_width(text) as u8;
    GlyphInfo {
        #[cfg(debug_assertions)]
        text: text.into(),
        num_cells,
        font_idx,
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

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
struct MetricsKey {
    font_idx: usize,
    size: NotNan<f64>,
    dpi: u32,
}

pub struct HarfbuzzShaper {
    handles: Vec<FontDataHandle>,
    fonts: Vec<RefCell<Option<FontPair>>>,
    lib: ftwrap::Library,
    metrics: RefCell<HashMap<MetricsKey, FontMetrics>>,
}

#[derive(Debug)]
struct NoMoreFallbacksError {
    text: String,
}

fn make_question_string(s: &str) -> String {
    let len = s.graphemes(true).count();
    let mut result = String::new();
    let c = if !is_question_string(s) { std::char::REPLACEMENT_CHARACTER } else { '?' };
    for _ in 0..len {
        result.push(c);
    }
    result
}

fn is_question_string(s: &str) -> bool {
    for c in s.chars() {
        if c != std::char::REPLACEMENT_CHARACTER {
            return false;
        }
    }
    true
}

impl HarfbuzzShaper {
    pub fn new(handles: &[FontDataHandle]) -> Fallible<Self> {
        let lib = ftwrap::Library::new()?;
        let handles = handles.to_vec();
        let mut fonts = vec![];
        for _ in 0..handles.len() {
            fonts.push(RefCell::new(None));
        }
        Ok(Self { fonts, handles, lib, metrics: RefCell::new(HashMap::new()) })
    }

    fn load_fallback(&self, font_idx: FallbackIdx) -> Fallible<Option<RefMut<FontPair>>> {
        if font_idx >= self.handles.len() {
            return Ok(None);
        }
        match self.fonts.get(font_idx) {
            None => Ok(None),
            Some(opt_pair) => {
                let mut opt_pair = opt_pair.borrow_mut();
                if opt_pair.is_none() {
                    let face = self.lib.face_from_locator(&self.handles[font_idx])?;
                    let mut font = harfbuzz::Font::new(face.face);
                    let (load_flags, _) = ftwrap::compute_load_flags();
                    font.set_load_flags(load_flags);
                    *opt_pair = Some(FontPair { face, font });
                }

                Ok(Some(RefMut::map(opt_pair, |opt_pair| opt_pair.as_mut().unwrap())))
            }
        }
    }

    fn do_shape(
        &self,
        font_idx: FallbackIdx,
        s: &str,
        font_size: f64,
        dpi: u32,
    ) -> Fallible<Vec<GlyphInfo>> {
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

        let cell_width;

        {
            match self.load_fallback(font_idx)? {
                #[allow(clippy::float_cmp)]
                Some(mut pair) => {
                    let (width, _height) = pair.face.set_font_size(font_size, dpi)?;
                    cell_width = width;
                    pair.font.shape(&mut buf, Some(features.as_slice()));
                }
                None => {
                    bail!("No more fallbacks while shaping {}", s);
                }
            }
        }

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

            let incomplete = infos.iter().find(|info| info.codepoint == 0).is_some();

            if incomplete {
                let mut shape = match self.do_shape(font_idx + 1, substr, font_size, dpi) {
                    Ok(shape) => Ok(shape),
                    Err(e) => {
                        error!("{:?} for {:?}", e, substr);
                        self.do_shape(0, &make_question_string(substr), font_size, dpi)
                    }
                }?;

                for mut info in &mut shape {
                    info.cluster += cluster_start as u32;
                }
                cluster.append(&mut shape);
                continue;
            }

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
                    make_glyphinfo(text, font_idx, info)
                } else {
                    make_glyphinfo("__", font_idx, info)
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
    fn shape(&self, text: &str, size: f64, dpi: u32, _: bool) -> Fallible<Vec<GlyphInfo>> {
        let result = self.do_shape(0, text, size, dpi);
        result
    }

    fn metrics_for_idx(&self, font_idx: usize, size: f64, dpi: u32) -> Fallible<FontMetrics> {
        let mut pair = self
            .load_fallback(font_idx)?
            .ok_or_else(|| format_err!("unable to load font idx {}!?", font_idx))?;

        let key = MetricsKey { font_idx, size: NotNan::new(size).unwrap(), dpi };
        if let Some(metrics) = self.metrics.borrow().get(&key) {
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

        self.metrics.borrow_mut().insert(key, metrics.clone());

        log::trace!("metrics: {:?}", metrics);

        Ok(metrics)
    }

    fn metrics(&self, size: f64, dpi: u32) -> Fallible<FontMetrics> {
        let theoretical_height = size * dpi as f64 / 72.0;
        let mut metrics_idx = 0;
        while let Ok(Some(mut pair)) = self.load_fallback(metrics_idx) {
            let (_, cell_height) = pair.face.set_font_size(size, dpi)?;
            let diff = (theoretical_height - cell_height).abs();
            let factor = diff / theoretical_height;
            if factor < 2.0 {
                break;
            }
            log::trace!(
                "skip idx {} because diff={} factor={} theoretical_height={} cell_height={}",
                metrics_idx,
                diff,
                factor,
                theoretical_height,
                cell_height
            );
            metrics_idx += 1;
        }

        self.metrics_for_idx(metrics_idx, size, dpi)
    }
}

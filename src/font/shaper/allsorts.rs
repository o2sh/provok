use crate::font::locator::parser::{MaybeShaped, ParsedFont};
use crate::font::locator::FontDataHandle;
use crate::font::shaper::{FallbackIdx, FontMetrics, FontShaper, GlyphInfo};
use failure::Fallible;

pub struct AllsortsShaper {
    fonts: Vec<Option<ParsedFont>>,
}

impl AllsortsShaper {
    pub fn new(handles: &[FontDataHandle]) -> Fallible<Self> {
        let mut fonts = vec![];
        let mut success = false;
        for handle in handles {
            match ParsedFont::from_locator(handle) {
                Ok(font) => {
                    fonts.push(Some(font));
                    success = true;
                }
                Err(_) => {
                    fonts.push(None);
                }
            }
        }
        if !success {
            bail!("failed to load any fonts in this fallback set!?");
        }
        Ok(Self { fonts })
    }

    #[allow(clippy::too_many_arguments)]
    fn shape_into(
        &self,
        font_index: FallbackIdx,
        s: &str,
        slice_index: usize,
        script: u32,
        lang: u32,
        font_size: f64,
        dpi: u32,
        results: &mut Vec<GlyphInfo>,
    ) -> Fallible<()> {
        let font = match self.fonts.get(font_index) {
            Some(Some(font)) => font,
            Some(None) => {
                return self.shape_into(
                    font_index + 1,
                    s,
                    slice_index,
                    script,
                    lang,
                    font_size,
                    dpi,
                    results,
                );
            }
            None => {
                let mut alt_text = String::new();
                for _ in s.chars() {
                    alt_text.push('?');
                }
                if alt_text == s {
                    return Err(format_err!("could not fallback to ? character"));
                }
                return self.shape_into(
                    0,
                    &alt_text,
                    slice_index,
                    script,
                    lang,
                    font_size,
                    dpi,
                    results,
                );
            }
        };

        let first_pass =
            font.shape_text(s, slice_index, font_index, script, lang, font_size, dpi)?;

        let mut item_iter = first_pass.into_iter();
        while let Some(item) = item_iter.next() {
            match item {
                MaybeShaped::Resolved(info) => {
                    results.push(info);
                }
                MaybeShaped::Unresolved { raw, slice_start } => {
                    self.shape_into(
                        font_index + 1,
                        &raw,
                        slice_start,
                        script,
                        lang,
                        font_size,
                        dpi,
                        results,
                    )?;
                }
            }
        }

        Ok(())
    }
}

impl FontShaper for AllsortsShaper {
    fn shape(&self, text: &str, size: f64, dpi: u32, is_arabic: bool) -> Fallible<Vec<GlyphInfo>> {
        let mut results = vec![];
        let script = if is_arabic { allsorts::tag::ARAB } else { allsorts::tag::LATN };
        let lang = if is_arabic { allsorts::tag::ARAB } else { allsorts::tag::LATN };
        self.shape_into(0, text, 0, script, lang, size, dpi, &mut results)?;
        Ok(results)
    }

    fn metrics_for_idx(&self, font_idx: usize, size: f64, dpi: u32) -> Fallible<FontMetrics> {
        let font =
            self.fonts.get(font_idx).ok_or_else(|| format_err!("invalid font_idx {}", font_idx))?;
        let font =
            font.as_ref().ok_or_else(|| format_err!("failed to load font_idx {}", font_idx))?;
        Ok(font.get_metrics(size, dpi))
    }

    fn metrics(&self, size: f64, dpi: u32) -> Fallible<FontMetrics> {
        for font in &self.fonts {
            if let Some(font) = font {
                return Ok(font.get_metrics(size, dpi));
            }
        }
        bail!("no fonts available for collecting metrics!?");
    }
}

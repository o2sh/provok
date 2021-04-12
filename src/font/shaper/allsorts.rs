use crate::font::locator::parser::{MaybeShaped, ParsedFont};
use crate::font::locator::FontDataHandle;
use crate::font::shaper::{FontMetrics, FontShaper, GlyphInfo};
use failure::Fallible;

pub struct AllsortsShaper {
    font: ParsedFont,
}

impl AllsortsShaper {
    pub fn new(handle: &FontDataHandle) -> Fallible<Self> {
        let parsed_font = ParsedFont::from_locator(handle)?;

        Ok(Self { font: parsed_font })
    }

    fn shape_into(
        &self,
        s: &str,
        slice_index: usize,
        script: u32,
        lang: u32,
        font_size: f64,
        dpi: u32,
        results: &mut Vec<GlyphInfo>,
    ) -> Fallible<()> {
        let first_pass = self.font.shape_text(s, slice_index, script, lang, font_size, dpi)?;

        let mut item_iter = first_pass.into_iter();
        while let Some(item) = item_iter.next() {
            match item {
                MaybeShaped::Resolved(info) => {
                    results.push(info);
                }
                MaybeShaped::Unresolved { raw: _, slice_start: _ } => {}
            }
        }

        Ok(())
    }
}

impl FontShaper for AllsortsShaper {
    fn shape(&self, text: &str, size: f64, dpi: u32) -> Fallible<Vec<GlyphInfo>> {
        let mut results = vec![];
        let script = allsorts::tag::LATN;
        let lang = allsorts::tag::LATN;
        self.shape_into(text, 0, script, lang, size, dpi, &mut results)?;
        Ok(results)
    }

    fn metrics(&self, size: f64, dpi: u32) -> Fallible<FontMetrics> {
        Ok(self.font.get_metrics(size, dpi))
    }
}

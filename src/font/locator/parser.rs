use crate::font::locator::unicode_column_width;
use crate::font::locator::FontDataHandle;
pub use crate::font::shaper::FontMetrics;
use crate::font::shaper::GlyphInfo;
use crate::input::FontAttributes;
use crate::utils::PixelLength;
use allsorts::binary::read::{ReadScope, ReadScopeOwned};
use allsorts::font_data_impl::read_cmap_subtable;
use allsorts::gpos::{gpos_apply, Info, Placement};
use allsorts::gsub::{gsub_apply_default, GlyphOrigin, GsubFeatureMask, RawGlyph};
use allsorts::layout::{new_layout_cache, GDEFTable, LayoutCache, LayoutTable, GPOS, GSUB};
use allsorts::post::PostTable;
use allsorts::tables::cmap::{Cmap, CmapSubtable};
use allsorts::tables::{
    HeadTable, HheaTable, HmtxTable, MaxpTable, OffsetTable, OpenTypeFile, OpenTypeFont,
};
use allsorts::tag;
use failure::{Fallible, ResultExt};
use std::path::{Path, PathBuf};
use tinyvec::*;
use unicode_general_category::{get_general_category, GeneralCategory};

#[derive(Debug)]
pub enum MaybeShaped {
    Resolved(GlyphInfo),
    Unresolved { raw: String, slice_start: usize },
}

pub struct ParsedFont {
    otf: OffsetTable<'static>,
    names: Names,

    cmap_subtable: CmapSubtable<'static>,
    gpos_cache: Option<LayoutCache<GPOS>>,
    gsub_cache: Option<LayoutCache<GSUB>>,
    gdef_table: Option<GDEFTable>,
    hmtx: HmtxTable<'static>,
    post: PostTable<'static>,
    hhea: HheaTable,
    num_glyphs: u16,
    units_per_em: u16,

    _scope: ReadScopeOwned,
}

#[derive(Debug)]
pub struct Names {
    full_name: String,
    unique: Option<String>,
    family: Option<String>,
    sub_family: Option<String>,
    postscript_name: Option<String>,
}

impl Names {
    fn from_name_table_data(name_table: &[u8]) -> Fallible<Names> {
        Ok(Names {
            full_name: get_name(name_table, 4).context("full_name")?,
            unique: get_name(name_table, 3).ok(),
            family: get_name(name_table, 1).ok(),
            sub_family: get_name(name_table, 2).ok(),
            postscript_name: get_name(name_table, 6).ok(),
        })
    }
}

impl ParsedFont {
    pub fn load_built_in_font(font_attributes: &FontAttributes) -> Fallible<FontDataHandle> {
        let mut font_info = vec![];
        load_built_in_fonts(&mut font_info).ok();
        Self::match_font_info(font_attributes, font_info)
    }

    fn match_font_info(
        attr: &FontAttributes,
        mut font_info: Vec<(Names, std::path::PathBuf, FontDataHandle)>,
    ) -> Fallible<FontDataHandle> {
        font_info.sort_by_key(|(names, _, _)| names.full_name.clone());

        for (names, _, handle) in &font_info {
            if font_info_matches(attr, &names) {
                return Ok(handle.clone());
            }
        }
        failure::bail!("Could not find font");
    }

    pub fn from_locator(handle: &FontDataHandle) -> Fallible<Self> {
        let (data, index) = match handle {
            FontDataHandle::Memory { data, index, .. } => (data.to_vec(), *index),
        };

        let index = index as usize;

        let owned_scope = ReadScopeOwned::new(ReadScope::new(&data));

        let file: OpenTypeFile<'static> = unsafe {
            std::mem::transmute(
                owned_scope.scope().read::<OpenTypeFile>().context("read OpenTypeFile")?,
            )
        };

        let otf = locate_offset_table(&file, index).context("locate_offset_table")?;
        let name_table = name_table_data(&otf, &file.scope).context("name_table_data")?;
        let names =
            Names::from_name_table_data(name_table).context("Names::from_name_table_data")?;

        let head = otf
            .read_table(&file.scope, tag::HEAD)?
            .ok_or_else(|| format_err!("HEAD table missing or broken"))?
            .read::<HeadTable>()
            .context("read HeadTable")?;
        let cmap = otf
            .read_table(&file.scope, tag::CMAP)?
            .ok_or_else(|| format_err!("CMAP table missing or broken"))?
            .read::<Cmap>()
            .context("read Cmap")?;
        let cmap_subtable: CmapSubtable<'static> =
            read_cmap_subtable(&cmap)?.ok_or_else(|| format_err!("CMAP subtable not found"))?.1;

        let maxp = otf
            .read_table(&file.scope, tag::MAXP)?
            .ok_or_else(|| format_err!("MAXP table not found"))?
            .read::<MaxpTable>()
            .context("read MaxpTable")?;
        let num_glyphs = maxp.num_glyphs;

        let post = otf
            .read_table(&file.scope, tag::POST)?
            .ok_or_else(|| format_err!("POST table not found"))?
            .read::<PostTable>()
            .context("read PostTable")?;

        let hhea = otf
            .read_table(&file.scope, tag::HHEA)?
            .ok_or_else(|| format_err!("HHEA table not found"))?
            .read::<HheaTable>()
            .context("read HheaTable")?;
        let hmtx = otf
            .read_table(&file.scope, tag::HMTX)?
            .ok_or_else(|| format_err!("HMTX table not found"))?
            .read_dep::<HmtxTable>((usize::from(maxp.num_glyphs), usize::from(hhea.num_h_metrics)))
            .context("read_dep HmtxTable")?;

        let gdef_table: Option<GDEFTable> = otf
            .find_table_record(tag::GDEF)
            .map(|gdef_record| -> Fallible<GDEFTable> {
                Ok(gdef_record
                    .read_table(&file.scope)?
                    .read::<GDEFTable>()
                    .context("read GDEFTable")?)
            })
            .transpose()?;
        let opt_gpos_table = otf
            .find_table_record(tag::GPOS)
            .map(|gpos_record| -> Fallible<LayoutTable<GPOS>> {
                Ok(gpos_record
                    .read_table(&file.scope)?
                    .read::<LayoutTable<GPOS>>()
                    .context("read LayoutTable<GPOS>")?)
            })
            .transpose()?;
        let gpos_cache = opt_gpos_table.map(new_layout_cache);

        let gsub_cache = otf
            .find_table_record(tag::GSUB)
            .map(|gsub| -> Fallible<LayoutTable<GSUB>> {
                Ok(gsub
                    .read_table(&file.scope)?
                    .read::<LayoutTable<GSUB>>()
                    .context("read LayoutTable<GSUB>")?)
            })
            .transpose()?
            .map(new_layout_cache);

        Ok(Self {
            otf,
            names,
            cmap_subtable,
            post,
            hmtx,
            hhea,
            gpos_cache,
            gsub_cache,
            gdef_table,
            num_glyphs,
            units_per_em: head.units_per_em,
            _scope: owned_scope,
        })
    }

    pub fn names(&self) -> &Names {
        &self.names
    }

    pub fn glyph_index_for_char(&self, c: char) -> Fallible<u16> {
        let glyph = self
            .cmap_subtable
            .map_glyph(c as u32)
            .map_err(|e| format_err!("Error while looking up glyph {}: {}", c, e))?;

        if c == '\u{200C}' && glyph.is_none() {
            self.glyph_index_for_char(' ')
        } else {
            glyph.ok_or_else(|| format_err!("Font doesn't contain glyph for char {:?}", c))
        }
    }

    pub fn get_metrics(&self, point_size: f64, dpi: u32) -> FontMetrics {
        let pixel_scale = (dpi as f64 / 72.) * point_size / self.units_per_em as f64;
        let underline_thickness =
            PixelLength::new(self.post.header.underline_thickness as f64 * pixel_scale);
        let underline_position =
            PixelLength::new(self.post.header.underline_position as f64 * pixel_scale);
        let descender = PixelLength::new(self.hhea.descender as f64 * pixel_scale);
        let cell_height = PixelLength::new(
            (self.hhea.ascender - self.hhea.descender + self.hhea.line_gap) as f64 * pixel_scale,
        );
        log::trace!(
            "hhea: ascender={} descender={} line_gap={} \
             advance_width_max={} min_lsb={} min_rsb={} \
             x_max_extent={}",
            self.hhea.ascender,
            self.hhea.descender,
            self.hhea.line_gap,
            self.hhea.advance_width_max,
            self.hhea.min_left_side_bearing,
            self.hhea.min_right_side_bearing,
            self.hhea.x_max_extent
        );

        let mut cell_width = 0;
        for i in 0x20..0x7fu8 {
            if let Ok(glyph_index) = self.glyph_index_for_char(i as char) {
                if let Ok(h) = self.hmtx.horizontal_advance(glyph_index, self.hhea.num_h_metrics) {
                    cell_width = cell_width.max(h);
                }
            }
        }
        let cell_width =
            PixelLength::new((PixelLength::new(cell_width as f64) * pixel_scale).get().floor());

        let metrics = FontMetrics {
            cell_width,
            cell_height,
            descender,
            underline_thickness,
            underline_position,
        };

        log::trace!("metrics: {:?}", metrics);

        metrics
    }

    #[allow(clippy::too_many_arguments)]
    pub fn shape_text<T: AsRef<str>>(
        &self,
        text: T,
        slice_index: usize,
        script: u32,
        lang: u32,
        point_size: f64,
        dpi: u32,
    ) -> Fallible<Vec<MaybeShaped>> {
        #[derive(Debug)]
        enum Run {
            Unresolved(String),
            Glyphs(Vec<RawGlyph<()>>),
        }

        let mut runs = vec![];
        use allsorts::unicode::VariationSelector;
        use std::convert::TryFrom;

        let mut chars_iter = text.as_ref().chars().peekable();
        while let Some(c) = chars_iter.next() {
            match VariationSelector::try_from(c) {
                Ok(_) => {}
                Err(_) => {
                    let variation =
                        chars_iter.peek().and_then(|&next| VariationSelector::try_from(next).ok());

                    match self.glyph_index_for_char(c) {
                        Ok(glyph_index) => {
                            let glyph = RawGlyph {
                                unicodes: tiny_vec!([char; 1] => c),
                                glyph_index,
                                liga_component_pos: 0,
                                glyph_origin: GlyphOrigin::Char(c),
                                small_caps: false,
                                multi_subst_dup: false,
                                is_vert_alt: false,
                                fake_bold: false,
                                fake_italic: false,
                                variation,
                                extra_data: (),
                            };
                            if let Some(Run::Glyphs(ref mut glyphs)) = runs.last_mut() {
                                glyphs.push(glyph);
                            } else {
                                runs.push(Run::Glyphs(vec![glyph]));
                            }
                        }
                        Err(_) => {
                            match get_general_category(c) {
                                GeneralCategory::EnclosingMark => {
                                    let glyph = match runs.last_mut() {
                                        Some(Run::Glyphs(ref mut glyphs)) => glyphs.pop(),
                                        _ => None,
                                    };
                                    if let Some(glyph) = glyph {
                                        let mut s = glyph.unicodes[0].to_string();
                                        match glyph.variation {
                                            None => {}
                                            Some(VariationSelector::VS01) => s.push('\u{FE00}'),
                                            Some(VariationSelector::VS02) => s.push('\u{FE01}'),
                                            Some(VariationSelector::VS03) => s.push('\u{FE02}'),
                                            Some(VariationSelector::VS15) => s.push('\u{FE0E}'),
                                            Some(VariationSelector::VS16) => s.push('\u{FE0F}'),
                                        }
                                        runs.push(Run::Unresolved(s));
                                    }
                                }
                                _ => {}
                            }

                            if let Some(Run::Unresolved(ref mut s)) = runs.last_mut() {
                                s.push(c);
                            } else {
                                runs.push(Run::Unresolved(c.to_string()));
                            }

                            if variation.is_some() {
                                if let Some(Run::Unresolved(ref mut s)) = runs.last_mut() {
                                    s.push(*chars_iter.peek().unwrap());
                                }
                            }
                        }
                    }
                }
            }
        }

        let feature_mask = GsubFeatureMask::default();
        let mut pos = Vec::new();
        let mut cluster = slice_index;

        for run in runs {
            match run {
                Run::Unresolved(raw) => {
                    let len = raw.len();
                    pos.push(MaybeShaped::Unresolved { raw, slice_start: cluster });
                    cluster += len;
                }
                Run::Glyphs(mut glyphs) => {
                    if let Some(gsub_cache) = self.gsub_cache.as_ref() {
                        gsub_apply_default(
                            &|| vec![],
                            gsub_cache,
                            self.gdef_table.as_ref(),
                            script,
                            Some(lang),
                            feature_mask,
                            self.num_glyphs,
                            &mut glyphs,
                        )?;
                    }

                    let mut infos = Info::init_from_glyphs(self.gdef_table.as_ref(), glyphs)?;
                    if let Some(gpos_cache) = self.gpos_cache.as_ref() {
                        let kerning = true;

                        gpos_apply(
                            gpos_cache,
                            self.gdef_table.as_ref(),
                            kerning,
                            script,
                            Some(lang),
                            &mut infos,
                        )?;
                    }

                    fn reverse_engineer_glyph_text(glyph: &RawGlyph<()>) -> String {
                        glyph.unicodes.iter().collect()
                    }

                    for glyph_info in infos.into_iter() {
                        let glyph_index = glyph_info.glyph.glyph_index;

                        let horizontal_advance = i32::from(
                            self.hmtx.horizontal_advance(glyph_index, self.hhea.num_h_metrics)?,
                        );

                        let (x_advance, y_advance) = match glyph_info.placement {
                            Placement::Distance(dx, dy) => (horizontal_advance + dx, dy),
                            Placement::Anchor(_, _) | Placement::None => (horizontal_advance, 0),
                        };

                        let text = reverse_engineer_glyph_text(&glyph_info.glyph);
                        let text_len = text.len();
                        let num_cells = unicode_column_width(&text);

                        let pixel_scale =
                            (dpi as f64 / 72.) * point_size / self.units_per_em as f64;
                        let x_advance = PixelLength::new(x_advance as f64 * pixel_scale);
                        let y_advance = PixelLength::new(y_advance as f64 * pixel_scale);

                        let info = GlyphInfo {
                            #[cfg(debug_assertions)]
                            text,
                            cluster: cluster as u32,
                            num_cells: num_cells as u8,
                            glyph_pos: glyph_index as u32,
                            x_advance,
                            y_advance,
                            x_offset: PixelLength::new(0.),
                            y_offset: PixelLength::new(0.),
                        };
                        cluster += text_len;

                        pos.push(MaybeShaped::Resolved(info));
                    }
                }
            }
        }

        Ok(pos)
    }
}

pub fn font_info_matches(attr: &FontAttributes, names: &Names) -> bool {
    if let Some(fam) = names.family.as_ref() {
        if attr.family == *fam {
            match names.sub_family.as_ref().map(String::as_str) {
                Some("Italic") if attr.italic && !attr.bold => return true,
                Some("Bold") if attr.bold && !attr.italic => return true,
                Some("Bold Italic") if attr.bold && attr.italic => return true,
                Some("Medium") | Some("Regular") | None if !attr.italic && !attr.bold => {
                    return true
                }
                _ => {}
            }
        }
    }
    if attr.family == names.full_name && !attr.bold && !attr.italic {
        true
    } else {
        false
    }
}

pub fn resolve_font_from_ttc_data(attr: &FontAttributes, data: &[u8]) -> Fallible<Option<usize>> {
    let scope = allsorts::binary::read::ReadScope::new(&data);
    let file = scope.read::<OpenTypeFile>()?;

    match &file.font {
        OpenTypeFont::Single(ttf) => {
            let name_table_data = ttf
                .read_table(&file.scope, allsorts::tag::NAME)?
                .ok_or_else(|| format_err!("name table is not present"))?;

            let names = Names::from_name_table_data(name_table_data.data())?;
            if font_info_matches(attr, &names) {
                Ok(Some(0))
            } else {
                Ok(None)
            }
        }
        OpenTypeFont::Collection(ttc) => {
            for (index, offset_table_offset) in ttc.offset_tables.iter().enumerate() {
                let ttf = file.scope.offset(offset_table_offset as usize).read::<OffsetTable>()?;
                let name_table_data = ttf
                    .read_table(&file.scope, allsorts::tag::NAME)?
                    .ok_or_else(|| format_err!("name table is not present"))?;
                let names = Names::from_name_table_data(name_table_data.data())?;
                if font_info_matches(attr, &names) {
                    return Ok(Some(index));
                }
            }
            Ok(None)
        }
    }
}

fn load_built_in_fonts(font_info: &mut Vec<(Names, PathBuf, FontDataHandle)>) -> Fallible<()> {
    macro_rules! font {
        ($font:literal) => {
            (include_bytes!($font) as &'static [u8], $font)
        };
    }
    for (data, name) in &[
        font!("../../../assets/fonts/jet_brains/JetBrainsMono-BoldItalic.ttf"),
        font!("../../../assets/fonts/jet_brains/JetBrainsMono-Bold.ttf"),
        font!("../../../assets/fonts/jet_brains/JetBrainsMono-ExtraBoldItalic.ttf"),
        font!("../../../assets/fonts/jet_brains/JetBrainsMono-ExtraBold.ttf"),
        font!("../../../assets/fonts/jet_brains/JetBrainsMono-ExtraLightItalic.ttf"),
        font!("../../../assets/fonts/jet_brains/JetBrainsMono-ExtraLight.ttf"),
        font!("../../../assets/fonts/jet_brains/JetBrainsMono-Italic.ttf"),
        font!("../../../assets/fonts/jet_brains/JetBrainsMono-LightItalic.ttf"),
        font!("../../../assets/fonts/jet_brains/JetBrainsMono-Light.ttf"),
        font!("../../../assets/fonts/jet_brains/JetBrainsMono-MediumItalic.ttf"),
        font!("../../../assets/fonts/jet_brains/JetBrainsMono-Medium.ttf"),
        font!("../../../assets/fonts/jet_brains/JetBrainsMono-Regular.ttf"),
        font!("../../../assets/fonts/jet_brains/JetBrainsMono-ThinItalic.ttf"),
        font!("../../../assets/fonts/jet_brains/JetBrainsMono-Thin.ttf"),
        font!("../../../assets/fonts/noto/NotoSansArabic-Bold.ttf"),
        font!("../../../assets/fonts/noto/NotoSansArabic-Regular.ttf"),
        font!("../../../assets/fonts/noto/NotoSansJP-Bold.otf"),
        font!("../../../assets/fonts/noto/NotoSansJP-Regular.otf"),
        font!("../../../assets/fonts/noto/NotoSansThai-Bold.ttf"),
        font!("../../../assets/fonts/noto/NotoSansThai-Regular.ttf"),
        font!("../../../assets/fonts/noto/NotoSansSC-Bold.otf"),
        font!("../../../assets/fonts/noto/NotoSansSC-Regular.otf"),
    ] {
        let scope = allsorts::binary::read::ReadScope::new(&data);
        let file = scope.read::<OpenTypeFile>()?;
        let path = Path::new("memory");

        match &file.font {
            OpenTypeFont::Single(ttf) => {
                let name_table_data = ttf
                    .read_table(&file.scope, allsorts::tag::NAME)?
                    .ok_or_else(|| format_err!("name table is not present"))?;

                let names = Names::from_name_table_data(name_table_data.data())?;
                font_info.push((
                    names,
                    path.to_path_buf(),
                    FontDataHandle::Memory {
                        data: data.to_vec(),
                        index: 0,
                        name: name.to_string(),
                    },
                ));
            }
            OpenTypeFont::Collection(ttc) => {
                for (index, offset_table_offset) in ttc.offset_tables.iter().enumerate() {
                    let ttf =
                        file.scope.offset(offset_table_offset as usize).read::<OffsetTable>()?;
                    let name_table_data = ttf
                        .read_table(&file.scope, allsorts::tag::NAME)?
                        .ok_or_else(|| format_err!("name table is not present"))?;
                    let names = Names::from_name_table_data(name_table_data.data())?;
                    font_info.push((
                        names,
                        path.to_path_buf(),
                        FontDataHandle::Memory {
                            data: data.to_vec(),
                            index: index as u32,
                            name: name.to_string(),
                        },
                    ));
                }
            }
        }
    }

    Ok(())
}

fn locate_offset_table<'a>(f: &OpenTypeFile<'a>, idx: usize) -> Fallible<OffsetTable<'a>> {
    match &f.font {
        OpenTypeFont::Single(ttf) => Ok(ttf.clone()),
        OpenTypeFont::Collection(ttc) => {
            let offset_table_offset = ttc
                .offset_tables
                .read_item(idx)
                .map_err(|e| format_err!("font idx={} is not present in ttc file: {}", idx, e))?;
            let ttf = f.scope.offset(offset_table_offset as usize).read::<OffsetTable>()?;
            Ok(ttf.clone())
        }
    }
}

fn name_table_data<'a>(otf: &OffsetTable<'a>, scope: &ReadScope<'a>) -> Fallible<&'a [u8]> {
    let data = otf
        .read_table(scope, allsorts::tag::NAME)?
        .ok_or_else(|| format_err!("name table is not present"))?;
    Ok(data.data())
}

fn get_name(name_table_data: &[u8], name_id: u16) -> Fallible<String> {
    let cstr = allsorts::get_name::fontcode_get_name(name_table_data, name_id)
        .with_context(|_| format_err!("fontcode_get_name name_id:{}", name_id))?
        .ok_or_else(|| format_err!("name_id {} not found", name_id))?;
    cstr.into_string()
        .map_err(|e| format_err!("name_id {} is not representable as String: {}", name_id, e))
}

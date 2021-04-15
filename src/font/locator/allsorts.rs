use crate::font::locator::FontDataHandle;
pub use crate::font::shaper::FontMetrics;
use crate::input::FontAttributes;
use allsorts::binary::read::ReadScope;
use allsorts::font_data::FontData;
use allsorts::tables::{OffsetTable, OpenTypeData};
use failure::{Fallible, ResultExt};
use std::path::{Path, PathBuf};

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

fn get_name(name_table_data: &[u8], name_id: u16) -> Fallible<String> {
    let cstr = allsorts::get_name::fontcode_get_name(name_table_data, name_id)
        .with_context(|_| format_err!("fontcode_get_name name_id:{}", name_id))?
        .ok_or_else(|| format_err!("name_id {} not found", name_id))?;
    cstr.into_string()
        .map_err(|e| format_err!("name_id {} is not representable as String: {}", name_id, e))
}

pub fn load_built_in_font(font_attributes: &FontAttributes) -> Fallible<FontDataHandle> {
    let mut font_info = vec![];
    load_built_in_fonts(&mut font_info).ok();
    match_font_info(font_attributes, font_info)
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

fn font_info_matches(attr: &FontAttributes, names: &Names) -> bool {
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
        font!("../../../assets/fonts/amiri-regular.ttf"),
    ] {
        let scope = ReadScope::new(&data);
        let file = scope.read::<FontData<'_>>()?;
        let path = Path::new("memory");
        match &file {
            FontData::OpenType(open_type_font) => match &open_type_font.data {
                OpenTypeData::Single(ttf) => {
                    let name_table_data = ttf
                        .read_table(&open_type_font.scope, allsorts::tag::NAME)?
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
                OpenTypeData::Collection(ttc) => {
                    for (index, offset_table_offset) in ttc.offset_tables.iter().enumerate() {
                        let ttf = open_type_font
                            .scope
                            .offset(offset_table_offset as usize)
                            .read::<OffsetTable>()?;
                        let name_table_data = ttf
                            .read_table(&open_type_font.scope, allsorts::tag::NAME)?
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
            },
            _ => failure::bail!("unhandled"),
        }
    }

    Ok(())
}

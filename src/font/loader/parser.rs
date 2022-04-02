use crate::font::loader::{FontDataHandle, Names};
use crate::input::FontAttributes;
use anyhow::{bail, Result};

pub fn load_built_in_font(font_attributes: &FontAttributes) -> Result<FontDataHandle> {
    let mut font_info = vec![];
    load_built_in_fonts(&mut font_info).ok();
    match_font_info(font_attributes, font_info)
}

fn match_font_info(
    attr: &FontAttributes,
    mut font_info: Vec<(Names, FontDataHandle)>,
) -> Result<FontDataHandle> {
    font_info.sort_by_key(|(names, _)| names.full_name.clone());

    for (names, handle) in &font_info {
        if font_info_matches(attr, names) {
            return Ok(handle.clone());
        }
    }
    bail!("Could not find font");
}

fn font_info_matches(attr: &FontAttributes, names: &Names) -> bool {
    if let Some(fam) = names.family.as_ref() {
        if attr.family == *fam {
            match names.sub_family.as_deref() {
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
    attr.family == names.full_name && !attr.bold && !attr.italic
}

fn load_built_in_fonts(font_info: &mut Vec<(Names, FontDataHandle)>) -> Result<()> {
    macro_rules! font {
        ($font:literal) => {
            (include_bytes!($font) as &'static [u8], $font)
        };
    }
    for (data, name) in &[
        font!("../../../assets/fonts/noto/NotoSansArabic-Bold.ttf"),
        font!("../../../assets/fonts/noto/NotoSansArabic-Regular.ttf"),
        font!("../../../assets/fonts/noto/NotoSansSC-Bold.otf"),
        font!("../../../assets/fonts/noto/NotoSansSC-Regular.otf"),
        font!("../../../assets/fonts/noto/NotoSansDevanagari-Bold.ttf"),
        font!("../../../assets/fonts/noto/NotoSansDevanagari-Regular.ttf"),
        font!("../../../assets/fonts/noto/NotoSans-Bold.ttf"),
        font!("../../../assets/fonts/noto/NotoSans-BoldItalic.ttf"),
        font!("../../../assets/fonts/noto/NotoSans-Italic.ttf"),
        font!("../../../assets/fonts/noto/NotoSans-Regular.ttf"),
        font!("../../../assets/fonts/noto/NotoSansThai-Bold.ttf"),
        font!("../../../assets/fonts/noto/NotoSansThai-Regular.ttf"),
        font!("../../../assets/fonts/siliguri/HindSiliguri-Bold.ttf"),
        font!("../../../assets/fonts/siliguri/HindSiliguri-Regular.ttf"),
    ] {
        let face = ttf_parser::Face::from_slice(data, 0)?;
        let full_name = face
            .names()
            .into_iter()
            .find(|name| name.name_id == ttf_parser::name_id::FULL_NAME)
            .and_then(|name| name.to_string())
            .unwrap();

        let postscript_name = face
            .names()
            .into_iter()
            .find(|name| name.name_id == ttf_parser::name_id::POST_SCRIPT_NAME)
            .and_then(|name| name.to_string());

        let unique = face
            .names()
            .into_iter()
            .find(|name| name.name_id == ttf_parser::name_id::UNIQUE_ID)
            .and_then(|name| name.to_string());

        let sub_family = face
            .names()
            .into_iter()
            .find(|name| name.name_id == ttf_parser::name_id::SUBFAMILY)
            .and_then(|name| name.to_string());

        let family = face
            .names()
            .into_iter()
            .find(|name| name.name_id == ttf_parser::name_id::FAMILY)
            .and_then(|name| name.to_string());

        let names = Names { full_name, unique, family, sub_family, postscript_name };

        font_info.push((
            names,
            FontDataHandle { data: data.to_vec(), name: name.to_string(), index: 0 },
        ));
    }

    Ok(())
}

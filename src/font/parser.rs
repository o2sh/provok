use crate::font::locator::FontDataHandle;
use crate::input::FontAttributes;
use allsorts::tables::{OffsetTable, OpenTypeFile, OpenTypeFont};
use failure::Fallible;
use std::path::{Path, PathBuf};

pub struct Names {
    full_name: String,
    family: Option<String>,
    sub_family: Option<String>,
}

impl Names {
    fn from_name_table_data(name_table: &[u8]) -> Fallible<Names> {
        Ok(Names {
            full_name: get_name(name_table, 4)?,
            family: get_name(name_table, 1).ok(),
            sub_family: get_name(name_table, 2).ok(),
        })
    }
}

pub fn load_fonts(attr: &FontAttributes) -> Fallible<Vec<FontDataHandle>> {
    let mut font_info = vec![];

    load_built_in_fonts(&mut font_info).ok();

    font_info.sort_by_key(|(names, _, _)| names.full_name.clone());
    for (names, _, _) in &font_info {
        log::warn!("available font: {}", names.full_name);
    }

    let mut handles = vec![];
    for (names, path, handle) in &font_info {
        if font_info_matches(attr, &names) {
            log::warn!("Using {} from {}", names.full_name, path.display(),);
            handles.push(handle.clone());
            break;
        }
    }
    Ok(handles)
}

fn font_info_matches(attr: &FontAttributes, names: &Names) -> bool {
    if attr.font_family == names.full_name {
        true
    } else if let Some(fam) = names.family.as_ref() {
        // TODO: correctly match using family and sub-family;
        // this is a pretty rough approximation
        if attr.font_family == *fam {
            match names.sub_family.as_ref().map(String::as_str) {
                Some("Italic") if attr.italic => true,
                Some("Bold") if attr.bold => true,
                Some("Regular") | None => true,
                _ => false,
            }
        } else {
            false
        }
    } else {
        false
    }
}

fn load_built_in_fonts(font_info: &mut Vec<(Names, PathBuf, FontDataHandle)>) -> Fallible<()> {
    for data in &[
        include_bytes!("../../assets/fonts/JetBrainsMono-Bold-Italic.ttf") as &'static [u8],
        include_bytes!("../../assets/fonts/JetBrainsMono-Bold.ttf"),
        include_bytes!("../../assets/fonts/JetBrainsMono-ExtraBold-Italic.ttf"),
        include_bytes!("../../assets/fonts/JetBrainsMono-ExtraBold.ttf"),
        include_bytes!("../../assets/fonts/JetBrainsMono-ExtraLight-Italic.ttf"),
        include_bytes!("../../assets/fonts/JetBrainsMono-ExtraLight.ttf"),
        include_bytes!("../../assets/fonts/JetBrainsMono-Italic.ttf"),
        include_bytes!("../../assets/fonts/JetBrainsMono-Light-Italic.ttf"),
        include_bytes!("../../assets/fonts/JetBrainsMono-Light.ttf"),
        include_bytes!("../../assets/fonts/JetBrainsMono-Medium-Italic.ttf"),
        include_bytes!("../../assets/fonts/JetBrainsMono-Medium.ttf"),
        include_bytes!("../../assets/fonts/JetBrainsMono-Regular.ttf"),
        include_bytes!("../../assets/fonts/JetBrainsMono-SemiLight-Italic.ttf"),
        include_bytes!("../../assets/fonts/JetBrainsMono-SemiLight.ttf"),
        include_bytes!("../../assets/fonts/Cairo-Black.ttf"),
        include_bytes!("../../assets/fonts/Cairo-Bold.ttf"),
        include_bytes!("../../assets/fonts/Cairo-ExtraLight.ttf"),
        include_bytes!("../../assets/fonts/Cairo-Light.ttf"),
        include_bytes!("../../assets/fonts/Cairo-Regular.ttf"),
        include_bytes!("../../assets/fonts/Cairo-SemiBold.ttf"),
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
                    FontDataHandle::Memory { data: data.to_vec(), index: 0 },
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
                        FontDataHandle::Memory { data: data.to_vec(), index: index as u32 },
                    ));
                }
            }
        }
    }

    Ok(())
}

fn get_name(name_table_data: &[u8], name_id: u16) -> Fallible<String> {
    let cstr = allsorts::get_name::fontcode_get_name(name_table_data, name_id)?
        .ok_or_else(|| format_err!("name_id {} not found", name_id))?;
    cstr.into_string()
        .map_err(|e| format_err!("name_id {} is not representable as String: {}", name_id, e))
}

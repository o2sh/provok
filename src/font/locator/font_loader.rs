use crate::font::locator::{FontDataHandle, FontLocator};
use crate::input::FontAttributes;
use failure::Fallible;
use font_loader::system_fonts;

pub struct FontLoaderFontLocator {}

impl FontLocator for FontLoaderFontLocator {
    fn load_font(&self, attr: &FontAttributes) -> Fallible<Vec<FontDataHandle>> {
        let mut fonts = Vec::new();
        let mut font_props =
            system_fonts::FontPropertyBuilder::new().family(&attr.font_family).monospace();
        font_props = if attr.bold { font_props.bold() } else { font_props };
        font_props = if attr.italic { font_props.italic() } else { font_props };
        let font_props = font_props.build();

        if let Some((data, index)) = system_fonts::get(&font_props) {
            let handle = FontDataHandle::Memory { data, index: index as u32 };
            fonts.push(handle);
        }
        Ok(fonts)
    }
}

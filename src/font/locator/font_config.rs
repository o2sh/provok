use crate::font::fcwrap;
use crate::font::locator::{FontDataHandle, FontLocator};
use crate::input::FontAttributes;
use failure::Fallible;
use fcwrap::Pattern as FontPattern;

pub struct FontConfigFontLocator {}

impl FontLocator for FontConfigFontLocator {
    fn load_fonts(&self, attrs: &[FontAttributes]) -> Fallible<Vec<FontDataHandle>> {
        let mut fonts = vec![];
        let mut fallback = vec![];

        for attr in attrs {
            let mut pattern = FontPattern::new()?;
            pattern.family(&attr.family)?;
            if attr.bold {
                pattern.add_integer("weight", 200)?;
            }
            if attr.italic {
                pattern.add_integer("slant", 100)?;
            }
            pattern.monospace()?;
            pattern.config_substitute(fcwrap::MatchKind::Pattern)?;
            pattern.default_substitute();

            let font_list = pattern.sort(true)?;

            for (idx, pat) in font_list.iter().enumerate() {
                pattern.render_prepare(&pat)?;
                let file = pat.get_file()?;
                let handle = FontDataHandle::OnDisk { path: file.into(), index: 0 };

                if idx == 0 {
                    fonts.push(handle);
                } else {
                    fallback.push(handle);
                }
            }

            fonts.append(&mut fallback);
        }
        Ok(fonts)
    }
}

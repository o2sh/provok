use crate::font::hbwrap as harfbuzz;

macro_rules! languages {
    ($( { $hb_script:ident, $font:literal } ),* ,) => {
        pub fn get_font(hb_script: &u32 ) -> &str {
           match *hb_script{
                $( harfbuzz::$hb_script=> $font, )*
                _ => unimplemented!("font: Language {:?}", hb_script),
            }
        }
    };
}

languages! {
    { HB_SCRIPT_LATIN, "Noto Sans" },
    { HB_SCRIPT_ARABIC, "Noto Sans Arabic" },
    { HB_SCRIPT_HAN, "Noto Sans SC" },
    { HB_SCRIPT_KATAKANA, "Noto Sans JP" },
    { HB_SCRIPT_CYRILLIC, "Noto Sans" },
    { HB_SCRIPT_DEVANAGARI, "Noto Sans Devanagari" },
    { HB_SCRIPT_THAI, "Noto Sans Thai" },
    { HB_SCRIPT_BENGALI, "Hind Siliguri" },
}

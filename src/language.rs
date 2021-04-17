use crate::font::hbwrap as harfbuzz;
use lingua::Language;

macro_rules! languages {
    ($( { $name:ident, $font:literal, $hb_lang:literal, $hb_script:ident, $hb_direction:ident } ),* ,) => {
        pub fn get_font(language: &Language) -> &str {
           match language {
                $( Language::$name => $font, )*
                _ => unimplemented!("font: Language {:?}", language),
            }
        }
        pub fn get_hb_lang(language: &Language) -> &str {
           match language {
                $( Language::$name => $hb_lang, )*
                _ => unimplemented!("hb_lang: Language {:?}", language),
            }
        }
        pub fn get_hb_script(language: &Language) -> u32 {
           match language {
                $( Language::$name => harfbuzz::$hb_script, )*
                _ => unimplemented!("hb_script: Language {:?}", language),
            }
        }
        pub fn get_hb_direction(language: &Language) -> u32 {
           match language {
                $( Language::$name => harfbuzz::$hb_direction, )*
                _ => unimplemented!("hb_direction: Language {:?}", language),
            }
        }
    };
}

languages! {
    { English, "JetBrains Mono", "en", HB_SCRIPT_LATIN, HB_DIRECTION_LTR },
    { Arabic, "Noto Sans Arabic", "ar", HB_SCRIPT_ARABIC, HB_DIRECTION_RTL },
    { Chinese, "Noto Sans SC", "ch", HB_SCRIPT_HAN, HB_DIRECTION_LTR },
    { Japanese, "Noto Sans JP", "jp", HB_SCRIPT_KATAKANA, HB_DIRECTION_LTR },
    { Russian, "Noto Sans", "ru", HB_SCRIPT_CYRILLIC, HB_DIRECTION_LTR },
    { Hindi, "Noto Sans Devanagari", "hi", HB_SCRIPT_DEVANAGARI, HB_DIRECTION_LTR },
}

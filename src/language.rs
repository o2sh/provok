use lingua::Language;

macro_rules! languages {
    ($( { $name:ident, $font:literal } ),* ,) => {
        pub fn get_font(language: &Language) -> &str {
           match language {
                $( Language::$name => $font, )*
                _ => unimplemented!("font: Language {:?}", language),
            }
        }
    };
}

languages! {
    { English, "Noto Sans" },
    { Arabic, "Noto Sans Arabic" },
    { Chinese, "Noto Sans SC" },
    { Japanese, "Noto Sans JP" },
    { Russian, "Noto Sans" },
    { Hindi, "Noto Sans Devanagari" },
}

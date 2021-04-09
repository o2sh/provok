use crate::color::RgbColor;
use failure::Fallible;
use serde::Deserialize;

const FONT_FAMILY: &str = "JetBrains Mono";

#[derive(Debug, Deserialize, Clone)]
struct InputJson {
    font_size: usize,
    words: Vec<WordJson>,
}

#[derive(Debug, Deserialize, Clone)]
struct WordJson {
    text: String,
    canvas_color: String,
    fg_color: String,
    bg_color: Option<String>,
    bold: Option<bool>,
    italic: Option<bool>,
    underline: Option<bool>,
    strikethrough: Option<bool>,
}

pub struct Input {
    pub config: Config,
    pub words: Vec<Word>,
}

pub struct Config {
    pub font_size: f64,
    pub dpi: f64,
}

pub struct Word {
    pub text: String,
    pub canvas_color: RgbColor,
    pub style: TextStyle,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TextStyle {
    pub fg_color: RgbColor,
    pub bg_color: Option<RgbColor>,
    pub underline: bool,
    pub strikethrough: bool,
    pub font: FontAttributes,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct FontAttributes {
    pub family: String,
    #[serde(default)]
    pub bold: bool,
    #[serde(default)]
    pub italic: bool,
}

impl FontAttributes {
    pub fn new(family: &str, bold: bool, italic: bool) -> Self {
        Self { family: family.into(), bold, italic }
    }
}

impl Input {
    pub fn new(path: &str) -> Fallible<Self> {
        let input_json = InputJson::parse(path)?;
        let mut words: Vec<Word> = Vec::new();
        for word_json in input_json.words.iter() {
            let bg_color = if let Some(c) = &word_json.bg_color {
                Some(RgbColor::from_named_or_rgb_string(c).unwrap())
            } else {
                None
            };

            words.push(Word {
                text: String::from(&word_json.text),
                canvas_color: RgbColor::from_named_or_rgb_string(&word_json.canvas_color).unwrap(),
                style: TextStyle {
                    fg_color: RgbColor::from_named_or_rgb_string(&word_json.fg_color).unwrap(),
                    bg_color,
                    underline: word_json.underline.unwrap_or(false),
                    strikethrough: word_json.strikethrough.unwrap_or(false),
                    font: FontAttributes {
                        family: FONT_FAMILY.into(),
                        bold: word_json.bold.unwrap_or(false),
                        italic: word_json.italic.unwrap_or(false),
                    },
                },
            })
        }

        Ok(Self { config: Config { font_size: input_json.font_size as f64, dpi: 96. }, words })
    }
}

impl Default for FontAttributes {
    fn default() -> Self {
        Self { family: FONT_FAMILY.into(), bold: false, italic: false }
    }
}

impl TextStyle {
    pub fn make_bold(&mut self) {
        self.font.bold = true;
    }

    pub fn make_italic(&mut self) {
        self.font.italic = true;
    }

    pub fn font_with_fallback(&self) -> Vec<FontAttributes> {
        let mut fonts = Vec::new();
        fonts.push(self.font.clone());
        fonts.push(FontAttributes::new("Noto Sans Arabic", self.font.bold, self.font.italic));
        fonts
    }
}
impl InputJson {
    fn parse(path: &str) -> Fallible<Self> {
        let data = std::fs::read_to_string(path)?;
        let input: InputJson = serde_json::from_str(&data)?;
        Ok(input)
    }
}

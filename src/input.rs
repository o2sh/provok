use crate::color::RgbColor;
use failure::Fallible;
use serde::Deserialize;

#[cfg(target_os = "macos")]
const FONT_FAMILY: &str = "Menlo";

#[cfg(not(target_os = "macos"))]
const FONT_FAMILY: &str = "monospace";

#[derive(Debug, Deserialize, Clone)]
struct InputJson {
    font_size: usize,
    dpi: usize,
    words: Vec<WordJson>,
}

#[derive(Debug, Deserialize, Clone)]
struct WordJson {
    text: String,
    font_family: Option<String>,
    fg_color: String,
    bg_color: String,
    bold: Option<bool>,
    italic: Option<bool>,
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
    pub style: TextStyle,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TextStyle {
    pub fg_color: RgbColor,
    pub bg_color: RgbColor,
    pub font_attributes: FontAttributes,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FontAttributes {
    pub font_family: String,
    pub bold: bool,
    pub italic: bool,
}

impl Input {
    pub fn new(path: &str) -> Fallible<Self> {
        let input_json = InputJson::parse(path)?;
        let mut words: Vec<Word> = Vec::new();
        for word_json in input_json.words.iter() {
            let font_family = if let Some(f) = &word_json.font_family { f } else { FONT_FAMILY };
            words.push(Word {
                text: String::from(&word_json.text),
                style: TextStyle {
                    fg_color: RgbColor::from_named_or_rgb_string(&word_json.fg_color).unwrap(),
                    bg_color: RgbColor::from_named_or_rgb_string(&word_json.bg_color).unwrap(),
                    font_attributes: FontAttributes {
                        font_family: font_family.into(),
                        bold: word_json.bold.unwrap_or(false),
                        italic: word_json.italic.unwrap_or(false),
                    },
                },
            })
        }

        Ok(Self {
            config: Config { font_size: input_json.font_size as f64, dpi: input_json.dpi as f64 },
            words,
        })
    }
}

impl InputJson {
    fn parse(path: &str) -> Fallible<Self> {
        let data = std::fs::read_to_string(path)?;
        let input: InputJson = serde_json::from_str(&data)?;
        Ok(input)
    }
}
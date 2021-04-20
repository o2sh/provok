use crate::color::RgbColor;
use crate::font::hbwrap as harfbuzz;
use crate::language;
use failure::Fallible;
use serde::Deserialize;

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

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub font_size: f64,
    pub dpi: u32,
}

pub struct Word {
    pub text: String,
    pub canvas_color: RgbColor,
    pub style: TextStyle,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TextStyle {
    pub fg_color: RgbColor,
    pub font_attributes: FontAttributes,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct FontAttributes {
    pub family: String,
    pub bold: bool,
    pub italic: bool,
}

impl Input {
    pub fn new(path: &str) -> Fallible<Self> {
        let input_json = InputJson::parse(path)?;
        let mut words: Vec<Word> = Vec::new();
        for word_json in input_json.words.iter() {
            let mut buf = harfbuzz::Buffer::new()?;
            buf.add_str(&word_json.text);
            buf.guess_segment_properties();
            let hb_script = buf.get_script();
            words.push(Word {
                text: String::from(&word_json.text),
                canvas_color: RgbColor::from_named_or_rgb_string(&word_json.canvas_color).unwrap(),
                style: TextStyle {
                    fg_color: RgbColor::from_named_or_rgb_string(&word_json.fg_color).unwrap(),
                    font_attributes: FontAttributes {
                        family: language::get_font(&hb_script).into(),
                        bold: word_json.bold.unwrap_or(false),
                        italic: word_json.italic.unwrap_or(false),
                    },
                },
            });
        }

        Ok(Self { config: Config { font_size: input_json.font_size as f64, dpi: 96 }, words })
    }
}

impl InputJson {
    fn parse(path: &str) -> Fallible<Self> {
        let data = std::fs::read_to_string(path)?;
        let input: InputJson = serde_json::from_str(&data)?;
        Ok(input)
    }
}

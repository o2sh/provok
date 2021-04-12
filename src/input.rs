use crate::color::RgbColor;
use failure::Fallible;
use lazy_static::lazy_static;
use lingua::{Language, LanguageDetector, LanguageDetectorBuilder};
use serde::Deserialize;
use std::collections::HashMap;

const DEFAULT_FONT_FAMILY: &str = "JetBrains Mono";

lazy_static! {
    static ref LANG_TO_FONT: HashMap<Language, &'static str> = {
        let mut m = HashMap::new();
        m.insert(Language::Arabic, "Noto Sans Arabic");
        m.insert(Language::Chinese, "Noto Sans JP");
        m.insert(Language::Japanese, "Noto Sans JP");
        m.insert(Language::Thai, "Noto Sans Thai");
        m
    };
}

pub enum Direction {
    LTR,
    RTL,
}

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
    pub dpi: f64,
}

pub struct Word {
    pub text: String,
    pub canvas_color: RgbColor,
    pub direction: Direction,
    pub style: TextStyle,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TextStyle {
    pub fg_color: RgbColor,
    pub bg_color: Option<RgbColor>,
    pub underline: bool,
    pub strikethrough: bool,
    pub font_attributes: FontAttributes,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct FontAttributes {
    pub family: String,
    #[serde(default)]
    pub bold: bool,
    #[serde(default)]
    pub italic: bool,
}

impl Input {
    pub fn new(path: &str) -> Fallible<Self> {
        let languages =
            vec![Language::Arabic, Language::Thai, Language::Chinese, Language::Japanese];
        let detector: LanguageDetector =
            LanguageDetectorBuilder::from_languages(&languages).build();
        let input_json = InputJson::parse(path)?;
        let mut words: Vec<Word> = Vec::new();
        for word_json in input_json.words.iter() {
            let bg_color = if let Some(c) = &word_json.bg_color {
                Some(RgbColor::from_named_or_rgb_string(c).unwrap())
            } else {
                None
            };

            let (family, direction) =
                if let Some(lang) = detector.detect_language_of(&word_json.text) {
                    let direction =
                        if lang == Language::Arabic { Direction::RTL } else { Direction::LTR };
                    (LANG_TO_FONT[&lang], direction)
                } else {
                    (DEFAULT_FONT_FAMILY, Direction::LTR)
                };
            words.push(Word {
                text: String::from(&word_json.text),
                direction,
                canvas_color: RgbColor::from_named_or_rgb_string(&word_json.canvas_color).unwrap(),
                style: TextStyle {
                    fg_color: RgbColor::from_named_or_rgb_string(&word_json.fg_color).unwrap(),
                    bg_color,
                    underline: word_json.underline.unwrap_or(false),
                    strikethrough: word_json.strikethrough.unwrap_or(false),
                    font_attributes: FontAttributes {
                        family: family.into(),
                        bold: word_json.bold.unwrap_or(false),
                        italic: word_json.italic.unwrap_or(false),
                    },
                },
            })
        }

        Ok(Self { config: Config { font_size: input_json.font_size as f64, dpi: 96. }, words })
    }
}

impl InputJson {
    fn parse(path: &str) -> Fallible<Self> {
        let data = std::fs::read_to_string(path)?;
        let input: InputJson = serde_json::from_str(&data)?;
        Ok(input)
    }
}

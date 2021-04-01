use crate::cell::{Blink, Intensity, Underline};
use crate::color::RgbColor;
use serde_derive::*;

#[cfg(target_os = "macos")]
const FONT_FAMILY: &str = "Menlo";

#[cfg(not(target_os = "macos"))]
const FONT_FAMILY: &str = "monospace";

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    #[serde(default = "default_font_size")]
    pub font_size: f64,
    #[serde(default = "default_dpi")]
    pub dpi: f64,
    #[serde(default)]
    pub font: TextStyle,
    #[serde(default)]
    pub font_rules: Vec<StyleRule>,
}

fn default_font_size() -> f64 {
    17.0
}

fn default_dpi() -> f64 {
    96.0
}

impl Default for Config {
    fn default() -> Self {
        Self {
            font_size: default_font_size(),
            dpi: default_dpi(),
            font: TextStyle::default(),
            font_rules: Vec::new(),
        }
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct FontAttributes {
    pub family: String,
    pub bold: Option<bool>,
    pub italic: Option<bool>,
}

impl Default for FontAttributes {
    fn default() -> Self {
        Self { family: FONT_FAMILY.into(), bold: None, italic: None }
    }
}

fn empty_font_attributes() -> Vec<FontAttributes> {
    Vec::new()
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct TextStyle {
    #[serde(default = "empty_font_attributes")]
    pub font: Vec<FontAttributes>,
    pub foreground: Option<RgbColor>,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self { foreground: None, font: vec![FontAttributes::default()] }
    }
}

impl TextStyle {
    fn make_bold(&self) -> Self {
        Self {
            foreground: self.foreground,
            font: self
                .font
                .iter()
                .map(|attr| {
                    let mut attr = attr.clone();
                    attr.bold = Some(true);
                    attr
                })
                .collect(),
        }
    }

    fn make_italic(&self) -> Self {
        Self {
            foreground: self.foreground,
            font: self
                .font
                .iter()
                .map(|attr| {
                    let mut attr = attr.clone();
                    attr.italic = Some(true);
                    attr
                })
                .collect(),
        }
    }

    #[cfg_attr(feature = "cargo-clippy", allow(clippy::let_and_return))]
    pub fn font_with_fallback(&self) -> Vec<FontAttributes> {
        #[allow(unused_mut)]
        let mut font = self.font.clone();

        if font.is_empty() {
            font.push(FontAttributes::default());
        }

        #[cfg(target_os = "macos")]
        font.push(FontAttributes { family: "Apple Color Emoji".into(), bold: None, italic: None });
        #[cfg(target_os = "macos")]
        font.push(FontAttributes { family: "Apple Symbols".into(), bold: None, italic: None });
        #[cfg(target_os = "macos")]
        font.push(FontAttributes { family: "Zapf Dingbats".into(), bold: None, italic: None });
        #[cfg(target_os = "macos")]
        font.push(FontAttributes { family: "Apple LiGothic".into(), bold: None, italic: None });
        #[cfg(not(target_os = "macos"))]
        font.push(FontAttributes { family: "Noto Color Emoji".into(), bold: None, italic: None });

        font
    }
}

#[derive(Debug, Default, Deserialize, Clone)]
pub struct StyleRule {
    pub intensity: Option<Intensity>,
    pub underline: Option<Underline>,
    pub italic: Option<bool>,
    pub blink: Option<Blink>,
    pub reverse: Option<bool>,
    pub strikethrough: Option<bool>,
    pub invisible: Option<bool>,
    pub font: TextStyle,
}

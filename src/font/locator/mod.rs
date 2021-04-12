#![allow(dead_code)]
use crate::cell::grapheme_column_width;
use crate::input::FontAttributes;
use failure::Fallible;
use unicode_segmentation::UnicodeSegmentation;

pub mod parser;

#[derive(Clone, PartialEq, Eq)]
pub enum FontDataHandle {
    Memory { name: String, data: Vec<u8>, index: u32 },
}

pub trait FontLocator {
    fn load_fonts(&self, font_attributes: &[FontAttributes]) -> Fallible<Vec<FontDataHandle>>;
}

pub fn unicode_column_width(s: &str) -> usize {
    s.graphemes(true).map(grapheme_column_width).sum()
}

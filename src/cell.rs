use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use unicode_width::UnicodeWidthStr;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Cell {
    text: SmallVec<[u8; 4]>,
}

impl Default for Cell {
    fn default() -> Self {
        Cell::new(' ')
    }
}

impl Cell {
    pub fn new(text: char) -> Self {
        let len = text.len_utf8();
        let mut storage = SmallVec::with_capacity(len);
        unsafe {
            storage.set_len(len);
        }
        text.encode_utf8(&mut storage);

        Self { text: storage }
    }

    pub fn new_grapheme(text: &str) -> Self {
        let storage = SmallVec::from_slice(text.as_bytes());

        Self { text: storage }
    }

    pub fn str(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(&self.text) }
    }

    pub fn width(&self) -> usize {
        grapheme_column_width(self.str())
    }
}

pub fn grapheme_column_width(s: &str) -> usize {
    use xi_unicode::EmojiExt;
    for c in s.chars() {
        if c.is_emoji_modifier_base() || c.is_emoji_modifier() {
            return 2;
        }
    }
    UnicodeWidthStr::width(s)
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[repr(u16)]
pub enum Intensity {
    Normal = 0,
    Bold = 1,
    Half = 2,
}

impl Into<bool> for Intensity {
    fn into(self) -> bool {
        self == Intensity::Bold
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[repr(u16)]
pub enum Underline {
    None = 0,
    Single = 1,
    Double = 2,
}

impl Into<bool> for Underline {
    fn into(self) -> bool {
        self != Underline::None
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[repr(u16)]
pub enum Blink {
    None = 0,
    Slow = 1,
    Rapid = 2,
}

impl Into<bool> for Blink {
    fn into(self) -> bool {
        self != Blink::None
    }
}

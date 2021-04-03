use crate::color::ColorAttribute;
use crate::input::TextStyle;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use std::mem;
use unicode_width::UnicodeWidthStr;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Cell {
    text: SmallVec<[u8; 4]>,
    attrs: CellAttributes,
}

impl Default for Cell {
    fn default() -> Self {
        Cell::new(' ', CellAttributes::default())
    }
}

impl Cell {
    fn nerf_control_char(text: &mut SmallVec<[u8; 4]>) {
        if text.is_empty() {
            text.push(b' ');
            return;
        }

        if text.as_slice() == [b'\r', b'\n'] {
            text.remove(1);
            text[0] = b' ';
            return;
        }

        if text.len() != 1 {
            return;
        }

        if text[0] < 0x20 || text[0] == 0x7f {
            text[0] = b' ';
        }
    }

    pub fn new(text: char, attrs: CellAttributes) -> Self {
        let len = text.len_utf8();
        let mut storage = SmallVec::with_capacity(len);
        unsafe {
            storage.set_len(len);
        }
        text.encode_utf8(&mut storage);
        Self::nerf_control_char(&mut storage);

        Self { text: storage, attrs }
    }

    pub fn new_grapheme(text: &str, attrs: CellAttributes) -> Self {
        let mut storage = SmallVec::from_slice(text.as_bytes());
        Self::nerf_control_char(&mut storage);

        Self { text: storage, attrs }
    }

    pub fn str(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(&self.text) }
    }

    pub fn width(&self) -> usize {
        grapheme_column_width(self.str())
    }

    pub fn attrs(&self) -> &CellAttributes {
        &self.attrs
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

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct CellAttributes {
    attributes: u16,
    pub foreground: ColorAttribute,
    pub background: ColorAttribute,
}

macro_rules! bitfield {
    ($getter:ident, $setter:ident, $bitnum:expr) => {
        #[inline]
        pub fn $getter(&self) -> bool {
            (self.attributes & (1 << $bitnum)) == (1 << $bitnum)
        }

        #[inline]
        pub fn $setter(&mut self, value: bool) -> &mut Self {
            let attr_value = if value { 1 << $bitnum } else { 0 };
            self.attributes = (self.attributes & !(1 << $bitnum)) | attr_value;
            self
        }
    };

    ($getter:ident, $setter:ident, $bitmask:expr, $bitshift:expr) => {
        #[inline]
        pub fn $getter(&self) -> u16 {
            (self.attributes >> $bitshift) & $bitmask
        }

        #[inline]
        pub fn $setter(&mut self, value: u16) -> &mut Self {
            let clear = !($bitmask << $bitshift);
            let attr_value = (value & $bitmask) << $bitshift;
            self.attributes = (self.attributes & clear) | attr_value;
            self
        }
    };

    ($getter:ident, $setter:ident, $enum:ident, $bitmask:expr, $bitshift:expr) => {
        #[inline]
        pub fn $getter(&self) -> $enum {
            unsafe { mem::transmute(((self.attributes >> $bitshift) & $bitmask) as u16) }
        }

        #[inline]
        pub fn $setter(&mut self, value: $enum) -> &mut Self {
            let value = value as u16;
            let clear = !($bitmask << $bitshift);
            let attr_value = (value & $bitmask) << $bitshift;
            self.attributes = (self.attributes & clear) | attr_value;
            self
        }
    };
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

impl CellAttributes {
    pub fn from_text_style(text_style: &TextStyle) -> Self {
        let mut attr = CellAttributes::default();
        if text_style.font_attributes.bold {
            attr.set_intensity(Intensity::Bold);
        }
        if text_style.font_attributes.italic {
            attr.set_italic(true);
        }

        attr.set_foreground(ColorAttribute::TrueColorWithDefaultFallback(text_style.fg_color));
        attr.set_background(ColorAttribute::TrueColorWithDefaultFallback(text_style.bg_color));
        attr
    }
    bitfield!(intensity, set_intensity, Intensity, 0b11, 0);
    bitfield!(underline, set_underline, Underline, 0b11, 2);
    bitfield!(blink, set_blink, Blink, 0b11, 4);
    bitfield!(italic, set_italic, 6);
    bitfield!(reverse, set_reverse, 7);
    bitfield!(strikethrough, set_strikethrough, 8);
    bitfield!(invisible, set_invisible, 9);
    bitfield!(wrapped, set_wrapped, 10);

    pub fn set_foreground<C: Into<ColorAttribute>>(&mut self, foreground: C) -> &mut Self {
        self.foreground = foreground.into();
        self
    }

    pub fn set_background<C: Into<ColorAttribute>>(&mut self, background: C) -> &mut Self {
        self.background = background.into();
        self
    }
}

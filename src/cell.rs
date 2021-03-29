use crate::color::ColorAttribute;
use smallvec::SmallVec;
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

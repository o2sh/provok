use crate::utils::Operator;
use palette::{Blend, LinSrgb, LinSrgba, Srgb, Srgba};
use serde::{self, Deserialize, Deserializer, Serialize, Serializer};
use std::result::Result;

#[derive(Clone)]
pub struct Palette256(pub [RgbColor; 256]);

#[derive(Clone)]
pub struct ColorPalette {
    pub colors: Palette256,
    pub foreground: RgbColor,
    pub background: RgbColor,
    pub cursor_fg: RgbColor,
    pub cursor_bg: RgbColor,
    pub selection_fg: RgbColor,
    pub selection_bg: RgbColor,
}

impl Default for ColorPalette {
    fn default() -> ColorPalette {
        let mut colors = [RgbColor::default(); 256];

        static ANSI: [RgbColor; 16] = [
            RgbColor { red: 0x00, green: 0x00, blue: 0x00 },
            RgbColor { red: 0xcc, green: 0x55, blue: 0x55 },
            RgbColor { red: 0x55, green: 0xcc, blue: 0x55 },
            RgbColor { red: 0xcd, green: 0xcd, blue: 0x55 },
            RgbColor { red: 0x54, green: 0x55, blue: 0xcb },
            RgbColor { red: 0xcc, green: 0x55, blue: 0xcc },
            RgbColor { red: 0x7a, green: 0xca, blue: 0xca },
            RgbColor { red: 0xcc, green: 0xcc, blue: 0xcc },
            RgbColor { red: 0x55, green: 0x55, blue: 0x55 },
            RgbColor { red: 0xff, green: 0x55, blue: 0x55 },
            RgbColor { red: 0x55, green: 0xff, blue: 0x55 },
            RgbColor { red: 0xff, green: 0xff, blue: 0x55 },
            RgbColor { red: 0x55, green: 0x55, blue: 0xff },
            RgbColor { red: 0xff, green: 0x55, blue: 0xff },
            RgbColor { red: 0x55, green: 0xff, blue: 0xff },
            RgbColor { red: 0xff, green: 0xff, blue: 0xff },
        ];

        colors[0..16].copy_from_slice(&ANSI);

        static RAMP6: [u8; 6] = [0x00, 0x33, 0x66, 0x99, 0xCC, 0xFF];
        for idx in 0..216 {
            let blue = RAMP6[idx % 6];
            let green = RAMP6[idx / 6 % 6];
            let red = RAMP6[idx / 6 / 6 % 6];

            colors[16 + idx] = RgbColor { red, green, blue };
        }

        static GREYS: [u8; 24] = [
            0x08, 0x12, 0x1c, 0x26, 0x30, 0x3a, 0x44, 0x4e, 0x58, 0x62, 0x6c, 0x76, 0x80, 0x8a,
            0x94, 0x9e, 0xa8, 0xb2, /* Grey70 */
            0xbc, 0xc6, 0xd0, 0xda, 0xe4, 0xee,
        ];

        for idx in 0..24 {
            let grey = GREYS[idx];
            colors[232 + idx] = RgbColor::new(grey, grey, grey);
        }

        let foreground = colors[249];
        let background = colors[AnsiColor::Black as usize];

        let cursor_bg = RgbColor::new(0x52, 0xad, 0x70);
        let cursor_fg = colors[AnsiColor::Black as usize];

        let selection_fg = colors[AnsiColor::Black as usize];
        let selection_bg = RgbColor::new(0xff, 0xfa, 0xcd);

        ColorPalette {
            colors: Palette256(colors),
            foreground,
            background,
            cursor_fg,
            cursor_bg,
            selection_fg,
            selection_bg,
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]

pub enum AnsiColor {
    Black = 0,
    Maroon,
    Green,
    Olive,
    Navy,
    Purple,
    Teal,
    Silver,
    Grey,
    Red,
    Lime,
    Yellow,
    Blue,
    Fuschia,
    Aqua,
    White,
}

impl From<AnsiColor> for u8 {
    fn from(col: AnsiColor) -> u8 {
        col as u8
    }
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
pub struct RgbColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl RgbColor {
    pub fn new(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }

    pub fn to_rgb_string(self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.red, self.green, self.blue)
    }

    pub fn from_rgb_str(s: &str) -> Option<RgbColor> {
        if s.as_bytes()[0] == b'#' && s.len() == 7 {
            let mut chars = s.chars().skip(1);

            macro_rules! digit {
                () => {{
                    let hi = match chars.next().unwrap().to_digit(16) {
                        Some(v) => (v as u8) << 4,
                        None => return None,
                    };
                    let lo = match chars.next().unwrap().to_digit(16) {
                        Some(v) => v as u8,
                        None => return None,
                    };
                    hi | lo
                }};
            }
            Some(Self::new(digit!(), digit!(), digit!()))
        } else {
            None
        }
    }

    pub fn from_named(name: &str) -> Option<RgbColor> {
        palette::named::from_str(&name.to_ascii_lowercase()).map(|color| {
            let color = Srgb::<u8>::from_format(color);
            Self::new(color.red, color.green, color.blue)
        })
    }

    pub fn from_named_or_rgb_string(s: &str) -> Option<Self> {
        RgbColor::from_rgb_str(&s).or_else(|| RgbColor::from_named(&s))
    }
}

impl Serialize for RgbColor {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = self.to_rgb_string();
        s.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for RgbColor {
    fn deserialize<D>(deserializer: D) -> Result<RgbColor, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        RgbColor::from_named_or_rgb_string(&s)
            .ok_or_else(|| format!("unknown color name: {}", s))
            .map_err(serde::de::Error::custom)
    }
}

pub type PaletteIndex = u8;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum ColorAttribute {
    TrueColorWithPaletteFallback(RgbColor, PaletteIndex),
    TrueColorWithDefaultFallback(RgbColor),
    PaletteIndex(PaletteIndex),
    Default,
}

impl Default for ColorAttribute {
    fn default() -> Self {
        ColorAttribute::Default
    }
}

lazy_static::lazy_static! {
    static ref SRGB_TO_F32_TABLE: [f32;256] = generate_srgb8_to_linear_f32_table();
    static ref F32_TO_U8_TABLE: [u32;104] = generate_linear_f32_to_srgb8_table();
}

fn generate_srgb8_to_linear_f32_table() -> [f32; 256] {
    let mut table = [0.; 256];
    for (val, entry) in table.iter_mut().enumerate() {
        let c = (val as f32) / 255.0;
        *entry = if c < 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) };
    }
    table
}

#[allow(clippy::unreadable_literal)]
fn generate_linear_f32_to_srgb8_table() -> [u32; 104] {
    [
        0x0073000d, 0x007a000d, 0x0080000d, 0x0087000d, 0x008d000d, 0x0094000d, 0x009a000d,
        0x00a1000d, 0x00a7001a, 0x00b4001a, 0x00c1001a, 0x00ce001a, 0x00da001a, 0x00e7001a,
        0x00f4001a, 0x0101001a, 0x010e0033, 0x01280033, 0x01410033, 0x015b0033, 0x01750033,
        0x018f0033, 0x01a80033, 0x01c20033, 0x01dc0067, 0x020f0067, 0x02430067, 0x02760067,
        0x02aa0067, 0x02dd0067, 0x03110067, 0x03440067, 0x037800ce, 0x03df00ce, 0x044600ce,
        0x04ad00ce, 0x051400ce, 0x057b00c5, 0x05dd00bc, 0x063b00b5, 0x06970158, 0x07420142,
        0x07e30130, 0x087b0120, 0x090b0112, 0x09940106, 0x0a1700fc, 0x0a9500f2, 0x0b0f01cb,
        0x0bf401ae, 0x0ccb0195, 0x0d950180, 0x0e56016e, 0x0f0d015e, 0x0fbc0150, 0x10630143,
        0x11070264, 0x1238023e, 0x1357021d, 0x14660201, 0x156601e9, 0x165a01d3, 0x174401c0,
        0x182401af, 0x18fe0331, 0x1a9602fe, 0x1c1502d2, 0x1d7e02ad, 0x1ed4028d, 0x201a0270,
        0x21520256, 0x227d0240, 0x239f0443, 0x25c003fe, 0x27bf03c4, 0x29a10392, 0x2b6a0367,
        0x2d1d0341, 0x2ebe031f, 0x304d0300, 0x31d105b0, 0x34a80555, 0x37520507, 0x39d504c5,
        0x3c37048b, 0x3e7c0458, 0x40a8042a, 0x42bd0401, 0x44c20798, 0x488e071e, 0x4c1c06b6,
        0x4f76065d, 0x52a50610, 0x55ac05cc, 0x5892058f, 0x5b590559, 0x5e0c0a23, 0x631c0980,
        0x67db08f6, 0x6c55087f, 0x70940818, 0x74a007bd, 0x787d076c, 0x7c330723,
    ]
}

#[allow(clippy::unreadable_literal)]
const ALMOST_ONE: u32 = 0x3f7fffff;
#[allow(clippy::unreadable_literal)]
const MINVAL: u32 = (127 - 13) << 23;

fn linear_f32_to_srgb8_using_table(f: f32) -> u8 {
    let minval = f32::from_bits(MINVAL);
    let almost_one = f32::from_bits(ALMOST_ONE);

    let f = if f < minval {
        minval
    } else if f > almost_one {
        almost_one
    } else {
        f
    };

    let f_bits = f.to_bits();
    let tab = unsafe { *F32_TO_U8_TABLE.get_unchecked(((f_bits - MINVAL) >> 20) as usize) };
    let bias = (tab >> 16) << 9;
    let scale = tab & 0xffff;

    let t = (f_bits >> 12) & 0xff;

    ((bias + scale * t) >> 16) as u8
}

fn srgb8_to_linear_f32(val: u8) -> f32 {
    unsafe { *SRGB_TO_F32_TABLE.get_unchecked(val as usize) }
}

#[derive(Copy, Clone, Debug)]
pub struct Color(pub u32);

impl From<LinSrgba> for Color {
    #[inline]
    #[allow(clippy::many_single_char_names)]
    fn from(s: LinSrgba) -> Color {
        let r = linear_f32_to_srgb8_using_table(s.red);
        let g = linear_f32_to_srgb8_using_table(s.green);
        let b = linear_f32_to_srgb8_using_table(s.blue);
        let a = linear_f32_to_srgb8_using_table(s.alpha);
        Color::rgba(r, g, b, a)
    }
}

impl From<Srgb> for Color {
    #[inline]
    fn from(s: Srgb) -> Color {
        let b: Srgb<u8> = s.into_format();
        let b = b.into_components();
        Color::rgb(b.0, b.1, b.2)
    }
}

impl From<Srgba> for Color {
    #[inline]
    fn from(s: Srgba) -> Color {
        let b: Srgba<u8> = s.into_format();
        let b = b.into_components();
        Color::rgba(b.0, b.1, b.2, b.3)
    }
}

impl From<Color> for LinSrgb {
    #[inline]
    fn from(c: Color) -> LinSrgb {
        let c = c.as_rgba();
        LinSrgb::new(srgb8_to_linear_f32(c.0), srgb8_to_linear_f32(c.1), srgb8_to_linear_f32(c.2))
    }
}

impl From<Color> for LinSrgba {
    #[inline]
    fn from(c: Color) -> LinSrgba {
        let c = c.as_rgba();
        LinSrgba::new(
            srgb8_to_linear_f32(c.0),
            srgb8_to_linear_f32(c.1),
            srgb8_to_linear_f32(c.2),
            srgb8_to_linear_f32(c.3),
        )
    }
}

impl From<Color> for Srgb {
    #[inline]
    fn from(c: Color) -> Srgb {
        let c = c.as_rgba();
        let s = Srgb::<u8>::new(c.0, c.1, c.2);
        s.into_format()
    }
}

impl From<Color> for Srgba {
    #[inline]
    fn from(c: Color) -> Srgba {
        let c = c.as_rgba();
        let s = Srgba::<u8>::new(c.0, c.1, c.2, c.3);
        s.into_format()
    }
}

impl Color {
    #[inline]
    pub fn rgb(red: u8, green: u8, blue: u8) -> Color {
        Color::rgba(red, green, blue, 0xff)
    }

    #[inline]
    pub fn rgba(red: u8, green: u8, blue: u8, alpha: u8) -> Color {
        #[allow(clippy::cast_lossless)]
        let word = (blue as u32) << 24 | (green as u32) << 16 | (red as u32) << 8 | alpha as u32;
        Color(word.to_be())
    }

    #[inline]
    pub fn as_rgba(self) -> (u8, u8, u8, u8) {
        let host = u32::from_be(self.0);
        ((host >> 8) as u8, (host >> 16) as u8, (host >> 24) as u8, (host & 0xff) as u8)
    }

    #[inline]
    pub fn to_tuple_rgba(self) -> (f32, f32, f32, f32) {
        let c: Srgba = self.into();
        c.into_format().into_components()
    }

    #[inline]
    pub fn composite(self, dest: Color, operator: Operator) -> Color {
        match operator {
            Operator::Over => {
                let src: LinSrgba = self.into();
                let dest: LinSrgba = dest.into();
                src.over(dest).into()
            }
            Operator::Source => self,
            Operator::Multiply => {
                let src: LinSrgba = self.into();
                let dest: LinSrgba = dest.into();
                let result: Color = src.multiply(dest).into();
                result
            }
            Operator::MultiplyThenOver(ref tint) => {
                let src: LinSrgba = self.into();
                let tint: LinSrgba = (*tint).into();
                let mut tinted = src.multiply(tint);

                tinted.alpha = src.alpha;

                let dest: LinSrgba = dest.into();
                tinted.over(dest).into()
            }
        }
    }
}

pub fn rgbcolor_to_color(color: RgbColor) -> Color {
    Color::rgba(color.red, color.green, color.blue, 0xff)
}

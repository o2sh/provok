use crate::color::Color;

#[derive(Debug, Clone, Copy)]
pub enum Operator {
    Over,
    Source,
    Multiply,
    MultiplyThenOver(Color),
}

#[derive(Debug, Clone, Copy)]
pub struct Dimensions {
    pub pixel_width: usize,
    pub pixel_height: usize,
    pub dpi: usize,
}

pub struct PixelUnit;
pub type PixelLength = euclid::Length<f64, PixelUnit>;
pub type IntPixelLength = isize;
pub type Point = euclid::Point2D<isize, PixelUnit>;
pub type Rect = euclid::Rect<isize, PixelUnit>;
pub type Size = euclid::Size2D<isize, PixelUnit>;

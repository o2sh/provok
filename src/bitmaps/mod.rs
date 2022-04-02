use crate::color::Color;
use crate::utils::Rect;
use glium::texture::SrgbTexture2d;

pub mod atlas;

pub struct TextureUnit;
pub type TextureCoord = euclid::Point2D<f32, TextureUnit>;
pub type TextureRect = euclid::Rect<f32, TextureUnit>;
pub type TextureSize = euclid::Size2D<f32, TextureUnit>;

pub trait Texture2d {
    fn write(&self, rect: Rect, im: &dyn BitmapImage);
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn to_texture_coords(&self, coords: Rect) -> TextureRect {
        let coords = coords.to_f32();
        let width = self.width() as f32;
        let height = self.height() as f32;
        TextureRect::new(
            TextureCoord::new(coords.min_x() / width, coords.min_y() / height),
            TextureSize::new(coords.size.width / width, coords.size.height / height),
        )
    }
}

impl Texture2d for SrgbTexture2d {
    fn write(&self, rect: Rect, im: &dyn BitmapImage) {
        let (im_width, im_height) = im.image_dimensions();

        let source = glium::texture::RawImage2d {
            data: im
                .pixels()
                .iter()
                .map(|&p| {
                    let (r, g, b, a) = Color(p).as_rgba();

                    fn conv(v: u8) -> u8 {
                        let f = (v as f32) / 255.;
                        let c = if f <= 0.0031308 {
                            f * 12.92
                        } else {
                            f.powf(1.0 / 2.4) * 1.055 - 0.055
                        };
                        (c * 255.).ceil() as u8
                    }
                    Color::rgba(conv(b), conv(g), conv(r), conv(a)).0
                })
                .collect(),
            width: im_width as u32,
            height: im_height as u32,
            format: glium::texture::ClientFormat::U8U8U8U8,
        };

        SrgbTexture2d::write(
            self,
            glium::Rect {
                left: rect.min_x() as u32,
                bottom: rect.min_y() as u32,
                width: rect.size.width as u32,
                height: rect.size.height as u32,
            },
            source,
        )
    }

    fn width(&self) -> usize {
        SrgbTexture2d::width(self) as usize
    }

    fn height(&self) -> usize {
        SrgbTexture2d::height(self) as usize
    }
}

pub trait BitmapImage {
    unsafe fn pixel_data(&self) -> *const u8;

    fn image_dimensions(&self) -> (usize, usize);

    fn pixels(&self) -> &[u32] {
        let (width, height) = self.image_dimensions();
        unsafe {
            let first = self.pixel_data() as *const u32;
            std::slice::from_raw_parts(first, width * height)
        }
    }
}

pub struct Image {
    data: Vec<u8>,
    width: usize,
    height: usize,
}

impl Image {
    pub fn new(width: usize, height: usize) -> Image {
        let size = height * width * 4;
        let mut data = vec![0; size];
        data.resize(size, 0);
        Image { data, width, height }
    }

    pub fn with_rgba32(width: usize, height: usize, stride: usize, data: &[u8]) -> Image {
        let mut image = Image::new(width, height);
        for y in 0..height {
            let src_offset = y * stride;
            let dest_offset = y * width * 4;
            for x in 0..width {
                let red = data[src_offset + (x * 4)];
                let green = data[src_offset + (x * 4) + 1];
                let blue = data[src_offset + (x * 4) + 2];
                let alpha = data[src_offset + (x * 4) + 3];
                image.data[dest_offset + (x * 4)] = blue;
                image.data[dest_offset + (x * 4) + 1] = green;
                image.data[dest_offset + (x * 4) + 2] = red;
                image.data[dest_offset + (x * 4) + 3] = alpha;
            }
        }
        image
    }
}

impl BitmapImage for Image {
    unsafe fn pixel_data(&self) -> *const u8 {
        self.data.as_ptr()
    }

    fn image_dimensions(&self) -> (usize, usize) {
        (self.width, self.height)
    }
}

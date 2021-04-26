use freetype;

pub use harfbuzz::*;
use harfbuzz_sys as harfbuzz;

use failure::{ensure, Error};
use std::mem;
use std::slice;

extern "C" {
    pub fn hb_ft_font_create_referenced(face: freetype::freetype::FT_Face) -> *mut hb_font_t;
}

pub fn feature_from_string(s: &str) -> Result<hb_feature_t, Error> {
    unsafe {
        let mut feature = mem::zeroed();
        ensure!(
            hb_feature_from_string(s.as_ptr() as *const i8, s.len() as i32, &mut feature as *mut _,)
                != 0,
            "failed to create feature from {}",
            s
        );
        Ok(feature)
    }
}

pub struct Font {
    font: *mut hb_font_t,
}

impl Drop for Font {
    fn drop(&mut self) {
        unsafe {
            hb_font_destroy(self.font);
        }
    }
}

impl Font {
    pub fn new(face: freetype::freetype::FT_Face) -> Font {
        Font { font: unsafe { hb_ft_font_create_referenced(face as _) } }
    }

    pub fn shape(&mut self, buf: &mut Buffer, features: &[hb_feature_t]) {
        unsafe { hb_shape(self.font, buf.buf, features.as_ptr(), features.len() as u32) }
    }
}

pub struct Buffer {
    buf: *mut hb_buffer_t,
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            hb_buffer_destroy(self.buf);
        }
    }
}

impl Buffer {
    pub fn new() -> Result<Buffer, Error> {
        let buf = unsafe { hb_buffer_create() };
        ensure!(unsafe { hb_buffer_allocation_successful(buf) } != 0, "hb_buffer_create failed");
        Ok(Buffer { buf })
    }

    pub fn guess_segment_properties(&mut self) {
        unsafe { hb_buffer_guess_segment_properties(self.buf) };
    }
    pub fn get_script(&self) -> hb_script_t {
        unsafe { hb_buffer_get_script(self.buf) }
    }

    pub fn add_utf8(&mut self, buf: &[u8]) {
        unsafe {
            hb_buffer_add_utf8(
                self.buf,
                buf.as_ptr() as *const i8,
                buf.len() as i32,
                0,
                buf.len() as i32,
            );
        }
    }

    pub fn add_str(&mut self, s: &str) {
        self.add_utf8(s.as_bytes())
    }

    pub fn glyph_infos(&self) -> &[hb_glyph_info_t] {
        unsafe {
            let mut len: u32 = 0;
            let info = hb_buffer_get_glyph_infos(self.buf, &mut len as *mut _);
            slice::from_raw_parts(info, len as usize)
        }
    }

    pub fn glyph_positions(&self) -> &[hb_glyph_position_t] {
        unsafe {
            let mut len: u32 = 0;
            let pos = hb_buffer_get_glyph_positions(self.buf, &mut len as *mut _);
            slice::from_raw_parts(pos, len as usize)
        }
    }
}

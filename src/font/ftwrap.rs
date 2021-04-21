use crate::font::loader::FontDataHandle;
use failure::{format_err, Fallible, ResultExt};
pub use freetype::freetype::*;
use std::ptr;

#[inline]
pub fn succeeded(error: FT_Error) -> bool {
    error == freetype::freetype::FT_Err_Ok as FT_Error
}

fn ft_result<T>(err: FT_Error, t: T) -> Fallible<T> {
    if succeeded(err) {
        Ok(t)
    } else {
        Err(format_err!("FreeType error {:?} 0x{:x}", err, err))
    }
}

fn render_mode_to_load_target(render_mode: FT_Render_Mode) -> u32 {
    (render_mode as u32) & 15 << 16
}

pub fn compute_load_flags() -> (i32, FT_Render_Mode) {
    let render = FT_Render_Mode::FT_RENDER_MODE_LCD;

    let flags = render_mode_to_load_target(render);

    (flags as i32, render)
}

impl Clone for Face {
    fn clone(&self) -> Self {
        let err = unsafe { FT_Reference_Library(self.lib) };
        if err != freetype::freetype::FT_Err_Ok as FT_Error {
            panic!("Failed to reference library");
        }
        let err = unsafe { FT_Reference_Face(self.face) };
        if err != freetype::freetype::FT_Err_Ok as FT_Error {
            panic!("Failed to reference face");
        }
        Face { lib: self.lib, face: self.face, bytes: self.bytes.clone() }
    }
}

pub struct Face {
    lib: FT_Library,
    pub face: FT_Face,
    bytes: Vec<u8>,
}

impl Drop for Face {
    fn drop(&mut self) {
        let err = unsafe { FT_Done_Face(self.face) };
        if err != freetype::freetype::FT_Err_Ok as FT_Error {
            panic!("Failed to drop face");
        }
        let err = unsafe { FT_Done_Library(self.lib) };
        if err != freetype::freetype::FT_Err_Ok as FT_Error {
            panic!("Failed to drop library")
        }
    }
}

impl Face {
    pub fn set_font_size(&mut self, point_size: f64, dpi: u32) -> Fallible<()> {
        let size = (point_size * 64.0) as FT_F26Dot6;
        self.set_char_size(size, 0, dpi, 0)
    }

    fn set_char_size(
        &mut self,
        char_width: FT_F26Dot6,
        char_height: FT_F26Dot6,
        horz_resolution: FT_UInt,
        vert_resolution: FT_UInt,
    ) -> Fallible<()> {
        ft_result(
            unsafe {
                FT_Set_Char_Size(
                    self.face,
                    char_width,
                    char_height,
                    horz_resolution,
                    vert_resolution,
                )
            },
            (),
        )
    }

    pub fn load_and_render_glyph(
        &mut self,
        glyph_index: FT_UInt,
        load_flags: FT_Int32,
        render_mode: FT_Render_Mode,
    ) -> Fallible<&FT_GlyphSlotRec_> {
        unsafe {
            let res = FT_Load_Glyph(self.face, glyph_index, load_flags);
            let slot = ft_result(res, &mut *(*self.face).glyph)?;
            ft_result(FT_Render_Glyph(slot, render_mode), slot)
        }
    }
}

pub struct Library {
    lib: FT_Library,
}

impl Drop for Library {
    fn drop(&mut self) {
        unsafe {
            FT_Done_FreeType(self.lib);
        }
    }
}

impl Library {
    pub fn new() -> Fallible<Library> {
        let mut lib = ptr::null_mut();
        let res = unsafe { FT_Init_FreeType(&mut lib as *mut _) };
        let lib = ft_result(res, lib).context("FT_Init_FreeType")?;
        let mut lib = Library { lib };

        let interpreter_version: FT_UInt = 38;
        unsafe {
            FT_Property_Set(
                lib.lib,
                b"truetype\0" as *const u8 as *const FT_String,
                b"interpreter-version\0" as *const u8 as *const FT_String,
                &interpreter_version as *const FT_UInt as *const _,
            );
        }

        lib.set_lcd_filter(FT_LcdFilter::FT_LCD_FILTER_DEFAULT).ok();

        Ok(lib)
    }

    pub fn new_face(&self, handle: &FontDataHandle) -> Fallible<Face> {
        let res = unsafe { self.new_memory_face(self.lib, handle) };
        res
    }
    pub unsafe fn new_memory_face(
        &self,
        library_raw: FT_Library,
        handle: &FontDataHandle,
    ) -> Fallible<Face> {
        FT_Reference_Library(library_raw);
        let data = handle.data.to_vec();
        let mut face = ptr::null_mut();

        let res = FT_New_Memory_Face(
            self.lib,
            data.as_ptr(),
            data.len() as _,
            handle.index as _,
            &mut face as *mut _,
        );
        Ok(Face {
            lib: library_raw,
            face: ft_result(res, face)
                .with_context(|_| format!("FT_New_Memory_Face for index {}", handle.index))?,
            bytes: data,
        })
    }

    pub fn set_lcd_filter(&mut self, filter: FT_LcdFilter) -> Fallible<()> {
        unsafe { ft_result(FT_Library_SetLcdFilter(self.lib, filter), ()) }
    }
}

use crate::font::loader::FontDataHandle;
use anyhow::{anyhow, Context, Result};
pub use freetype::freetype::*;
use libc::{self, c_long, c_void, size_t};
use std::ptr;
use std::rc::Rc;

#[inline]
pub fn succeeded(error: FT_Error) -> bool {
    error == freetype::freetype::FT_Err_Ok as FT_Error
}

fn ft_result<T>(err: FT_Error, t: T) -> Result<T> {
    if succeeded(err) {
        Ok(t)
    } else {
        Err(anyhow!("FreeType error {:?} 0x{:x}", err, err))
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
    bytes: Rc<Vec<u8>>,
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
    pub fn set_font_size(&mut self, point_size: f64, dpi: u32) -> Result<()> {
        let size = (point_size * 64.0) as FT_F26Dot6;
        self.set_char_size(size, 0, dpi, 0)
    }

    fn set_char_size(
        &mut self,
        char_width: FT_F26Dot6,
        char_height: FT_F26Dot6,
        horz_resolution: FT_UInt,
        vert_resolution: FT_UInt,
    ) -> Result<()> {
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
    ) -> Result<&FT_GlyphSlotRec_> {
        unsafe {
            let res = FT_Load_Glyph(self.face, glyph_index, load_flags);
            let slot = ft_result(res, &mut *(*self.face).glyph)?;
            ft_result(FT_Render_Glyph(slot, render_mode), slot)
        }
    }
}

extern "C" fn alloc_library(_memory: FT_Memory, size: c_long) -> *mut c_void {
    unsafe { libc::malloc(size as size_t) }
}

extern "C" fn free_library(_memory: FT_Memory, block: *mut c_void) {
    unsafe { libc::free(block) }
}

extern "C" fn realloc_library(
    _memory: FT_Memory,
    _cur_size: c_long,
    new_size: c_long,
    block: *mut c_void,
) -> *mut c_void {
    unsafe { libc::realloc(block, new_size as size_t) }
}

static mut MEMORY: FT_MemoryRec_ = FT_MemoryRec_ {
    user: 0 as *mut c_void,
    alloc: Some(alloc_library),
    free: Some(free_library),
    realloc: Some(realloc_library),
};

pub struct Library {
    lib: FT_Library,
}

impl Drop for Library {
    fn drop(&mut self) {
        unsafe {
            FT_Done_Library(self.lib);
        }
    }
}

impl Library {
    pub fn new() -> Result<Library> {
        let mut lib = ptr::null_mut();

        let err = unsafe { FT_New_Library(&mut MEMORY, &mut lib) };
        if err == freetype::freetype::FT_Err_Ok as FT_Error {
            unsafe {
                FT_Add_Default_Modules(lib);
            }
            Ok(Library { lib })
        } else {
            panic!("failed to create new library")
        }
    }

    pub fn new_face(&self, handle: &FontDataHandle) -> Result<Face> {
        unsafe { self.new_memory_face(self.lib, handle) }
    }
    pub unsafe fn new_memory_face(
        &self,
        library_raw: FT_Library,
        handle: &FontDataHandle,
    ) -> Result<Face> {
        let data = Rc::new(handle.data.to_vec());
        let mut face = ptr::null_mut();

        let res = FT_New_Memory_Face(
            self.lib,
            data.as_ptr(),
            data.len() as _,
            handle.index as _,
            &mut face as *mut _,
        );
        FT_Reference_Library(library_raw);
        Ok(Face {
            lib: library_raw,
            face: ft_result(res, face)
                .with_context(|| format!("FT_New_Memory_Face for index {}", handle.index))?,
            bytes: data,
        })
    }
}

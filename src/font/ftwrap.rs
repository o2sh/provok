use crate::font::locator::FontDataHandle;
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

    let flags = render_mode_to_load_target(FT_Render_Mode::FT_RENDER_MODE_LCD);

    let flags = flags | FT_LOAD_COLOR;

    (flags as i32, render)
}

pub struct Face {
    pub face: FT_Face,
    _bytes: Vec<u8>,
    size: Option<FaceSize>,
}

impl Drop for Face {
    fn drop(&mut self) {
        unsafe {
            FT_Done_Face(self.face);
        }
    }
}

struct FaceSize {
    size: f64,
    dpi: u32,
    cell_width: f64,
    cell_height: f64,
}

impl Face {
    pub fn set_font_size(&mut self, point_size: f64, dpi: u32) -> Fallible<(f64, f64)> {
        if let Some(face_size) = self.size.as_ref() {
            if face_size.size == point_size && face_size.dpi == dpi {
                return Ok((face_size.cell_width, face_size.cell_height));
            }
        }

        log::debug!("set_char_size computing {} dpi={}", point_size, dpi);
        let size = (point_size * 64.0) as FT_F26Dot6;

        let (cell_width, cell_height) = match self.set_char_size(size, size, dpi, dpi) {
            Ok(_) => self.cell_metrics(),
            Err(err) => {
                let sizes = unsafe {
                    let rec = &(*self.face);
                    std::slice::from_raw_parts(rec.available_sizes, rec.num_fixed_sizes as usize)
                };
                if sizes.is_empty() {
                    return Err(err);
                }
                let mut best = 0;
                let mut best_size = 0;
                let mut cell_width = 0;
                let mut cell_height = 0;

                for (idx, info) in sizes.iter().enumerate() {
                    let size = best_size.max(info.height);
                    if size > best_size {
                        best = idx;
                        best_size = size;
                        cell_width = info.width;
                        cell_height = info.height;
                    }
                }
                self.select_size(best)?;
                (f64::from(cell_width), f64::from(cell_height))
            }
        };

        self.size.replace(FaceSize { size: point_size, dpi, cell_width, cell_height });

        Ok((cell_width, cell_height))
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

    fn select_size(&mut self, idx: usize) -> Fallible<()> {
        ft_result(unsafe { FT_Select_Size(self.face, idx as i32) }, ())
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

    pub fn cell_metrics(&mut self) -> (f64, f64) {
        unsafe {
            let metrics = &(*(*self.face).size).metrics;
            let height = (metrics.y_scale as f64 * f64::from((*self.face).height))
                / (f64::from(0x1_0000) * 64.0);

            let mut width = 0.0;
            for i in 32..128 {
                let glyph_pos = FT_Get_Char_Index(self.face, i);
                if glyph_pos == 0 {
                    continue;
                }
                let res = FT_Load_Glyph(self.face, glyph_pos, FT_LOAD_COLOR as i32);
                if succeeded(res) {
                    let glyph = &(*(*self.face).glyph);
                    if glyph.metrics.horiAdvance as f64 > width {
                        width = glyph.metrics.horiAdvance as f64;
                    }
                }
            }
            (width / 64.0, height)
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

    pub fn face_from_locator(&self, handle: &FontDataHandle) -> Fallible<Face> {
        match handle {
            FontDataHandle::OnDisk { path, index } => {
                self.new_face(path.to_str().unwrap(), *index as _)
            }
            FontDataHandle::Memory { data, index, .. } => {
                self.new_face_from_slice(&data, *index as _)
            }
        }
    }

    pub fn new_face<P>(&self, path: P, face_index: FT_Long) -> Fallible<Face>
    where
        P: AsRef<std::path::Path>,
    {
        let mut face = ptr::null_mut();
        let path = path.as_ref();

        let data = std::fs::read(path)?;
        log::trace!("Loading {} for freetype!", path.display());

        let res = unsafe {
            FT_New_Memory_Face(
                self.lib,
                data.as_ptr(),
                data.len() as _,
                face_index,
                &mut face as *mut _,
            )
        };
        Ok(Face {
            face: ft_result(res, face).with_context(|_| {
                format!("FT_New_Memory_Face for {} index {}", path.display(), face_index)
            })?,
            _bytes: data,
            size: None,
        })
    }

    pub fn new_face_from_slice(&self, data: &[u8], face_index: FT_Long) -> Fallible<Face> {
        let data = data.to_vec();
        let mut face = ptr::null_mut();

        let res = unsafe {
            FT_New_Memory_Face(
                self.lib,
                data.as_ptr(),
                data.len() as _,
                face_index,
                &mut face as *mut _,
            )
        };
        Ok(Face {
            face: ft_result(res, face)
                .with_context(|_| format!("FT_New_Memory_Face for index {}", face_index))?,
            _bytes: data,
            size: None,
        })
    }

    pub fn set_lcd_filter(&mut self, filter: FT_LcdFilter) -> Fallible<()> {
        unsafe { ft_result(FT_Library_SetLcdFilter(self.lib, filter), ()) }
    }
}

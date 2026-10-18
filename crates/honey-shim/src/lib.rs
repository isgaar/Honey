//! C ABI drop-in replacement library for low-level OS FreeType substitution.
//!
//! Exposes standard FreeType 2 structs and functions so that GTK, Qt, Cairo,
//! and Chromium/Skia can link or preload directly against Honey's Quartz engine.

#![allow(non_snake_case)]

use honey_core::HoneyFace;
use std::ffi::{c_char, c_int, c_long, c_uchar, c_uint, c_ushort, c_void, CStr, CString};
use std::fs;
use std::ptr;
use std::slice;

pub const FT_GLYPH_FORMAT_BITMAP: c_uint = 0x6269_746d; // 'bitm'
pub const FT_PIXEL_MODE_GRAY: c_uchar = 2;

#[repr(C)]
pub struct FT_Bitmap {
    pub rows: c_uint,
    pub width: c_uint,
    pub pitch: c_int,
    pub buffer: *mut c_uchar,
    pub num_grays: c_ushort,
    pub pixel_mode: c_uchar,
    pub palette_mode: c_uchar,
    pub palette: *mut c_void,
}

#[repr(C)]
#[derive(Default)]
pub struct FT_Glyph_Metrics {
    pub width: c_long,
    pub height: c_long,
    pub horiBearingX: c_long,
    pub horiBearingY: c_long,
    pub horiAdvance: c_long,
    pub vertBearingX: c_long,
    pub vertBearingY: c_long,
    pub vertAdvance: c_long,
}

#[repr(C)]
#[derive(Default)]
pub struct FT_Vector {
    pub x: c_long,
    pub y: c_long,
}

#[repr(C)]
pub struct FT_GlyphSlotRec {
    pub library: *mut c_void,
    pub face: *mut FT_FaceRec,
    pub next: *mut FT_GlyphSlotRec,
    pub glyph_index: c_uint,
    pub generic: [usize; 2],
    pub metrics: FT_Glyph_Metrics,
    pub linearHoriAdvance: c_long,
    pub linearVertAdvance: c_long,
    pub advance: FT_Vector,
    pub format: c_uint,
    pub bitmap: FT_Bitmap,
    pub bitmap_left: c_int,
    pub bitmap_top: c_int,
}

#[repr(C)]
pub struct FT_FaceRec {
    pub num_faces: c_long,
    pub face_index: c_long,
    pub face_flags: c_long,
    pub style_flags: c_long,
    pub num_glyphs: c_long,
    pub family_name: *mut c_char,
    pub style_name: *mut c_char,
    pub num_fixed_sizes: c_int,
    pub available_sizes: *mut c_void,
    pub num_charmaps: c_int,
    pub charmaps: *mut c_void,
    pub generic: [usize; 2],
    pub bbox_xMin: c_long,
    pub bbox_yMin: c_long,
    pub bbox_xMax: c_long,
    pub bbox_yMax: c_long,
    pub units_per_EM: c_ushort,
    pub ascender: c_short,
    pub descender: c_short,
    pub height: c_short,
    pub max_advance_width: c_short,
    pub max_advance_height: c_short,
    pub underline_position: c_short,
    pub underline_thickness: c_short,
    pub glyph: *mut FT_GlyphSlotRec,
    pub size: *mut c_void,
    pub charmap: *mut c_void,

    // Internal context managed by Honey
    pub internal_context: *mut FaceContext,
}

#[allow(non_camel_case_types)]
type c_short = i16;

/// Internal state holding font bytes and current rasterization properties.
pub struct FaceContext {
    pub font_data: Vec<u8>,
    pub pixel_size: f32,
    pub last_bitmap_buffer: Vec<u8>,
}

#[no_mangle]
pub extern "C" fn honey_version() -> *const c_char {
    static VERSION: &[u8] = b"Honey Core 0.7.0 (Rust Quartz Parity Engine)\0";
    VERSION.as_ptr() as *const c_char
}

#[no_mangle]
pub extern "C" fn honey_blend_pixel_linear(bg_argb: u32, fg_argb: u32, alpha: u8) -> u32 {
    honey_core::blend_linear(bg_argb, fg_argb, alpha)
}

#[no_mangle]
pub extern "C" fn honey_dilate_mask(
    src: *const u8,
    dst: *mut u8,
    width: usize,
    height: usize,
    factor: f32,
) -> c_int {
    if src.is_null() || dst.is_null() || width == 0 || height == 0 {
        return -1;
    }
    let src_slice = unsafe { slice::from_raw_parts(src, width * height) };
    let dilated = honey_core::dilate_mask(src_slice, width, height, factor);
    unsafe {
        ptr::copy_nonoverlapping(dilated.as_ptr(), dst, width * height);
    }
    0
}

#[no_mangle]
pub extern "C" fn honey_compute_ink_energy(mask: *const u8, len: usize) -> u64 {
    if mask.is_null() || len == 0 {
        return 0;
    }
    let mask_slice = unsafe { slice::from_raw_parts(mask, len) };
    honey_core::compute_ink_energy(mask_slice)
}

// =========================================================================
// FreeType 2 C ABI Implementation
// =========================================================================

#[no_mangle]
pub extern "C" fn FT_Init_FreeType(alibrary: *mut *mut c_void) -> c_int {
    if alibrary.is_null() {
        return 1;
    }
    unsafe {
        *alibrary = 0xCAFE_BABE as *mut c_void;
    }
    0
}

#[no_mangle]
pub extern "C" fn FT_Done_FreeType(library: *mut c_void) -> c_int {
    let _ = library;
    0
}

#[no_mangle]
pub extern "C" fn FT_New_Memory_Face(
    library: *mut c_void,
    file_base: *const c_uchar,
    file_size: c_long,
    face_index: c_long,
    aface: *mut *mut FT_FaceRec,
) -> c_int {
    let _ = library;
    if file_base.is_null() || file_size <= 0 || aface.is_null() {
        return 1;
    }

    let font_bytes = unsafe { slice::from_raw_parts(file_base, file_size as usize) }.to_vec();
    init_face_from_bytes(font_bytes, face_index as u32, aface)
}

#[no_mangle]
pub extern "C" fn FT_New_Face(
    library: *mut c_void,
    filepathname: *const c_char,
    face_index: c_long,
    aface: *mut *mut FT_FaceRec,
) -> c_int {
    let _ = library;
    if filepathname.is_null() || aface.is_null() {
        return 1;
    }

    let c_str = unsafe { CStr::from_ptr(filepathname) };
    let path = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return 1,
    };

    let font_bytes = match fs::read(path) {
        Ok(b) => b,
        Err(_) => return 2, // Cannot open resource
    };

    init_face_from_bytes(font_bytes, face_index as u32, aface)
}

fn init_face_from_bytes(
    font_bytes: Vec<u8>,
    face_index: u32,
    aface: *mut *mut FT_FaceRec,
) -> c_int {
    let (num_glyphs, units_per_em, ascender, descender) = match HoneyFace::from_slice(&font_bytes, face_index) {
        Ok(f) => (
            f.number_of_glyphs() as c_long,
            f.units_per_em(),
            f.ascender(),
            f.descender(),
        ),
        Err(_) => return 3, // Unknown file format
    };

    let context = Box::new(FaceContext {
        font_data: font_bytes,
        pixel_size: 16.0,
        last_bitmap_buffer: Vec::new(),
    });

    let slot = Box::new(FT_GlyphSlotRec {
        library: ptr::null_mut(),
        face: ptr::null_mut(),
        next: ptr::null_mut(),
        glyph_index: 0,
        generic: [0; 2],
        metrics: FT_Glyph_Metrics::default(),
        linearHoriAdvance: 0,
        linearVertAdvance: 0,
        advance: FT_Vector::default(),
        format: FT_GLYPH_FORMAT_BITMAP,
        bitmap: FT_Bitmap {
            rows: 0,
            width: 0,
            pitch: 0,
            buffer: ptr::null_mut(),
            num_grays: 256,
            pixel_mode: FT_PIXEL_MODE_GRAY,
            palette_mode: 0,
            palette: ptr::null_mut(),
        },
        bitmap_left: 0,
        bitmap_top: 0,
    });

    let family = CString::new("Honey Font").unwrap().into_raw();
    let style = CString::new("Regular").unwrap().into_raw();
    let slot_raw = Box::into_raw(slot);

    let face = Box::new(FT_FaceRec {
        num_faces: 1,
        face_index: face_index as c_long,
        face_flags: 1 | (1 << 3), // FT_FACE_FLAG_SCALABLE | FT_FACE_FLAG_HORIZONTAL
        style_flags: 0,
        num_glyphs,
        family_name: family,
        style_name: style,
        num_fixed_sizes: 0,
        available_sizes: ptr::null_mut(),
        num_charmaps: 1,
        charmaps: ptr::null_mut(),
        generic: [0; 2],
        bbox_xMin: 0,
        bbox_yMin: 0,
        bbox_xMax: 0,
        bbox_yMax: 0,
        units_per_EM: units_per_em,
        ascender,
        descender,
        height: ascender - descender,
        max_advance_width: 0,
        max_advance_height: 0,
        underline_position: -100,
        underline_thickness: 50,
        glyph: slot_raw,
        size: ptr::null_mut(),
        charmap: ptr::null_mut(),
        internal_context: Box::into_raw(context),
    });

    let face_raw = Box::into_raw(face);
    unsafe {
        (*slot_raw).face = face_raw;
        *aface = face_raw;
    }
    0
}

#[no_mangle]
pub extern "C" fn FT_Done_Face(face: *mut FT_FaceRec) -> c_int {
    if face.is_null() {
        return 1;
    }
    unsafe {
        let face_box = Box::from_raw(face);
        if !face_box.glyph.is_null() {
            let _ = Box::from_raw(face_box.glyph);
        }
        if !face_box.internal_context.is_null() {
            let _ = Box::from_raw(face_box.internal_context);
        }
        if !face_box.family_name.is_null() {
            let _ = CString::from_raw(face_box.family_name);
        }
        if !face_box.style_name.is_null() {
            let _ = CString::from_raw(face_box.style_name);
        }
    }
    0
}

#[no_mangle]
pub extern "C" fn FT_Set_Pixel_Sizes(
    face: *mut FT_FaceRec,
    pixel_width: c_uint,
    pixel_height: c_uint,
) -> c_int {
    if face.is_null() {
        return 1;
    }
    let target = if pixel_height > 0 {
        pixel_height as f32
    } else {
        pixel_width as f32
    };

    unsafe {
        let ctx = (*face).internal_context;
        if !ctx.is_null() {
            (*ctx).pixel_size = target.max(4.0);
        }
    }
    0
}

#[no_mangle]
pub extern "C" fn FT_Set_Char_Size(
    face: *mut FT_FaceRec,
    _char_width: c_long,
    char_height: c_long,
    _horz_res: c_uint,
    vert_res: c_uint,
) -> c_int {
    if face.is_null() {
        return 1;
    }
    let dpi = if vert_res > 0 { vert_res as f32 } else { 72.0 };
    let pt = (char_height as f32) / 64.0;
    let px = pt * (dpi / 72.0);

    unsafe {
        let ctx = (*face).internal_context;
        if !ctx.is_null() {
            (*ctx).pixel_size = px.max(4.0);
        }
    }
    0
}

#[no_mangle]
pub extern "C" fn FT_Get_Char_Index(face: *mut FT_FaceRec, charcode: c_ulong) -> c_uint {
    if face.is_null() {
        return 0;
    }
    unsafe {
        let ctx = (*face).internal_context;
        if ctx.is_null() {
            return 0;
        }
        if let Ok(hf) = HoneyFace::from_slice(&(*ctx).font_data, (*face).face_index as u32) {
            if let Some(ch) = char::from_u32(charcode as u32) {
                return hf.glyph_index(ch).unwrap_or(0);
            }
        }
    }
    0
}

#[allow(non_camel_case_types)]
type c_ulong = usize;

#[no_mangle]
pub extern "C" fn FT_Load_Glyph(
    face: *mut FT_FaceRec,
    glyph_index: c_uint,
    _load_flags: c_int,
) -> c_int {
    if face.is_null() {
        return 1;
    }
    unsafe {
        let slot = (*face).glyph;
        if !slot.is_null() {
            (*slot).glyph_index = glyph_index;
            (*slot).format = FT_GLYPH_FORMAT_BITMAP;
        }
    }
    0
}

#[no_mangle]
pub extern "C" fn FT_Load_Char(face: *mut FT_FaceRec, char_code: c_ulong, load_flags: c_int) -> c_int {
    let gid = FT_Get_Char_Index(face, char_code);
    FT_Load_Glyph(face, gid, load_flags)
}

#[no_mangle]
pub extern "C" fn FT_Render_Glyph(slot: *mut FT_GlyphSlotRec, _render_mode: c_uint) -> c_int {
    if slot.is_null() {
        return 1;
    }
    unsafe {
        let face = (*slot).face;
        if face.is_null() {
            return 1;
        }
        let ctx = (*face).internal_context;
        if ctx.is_null() {
            return 1;
        }

        let gid = (*slot).glyph_index;
        let px_size = (*ctx).pixel_size;

        if let Ok(hf) = HoneyFace::from_slice(&(*ctx).font_data, (*face).face_index as u32) {
            if let Some((bmp, left, top, adv)) = hf.render_glyph(gid, px_size, 0.0, 0.0) {
                (*ctx).last_bitmap_buffer = bmp.buffer;

                (*slot).bitmap.width = bmp.width as c_uint;
                (*slot).bitmap.rows = bmp.height as c_uint;
                (*slot).bitmap.pitch = bmp.width as c_int;
                (*slot).bitmap.buffer = (*ctx).last_bitmap_buffer.as_mut_ptr();
                (*slot).bitmap.num_grays = 256;
                (*slot).bitmap.pixel_mode = FT_PIXEL_MODE_GRAY;

                (*slot).bitmap_left = left;
                (*slot).bitmap_top = top;
                (*slot).advance.x = (adv * 64.0) as c_long;
                (*slot).advance.y = 0;
                (*slot).format = FT_GLYPH_FORMAT_BITMAP;

                return 0;
            }
        }
    }
    1
}

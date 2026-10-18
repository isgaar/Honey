//! C ABI bridge for Honey Core.
//!
//! Allows Honey to be loaded directly by C runtimes, GTK, Cairo, or dynamically
//! preloaded to replace legacy FreeType rasterization and blending.

use honey_core::{
    blend_linear, compute_ink_energy as core_compute_ink_energy, dilate_mask, GlyphOutline,
    QuartzRasterizer,
};
use std::ffi::{c_char, c_int, c_void};
use std::slice;

#[no_mangle]
pub extern "C" fn honey_version() -> *const c_char {
    static VERSION: &[u8] = b"Honey Core 0.7.0 (Rust Quartz Parity Engine)\0";
    VERSION.as_ptr() as *const c_char
}

#[no_mangle]
pub extern "C" fn honey_blend_pixel_linear(bg_argb: u32, fg_argb: u32, alpha: u8) -> u32 {
    blend_linear(bg_argb, fg_argb, alpha)
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
    let dilated = dilate_mask(src_slice, width, height, factor);
    unsafe {
        std::ptr::copy_nonoverlapping(dilated.as_ptr(), dst, width * height);
    }
    0
}

#[no_mangle]
pub extern "C" fn honey_compute_ink_energy(mask: *const u8, len: usize) -> u64 {
    if mask.is_null() || len == 0 {
        return 0;
    }
    let mask_slice = unsafe { slice::from_raw_parts(mask, len) };
    core_compute_ink_energy(mask_slice)
}

#[no_mangle]
pub extern "C" fn honey_rasterize_rect(
    width: usize,
    height: usize,
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    out_buf: *mut u8,
) -> c_int {
    if out_buf.is_null() || width == 0 || height == 0 {
        return -1;
    }

    let mut outline = GlyphOutline::new();
    outline.move_to(x0, y0);
    outline.line_to(x1, y0);
    outline.line_to(x1, y1);
    outline.line_to(x0, y1);
    outline.close();

    let rasterizer = QuartzRasterizer::new(width, height);
    let bmp = rasterizer.rasterize(&outline);

    unsafe {
        std::ptr::copy_nonoverlapping(bmp.buffer.as_ptr(), out_buf, width * height);
    }
    0
}

/// FreeType ABI drop-in initialization hook.
/// Demonstrates direct ABI compatibility for replacing libfreetype.so.
#[no_mangle]
pub extern "C" fn FT_Init_FreeType(alibrary: *mut *mut c_void) -> c_int {
    if !alibrary.is_null() {
        // Return dummy handle indicating successful initialization of Honey typography core
        unsafe {
            *alibrary = 0xCAFE_BABE as *mut c_void;
        }
    }
    0
}

/// FreeType ABI cleanup hook.
#[no_mangle]
pub extern "C" fn FT_Done_FreeType(library: *mut c_void) -> c_int {
    let _ = library;
    0
}

//! Honey Core - Native Quartz-Parity Typography & Linear Compositing Engine
//!
//! Developed to replace legacy fontconfig/FreeType patching with a direct,
//! low-level OS typography pipeline in Rust.

pub mod dilation;
pub mod font;
pub mod gamma;
pub mod rasterizer;

pub use dilation::{
    compute_dilation_factor, compute_dilation_factor_with_profile, compute_ink_energy, dilate_mask,
    DesktopProfile,
};
pub use font::{count_faces, GlyphMetrics, HoneyFace};
pub use gamma::{blend_glyph_buffer, blend_linear, linear_to_srgb, srgb_to_linear};
pub use rasterizer::{GlyphOutline, Point, QuartzRasterizer, RasterBitmap};
pub use ttf_parser::OutlineBuilder;

/// Render a vector glyph outline with complete Quartz compositor parity:
/// 1. Anti-hinted vector sampling (true Bezier contours).
/// 2. Fractional subpixel positioning (1/64 pixel).
/// 3. Optical stem dilation (+13.4% to +16.0% ink energy calibrated to font size).
/// 4. Pure grayscale antialiasing (universal R=G=B).
pub fn render_glyph_quartz(
    outline: &GlyphOutline,
    width: usize,
    height: usize,
    pt_size: f32,
    subpixel_offset_x: f32,
    subpixel_offset_y: f32,
) -> RasterBitmap {
    let rasterizer = QuartzRasterizer::new(width, height)
        .with_subpixel_offset(subpixel_offset_x, subpixel_offset_y);
    let base_bmp = rasterizer.rasterize(outline);

    let factor = compute_dilation_factor(pt_size);
    let dilated_buffer = dilate_mask(&base_bmp.buffer, width, height, factor);

    RasterBitmap {
        width,
        height,
        left: base_bmp.left,
        top: base_bmp.top,
        buffer: dilated_buffer,
    }
}

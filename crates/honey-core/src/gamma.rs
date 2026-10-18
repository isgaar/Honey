//! Photometric linear gamma compositing engine.
//!
//! Replicates Apple Quartz / Metal `UberCompositeFragment` linear blending.

use std::sync::OnceLock;

/// Lookup tables for high-throughput photometric conversions.
pub struct GammaLuts {
    srgb_to_lin: [f32; 256],
    lin_to_srgb: [u8; 4096],
}

impl GammaLuts {
    fn new() -> Self {
        let mut srgb_to_lin = [0.0f32; 256];
        let mut lin_to_srgb = [0u8; 4096];

        // IEC 61966-2-1 transfer function
        for i in 0..256 {
            let c = i as f32 / 255.0;
            if c <= 0.04045 {
                srgb_to_lin[i] = c / 12.92;
            } else {
                srgb_to_lin[i] = ((c + 0.055) / 1.055).powf(2.4);
            }
        }

        for i in 0..4096 {
            let lin = i as f32 / 4095.0;
            let srgb = if lin <= 0.0031308 {
                lin * 12.92
            } else {
                1.055 * lin.powf(1.0 / 2.4) - 0.055
            };
            lin_to_srgb[i] = (srgb.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
        }

        Self {
            srgb_to_lin,
            lin_to_srgb,
        }
    }
}

static LUTS: OnceLock<GammaLuts> = OnceLock::new();

#[inline(always)]
fn get_luts() -> &'static GammaLuts {
    LUTS.get_or_init(GammaLuts::new)
}

/// Convert an 8-bit sRGB color channel to linear luminance (0.0 to 1.0).
#[inline(always)]
pub fn srgb_to_linear(c: u8) -> f32 {
    get_luts().srgb_to_lin[c as usize]
}

/// Convert linear luminance (0.0 to 1.0) to an 8-bit sRGB value.
#[inline(always)]
pub fn linear_to_srgb(lin: f32) -> u8 {
    if lin <= 0.0 {
        return 0;
    }
    if lin >= 1.0 {
        return 255;
    }
    let idx = (lin * 4095.0 + 0.5) as usize;
    get_luts().lin_to_srgb[idx.min(4095)]
}

/// Blend a foreground color over a background color in linear gamma space.
///
/// Returns 32-bit ARGB (0xAARRGGBB).
#[inline]
pub fn blend_linear(bg_argb: u32, fg_argb: u32, alpha: u8) -> u32 {
    if alpha == 0 {
        return bg_argb;
    }
    if alpha == 255 {
        return fg_argb;
    }

    let luts = get_luts();
    let a = alpha as f32 / 255.0;
    let inv_a = 1.0 - a;

    let bg_r = ((bg_argb >> 16) & 0xFF) as usize;
    let bg_g = ((bg_argb >> 8) & 0xFF) as usize;
    let bg_b = (bg_argb & 0xFF) as usize;

    let fg_r = ((fg_argb >> 16) & 0xFF) as usize;
    let fg_g = ((fg_argb >> 8) & 0xFF) as usize;
    let fg_b = (fg_argb & 0xFF) as usize;

    let lin_r = luts.srgb_to_lin[fg_r] * a + luts.srgb_to_lin[bg_r] * inv_a;
    let lin_g = luts.srgb_to_lin[fg_g] * a + luts.srgb_to_lin[bg_g] * inv_a;
    let lin_b = luts.srgb_to_lin[fg_b] * a + luts.srgb_to_lin[bg_b] * inv_a;

    let out_r = linear_to_srgb(lin_r);
    let out_g = linear_to_srgb(lin_g);
    let out_b = linear_to_srgb(lin_b);

    0xFF00_0000 | ((out_r as u32) << 16) | ((out_g as u32) << 8) | (out_b as u32)
}

/// Blend an entire glyph alpha mask onto a destination ARGB framebuffer.
pub fn blend_glyph_buffer(
    dst: &mut [u32],
    dst_stride: usize,
    dst_x: usize,
    dst_y: usize,
    mask: &[u8],
    mask_width: usize,
    mask_height: usize,
    fg_argb: u32,
) {
    for row in 0..mask_height {
        let y = dst_y + row;
        for col in 0..mask_width {
            let x = dst_x + col;
            let alpha = mask[row * mask_width + col];
            if alpha > 0 {
                let idx = y * dst_stride + x;
                if idx < dst.len() {
                    dst[idx] = blend_linear(dst[idx], fg_argb, alpha);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_srgb_roundtrip() {
        for c in 0..=255u8 {
            let lin = srgb_to_linear(c);
            let srgb = linear_to_srgb(lin);
            assert_eq!(c, srgb, "Mismatch for channel value {c}");
        }
    }

    #[test]
    fn test_dark_mode_boost() {
        let bg_dark = 0xFF1E1E1E;
        let fg_white = 0xFFFFFFFF;
        let alpha = 32u8;

        // Non-linear sRGB: 0x1E + 32/255 * (255 - 0x1E) = 58
        let direct_srgb = 0x1Eu32 + ((32 * (255 - 0x1E) + 127) / 255);
        let linear_blended = blend_linear(bg_dark, fg_white, alpha) & 0xFF;

        // Linear compositing must produce higher luminance (around 103)
        assert!(linear_blended > direct_srgb);
        assert_eq!(linear_blended, 103);
    }
}

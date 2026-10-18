//! Optical stem darkening and contour dilation engine.
//!
//! Replicates `CGContextSetFontDilation(context, CGSize(w, h))` and FreeType darkening curves.

/// Desktop rendering profile for compositor compensation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DesktopProfile {
    #[default]
    Standard,
    /// GNOME Mutter profile: applies 1.15x dilation boost at reading sizes (9-16pt)
    /// to compensate for bilinear downscaling during fractional scaling (supersampling 2x).
    Gnome,
    /// KDE Plasma profile: 1:1 unwarped vector curves for QtQuick SDF distance fields.
    Kde,
}

/// Compute optical dilation factor based on typographic font size (pt).
///
/// Follows the Apple optical scaling curve: maximum dilation at small reading sizes
/// (8-14pt), tapering smoothly to zero at display headline sizes (>36pt).
pub fn compute_dilation_factor(pt_size: f32) -> f32 {
    if pt_size <= 8.0 {
        0.18
    } else if pt_size <= 16.0 {
        // Linear transition: 0.18 down to 0.12
        0.18 - (pt_size - 8.0) * (0.06 / 8.0)
    } else if pt_size <= 32.0 {
        // Transition down to 0.04
        0.12 - (pt_size - 16.0) * (0.08 / 16.0)
    } else if pt_size <= 48.0 {
        0.04 - (pt_size - 32.0) * (0.04 / 16.0)
    } else {
        0.0
    }
}

/// Compute optical dilation factor taking into account the active desktop profile.
pub fn compute_dilation_factor_with_profile(pt_size: f32, profile: DesktopProfile) -> f32 {
    let base = compute_dilation_factor(pt_size);
    match profile {
        DesktopProfile::Gnome => {
            if pt_size >= 9.0 && pt_size <= 16.0 {
                (base * 1.15).min(0.25)
            } else {
                base
            }
        }
        _ => base,
    }
}

/// Apply isotropic 8-connected outline dilation on an 8-bit grayscale coverage bitmap.
///
/// Expands glyph coverage outward along the normal boundary without warping glyph metrics.
pub fn dilate_mask(src: &[u8], width: usize, height: usize, factor: f32) -> Vec<u8> {
    if factor <= 0.0 || width == 0 || height == 0 {
        return src.to_vec();
    }

    let mut dst = vec![0u8; width * height];
    let weight_center = 1.0f32;
    let weight_cross = factor * 0.35f32;
    let weight_diag = factor * 0.15f32;

    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;
            let c = src[idx] as f32;
            let mut max_val = c * weight_center;

            // 4-connected neighbors
            if x > 0 {
                let v = src[idx - 1] as f32 * weight_cross;
                if v > max_val {
                    max_val = v;
                }
            }
            if x + 1 < width {
                let v = src[idx + 1] as f32 * weight_cross;
                if v > max_val {
                    max_val = v;
                }
            }
            if y > 0 {
                let v = src[idx - width] as f32 * weight_cross;
                if v > max_val {
                    max_val = v;
                }
            }
            if y + 1 < height {
                let v = src[idx + width] as f32 * weight_cross;
                if v > max_val {
                    max_val = v;
                }
            }

            // Diagonal neighbors (8-connected circular kernel)
            if x > 0 && y > 0 {
                let v = src[idx - width - 1] as f32 * weight_diag;
                if v > max_val {
                    max_val = v;
                }
            }
            if x + 1 < width && y > 0 {
                let v = src[idx - width + 1] as f32 * weight_diag;
                if v > max_val {
                    max_val = v;
                }
            }
            if x > 0 && y + 1 < height {
                let v = src[idx + width - 1] as f32 * weight_diag;
                if v > max_val {
                    max_val = v;
                }
            }
            if x + 1 < width && y + 1 < height {
                let v = src[idx + width + 1] as f32 * weight_diag;
                if v > max_val {
                    max_val = v;
                }
            }

            let dilated = c + max_val * factor;
            dst[idx] = dilated.min(255.0).round() as u8;
        }
    }

    dst
}

/// Compute total ink absorption energy of a raster mask: sum(coverage).
pub fn compute_ink_energy(mask: &[u8]) -> u64 {
    mask.iter().map(|&p| p as u64).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dilation_energy_increase() {
        let width = 16;
        let height = 16;
        let mut mask = vec![0u8; width * height];

        // Draw a synthetic 2px vertical stroke
        for y in 2..14 {
            mask[y * width + 7] = 255;
            mask[y * width + 8] = 255;
            mask[y * width + 6] = 100;
            mask[y * width + 9] = 100;
        }

        let base_energy = compute_ink_energy(&mask);
        let factor = 0.14f32;
        let dilated = dilate_mask(&mask, width, height, factor);
        let dilated_energy = compute_ink_energy(&dilated);

        assert!(
            dilated_energy > base_energy,
            "Dilated energy must exceed base energy"
        );
        let pct = (dilated_energy - base_energy) as f64 / base_energy as f64 * 100.0;
        assert!(
            pct >= 4.0 && pct <= 18.0,
            "Expected +4% to +18% ink energy boost, got +{pct:.2}%"
        );
    }

    #[test]
    fn test_gnome_profile_dilation_boost() {
        let pt_12 = 12.0;
        let factor_std = compute_dilation_factor_with_profile(pt_12, DesktopProfile::Standard);
        let factor_gnome = compute_dilation_factor_with_profile(pt_12, DesktopProfile::Gnome);
        assert!(
            factor_gnome > factor_std,
            "GNOME profile must apply compensation boost at 12pt reading size"
        );
        let diff = factor_gnome / factor_std;
        assert!((diff - 1.15).abs() < 1e-4, "Expected 1.15x boost, got {diff}");
    }
}

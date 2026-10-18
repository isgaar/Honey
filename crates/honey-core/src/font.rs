//! OpenType / TrueType font vector outline extraction.
//!
//! Uses zero-allocation safe parsing via ttf-parser to convert glyph curves
//! into Quartz-compatible vector paths.

use crate::dilation::compute_dilation_factor;
use crate::rasterizer::{GlyphOutline, QuartzRasterizer, RasterBitmap};
use ttf_parser::{Face, GlyphId, OutlineBuilder};

/// Adapter converting ttf-parser vector callbacks into Honey's GlyphOutline.
struct OutlineCollector {
    outline: GlyphOutline,
    scale: f32,
    offset_x: f32,
    ascender: f32,
}

impl OutlineCollector {
    fn new(scale: f32, offset_x: f32, ascender: f32) -> Self {
        Self {
            outline: GlyphOutline::new(),
            scale,
            offset_x,
            ascender,
        }
    }

    #[inline]
    fn map_x(&self, x: f32) -> f32 {
        (x - self.offset_x) * self.scale
    }

    #[inline]
    fn map_y(&self, y: f32) -> f32 {
        // Invert Y axis: TrueType is Y-up, raster display is Y-down
        (self.ascender - y) * self.scale
    }
}

impl OutlineBuilder for OutlineCollector {
    fn move_to(&mut self, x: f32, y: f32) {
        self.outline.move_to(self.map_x(x), self.map_y(y));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.outline.line_to(self.map_x(x), self.map_y(y));
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.outline
            .quad_to(self.map_x(x1), self.map_y(y1), self.map_x(x), self.map_y(y));
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.outline.cubic_to(
            self.map_x(x1),
            self.map_y(y1),
            self.map_x(x2),
            self.map_y(y2),
            self.map_x(x),
            self.map_y(y),
        );
    }

    fn close(&mut self) {
        self.outline.close();
    }
}

/// A parsed font face backed by raw font memory.
pub struct HoneyFace<'a> {
    face: Face<'a>,
}

impl<'a> HoneyFace<'a> {
    /// Parse a font from raw byte slice (supports TTF, OTF, TTC).
    pub fn from_slice(data: &'a [u8], index: u32) -> Result<Self, ttf_parser::FaceParsingError> {
        let face = Face::parse(data, index)?;
        Ok(Self { face })
    }

    /// Resolve character to glyph index.
    pub fn glyph_index(&self, c: char) -> Option<u32> {
        self.face.glyph_index(c).map(|id| id.0 as u32)
    }

    /// Number of glyphs in the font.
    pub fn number_of_glyphs(&self) -> u16 {
        self.face.number_of_glyphs()
    }

    /// Horizontal advance width for a glyph in font design units.
    pub fn glyph_hor_advance(&self, glyph_id: u32) -> Option<u16> {
        self.face.glyph_hor_advance(GlyphId(glyph_id as u16))
    }

    /// Units per EM (typically 1000 or 2048).
    pub fn units_per_em(&self) -> u16 {
        self.face.units_per_em()
    }

    /// Ascender in font design units.
    pub fn ascender(&self) -> i16 {
        self.face.ascender()
    }

    /// Descender in font design units (typically negative).
    pub fn descender(&self) -> i16 {
        self.face.descender()
    }

    /// Retrieve raw OpenType table bytes (e.g. b"GSUB", b"GPOS", b"head").
    pub fn table_data(&self, tag: [u8; 4]) -> Option<&'a [u8]> {
        self.face.raw_face().table(ttf_parser::Tag::from_bytes(&tag))
    }

    /// Retrieve the PostScript name of the font if available.
    pub fn postscript_name(&self) -> Option<String> {
        for name in self.face.names() {
            if name.name_id == ttf_parser::name_id::POST_SCRIPT_NAME {
                if let Some(s) = name.to_string() {
                    return Some(s);
                }
            }
        }
        None
    }

    /// Retrieve the font family name if available.
    pub fn family_name(&self) -> Option<String> {
        for name in self.face.names() {
            if name.name_id == ttf_parser::name_id::FAMILY {
                if let Some(s) = name.to_string() {
                    return Some(s);
                }
            }
        }
        None
    }

    /// Retrieve the glyph name if stored in post table.
    pub fn glyph_name(&self, glyph_id: u32) -> Option<&str> {
        self.face.glyph_name(GlyphId(glyph_id as u16))
    }

    /// Query the glyph design-unit bounding box (x_min, y_min, x_max, y_max).
    pub fn glyph_bounding_box(&self, glyph_id: u32) -> Option<(i16, i16, i16, i16)> {
        let bbox = self.face.glyph_bounding_box(GlyphId(glyph_id as u16))?;
        Some((bbox.x_min, bbox.y_min, bbox.x_max, bbox.y_max))
    }

    /// Extract raw outline vectors for custom decomp or vector drawing.
    pub fn outline_glyph<B: OutlineBuilder>(&self, glyph_id: u32, builder: &mut B) -> Option<()> {
        self.face.outline_glyph(GlyphId(glyph_id as u16), builder).map(|_| ())
    }

    /// Render a glyph with complete Quartz parity (anti-hinting, subpixel positioning, stem dilation).
    pub fn render_glyph(
        &self,
        glyph_id: u32,
        pixel_size: f32,
        subpixel_offset_x: f32,
        subpixel_offset_y: f32,
    ) -> Option<(RasterBitmap, i32, i32, f32)> {
        let id = GlyphId(glyph_id as u16);
        let bbox = self.face.glyph_bounding_box(id)?;
        let upem = self.face.units_per_em() as f32;
        let scale = pixel_size / upem;

        let min_x = bbox.x_min as f32;
        let max_x = bbox.x_max as f32;
        let min_y = bbox.y_min as f32;
        let max_y = bbox.y_max as f32;

        let width_px = (((max_x - min_x) * scale).ceil() as usize).max(1) + 2;
        let height_px = (((max_y - min_y) * scale).ceil() as usize).max(1) + 2;

        let mut collector = OutlineCollector::new(scale, min_x - 1.0 / scale, max_y + 1.0 / scale);
        self.face.outline_glyph(id, &mut collector)?;

        let rasterizer = QuartzRasterizer::new(width_px, height_px)
            .with_subpixel_offset(subpixel_offset_x, subpixel_offset_y);
        let base_bmp = rasterizer.rasterize(&collector.outline);

        // Apply optical stem dilation
        let pt_size = pixel_size * 72.0 / 96.0;
        let factor = compute_dilation_factor(pt_size);
        let dilated_buffer = crate::dilation::dilate_mask(&base_bmp.buffer, width_px, height_px, factor);

        let bitmap_left = (min_x * scale).floor() as i32;
        let bitmap_top = (max_y * scale).ceil() as i32;
        let advance = self.face.glyph_hor_advance(id).unwrap_or(0) as f32 * scale;

        let final_bmp = RasterBitmap {
            width: width_px,
            height: height_px,
            left: bitmap_left,
            top: bitmap_top,
            buffer: dilated_buffer,
        };

        Some((final_bmp, bitmap_left, bitmap_top, advance))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_parsing() {
        let res = HoneyFace::from_slice(&[], 0);
        assert!(res.is_err(), "Empty slice must fail parsing");
    }
}

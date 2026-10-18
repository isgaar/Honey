//! OpenType / TrueType font vector outline extraction.
//!
//! Uses zero-allocation safe parsing via ttf-parser to convert glyph curves
//! into Quartz-compatible vector paths.

use crate::dilation::compute_dilation_factor;
use crate::rasterizer::{GlyphOutline, PathSegment, Point, QuartzRasterizer, RasterBitmap};
use ttf_parser::{Face, GlyphId, OutlineBuilder};

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct GlyphMetrics {
    pub left: i32,
    pub top: i32,
    pub width: usize,
    pub height: usize,
    pub advance: f32,
}

#[derive(Default)]
struct RawOutlineCollector {
    segments: Vec<PathSegment>,
    min_x: f32,
    max_x: f32,
    min_y: f32,
    max_y: f32,
    has_points: bool,
    scale: f32,
}

impl RawOutlineCollector {
    fn new(scale: f32) -> Self {
        Self {
            segments: Vec::new(),
            min_x: f32::INFINITY,
            max_x: f32::NEG_INFINITY,
            min_y: f32::INFINITY,
            max_y: f32::NEG_INFINITY,
            has_points: false,
            scale,
        }
    }

    #[inline]
    fn update_bounds(&mut self, x: f32, y: f32) {
        let sx = x * self.scale;
        let sy = y * self.scale;
        if sx < self.min_x {
            self.min_x = sx;
        }
        if sx > self.max_x {
            self.max_x = sx;
        }
        if sy < self.min_y {
            self.min_y = sy;
        }
        if sy > self.max_y {
            self.max_y = sy;
        }
        self.has_points = true;
    }
}

impl OutlineBuilder for RawOutlineCollector {
    fn move_to(&mut self, x: f32, y: f32) {
        self.update_bounds(x, y);
        self.segments.push(PathSegment::MoveTo(Point::new(x * self.scale, y * self.scale)));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.update_bounds(x, y);
        self.segments.push(PathSegment::LineTo(Point::new(x * self.scale, y * self.scale)));
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.update_bounds(x1, y1);
        self.update_bounds(x, y);
        self.segments.push(PathSegment::QuadTo(
            Point::new(x1 * self.scale, y1 * self.scale),
            Point::new(x * self.scale, y * self.scale),
        ));
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.update_bounds(x1, y1);
        self.update_bounds(x2, y2);
        self.update_bounds(x, y);
        self.segments.push(PathSegment::CubicTo(
            Point::new(x1 * self.scale, y1 * self.scale),
            Point::new(x2 * self.scale, y2 * self.scale),
            Point::new(x * self.scale, y * self.scale),
        ));
    }

    fn close(&mut self) {
        self.segments.push(PathSegment::Close);
    }
}

/// Count number of font faces in font data (TTC collections or single fonts).
pub fn count_faces(data: &[u8]) -> u32 {
    ttf_parser::fonts_in_collection(data).unwrap_or(1)
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

    /// Calculate exact Quartz metrics for a glyph.
    pub fn glyph_metrics(&self, glyph_id: u32, pixel_size: f32) -> GlyphMetrics {
        let id = GlyphId(glyph_id as u16);
        let upem = self.face.units_per_em().max(1) as f32;
        let scale = pixel_size / upem;
        let advance = self.face.glyph_hor_advance(id).unwrap_or(0) as f32 * scale;

        let mut collector = RawOutlineCollector::new(scale);
        let outline = self.face.outline_glyph(id, &mut collector);

        if outline.is_none() || !collector.has_points {
            return GlyphMetrics {
                left: 0,
                top: 0,
                width: 0,
                height: 0,
                advance,
            };
        }

        let left = collector.min_x.floor() as i32;
        let top = collector.max_y.ceil() as i32;
        let right = collector.max_x.ceil() as i32;
        let bottom = collector.min_y.floor() as i32;

        let width = (right - left).max(0) as usize;
        let height = (top - bottom).max(0) as usize;

        GlyphMetrics {
            left,
            top,
            width,
            height,
            advance,
        }
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
        let upem = self.face.units_per_em().max(1) as f32;
        let scale = pixel_size / upem;
        let advance = self.face.glyph_hor_advance(id).unwrap_or(0) as f32 * scale;

        let mut collector = RawOutlineCollector::new(scale);
        let outline = self.face.outline_glyph(id, &mut collector);

        if outline.is_none() || !collector.has_points {
            let bmp = RasterBitmap {
                width: 0,
                height: 0,
                left: 0,
                top: 0,
                buffer: Vec::new(),
            };
            return Some((bmp, 0, 0, advance));
        }

        let left = collector.min_x.floor() as i32;
        let top = collector.max_y.ceil() as i32;
        let right = collector.max_x.ceil() as i32;
        let bottom = collector.min_y.floor() as i32;

        let width = (right - left).max(0) as usize;
        let height = (top - bottom).max(0) as usize;

        if width == 0 || height == 0 {
            let bmp = RasterBitmap {
                width: 0,
                height: 0,
                left: 0,
                top: 0,
                buffer: Vec::new(),
            };
            return Some((bmp, 0, 0, advance));
        }

        let left_f = left as f32;
        let top_f = top as f32;

        let mut mapped_outline = GlyphOutline::new();
        mapped_outline.segments.reserve(collector.segments.len());

        let ox = subpixel_offset_x;
        let oy = subpixel_offset_y;

        for seg in collector.segments {
            let mapped = match seg {
                PathSegment::MoveTo(p) => PathSegment::MoveTo(Point::new(
                    p.x - left_f + ox,
                    top_f - p.y - oy,
                )),
                PathSegment::LineTo(p) => PathSegment::LineTo(Point::new(
                    p.x - left_f + ox,
                    top_f - p.y - oy,
                )),
                PathSegment::QuadTo(c, to) => PathSegment::QuadTo(
                    Point::new(c.x - left_f + ox, top_f - c.y - oy),
                    Point::new(to.x - left_f + ox, top_f - to.y - oy),
                ),
                PathSegment::CubicTo(c1, c2, to) => PathSegment::CubicTo(
                    Point::new(c1.x - left_f + ox, top_f - c1.y - oy),
                    Point::new(c2.x - left_f + ox, top_f - c2.y - oy),
                    Point::new(to.x - left_f + ox, top_f - to.y - oy),
                ),
                PathSegment::Close => PathSegment::Close,
            };
            mapped_outline.segments.push(mapped);
        }

        let rasterizer = QuartzRasterizer::new(width, height);
        let base_bmp = rasterizer.rasterize(&mapped_outline);

        // Apply optical stem dilation
        let pt_size = pixel_size * 72.0 / 96.0;
        let factor = compute_dilation_factor(pt_size);
        let dilated_buffer = crate::dilation::dilate_mask(&base_bmp.buffer, width, height, factor);

        let final_bmp = RasterBitmap {
            width,
            height,
            left,
            top,
            buffer: dilated_buffer,
        };

        Some((final_bmp, left, top, advance))
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

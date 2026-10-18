//! Analytic fractional subpixel area rasterizer.
//!
//! Replicates Quartz anti-hinting philosophy: outlines retain their natural Bezier geometry
//! and are sampled with subpixel fractional offsets (1/64 pixel precision) without grid snapping.

/// 2D point with subpixel floating-point coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// Vector path segment.
#[derive(Debug, Clone)]
pub enum PathSegment {
    MoveTo(Point),
    LineTo(Point),
    QuadTo(Point, Point),
    CubicTo(Point, Point, Point),
    Close,
}

/// Vector glyph outline made of multiple contours.
#[derive(Debug, Clone, Default)]
pub struct GlyphOutline {
    pub segments: Vec<PathSegment>,
}

impl GlyphOutline {
    pub fn new() -> Self {
        Self {
            segments: Vec::new(),
        }
    }

    pub fn move_to(&mut self, x: f32, y: f32) {
        self.segments.push(PathSegment::MoveTo(Point::new(x, y)));
    }

    pub fn line_to(&mut self, x: f32, y: f32) {
        self.segments.push(PathSegment::LineTo(Point::new(x, y)));
    }

    pub fn quad_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) {
        self.segments
            .push(PathSegment::QuadTo(Point::new(cx, cy), Point::new(x, y)));
    }

    pub fn cubic_to(&mut self, c1x: f32, c1y: f32, c2x: f32, c2y: f32, x: f32, y: f32) {
        self.segments.push(PathSegment::CubicTo(
            Point::new(c1x, c1y),
            Point::new(c2x, c2y),
            Point::new(x, y),
        ));
    }

    pub fn close(&mut self) {
        self.segments.push(PathSegment::Close);
    }
}

/// A line segment edge for scanline area sampling.
#[derive(Debug, Clone, Copy)]
struct Edge {
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    dir: f32,
}

/// Rasterized 8-bit grayscale glyph bitmap.
#[derive(Debug, Clone)]
pub struct RasterBitmap {
    pub width: usize,
    pub height: usize,
    pub left: i32,
    pub top: i32,
    pub buffer: Vec<u8>,
}

/// Analytic area rasterizer for Quartz-grade glyph rendering.
pub struct QuartzRasterizer {
    width: usize,
    height: usize,
    subpixel_offset_x: f32,
    subpixel_offset_y: f32,
}

impl QuartzRasterizer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            subpixel_offset_x: 0.0,
            subpixel_offset_y: 0.0,
        }
    }

    /// Set fractional subpixel offset (e.g. 0.0, 0.25, 0.5, 0.75).
    pub fn with_subpixel_offset(mut self, x: f32, y: f32) -> Self {
        self.subpixel_offset_x = x.fract();
        self.subpixel_offset_y = y.fract();
        self
    }

    /// Rasterize an outline into an 8-bit coverage mask.
    pub fn rasterize(&self, outline: &GlyphOutline) -> RasterBitmap {
        let mut edges = Vec::new();
        let mut curr = Point::default();
        let mut start = Point::default();

        let ox = self.subpixel_offset_x;
        let oy = self.subpixel_offset_y;

        for seg in &outline.segments {
            match seg {
                PathSegment::MoveTo(p) => {
                    curr = Point::new(p.x + ox, p.y + oy);
                    start = curr;
                }
                PathSegment::LineTo(p) => {
                    let next = Point::new(p.x + ox, p.y + oy);
                    if (curr.y - next.y).abs() > 1e-5 {
                        let (p0, p1, dir) = if curr.y < next.y {
                            (curr, next, 1.0f32)
                        } else {
                            (next, curr, -1.0f32)
                        };
                        edges.push(Edge {
                            x0: p0.x,
                            y0: p0.y,
                            x1: p1.x,
                            y1: p1.y,
                            dir,
                        });
                    }
                    curr = next;
                }
                PathSegment::QuadTo(ctrl, to) => {
                    // Approximate quad curve with 4 line segments
                    let c = Point::new(ctrl.x + ox, ctrl.y + oy);
                    let end = Point::new(to.x + ox, to.y + oy);
                    let mut prev = curr;
                    for step in 1..=4 {
                        let t = step as f32 / 4.0;
                        let inv_t = 1.0 - t;
                        let pt = Point::new(
                            inv_t * inv_t * curr.x + 2.0 * inv_t * t * c.x + t * t * end.x,
                            inv_t * inv_t * curr.y + 2.0 * inv_t * t * c.y + t * t * end.y,
                        );
                        if (prev.y - pt.y).abs() > 1e-5 {
                            let (p0, p1, dir) = if prev.y < pt.y {
                                (prev, pt, 1.0f32)
                            } else {
                                (pt, prev, -1.0f32)
                            };
                            edges.push(Edge {
                                x0: p0.x,
                                y0: p0.y,
                                x1: p1.x,
                                y1: p1.y,
                                dir,
                            });
                        }
                        prev = pt;
                    }
                    curr = end;
                }
                PathSegment::CubicTo(c1, c2, to) => {
                    let cp1 = Point::new(c1.x + ox, c1.y + oy);
                    let cp2 = Point::new(c2.x + ox, c2.y + oy);
                    let end = Point::new(to.x + ox, to.y + oy);
                    let mut prev = curr;
                    for step in 1..=6 {
                        let t = step as f32 / 6.0;
                        let it = 1.0 - t;
                        let pt = Point::new(
                            it * it * it * curr.x
                                + 3.0 * it * it * t * cp1.x
                                + 3.0 * it * t * t * cp2.x
                                + t * t * t * end.x,
                            it * it * it * curr.y
                                + 3.0 * it * it * t * cp1.y
                                + 3.0 * it * t * t * cp2.y
                                + t * t * t * end.y,
                        );
                        if (prev.y - pt.y).abs() > 1e-5 {
                            let (p0, p1, dir) = if prev.y < pt.y {
                                (prev, pt, 1.0f32)
                            } else {
                                (pt, prev, -1.0f32)
                            };
                            edges.push(Edge {
                                x0: p0.x,
                                y0: p0.y,
                                x1: p1.x,
                                y1: p1.y,
                                dir,
                            });
                        }
                        prev = pt;
                    }
                    curr = end;
                }
                PathSegment::Close => {
                    if (curr.x - start.x).hypot(curr.y - start.y) > 1e-5 {
                        let (p0, p1, dir) = if curr.y < start.y {
                            (curr, start, 1.0f32)
                        } else {
                            (start, curr, -1.0f32)
                        };
                        edges.push(Edge {
                            x0: p0.x,
                            y0: p0.y,
                            x1: p1.x,
                            y1: p1.y,
                            dir,
                        });
                    }
                    curr = start;
                }
            }
        }

        // Multi-sample scanline coverage (8 subpixel samples per scanline)
        let sub_samples = 8;
        let mut cov_buffer = vec![0.0f32; self.width * self.height];

        for y in 0..self.height {
            for sub_y in 0..sub_samples {
                let sample_y = y as f32 + (sub_y as f32 + 0.5) / sub_samples as f32;
                let mut crossings = Vec::new();

                for edge in &edges {
                    if edge.y0 <= sample_y && sample_y < edge.y1 {
                        let t = (sample_y - edge.y0) / (edge.y1 - edge.y0);
                        let x = edge.x0 + t * (edge.x1 - edge.x0);
                        crossings.push((x, edge.dir));
                    }
                }

                crossings.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

                let mut winding = 0.0f32;
                let mut last_x = 0.0f32;

                for (x, dir) in crossings {
                    if winding.abs() > 0.01 {
                        let x_start = (last_x.floor() as isize).max(0) as usize;
                        let x_end = (x.ceil() as isize).min(self.width as isize).max(0) as usize;

                        for px in x_start..x_end {
                            let cov_left = (px as f32).max(last_x);
                            let cov_right = ((px + 1) as f32).min(x);
                            if cov_right > cov_left {
                                let span = (cov_right - cov_left).clamp(0.0, 1.0);
                                let inc = span / sub_samples as f32;
                                let idx = y * self.width + px;
                                cov_buffer[idx] += inc;
                            }
                        }
                    }
                    winding += dir;
                    last_x = x;
                }
            }
        }

        let buffer = cov_buffer
            .into_iter()
            .map(|cov| (cov * 255.0 + 0.5).min(255.0) as u8)
            .collect();

        RasterBitmap {
            width: self.width,
            height: self.height,
            left: 0,
            top: 0,
            buffer,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rasterize_rectangle() {
        let mut outline = GlyphOutline::new();
        outline.move_to(2.0, 2.0);
        outline.line_to(6.0, 2.0);
        outline.line_to(6.0, 6.0);
        outline.line_to(2.0, 6.0);
        outline.close();

        let rasterizer = QuartzRasterizer::new(8, 8);
        let bmp = rasterizer.rasterize(&outline);

        // Center pixels (3, 3), (4, 4) should be fully covered (255)
        assert_eq!(bmp.buffer[3 * 8 + 3], 255);
        assert_eq!(bmp.buffer[4 * 8 + 4], 255);
        // Outer pixels should be 0
        assert_eq!(bmp.buffer[0 * 8 + 0], 0);
    }
}

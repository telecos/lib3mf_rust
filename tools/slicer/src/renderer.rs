//! Slice rendering module for generating PNG images from contours
//!
//! Uses scanline rasterization with the nonzero winding rule to implement
//! the 3MF positive fill rule. All contours are rendered together so that
//! nested contours (outer boundaries and holes) are handled correctly.
//!
//! When color information is available, thick colored borders are drawn
//! along contour edges to visualize surface materials/textures.

use crate::color::{Rgba, lerp_color};
use image::{Rgb, RgbImage};
use std::path::Path;

/// 2D point for slice rendering
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point2D {
    pub x: f64,
    pub y: f64,
}

impl Point2D {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

/// A contour in a slice (closed polygon)
#[derive(Debug, Clone)]
pub struct SliceContour {
    pub points: Vec<Point2D>,
}

impl SliceContour {
    pub fn new(points: Vec<Point2D>) -> Self {
        Self { points }
    }
}

/// A contour with per-vertex RGBA colors for surface visualization
#[derive(Debug, Clone)]
pub struct ColoredContour {
    pub points: Vec<Point2D>,
    pub colors: Vec<Rgba>,
}

/// An edge of a contour in pixel space, used for scanline intersection
#[derive(Debug, Clone)]
struct Edge {
    /// X coordinate at the lower Y endpoint (in pixel space)
    x_at_y_min: f64,
    /// Y minimum (top of edge in image coords)
    y_min: f64,
    /// Y maximum (bottom of edge in image coords)
    y_max: f64,
    /// Inverse slope: dx/dy (how much x changes per unit y)
    inv_slope: f64,
    /// Direction: +1 if edge goes downward (increasing y), -1 if upward
    direction: i32,
}

/// Slice renderer for creating PNG images
///
/// Uses scanline fill with the nonzero winding rule (3MF positive fill rule).
/// All contours for a slice are processed together, so nested boundaries
/// and holes are rendered correctly.
pub struct SliceRenderer {
    width: u32,
    height: u32,
    scale_x: f64,
    scale_y: f64,
    offset_x: f64,
    offset_y: f64,
}

impl SliceRenderer {
    /// Create a new renderer with the given dimensions and printable box
    pub fn new(
        width: u32,
        height: u32,
        box_origin_x: f64,
        box_origin_y: f64,
        box_width: f64,
        box_height: f64,
    ) -> Self {
        let scale_x = width as f64 / box_width;
        let scale_y = height as f64 / box_height;

        Self {
            width,
            height,
            scale_x,
            scale_y,
            offset_x: box_origin_x,
            offset_y: box_origin_y,
        }
    }

    /// Transform a world coordinate to sub-pixel image coordinate (f64)
    fn world_to_pixel_f64(&self, x: f64, y: f64) -> (f64, f64) {
        let px = (x - self.offset_x) * self.scale_x;
        // Flip Y: image Y goes down, world Y goes up
        let py = (self.height as f64) - (y - self.offset_y) * self.scale_y;
        (px, py)
    }

    /// Build the list of edges from all contours (in pixel space)
    fn build_edges(&self, contours: &[SliceContour]) -> Vec<Edge> {
        let mut edges = Vec::new();

        for contour in contours {
            let n = contour.points.len();
            if n < 3 {
                continue;
            }

            for i in 0..n {
                let j = (i + 1) % n;
                let (x0, y0) = self.world_to_pixel_f64(contour.points[i].x, contour.points[i].y);
                let (x1, y1) = self.world_to_pixel_f64(contour.points[j].x, contour.points[j].y);

                // Skip horizontal edges (they don't contribute to scanline crossings)
                let dy = y1 - y0;
                if dy.abs() < 1e-10 {
                    continue;
                }

                // Direction: +1 if going downward (y increases), -1 if upward
                let direction = if dy > 0.0 { 1 } else { -1 };

                let (y_min, y_max, x_at_y_min) = if y0 < y1 { (y0, y1, x0) } else { (y1, y0, x1) };

                let inv_slope = (x1 - x0) / (y1 - y0);

                edges.push(Edge {
                    x_at_y_min,
                    y_min,
                    y_max,
                    inv_slope,
                    direction,
                });
            }
        }

        edges
    }

    /// Render contours to a PNG image using scanline fill with nonzero winding rule.
    ///
    /// If `colored_contours` is non-empty, draws thick colored borders along
    /// the contour edges to show surface material/texture colors.
    ///
    /// When `use_gray_fill` is true (model has color/material data), the solid
    /// interior is rendered in mid-gray so the colored borders stand out.
    pub fn render_to_file(
        &self,
        contours: &[SliceContour],
        colored_contours: &[ColoredContour],
        border_width: u32,
        use_gray_fill: bool,
        output_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut img = RgbImage::new(self.width, self.height);

        // Fill with white background
        for pixel in img.pixels_mut() {
            *pixel = Rgb([255, 255, 255]);
        }

        if contours.is_empty() {
            img.save(output_path)?;
            return Ok(());
        }

        // Use mid-gray for colored models so borders are visible, black otherwise
        let fill_color = if use_gray_fill {
            Rgb([128u8, 128, 128])
        } else {
            Rgb([0u8, 0, 0])
        };
        let edges = self.build_edges(contours);

        if edges.is_empty() {
            img.save(output_path)?;
            return Ok(());
        }

        // Process each scanline
        for y in 0..self.height {
            let scan_y = y as f64 + 0.5; // Sample at pixel center

            // Find all edge intersections with this scanline
            let mut intersections: Vec<(f64, i32)> = Vec::new();

            for edge in &edges {
                // Check if edge spans this scanline
                if scan_y >= edge.y_min && scan_y < edge.y_max {
                    let x = edge.x_at_y_min + (scan_y - edge.y_min) * edge.inv_slope;
                    intersections.push((x, edge.direction));
                }
            }

            // Sort intersections by x
            intersections
                .sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

            // Walk intersections and fill spans using nonzero winding rule
            let mut winding = 0i32;
            let mut i = 0;

            while i < intersections.len() {
                let (x_start, dir) = intersections[i];
                let prev_winding = winding;
                winding += dir;

                // If we transitioned from outside (winding == 0) to inside (winding != 0),
                // start a fill span
                if prev_winding == 0 && winding != 0 {
                    // Find where we transition back to outside
                    let fill_start = x_start;
                    i += 1;

                    while i < intersections.len() && winding != 0 {
                        winding += intersections[i].1;
                        if winding == 0 {
                            let fill_end = intersections[i].0;

                            // Fill pixels in [fill_start, fill_end]
                            let px_start = (fill_start.ceil() as i32).max(0) as u32;
                            let px_end =
                                (fill_end.floor() as i32).min(self.width as i32 - 1).max(0) as u32;

                            for px in px_start..=px_end {
                                img.put_pixel(px, y, fill_color);
                            }
                        }
                        i += 1;
                    }
                } else {
                    i += 1;
                }
            }
        }

        // Pass 2: draw thick colored borders along colored contour edges
        if !colored_contours.is_empty() && border_width > 0 {
            self.draw_colored_borders(&mut img, colored_contours, border_width);
        }

        img.save(output_path)?;
        Ok(())
    }

    /// Draw thick colored borders along colored contour edges.
    ///
    /// For each edge of each colored contour, walks along the edge in pixel
    /// space and paints all pixels within `border_width` of the edge with
    /// the interpolated edge color.
    fn draw_colored_borders(
        &self,
        img: &mut RgbImage,
        colored_contours: &[ColoredContour],
        border_width: u32,
    ) {
        let bw = border_width as f64;
        let bw_sq = bw * bw;

        for contour in colored_contours {
            let n = contour.points.len();
            if n < 2 || contour.colors.len() != n {
                continue;
            }

            for i in 0..n {
                let j = (i + 1) % n;
                let (px0, py0) = self.world_to_pixel_f64(contour.points[i].x, contour.points[i].y);
                let (px1, py1) = self.world_to_pixel_f64(contour.points[j].x, contour.points[j].y);
                let c0 = contour.colors[i];
                let c1 = contour.colors[j];

                // Determine bounding box of pixels to check
                let min_x = (px0.min(px1) - bw).floor().max(0.0) as u32;
                let max_x = (px0.max(px1) + bw).ceil().min(self.width as f64 - 1.0) as u32;
                let min_y = (py0.min(py1) - bw).floor().max(0.0) as u32;
                let max_y = (py0.max(py1) + bw).ceil().min(self.height as f64 - 1.0) as u32;

                let dx = px1 - px0;
                let dy = py1 - py0;
                let seg_len_sq = dx * dx + dy * dy;

                if seg_len_sq < 1e-10 {
                    // Degenerate edge (zero length) — paint a dot
                    let color = Rgb([c0[0], c0[1], c0[2]]);
                    for y in min_y..=max_y {
                        for x in min_x..=max_x {
                            let ddx = x as f64 + 0.5 - px0;
                            let ddy = y as f64 + 0.5 - py0;
                            if ddx * ddx + ddy * ddy <= bw_sq {
                                img.put_pixel(x, y, color);
                            }
                        }
                    }
                    continue;
                }

                for y in min_y..=max_y {
                    for x in min_x..=max_x {
                        let qx = x as f64 + 0.5;
                        let qy = y as f64 + 0.5;

                        // Project (qx,qy) onto the edge to get parameter t ∈ [0,1]
                        let t = ((qx - px0) * dx + (qy - py0) * dy) / seg_len_sq;
                        let t_clamped = t.clamp(0.0, 1.0);

                        // Closest point on edge
                        let cx = px0 + t_clamped * dx;
                        let cy = py0 + t_clamped * dy;
                        let dist_sq = (qx - cx) * (qx - cx) + (qy - cy) * (qy - cy);

                        if dist_sq <= bw_sq {
                            let color = lerp_color(c0, c1, t_clamped);
                            img.put_pixel(x, y, Rgb([color[0], color[1], color[2]]));
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slice_renderer_creation() {
        let renderer = SliceRenderer::new(1920, 1080, 0.0, 0.0, 200.0, 200.0);
        assert_eq!(renderer.width, 1920);
        assert_eq!(renderer.height, 1080);
    }

    #[test]
    fn test_world_to_pixel() {
        let renderer = SliceRenderer::new(1000, 1000, 0.0, 0.0, 100.0, 100.0);
        let (px, py) = renderer.world_to_pixel_f64(50.0, 50.0);
        assert!((px - 500.0).abs() < 1.0);
        assert!((py - 500.0).abs() < 1.0);
    }

    #[test]
    fn test_renders_square_contour() {
        // A simple square contour should produce filled pixels
        let renderer = SliceRenderer::new(100, 100, 0.0, 0.0, 100.0, 100.0);
        let contour = SliceContour::new(vec![
            Point2D::new(20.0, 20.0),
            Point2D::new(80.0, 20.0),
            Point2D::new(80.0, 80.0),
            Point2D::new(20.0, 80.0),
        ]);

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.png");
        renderer
            .render_to_file(&[contour], &[], 0, false, &path)
            .unwrap();

        let img = image::open(&path).unwrap().to_rgb8();
        // Center pixel should be black (filled)
        assert_eq!(img.get_pixel(50, 50), &Rgb([0, 0, 0]));
        // Corner pixel should be white (outside)
        assert_eq!(img.get_pixel(5, 5), &Rgb([255, 255, 255]));
    }

    #[test]
    fn test_nonzero_winding_rule() {
        // An outer square with an inner square hole (opposite winding)
        let renderer = SliceRenderer::new(100, 100, 0.0, 0.0, 100.0, 100.0);

        // Outer contour: counterclockwise (in world coords)
        let outer = SliceContour::new(vec![
            Point2D::new(10.0, 10.0),
            Point2D::new(90.0, 10.0),
            Point2D::new(90.0, 90.0),
            Point2D::new(10.0, 90.0),
        ]);

        // Inner contour: clockwise (opposite winding = hole)
        let inner = SliceContour::new(vec![
            Point2D::new(40.0, 40.0),
            Point2D::new(40.0, 60.0),
            Point2D::new(60.0, 60.0),
            Point2D::new(60.0, 40.0),
        ]);

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test_hole.png");
        renderer
            .render_to_file(&[outer, inner], &[], 0, false, &path)
            .unwrap();

        let img = image::open(&path).unwrap().to_rgb8();
        // Pixel inside outer but outside inner should be black
        assert_eq!(img.get_pixel(25, 50), &Rgb([0, 0, 0]));
    }
}

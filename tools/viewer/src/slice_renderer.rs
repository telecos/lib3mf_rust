//! 2D Slice Image Renderer
//!
//! This module provides functionality to convert slice contours into raster images (PNG)
//! with filled polygons. It supports:
//! - White background for clean, high-contrast output
//! - Configurable resolution
//! - Auto-fit with aspect ratio preservation
//! - Filled polygons with proper hole handling
//! - Multiple objects with different colors
//! - Triangle-based rasterization for accurate fills

#![forbid(unsafe_code)]

use image::{ImageBuffer, Rgb, RgbImage};
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

/// 2D bounding box
#[derive(Debug, Clone, Copy)]
pub struct BoundingBox2D {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

impl BoundingBox2D {
    pub fn new(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Self {
        Self {
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }

    pub fn width(&self) -> f64 {
        self.max_x - self.min_x
    }

    pub fn height(&self) -> f64 {
        self.max_y - self.min_y
    }

    pub fn min(&self) -> Point2D {
        Point2D::new(self.min_x, self.min_y)
    }
}

/// A contour in a slice, which may contain holes
#[derive(Debug, Clone)]
pub struct SliceContour {
    /// The outer boundary of the contour
    pub boundary: Vec<Point2D>,
    /// Inner holes (each is a polygon that should remain unfilled)
    pub holes: Vec<Vec<Point2D>>,
}

impl SliceContour {
    pub fn new(boundary: Vec<Point2D>) -> Self {
        Self {
            boundary,
            holes: Vec::new(),
        }
    }

    pub fn with_holes(boundary: Vec<Point2D>, holes: Vec<Vec<Point2D>>) -> Self {
        Self { boundary, holes }
    }
}

/// 2D coordinate transformation helper
pub struct Transform2D {
    scale: f64,
    offset_x: f64,
    offset_y: f64,
    origin: Point2D,
}

impl Transform2D {
    pub fn new(scale: f64, offset_x: f64, offset_y: f64, origin: Point2D) -> Self {
        Self {
            scale,
            offset_x,
            offset_y,
            origin,
        }
    }

    /// Apply transformation to convert world coordinates to pixel coordinates
    pub fn apply(&self, p: Point2D) -> (f64, f64) {
        let x = (p.x - self.origin.x) * self.scale + self.offset_x;
        let y = (p.y - self.origin.y) * self.scale + self.offset_y;
        (x, y)
    }
}

/// 2D Slice Image Renderer
///
/// Converts slice contours into raster images with filled polygons.
pub struct SliceRenderer {
    width: u32,
    height: u32,
    margin: f64,
    background: Rgb<u8>,
    default_fill: Rgb<u8>,
}

impl SliceRenderer {
    /// Create a new slice renderer with the specified dimensions
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            margin: 20.0,
            background: Rgb([255, 255, 255]), // White
            default_fill: Rgb([80, 80, 80]),   // Dark gray
        }
    }

    /// Set the margin in pixels
    pub fn with_margin(mut self, margin: f64) -> Self {
        self.margin = margin;
        self
    }

    /// Set the background color
    pub fn with_background(mut self, color: Rgb<u8>) -> Self {
        self.background = color;
        self
    }

    /// Set the default fill color
    pub fn with_default_fill(mut self, color: Rgb<u8>) -> Self {
        self.default_fill = color;
        self
    }

    /// Render a slice with contours to an image
    pub fn render_slice(&self, contours: &[SliceContour], bounds: &BoundingBox2D) -> RgbImage {
        let mut image = ImageBuffer::from_pixel(self.width, self.height, self.background);
        let transform = self.calculate_transform(bounds);

        for contour in contours {
            self.render_contour(&mut image, contour, &transform, self.default_fill);
        }

        image
    }

    /// Render a slice with multiple colored objects
    pub fn render_slice_multi_color(
        &self,
        objects: &[(SliceContour, Rgb<u8>)],
        bounds: &BoundingBox2D,
    ) -> RgbImage {
        let mut image = ImageBuffer::from_pixel(self.width, self.height, self.background);
        let transform = self.calculate_transform(bounds);

        for (contour, color) in objects {
            self.render_contour(&mut image, contour, &transform, *color);
        }

        image
    }

    /// Calculate transformation from world to pixel coordinates
    fn calculate_transform(&self, bounds: &BoundingBox2D) -> Transform2D {
        let available_width = self.width as f64 - 2.0 * self.margin;
        let available_height = self.height as f64 - 2.0 * self.margin;

        let scale_x = available_width / bounds.width();
        let scale_y = available_height / bounds.height();
        let scale = scale_x.min(scale_y); // Uniform scale to preserve aspect ratio

        let offset_x = self.margin + (available_width - bounds.width() * scale) / 2.0;
        let offset_y = self.margin + (available_height - bounds.height() * scale) / 2.0;

        Transform2D::new(scale, offset_x, offset_y, bounds.min())
    }

    /// Render a single contour with holes
    fn render_contour(
        &self,
        image: &mut RgbImage,
        contour: &SliceContour,
        transform: &Transform2D,
        color: Rgb<u8>,
    ) {
        // Triangulate the contour (with holes)
        let triangles = self.triangulate_with_holes(&contour.boundary, &contour.holes);

        // Fill each triangle
        for triangle in &triangles {
            self.fill_triangle(image, triangle, transform, color);
        }
    }

    /// Triangulate a polygon with holes using ear clipping
    fn triangulate_with_holes(
        &self,
        boundary: &[Point2D],
        holes: &[Vec<Point2D>],
    ) -> Vec<[Point2D; 3]> {
        if boundary.len() < 3 {
            return Vec::new();
        }

        // Convert to flat f64 array for earcutr
        let mut vertices = Vec::new();
        for point in boundary {
            vertices.push(point.x);
            vertices.push(point.y);
        }

        // Add holes and track their starting indices
        let mut hole_indices = Vec::new();
        for hole in holes {
            if hole.len() >= 3 {
                hole_indices.push(vertices.len() / 2);
                for point in hole {
                    vertices.push(point.x);
                    vertices.push(point.y);
                }
            }
        }

        // Triangulate
        let triangle_indices = match earcutr::earcut(&vertices, &hole_indices, 2) {
            Ok(indices) => indices,
            Err(_) => return Vec::new(), // Return empty on triangulation error
        };

        // Convert indices to triangles
        let mut triangles = Vec::new();
        for chunk in triangle_indices.chunks(3) {
            if chunk.len() == 3 {
                let i0 = chunk[0];
                let i1 = chunk[1];
                let i2 = chunk[2];

                let p0 = Point2D::new(vertices[i0 * 2], vertices[i0 * 2 + 1]);
                let p1 = Point2D::new(vertices[i1 * 2], vertices[i1 * 2 + 1]);
                let p2 = Point2D::new(vertices[i2 * 2], vertices[i2 * 2 + 1]);

                triangles.push([p0, p1, p2]);
            }
        }

        triangles
    }

    /// Fill a triangle using scanline rasterization
    fn fill_triangle(
        &self,
        image: &mut RgbImage,
        triangle: &[Point2D; 3],
        transform: &Transform2D,
        color: Rgb<u8>,
    ) {
        // Transform vertices to pixel space
        let p0 = transform.apply(triangle[0]);
        let p1 = transform.apply(triangle[1]);
        let p2 = transform.apply(triangle[2]);

        // Compute bounding box
        let min_x = p0.0.min(p1.0).min(p2.0).floor().max(0.0) as u32;
        let max_x = p0
            .0
            .max(p1.0)
            .max(p2.0)
            .ceil()
            .min(self.width as f64 - 1.0) as u32;
        let min_y = p0.1.min(p1.1).min(p2.1).floor().max(0.0) as u32;
        let max_y = p0
            .1
            .max(p1.1)
            .max(p2.1)
            .ceil()
            .min(self.height as f64 - 1.0) as u32;

        // Scanline fill using point-in-triangle test
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let px = x as f64 + 0.5; // Pixel center
                let py = y as f64 + 0.5;

                if point_in_triangle(px, py, p0, p1, p2) {
                    image.put_pixel(x, y, color);
                }
            }
        }
    }

    /// Save the image to a PNG file
    pub fn save_png(&self, image: &RgbImage, path: &Path) -> Result<(), image::ImageError> {
        image.save(path)
    }
}

/// Check if a point is inside a triangle using barycentric coordinates
fn point_in_triangle(
    px: f64,
    py: f64,
    p0: (f64, f64),
    p1: (f64, f64),
    p2: (f64, f64),
) -> bool {
    // Barycentric coordinate method
    let v0x = p2.0 - p0.0;
    let v0y = p2.1 - p0.1;
    let v1x = p1.0 - p0.0;
    let v1y = p1.1 - p0.1;
    let v2x = px - p0.0;
    let v2y = py - p0.1;

    let d00 = v0x * v0x + v0y * v0y;
    let d01 = v0x * v1x + v0y * v1y;
    let d11 = v1x * v1x + v1y * v1y;
    let d20 = v2x * v0x + v2y * v0y;
    let d21 = v2x * v1x + v2y * v1y;

    let denom = d00 * d11 - d01 * d01;
    if denom.abs() < 1e-10 {
        return false; // Degenerate triangle
    }

    let v = (d11 * d20 - d01 * d21) / denom;
    let w = (d00 * d21 - d01 * d20) / denom;
    let u = 1.0 - v - w;

    // Point is inside if all barycentric coordinates are non-negative
    u >= -1e-10 && v >= -1e-10 && w >= -1e-10
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bounding_box() {
        let bbox = BoundingBox2D::new(0.0, 0.0, 100.0, 50.0);
        assert_eq!(bbox.width(), 100.0);
        assert_eq!(bbox.height(), 50.0);
    }

    #[test]
    fn test_transform() {
        let origin = Point2D::new(0.0, 0.0);
        let transform = Transform2D::new(2.0, 10.0, 20.0, origin);

        let p = Point2D::new(5.0, 3.0);
        let (x, y) = transform.apply(p);
        assert_eq!(x, 20.0); // 5.0 * 2.0 + 10.0
        assert_eq!(y, 26.0); // 3.0 * 2.0 + 20.0
    }

    #[test]
    fn test_point_in_triangle() {
        let p0 = (0.0, 0.0);
        let p1 = (10.0, 0.0);
        let p2 = (5.0, 10.0);

        // Point inside
        assert!(point_in_triangle(5.0, 3.0, p0, p1, p2));

        // Point outside
        assert!(!point_in_triangle(15.0, 5.0, p0, p1, p2));

        // Point on vertex
        assert!(point_in_triangle(0.0, 0.0, p0, p1, p2));

        // Point on edge
        assert!(point_in_triangle(5.0, 0.0, p0, p1, p2));
    }

    #[test]
    fn test_renderer_creation() {
        let renderer = SliceRenderer::new(1024, 1024);
        assert_eq!(renderer.width, 1024);
        assert_eq!(renderer.height, 1024);
        assert_eq!(renderer.background, Rgb([255, 255, 255]));
    }

    #[test]
    fn test_renderer_with_custom_settings() {
        let renderer = SliceRenderer::new(800, 600)
            .with_margin(50.0)
            .with_background(Rgb([200, 200, 200]))
            .with_default_fill(Rgb([50, 50, 50]));

        assert_eq!(renderer.margin, 50.0);
        assert_eq!(renderer.background, Rgb([200, 200, 200]));
        assert_eq!(renderer.default_fill, Rgb([50, 50, 50]));
    }

    #[test]
    fn test_triangulate_simple_square() {
        let renderer = SliceRenderer::new(100, 100);

        // Simple square
        let boundary = vec![
            Point2D::new(0.0, 0.0),
            Point2D::new(10.0, 0.0),
            Point2D::new(10.0, 10.0),
            Point2D::new(0.0, 10.0),
        ];

        let triangles = renderer.triangulate_with_holes(&boundary, &[]);

        // Square should be triangulated into 2 triangles
        assert_eq!(triangles.len(), 2);
    }

    #[test]
    fn test_triangulate_with_hole() {
        let renderer = SliceRenderer::new(100, 100);

        // Outer square
        let boundary = vec![
            Point2D::new(0.0, 0.0),
            Point2D::new(20.0, 0.0),
            Point2D::new(20.0, 20.0),
            Point2D::new(0.0, 20.0),
        ];

        // Inner square (hole)
        let hole = vec![
            Point2D::new(5.0, 5.0),
            Point2D::new(15.0, 5.0),
            Point2D::new(15.0, 15.0),
            Point2D::new(5.0, 15.0),
        ];

        let triangles = renderer.triangulate_with_holes(&boundary, &[hole]);

        // Should produce triangles (exact count depends on triangulation algorithm)
        assert!(triangles.len() > 2);
    }

    #[test]
    fn test_render_simple_triangle() {
        let renderer = SliceRenderer::new(100, 100);

        let contour = SliceContour::new(vec![
            Point2D::new(0.0, 0.0),
            Point2D::new(10.0, 0.0),
            Point2D::new(5.0, 10.0),
        ]);

        let bounds = BoundingBox2D::new(0.0, 0.0, 10.0, 10.0);

        let image = renderer.render_slice(&[contour], &bounds);

        // Image should be created with correct dimensions
        assert_eq!(image.width(), 100);
        assert_eq!(image.height(), 100);

        // Background should be white
        assert_eq!(image.get_pixel(0, 0), &Rgb([255, 255, 255]));
    }

    #[test]
    fn test_render_multi_color() {
        let renderer = SliceRenderer::new(200, 200);

        let contour1 = SliceContour::new(vec![
            Point2D::new(0.0, 0.0),
            Point2D::new(10.0, 0.0),
            Point2D::new(5.0, 10.0),
        ]);

        let contour2 = SliceContour::new(vec![
            Point2D::new(15.0, 0.0),
            Point2D::new(25.0, 0.0),
            Point2D::new(20.0, 10.0),
        ]);

        let objects = vec![
            (contour1, Rgb([255, 0, 0])),   // Red
            (contour2, Rgb([0, 0, 255])),   // Blue
        ];

        let bounds = BoundingBox2D::new(0.0, 0.0, 25.0, 10.0);

        let image = renderer.render_slice_multi_color(&objects, &bounds);

        assert_eq!(image.width(), 200);
        assert_eq!(image.height(), 200);
    }
}

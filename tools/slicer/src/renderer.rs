//! Slice rendering module for generating PNG images from contours

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

/// Slice renderer for creating PNG images
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
        // Calculate scale to fit the printable box in the image
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
    
    /// Transform a world coordinate to image pixel coordinate
    fn world_to_pixel(&self, x: f64, y: f64) -> (i32, i32) {
        let px = ((x - self.offset_x) * self.scale_x) as i32;
        let py = ((y - self.offset_y) * self.scale_y) as i32;
        
        // Flip Y coordinate (image Y goes down, world Y goes up)
        let py = self.height as i32 - 1 - py;
        
        (px, py)
    }
    
    /// Render contours to a PNG image
    pub fn render_to_file(
        &self,
        contours: &[SliceContour],
        output_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut img = RgbImage::new(self.width, self.height);
        
        // Fill with white background
        for pixel in img.pixels_mut() {
            *pixel = Rgb([255, 255, 255]);
        }
        
        // Render each contour
        for contour in contours {
            self.render_contour(&mut img, contour);
        }
        
        img.save(output_path)?;
        Ok(())
    }
    
    /// Render a single contour (filled polygon)
    fn render_contour(&self, img: &mut RgbImage, contour: &SliceContour) {
        if contour.points.len() < 3 {
            return; // Need at least 3 points for a polygon
        }
        
        // Convert points to pixel coordinates
        let pixel_points: Vec<(i32, i32)> = contour
            .points
            .iter()
            .map(|p| self.world_to_pixel(p.x, p.y))
            .collect();
        
        // Triangulate the polygon using earcutr
        let flat_coords: Vec<f64> = pixel_points
            .iter()
            .flat_map(|(x, y)| vec![*x as f64, *y as f64])
            .collect();
        
        if flat_coords.len() < 6 {
            return;
        }
        
        // Get triangle indices
        let indices = match earcutr::earcut(&flat_coords, &[], 2) {
            Ok(indices) => indices,
            Err(_) => return, // Skip malformed polygons
        };
        
        // Render each triangle
        for triangle_indices in indices.chunks(3) {
            if triangle_indices.len() == 3 {
                let i0 = triangle_indices[0];
                let i1 = triangle_indices[1];
                let i2 = triangle_indices[2];
                
                if i0 < pixel_points.len() && i1 < pixel_points.len() && i2 < pixel_points.len() {
                    self.fill_triangle(
                        img,
                        pixel_points[i0],
                        pixel_points[i1],
                        pixel_points[i2],
                        Rgb([0, 0, 0]), // Black fill
                    );
                }
            }
        }
    }
    
    /// Fill a triangle using barycentric coordinates
    fn fill_triangle(
        &self,
        img: &mut RgbImage,
        p0: (i32, i32),
        p1: (i32, i32),
        p2: (i32, i32),
        color: Rgb<u8>,
    ) {
        // Calculate bounding box
        let min_x = p0.0.min(p1.0).min(p2.0).max(0);
        let max_x = p0.0.max(p1.0).max(p2.0).min(self.width as i32 - 1);
        let min_y = p0.1.min(p1.1).min(p2.1).max(0);
        let max_y = p0.1.max(p1.1).max(p2.1).min(self.height as i32 - 1);
        
        // Helper function for edge test
        let edge = |ax: i32, ay: i32, bx: i32, by: i32, px: i32, py: i32| -> i32 {
            (px - ax) * (by - ay) - (py - ay) * (bx - ax)
        };
        
        // Rasterize triangle
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let w0 = edge(p1.0, p1.1, p2.0, p2.1, x, y);
                let w1 = edge(p2.0, p2.1, p0.0, p0.1, x, y);
                let w2 = edge(p0.0, p0.1, p1.0, p1.1, x, y);
                
                // Point is inside triangle if all weights have same sign
                if (w0 >= 0 && w1 >= 0 && w2 >= 0) || (w0 <= 0 && w1 <= 0 && w2 <= 0) {
                    if let Some(pixel) = img.get_pixel_mut_checked(x as u32, y as u32) {
                        *pixel = color;
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
        let (px, py) = renderer.world_to_pixel(50.0, 50.0);
        assert!(px >= 450 && px <= 550); // Approximately center
        assert!(py >= 450 && py <= 550);
    }
}

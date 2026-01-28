//! Demo program for the slice renderer
//!
//! This example demonstrates how to use the SliceRenderer to create
//! raster images from slice contours.

#![forbid(unsafe_code)]

// Import from the parent crate
use std::path::Path;

// We need to replicate the necessary types since this is an example
use image::Rgb;

#[path = "../src/slice_renderer.rs"]
mod slice_renderer;

use slice_renderer::{BoundingBox2D, Point2D, SliceContour, SliceRenderer};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Slice Renderer Demo");
    println!("===================\n");

    // Example 1: Simple triangle
    println!("Example 1: Rendering a simple triangle...");
    render_simple_triangle()?;
    println!("  ✓ Saved to /tmp/triangle.png\n");

    // Example 2: Square with a hole
    println!("Example 2: Rendering a square with a circular hole...");
    render_square_with_hole()?;
    println!("  ✓ Saved to /tmp/square_with_hole.png\n");

    // Example 3: Multiple colored objects
    println!("Example 3: Rendering multiple colored objects...");
    render_multi_color()?;
    println!("  ✓ Saved to /tmp/multi_color.png\n");

    println!("Demo complete! Check /tmp for generated PNG files.");

    Ok(())
}

fn render_simple_triangle() -> Result<(), Box<dyn std::error::Error>> {
    let renderer = SliceRenderer::new(800, 800);

    let contour = SliceContour::new(vec![
        Point2D::new(10.0, 10.0),
        Point2D::new(90.0, 10.0),
        Point2D::new(50.0, 80.0),
    ]);

    let bounds = BoundingBox2D::new(0.0, 0.0, 100.0, 100.0);
    let image = renderer.render_slice(&[contour], &bounds);

    renderer.save_png(&image, Path::new("/tmp/triangle.png"))?;

    Ok(())
}

fn render_square_with_hole() -> Result<(), Box<dyn std::error::Error>> {
    let renderer = SliceRenderer::new(1024, 1024);

    // Outer square
    let boundary = vec![
        Point2D::new(10.0, 10.0),
        Point2D::new(90.0, 10.0),
        Point2D::new(90.0, 90.0),
        Point2D::new(10.0, 90.0),
    ];

    // Inner hole (approximating a circle with octagon)
    let hole = create_circle(50.0, 50.0, 20.0, 16);

    let contour = SliceContour::with_holes(boundary, vec![hole]);

    let bounds = BoundingBox2D::new(0.0, 0.0, 100.0, 100.0);
    let image = renderer.render_slice(&[contour], &bounds);

    renderer.save_png(&image, Path::new("/tmp/square_with_hole.png"))?;

    Ok(())
}

fn render_multi_color() -> Result<(), Box<dyn std::error::Error>> {
    let renderer = SliceRenderer::new(1200, 800).with_margin(30.0);

    // Red triangle
    let triangle = SliceContour::new(vec![
        Point2D::new(20.0, 20.0),
        Point2D::new(60.0, 20.0),
        Point2D::new(40.0, 60.0),
    ]);

    // Blue square
    let square = SliceContour::new(vec![
        Point2D::new(80.0, 20.0),
        Point2D::new(120.0, 20.0),
        Point2D::new(120.0, 60.0),
        Point2D::new(80.0, 60.0),
    ]);

    // Green pentagon
    let pentagon = create_circle(170.0, 40.0, 20.0, 5);
    let pentagon_contour = SliceContour::new(pentagon);

    let objects = vec![
        (triangle, Rgb([220, 60, 60])),      // Red
        (square, Rgb([60, 120, 220])),       // Blue
        (pentagon_contour, Rgb([60, 200, 80])), // Green
    ];

    let bounds = BoundingBox2D::new(0.0, 0.0, 200.0, 80.0);
    let image = renderer.render_slice_multi_color(&objects, &bounds);

    renderer.save_png(&image, Path::new("/tmp/multi_color.png"))?;

    Ok(())
}

/// Helper function to create a circle (or polygon approximation)
fn create_circle(cx: f64, cy: f64, radius: f64, segments: usize) -> Vec<Point2D> {
    let mut points = Vec::new();
    for i in 0..segments {
        let angle = 2.0 * std::f64::consts::PI * i as f64 / segments as f64;
        let x = cx + radius * angle.cos();
        let y = cy + radius * angle.sin();
        points.push(Point2D::new(x, y));
    }
    points
}

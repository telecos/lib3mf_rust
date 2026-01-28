//! Resolution comparison demo
//!
//! This example demonstrates the slice renderer at different resolutions
//! to show quality differences.

#![forbid(unsafe_code)]

use std::path::Path;
use image::Rgb;

#[path = "../src/slice_renderer.rs"]
mod slice_renderer;

use slice_renderer::{BoundingBox2D, Point2D, SliceContour, SliceRenderer};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Slice Renderer - Resolution Comparison Demo");
    println!("============================================\n");

    // Create a complex shape - a star
    let star_points = create_star(50.0, 50.0, 40.0, 15.0, 5);
    let star_contour = SliceContour::new(star_points);
    
    let bounds = BoundingBox2D::new(0.0, 0.0, 100.0, 100.0);

    // Render at different resolutions
    let resolutions = vec![
        (256, "low"),
        (512, "medium"),
        (1024, "high"),
        (2048, "very_high"),
    ];

    for (size, label) in resolutions {
        println!("Rendering at {}x{} ({})...", size, size, label);
        
        let renderer = SliceRenderer::new(size, size);
        let image = renderer.render_slice(&[star_contour.clone()], &bounds);
        
        let filename = format!("/tmp/star_{}.png", label);
        renderer.save_png(&image, Path::new(&filename))?;
        
        println!("  ✓ Saved to {}", filename);
    }

    println!("\nRendering complex multi-object scene...");
    render_complex_scene()?;
    println!("  ✓ Saved to /tmp/complex_scene.png");

    println!("\nDemo complete! Check /tmp for generated PNG files.");

    Ok(())
}

/// Create a star polygon
fn create_star(cx: f64, cy: f64, outer_radius: f64, inner_radius: f64, points: usize) -> Vec<Point2D> {
    let mut vertices = Vec::new();
    let angle_step = std::f64::consts::PI / points as f64;
    
    for i in 0..(points * 2) {
        let angle = i as f64 * angle_step - std::f64::consts::PI / 2.0;
        let radius = if i % 2 == 0 { outer_radius } else { inner_radius };
        let x = cx + radius * angle.cos();
        let y = cy + radius * angle.sin();
        vertices.push(Point2D::new(x, y));
    }
    
    vertices
}

/// Create a complex scene with multiple objects
fn render_complex_scene() -> Result<(), Box<dyn std::error::Error>> {
    let renderer = SliceRenderer::new(1600, 1200).with_margin(40.0);

    // Create various shapes
    let star1 = create_star(50.0, 50.0, 30.0, 12.0, 5);
    let star2 = create_star(150.0, 50.0, 25.0, 10.0, 6);
    let star3 = create_star(100.0, 120.0, 35.0, 14.0, 7);

    // Create a hexagon
    let hexagon = create_regular_polygon(50.0, 180.0, 25.0, 6);
    
    // Create a pentagon
    let pentagon = create_regular_polygon(150.0, 180.0, 25.0, 5);

    let objects = vec![
        (SliceContour::new(star1), Rgb([255, 100, 100])),      // Red star
        (SliceContour::new(star2), Rgb([100, 100, 255])),      // Blue star
        (SliceContour::new(star3), Rgb([100, 255, 100])),      // Green star
        (SliceContour::new(hexagon), Rgb([255, 200, 100])),    // Orange hexagon
        (SliceContour::new(pentagon), Rgb([200, 100, 255])),   // Purple pentagon
    ];

    let bounds = BoundingBox2D::new(0.0, 0.0, 200.0, 220.0);
    let image = renderer.render_slice_multi_color(&objects, &bounds);

    renderer.save_png(&image, Path::new("/tmp/complex_scene.png"))?;

    Ok(())
}

/// Create a regular polygon
fn create_regular_polygon(cx: f64, cy: f64, radius: f64, sides: usize) -> Vec<Point2D> {
    let mut vertices = Vec::new();
    let angle_step = 2.0 * std::f64::consts::PI / sides as f64;
    
    for i in 0..sides {
        let angle = i as f64 * angle_step - std::f64::consts::PI / 2.0;
        let x = cx + radius * angle.cos();
        let y = cy + radius * angle.sin();
        vertices.push(Point2D::new(x, y));
    }
    
    vertices
}

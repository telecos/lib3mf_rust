//! Example demonstrating polygon triangulation for filled 2D rendering
//!
//! This example shows how to use the polygon_triangulation module to convert
//! 2D polygon contours (with or without holes) into triangles suitable for
//! rendering with graphics APIs or rasterization.

use lib3mf::model::Vertex2D;
use lib3mf::polygon_triangulation::{triangulate_simple, triangulate_with_holes};

fn main() {
    println!("Polygon Triangulation Demo");
    println!("==========================\n");

    // Demo 1: Triangulate a simple square
    demo_simple_square();

    // Demo 2: Triangulate a complex polygon (pentagon)
    demo_pentagon();

    // Demo 3: Triangulate a polygon with a single hole
    demo_polygon_with_hole();

    // Demo 4: Triangulate a polygon with multiple holes
    demo_polygon_with_multiple_holes();

    println!("\n✅ All triangulation operations completed successfully!");
}

fn demo_simple_square() {
    println!("1. Simple Square Triangulation");
    println!("   ---------------------------");

    // Create a simple 10x10 square
    let vertices = vec![
        Vertex2D::new(0.0, 0.0),
        Vertex2D::new(10.0, 0.0),
        Vertex2D::new(10.0, 10.0),
        Vertex2D::new(0.0, 10.0),
    ];

    println!("   Input: Square with {} vertices", vertices.len());

    match triangulate_simple(&vertices) {
        Ok(triangles) => {
            println!("   Output: {} triangles", triangles.len() / 3);
            println!("   Triangle indices: {:?}", triangles);

            // Print triangles in readable format
            for i in (0..triangles.len()).step_by(3) {
                println!(
                    "   Triangle {}: ({}, {}, {})",
                    i / 3,
                    triangles[i],
                    triangles[i + 1],
                    triangles[i + 2]
                );
            }
        }
        Err(e) => {
            println!("   Error: {}", e);
        }
    }
    println!();
}

fn demo_pentagon() {
    println!("2. Pentagon Triangulation");
    println!("   ----------------------");

    // Create a regular pentagon
    let vertices = vec![
        Vertex2D::new(0.0, 5.0),
        Vertex2D::new(4.75, 1.54),
        Vertex2D::new(2.94, -4.05),
        Vertex2D::new(-2.94, -4.05),
        Vertex2D::new(-4.75, 1.54),
    ];

    println!("   Input: Pentagon with {} vertices", vertices.len());

    match triangulate_simple(&vertices) {
        Ok(triangles) => {
            println!("   Output: {} triangles", triangles.len() / 3);
            println!("   Triangle indices: {:?}", triangles);
        }
        Err(e) => {
            println!("   Error: {}", e);
        }
    }
    println!();
}

fn demo_polygon_with_hole() {
    println!("3. Polygon with One Hole");
    println!("   ---------------------");

    // Create a 100x100 square with a triangular hole in the center
    let outer = vec![
        Vertex2D::new(0.0, 0.0),
        Vertex2D::new(100.0, 0.0),
        Vertex2D::new(100.0, 100.0),
        Vertex2D::new(0.0, 100.0),
    ];

    // Triangular hole (clockwise)
    let hole = vec![
        Vertex2D::new(25.0, 25.0),
        Vertex2D::new(75.0, 25.0),
        Vertex2D::new(50.0, 75.0),
    ];

    println!("   Input: Square (4 vertices) with triangular hole (3 vertices)");

    match triangulate_with_holes(&outer, &[hole]) {
        Ok(triangles) => {
            println!("   Output: {} triangles", triangles.len() / 3);
            println!("   Total vertex count: {}", outer.len() + 3);
            println!("   Triangle indices: {:?}", triangles);

            // Note: indices 0-3 refer to outer vertices, 4-6 refer to hole vertices
            println!("\n   Vertex mapping:");
            println!("   - Indices 0-3: outer boundary");
            println!("   - Indices 4-6: hole vertices");
        }
        Err(e) => {
            println!("   Error: {}", e);
        }
    }
    println!();
}

fn demo_polygon_with_multiple_holes() {
    println!("4. Polygon with Multiple Holes");
    println!("   ---------------------------");

    // Create a large square with two smaller rectangular holes
    let outer = vec![
        Vertex2D::new(0.0, 0.0),
        Vertex2D::new(100.0, 0.0),
        Vertex2D::new(100.0, 100.0),
        Vertex2D::new(0.0, 100.0),
    ];

    // First hole (left side, clockwise)
    let hole1 = vec![
        Vertex2D::new(10.0, 10.0),
        Vertex2D::new(30.0, 10.0),
        Vertex2D::new(30.0, 30.0),
        Vertex2D::new(10.0, 30.0),
    ];

    // Second hole (right side, clockwise)
    let hole2 = vec![
        Vertex2D::new(70.0, 70.0),
        Vertex2D::new(90.0, 70.0),
        Vertex2D::new(90.0, 90.0),
        Vertex2D::new(70.0, 90.0),
    ];

    println!("   Input: Square (4 vertices) with 2 rectangular holes (4 vertices each)");

    match triangulate_with_holes(&outer, &[hole1, hole2]) {
        Ok(triangles) => {
            println!("   Output: {} triangles", triangles.len() / 3);
            println!("   Total vertex count: {}", outer.len() + 4 + 4);
            println!("   Number of triangle indices: {}", triangles.len());

            println!("\n   Vertex mapping:");
            println!("   - Indices 0-3:   outer boundary");
            println!("   - Indices 4-7:   first hole");
            println!("   - Indices 8-11:  second hole");

            // Show a few sample triangles
            println!("\n   Sample triangles:");
            for i in (0..triangles.len().min(15)).step_by(3) {
                println!(
                    "   Triangle {}: ({}, {}, {})",
                    i / 3,
                    triangles[i],
                    triangles[i + 1],
                    triangles[i + 2]
                );
            }
        }
        Err(e) => {
            println!("   Error: {}", e);
        }
    }
    println!();
}

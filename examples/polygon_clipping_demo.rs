//! Example demonstrating polygon clipping operations for resolving self-intersections
//!
//! This example shows how to use the polygon_clipping module to perform various
//! boolean operations on slice polygons, including resolving self-intersections.

use lib3mf::model::{SlicePolygon, SliceSegment, Vertex2D};
use lib3mf::polygon_clipping::{
    difference_polygons, intersect_polygons, resolve_self_intersections, union_polygons,
};

fn main() {
    println!("Polygon Clipping Demo for 3MF Slices");
    println!("=====================================\n");

    // Demo 1: Resolve self-intersections
    demo_resolve_self_intersections();

    // Demo 2: Union of overlapping polygons
    demo_union();

    // Demo 3: Intersection of overlapping polygons
    demo_intersection();

    // Demo 4: Difference between polygons
    demo_difference();

    println!("\n✅ All polygon clipping operations completed successfully!");
}

fn demo_resolve_self_intersections() {
    println!("1. Resolving Self-Intersections");
    println!("   -----------------------------");

    // Create a simple square polygon (no self-intersection)
    let vertices = vec![
        Vertex2D::new(0.0, 0.0),
        Vertex2D::new(100.0, 0.0),
        Vertex2D::new(100.0, 100.0),
        Vertex2D::new(0.0, 100.0),
    ];

    let mut polygon = SlicePolygon::new(0);
    polygon.segments.push(SliceSegment::new(1));
    polygon.segments.push(SliceSegment::new(2));
    polygon.segments.push(SliceSegment::new(3));

    println!("   Input: Square with {} vertices", vertices.len());

    let mut result_vertices = Vec::new();
    match resolve_self_intersections(&polygon, &vertices, &mut result_vertices) {
        Ok(result) => {
            println!(
                "   Output: {} polygon(s) with {} total vertices",
                result.len(),
                result_vertices.len()
            );
        }
        Err(e) => {
            println!("   Error: {}", e);
        }
    }
    println!();
}

fn demo_union() {
    println!("2. Union of Overlapping Polygons");
    println!("   ------------------------------");

    // Create two overlapping squares
    let vertices = vec![
        // First square: (0,0) to (100,100)
        Vertex2D::new(0.0, 0.0),
        Vertex2D::new(100.0, 0.0),
        Vertex2D::new(100.0, 100.0),
        Vertex2D::new(0.0, 100.0),
        // Second square: (50,50) to (150,150)
        Vertex2D::new(50.0, 50.0),
        Vertex2D::new(150.0, 50.0),
        Vertex2D::new(150.0, 150.0),
        Vertex2D::new(50.0, 150.0),
    ];

    let mut polygon1 = SlicePolygon::new(0);
    polygon1.segments.push(SliceSegment::new(1));
    polygon1.segments.push(SliceSegment::new(2));
    polygon1.segments.push(SliceSegment::new(3));

    let mut polygon2 = SlicePolygon::new(4);
    polygon2.segments.push(SliceSegment::new(5));
    polygon2.segments.push(SliceSegment::new(6));
    polygon2.segments.push(SliceSegment::new(7));

    println!("   Input: 2 overlapping squares");

    let mut result_vertices = Vec::new();
    match union_polygons(&[polygon1, polygon2], &vertices, &mut result_vertices) {
        Ok(result) => {
            println!(
                "   Output: {} unified polygon(s) with {} total vertices",
                result.len(),
                result_vertices.len()
            );
        }
        Err(e) => {
            println!("   Error: {}", e);
        }
    }
    println!();
}

fn demo_intersection() {
    println!("3. Intersection of Overlapping Polygons");
    println!("   ------------------------------------");

    // Create two overlapping squares
    let vertices = vec![
        // First square: (0,0) to (100,100)
        Vertex2D::new(0.0, 0.0),
        Vertex2D::new(100.0, 0.0),
        Vertex2D::new(100.0, 100.0),
        Vertex2D::new(0.0, 100.0),
        // Second square: (50,50) to (150,150)
        Vertex2D::new(50.0, 50.0),
        Vertex2D::new(150.0, 50.0),
        Vertex2D::new(150.0, 150.0),
        Vertex2D::new(50.0, 150.0),
    ];

    let mut polygon1 = SlicePolygon::new(0);
    polygon1.segments.push(SliceSegment::new(1));
    polygon1.segments.push(SliceSegment::new(2));
    polygon1.segments.push(SliceSegment::new(3));

    let mut polygon2 = SlicePolygon::new(4);
    polygon2.segments.push(SliceSegment::new(5));
    polygon2.segments.push(SliceSegment::new(6));
    polygon2.segments.push(SliceSegment::new(7));

    println!("   Input: 2 overlapping squares");
    println!("   Expected overlap region: (50,50) to (100,100)");

    let mut result_vertices = Vec::new();
    match intersect_polygons(&[polygon1], &[polygon2], &vertices, &mut result_vertices) {
        Ok(result) => {
            println!(
                "   Output: {} intersection polygon(s) with {} total vertices",
                result.len(),
                result_vertices.len()
            );
        }
        Err(e) => {
            println!("   Error: {}", e);
        }
    }
    println!();
}

fn demo_difference() {
    println!("4. Difference Between Polygons");
    println!("   ---------------------------");

    // Create two overlapping squares
    let vertices = vec![
        // First square: (0,0) to (100,100)
        Vertex2D::new(0.0, 0.0),
        Vertex2D::new(100.0, 0.0),
        Vertex2D::new(100.0, 100.0),
        Vertex2D::new(0.0, 100.0),
        // Second square: (50,50) to (150,150)
        Vertex2D::new(50.0, 50.0),
        Vertex2D::new(150.0, 50.0),
        Vertex2D::new(150.0, 150.0),
        Vertex2D::new(50.0, 150.0),
    ];

    let mut polygon1 = SlicePolygon::new(0);
    polygon1.segments.push(SliceSegment::new(1));
    polygon1.segments.push(SliceSegment::new(2));
    polygon1.segments.push(SliceSegment::new(3));

    let mut polygon2 = SlicePolygon::new(4);
    polygon2.segments.push(SliceSegment::new(5));
    polygon2.segments.push(SliceSegment::new(6));
    polygon2.segments.push(SliceSegment::new(7));

    println!("   Input: Square 1 minus Square 2");
    println!("   (Subtracting overlapping region from first square)");

    let mut result_vertices = Vec::new();
    match difference_polygons(&[polygon1], &[polygon2], &vertices, &mut result_vertices) {
        Ok(result) => {
            println!(
                "   Output: {} difference polygon(s) with {} total vertices",
                result.len(),
                result_vertices.len()
            );
        }
        Err(e) => {
            println!("   Error: {}", e);
        }
    }
    println!();
}

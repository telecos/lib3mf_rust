//! Demonstration of mesh-plane slicing with contour extraction
//!
//! This example shows how to:
//! 1. Load or create a 3D mesh
//! 2. Slice the mesh at various Z heights
//! 3. Extract closed contour loops from the slice
//! 4. Display the results
//!
//! Usage: cargo run --example mesh_slicing_demo [path/to/file.3mf] [z_height]

use lib3mf::{
    assemble_contours, collect_intersection_segments, triangle_plane_intersection, Mesh, Model,
    Triangle, Vertex,
};
use std::env;
use std::fs::File;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    // Either load a 3MF file or create a demo mesh
    let (mesh, z_height) = if args.len() >= 2 {
        // Load from file
        let file = File::open(&args[1])?;
        let model = Model::from_reader(file)?;

        // Get the first object's mesh
        let mesh = model
            .resources
            .objects
            .first()
            .and_then(|obj| obj.mesh.as_ref())
            .ok_or("No mesh found in 3MF file")?
            .clone();

        // Use provided Z height or default to middle of bounding box
        let z = if args.len() >= 3 {
            args[2].parse::<f64>()?
        } else {
            // Calculate middle Z from vertices
            let min_z = mesh
                .vertices
                .iter()
                .map(|v| v.z)
                .fold(f64::INFINITY, f64::min);
            let max_z = mesh
                .vertices
                .iter()
                .map(|v| v.z)
                .fold(f64::NEG_INFINITY, f64::max);
            (min_z + max_z) / 2.0
        };

        (mesh, z)
    } else {
        // Create a demo pyramid mesh
        let (mesh, z) = create_demo_pyramid();
        println!("No file provided - using demo pyramid mesh");
        (mesh, z)
    };

    println!("\n=== Mesh Slicing Demo ===\n");
    println!("Mesh statistics:");
    println!("  Vertices: {}", mesh.vertices.len());
    println!("  Triangles: {}", mesh.triangles.len());
    println!("  Slicing at Z = {:.2}", z_height);

    // Step 1: Demonstrate triangle-plane intersection
    println!("\n--- Step 1: Triangle-Plane Intersection ---");
    if !mesh.triangles.is_empty() && mesh.triangles.len() > 0 {
        let tri = &mesh.triangles[0];
        if tri.v1 < mesh.vertices.len()
            && tri.v2 < mesh.vertices.len()
            && tri.v3 < mesh.vertices.len()
        {
            let v0 = &mesh.vertices[tri.v1];
            let v1 = &mesh.vertices[tri.v2];
            let v2 = &mesh.vertices[tri.v3];

            println!(
                "Testing first triangle: v({:.2}, {:.2}, {:.2}), v({:.2}, {:.2}, {:.2}), v({:.2}, {:.2}, {:.2})",
                v0.x, v0.y, v0.z, v1.x, v1.y, v1.z, v2.x, v2.y, v2.z
            );

            match triangle_plane_intersection(v0, v1, v2, z_height) {
                Some((p1, p2)) => {
                    println!(
                        "  Intersection found: ({:.2}, {:.2}) to ({:.2}, {:.2})",
                        p1.0, p1.1, p2.0, p2.1
                    );
                }
                None => {
                    println!("  No intersection (triangle doesn't cross the Z plane)");
                }
            }
        }
    }

    // Step 2: Collect all intersection segments
    println!("\n--- Step 2: Segment Collection ---");
    let segments = collect_intersection_segments(&mesh, z_height);
    println!("Found {} intersection segments", segments.len());

    if segments.len() <= 10 {
        for (i, (p1, p2)) in segments.iter().enumerate() {
            println!(
                "  Segment {}: ({:.2}, {:.2}) to ({:.2}, {:.2})",
                i + 1,
                p1.0,
                p1.1,
                p2.0,
                p2.1
            );
        }
    } else {
        println!("  (showing first 5)");
        for (i, (p1, p2)) in segments.iter().take(5).enumerate() {
            println!(
                "  Segment {}: ({:.2}, {:.2}) to ({:.2}, {:.2})",
                i + 1,
                p1.0,
                p1.1,
                p2.0,
                p2.1
            );
        }
        println!("  ...");
    }

    // Step 3: Assemble contours
    println!("\n--- Step 3: Contour Assembly ---");
    let tolerance = 1e-6;
    let contours = assemble_contours(segments, tolerance);
    println!(
        "Assembled {} closed contour(s) with tolerance {:.0e}",
        contours.len(),
        tolerance
    );

    for (i, contour) in contours.iter().enumerate() {
        println!("\nContour {} ({} vertices):", i + 1, contour.len());

        if contour.len() <= 10 {
            for (j, point) in contour.iter().enumerate() {
                println!("  Point {}: ({:.2}, {:.2})", j + 1, point.0, point.1);
            }
        } else {
            println!("  (showing first and last 3 points)");
            for (j, point) in contour.iter().take(3).enumerate() {
                println!("  Point {}: ({:.2}, {:.2})", j + 1, point.0, point.1);
            }
            println!("  ...");
            for (j, point) in contour.iter().skip(contour.len() - 3).enumerate() {
                println!(
                    "  Point {}: ({:.2}, {:.2})",
                    contour.len() - 3 + j + 1,
                    point.0,
                    point.1
                );
            }
        }

        // Calculate contour area (for closed polygons)
        if contour.len() >= 3 {
            let area = calculate_polygon_area(contour);
            println!("  Area: {:.2} square units", area.abs());
        }
    }

    println!("\n=== Slicing Complete ===\n");

    Ok(())
}

/// Create a demo pyramid mesh for testing
fn create_demo_pyramid() -> (Mesh, f64) {
    let mut mesh = Mesh::new();

    // Base vertices (Z=0)
    mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0)); // 0
    mesh.vertices.push(Vertex::new(10.0, 0.0, 0.0)); // 1
    mesh.vertices.push(Vertex::new(10.0, 10.0, 0.0)); // 2
    mesh.vertices.push(Vertex::new(0.0, 10.0, 0.0)); // 3

    // Apex (Z=10)
    mesh.vertices.push(Vertex::new(5.0, 5.0, 10.0)); // 4

    // Base triangles
    mesh.triangles.push(Triangle::new(0, 2, 1));
    mesh.triangles.push(Triangle::new(0, 3, 2));

    // Side triangles
    mesh.triangles.push(Triangle::new(0, 1, 4));
    mesh.triangles.push(Triangle::new(1, 2, 4));
    mesh.triangles.push(Triangle::new(2, 3, 4));
    mesh.triangles.push(Triangle::new(3, 0, 4));

    // Slice at half height
    (mesh, 5.0)
}

/// Calculate the area of a 2D polygon using the shoelace formula
fn calculate_polygon_area(points: &[(f64, f64)]) -> f64 {
    if points.len() < 3 {
        return 0.0;
    }

    let mut area = 0.0;
    for i in 0..points.len() {
        let j = (i + 1) % points.len();
        area += points[i].0 * points[j].1;
        area -= points[j].0 * points[i].1;
    }

    area / 2.0
}

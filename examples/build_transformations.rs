//! Example: Working with build items and transformations
//!
//! This example demonstrates how to:
//! 1. Access build items in a 3MF file
//! 2. Parse and understand transformation matrices
//! 3. Apply transformations to geometry
//! 4. Create build plates with multiple instances
//!
//! Build items specify which objects to manufacture and can include transformations
//! for positioning, rotating, and scaling objects on the build plate.

use lib3mf::Model;
use std::env;
use std::fs::File;
use std::process;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <3mf-file>", args[0]);
        eprintln!();
        eprintln!("Demonstrates build items and transformation matrices");
        process::exit(1);
    }

    let filename = &args[1];

    println!("=== Build Items and Transformations ===");
    println!("File: {}", filename);
    println!();

    // Parse the 3MF file
    let file = File::open(filename)?;
    let model = Model::from_reader(file)?;

    println!("Model Information:");
    println!("  Unit: {}", model.unit);
    println!("  Objects: {}", model.resources.objects.len());
    println!("  Build items: {}", model.build.items.len());
    println!();

    if model.build.items.is_empty() {
        println!("⚠ No build items found in this file.");
        println!("   Build items specify which objects should be manufactured.");
        return Ok(());
    }

    // Analyze each build item
    for (idx, item) in model.build.items.iter().enumerate() {
        println!("─────────────────────────────────────");
        println!("Build Item {}:", idx);
        println!("  References Object ID: {}", item.objectid);

        // Find the referenced object
        let obj = model
            .resources
            .objects
            .iter()
            .find(|o| o.id == item.objectid);

        if let Some(obj) = obj {
            if let Some(ref name) = obj.name {
                println!("  Object Name: {}", name);
            }
            println!("  Object Type: {:?}", obj.object_type);

            if let Some(ref mesh) = obj.mesh {
                println!("  Mesh: {} vertices, {} triangles", 
                    mesh.vertices.len(), 
                    mesh.triangles.len()
                );

                // Calculate bounding box
                if !mesh.vertices.is_empty() {
                    let (min_x, max_x, min_y, max_y, min_z, max_z) = 
                        calculate_bounds(&mesh.vertices);
                    
                    println!("  Bounding box (before transformation):");
                    println!("    X: [{:.2}, {:.2}] (width: {:.2})", 
                        min_x, max_x, max_x - min_x);
                    println!("    Y: [{:.2}, {:.2}] (depth: {:.2})", 
                        min_y, max_y, max_y - min_y);
                    println!("    Z: [{:.2}, {:.2}] (height: {:.2})", 
                        min_z, max_z, max_z - min_z);
                }
            }
        } else {
            println!("  ⚠ Warning: Referenced object not found!");
        }

        // Analyze transformation
        if let Some(transform) = item.transform {
            println!();
            println!("  Transformation Matrix:");
            println!("    ┌                                          ┐");
            println!("    │ {:8.4} {:8.4} {:8.4} {:8.4} │", 
                transform[0], transform[1], transform[2], transform[3]);
            println!("    │ {:8.4} {:8.4} {:8.4} {:8.4} │", 
                transform[4], transform[5], transform[6], transform[7]);
            println!("    │ {:8.4} {:8.4} {:8.4} {:8.4} │", 
                transform[8], transform[9], transform[10], transform[11]);
            println!("    └                                          ┘");

            // Analyze transformation components
            analyze_transformation(&transform);

            // Show transformed bounding box if we have mesh data
            if let Some(obj) = obj {
                if let Some(ref mesh) = obj.mesh {
                    if !mesh.vertices.is_empty() {
                        let transformed_verts: Vec<_> = mesh.vertices
                            .iter()
                            .map(|v| apply_transform(v, &transform))
                            .collect();
                        
                        let (min_x, max_x, min_y, max_y, min_z, max_z) = 
                            calculate_bounds_from_points(&transformed_verts);
                        
                        println!();
                        println!("  Bounding box (after transformation):");
                        println!("    X: [{:.2}, {:.2}] (width: {:.2})", 
                            min_x, max_x, max_x - min_x);
                        println!("    Y: [{:.2}, {:.2}] (depth: {:.2})", 
                            min_y, max_y, max_y - min_y);
                        println!("    Z: [{:.2}, {:.2}] (height: {:.2})", 
                            min_z, max_z, max_z - min_z);
                    }
                }
            }
        } else {
            println!();
            println!("  Transformation: None (identity - no change)");
            println!("    Object will be placed at its original position");
        }
        println!();
    }

    // Summary
    println!("─────────────────────────────────────");
    println!("Build Plate Summary:");
    
    let items_with_transform = model.build.items.iter()
        .filter(|i| i.transform.is_some())
        .count();
    
    println!("  Total build items: {}", model.build.items.len());
    println!("  Items with transformations: {}", items_with_transform);
    println!("  Items without transformations: {}", 
        model.build.items.len() - items_with_transform);

    // Calculate overall build volume
    if let Some((min, max)) = calculate_build_volume(&model) {
        println!();
        println!("  Overall build volume:");
        println!("    X: [{:.2}, {:.2}] (width: {:.2})", 
            min.0, max.0, max.0 - min.0);
        println!("    Y: [{:.2}, {:.2}] (depth: {:.2})", 
            min.1, max.1, max.1 - min.1);
        println!("    Z: [{:.2}, {:.2}] (height: {:.2})", 
            min.2, max.2, max.2 - min.2);
    }

    Ok(())
}

/// Calculate bounding box from vertices
fn calculate_bounds(vertices: &[lib3mf::Vertex]) -> (f64, f64, f64, f64, f64, f64) {
    let mut min_x = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    let mut min_z = f64::INFINITY;
    let mut max_z = f64::NEG_INFINITY;

    for v in vertices {
        min_x = min_x.min(v.x);
        max_x = max_x.max(v.x);
        min_y = min_y.min(v.y);
        max_y = max_y.max(v.y);
        min_z = min_z.min(v.z);
        max_z = max_z.max(v.z);
    }

    (min_x, max_x, min_y, max_y, min_z, max_z)
}

/// Calculate bounding box from transformed points
fn calculate_bounds_from_points(points: &[(f64, f64, f64)]) -> (f64, f64, f64, f64, f64, f64) {
    let mut min_x = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    let mut min_z = f64::INFINITY;
    let mut max_z = f64::NEG_INFINITY;

    for p in points {
        min_x = min_x.min(p.0);
        max_x = max_x.max(p.0);
        min_y = min_y.min(p.1);
        max_y = max_y.max(p.1);
        min_z = min_z.min(p.2);
        max_z = max_z.max(p.2);
    }

    (min_x, max_x, min_y, max_y, min_z, max_z)
}

/// Apply a 3MF affine transformation matrix to a vertex
fn apply_transform(vertex: &lib3mf::Vertex, transform: &[f64; 12]) -> (f64, f64, f64) {
    let x = vertex.x;
    let y = vertex.y;
    let z = vertex.z;

    let tx = transform[0] * x + transform[1] * y + transform[2] * z + transform[3];
    let ty = transform[4] * x + transform[5] * y + transform[6] * z + transform[7];
    let tz = transform[8] * x + transform[9] * y + transform[10] * z + transform[11];

    (tx, ty, tz)
}

/// Analyze transformation matrix components
fn analyze_transformation(transform: &[f64; 12]) {
    println!();
    println!("  Transformation Analysis:");

    // Extract translation
    let translation = (transform[3], transform[7], transform[11]);
    println!("    Translation: ({:.2}, {:.2}, {:.2})", 
        translation.0, translation.1, translation.2);

    // Check for rotation (non-diagonal elements in 3x3 part)
    let has_rotation = transform[1].abs() > 0.001
        || transform[2].abs() > 0.001
        || transform[4].abs() > 0.001
        || transform[6].abs() > 0.001
        || transform[8].abs() > 0.001
        || transform[9].abs() > 0.001;

    if has_rotation {
        println!("    Rotation: Present (complex rotation matrix)");
    } else {
        println!("    Rotation: None (axis-aligned)");
    }

    // Check for scaling
    let scale_x = (transform[0] * transform[0] + transform[4] * transform[4] + transform[8] * transform[8]).sqrt();
    let scale_y = (transform[1] * transform[1] + transform[5] * transform[5] + transform[9] * transform[9]).sqrt();
    let scale_z = (transform[2] * transform[2] + transform[6] * transform[6] + transform[10] * transform[10]).sqrt();

    let is_uniform_scale = (scale_x - scale_y).abs() < 0.001 
        && (scale_y - scale_z).abs() < 0.001;

    if is_uniform_scale && (scale_x - 1.0).abs() < 0.001 {
        println!("    Scale: None (1:1:1)");
    } else if is_uniform_scale {
        println!("    Scale: Uniform ({:.3})", scale_x);
    } else {
        println!("    Scale: Non-uniform ({:.3}, {:.3}, {:.3})", 
            scale_x, scale_y, scale_z);
    }

    // Check for identity matrix
    let is_identity = (transform[0] - 1.0).abs() < 0.001
        && transform[1].abs() < 0.001
        && transform[2].abs() < 0.001
        && transform[3].abs() < 0.001
        && transform[4].abs() < 0.001
        && (transform[5] - 1.0).abs() < 0.001
        && transform[6].abs() < 0.001
        && transform[7].abs() < 0.001
        && transform[8].abs() < 0.001
        && transform[9].abs() < 0.001
        && (transform[10] - 1.0).abs() < 0.001
        && transform[11].abs() < 0.001;

    if is_identity {
        println!("    Type: Identity (no transformation)");
    }
}

/// Calculate the overall build volume encompassing all build items
fn calculate_build_volume(model: &Model) -> Option<((f64, f64, f64), (f64, f64, f64))> {
    let mut all_points = Vec::new();

    for item in &model.build.items {
        if let Some(obj) = model.resources.objects.iter().find(|o| o.id == item.objectid) {
            if let Some(ref mesh) = obj.mesh {
                for vertex in &mesh.vertices {
                    let point = if let Some(transform) = item.transform {
                        apply_transform(vertex, &transform)
                    } else {
                        (vertex.x, vertex.y, vertex.z)
                    };
                    all_points.push(point);
                }
            }
        }
    }

    if all_points.is_empty() {
        return None;
    }

    let (min_x, max_x, min_y, max_y, min_z, max_z) = calculate_bounds_from_points(&all_points);
    Some(((min_x, min_y, min_z), (max_x, max_y, max_z)))
}

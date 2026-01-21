//! Example: Converting 3MF to STL format
//!
//! This example demonstrates how to:
//! 1. Parse a 3MF file
//! 2. Extract mesh geometry from all objects
//! 3. Export the geometry to STL (ASCII) format
//!
//! STL (STereoLithography) is a widely-used format for 3D printing.
//! It stores triangle mesh data in a simple text or binary format.

use lib3mf::Model;
use std::env;
use std::fs::File;
use std::io::Write;
use std::process;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <input.3mf> [output.stl]", args[0]);
        eprintln!();
        eprintln!("Converts a 3MF file to STL (ASCII) format");
        eprintln!();
        eprintln!("If output file is not specified, prints to stdout");
        process::exit(1);
    }

    let input_file = &args[1];
    let output_file = args.get(2);

    println!("Reading 3MF file: {}", input_file);

    // Parse the 3MF file
    let file = File::open(input_file)?;
    let model = Model::from_reader(file)?;

    println!("Model loaded successfully!");
    println!("  Unit: {}", model.unit);
    println!("  Objects: {}", model.resources.objects.len());
    println!();

    // Generate STL content
    let mut stl_content = String::new();
    stl_content.push_str("solid 3mf_model\n");

    let mut total_triangles = 0;

    // Iterate through all build items to get transforms
    for build_item in &model.build.items {
        // Find the object
        let obj = model
            .resources
            .objects
            .iter()
            .find(|o| o.id == build_item.objectid);

        if let Some(obj) = obj {
            if let Some(ref mesh) = obj.mesh {
                println!(
                    "Processing object {}: {} triangles",
                    obj.id,
                    mesh.triangles.len()
                );

                for triangle in &mesh.triangles {
                    // Get vertex coordinates
                    let v1 = &mesh.vertices[triangle.v1];
                    let v2 = &mesh.vertices[triangle.v2];
                    let v3 = &mesh.vertices[triangle.v3];

                    // Apply transformation if present
                    let (tv1, tv2, tv3) = if let Some(transform) = build_item.transform {
                        (
                            apply_transform(v1, &transform),
                            apply_transform(v2, &transform),
                            apply_transform(v3, &transform),
                        )
                    } else {
                        ((v1.x, v1.y, v1.z), (v2.x, v2.y, v2.z), (v3.x, v3.y, v3.z))
                    };

                    // Calculate normal vector (using cross product)
                    let edge1 = (tv2.0 - tv1.0, tv2.1 - tv1.1, tv2.2 - tv1.2);
                    let edge2 = (tv3.0 - tv1.0, tv3.1 - tv1.1, tv3.2 - tv1.2);
                    let normal = (
                        edge1.1 * edge2.2 - edge1.2 * edge2.1,
                        edge1.2 * edge2.0 - edge1.0 * edge2.2,
                        edge1.0 * edge2.1 - edge1.1 * edge2.0,
                    );

                    // Normalize
                    let length =
                        (normal.0 * normal.0 + normal.1 * normal.1 + normal.2 * normal.2).sqrt();
                    let normal = if length > 0.0 {
                        (normal.0 / length, normal.1 / length, normal.2 / length)
                    } else {
                        (0.0, 0.0, 1.0) // fallback
                    };

                    // Write facet
                    stl_content.push_str(&format!(
                        "  facet normal {:.6e} {:.6e} {:.6e}\n",
                        normal.0, normal.1, normal.2
                    ));
                    stl_content.push_str("    outer loop\n");
                    stl_content.push_str(&format!(
                        "      vertex {:.6e} {:.6e} {:.6e}\n",
                        tv1.0, tv1.1, tv1.2
                    ));
                    stl_content.push_str(&format!(
                        "      vertex {:.6e} {:.6e} {:.6e}\n",
                        tv2.0, tv2.1, tv2.2
                    ));
                    stl_content.push_str(&format!(
                        "      vertex {:.6e} {:.6e} {:.6e}\n",
                        tv3.0, tv3.1, tv3.2
                    ));
                    stl_content.push_str("    endloop\n");
                    stl_content.push_str("  endfacet\n");

                    total_triangles += 1;
                }
            }
        }
    }

    stl_content.push_str("endsolid 3mf_model\n");

    println!("Converted {} triangles to STL format", total_triangles);

    // Write output
    if let Some(output_path) = output_file {
        let mut file = File::create(output_path)?;
        file.write_all(stl_content.as_bytes())?;
        println!("✓ STL file written to: {}", output_path);
    } else {
        println!();
        println!("STL Output:");
        println!("{}", stl_content);
    }

    Ok(())
}

/// Apply a 3MF affine transformation matrix to a vertex
/// Transform is a 4x3 matrix stored as 12 values in row-major order
fn apply_transform(vertex: &lib3mf::Vertex, transform: &[f64; 12]) -> (f64, f64, f64) {
    let x = vertex.x;
    let y = vertex.y;
    let z = vertex.z;

    // Apply transformation: [x' y' z'] = [x y z 1] * M
    // where M is the 4x3 transformation matrix
    let tx = transform[0] * x + transform[1] * y + transform[2] * z + transform[3];
    let ty = transform[4] * x + transform[5] * y + transform[6] * z + transform[7];
    let tz = transform[8] * x + transform[9] * y + transform[10] * z + transform[11];

    (tx, ty, tz)
}

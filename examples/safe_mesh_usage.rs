//! Example: Safely working with meshes that may have no triangles
//!
//! This example demonstrates how to safely check 3MF meshes before using them
//! with external libraries that require triangles (like collision detection libraries).
//!
//! In 3MF, particularly with the Beam Lattice extension, meshes can have vertices
//! but no triangles. This is valid per the 3MF specification, as the vertices may
//! serve as connection points for beams.
//!
//! However, many geometry processing libraries (such as parry3d for collision detection)
//! require meshes to have at least one triangle and will panic if given an empty
//! triangle list.

use lib3mf::{Model, ParserConfig};
use std::env;
use std::fs::File;
use std::process;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <input.3mf>", args[0]);
        eprintln!();
        eprintln!("Checks meshes for safe usage with external libraries");
        eprintln!("Reports meshes that have vertices but no triangles");
        process::exit(1);
    }

    let input_file = &args[1];

    println!("=== Mesh Safety Check ===\n");
    println!("Reading 3MF file: {}\n", input_file);

    // Parse the 3MF file
    let file = File::open(input_file)?;
    let model = Model::from_reader_with_config(file, ParserConfig::with_all_extensions())?;

    println!("Model loaded successfully!");
    println!("  Unit: {}", model.unit);
    println!("  Objects: {}\n", model.resources.objects.len());

    let mut meshes_with_triangles = 0;
    let mut meshes_without_triangles = 0;
    let mut meshes_with_beamsets = 0;

    // Analyze each object's mesh
    for obj in &model.resources.objects {
        if let Some(ref mesh) = obj.mesh {
            println!("Object {} - {}", obj.id, obj.name.as_deref().unwrap_or("(unnamed)"));
            println!("  Vertices: {}", mesh.vertices.len());
            println!("  Triangles: {}", mesh.triangles.len());
            
            if mesh.beamset.is_some() {
                println!("  ✓ Has beam lattice data");
                meshes_with_beamsets += 1;
            }

            // Check if mesh is safe for external libraries
            if mesh.has_triangles() {
                meshes_with_triangles += 1;
                println!("  ✓ Safe for triangle-based libraries (has {} triangles)", mesh.triangles.len());
            } else if mesh.has_vertices() {
                meshes_without_triangles += 1;
                println!("  ⚠  WARNING: Has vertices but NO triangles");
                println!("     This mesh should NOT be passed to libraries like parry3d");
                println!("     that require at least one triangle (they will panic!)");
                
                if mesh.beamset.is_some() {
                    println!("     → This appears to be a beam lattice mesh (vertices are for beams)");
                } else {
                    println!("     → This may be an incomplete or invalid mesh");
                }
            } else {
                println!("  ⚠  Empty mesh (no vertices or triangles)");
            }
            
            println!();
        }
    }

    println!("=== Summary ===");
    println!("Meshes with triangles: {}", meshes_with_triangles);
    println!("Meshes without triangles (vertex-only): {}", meshes_without_triangles);
    println!("Meshes with beam lattice data: {}", meshes_with_beamsets);
    println!();

    if meshes_without_triangles > 0 {
        println!("⚠  {} mesh(es) have no triangles!", meshes_without_triangles);
        println!();
        println!("Before using these meshes with external libraries:");
        println!("  1. Always check mesh.has_triangles() first");
        println!("  2. Or check !mesh.triangles.is_empty()");
        println!("  3. Never assume all 3MF meshes have triangles");
        println!();
        println!("Example safe code:");
        println!("  if mesh.has_triangles() {{");
        println!("      // Safe to create TriMesh for collision detection");
        println!("      let trimesh = create_collision_mesh(&mesh);");
        println!("  }} else {{");
        println!("      eprintln!(\"Skipping mesh: no triangles\");");
        println!("  }}");
    } else {
        println!("✓ All meshes have triangles - safe for external libraries");
    }

    Ok(())
}

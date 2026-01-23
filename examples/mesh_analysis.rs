//! Example: Mesh Operations and Build Volume Analysis
//!
//! This example demonstrates the triangle mesh operations using parry3d:
//! - Computing mesh volume for validation
//! - Calculating bounding boxes
//! - Applying affine transformations
//! - Analyzing overall build volume
//!
//! These capabilities help with:
//! - Detecting inverted meshes (negative volume)
//! - Validating build item placements (N_XXX_0421)
//! - Computing spatial extents for manufacturing constraints

use lib3mf::{mesh_ops, Model};
use std::env;
use std::fs::File;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <3mf_file>", args[0]);
        eprintln!();
        eprintln!("Example: {} test_files/core/box.3mf", args[0]);
        std::process::exit(1);
    }

    let filename = &args[1];
    println!("Analyzing 3MF file: {}", filename);
    println!();

    // Load the 3MF model
    let file = File::open(filename)?;
    let model = Model::from_reader(file)?;

    println!("Model Information:");
    println!("  Unit: {}", model.unit);
    println!("  Objects: {}", model.resources.objects.len());
    println!("  Build Items: {}", model.build.items.len());
    println!();

    // Analyze each object
    for object in &model.resources.objects {
        println!("Object ID: {}", object.id);

        if let Some(ref mesh) = object.mesh {
            println!("  Vertices: {}", mesh.vertices.len());
            println!("  Triangles: {}", mesh.triangles.len());

            // Compute signed volume
            match mesh_ops::compute_mesh_signed_volume(mesh) {
                Ok(signed_volume) => {
                    println!("  Signed Volume: {:.6} cubic units", signed_volume);
                    if signed_volume < 0.0 {
                        println!(
                            "    ⚠️  WARNING: Negative volume detected - mesh may be inverted!"
                        );
                    } else {
                        println!("    ✓ Positive volume - mesh orientation is correct");
                    }
                }
                Err(e) => {
                    println!("    Error computing volume: {}", e);
                }
            }

            // Compute unsigned volume using parry3d
            match mesh_ops::compute_mesh_volume(mesh) {
                Ok(volume) => {
                    println!("  Absolute Volume: {:.6} cubic units", volume);
                }
                Err(e) => {
                    println!("    Error computing absolute volume: {}", e);
                }
            }

            // Compute bounding box
            match mesh_ops::compute_mesh_aabb(mesh) {
                Ok((min, max)) => {
                    println!("  Bounding Box:");
                    println!("    Min: ({:.2}, {:.2}, {:.2})", min.0, min.1, min.2);
                    println!("    Max: ({:.2}, {:.2}, {:.2})", max.0, max.1, max.2);
                    println!(
                        "    Dimensions: {:.2} x {:.2} x {:.2}",
                        max.0 - min.0,
                        max.1 - min.1,
                        max.2 - min.2
                    );
                }
                Err(e) => {
                    println!("    Error computing bounding box: {}", e);
                }
            }
        } else {
            println!("  No mesh data");
        }

        println!();
    }

    // Analyze build items and their transformations
    if !model.build.items.is_empty() {
        println!("Build Items Analysis:");
        println!();

        for (idx, item) in model.build.items.iter().enumerate() {
            println!("  Build Item {}: Object {}", idx + 1, item.objectid);

            // Find the referenced object
            let object = model
                .resources
                .objects
                .iter()
                .find(|obj| obj.id == item.objectid);

            if let Some(object) = object {
                if let Some(ref mesh) = object.mesh {
                    // Show transform if present
                    if let Some(ref transform) = item.transform {
                        println!("    Transform:");
                        println!(
                            "      [ {:.2} {:.2} {:.2} {:.2} ]",
                            transform[0], transform[1], transform[2], transform[3]
                        );
                        println!(
                            "      [ {:.2} {:.2} {:.2} {:.2} ]",
                            transform[4], transform[5], transform[6], transform[7]
                        );
                        println!(
                            "      [ {:.2} {:.2} {:.2} {:.2} ]",
                            transform[8], transform[9], transform[10], transform[11]
                        );

                        // Extract translation component
                        let tx = transform[3];
                        let ty = transform[7];
                        let tz = transform[11];
                        println!("    Translation: ({:.2}, {:.2}, {:.2})", tx, ty, tz);

                        // Compute transformed bounding box
                        match mesh_ops::compute_transformed_aabb(mesh, Some(transform)) {
                            Ok((min, max)) => {
                                println!("    Transformed Bounding Box:");
                                println!("      Min: ({:.2}, {:.2}, {:.2})", min.0, min.1, min.2);
                                println!("      Max: ({:.2}, {:.2}, {:.2})", max.0, max.1, max.2);

                                // Check for potential issues (N_XXX_0421)
                                if max.0 < 0.0 && max.1 < 0.0 && max.2 < 0.0 {
                                    println!(
                                        "      ⚠️  WARNING: Entire mesh is in negative coordinate space!"
                                    );
                                    println!(
                                        "          This may indicate an incorrect transformation."
                                    );
                                }
                            }
                            Err(e) => {
                                println!("      Error computing transformed AABB: {}", e);
                            }
                        }
                    } else {
                        println!("    No transform (using identity)");

                        // Show original bounding box
                        if let Ok((min, max)) = mesh_ops::compute_mesh_aabb(mesh) {
                            println!("    Bounding Box:");
                            println!("      Min: ({:.2}, {:.2}, {:.2})", min.0, min.1, min.2);
                            println!("      Max: ({:.2}, {:.2}, {:.2})", max.0, max.1, max.2);
                        }
                    }
                }
            } else {
                println!("    ⚠️  Referenced object not found!");
            }

            println!();
        }

        // Compute overall build volume
        println!("Overall Build Volume:");
        match mesh_ops::compute_build_volume(&model) {
            Some((min, max)) => {
                println!("  Min: ({:.2}, {:.2}, {:.2})", min.0, min.1, min.2);
                println!("  Max: ({:.2}, {:.2}, {:.2})", max.0, max.1, max.2);
                println!(
                    "  Dimensions: {:.2} x {:.2} x {:.2} {}",
                    max.0 - min.0,
                    max.1 - min.1,
                    max.2 - min.2,
                    model.unit
                );

                let volume = (max.0 - min.0) * (max.1 - min.1) * (max.2 - min.2);
                println!("  Bounding Volume: {:.2} cubic {}", volume, model.unit);
            }
            None => {
                println!("  Could not compute build volume (no meshes or build items)");
            }
        }
    }

    Ok(())
}

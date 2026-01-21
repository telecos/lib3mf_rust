//! Demonstrate component parsing and validation
//!
//! This example shows how components are parsed from 3MF files and how
//! component references are validated.

use lib3mf::Model;
use std::fs::File;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Component Parsing Demo ===\n");

    // Parse the test file with components
    let file = File::open("test_files/components/assembly.3mf")?;
    let model = Model::from_reader(file)?;

    println!("Successfully parsed 3MF file with components!");
    println!("\nModel Details:");
    println!("  Unit: {}", model.unit);
    println!("  Objects: {}", model.resources.objects.len());
    println!("  Build Items: {}", model.build.items.len());

    // Display object information
    println!("\n=== Objects ===");
    for obj in &model.resources.objects {
        println!("\nObject ID: {}", obj.id);
        println!("  Type: {:?}", obj.object_type);

        if let Some(ref mesh) = obj.mesh {
            println!(
                "  Mesh: {} vertices, {} triangles",
                mesh.vertices.len(),
                mesh.triangles.len()
            );
        }

        if !obj.components.is_empty() {
            println!("  Components: {}", obj.components.len());
            for (i, comp) in obj.components.iter().enumerate() {
                println!(
                    "    Component {}: references object {}",
                    i + 1,
                    comp.objectid
                );

                if let Some(transform) = comp.transform {
                    // Extract translation from transform matrix
                    let tx = transform[9];
                    let ty = transform[10];
                    let tz = transform[11];
                    println!("      Transform: translation ({}, {}, {})", tx, ty, tz);
                }
            }
        }
    }

    // Display build items
    println!("\n=== Build Items ===");
    for (i, item) in model.build.items.iter().enumerate() {
        println!("Build Item {}: object {}", i + 1, item.objectid);
    }

    println!("\n=== Validation ===");
    println!("All component references validated successfully!");
    println!("  ✓ All component objectid values reference existing objects");
    println!("  ✓ No circular component references detected");

    Ok(())
}

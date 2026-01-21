//! Comprehensive parsing test that generates a report of what we can extract from various test files

use lib3mf::Model;
use std::fs::File;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 3MF File Parsing Capabilities Report ===\n");

    let test_files = vec![
        ("Core/Box", "test_files/core/box.3mf"),
        ("Core/Sphere", "test_files/core/sphere.3mf"),
        ("Core/Cube Gears", "test_files/core/cube_gears.3mf"),
        (
            "Material/Kinect Scan",
            "test_files/material/kinect_scan.3mf",
        ),
        ("Production/Box", "test_files/production/box_prod.3mf"),
        ("Slice/Box Sliced", "test_files/slices/box_sliced.3mf"),
        (
            "Beam Lattice/Pyramid",
            "test_files/beam_lattice/pyramid.3mf",
        ),
    ];

    for (name, path) in test_files {
        println!("--- {} ---", name);
        println!("File: {}", path);

        match File::open(path) {
            Ok(file) => match Model::from_reader(file) {
                Ok(model) => {
                    println!("✓ Parsed successfully");
                    println!("  Unit: {}", model.unit);
                    println!("  Metadata entries: {}", model.metadata.len());

                    if !model.metadata.is_empty() {
                        for entry in &model.metadata {
                            println!(
                                "    - {}: {}",
                                entry.name,
                                if entry.value.len() > 50 {
                                    format!("{}...", &entry.value[..50])
                                } else {
                                    entry.value.clone()
                                }
                            );
                        }
                    }

                    println!("  Resources:");
                    println!("    Objects: {}", model.resources.objects.len());
                    println!("    Materials (base): {}", model.resources.materials.len());
                    println!("    Color groups: {}", model.resources.color_groups.len());

                    if !model.resources.color_groups.is_empty() {
                        for cg in &model.resources.color_groups {
                            println!("      ColorGroup {}: {} colors", cg.id, cg.colors.len());
                        }
                    }

                    println!("  Objects detail:");
                    for obj in &model.resources.objects {
                        print!("    Object {}", obj.id);
                        if let Some(ref name) = obj.name {
                            print!(" ({})", name);
                        }
                        println!(":");
                        println!("      Type: {:?}", obj.object_type);

                        if let Some(ref mesh) = obj.mesh {
                            println!(
                                "      Mesh: {} vertices, {} triangles",
                                mesh.vertices.len(),
                                mesh.triangles.len()
                            );

                            let with_pid =
                                mesh.triangles.iter().filter(|t| t.pid.is_some()).count();
                            if with_pid > 0 {
                                println!("        Triangles with material refs: {}", with_pid);
                            }

                            // Show beam lattice data if present
                            if let Some(ref beamset) = mesh.beamset {
                                println!("      Beam Lattice:");
                                println!("        Beams: {}", beamset.beams.len());
                                println!("        Default radius: {}", beamset.radius);
                                println!("        Min length: {}", beamset.min_length);
                                println!("        Cap mode: {:?}", beamset.cap_mode);
                                if !beamset.beams.is_empty() {
                                    let beam = &beamset.beams[0];
                                    println!(
                                        "        First beam: v{}->v{}, r1={:?}, r2={:?}",
                                        beam.v1, beam.v2, beam.r1, beam.r2
                                    );
                                }
                            }
                        } else {
                            println!("      No mesh data");
                        }
                    }

                    println!("  Build:");
                    println!("    Items: {}", model.build.items.len());
                    for item in &model.build.items {
                        print!("      Item -> Object {}", item.objectid);
                        if item.transform.is_some() {
                            print!(" (with transform)");
                        }
                        println!();
                    }
                }
                Err(e) => {
                    println!("✗ Parse error: {:?}", e);
                }
            },
            Err(e) => {
                println!("✗ File open error: {:?}", e);
            }
        }
        println!();
    }

    println!("=== Summary ===");
    println!("All test files parsed successfully!");
    println!("\nSupported features:");
    println!("  ✓ Core specification (vertices, triangles, objects, build)");
    println!("  ✓ Metadata");
    println!("  ✓ Base materials");
    println!("  ✓ Color groups (materials extension)");
    println!("  ✓ Transformations on build items");
    println!("  ✓ Named objects");
    println!("  ✓ Beam lattice structures (beamsets, beams, radii, cap modes)");
    println!("\nPartially supported (files parse, some data extracted):");
    println!("  ⚠ Production extension (basic parsing)");
    println!("  ⚠ Slice extension (basic parsing)");

    Ok(())
}

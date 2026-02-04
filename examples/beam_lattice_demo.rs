//! Demonstration of Beam Lattice Extension support
//!
//! This example shows how to:
//! - Parse 3MF files with beam lattice structures
//! - Access beam lattice data (BeamSet, Beam, BeamCapMode)
//! - Inspect beam properties (vertex connections, radii, cap modes)

use lib3mf::Model;
use std::fs::File;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Beam Lattice Extension Demo ===\n");

    // Open a 3MF file with beam lattice data
    let file = File::open("test_files/beam_lattice/pyramid.3mf")?;
    let model = Model::from_reader(file)?;

    println!("Model: {}", model.unit);
    println!("Objects: {}\n", model.resources.objects.len());

    // Iterate through objects looking for beam lattice data
    for obj in &model.resources.objects {
        println!(
            "Object {} - {}",
            obj.id,
            obj.name.as_deref().unwrap_or("(unnamed)")
        );

        if let Some(ref mesh) = obj.mesh {
            println!("  Mesh:");
            println!("    Vertices: {}", mesh.vertices.len());
            println!("    Triangles: {}", mesh.triangles.len());

            // Check for beam lattice data
            if let Some(ref beamset) = mesh.beamset {
                println!("\n  Beam Lattice Structure:");
                println!("    Total beams: {}", beamset.beams.len());
                println!("    Default radius: {} mm", beamset.radius);
                println!("    Minimum length: {} mm", beamset.min_length);
                println!("    Cap mode: {:?}", beamset.cap_mode);

                // Analyze beam properties
                let beams_with_custom_r1 = beamset.beams.iter().filter(|b| b.r1.is_some()).count();
                let beams_with_custom_r2 = beamset.beams.iter().filter(|b| b.r2.is_some()).count();

                println!("\n  Beam Analysis:");
                println!("    Beams with custom r1: {}", beams_with_custom_r1);
                println!("    Beams with custom r2: {}", beams_with_custom_r2);
                println!(
                    "    Beams using default radius: {}",
                    beamset.beams.len() - beams_with_custom_r1
                );

                // Show some example beams
                println!("\n  Example Beams:");
                for (i, beam) in beamset.beams.iter().take(5).enumerate() {
                    print!("    Beam {}: v{} -> v{}", i + 1, beam.v1, beam.v2);

                    // Show radius information
                    match (beam.r1, beam.r2) {
                        (Some(r1), Some(r2)) => {
                            println!(", r1={:.5} mm, r2={:.5} mm (varying radius)", r1, r2);
                        }
                        (Some(r1), None) => {
                            println!(", r1={:.5} mm (uniform radius)", r1);
                        }
                        (None, _) => {
                            println!(", using default radius ({} mm)", beamset.radius);
                        }
                    }
                }

                // Vertex connectivity analysis
                let mut vertex_connections: std::collections::HashMap<usize, usize> =
                    std::collections::HashMap::new();
                for beam in &beamset.beams {
                    *vertex_connections.entry(beam.v1).or_insert(0) += 1;
                    *vertex_connections.entry(beam.v2).or_insert(0) += 1;
                }

                let max_connections = vertex_connections.values().max().unwrap_or(&0);
                let avg_connections = if !vertex_connections.is_empty() {
                    vertex_connections.values().sum::<usize>() as f64
                        / vertex_connections.len() as f64
                } else {
                    0.0
                };

                println!("\n  Vertex Connectivity:");
                println!(
                    "    Unique vertices with beams: {}",
                    vertex_connections.len()
                );
                println!("    Average connections per vertex: {:.2}", avg_connections);
                println!("    Maximum connections: {}", max_connections);

                // Find highly connected vertices (hubs)
                let hubs: Vec<_> = vertex_connections
                    .iter()
                    .filter(|&(_, &count)| count > 5)
                    .collect();

                if !hubs.is_empty() {
                    println!("    Hub vertices (>5 connections): {}", hubs.len());
                    for (vertex, count) in hubs.iter().take(3) {
                        println!("      Vertex {}: {} connections", vertex, count);
                    }
                }
            } else {
                println!("\n  No beam lattice data");
            }
        }
        println!();
    }

    // Show extension information
    println!("Required Extensions:");
    for ext in &model.required_extensions {
        println!("  - {:?}", ext);
    }

    println!("\n=== Beam Lattice Extension Features ===");
    println!("✓ BeamSet structure with default properties");
    println!("✓ Individual Beam definitions");
    println!("✓ Vertex connections (v1, v2)");
    println!("✓ Per-beam radii (r1, r2)");
    println!("✓ Cap modes (Sphere, Butt)");
    println!("✓ Minimum length constraints");

    Ok(())
}

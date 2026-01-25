use lib3mf::parser::parse_3mf_with_config;
use lib3mf::{ParserConfig, Extension};
use std::fs::File;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = "test_suites/suite11_Displacement/Negative Tests/N_DPX_3314_05.3mf";
    
    let config = ParserConfig::new()
        .with_extension(Extension::Displacement)
        .with_extension(Extension::BooleanOperations)
        .with_extension(Extension::Production)
        .with_custom_extension(
            "http://schemas.3mf.io/3dmanufacturing/displacement/2023/10",
            "Displacement 2023/10",
        );
    
    let file = File::open(path)?;
    
    match parse_3mf_with_config(file, config) {
        Ok(model) => {
            println!("✓ File parsed successfully (UNEXPECTED - should fail!)");
            println!("\nObjects in model:");
            for obj in &model.resources.objects {
                if let Some(disp_mesh) = &obj.displacement_mesh {
                    println!("\nObject {} (displacement mesh):", obj.id);
                    println!("  Vertices: {}", disp_mesh.vertices.len());
                    for (i, v) in disp_mesh.vertices.iter().enumerate() {
                        println!("    v{}: ({}, {}, {})", i, v.x, v.y, v.z);
                    }
                    
                    println!("  Triangles: {}", disp_mesh.triangles.len());
                    for (i, t) in disp_mesh.triangles.iter().enumerate() {
                        println!("    t{}: v1={}, v2={}, v3={}", i, t.v1, t.v2, t.v3);
                        
                        // Calculate triangle normal and volume contribution
                        let v1 = &disp_mesh.vertices[t.v1];
                        let v2 = &disp_mesh.vertices[t.v2];
                        let v3 = &disp_mesh.vertices[t.v3];
                        
                        // Edge vectors
                        let edge1 = (v2.x - v1.x, v2.y - v1.y, v2.z - v1.z);
                        let edge2 = (v3.x - v1.x, v3.y - v1.y, v3.z - v1.z);
                        
                        // Cross product (normal)
                        let normal = (
                            edge1.1 * edge2.2 - edge1.2 * edge2.1,
                            edge1.2 * edge2.0 - edge1.0 * edge2.2,
                            edge1.0 * edge2.1 - edge1.1 * edge2.0
                        );
                        
                        // Volume contribution
                        let vol_contrib = v1.x * (v2.y * v3.z - v2.z * v3.y)
                            + v2.x * (v3.y * v1.z - v3.z * v1.y)
                            + v3.x * (v1.y * v2.z - v1.z * v2.y);
                        
                        println!("      normal: ({:.3}, {:.3}, {:.3})", normal.0, normal.1, normal.2);
                        println!("      volume_contribution: {:.6}", vol_contrib / 6.0);
                    }
                    
                    // Calculate total volume
                    let mut volume = 0.0;
                    for triangle in &disp_mesh.triangles {
                        let v1 = &disp_mesh.vertices[triangle.v1];
                        let v2 = &disp_mesh.vertices[triangle.v2];
                        let v3 = &disp_mesh.vertices[triangle.v3];
                        volume += v1.x * (v2.y * v3.z - v2.z * v3.y)
                            + v2.x * (v3.y * v1.z - v3.z * v1.y)
                            + v3.x * (v1.y * v2.z - v1.z * v2.y);
                    }
                    volume /= 6.0;
                    println!("\n  Total volume: {:.10}", volume);
                    if volume < 0.0 {
                        println!("  ⚠️  NEGATIVE VOLUME - mesh is inverted!");
                    } else if volume < 1e-10 {
                        println!("  ⚠️  NEAR-ZERO VOLUME - mesh is degenerate!");
                    } else {
                        println!("  ✓ Positive volume");
                    }
                }
            }
        }
        Err(e) => {
            println!("✓ File failed to parse (expected): {}", e);
        }
    }
    
    Ok(())
}

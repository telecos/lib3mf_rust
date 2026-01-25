use lib3mf::parser::parse_3mf_with_config;
use lib3mf::{ParserConfig, Extension};
use std::fs::File;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = "test_suites/suite11_Displacement/Positive Tests/P_DPX_3218_01.3mf";
    
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
            println!("✓ File parsed successfully");
            println!("\nObjects in model:");
            for obj in &model.resources.objects {
                if let Some(disp_mesh) = &obj.displacement_mesh {
                    println!("\nObject {} (displacement mesh):", obj.id);
                    println!("  Vertices: {}", disp_mesh.vertices.len());
                    
                    println!("  Triangles: {}", disp_mesh.triangles.len());
                    
                    // Calculate total volume and contributions
                    let mut volume = 0.0;
                    let mut positive_contribs = 0;
                    let mut negative_contribs = 0;
                    let mut zero_contribs = 0;
                    
                    for triangle in &disp_mesh.triangles {
                        let v1 = &disp_mesh.vertices[triangle.v1];
                        let v2 = &disp_mesh.vertices[triangle.v2];
                        let v3 = &disp_mesh.vertices[triangle.v3];
                        
                        let contrib = v1.x * (v2.y * v3.z - v2.z * v3.y)
                            + v2.x * (v3.y * v1.z - v3.z * v1.y)
                            + v3.x * (v1.y * v2.z - v1.z * v2.y);
                        
                        volume += contrib;
                        
                        const THRESHOLD: f64 = 1e-6;
                        if contrib > THRESHOLD {
                            positive_contribs += 1;
                        } else if contrib < -THRESHOLD {
                            negative_contribs += 1;
                        } else {
                            zero_contribs += 1;
                        }
                    }
                    volume /= 6.0;
                    
                    println!("  Total volume: {:.10}", volume);
                    println!("  Positive contributions: {}", positive_contribs);
                    println!("  Negative contributions: {}", negative_contribs);
                    println!("  Near-zero contributions: {}", zero_contribs);
                }
            }
        }
        Err(e) => {
            println!("✗ File failed to parse: {}", e);
        }
    }
    
    Ok(())
}

//! Test material parsing capabilities

use lib3mf::Model;
use std::fs::File;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open("test_files/material/kinect_scan.3mf")?;
    let model = Model::from_reader(file)?;
    
    println!("Materials found: {}", model.resources.materials.len());
    println!("Color groups found: {}", model.resources.color_groups.len());
    println!("Objects found: {}", model.resources.objects.len());
    
    if !model.resources.materials.is_empty() {
        println!("\nFirst few materials:");
        for (i, mat) in model.resources.materials.iter().take(5).enumerate() {
            println!("  Material {}: ID={}, name={:?}, color={:?}", 
                i, mat.id, mat.name, mat.color);
        }
    }
    
    if !model.resources.color_groups.is_empty() {
        println!("\nColor groups:");
        for colorgroup in &model.resources.color_groups {
            println!("  ColorGroup {}: {} colors", 
                colorgroup.id, colorgroup.colors.len());
            println!("    First few colors:");
            for (i, color) in colorgroup.colors.iter().take(5).enumerate() {
                println!("      Color {}: #{:02X}{:02X}{:02X}{:02X}", 
                    i, color.0, color.1, color.2, color.3);
            }
        }
    }
    
    println!("\nObject info:");
    for obj in &model.resources.objects {
        let mesh = obj.mesh.as_ref().unwrap();
        println!("  Object {}: {} vertices, {} triangles, pid={:?}", 
            obj.id, mesh.vertices.len(), mesh.triangles.len(), obj.pid);
        
        // Check if triangles have material references
        let triangles_with_pid = mesh.triangles.iter().filter(|t| t.pid.is_some()).count();
        println!("    Triangles with material reference: {}", triangles_with_pid);
        
        // Show first few triangle PIDs
        if triangles_with_pid > 0 {
            println!("    First few triangle PIDs:");
            for (i, tri) in mesh.triangles.iter().take(5).enumerate() {
                if let Some(pid) = tri.pid {
                    println!("      Triangle {}: pid={}", i, pid);
                }
            }
        }
    }
    
    Ok(())
}

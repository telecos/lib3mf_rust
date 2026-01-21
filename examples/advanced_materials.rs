//! Demonstrate advanced materials extension features
//! 
//! This example shows how to access advanced materials extension features
//! including Texture2D, composite materials, and multi-properties.

use lib3mf::Model;
use std::fs::File;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Try to parse a 3MF file with advanced materials
    // For now, this demonstrates the API structure
    println!("Advanced Materials Extension Example\n");
    println!("This example demonstrates the new data structures for:");
    println!("  - Texture2D resources and texture coordinate groups");
    println!("  - Composite materials mixing base materials");
    println!("  - Multi-properties layering multiple property groups");
    println!();

    // Example: Parse a file and check for advanced materials
    let file_path = "test_files/material/kinect_scan.3mf";
    match File::open(file_path) {
        Ok(file) => {
            let model = Model::from_reader(file)?;
            display_material_info(&model);
        }
        Err(_) => {
            println!("Note: Test file not found, showing API structure instead.\n");
            show_api_structure();
        }
    }

    Ok(())
}

fn display_material_info(model: &Model) {
    println!("=== Material Resources Summary ===\n");
    
    // Basic materials (already supported)
    println!("Basic Materials: {}", model.resources.materials.len());
    println!("Color Groups: {}", model.resources.color_groups.len());
    println!("Base Material Groups: {}", model.resources.base_material_groups.len());
    
    // Advanced materials (newly added)
    println!("\n=== Advanced Materials (New) ===\n");
    println!("Texture2D Resources: {}", model.resources.texture2d_resources.len());
    println!("Texture2D Groups: {}", model.resources.texture2d_groups.len());
    println!("Composite Materials: {}", model.resources.composite_materials.len());
    println!("Multi-Properties Groups: {}", model.resources.multi_properties.len());
    
    // Display Texture2D details
    if !model.resources.texture2d_resources.is_empty() {
        println!("\n--- Texture2D Resources ---");
        for tex in &model.resources.texture2d_resources {
            println!("  Texture ID {}: path={}, contenttype={}", 
                tex.id, tex.path, tex.contenttype);
            println!("    Tile styles: u={:?}, v={:?}", tex.tilestyleu, tex.tilestylev);
            println!("    Filter: {:?}", tex.filter);
        }
    }
    
    // Display Texture2DGroup details
    if !model.resources.texture2d_groups.is_empty() {
        println!("\n--- Texture2D Groups ---");
        for group in &model.resources.texture2d_groups {
            println!("  Group ID {}: references texture={}, {} coordinates", 
                group.id, group.texid, group.tex2coords.len());
            if !group.tex2coords.is_empty() {
                println!("    First few coordinates:");
                for (i, coord) in group.tex2coords.iter().take(3).enumerate() {
                    println!("      [{}] u={:.3}, v={:.3}", i, coord.u, coord.v);
                }
            }
        }
    }
    
    // Display Composite Materials
    if !model.resources.composite_materials.is_empty() {
        println!("\n--- Composite Materials ---");
        for comp in &model.resources.composite_materials {
            println!("  Group ID {}: base material group={}, {} material indices", 
                comp.id, comp.matid, comp.matindices.len());
            println!("    Material indices: {:?}", comp.matindices);
            println!("    {} composite definitions", comp.composites.len());
            for (i, composite) in comp.composites.iter().take(3).enumerate() {
                println!("      Composite [{}]: values={:?}", i, composite.values);
            }
        }
    }
    
    // Display Multi-Properties
    if !model.resources.multi_properties.is_empty() {
        println!("\n--- Multi-Properties Groups ---");
        for multi in &model.resources.multi_properties {
            println!("  Group ID {}: {} property groups layered", 
                multi.id, multi.pids.len());
            println!("    Property IDs: {:?}", multi.pids);
            println!("    Blend methods: {:?}", multi.blendmethods);
            println!("    {} multi definitions", multi.multis.len());
            for (i, m) in multi.multis.iter().take(3).enumerate() {
                println!("      Multi [{}]: property indices={:?}", i, m.pindices);
            }
        }
    }
}

fn show_api_structure() {
    println!("=== API Structure ===\n");
    
    println!("After parsing a 3MF file, you can access advanced materials via:");
    println!();
    println!("  let model = Model::from_reader(file)?;");
    println!();
    println!("  // Texture2D resources");
    println!("  for texture in &model.resources.texture2d_resources {{");
    println!("      println!(\"Texture: {{}} - {{}}\", texture.id, texture.path);");
    println!("  }}");
    println!();
    println!("  // Texture2D groups with coordinates");
    println!("  for group in &model.resources.texture2d_groups {{");
    println!("      println!(\"Texture group {{}} has {{}} coordinates\",");
    println!("               group.id, group.tex2coords.len());");
    println!("  }}");
    println!();
    println!("  // Composite materials");
    println!("  for composite in &model.resources.composite_materials {{");
    println!("      println!(\"Composite {{}} mixes materials: {{:?}}\",");
    println!("               composite.id, composite.matindices);");
    println!("  }}");
    println!();
    println!("  // Multi-properties");
    println!("  for multi in &model.resources.multi_properties {{");
    println!("      println!(\"Multi-properties {{}} layers: {{:?}}\",");
    println!("               multi.id, multi.pids);");
    println!("  }}");
}

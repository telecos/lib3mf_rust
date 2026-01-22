use lib3mf::{Model, ParserConfig, Extension};
use std::fs::File;
use std::path::PathBuf;
use walkdir::WalkDir;

fn main() {
    let neg_dir = "test_suites/suite11_Displacement/Negative Tests";
    
    let files: Vec<PathBuf> = WalkDir::new(neg_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("3mf"))
        .map(|e| e.path().to_path_buf())
        .collect();
    
    let config = ParserConfig::new()
        .with_extension(Extension::Displacement)
        .with_extension(Extension::BooleanOperations)
        .with_extension(Extension::Production)
        .with_custom_extension(
            "http://schemas.3mf.io/3dmanufacturing/displacement/2023/10",
            "Displacement 2023/10",
        );
    
    println!("=== Analyzing {} negative test files ===\n", files.len());
    
    for file_path in &files {
        let filename = file_path.file_name().unwrap().to_str().unwrap();
        
        match File::open(file_path) {
            Ok(file) => {
                match Model::from_reader_with_config(file, config.clone()) {
                    Ok(model) => {
                        println!("❌ {} - Should fail but succeeded", filename);
                        
                        // Analyze the model to understand what should be validated
                        println!("   Displacement maps: {}", model.resources.displacement_maps.len());
                        println!("   Norm vector groups: {}", model.resources.normvector_groups.len());
                        println!("   Disp2D groups: {}", model.resources.disp2d_groups.len());
                        
                        // Check for invalid references
                        for disp_group in &model.resources.disp2d_groups {
                            let disp_exists = model.resources.displacement_maps.iter()
                                .any(|d| d.id == disp_group.dispid);
                            let norm_exists = model.resources.normvector_groups.iter()
                                .any(|n| n.id == disp_group.nid);
                            
                            if !disp_exists {
                                println!("   ⚠ Disp2DGroup {} references invalid dispid {}", 
                                    disp_group.id, disp_group.dispid);
                            }
                            if !norm_exists {
                                println!("   ⚠ Disp2DGroup {} references invalid nid {}", 
                                    disp_group.id, disp_group.nid);
                            }
                            
                            // Check normvector references
                            if let Some(norm_group) = model.resources.normvector_groups.iter()
                                .find(|n| n.id == disp_group.nid) {
                                for coord in &disp_group.coords {
                                    if coord.n >= norm_group.normvectors.len() {
                                        println!("   ⚠ Disp2DCoord references invalid normvector index {}", coord.n);
                                    }
                                }
                                
                                // Check if normvectors are normalized
                                for (i, nv) in norm_group.normvectors.iter().enumerate() {
                                    let len = (nv.x * nv.x + nv.y * nv.y + nv.z * nv.z).sqrt();
                                    if (len - 1.0).abs() > 0.0001 {
                                        println!("   ⚠ Normvector {} is not normalized: length = {:.6}", i, len);
                                    }
                                }
                            }
                        }
                        
                        // Check for invalid displacement texture paths
                        for disp in &model.resources.displacement_maps {
                            println!("   Displacement {} path: {}", disp.id, disp.path);
                        }
                        
                        // Check objects with displacement meshes
                        for obj in &model.resources.objects {
                            if let Some(ref disp_mesh) = obj.displacement_mesh {
                                println!("   Object {} has displacement mesh", obj.id);
                                println!("     Vertices: {}", disp_mesh.vertices.len());
                                println!("     Triangles: {}", disp_mesh.triangles.len());
                                
                                // Check triangle references
                                for (tri_idx, tri) in disp_mesh.triangles.iter().enumerate() {
                                    // Check vertex references
                                    if tri.v1 >= disp_mesh.vertices.len() 
                                        || tri.v2 >= disp_mesh.vertices.len()
                                        || tri.v3 >= disp_mesh.vertices.len() {
                                        println!("     ⚠ Triangle {} has invalid vertex reference", tri_idx);
                                    }
                                    
                                    // Check displacement coord references  
                                    if let Some(did) = tri.did {
                                        let group_exists = model.resources.disp2d_groups.iter()
                                            .any(|g| g.id == did);
                                        if !group_exists {
                                            println!("     ⚠ Triangle {} references invalid disp2d group {}", tri_idx, did);
                                        } else if let Some(group) = model.resources.disp2d_groups.iter().find(|g| g.id == did) {
                                            // Check d1, d2, d3 indices
                                            if let Some(d1) = tri.d1 {
                                                if d1 >= group.coords.len() {
                                                    println!("     ⚠ Triangle {} d1={} exceeds group {} size {}", 
                                                        tri_idx, d1, did, group.coords.len());
                                                }
                                            }
                                            if let Some(d2) = tri.d2 {
                                                if d2 >= group.coords.len() {
                                                    println!("     ⚠ Triangle {} d2={} exceeds group {} size {}", 
                                                        tri_idx, d2, did, group.coords.len());
                                                }
                                            }
                                            if let Some(d3) = tri.d3 {
                                                if d3 >= group.coords.len() {
                                                    println!("     ⚠ Triangle {} d3={} exceeds group {} size {}", 
                                                        tri_idx, d3, did, group.coords.len());
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        
                        println!();
                    }
                    Err(e) => {
                        println!("✓ {} - Correctly failed: {}", filename, e);
                    }
                }
            }
            Err(e) => {
                println!("❌ {} - Cannot open: {}", filename, e);
            }
        }
    }
}

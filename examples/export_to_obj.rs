//! Example: Converting 3MF to OBJ format
//!
//! This example demonstrates how to:
//! 1. Parse a 3MF file
//! 2. Extract mesh geometry and materials
//! 3. Export the geometry to OBJ format with MTL material file
//!
//! OBJ (Wavefront Object) is a widely-supported 3D geometry format that can store
//! vertices, normals, texture coordinates, and material references.

use lib3mf::Model;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::Write;
use std::process;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <input.3mf> [output.obj]", args[0]);
        eprintln!();
        eprintln!("Converts a 3MF file to OBJ format");
        eprintln!("Also creates a .mtl file for materials if colors are present");
        eprintln!();
        eprintln!("If output file is not specified, prints to stdout");
        process::exit(1);
    }

    let input_file = &args[1];
    let output_file = args.get(2);

    println!("Reading 3MF file: {}", input_file);

    // Parse the 3MF file
    let file = File::open(input_file)?;
    let model = Model::from_reader(file)?;

    println!("Model loaded successfully!");
    println!("  Unit: {}", model.unit);
    println!("  Objects: {}", model.resources.objects.len());
    println!("  Materials: {}", model.resources.materials.len());
    println!("  Color groups: {}", model.resources.color_groups.len());
    println!();

    // Generate OBJ content
    let mut obj_content = String::new();
    let mut mtl_content = String::new();

    obj_content.push_str("# 3MF to OBJ conversion\n");
    obj_content.push_str(&format!("# Source: {}\n", input_file));
    obj_content.push_str(&format!("# Unit: {}\n", model.unit));

    // Check if we need a material file
    let has_materials = !model.resources.materials.is_empty()
        || !model.resources.color_groups.is_empty();

    if has_materials {
        if let Some(output_path) = output_file {
            let mtl_file = output_path.replace(".obj", ".mtl");
            obj_content.push_str(&format!("mtllib {}\n", mtl_file));
        } else {
            obj_content.push_str("mtllib materials.mtl\n");
        }
    }

    obj_content.push('\n');

    // Create materials
    let mut material_map: HashMap<usize, String> = HashMap::new();
    
    mtl_content.push_str("# Materials from 3MF file\n\n");

    // Export base materials
    for material in &model.resources.materials {
        let mat_name = format!("material_{}", material.id);
        material_map.insert(material.id, mat_name.clone());

        mtl_content.push_str(&format!("newmtl {}\n", mat_name));
        if let Some((r, g, b, a)) = material.color {
            let rf = r as f32 / 255.0;
            let gf = g as f32 / 255.0;
            let bf = b as f32 / 255.0;
            let af = a as f32 / 255.0;
            mtl_content.push_str(&format!("Kd {:.4} {:.4} {:.4}\n", rf, gf, bf));
            if af < 1.0 {
                mtl_content.push_str(&format!("d {:.4}\n", af));
            }
        } else {
            mtl_content.push_str("Kd 0.8 0.8 0.8\n");
        }
        mtl_content.push('\n');
    }

    // Export color groups
    for color_group in &model.resources.color_groups {
        for (idx, color) in color_group.colors.iter().enumerate() {
            let mat_name = format!("colorgroup_{}_{}", color_group.id, idx);
            let color_id = color_group.id * 10000 + idx; // Create unique ID
            material_map.insert(color_id, mat_name.clone());

            mtl_content.push_str(&format!("newmtl {}\n", mat_name));
            let rf = color.0 as f32 / 255.0;
            let gf = color.1 as f32 / 255.0;
            let bf = color.2 as f32 / 255.0;
            let af = color.3 as f32 / 255.0;
            mtl_content.push_str(&format!("Kd {:.4} {:.4} {:.4}\n", rf, gf, bf));
            if af < 1.0 {
                mtl_content.push_str(&format!("d {:.4}\n", af));
            }
            mtl_content.push('\n');
        }
    }

    let mut vertex_offset = 1; // OBJ indices start at 1

    // Process each build item
    for build_item in &model.build.items {
        // Find the object
        let obj = model
            .resources
            .objects
            .iter()
            .find(|o| o.id == build_item.objectid);

        if let Some(obj) = obj {
            if let Some(ref mesh) = obj.mesh {
                obj_content.push_str(&format!("\n# Object {}\n", obj.id));
                if let Some(ref name) = obj.name {
                    obj_content.push_str(&format!("o {}\n", name));
                } else {
                    obj_content.push_str(&format!("o object_{}\n", obj.id));
                }

                // Write vertices
                for vertex in &mesh.vertices {
                    let (x, y, z) = if let Some(transform) = build_item.transform {
                        apply_transform(vertex, &transform)
                    } else {
                        (vertex.x, vertex.y, vertex.z)
                    };
                    obj_content.push_str(&format!("v {:.6} {:.6} {:.6}\n", x, y, z));
                }

                // Write faces grouped by material
                let mut last_material: Option<usize> = None;

                for triangle in &mesh.triangles {
                    // Determine material
                    let material_id = triangle.pid.or(obj.pid);

                    // Change material if needed
                    if material_id != last_material {
                        if let Some(mat_id) = material_id {
                            if let Some(mat_name) = material_map.get(&mat_id) {
                                obj_content.push_str(&format!("usemtl {}\n", mat_name));
                            }
                        }
                        last_material = material_id;
                    }

                    // Write face (OBJ uses 1-based indexing)
                    let v1 = vertex_offset + triangle.v1;
                    let v2 = vertex_offset + triangle.v2;
                    let v3 = vertex_offset + triangle.v3;
                    obj_content.push_str(&format!("f {} {} {}\n", v1, v2, v3));
                }

                vertex_offset += mesh.vertices.len();
            }
        }
    }

    println!("Conversion complete!");

    // Write output files
    if let Some(output_path) = output_file {
        let mut file = File::create(output_path)?;
        file.write_all(obj_content.as_bytes())?;
        println!("✓ OBJ file written to: {}", output_path);

        if has_materials {
            let mtl_path = output_path.replace(".obj", ".mtl");
            let mut file = File::create(&mtl_path)?;
            file.write_all(mtl_content.as_bytes())?;
            println!("✓ MTL file written to: {}", mtl_path);
        }
    } else {
        println!();
        println!("OBJ Output:");
        println!("{}", obj_content);
        if has_materials {
            println!();
            println!("MTL Output:");
            println!("{}", mtl_content);
        }
    }

    Ok(())
}

/// Apply a 3MF affine transformation matrix to a vertex
fn apply_transform(vertex: &lib3mf::Vertex, transform: &[f64; 12]) -> (f64, f64, f64) {
    let x = vertex.x;
    let y = vertex.y;
    let z = vertex.z;

    let tx = transform[0] * x + transform[1] * y + transform[2] * z + transform[3];
    let ty = transform[4] * x + transform[5] * y + transform[6] * z + transform[7];
    let tz = transform[8] * x + transform[9] * y + transform[10] * z + transform[11];

    (tx, ty, tz)
}

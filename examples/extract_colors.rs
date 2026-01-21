//! Example: Extracting color information for rendering
//!
//! This example demonstrates how to:
//! 1. Extract material and color information from 3MF files
//! 2. Map colors to triangles for rendering
//! 3. Handle different color specifications (base materials, color groups)
//! 4. Export color data in a format suitable for rendering engines
//!
//! This is useful for implementing 3D viewers and renderers that need to
//! display colored 3D models.

use lib3mf::Model;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::process;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <3mf-file>", args[0]);
        eprintln!();
        eprintln!("Extracts color information from a 3MF file for rendering");
        process::exit(1);
    }

    let filename = &args[1];

    println!("=== Color Information Extraction ===");
    println!("File: {}", filename);
    println!();

    // Parse the 3MF file
    let file = File::open(filename)?;
    let model = Model::from_reader(file)?;

    println!("Model Information:");
    println!("  Objects: {}", model.resources.objects.len());
    println!("  Materials: {}", model.resources.materials.len());
    println!("  Color Groups: {}", model.resources.color_groups.len());
    println!();

    // Build a material lookup table
    let mut material_colors: HashMap<usize, (u8, u8, u8, u8)> = HashMap::new();

    // Add base materials
    for material in &model.resources.materials {
        if let Some(color) = material.color {
            material_colors.insert(material.id, color);
            println!("Base Material {}:", material.id);
            if let Some(ref name) = material.name {
                println!("  Name: {}", name);
            }
            println!(
                "  Color: RGB({}, {}, {}) Alpha: {}",
                color.0, color.1, color.2, color.3
            );
            println!(
                "  Hex: #{:02X}{:02X}{:02X}{:02X}",
                color.0, color.1, color.2, color.3
            );
            println!(
                "  Float: ({:.3}, {:.3}, {:.3}, {:.3})",
                color.0 as f32 / 255.0,
                color.1 as f32 / 255.0,
                color.2 as f32 / 255.0,
                color.3 as f32 / 255.0
            );
            println!();
        }
    }

    // Add color groups
    for color_group in &model.resources.color_groups {
        println!(
            "Color Group {}: {} colors",
            color_group.id,
            color_group.colors.len()
        );

        // Show first few colors
        let preview_count = color_group.colors.len().min(5);
        for (idx, color) in color_group.colors.iter().take(preview_count).enumerate() {
            println!(
                "  [{}] RGB({}, {}, {}) Alpha: {} (#{:02X}{:02X}{:02X}{:02X})",
                idx, color.0, color.1, color.2, color.3, color.0, color.1, color.2, color.3
            );
        }

        if color_group.colors.len() > preview_count {
            println!(
                "  ... and {} more colors",
                color_group.colors.len() - preview_count
            );
        }
        println!();
    }

    // Create a color lookup helper
    let color_lookup = ColorLookup::new(&model);

    // Process each build item
    for (build_idx, item) in model.build.items.iter().enumerate() {
        println!("─────────────────────────────────────");
        println!("Build Item {} (Object ID: {})", build_idx, item.objectid);

        let obj = model
            .resources
            .objects
            .iter()
            .find(|o| o.id == item.objectid);

        if let Some(obj) = obj {
            if let Some(ref name) = obj.name {
                println!("  Name: {}", name);
            }

            if let Some(ref mesh) = obj.mesh {
                println!("  Triangles: {}", mesh.triangles.len());

                // Analyze triangle colors
                let mut color_stats: HashMap<String, usize> = HashMap::new();
                let mut triangles_with_color = 0;

                for triangle in &mesh.triangles {
                    let color = color_lookup.get_triangle_color(obj, triangle);

                    if let Some(color) = color {
                        triangles_with_color += 1;
                        let color_key = format!(
                            "#{:02X}{:02X}{:02X}{:02X}",
                            color.0, color.1, color.2, color.3
                        );
                        *color_stats.entry(color_key).or_insert(0) += 1;
                    }
                }

                println!("  Triangles with color: {}", triangles_with_color);
                println!(
                    "  Triangles without color: {}",
                    mesh.triangles.len() - triangles_with_color
                );

                if !color_stats.is_empty() {
                    println!();
                    println!("  Color Distribution:");

                    let mut sorted_colors: Vec<_> = color_stats.iter().collect();
                    sorted_colors.sort_by(|a, b| b.1.cmp(a.1));

                    for (color, count) in sorted_colors.iter().take(10) {
                        let percentage = (**count as f32 / mesh.triangles.len() as f32) * 100.0;
                        println!("    {} - {} triangles ({:.1}%)", color, count, percentage);
                    }

                    if sorted_colors.len() > 10 {
                        println!("    ... and {} more colors", sorted_colors.len() - 10);
                    }
                }

                // Export first few triangles with colors for rendering
                println!();
                println!("  Rendering Data Sample (first 5 triangles):");
                for (i, triangle) in mesh.triangles.iter().take(5).enumerate() {
                    let v1 = &mesh.vertices[triangle.v1];
                    let v2 = &mesh.vertices[triangle.v2];
                    let v3 = &mesh.vertices[triangle.v3];

                    let color = color_lookup
                        .get_triangle_color(obj, triangle)
                        .unwrap_or((180, 180, 180, 255)); // Default gray

                    println!("    Triangle {}:", i);
                    println!("      Vertices: [{:.2}, {:.2}, {:.2}], [{:.2}, {:.2}, {:.2}], [{:.2}, {:.2}, {:.2}]",
                        v1.x, v1.y, v1.z, v2.x, v2.y, v2.z, v3.x, v3.y, v3.z);
                    println!(
                        "      Color: RGB({}, {}, {}) Alpha: {}",
                        color.0, color.1, color.2, color.3
                    );
                    println!(
                        "      Color (float): ({:.3}, {:.3}, {:.3}, {:.3})",
                        color.0 as f32 / 255.0,
                        color.1 as f32 / 255.0,
                        color.2 as f32 / 255.0,
                        color.3 as f32 / 255.0
                    );
                }
            }
        }
        println!();
    }

    // Summary
    println!("─────────────────────────────────────");
    println!("Rendering Integration Guide:");
    println!();
    println!("For each triangle, you can get its color using:");
    println!("  1. Check triangle.pid for material reference");
    println!("  2. If not set, check object.pid for object-level material");
    println!("  3. Look up the color from materials or color groups");
    println!("  4. Use a default color if no material is specified");
    println!();
    println!("Per-vertex colors:");
    println!("  - Use triangle.p1, p2, p3 for per-vertex color indices");
    println!("  - Interpolate colors across the triangle face");
    println!();
    println!("Color format conversions:");
    println!("  - RGB (0-255): For byte-based rendering");
    println!("  - Float (0.0-1.0): For OpenGL/WebGL/Vulkan");
    println!("  - Hex: For web-based renderers");

    Ok(())
}

/// Helper struct for looking up colors
struct ColorLookup<'a> {
    materials: HashMap<usize, (u8, u8, u8, u8)>,
    color_groups: HashMap<usize, &'a [(u8, u8, u8, u8)]>,
}

impl<'a> ColorLookup<'a> {
    fn new(model: &'a Model) -> Self {
        let mut materials = HashMap::new();
        let mut color_groups = HashMap::new();

        // Index base materials
        for material in &model.resources.materials {
            if let Some(color) = material.color {
                materials.insert(material.id, color);
            }
        }

        // Index color groups
        for cg in &model.resources.color_groups {
            color_groups.insert(cg.id, cg.colors.as_slice());
        }

        Self {
            materials,
            color_groups,
        }
    }

    /// Get the color for a triangle
    fn get_triangle_color(
        &self,
        obj: &lib3mf::Object,
        triangle: &lib3mf::Triangle,
    ) -> Option<(u8, u8, u8, u8)> {
        // First check triangle-level material
        if let Some(pid) = triangle.pid {
            return self.get_color_by_id(pid, triangle.pindex);
        }

        // Then check object-level material
        if let Some(pid) = obj.pid {
            return self.get_color_by_id(pid, obj.pindex);
        }

        None
    }

    /// Get color by material/color group ID and optional index
    fn get_color_by_id(&self, pid: usize, pindex: Option<usize>) -> Option<(u8, u8, u8, u8)> {
        // Check if it's a base material
        if let Some(&color) = self.materials.get(&pid) {
            return Some(color);
        }

        // Check if it's a color group
        if let Some(colors) = self.color_groups.get(&pid) {
            if let Some(idx) = pindex {
                if idx < colors.len() {
                    return Some(colors[idx]);
                }
            }
        }

        None
    }
}

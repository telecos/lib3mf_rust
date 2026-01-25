//! 3MF File Viewer and Analyzer
//!
//! A command-line tool for viewing and analyzing 3MF files using the lib3mf_rust library.
//! This tool demonstrates how to use lib3mf to parse and inspect 3D models
//! stored in the 3MF format.
//!
//! Features:
//! - Parse and display comprehensive 3MF file information
//! - Show model structure, objects, meshes, and materials
//! - Display build items and transformations
//! - Support for all 3MF extensions
//! - Export preview images of the model
//! - Analyze mesh properties (vertex count, bounding box, etc.)

#![forbid(unsafe_code)]

use clap::Parser;
use image::{ImageBuffer, Rgb};
use lib3mf::{Model, Object};
use std::fs::File;
use std::path::PathBuf;

/// Command-line arguments for the 3MF viewer
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the 3MF file to view
    #[arg(value_name = "FILE")]
    file_path: PathBuf,

    /// Show detailed mesh information
    #[arg(short, long)]
    detailed: bool,

    /// Show all vertices and triangles (warning: can be very verbose)
    #[arg(short = 'a', long)]
    show_all: bool,

    /// Export a wireframe preview image
    #[arg(short, long, value_name = "OUTPUT")]
    export_preview: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Load and parse the 3MF file
    println!("═══════════════════════════════════════════════════════════");
    println!("  3MF File Viewer");
    println!("═══════════════════════════════════════════════════════════");
    println!();
    println!("Loading: {}", args.file_path.display());
    println!();

    let file = File::open(&args.file_path)?;
    let model = Model::from_reader(file)?;

    println!("✓ Model loaded successfully!");
    println!();

    // Display model information
    display_model_info(&model);

    // Display metadata
    if !model.metadata.is_empty() {
        display_metadata(&model);
    }

    // Display resources
    display_resources(&model, args.detailed);

    // Display build items
    display_build(&model);

    // Display detailed mesh information if requested
    if args.detailed || args.show_all {
        display_detailed_meshes(&model, args.show_all);
    }

    // Export preview if requested
    if let Some(output_path) = args.export_preview {
        export_wireframe_preview(&model, &output_path)?;
        println!();
        println!("✓ Preview exported to: {}", output_path.display());
    }

    println!();
    println!("═══════════════════════════════════════════════════════════");

    Ok(())
}

/// Display basic model information
fn display_model_info(model: &Model) {
    println!("┌─ Model Information ────────────────────────────────────┐");
    println!("│ Unit:                 {:<34} │", model.unit);
    let xmlns_display = if model.xmlns.len() > 34 {
        format!("{}...", &model.xmlns[..31])
    } else {
        model.xmlns.clone()
    };
    println!("│ XML Namespace:        {:<34} │", xmlns_display);
    if let Some(ref thumbnail) = model.thumbnail {
        let thumb_display = if thumbnail.path.len() > 28 {
            format!("{}...", &thumbnail.path[..25])
        } else {
            thumbnail.path.clone()
        };
        println!("│ Thumbnail:            {:<34} │", thumb_display);
    }
    if !model.required_extensions.is_empty() {
        let ext_names: Vec<&str> = model.required_extensions.iter().map(|e| e.name()).collect();
        let ext_display = ext_names.join(", ");
        let display = if ext_display.len() > 34 {
            format!("{}...", &ext_display[..31])
        } else {
            ext_display
        };
        println!("│ Required Extensions:  {:<34} │", display);
    }
    println!("└────────────────────────────────────────────────────────┘");
    println!();
}

/// Display metadata entries
fn display_metadata(model: &Model) {
    println!("┌─ Metadata ─────────────────────────────────────────────┐");
    for entry in &model.metadata {
        let value_display = if entry.value.len() > 40 {
            format!("{}...", &entry.value[..37])
        } else {
            entry.value.clone()
        };
        println!("│ {:<20} {:<33} │", entry.name, value_display);
    }
    println!("└────────────────────────────────────────────────────────┘");
    println!();
}

/// Display resource information
fn display_resources(model: &Model, detailed: bool) {
    println!("┌─ Resources ────────────────────────────────────────────┐");
    println!("│ Objects:              {:<34} │", model.resources.objects.len());
    println!("│ Base Materials:       {:<34} │", model.resources.materials.len());
    println!("│ Color Groups:         {:<34} │", model.resources.color_groups.len());
    println!("│ Texture 2D Groups:    {:<34} │", model.resources.texture2d_groups.len());
    println!("│ Composite Materials:  {:<34} │", model.resources.composite_materials.len());
    println!("│ Multi-Properties:     {:<34} │", model.resources.multi_properties.len());
    println!("└────────────────────────────────────────────────────────┘");
    println!();

    if detailed {
        display_object_details(&model.resources.objects);
        display_material_details(model);
    }
}

/// Display detailed object information
fn display_object_details(objects: &[Object]) {
    if objects.is_empty() {
        return;
    }

    println!("┌─ Object Details ───────────────────────────────────────┐");
    for obj in objects {
        println!("│ Object ID: {:<43} │", obj.id);
        if let Some(ref name) = obj.name {
            let name_display = if name.len() > 40 {
                format!("{}...", &name[..37])
            } else {
                name.clone()
            };
            println!("│   Name:           {:<36} │", name_display);
        }
        println!("│   Type:           {:<36} │", format!("{:?}", obj.object_type));

        if let Some(ref mesh) = obj.mesh {
            let (min, max) = calculate_bounding_box(mesh);
            let size = (
                max.0 - min.0,
                max.1 - min.1,
                max.2 - min.2,
            );

            println!("│   Vertices:       {:<36} │", mesh.vertices.len());
            println!("│   Triangles:      {:<36} │", mesh.triangles.len());
            println!(
                "│   Bounding Box:   {:<36} │",
                format!(
                    "{:.1} x {:.1} x {:.1}",
                    size.0, size.1, size.2
                )
            );
        }

        if !obj.components.is_empty() {
            println!("│   Components:     {:<36} │", obj.components.len());
        }

        println!("│{:─<56}│", "");
    }
    println!("└────────────────────────────────────────────────────────┘");
    println!();
}

/// Display material information
fn display_material_details(model: &Model) {
    if model.resources.materials.is_empty() && model.resources.color_groups.is_empty() {
        return;
    }

    println!("┌─ Material Details ─────────────────────────────────────┐");

    for mat in &model.resources.materials {
        println!("│ Material ID: {:<41} │", mat.id);
        if let Some(ref name) = mat.name {
            println!("│   Name:           {:<36} │", name);
        }
        if let Some((r, g, b, a)) = mat.color {
            println!(
                "│   Color:          {:<36} │",
                format!("RGBA({}, {}, {}, {})", r, g, b, a)
            );
        }
        println!("│{:─<56}│", "");
    }

    for cg in &model.resources.color_groups {
        println!("│ Color Group ID: {:<38} │", cg.id);
        println!("│   Colors:         {:<36} │", cg.colors.len());
        println!("│{:─<56}│", "");
    }

    println!("└────────────────────────────────────────────────────────┘");
    println!();
}

/// Display build items
fn display_build(model: &Model) {
    println!("┌─ Build Items ──────────────────────────────────────────┐");
    println!("│ Total Items:          {:<34} │", model.build.items.len());

    for (i, item) in model.build.items.iter().enumerate() {
        println!("│{:─<56}│", "");
        println!("│ Item {}:               {:<34} │", i, "");
        println!("│   Object ID:          {:<34} │", item.objectid);

        if let Some(ref uuid) = item.production_uuid {
            let display = if uuid.len() > 34 {
                format!("{}...", &uuid[..31])
            } else {
                uuid.clone()
            };
            println!("│   UUID:               {:<34} │", display);
        }

        if let Some(ref path) = item.production_path {
            let display = if path.len() > 34 {
                format!("{}...", &path[..31])
            } else {
                path.clone()
            };
            println!("│   Production Path:    {:<34} │", display);
        }

        if item.transform.is_some() {
            println!("│   Has Transform:      {:<34} │", "Yes");
        }
    }

    println!("└────────────────────────────────────────────────────────┘");
    println!();
}

/// Display detailed mesh data
fn display_detailed_meshes(model: &Model, show_all: bool) {
    println!("┌─ Detailed Mesh Information ────────────────────────────┐");

    for obj in &model.resources.objects {
        if let Some(ref mesh) = obj.mesh {
            println!("│ Object {}: {} vertices, {} triangles", 
                obj.id, mesh.vertices.len(), mesh.triangles.len());
            println!("│{:─<56}│", "");

            if show_all {
                println!("│ Vertices:");
                for (i, v) in mesh.vertices.iter().enumerate().take(100) {
                    println!(
                        "│   [{:4}] ({:8.2}, {:8.2}, {:8.2})",
                        i, v.x, v.y, v.z
                    );
                }
                if mesh.vertices.len() > 100 {
                    println!("│   ... and {} more vertices", mesh.vertices.len() - 100);
                }

                println!("│");
                println!("│ Triangles:");
                for (i, t) in mesh.triangles.iter().enumerate().take(50) {
                    println!(
                        "│   [{:4}] ({:5}, {:5}, {:5})",
                        i, t.v1, t.v2, t.v3
                    );
                }
                if mesh.triangles.len() > 50 {
                    println!("│   ... and {} more triangles", mesh.triangles.len() - 50);
                }
            }

            println!("│{:─<56}│", "");
        }
    }

    println!("└────────────────────────────────────────────────────────┘");
    println!();
}

/// Calculate bounding box for a mesh
fn calculate_bounding_box(mesh: &lib3mf::Mesh) -> ((f64, f64, f64), (f64, f64, f64)) {
    if mesh.vertices.is_empty() {
        return ((0.0, 0.0, 0.0), (0.0, 0.0, 0.0));
    }

    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut min_z = f64::MAX;
    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;
    let mut max_z = f64::MIN;

    for v in &mesh.vertices {
        min_x = min_x.min(v.x);
        min_y = min_y.min(v.y);
        min_z = min_z.min(v.z);
        max_x = max_x.max(v.x);
        max_y = max_y.max(v.y);
        max_z = max_z.max(v.z);
    }

    ((min_x, min_y, min_z), (max_x, max_y, max_z))
}

/// Export a simple wireframe preview
fn export_wireframe_preview(model: &Model, output_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    const WIDTH: u32 = 800;
    const HEIGHT: u32 = 600;

    let mut img = ImageBuffer::from_pixel(WIDTH, HEIGHT, Rgb([255u8, 255u8, 255u8]));

    // Calculate scene bounds
    let mut all_vertices = Vec::new();
    for item in &model.build.items {
        if let Some(obj) = model.resources.objects.iter().find(|o| o.id == item.objectid) {
            if let Some(ref mesh) = obj.mesh {
                for v in &mesh.vertices {
                    all_vertices.push((v.x, v.y, v.z));
                }
            }
        }
    }

    if all_vertices.is_empty() {
        return Ok(());
    }

    // Calculate bounds
    let min_x = all_vertices.iter().map(|v| v.0).fold(f64::MAX, f64::min);
    let max_x = all_vertices.iter().map(|v| v.0).fold(f64::MIN, f64::max);
    let min_y = all_vertices.iter().map(|v| v.1).fold(f64::MAX, f64::min);
    let max_y = all_vertices.iter().map(|v| v.1).fold(f64::MIN, f64::max);

    let range_x = max_x - min_x;
    let range_y = max_y - min_y;
    let range = range_x.max(range_y).max(0.001);

    // Simple orthographic projection (top view)
    let margin = 50.0;
    let scale = ((WIDTH as f64 - 2.0 * margin).min(HEIGHT as f64 - 2.0 * margin)) / range;

    // Draw triangles
    for item in &model.build.items {
        if let Some(obj) = model.resources.objects.iter().find(|o| o.id == item.objectid) {
            if let Some(ref mesh) = obj.mesh {
                for tri in &mesh.triangles {
                    if tri.v1 < mesh.vertices.len() && tri.v2 < mesh.vertices.len() && tri.v3 < mesh.vertices.len() {
                        let v1 = &mesh.vertices[tri.v1];
                        let v2 = &mesh.vertices[tri.v2];
                        let v3 = &mesh.vertices[tri.v3];

                        // Convert to screen coordinates
                        let p1 = (
                            ((v1.x - min_x) * scale + margin) as i32,
                            ((v1.y - min_y) * scale + margin) as i32,
                        );
                        let p2 = (
                            ((v2.x - min_x) * scale + margin) as i32,
                            ((v2.y - min_y) * scale + margin) as i32,
                        );
                        let p3 = (
                            ((v3.x - min_x) * scale + margin) as i32,
                            ((v3.y - min_y) * scale + margin) as i32,
                        );

                        // Draw triangle edges
                        draw_line(&mut img, p1, p2, Rgb([0u8, 0u8, 0u8]));
                        draw_line(&mut img, p2, p3, Rgb([0u8, 0u8, 0u8]));
                        draw_line(&mut img, p3, p1, Rgb([0u8, 0u8, 0u8]));
                    }
                }
            }
        }
    }

    img.save(output_path)?;
    Ok(())
}

/// Draw a line using Bresenham's algorithm
fn draw_line(img: &mut ImageBuffer<Rgb<u8>, Vec<u8>>, p1: (i32, i32), p2: (i32, i32), color: Rgb<u8>) {
    let (mut x0, mut y0) = p1;
    let (x1, y1) = p2;

    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx - dy;

    loop {
        if x0 >= 0 && x0 < img.width() as i32 && y0 >= 0 && y0 < img.height() as i32 {
            img.put_pixel(x0 as u32, y0 as u32, color);
        }

        if x0 == x1 && y0 == y1 {
            break;
        }

        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            x0 += sx;
        }
        if e2 < dx {
            err += dx;
            y0 += sy;
        }
    }
}

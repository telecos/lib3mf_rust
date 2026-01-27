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
use rfd::FileDialog;
use std::fs::File;
use std::path::PathBuf;

mod ui_viewer;

/// Formatting constant for output field width
const FIELD_WIDTH: usize = 34;

/// Type alias for a colored 3D triangle
type ColoredTriangle = ((f64, f64, f64), (f64, f64, f64), (f64, f64, f64), Rgb<u8>);

/// Constants for isometric projection (30 degree rotation)
const ISO_COS_30: f64 = 0.866_025_403_784_438_6; // cos(30°)
const ISO_SIN_30: f64 = 0.5; // sin(30°)

/// Command-line arguments for the 3MF viewer
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the 3MF file to view (optional - will show file dialog if not provided)
    #[arg(value_name = "FILE")]
    file_path: Option<PathBuf>,

    /// Launch interactive 3D UI viewer
    #[arg(short, long)]
    ui: bool,

    /// Show detailed mesh information
    #[arg(short, long)]
    detailed: bool,

    /// Show all vertices and triangles (warning: can be very verbose)
    #[arg(short = 'a', long)]
    show_all: bool,

    /// Export a wireframe preview image
    #[arg(short, long, value_name = "OUTPUT")]
    export_preview: Option<PathBuf>,

    /// View angle for preview
    #[arg(long, default_value = "isometric", value_parser = ["isometric", "top", "front", "side"])]
    view_angle: String,

    /// Render style for preview
    #[arg(long, default_value = "shaded", value_parser = ["shaded", "wireframe"])]
    render_style: String,
}

/// Open a file dialog to select a 3MF file
fn open_file_dialog() -> Option<PathBuf> {
    FileDialog::new()
        .add_filter("3MF Files", &["3mf"])
        .add_filter("All Files", &["*"])
        .set_title("Open 3MF File")
        .pick_file()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // If UI mode is requested or no file provided, launch the interactive viewer
    if args.ui || args.file_path.is_none() {
        // Get file path from arguments or show file dialog
        let file_path = if let Some(path) = args.file_path {
            Some(path)
        } else if args.ui {
            // If --ui is specified without a file, show file dialog
            open_file_dialog()
        } else {
            // If neither file nor --ui is specified, launch with empty scene
            None
        };
        
        ui_viewer::launch_ui_viewer(file_path)?;
        return Ok(());
    }

    // Non-UI mode with file provided - display model information
    let file_path = args.file_path.as_ref().unwrap(); // Safe because we checked ui || file_path.is_none() above

    // Load and parse the 3MF file
    println!("═══════════════════════════════════════════════════════════");
    println!("  3MF File Viewer");
    println!("═══════════════════════════════════════════════════════════");
    println!();
    println!("Loading: {}", file_path.display());
    println!();

    let file = File::open(file_path)?;
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
        export_preview(&model, &output_path, &args.view_angle, &args.render_style)?;
        println!();
        println!("✓ Preview exported to: {} ({} view, {} style)", 
            output_path.display(), args.view_angle, args.render_style);
    }

    println!();
    println!("═══════════════════════════════════════════════════════════");

    Ok(())
}

/// Display basic model information
fn display_model_info(model: &Model) {
    println!("┌─ Model Information ────────────────────────────────────┐");
    println!("│ Unit:                 {:<FIELD_WIDTH$} │", model.unit);
    let xmlns_display = if model.xmlns.len() > FIELD_WIDTH {
        format!("{}...", &model.xmlns[..(FIELD_WIDTH - 3)])
    } else {
        model.xmlns.clone()
    };
    println!("│ XML Namespace:        {:<FIELD_WIDTH$} │", xmlns_display);
    if let Some(ref thumbnail) = model.thumbnail {
        let thumb_display = if thumbnail.path.len() > FIELD_WIDTH - 6 {
            format!("{}...", &thumbnail.path[..(FIELD_WIDTH - 9)])
        } else {
            thumbnail.path.clone()
        };
        println!("│ Thumbnail:            {:<FIELD_WIDTH$} │", thumb_display);
    }
    if !model.required_extensions.is_empty() {
        let ext_names: Vec<&str> = model.required_extensions.iter().map(|e| e.name()).collect();
        let ext_display = ext_names.join(", ");
        let display = if ext_display.len() > FIELD_WIDTH {
            format!("{}...", &ext_display[..(FIELD_WIDTH - 3)])
        } else {
            ext_display
        };
        println!("│ Required Extensions:  {:<FIELD_WIDTH$} │", display);
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

/// Display resources
fn display_resources(model: &Model, detailed: bool) {
    println!("┌─ Resources ────────────────────────────────────────────┐");
    println!("│ Objects:              {:<FIELD_WIDTH$} │", model.resources.objects.len());
    println!("│ Base Materials:       {:<FIELD_WIDTH$} │", model.resources.materials.len());
    println!("│ Color Groups:         {:<FIELD_WIDTH$} │", model.resources.color_groups.len());
    println!("│ Texture 2D Groups:    {:<FIELD_WIDTH$} │", model.resources.texture2d_groups.len());
    println!("│ Composite Materials:  {:<FIELD_WIDTH$} │", model.resources.composite_materials.len());
    println!("│ Multi-Properties:     {:<FIELD_WIDTH$} │", model.resources.multi_properties.len());
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
    println!("│ Total Items:          {:<FIELD_WIDTH$} │", model.build.items.len());

    for (i, item) in model.build.items.iter().enumerate() {
        println!("│{:─<56}│", "");
        println!("│ Item {}:               {:<FIELD_WIDTH$} │", i, "");
        println!("│   Object ID:          {:<FIELD_WIDTH$} │", item.objectid);

        if let Some(ref uuid) = item.production_uuid {
            let display = if uuid.len() > FIELD_WIDTH {
                format!("{}...", &uuid[..(FIELD_WIDTH - 3)])
            } else {
                uuid.clone()
            };
            println!("│   UUID:               {:<FIELD_WIDTH$} │", display);
        }

        if let Some(ref path) = item.production_path {
            let display = if path.len() > FIELD_WIDTH {
                format!("{}...", &path[..(FIELD_WIDTH - 3)])
            } else {
                path.clone()
            };
            println!("│   Production Path:    {:<FIELD_WIDTH$} │", display);
        }

        if item.transform.is_some() {
            println!("│   Has Transform:      {:<FIELD_WIDTH$} │", "Yes");
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

/// Export a preview image with enhanced 3D rendering
fn export_preview(
    model: &Model, 
    output_path: &PathBuf,
    view_angle: &str,
    render_style: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    const WIDTH: u32 = 1200;
    const HEIGHT: u32 = 900;

    let mut img = ImageBuffer::from_pixel(WIDTH, HEIGHT, Rgb([245u8, 245u8, 250u8]));

    // Collect all triangles from build items
    let mut triangles_3d = Vec::new();
    for item in &model.build.items {
        if let Some(obj) = model.resources.objects.iter().find(|o| o.id == item.objectid) {
            if let Some(ref mesh) = obj.mesh {
                // Get color for this object
                let color = get_object_color(model, obj);
                
                for tri in &mesh.triangles {
                    let vertex_count = mesh.vertices.len();
                    if tri.v1 < vertex_count && tri.v2 < vertex_count && tri.v3 < vertex_count {
                        let v1 = &mesh.vertices[tri.v1];
                        let v2 = &mesh.vertices[tri.v2];
                        let v3 = &mesh.vertices[tri.v3];
                        
                        triangles_3d.push((
                            (v1.x, v1.y, v1.z),
                            (v2.x, v2.y, v2.z),
                            (v3.x, v3.y, v3.z),
                            color,
                        ));
                    }
                }
            }
        }
    }

    if triangles_3d.is_empty() {
        return Ok(());
    }

    // Calculate scene bounds
    let (min_x, max_x, min_y, max_y, min_z, max_z) = calculate_scene_bounds(&triangles_3d);
    let range_x = max_x - min_x;
    let range_y = max_y - min_y;
    let range_z = max_z - min_z;
    let range = range_x.max(range_y).max(range_z).max(0.001);

    // Project and render triangles based on view angle
    let margin = 60.0;
    let scale = ((WIDTH as f64 - 2.0 * margin).min(HEIGHT as f64 - 2.0 * margin)) / range;

    // Calculate center for centering the view
    let center_x = (min_x + max_x) / 2.0;
    let center_y = (min_y + max_y) / 2.0;
    let center_z = (min_z + max_z) / 2.0;

    // Project and sort triangles by depth for proper rendering
    let mut projected_triangles = Vec::new();
    for (v1, v2, v3, color) in &triangles_3d {
        let (p1, d1) = project_vertex(v1, view_angle, center_x, center_y, center_z, scale, WIDTH, HEIGHT);
        let (p2, _) = project_vertex(v2, view_angle, center_x, center_y, center_z, scale, WIDTH, HEIGHT);
        let (p3, _) = project_vertex(v3, view_angle, center_x, center_y, center_z, scale, WIDTH, HEIGHT);
        
        // Calculate normal for shading
        let normal = calculate_normal(*v1, *v2, *v3);
        
        projected_triangles.push((p1, p2, p3, d1, normal, *color));
    }

    // Sort by depth (back to front for proper rendering)
    projected_triangles.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap_or(std::cmp::Ordering::Equal));

    // Render based on style
    match render_style {
        "shaded" => {
            for (p1, p2, p3, _, normal, base_color) in &projected_triangles {
                // Apply simple lighting based on normal
                let light_dir = (0.5, 0.5, 0.7); // Light from top-right-front
                let light_intensity = (normal.0 * light_dir.0 + normal.1 * light_dir.1 + normal.2 * light_dir.2)
                    .clamp(0.2, 1.0);
                
                let shaded_color = Rgb([
                    (base_color.0[0] as f64 * light_intensity) as u8,
                    (base_color.0[1] as f64 * light_intensity) as u8,
                    (base_color.0[2] as f64 * light_intensity) as u8,
                ]);
                
                // Fill triangle
                fill_triangle(&mut img, *p1, *p2, *p3, shaded_color);
                
                // Draw edges in darker color for definition
                let edge_color = Rgb([
                    (base_color.0[0] as f64 * 0.3) as u8,
                    (base_color.0[1] as f64 * 0.3) as u8,
                    (base_color.0[2] as f64 * 0.3) as u8,
                ]);
                draw_line(&mut img, *p1, *p2, edge_color);
                draw_line(&mut img, *p2, *p3, edge_color);
                draw_line(&mut img, *p3, *p1, edge_color);
            }
        }
        "wireframe" => {
            // Wireframe mode
            for (p1, p2, p3, _, _, color) in &projected_triangles {
                draw_line(&mut img, *p1, *p2, *color);
                draw_line(&mut img, *p2, *p3, *color);
                draw_line(&mut img, *p3, *p1, *color);
            }
        }
        _ => {
            // Default to wireframe for any unknown style
            for (p1, p2, p3, _, _, color) in &projected_triangles {
                draw_line(&mut img, *p1, *p2, *color);
                draw_line(&mut img, *p2, *p3, *color);
                draw_line(&mut img, *p3, *p1, *color);
            }
        }
    }

    img.save(output_path)?;
    Ok(())
}

/// Get color for an object (from materials or default)
fn get_object_color(model: &Model, obj: &Object) -> Rgb<u8> {
    // Check if object has a default material
    if let Some(pid) = obj.pid {
        // Try to find in base materials
        if let Some(mat) = model.resources.materials.iter().find(|m| m.id == pid) {
            if let Some((r, g, b, _)) = mat.color {
                return Rgb([r, g, b]);
            }
        }
        // Try to find in color groups (use first color)
        if let Some(cg) = model.resources.color_groups.iter().find(|c| c.id == pid) {
            if !cg.colors.is_empty() {
                let (r, g, b, _) = cg.colors[0];
                return Rgb([r, g, b]);
            }
        }
    }
    
    // Default color: nice blue-gray
    Rgb([100u8, 150u8, 200u8])
}

/// Calculate scene bounds for all triangles
fn calculate_scene_bounds(triangles: &[ColoredTriangle]) 
    -> (f64, f64, f64, f64, f64, f64) {
    let mut min_x = f64::MAX;
    let mut max_x = f64::MIN;
    let mut min_y = f64::MAX;
    let mut max_y = f64::MIN;
    let mut min_z = f64::MAX;
    let mut max_z = f64::MIN;

    for (v1, v2, v3, _) in triangles {
        for v in &[v1, v2, v3] {
            min_x = min_x.min(v.0);
            max_x = max_x.max(v.0);
            min_y = min_y.min(v.1);
            max_y = max_y.max(v.1);
            min_z = min_z.min(v.2);
            max_z = max_z.max(v.2);
        }
    }

    (min_x, max_x, min_y, max_y, min_z, max_z)
}

/// Calculate isometric projection coordinates
fn calculate_isometric(x: f64, y: f64, z: f64) -> (f64, f64, f64) {
    let iso_x = x * ISO_COS_30 - y * ISO_COS_30;
    let iso_y = x * ISO_SIN_30 + y * ISO_SIN_30 - z;
    let iso_depth = x * ISO_SIN_30 + y * ISO_SIN_30 + z;
    (iso_x, iso_y, iso_depth)
}

/// Project a 3D vertex to 2D screen coordinates based on view angle
#[allow(clippy::too_many_arguments)]
fn project_vertex(
    v: &(f64, f64, f64),
    view_angle: &str,
    center_x: f64,
    center_y: f64,
    center_z: f64,
    scale: f64,
    width: u32,
    height: u32,
) -> ((i32, i32), f64) {
    // Translate to center
    let x = v.0 - center_x;
    let y = v.1 - center_y;
    let z = v.2 - center_z;

    let (screen_x, screen_y, depth) = match view_angle {
        "top" => {
            // Top view (looking down Z axis)
            (x, y, z)
        }
        "front" => {
            // Front view (looking down Y axis)
            (x, z, -y)
        }
        "side" => {
            // Side view (looking down X axis)
            (y, z, x)
        }
        "isometric" => calculate_isometric(x, y, z),
        _ => {
            // Default to isometric view for unknown angles
            calculate_isometric(x, y, z)
        }
    };

    let screen_x_final = (screen_x * scale + width as f64 / 2.0) as i32;
    let screen_y_final = (height as f64 / 2.0 - screen_y * scale) as i32; // Flip Y for screen coords

    ((screen_x_final, screen_y_final), depth)
}

/// Calculate surface normal for lighting
fn calculate_normal(v1: (f64, f64, f64), v2: (f64, f64, f64), v3: (f64, f64, f64)) -> (f64, f64, f64) {
    // Two edge vectors
    let edge1 = (v2.0 - v1.0, v2.1 - v1.1, v2.2 - v1.2);
    let edge2 = (v3.0 - v1.0, v3.1 - v1.1, v3.2 - v1.2);
    
    // Cross product
    let nx = edge1.1 * edge2.2 - edge1.2 * edge2.1;
    let ny = edge1.2 * edge2.0 - edge1.0 * edge2.2;
    let nz = edge1.0 * edge2.1 - edge1.1 * edge2.0;
    
    // Normalize
    let length = (nx * nx + ny * ny + nz * nz).sqrt().max(0.001);
    (nx / length, ny / length, nz / length)
}

/// Fill a triangle with a solid color
fn fill_triangle(
    img: &mut ImageBuffer<Rgb<u8>, Vec<u8>>,
    p1: (i32, i32),
    p2: (i32, i32),
    p3: (i32, i32),
    color: Rgb<u8>,
) {
    // Sort vertices by y coordinate
    let mut pts = [p1, p2, p3];
    pts.sort_by_key(|p| p.1);
    let (p1, p2, p3) = (pts[0], pts[1], pts[2]);

    // Handle degenerate triangles
    if p1.1 == p3.1 {
        return;
    }

    // Scan line fill algorithm
    let total_height = p3.1 - p1.1;
    
    for y in p1.1..=p3.1 {
        if y < 0 || y >= img.height() as i32 {
            continue;
        }

        let second_half = y > p2.1 || p2.1 == p1.1;
        let segment_height = if second_half { p3.1 - p2.1 } else { p2.1 - p1.1 };
        
        if segment_height == 0 {
            continue;
        }

        let alpha = (y - p1.1) as f64 / total_height as f64;
        let beta = if second_half {
            (y - p2.1) as f64 / segment_height as f64
        } else {
            (y - p1.1) as f64 / segment_height as f64
        };

        let mut x1 = (p1.0 as f64 + (p3.0 - p1.0) as f64 * alpha) as i32;
        let mut x2 = if second_half {
            (p2.0 as f64 + (p3.0 - p2.0) as f64 * beta) as i32
        } else {
            (p1.0 as f64 + (p2.0 - p1.0) as f64 * beta) as i32
        };

        if x1 > x2 {
            std::mem::swap(&mut x1, &mut x2);
        }

        for x in x1..=x2 {
            if x >= 0 && x < img.width() as i32 {
                img.put_pixel(x as u32, y as u32, color);
            }
        }
    }
}

/// Export a simple wireframe preview (legacy function)
#[allow(dead_code)]
fn export_wireframe_preview(model: &Model, output_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    export_preview(model, output_path, "isometric", "shaded")
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
